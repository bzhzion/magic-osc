// Interface console ETC Eos (Ion XE) : 20 submasters, 2 Master Playback,
// 4 encoders, 1 level wheel, Go et Stop/Back.
//
// Chaque geste utilisateur appelle une commande Tauri qui compose la bonne
// adresse OSC ETC et envoie. L'UI ne connait pas les adresses OSC : c'est
// src-tauri/src/main.rs qui les porte, un seul endroit a corriger si Eos
// bouge son protocole.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Diffuse par le backend (osc.rs) a chaque feedback /eos/out/active/wheel/N
// recu depuis la console. IMPORTANT : Eos n'expose PAS "ce qui est sur les 4
// encodeurs physiques a l'instant T" -- /eos/active/wheel/<index> est la
// liste COMPLETE et FIXE des parametres actifs du projecteur selectionne
// (jusqu'a 40+ pour un robot complexe), dans l'ordre du patch, et ne se
// reordonne JAMAIS quand on change de categorie CIA (confirme par capture :
// wheel/1-4 restent toujours identiques quel que soit le bouton presse).
// magic-osc doit donc reconstituer lui-meme la pagination par categorie, en
// filtrant cette liste complete par mot-cle de nom de parametre -- voir
// `recalculerPageEncodeurs` plus bas.
interface FeedbackEncodeur {
  index: number;      // index OSC reel (1..40+), PAS le numero d'encodeur physique
  libelle: string;    // vide = parametre inexistant a cet index pour ce patch
  valeur: number | null;
}

// Tous les index OSC reels recus pour le projecteur actuellement selectionne.
const tousLesWheels = new Map<number, FeedbackEncodeur>();
// Callback d'affichage par EMPLACEMENT physique (1..4), pas par index OSC reel.
const feedbackEncodeurs = new Map<number, (f: FeedbackEncodeur) => void>();
// emplacement physique i (0-based) -> index OSC reel actuellement affiche/pilote.
// -1 = aucun parametre de la categorie choisie a cet emplacement.
const assignationEncodeurs: number[] = [1, 2, 3, 4];
// Categorie choisie par le dernier bouton presse (voir monterCategories), ou
// null tant qu'aucun bouton n'a ete presse (affiche alors les 4 premiers
// parametres du patch, comportement par defaut).
let categorieActive: string | null = null;
// Sous-page (bloc de 4 parametres) dans la categorie active. Rappuyer sur le
// MEME bouton de categorie avance d'une sous-page (avec retour au debut),
// exactement comme le bandeau physique d'une Ion XE : ex. Shutter montre
// d'abord Frame Angle/Thrust C puis A, puis au rappui D puis B, puis au
// rappui suivant Frame Assembly seul.
let pageCourante = 0;

// Mots-cles (sous-chaine, insensible a la casse) identifiant les noms de
// parametre ETC habituels de chaque categorie CIA. Heuristique necessaire
// car Eos n'envoie aucune metadonnee de categorie par parametre -- peut
// manquer un parametre au nom atypique selon le fabricant du projecteur.
// Repartition confirmee par la doc officielle ETC (support.etcconnect.com,
// classe d'attribut "Beam" divisee en 3 sous-categories) :
// - Shutter = dispositifs de CADRAGE (framing) : Frame Angle/Thrust A-D.
// - Form = qualite/taille du faisceau : edge, zoom, iris, frost.
// - Image = tout ce qui "tombe dans le faisceau" : gobos, effets, prismes.
const MOTS_CLES_CATEGORIE: Record<string, string[]> = {
  intensity: ["intens"],
  focus: ["pan", "tilt", "focus", "position"],
  color: ["red", "green", "blue", "hue", "saturation", "color", "tint", "cri"],
  form: ["edge", "zoom", "iris", "frost", "diffusion"],
  image: ["gobo", "animation", "effect", "prism", "beam fx"],
  shutter: ["frame", "shutter strobe"],
};

