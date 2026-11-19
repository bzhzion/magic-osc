# Changelog

Toutes les évolutions notables de magic-osc sont documentées ici.

Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), versionnage
[SemVer](https://semver.org/lang/fr/). La section `[Unreleased]` accumule au fil de l'eau et
est renommée en numéro de version au moment de poser le tag.

## [Unreleased]

### Ajouté
- **Console graphique ETC Eos (Ion XE)** : 20 submasters avec leur bump button, 2 Master
  Playback faders, boutons GO et STOP/BACK, 4 encoders (Pan/Tilt/Zoom/Iris), Level Wheel.
  L'utilisateur saisit l'IP et le port de la console dans une barre en haut, et les gestes
  à la souris ou au tactile partent en OSC directement. Adresses OSC conformes au « Eos
  Family Show Control User Guide » (chapitre OSC) : `/eos/fader/1/N` pour les submasters
  avec `/fire` pour le bump, `/eos/fader/10/{1,2}` pour les deux Master Playbacks,
  `/eos/key/go_0` et `/eos/key/stop_back` pour Go/Stop (press 1 / release 0),
  `/eos/wheel/pan|tilt|zoom|iris` pour les 4 encoders avec delta flottant,
  `/eos/wheel/level` pour la Level Wheel.
