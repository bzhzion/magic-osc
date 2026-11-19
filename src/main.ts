// Interface console ETC Eos (Ion XE) : 20 submasters, 2 Master Playback,
// 4 encoders, 1 level wheel, Go et Stop/Back.
//
// Chaque geste utilisateur appelle une commande Tauri qui compose la bonne
// adresse OSC ETC et envoie. L'UI ne connait pas les adresses OSC : c'est
// src-tauri/src/main.rs qui les porte, un seul endroit a corriger si Eos
// bouge son protocole.

import { invoke } from "@tauri-apps/api/core";

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
    <span id="statut" class="statut">Non connecte</span>
  `;
  racine.appendChild(barre);

  const hote = barre.querySelector<HTMLInputElement>("#hote")!;
  const port = barre.querySelector<HTMLInputElement>("#port")!;
  const bouton = barre.querySelector<HTMLButtonElement>("#connecter")!;
  const statut = barre.querySelector<HTMLSpanElement>("#statut")!;

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
    } catch (e) {
      statut.className = "statut ko";
      statut.textContent = `Echec : ${e}`;
    }
  });
}

// ── Encoder rotatif + menu du parametre pilote ────────────────────────
// ETC Eos n'expose pas les 4 encoders physiques par index : on doit dire
// quel parametre (pan, tilt, intens, zoom...) on veut pousser. L'utilisateur
// choisit pour chaque encoder ; on sauvegarde le choix dans localStorage.
const PARAMETRES_EOS = [
  { val: "", libelle: "(aucun)" },
  { val: "intens", libelle: "Intens" },
  { val: "pan", libelle: "Pan" },
  { val: "tilt", libelle: "Tilt" },
  { val: "zoom", libelle: "Zoom" },
  { val: "iris", libelle: "Iris" },
  { val: "edge", libelle: "Edge" },
  { val: "shutter", libelle: "Shutter" },
  { val: "red", libelle: "Red" },
  { val: "green", libelle: "Green" },
  { val: "blue", libelle: "Blue" },
  { val: "amber", libelle: "Amber" },
  { val: "white", libelle: "White" },
  { val: "cto", libelle: "CTO" },
  { val: "cts", libelle: "CTS" },
  { val: "hue", libelle: "Hue" },
  { val: "saturation", libelle: "Saturation" },
  { val: "gobo_select", libelle: "Gobo Select" },
  { val: "rot1", libelle: "Rot 1" },
  { val: "rot2", libelle: "Rot 2" },
];

function monterEncoder(parent: HTMLElement, index: number) {
  const bloc = document.createElement("div");
  bloc.className = "encoder";
  bloc.innerHTML = `
    <div class="nom">E${index}</div>
    <div class="dial"></div>
    <select></select>
  `;
  parent.appendChild(bloc);

  const dial = bloc.querySelector<HTMLDivElement>(".dial")!;
  const menu = bloc.querySelector<HTMLSelectElement>("select")!;

  PARAMETRES_EOS.forEach(p => {
    const o = document.createElement("option");
    o.value = p.val; o.textContent = p.libelle;
    menu.appendChild(o);
  });

  const cleSauvegarde = `magic-osc.encoder.${index}`;
  const sauv = localStorage.getItem(cleSauvegarde);
  if (sauv !== null) menu.value = sauv;
  menu.addEventListener("change", () => localStorage.setItem(cleSauvegarde, menu.value));

  let actif = false;
  let dernier = 0;
  let angle = 0;

  const debut = (ev: PointerEvent) => {
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
    const param = menu.value;
    if (!param) return;   // encoder non assigne : rien a envoyer
    const delta = dy / 20;
    invoke("eos_wheel_encoder", { param, delta }).catch(() => {});
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
}) {
  const bloc = document.createElement("div");
  bloc.className = opts.classe;
  bloc.innerHTML = `
    <div class="num">${opts.numero}</div>
    <div class="piste"><div class="poignee"></div></div>
    <div class="valeur">0 %</div>
    ${opts.avecBump ? `<button class="bump">BUMP</button>` : ""}
  `;
  opts.parent.appendChild(bloc);

  const piste = bloc.querySelector<HTMLDivElement>(".piste")!;
  const poignee = bloc.querySelector<HTMLDivElement>(".poignee")!;
  const valeurAff = bloc.querySelector<HTMLDivElement>(".valeur")!;

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

  // Haut : 4 encoders + 1 level wheel
  const haut = document.createElement("div");
  haut.className = "haut";
  console_.appendChild(haut);

  // Encoders : chacun choisit son parametre Eos via un menu, voir
  // monterEncoder plus haut. Pas d'assignation par defaut.
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