// Recalcule, pour les 4 emplacements physiques, quel index OSC reel afficher
// et piloter : soit les 4 premiers parametres (categorie par defaut), soit
// la sous-page courante des parametres de la categorie choisie (filtrage
// par mot-cle, puis decoupage en blocs de 4 selon `pageCourante`).
function recalculerPageEncodeurs() {
  let indices: number[];
  if (categorieActive === null) {
    indices = [1, 2, 3, 4];
  } else {
    const mots = MOTS_CLES_CATEGORIE[categorieActive] ?? [];
    const correspondants = Array.from(tousLesWheels.entries())
      .filter(([, f]) => {
        const nom = f.libelle.split("  [")[0].trim().toLowerCase();
        return nom.length > 0 && mots.some((m) => nom.includes(m));
      })
      .sort((a, b) => a[0] - b[0])
      .map(([idx]) => idx);
    const nbPages = Math.max(1, Math.ceil(correspondants.length / 4));
    pageCourante = ((pageCourante % nbPages) + nbPages) % nbPages;
    indices = correspondants.slice(pageCourante * 4, pageCourante * 4 + 4);
  }
  for (let emplacement = 0; emplacement < 4; emplacement++) {
    const idx = indices[emplacement];
    assignationEncodeurs[emplacement] = idx ?? -1;
    const cb = feedbackEncodeurs.get(emplacement + 1);
    if (!cb) continue;
    const f = idx !== undefined ? tousLesWheels.get(idx) : undefined;
    cb(f ?? { index: -1, libelle: "", valeur: null });
  }
}

listen<FeedbackEncodeur>("eos-wheel-feedback", (event) => {
  tousLesWheels.set(event.payload.index, event.payload);
  recalculerPageEncodeurs();
});

// Reponse a eos_sub_demander_nom : diffuse par le backend quand Eos repond
// a /eos/get/sub/<n>. Permet d'afficher sous chaque fader le VRAI nom
// donne au submaster dans le show, sans que l'utilisateur ait a le
// ressaisir a la main.
interface FeedbackNomSub {
  n: number;
  libelle: string;
}

const feedbackNomsSub = new Map<number, (f: FeedbackNomSub) => void>();

listen<FeedbackNomSub>("eos-sub-name-feedback", (event) => {
  const cb = feedbackNomsSub.get(event.payload.n);
  if (cb) cb(event.payload);
});

// ── Barre de reglages (hote + port + bouton « connecter ») ──────────────
function monterBarre(racine: HTMLElement) {
  const barre = document.createElement("div");
  barre.className = "barre";
  barre.innerHTML = `
    <label for="hote">Console Eos :</label>
    <input id="hote" type="text" placeholder="192.168.1.10" style="width: 160px;" />
    <label for="port">port</label>
    <input id="port" type="number" value="8000" style="width: 72px;" />
    <button id="connecter">Connecter</button>
    <span id="port-ecoute" class="statut"></span>
    <span id="statut" class="statut">Non connecte</span>
  `;
  racine.appendChild(barre);

  const hote = barre.querySelector<HTMLInputElement>("#hote")!;
  const port = barre.querySelector<HTMLInputElement>("#port")!;
  const bouton = barre.querySelector<HTMLButtonElement>("#connecter")!;
  const statut = barre.querySelector<HTMLSpanElement>("#statut")!;
  const portEcouteAff = barre.querySelector<HTMLSpanElement>("#port-ecoute")!;

  // Le feedback Eos (encodeurs, nom des subs) part vers un port FIXE
  // configure cote console (OSC TX Port Number), pas vers notre port
  // d'envoi : on affiche ce port pour que l'operateur le recopie dans
  // Setup > Show > ShowControl sur la console.
  invoke<{ port_ecoute: number }>("info_destination_osc").then((info) => {
    portEcouteAff.textContent = `Port d'ecoute magic-osc (a mettre en "OSC TX Port Number" cote Eos) : ${info.port_ecoute}`;
  }).catch(() => {});

  // Restaure les dernieres valeurs saisies (persistence legere cote JS).
  const sauvegarde = localStorage.getItem("magic-osc.destination");
  if (sauvegarde) {
    try {
      const o = JSON.parse(sauvegarde);
      if (o.hote) hote.value = o.hote;
      if (o.port) port.value = String(o.port);
    } catch { /* ignore */ }
  }

  bouton.addEventListener("click", async () => {
    const h = hote.value.trim();
    const p = Number(port.value);
    if (!h || !p) {
      statut.className = "statut ko";
      statut.textContent = "Hote et port requis";
      return;
    }
    try {
      await invoke("regler_destination_osc", { hote: h, port: p });
      statut.className = "statut ok";
      statut.textContent = `Connecte a ${h}:${p}`;
      localStorage.setItem("magic-osc.destination", JSON.stringify({ hote: h, port: p }));
      demanderNomsSubmasters();
    } catch (e) {
      statut.className = "statut ko";
      statut.textContent = `Echec : ${e}`;
    }
  });
}

