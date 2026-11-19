//! magic-osc : apprenez une surface MIDI a une console en OSC.
//!
//! Etape 2 (branche console-eos-ion-xe) : l'UI de console Ion XE est en
//! place et sait envoyer ses actions sur OSC a une console ETC Eos sur le
//! reseau. Pas encore de MIDI learn : c'est la souris/tactile qui declenche,
//! la prochaine etape reliera des messages MIDI aux memes actions.
//!
//! ⚠️ **Pas de `windows_subsystem = "windows"` ici, c'est deliberement.**
//! Ce drapeau coupe la sortie standard : `magic-osc.exe --version`
//! sortirait muet et le workflow de release qui compare cette sortie au
//! tag echouerait. Le vrai correctif (attacher la console du parent quand
//! des arguments sont presents) attend une etape ulterieure.

mod apprentissage;
mod association;
mod consoles;
mod midi;
mod osc;
mod platform;
mod reglages;
mod serveur;

use rosc::OscType;
use serde::Serialize;

/// Version DERIVEE par `build.rs` depuis le tag git.
pub const VERSION: &str = env!("MAGIC_OSC_VERSION");

#[tauri::command]
fn version() -> String {
    VERSION.to_string()
}

/// Regle (ou remplace) la destination OSC. Hote peut etre une IP ou un nom
/// resolvable. Port par defaut Eos : 8000.
#[tauri::command]
fn regler_destination_osc(hote: String, port: u16) -> Result<(), String> {
    osc::regler_destination(&hote, port)
}

/// Envoie un message OSC dont les arguments sont deja typees cote frontend.
/// Les types acceptes : "f" (float), "i" (int), "s" (string).
#[tauri::command]
fn envoyer_osc(adresse: String, args: Vec<ArgumentTypé>) -> Result<(), String> {
    let args_osc: Vec<OscType> = args.into_iter().map(|a| a.en_osc()).collect();
    osc::envoyer(&adresse, args_osc)
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", content = "valeur")]
pub enum ArgumentTypé {
    #[serde(rename = "f")]
    Flottant(f32),
    #[serde(rename = "i")]
    Entier(i32),
    #[serde(rename = "s")]
    Chaine(String),
}

impl ArgumentTypé {
    fn en_osc(self) -> OscType {
        match self {
            ArgumentTypé::Flottant(v) => OscType::Float(v),
            ArgumentTypé::Entier(v) => OscType::Int(v),
            ArgumentTypé::Chaine(v) => OscType::String(v),
        }
    }
}

// ── Commandes groupees pour l'UI Ion XE ───────────────────────────────────
// Chaque commande ci-dessous compose directement la bonne adresse OSC Eos
// et le bon argument. L'UI frontend ne manipule PAS les adresses OSC : on
// garde cette correspondance ici, et un changement de protocole Eos (tres
// peu probable) ne demande qu'une correction dans ce fichier.

/// Fader submaster N (1..=20), valeur 0.0..=1.0.
#[tauri::command]
fn eos_fader_submaster(n: u8, valeur: f32) -> Result<(), String> {
    if !(1..=20).contains(&n) {
        return Err(format!("submaster hors plage : {n}"));
    }
    let v = valeur.clamp(0.0, 1.0);
    let adresse = format!("{}{n}", consoles::etc_eos::FADER_SUBMASTER_PREFIXE);
    osc::envoyer(&adresse, vec![OscType::Float(v)])
}

/// Bump d'un submaster. Press/release distincts pour permettre a l'UI de
/// tenir la touche.
#[tauri::command]
fn eos_fader_bump(n: u8, press: bool) -> Result<(), String> {
    if !(1..=20).contains(&n) {
        return Err(format!("submaster hors plage : {n}"));
    }
    let adresse = format!(
        "{}{n}{}",
        consoles::etc_eos::FADER_SUBMASTER_PREFIXE,
        consoles::etc_eos::FADER_FIRE_SUFFIXE
    );
    let v = if press { 1 } else { 0 };
    osc::envoyer(&adresse, vec![OscType::Int(v)])
}

/// Master Playback A/B : fader. Idem submaster mais bank 2 en cas d'Alternate.
#[tauri::command]
fn eos_fader_master(bank: u8, valeur: f32) -> Result<(), String> {
    if !(1..=2).contains(&bank) {
        return Err(format!("master bank hors plage : {bank}"));
    }
    let v = valeur.clamp(0.0, 1.0);
    // Bank 1/5 Master Playback = /eos/fader/1/21 sur Ion XE classique ;
    // on utilise la convention « fader bank 10 » pour le Master Playback
    // qui est l'adresse documentee par ETC pour un fader dedie.
    let adresse = format!("/eos/fader/10/{bank}");
    osc::envoyer(&adresse, vec![OscType::Float(v)])
}

/// GO du Master Playback. Press/release.
#[tauri::command]
fn eos_key_go(press: bool) -> Result<(), String> {
    let v = if press { 1 } else { 0 };
    osc::envoyer(consoles::etc_eos::KEY_GO, vec![OscType::Int(v)])
}

/// STOP/BACK du Master Playback. Press/release.
#[tauri::command]
fn eos_key_stop(press: bool) -> Result<(), String> {
    let v = if press { 1 } else { 0 };
    osc::envoyer(consoles::etc_eos::KEY_STOP_BACK, vec![OscType::Int(v)])
}

/// Un des quatre encoders (1..=4), delta signe.
#[tauri::command]
fn eos_wheel_encoder(n: u8, delta: f32) -> Result<(), String> {
    let adresse = match n {
        1 => consoles::etc_eos::WHEEL_1,
        2 => consoles::etc_eos::WHEEL_2,
        3 => consoles::etc_eos::WHEEL_3,
        4 => consoles::etc_eos::WHEEL_4,
        _ => return Err(format!("encoder hors plage : {n}")),
    };
    osc::envoyer(adresse, vec![OscType::Float(delta)])
}

/// Level Wheel : modifie l'intensite du selected channel.
#[tauri::command]
fn eos_wheel_level(delta: f32) -> Result<(), String> {
    osc::envoyer(consoles::etc_eos::WHEEL_LEVEL, vec![OscType::Float(delta)])
}

#[derive(Serialize)]
struct InfoDestination {
    hote: Option<String>,
    port: Option<u16>,
}

#[tauri::command]
fn info_destination_osc() -> InfoDestination {
    // Pour l'instant on ne retient pas hote/port dans un etat ; l'UI les
    // garde cote JS. Branchement « vraie persistance » a l'etape reglages.
    InfoDestination {
        hote: None,
        port: None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("magic-osc {}", VERSION);
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            version,
            regler_destination_osc,
            envoyer_osc,
            info_destination_osc,
            eos_fader_submaster,
            eos_fader_bump,
            eos_fader_master,
            eos_key_go,
            eos_key_stop,
            eos_wheel_encoder,
            eos_wheel_level,
        ])
        .run(tauri::generate_context!())
        .expect("echec du demarrage de magic-osc");
}
