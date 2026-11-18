// Version du binaire DERIVEE du tag git, jamais lue dans Cargo.toml.
//
// Pourquoi : Cargo.toml reste a "0.0.0" en permanence. Le workflow de
// publication compare la sortie de `magic-osc --version` au tag qui la
// declenche, et un decalage entre les deux est une cause d'incident bete a
// chaque release. Un seul point de verite = pas de divergence.
//
// Preseance :
//   1. Si `git describe` reussit, on l'utilise.
//   2. Sinon, on tombe sur la variable d'environnement `MAGIC_OSC_VERSION`
//      (le workflow de release la pose quand git n'a pas l'historique).
//   3. En dernier recours, "dev" : signale explicitement un binaire sans
//      origine, au lieu de laisser croire que c'est une vraie version.

use std::process::Command;

fn main() {
    let version = version_depuis_git()
        .or_else(|| std::env::var("MAGIC_OSC_VERSION").ok())
        .unwrap_or_else(|| "dev".to_string());

    println!("cargo:rustc-env=MAGIC_OSC_VERSION={}", version);
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-env-changed=MAGIC_OSC_VERSION");

    tauri_build::build();
}

fn version_depuis_git() -> Option<String> {
    let sortie = Command::new("git")
        .args(["describe", "--tags", "--always", "--match", "v[0-9]*"])
        .output()
        .ok()?;
    if !sortie.status.success() {
        return None;
    }
    let brut = String::from_utf8(sortie.stdout).ok()?.trim().to_string();
    // `git describe` rend "v1.2.3" pour un tag, "v1.2.3-5-g1a2b3c" entre deux
    // tags. On enleve le "v" initial pour que la version affichee suive
    // SemVer de pres.
    Some(brut.strip_prefix('v').unwrap_or(&brut).to_string())
}
