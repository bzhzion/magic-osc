//! Banques de commandes OSC prepretes pour les consoles usuelles.
//!
//! Etape 1 : ETC Eos (famille Ion/Ion XE/Element 2/Gio/Ti). Les adresses OSC
//! suivent la reference officielle ETC « Eos Family Show Control User
//! Guide », chapitre OSC, port UDP par defaut 8000 cote console.
//!
//! Prerequis cote console : le serveur OSC doit etre active dans
//! Setup > System > Show Control > OSC. L'adresse IP de la machine qui
//! envoie (ou son broadcast) doit etre renseignee cote console si
//! « OSC RX IP » est filtre.
//!
//! Les 13+ banques de JustMakeQ seront portees aux etapes suivantes.
//! Pour l'instant, Eos est codee en dur dans les commandes Tauri :
//! l'etape 3 extraira ces chaines en table pour les presenter dans l'UI
//! « choisir une action sur Eos ».

pub mod etc_eos {
    //! Adresses OSC ETC Eos utilisees par la console graphique etape 2.

    // ── Submasters (Submaster 1 a 20, les 20 faders du bas) ────────────────
    // Fader de value float 0.0 a 1.0.
    pub const FADER_SUBMASTER_PREFIXE: &str = "/eos/fader/1/";
    // Bump (fire) : les touches sous chaque fader. int 1 = press, 0 = release.
    pub const FADER_FIRE_SUFFIXE: &str = "/fire";
    // Stop : certaines consoles exposent `/stop` par fader, utile pour
    // interrompre un chaser.
    pub const FADER_STOP_SUFFIXE: &str = "/stop";

    // ── Master Playback A et B (les deux grands faders de droite) ──────────
    // Sur Eos, le Master Playback fader n'a pas d'adresse propre : il manoeuvre
    // par les keys GO/STOP. Les deux grands faders de l'Ion XE sont la cue list
    // principale (A = Go/Stop) et son miroir B (Alternate). On mappe A sur le
    // Master Playback par defaut, B sur la « Alternate Cuelist » si l'utilisateur
    // en a configure une.
    pub const KEY_GO: &str = "/eos/key/go_0";
    pub const KEY_STOP_BACK: &str = "/eos/key/stop_back";

    // ── Encoders (les boutons rotatifs) ────────────────────────────────────
    // Eos expose quatre encoders logiques qui prennent un delta float.
    // Ils peuvent etre reassignes par l'utilisateur sur la console (Intens,
    // Pan, Tilt, Zoom, Edge, Iris...), ici on les cable sur les quatre
    // parametres les plus courants.
    pub const WHEEL_1: &str = "/eos/wheel/pan";
    pub const WHEEL_2: &str = "/eos/wheel/tilt";
    pub const WHEEL_3: &str = "/eos/wheel/zoom";
    pub const WHEEL_4: &str = "/eos/wheel/iris";

    // ── Level Wheel (la grosse molette d'intensite) ────────────────────────
    // La molette d'intensite modifie l'intensite du selected channel. Delta
    // float, l'amplitude depend du reglage « Wheel Sensitivity » cote console.
    pub const WHEEL_LEVEL: &str = "/eos/wheel/level";
}
