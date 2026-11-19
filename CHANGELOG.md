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
- **Encoders** : retour utilisateur — « ils doivent se peupler en fonction de ce à quoi ils
  sont assignés sur Eos en fonction de la sélection », pas d'un choix manuel. Après vérification
  dans le code source de Luminosus Eos Edition (`EosActiveChannelsManager.cpp`), il y a bien
  deux adresses dédiées : Eos **envoie** en continu `/eos/out/active/wheel/<1..4>` (chaîne
  `"<label>  [<valeur>]"`, vide si l'encodeur n'a rien d'assigné) dès que la sélection ou la
  page change, et **reçoit** sur `/eos/active/switch/<1..4>` pour faire tourner l'encodeur N
  quel que soit le paramètre qui lui est affecté à l'instant — adressage par index physique,
  jamais par nom. magic-osc écoute maintenant ce feedback sur la même socket UDP que l'envoi
  (Eos répond au port d'origine de la requête) et affiche en direct, sous chaque encodeur
  `E1`-`E4`, le nom exact du paramètre assigné par la console, grisé et inerte tant qu'aucun
  paramètre n'y est affecté. Le menu déroulant manuel de la version précédente est retiré.

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
