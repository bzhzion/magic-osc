//! Table des associations `(message MIDI) -> (adresse OSC + arguments)`.
//!
//! Chaque association est serialisable, persistee par `reglages.rs`. Ce module
//! ne touche pas au disque : il sert de modele partage entre l'UI et le
//! moteur de dispatch.

use serde::{Deserialize, Serialize};

/// Un declencheur MIDI reconnu. Un canal vide (`None`) signifie « peu importe
/// le canal » : utile pour reutiliser la meme association sur plusieurs
/// canaux de clavier.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Declencheur {
    pub famille: FamilleMidi,
    pub canal: Option<u8>,   // 1..=16
    pub octet1: u8,          // note ou numero de CC
    pub octet2_exigence: ExigenceVelocite,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FamilleMidi {
    NoteOn,
    NoteOff,
    ControlChange,
    ProgramChange,
    PitchBend,
}

/// Comment traiter la velocite / valeur du 3e octet.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ExigenceVelocite {
    /// On declenche quelle que soit la velocite. Utile pour un pad qui
    /// envoie velocity != 0 au toucher.
    Quelconque,
    /// On exige exactement cette valeur. Utile pour un bouton qui ne
    /// doit reagir qu'en appui (127) et pas en relachement (0).
    Egale(u8),
    /// On reporte la valeur recue vers un argument OSC (ex : un fader MIDI
    /// 0-127 devient un float OSC 0.0-1.0).
    RelayeeEnFlottant,
}

/// Action a executer cote OSC quand un declencheur est reconnu.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActionOsc {
    pub adresse: String,
    pub arguments: Vec<ArgumentOsc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ArgumentOsc {
    /// Un entier fixe.
    Int(i32),
    /// Un flottant fixe.
    Float(f32),
    /// Une chaine fixe.
    Str(String),
    /// La velocite/valeur MIDI, relayee comme flottant 0.0..=1.0.
    VelociteRelayee,
}

/// Une association complete : `quand ca arrive`, `ca part`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Association {
    pub id: String,
    pub nom: String,
    pub declencheur: Declencheur,
    pub action: ActionOsc,
    pub actif: bool,
}