// Demande le nom des 20 submasters a la console, etalee sur quelques
// centaines de ms pour ne pas arroser Eos de 20 requetes simultanees.
function demanderNomsSubmasters() {
  for (let i = 1; i <= 20; i++) {
    setTimeout(() => {
      invoke("eos_sub_demander_nom", { n: i }).catch(() => {});
    }, i * 30);
  }
}

// ── Encoder rotatif (dial drag vertical) ────────────────────────────────
// L'encodeur affiche EN DIRECT le parametre que la console lui a assigne
// (feedback /eos/out/active/wheel/N retransmis par le backend), au lieu
// qu'on le choisisse a la main : c'est exactement ce que fait le bandeau
// d'encodeurs physiques d'une Ion XE, qui se reaffecte selon la page
// ouverte sur l'ecran de la console. Un encodeur sans parametre assigne
// (libelle vide) est grise et ne pousse aucune valeur.
function monterEncoder(parent: HTMLElement, index: number) {
  const bloc = document.createElement("div");
  bloc.className = "encoder";
  bloc.innerHTML = `
    <div class="nom">E${index}</div>
    <div class="dial"></div>
    <div class="parametre">(en attente)</div>
    <div class="parametre-valeur"></div>
  `;
  parent.appendChild(bloc);

  const dial = bloc.querySelector<HTMLDivElement>(".dial")!;
  const paramAff = bloc.querySelector<HTMLDivElement>(".parametre")!;
  const valeurAff = bloc.querySelector<HTMLDivElement>(".parametre-valeur")!;

  let assigne = false;

  feedbackEncodeurs.set(index, (f) => {
    assigne = f.libelle.trim().length > 0;
    paramAff.textContent = assigne ? f.libelle : "(aucun)";
    valeurAff.textContent = assigne && f.valeur !== null ? String(f.valeur) : "";
    dial.classList.toggle("inassigne", !assigne);
  });

  let actif = false;
  let dernier = 0;
  let angle = 0;

  const debut = (ev: PointerEvent) => {
    if (!assigne) return;
    actif = true;
    dernier = ev.clientY;
    dial.classList.add("actif");
    (ev.target as Element).setPointerCapture(ev.pointerId);
    ev.preventDefault();
  };
  const mouvement = (ev: PointerEvent) => {
    if (!actif) return;
    const dy = dernier - ev.clientY;
    dernier = ev.clientY;
    angle += dy;
    dial.style.transform = `rotate(${angle}deg)`;
    const delta = dy / 20;
    // L'index OSC reel pilote par cet emplacement physique peut changer a
    // tout moment (categorie choisie), on le relit a chaque mouvement.
    const reel = assignationEncodeurs[index - 1];
    if (reel > 0) invoke("eos_wheel_switch", { n: reel, delta }).catch(() => {});
  };
  const fin = () => { actif = false; dial.classList.remove("actif"); };

  dial.addEventListener("pointerdown", debut);
  dial.addEventListener("pointermove", mouvement);
  dial.addEventListener("pointerup", fin);
  dial.addEventListener("pointercancel", fin);
}

