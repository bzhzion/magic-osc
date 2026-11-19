# Changelog

Toutes les évolutions notables de magic-osc sont documentées ici.

Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), versionnage
[SemVer](https://semver.org/lang/fr/). La section `[Unreleased]` accumule au fil de l'eau et
est renommée en numéro de version au moment de poser le tag.

## [Unreleased]

### Corrigé (console ETC Eos)
- **Submasters** : les faders n'avaient aucun effet parce qu'on utilisait
  `/eos/fader/1/N`, qui exige qu'un « OSC Fader Bank » soit configuré côté console
  (Setup > Show Control > OSC). Remplacé par `/eos/sub/N` qui pilote directement le
  submaster N sans aucune configuration préalable. Idem pour le bump : `/eos/sub/N/fire`.
- **STOP/BACK** faisait apparaître « cue1/Stop Back » côté console : l'adresse
  `/eos/key/stop_back` était invalide. La touche correcte est `/eos/key/stop` tout court.
- **Encoders** : `/eos/active_wheel/N` (index numérique) n'existe pas dans l'API OSC Eos —
  testé sur Mac, sans effet. L'API Eos adresse des paramètres **nommés**
  (`/eos/active/wheel/coarse/<param>`, ex. `pan`, `tilt`, `intens`, `zoom`), jamais un
  index d'encodeur physique. Chaque encodeur `E1`-`E4` a maintenant son propre menu
  déroulant pour choisir le paramètre qu'il pilote (Intens, Pan, Tilt, Zoom, Iris, Edge,
  Shutter, couleurs RGBAW, CTO/CTS, Hue/Saturation, Gobo Select, Rot 1/2), persisté en
  local. Sans sélection, l'encodeur n'envoie rien. Approche calquée sur Luminosus Eos
  Edition (`EosEncoderBlock.cpp`).

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
