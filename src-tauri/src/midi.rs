//! Lecture MIDI multiplateforme via `midir`.
//!
//! Etape 1 : liste les ports MIDI entrants disponibles. L'ecoute en
//! continu avec dispatch vers `apprentissage.rs` arrive a l'etape 2.
//!
//! Choix de `midir` plutot que trois caisses OS-specifiques
//! (`coremidi`, `windows_midi`, `alsa`) : le MIDI n'a pas de subtilite
//! visible par l'utilisateur d'un OS a l'autre, et tripler le code
//! conditionnel ne gagne rien. `midir` tourne aussi sur Raspberry Pi 4
//! sans ajustement, ALSA etant la couche MIDI de Linux embarque.

use midir::MidiInput;
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct PortEntree {
    pub id: String,
    pub nom: String,
}

/// Liste les ports MIDI entrants vus par le systeme a l'instant de l'appel.
/// Un controleur branche apres cet appel n'apparaitra qu'au prochain
/// rafraichissement (l'UI redemande toutes les N secondes a l'etape 2).
pub fn lister_ports_entree() -> Result<Vec<PortEntree>, String> {
    let entree = MidiInput::new("magic-osc").map_err(|e| e.to_string())?;
    let ports = entree.ports();
    let mut sortie = Vec::with_capacity(ports.len());
    for (idx, port) in ports.iter().enumerate() {
        let nom = entree.port_name(port).unwrap_or_else(|_| "?".to_string());
        sortie.push(PortEntree {
            id: format!("{idx}"),
            nom,
        });
    }
    Ok(sortie)
}
