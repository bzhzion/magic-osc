//! Persistance des reglages et des associations.
//!
//! Emplacement : `directories::ProjectDirs` choisit le bon pour chaque OS.
//! - Linux : `~/.config/com.breizhzion.magic-osc/`
//! - macOS : `~/Library/Application Support/com.breizhzion.magic-osc/`
//! - Windows : `%APPDATA%\com.breizhzion.magic-osc\`
//! Jamais a cote du binaire : une installation « pour tous les utilisateurs »
//! est en lecture seule, ecrire a cote planterait.
//!
//! Format : JSON lisible a la main, pour qu'un utilisateur puisse partager
//! sa configuration (copier le fichier) ou la debugger (ouvrir dans un
//! editeur). Les migrations futures porteront un champ `version_schema` en
//! tete du fichier.

use crate::association::Association;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Reglages {
    pub associations: Vec<Association>,
    pub port_midi_entree: Option<String>,
    pub destination_osc_hote: Option<String>,
    pub destination_osc_port: Option<u16>,
}

/// Retourne le chemin du fichier `reglages.json` sans le creer. L'appelant
/// gere la premiere ecriture.
pub fn chemin_reglages() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "breizhzion", "magic-osc")?;
    let config = dirs.config_dir();
    Some(config.join("reglages.json"))
}