// ── Level wheel (molette horizontale) ──────────────────────────────────
function monterLevelWheel(parent: HTMLElement) {
  const bloc = document.createElement("div");
  bloc.className = "wheel";
  bloc.innerHTML = `
    <div class="nom">Level</div>
    <div class="molette">
      <div class="graduations">${"<span></span>".repeat(20)}</div>
      <div class="indicateur"></div>
    </div>
  `;
  parent.appendChild(bloc);

  const molette = bloc.querySelector<HTMLDivElement>(".molette")!;
  let actif = false;
  let dernier = 0;

  const debut = (ev: PointerEvent) => {
    actif = true;
    dernier = ev.clientX;
    (ev.target as Element).setPointerCapture(ev.pointerId);
    ev.preventDefault();
  };
  const mouvement = (ev: PointerEvent) => {
    if (!actif) return;
    const dx = ev.clientX - dernier;
    dernier = ev.clientX;
    const delta = dx / 10;
    invoke("eos_wheel_level", { delta }).catch(() => {});
  };
  const fin = () => { actif = false; };

  molette.addEventListener("pointerdown", debut);
  molette.addEventListener("pointermove", mouvement);
  molette.addEventListener("pointerup", fin);
  molette.addEventListener("pointercancel", fin);
}

// ── Fader (piste verticale + poignee) ──────────────────────────────────
// Facteur commun entre submaster et master, parametre par un rappel OSC.
function monterFader(opts: {
  parent: HTMLElement;
  numero: number;
  valeurInitiale: number;
  classe: string;
  surValeur: (v: number) => void;
  avecBump?: { surPress: (p: boolean) => void };
  // Bouton carre present sous chaque fader sur la console physique :
  // selectionne le sub sur la ligne de commande Eos (/eos/sub + N). Nom du
  // sub recupere via eos_sub_demander_nom et affiche au-dessus de la piste.
  avecSelect?: {
    numeroSub: number;
    surSelect: () => void;
  };
}) {
  const bloc = document.createElement("div");
  bloc.className = opts.classe;
  bloc.innerHTML = `
    <div class="num">${opts.numero}</div>
    ${opts.avecSelect ? `<div class="nom-sub">—</div>` : ""}
    <div class="piste"><div class="poignee"></div></div>
    <div class="valeur">0 %</div>
    ${opts.avecBump ? `<button class="bump">BUMP</button>` : ""}
    ${opts.avecSelect ? `<button class="select-carre" title="Selectionner"></button>` : ""}
  `;
  opts.parent.appendChild(bloc);

  const piste = bloc.querySelector<HTMLDivElement>(".piste")!;
  const poignee = bloc.querySelector<HTMLDivElement>(".poignee")!;
  const valeurAff = bloc.querySelector<HTMLDivElement>(".valeur")!;

  if (opts.avecSelect) {
    const nomAff = bloc.querySelector<HTMLDivElement>(".nom-sub")!;
    feedbackNomsSub.set(opts.avecSelect.numeroSub, (f) => {
      nomAff.textContent = f.libelle || "—";
    });
    const sel = bloc.querySelector<HTMLButtonElement>(".select-carre")!;
    sel.addEventListener("click", () => {
      sel.classList.add("actif");
      setTimeout(() => sel.classList.remove("actif"), 150);
      opts.avecSelect!.surSelect();
    });
  }

  let valeur = opts.valeurInitiale; // 0..1

  const placerPoignee = () => {
    // 0 en bas, 1 en haut. La poignee est centree sur sa position.
    const h = piste.clientHeight;
    const y = (1 - valeur) * h - poignee.offsetHeight / 2;
    poignee.style.top = `${y}px`;
    valeurAff.textContent = `${Math.round(valeur * 100)} %`;
  };

  const depuisY = (clientY: number) => {
    const rect = piste.getBoundingClientRect();
    const y = Math.max(0, Math.min(rect.height, clientY - rect.top));
    return 1 - y / rect.height;
  };

  let actif = false;

  const debut = (ev: PointerEvent) => {
    actif = true;
    valeur = depuisY(ev.clientY);
    placerPoignee();
    opts.surValeur(valeur);
    (ev.target as Element).setPointerCapture(ev.pointerId);
    ev.preventDefault();
  };
  const mouvement = (ev: PointerEvent) => {
    if (!actif) return;
    valeur = depuisY(ev.clientY);
    placerPoignee();
    opts.surValeur(valeur);
  };
  const fin = () => { actif = false; };

  piste.addEventListener("pointerdown", debut);
  piste.addEventListener("pointermove", mouvement);
  piste.addEventListener("pointerup", fin);
  piste.addEventListener("pointercancel", fin);

  // Positionnement initial (apres layout).
  requestAnimationFrame(placerPoignee);

  if (opts.avecBump) {
    const b = bloc.querySelector<HTMLButtonElement>(".bump")!;
    const press = () => { b.classList.add("actif"); opts.avecBump!.surPress(true); };
    const release = () => { b.classList.remove("actif"); opts.avecBump!.surPress(false); };
    b.addEventListener("pointerdown", (e) => { e.preventDefault(); press(); });
    b.addEventListener("pointerup", release);
    b.addEventListener("pointerleave", release);
    b.addEventListener("pointercancel", release);
  }

  return { placerPoignee };
}

