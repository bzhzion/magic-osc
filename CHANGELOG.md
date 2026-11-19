# Changelog

Toutes les évolutions notables de magic-osc sont documentées ici.

Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), versionnage
[SemVer](https://semver.org/lang/fr/). La section `[Unreleased]` accumule au fil de l'eau et
est renommée en numéro de version au moment de poser le tag.

## [Unreleased]

### Ajouté (console ETC Eos)
- **6 boutons de catégorie d'attribut** au-dessus des encodeurs : Intens, Focus, Color,
  Shutter, Image, Form. Reproduisent le bandeau physique d'une Ion/Ion XE. Adresse
  `/eos/key/<nom>` avec `<nom>` = `intensity`, `focus`, `color`, `shutter`, `image`, `form`
  (noms confirmés par le document officiel ETC « Eos OSC Keys »), argument flottant
  1.0 au press / 0.0 au release.
- **Boutons Group, Assert, Load** sous chaque submaster, en plus du Bump existant. Il
  n'existe pas d'adresse OSC dédiée par sub pour ces trois actions : magic-osc reproduit
  la séquence console (sélectionner le sub avec `/eos/sub` + numéro, puis presser la
  touche `/eos/key/group`, `/eos/key/assert` ou `/eos/key/load`), exactement comme le
  ferait un régisseur au clavier.
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
- **Encoders, suite et vraie source du problème** : malgré l'adressage par index déjà en place
  (`/eos/active/switch/<1..4>`, confirmé bon), les encodeurs restaient inertes. Deux causes,
  confirmées cette fois par le document officiel ETC (« Eos Family Show Control User Guide »
  Rev C, chapitres « Active Parameters » et « Implicit OSC Output », PDF obtenu via le dépôt
  `jmcker/OSC-Control---ETC-Eos`, pas une implémentation tierce) :
  1. Eos n'envoie son feedback (quel paramètre est sur quel encodeur) QUE si **« Transmit »**
     est coché côté console, dans *Setup > System > Show Control > OSC*. « Receive » seul
     (qui suffit pour GO/STOP/faders, lesquels fonctionnaient déjà) ne suffit pas : Eos ne
     parle jamais de lui-même sans Transmit actif. **Si les encodeurs restent vides après
     cette version, c'est le premier réglage à vérifier côté console.**
  2. Le format du feedback `/eos/out/active/wheel/<1..4>` est **deux arguments séparés**
     (chaîne = nom du paramètre, nombre = valeur courante), pas une seule chaîne
     `"<label>  [<valeur>]"` comme le supposait la version précédente d'après du code tiers
     non officiel (Luminosus Eos Edition, abandonné). magic-osc envoie maintenant
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
