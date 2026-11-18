//! magic-osc : apprenez une surface MIDI a une console en OSC.
//!
//! Etape 1 : la fenetre existe et sait repondre a `--version`. Elle n'ecoute
//! pas encore le MIDI et n'envoie pas encore d'OSC.
//!
//! ⚠️ **Pas de `windows_subsystem = "windows"` ici, c'est deliberement.**
//! Ce drapeau supprime la console attachee sur Windows, ce qui evite qu'une
//! fenetre noire clignote au lancement. Mais il coupe aussi la sortie
//! standard : `magic-osc.exe --version` sortirait muet, et le workflow de
//! release qui compare cette sortie au tag echouerait. Le vrai correctif
//! (attacher la console du parent quand des arguments sont presents)
//! appartient a l'etape ou on finalise l'ergonomie Windows. D'ici la, on
//! garde une sortie qui fonctionne.

mod apprentissage;
mod association;
mod consoles;
mod midi;
mod osc;
mod platform;
mod reglages;
mod serveur;

/// Version DERIVEE par `build.rs` depuis le tag git, jamais lue dans le
/// manifeste Cargo.
pub const VERSION: &str = env!("MAGIC_OSC_VERSION");

#[tauri::command]
fn version() -> String {
    VERSION.to_string()
}

fn main() {
    // Mode ligne de commande minimal : `--version` et `-V` affichent la
    // version et sortent. Le workflow de release s'en sert.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("magic-osc {}", VERSION);
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![version])
        .run(tauri::generate_context!())
        .expect("echec du demarrage de magic-osc");
}