// ── Boutons de page d'encodeur (Intens, Focus, Color, Shutter, Image, Form) ──
// Boutons physiques d'une Ion/Ion XE : changent la categorie CIA affichee sur
// la console reelle (confirme par Julien, voir recalculerPageEncodeurs plus
// haut pour le cote magic-osc). Noms OSC confirmes dans "Eos OSC Keys" :
// prefixe encoder_category_<nom>. `categorie` sert UNIQUEMENT cote magic-osc
// a filtrer la liste complete des parametres actifs (MOTS_CLES_CATEGORIE).
const CATEGORIES_EOS: { nom: string; libelle: string; categorie: string }[] = [
  { nom: "encoder_category_intensity", libelle: "Intens", categorie: "intensity" },
  { nom: "encoder_category_focus", libelle: "Focus", categorie: "focus" },
  { nom: "encoder_category_color", libelle: "Color", categorie: "color" },
  { nom: "encoder_category_shutter", libelle: "Shutter", categorie: "shutter" },
  { nom: "encoder_category_image", libelle: "Image", categorie: "image" },
  { nom: "encoder_category_form", libelle: "Form", categorie: "form" },
];

function monterCategories(parent: HTMLElement) {
  const bloc = document.createElement("div");
  bloc.className = "categories";
  parent.appendChild(bloc);
  const boutons: HTMLButtonElement[] = [];
  for (const cat of CATEGORIES_EOS) {
    const b = document.createElement("button");
    b.className = "categorie";
    b.textContent = cat.libelle;
    bloc.appendChild(b);
    boutons.push(b);
    const press = () => {
      invoke("eos_key", { name: cat.nom, press: true }).catch(() => {});
    };
    const release = () => {
      invoke("eos_key", { name: cat.nom, press: false }).catch(() => {});
      // Selection "latchee" cote magic-osc (reste allumee jusqu'au prochain
      // bouton) : pilote la pagination des 4 encodeurs.
      boutons.forEach((autre) => autre.classList.remove("actif"));
      b.classList.add("actif");
      if (categorieActive === cat.categorie) {
        // Meme categorie rappuyee : sous-page suivante (comme la console
        // physique). Le wrap-around est gere dans recalculerPageEncodeurs.
        pageCourante += 1;
      } else {
        categorieActive = cat.categorie;
        pageCourante = 0;
      }
      recalculerPageEncodeurs();
    };
    b.addEventListener("pointerdown", (e) => { e.preventDefault(); press(); });
    b.addEventListener("pointerup", release);
    b.addEventListener("pointerleave", release);
    b.addEventListener("pointercancel", release);
  }
}

