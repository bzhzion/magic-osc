//! Moteur de « MIDI learn ».
//!
//! Etape 1 : squelette seulement. Les routes de l'etape 2 :
//! - `demarrer(canal)` : ecoute le prochain message MIDI de ce canal, et
//!   l'enregistre comme declencheur de l'association en cours de reglage
//!   dans l'UI.
//! - `annuler()` : abandonne un apprentissage lance.
//! - `dernier_message()` : rend ce qui vient d'arriver, pour que l'UI
//!   l'affiche en clair (« Note On do#3 velo 127 sur canal 1 »).
//!
//! L'appellation « canal » dans le code suit la numerotation MIDI 1-16, qui
//! est celle que l'utilisateur voit. Convertir 0-15 pour l'octet de statut
//! est fait au point d'emission/reception, jamais en amont.

/// Etat du moteur, maintenu pendant toute la vie de l'app.
#[derive(Default)]
pub struct Moteur {
    // Marqueur : y a-t-il un apprentissage en cours ?
    pub en_cours: bool,
}

impl Moteur {
    pub fn nouveau() -> Self {
        Self::default()
    }
}
