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
/// resolvable. Port par defaut Eos : 8000. `app` est injecte automatiquement
/// par Tauri : il sert a emettre l'evenement `eos-wheel-feedback` quand un
/// message de feedback Eos arrive sur la socket.
#[tauri::command]
fn regler_destination_osc(hote: String, port: u16, app: tauri::AppHandle) -> Result<(), String> {
    osc::regler_destination(&hote, port, app)
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

/// Submaster N (1..=20), valeur 0.0..=1.0. Pilote `/eos/sub/<N>`.
#[tauri::command]
fn eos_fader_submaster(n: u8, valeur: f32) -> Result<(), String> {
    if !(1..=20).contains(&n) {
        return Err(format!("submaster hors plage : {n}"));
    }
    let v = valeur.clamp(0.0, 1.0);
    let adresse = format!("{}{n}", consoles::etc_eos::SUB_PREFIXE);
    osc::envoyer(&adresse, vec![OscType::Float(v)])
}

/// Bump d'un submaster. Press/release distincts pour permettre a l'UI de
/// tenir la touche. Envoie `/eos/sub/<N>/fire` + int 1 (press) ou 0 (release).
#[tauri::command]
fn eos_fader_bump(n: u8, press: bool) -> Result<(), String> {
    if !(1..=20).contains(&n) {
        return Err(format!("submaster hors plage : {n}"));
    }
    let adresse = format!(
        "{}{n}{}",
        consoles::etc_eos::SUB_PREFIXE,
        consoles::etc_eos::SUB_FIRE_SUFFIXE
    );
    let v = if press { 1 } else { 0 };
    osc::envoyer(&adresse, vec![OscType::Int(v)])
}

/// Master Playback A/B : fader. Demande que le Fader Bank 1 soit configure
/// « Master Playback » cote console pour avoir un effet.
#[tauri::command]
fn eos_fader_master(bank: u8, valeur: f32) -> Result<(), String> {
    let v = valeur.clamp(0.0, 1.0);
    let adresse = match bank {
        1 => consoles::etc_eos::MASTER_FADER_1,
        2 => consoles::etc_eos::MASTER_FADER_2,
        _ => return Err(format!("master bank hors plage : {bank}")),
    };
    osc::envoyer(adresse, vec![OscType::Float(v)])
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
    osc::envoyer(consoles::etc_eos::KEY_STOP, vec![OscType::Int(v)])
}

/// Fait tourner le parametre actif N. Adresse par INDEX
/// (`/eos/active/wheel/<N>`), jamais par nom de parametre : c'est Eos qui
/// route en interne. N n'est PAS limite a 1..=4 : l'index reel dans la
/// liste complete des parametres actifs du projecteur selectionne peut
/// depasser 4 une fois la pagination par categorie geree cote frontend
/// (voir `recalculerPageEncodeurs` dans main.ts -- Eos n'expose aucun moyen
/// de savoir "quel parametre est sur quel encodeur physique a l'instant T",
/// seulement la liste complete et fixe des parametres actifs).
#[tauri::command]
fn eos_wheel_switch(n: u16, delta: f32) -> Result<(), String> {
    if n < 1 {
        return Err(format!("index de parametre hors plage : {n}"));
    }
    let adresse = format!("{}{n}", consoles::etc_eos::ACTIVE_WHEEL_PREFIXE);
    osc::envoyer(&adresse, vec![OscType::Float(delta)])
}

/// Level Wheel : modifie l'intensite du selected channel.
#[tauri::command]
fn eos_wheel_level(delta: f32) -> Result<(), String> {
    osc::envoyer(consoles::etc_eos::WHEEL_LEVEL, vec![OscType::Float(delta)])
}

/// Touche generique /eos/key/<name>, press/release. Utilisee pour les 6
/// boutons de page d'encodeur (Intens, Focus, Color, Shutter, Image, Form).
#[tauri::command]
fn eos_key(name: String, press: bool) -> Result<(), String> {
    let adresse = format!("{}{name}", consoles::etc_eos::KEY_PREFIXE);
    let v = if press { 1.0 } else { 0.0 };
    osc::envoyer(&adresse, vec![OscType::Float(v)])
}

/// Selectionne le submaster N sur la ligne de commande Eos. C'est le
/// bouton carre present sous chaque fader sur la console physique.
#[tauri::command]
fn eos_sub_select(n: u8) -> Result<(), String> {
    if !(1..=20).contains(&n) {
        return Err(format!("submaster hors plage : {n}"));
    }
    osc::envoyer(consoles::etc_eos::SUB_SELECT, vec![OscType::Int(n as i32)])
}

/// Demande le nom (label) du submaster N a la console. La reponse arrive
/// de maniere asynchrone via l'evenement Tauri `eos-sub-name-feedback`
/// (voir osc.rs), pas en valeur de retour : OSC est un protocole sans
/// requete/reponse synchrone.
#[tauri::command]
fn eos_sub_demander_nom(n: u8) -> Result<(), String> {
    if !(1..=20).contains(&n) {
        return Err(format!("submaster hors plage : {n}"));
    }
    let adresse = format!("{}{n}", consoles::etc_eos::SUB_GET_PREFIXE);
    osc::envoyer(&adresse, vec![])
}

#[derive(Serialize)]
struct InfoDestination {
    hote: Option<String>,
    port: Option<u16>,
    port_ecoute: u16,
}

#[tauri::command]
fn info_destination_osc() -> InfoDestination {
    // Pour l'instant on ne retient pas hote/port dans un etat ; l'UI les
    // garde cote JS. Branchement « vraie persistance » a l'etape reglages.
    InfoDestination {
        hote: None,
        port: None,
        port_ecoute: osc::PORT_ECOUTE,
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
            eos_wheel_switch,
            eos_wheel_level,
            eos_key,
            eos_sub_select,
            eos_sub_demander_nom,
        ])
        .run(tauri::generate_context!())
        .expect("echec du demarrage de magic-osc");
}