// ── Bouton Go / Stop (press/release distincts pour un flash propre) ────
function monterBoutonPersistant(parent: HTMLElement, classe: string, texte: string,
                                 surEtat: (press: boolean) => void) {
  const b = document.createElement("button");
  b.className = classe;
  b.textContent = texte;
  parent.appendChild(b);

  const press = () => { b.classList.add("actif"); surEtat(true); };
  const release = () => { b.classList.remove("actif"); surEtat(false); };
  b.addEventListener("pointerdown", (e) => { e.preventDefault(); press(); });
  b.addEventListener("pointerup", release);
  b.addEventListener("pointerleave", release);
  b.addEventListener("pointercancel", release);
}

// ── Montage global ────────────────────────────────────────────────────
async function demarrer() {
  const racine = document.getElementById("app")!;
  racine.innerHTML = "";

  monterBarre(racine);

  const console_ = document.createElement("div");
  console_.className = "console";
  racine.appendChild(console_);

  // Boutons de page d'encodeur, juste au-dessus des encodeurs (comme sur
  // une vraie Ion/Ion XE).
  monterCategories(console_);

  // Haut : 4 encoders + 1 level wheel
  const haut = document.createElement("div");
  haut.className = "haut";
  console_.appendChild(haut);

  // Encoders : affichent automatiquement ce qu'Eos leur assigne (feedback
  // /eos/out/active/wheel/N), voir monterEncoder plus haut.
  monterEncoder(haut, 1);
  monterEncoder(haut, 2);
  monterEncoder(haut, 3);
  monterEncoder(haut, 4);
  monterLevelWheel(haut);

  // Bas : 20 submasters + Master Playback (A + B + Go + Stop)
  const bas = document.createElement("div");
  bas.className = "bas";
  console_.appendChild(bas);

  const subs = document.createElement("div");
  subs.className = "submasters";
  bas.appendChild(subs);

  for (let i = 1; i <= 20; i++) {
    monterFader({
      parent: subs,
      numero: i,
      valeurInitiale: 0,
      classe: "fader",
      surValeur: (v) => { invoke("eos_fader_submaster", { n: i, valeur: v }).catch(() => {}); },
      avecBump: { surPress: (p) => { invoke("eos_fader_bump", { n: i, press: p }).catch(() => {}); } },
      avecSelect: {
        numeroSub: i,
        surSelect: () => { invoke("eos_sub_select", { n: i }).catch(() => {}); },
      },
    });
  }

  const masters = document.createElement("div");
  masters.className = "masters";
  bas.appendChild(masters);

  monterFader({
    parent: masters,
    numero: 1,
    valeurInitiale: 1,
    classe: "master-fader",
    surValeur: (v) => { invoke("eos_fader_master", { bank: 1, valeur: v }).catch(() => {}); },
  });
  monterFader({
    parent: masters,
    numero: 2,
    valeurInitiale: 1,
    classe: "master-fader",
    surValeur: (v) => { invoke("eos_fader_master", { bank: 2, valeur: v }).catch(() => {}); },
  });

  const gostop = document.createElement("div");
  gostop.className = "gostop";
  masters.appendChild(gostop);

  monterBoutonPersistant(gostop, "go", "GO", (p) =>
    invoke("eos_key_go", { press: p }).catch(() => {})
  );
  monterBoutonPersistant(gostop, "stop", "STOP", (p) =>
    invoke("eos_key_stop", { press: p }).catch(() => {})
  );

  // Au chargement, tente d'afficher la version dans la barre de titre.
  const version = await invoke<string>("version").catch(() => "");
  if (version) document.title = `magic-osc ${version} - console ETC Eos`;
}

demarrer();
