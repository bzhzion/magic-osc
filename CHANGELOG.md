# Changelog

Toutes les évolutions notables de magic-osc sont documentées ici.

Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), versionnage
[SemVer](https://semver.org/lang/fr/). La section `[Unreleased]` accumule au fil de l'eau et
est renommée en numéro de version au moment de poser le tag.

## [Unreleased]

### Diagnostic (console ETC Eos)
- Logs sur la réception du feedback OSC (adresse + arguments de chaque message reçu,
  erreurs de décodage, arrêt du thread d'écoute) pour comprendre pourquoi encodeurs et
  nom des subs restent muets alors que le réseau confirme (tcpdump) que les paquets Eos
  arrivent sur le port d'écoute.

### Ajouté (console ETC Eos)
- **6 boutons de page d'encodeur** au-dessus des encodeurs : Intens, Focus, Color,
  Shutter, Image, Form. Ce sont les boutons qui CHANGENT quels paramètres sont assignés
  aux 4 encodeurs physiques, pas des filtres d'affichage CIA (erreur de la version
  précédente, corrigée ci-dessous). Adresse `/eos/key/<nom>` avec `<nom>` =
  `encoder_category_intensity`, `encoder_category_focus`, `encoder_category_color`,
  `encoder_category_shutter`, `encoder_category_image`, `encoder_category_form` (noms
  confirmés par le document officiel ETC « Eos OSC Keys »), argument flottant 1.0 au
  press / 0.0 au release.
- **Bouton carré de sélection** sous chaque submaster, en plus du Bump existant :
  reproduit le bouton physique sous chaque fader de la console, envoie `/eos/sub` +
  numéro pour sélectionner le sub sur la ligne de commande Eos.
- **Nom du submaster affiché sous chaque fader**, demandé à la console via
  `/eos/get/sub/<N>` à la connexion (requêtes étalées sur 30 ms pour ne pas arroser Eos de
  20 messages simultanés). La réponse `/eos/out/get/sub/<N>/list/<index>/<count>` est
  documentée dans l'Appendix « Advanced OSC » du Show Control User Guide — le label est
  le 3ᵉ argument (index 2) de la liste.

### Corrigé (console ETC Eos)
- **Submasters** : les faders n'avaient aucun effet parce qu'on utilisait
  `/eos/fader/1/N`, qui exige qu'un « OSC Fader Bank » soit configuré côté console
  (Setup > Show Control > OSC). Remplacé par `/eos/sub/N` qui pilote directement le
  submaster N sans aucune configuration préalable. Idem pour le bump : `/eos/sub/N/fire`.
- **STOP/BACK** faisait apparaître « cue1/Stop Back » côté console : l'adresse
  `/eos/key/stop_back` était invalide. La touche correcte est `/eos/key/stop` tout court.
- **6 boutons d'encodeur mal nommés** : la première version utilisait les clés
  `intensity`/`focus`/`color`/`shutter`/`image`/`form`, qui correspondent en réalité aux
  filtres d'affichage CIA (boutons physiques différents sur la console), pas à la
  sélection de page d'encodeur. Remplacées par les clés `encoder_category_*` ci-dessus.
  Suppression des boutons Group/Assert/Load, qui n'étaient pas les bons boutons
  physiques recherchés — remplacés par le bouton carré de sélection unique.
- **Encodeurs et nom des subs, vraie cause (architecture réseau)** : malgré plusieurs correctifs
  précédents, aucun feedback n'arrivait jamais, y compris console bien réglée. Cause réelle,
  confirmée par le document officiel ETC (« Eos Family Show Control User Guide » Rev C,
  chapitre « Configuring OSC ») : **Eos n'envoie jamais son feedback UDP vers le port depuis
  lequel on vient de lui parler** (pas de comportement « reply-to-sender »), il l'envoie vers
  une destination **fixe** configurée côté console (*Setup > Show > ShowControl* : « OSC TX IP
  Address » / « OSC TX Port Number »). magic-osc clonait la socket d'envoi en supposant le
  contraire — il écoutait donc un port qu'Eos ne contactait jamais. Corrigé : magic-osc ouvre
  maintenant une socket d'écoute dédiée sur un port fixe (**8001**, conforme à la paire
  8000/8001 recommandée par ETC), affiché dans la barre de connexion pour que l'opérateur le
  reporte dans Setup > Show > ShowControl comme OSC TX Port Number (avec l'IP de la machine qui
  lance magic-osc comme OSC TX IP Address).
- **Encodeurs, mauvaise méthode OSC en entrée** : l'envoi utilisait `/eos/active/switch/<1..4>`,
  qui est la méthode « Switch » (paramètres à bascule), pas « Wheel » (encodeurs rotatifs
  continus). Remplacé par `/eos/active/wheel/<1..4>`, qui correspond à la méthode utilisée par
  les vrais encodeurs physiques d'une Ion/Ion XE et à l'adresse de feedback déjà en place.
- Le format réel du feedback `/eos/out/active/wheel/<1..4>` (confirmé par capture réseau
  sur Eos Nomad, pas seulement par la doc) est **trois arguments**, tag OSC `,sif` (string,
  int, float) — le Show Control User Guide n'en annonçait que deux. Le label contient déjà
  la valeur sous forme texte (`"Intens  [69]"`), l'int est un champ non identifié, et la
  vraie valeur numérique est le **float en position 2**, pas l'int en position 1 comme le
  lisait la version précédente. magic-osc envoie maintenant
     `/eos/reset` juste après connexion (force Eos à republier tout de suite son état
     courant au lieu d'attendre un changement) et lit les deux arguments séparément.

### Ajouté
- **Icônes de l'application** : jeu placeholder (rond rouge spectacle, lettres `mo` blanches)
  aux tailles exigées par Tauri (32×32, 128×128, 128×128@2x, icon.png 512px, icon.ico
  multi-page pour Windows). À remplacer par un jeu vectoriel soigné quand l'identité
  visuelle sera posée. Débloque le build : sans icône, `tauri::generate_context!` échoue.
- **Console graphique ETC Eos (Ion XE)** : 20 submasters avec leur bump button, 2 Master
  Playback faders, boutons GO et STOP/BACK, 4 encoders (Pan/Tilt/Zoom/Iris), Level Wheel.
  L'utilisateur saisit l'IP et le port de la console dans une barre en haut, et les gestes
  à la souris ou au tactile partent en OSC directement. Adresses OSC conformes au « Eos
  Family Show Control User Guide » (chapitre OSC) : `/eos/fader/1/N` pour les submasters
  avec `/fire` pour le bump, `/eos/fader/10/{1,2}` pour les deux Master Playbacks,
  `/eos/key/go_0` et `/eos/key/stop_back` pour Go/Stop (press 1 / release 0),
  `/eos/wheel/pan|tilt|zoom|iris` pour les 4 encoders avec delta flottant,
  `/eos/wheel/level` pour la Level Wheel.
