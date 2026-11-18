//! Banques de commandes OSC prepretes pour les consoles usuelles.
//!
//! Chaque banque expose ses actions avec leur libelle, leur adresse OSC et
//! leurs parametres (ex : n° de scene, intensite). L'UI s'en sert pour
//! remplir le cote droit de l'association : « quand ce message MIDI arrive,
//! envoie l'action X avec le parametre Y ».
//!
//! Les definitions viennent de la documentation officielle de chaque
//! fabricant. Le travail d'extraction a deja ete fait et valide dans le
//! parc JustMakeQ (`osc_banks.py`) : on reprend la meme structure et les
//! memes sources, pour que JustMakeQ et magic-osc parlent exactement le
//! meme OSC a une console donnee.
//!
//! Etape 1 : squelette. Les 13 banques de JustMakeQ seront portees a
//! l'etape 3 (ETC EOS, QLab, grandMA3, A&H SQ, dLive, Avantis, DM7, WING,
//! grandMA2 via OSC_Mate, MadMapper, Mitti, Reaper, D::Light, MagicQ, Hog 4,
//! Chimp, M32/X32, Yamaha DM3, Resolume, Millumin, OBS).

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Commande {
    pub cle: &'static str,
    pub libelle: &'static str,
    pub adresse: &'static str,
}

#[derive(Serialize, Clone, Debug)]
pub struct Banque {
    pub cle: &'static str,
    pub nom: &'static str,
    pub categorie: &'static str,
    pub port_defaut: Option<u16>,
    pub commandes: &'static [Commande],
}

/// Toutes les banques disponibles. Vide a l'etape 1, remplie a l'etape 3
/// avec les donnees issues de `osc_banks.py` de JustMakeQ.
pub const BANQUES: &[Banque] = &[];
