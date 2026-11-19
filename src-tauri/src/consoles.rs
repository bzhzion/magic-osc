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

    // ── Submasters 1..20 ───────────────────────────────────────────────────
    // `/eos/sub/<N>` pilote directement le submaster N, sans exiger la
    // configuration d'un « OSC Fader Bank » cote console. `/eos/fader/<bank>/<N>`
    // existe aussi mais demande que l'utilisateur ait d'abord cree un bank
    // OSC dans Setup > Show Control > OSC (sinon le message est ignore).
    // `/eos/sub/<N>` evite ce prerequis.
    pub const SUB_PREFIXE: &str = "/eos/sub/";
    // Bump (fire) sous chaque fader : /eos/sub/<N>/fire, int 1 = press,
    // int 0 = release.
    pub const SUB_FIRE_SUFFIXE: &str = "/fire";

    // ── Master Playback A et B (les deux grands faders de droite) ──────────
    // Deux approches possibles :
    // - `/eos/user/1/fader/1/1` pour le Master Playback, demande que
    //   Setup > OSC > Fader Bank 1 soit configure sur « Master Playback ».
    // - Plus robuste : la plupart des utilisateurs manipulent le Master
    //   Playback via les keys GO / STOP, le fader en lui-meme ne bougeant
    //   qu'a l'allumage. On expose quand meme un fader soft qui envoie
    //   `/eos/user/1/fader/1/1`, en sachant que la console l'ignorera si
    //   le bank 1 n'est pas assigne au Master Playback.
    pub const MASTER_FADER_1: &str = "/eos/user/1/fader/1/1";
    pub const MASTER_FADER_2: &str = "/eos/user/1/fader/1/2";

    // GO et STOP/BACK : touches physiques du Master Playback. L'API OSC Eos
    // les expose en /eos/key/<nom>, avec un argument int 1 au press et 0 au
    // release. Le nom de la touche STOP/BACK est `stop` tout court
    // (verifie dans la reference Eos Family OSC + confirme par le code de
    // Luminosus Eos Edition).
    pub const KEY_GO: &str = "/eos/key/go_0";
    pub const KEY_STOP: &str = "/eos/key/stop";

    // ── Encoders (4 boutons rotatifs au-dessus de l'ecran) ─────────────────
    // Deux adresses complementaires, confirmees par le code source de
    // Luminosus Eos Edition (EosActiveChannelsManager.cpp) :
    //
    // - EN SORTIE (feedback, Eos -> magic-osc) : `/eos/out/active/wheel/<1..4>`
    //   avec un argument string "<label>  [<valeur>]". Eos l'envoie des que
    //   la selection change ou qu'un encodeur change de parametre/valeur.
    //   Label vide = cet encodeur n'a aucun parametre assigne actuellement.
    //   C'est ce qui permet a magic-osc d'afficher le BON nom sous chaque
    //   encodeur sans que l'utilisateur ait a le choisir lui-meme.
    //
    // - EN ENTREE (commande, magic-osc -> Eos) : `/eos/active/switch/<1..4>`
    //   avec un delta flottant. Eos route en interne vers le parametre
    //   actuellement affecte a cet index, quel qu'il soit : adresser par
    //   INDEX physique, jamais par nom, contrairement a ce qu'un premier
    //   essai avait suppose.
    pub const OUT_ACTIVE_WHEEL_PREFIXE: &str = "/eos/out/active/wheel/";
    pub const ACTIVE_SWITCH_PREFIXE: &str = "/eos/active/switch/";

    // ── Level Wheel ────────────────────────────────────────────────────────
    // La grosse molette d'intensite. Delta float.
    pub const WHEEL_LEVEL: &str = "/eos/wheel/level";
}
