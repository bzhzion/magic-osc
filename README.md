# magic-osc

**Apprenez une surface MIDI à une console en OSC.** Branchez un contrôleur MIDI (clavier,
pad, boîte de potentiomètres, pédalier) et une console son ou lumière qui parle OSC. magic-osc
écoute ce qui arrive du MIDI, vous choisissez l'action côté console, et il traduit en continu.

Windows, macOS, Linux (dont Raspberry Pi). Gratuit, sans compte. Édité par
[BREIZHZION](https://breizhzion.com), association d'intérêt général.

**[magicosc.breizhzion.com](https://magicosc.breizhzion.com)** pour la présentation et le
téléchargement.

---

## Installer

**Windows** : un seul fichier, aucun mot de passe administrateur demandé.

> [Magic-OSC-Setup-x64.exe](https://dl.breizhzion.com/magic-osc/Magic-OSC-Setup-x64.exe)

**Debian et Ubuntu**, par le dépôt de l'association :

```sh
curl -fsSL https://apt.breizhzion.com/KEY.gpg \
  | sudo gpg --dearmor -o /usr/share/keyrings/breizhzion.gpg
echo "deb [signed-by=/usr/share/keyrings/breizhzion.gpg] https://apt.breizhzion.com stable main" \
  | sudo tee /etc/apt/sources.list.d/breizhzion.list
sudo apt update && sudo apt install magic-osc
```

**Arch** :

```sh
printf '\n[breizhzion]\nSigLevel = Required DatabaseRequired\nServer = https://pacman.breizhzion.com/$arch\n' \
  | sudo tee -a /etc/pacman.conf
curl -fsSL https://pacman.breizhzion.com/KEY.gpg | sudo pacman-key --add -
sudo pacman-key --lsign-key apt@breizhzion.com
sudo pacman -Syu magic-osc-bin
```

**macOS** : DMG signé à venir.

**Raspberry Pi** (Pi 4 ou plus récent, 64 bits) : paquet `apt` ARM64 par le même dépôt que
Debian.

---

## Compiler depuis les sources

Prérequis : Rust 1.85+ (Rust édition 2024), Node 20+, et la toolchain Tauri de votre
plateforme (voir la [documentation Tauri](https://tauri.app/start/prerequisites/)).

```sh
git clone https://github.com/bzhzion/magic-osc
cd magic-osc
npm install
npm run tauri dev
```

## Convention de développement

- Le code est en français (modules, variables, commentaires).
- Les hooks git sont dans `.githooks/` et s'activent par `git config core.hooksPath .githooks`
  après le premier clone.
- Une modification de code s'accompagne d'une entrée dans `CHANGELOG.md`, sous `[Unreleased]`,
  renommée en numéro de version au moment du tag.
- Un tag `vX.Y.Z` publie une version. `changelog-guard.yml` refuse un tag si `[Unreleased]`
  n'est pas vide ou si du code a bougé sans entrée CHANGELOG.
