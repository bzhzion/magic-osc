//! Banques de commandes OSC prepretes pour les consoles usuelles.
//!
//! Etape 1 : ETC Eos (famille Ion/Ion XE/Element 2/Gio/Ti). Les adresses OSC
//! suivent la reference officielle ETC « Eos Family Show Control User
//! Guide » Rev C, chapitre OSC + Appendix: Advanced OSC. Port UDP par
//! defaut 8000 cote console.
//!
//! Prerequis cote console (Setup > Show > ShowControl) :
//! - « String RX » pour que la console execute nos commandes (GO, faders...).
//! - « String TX » + OSC TX IP Address/Port pour que la console nous
//!   renvoie du feedback (quel parametre est assigne a chaque encodeur,
//!   nom des subs...). Contrairement a une reponse HTTP, l'UDP TX d'Eos
//!   part vers une destination FIXE configuree cote console, PAS vers le
//!   port depuis lequel on a envoye notre derniere requete. Voir le
//!   commentaire de `osc::PORT_ECOUTE`.
//!
//! Les 13+ banques de JustMakeQ seront portees aux etapes suivantes.

pub mod etc_eos {
    //! Adresses OSC ETC Eos utilisees par la console graphique etape 2.

    // ── Submasters 1..20 ───────────────────────────────────────────────────
    // `/eos/sub/<N>` pilote directement le submaster N, sans exiger la
    // configuration d'un « OSC Fader Bank » cote console.
    pub const SUB_PREFIXE: &str = "/eos/sub/";
    // Bump (fire) : /eos/sub/<N>/fire, arg 1.0 = press, 0.0 = release.
    pub const SUB_FIRE_SUFFIXE: &str = "/fire";
    // Selection du sub (necessaire avant Group/Assert/Load, qui agissent
    // sur le sub actuellement cible) : /eos/sub + int N.
    pub const SUB_SELECT: &str = "/eos/sub";
    // Requete du nom : /eos/get/sub/<N>, Eos repond sur
    // /eos/out/get/sub/<N>/list/<index>/<count> avec le label en 3e argument
    // (index 2) -- voir Appendix: Advanced OSC, bloc "SUB (1 OF 2)".
    pub const SUB_GET_PREFIXE: &str = "/eos/get/sub/";

    // ── Master Playback A et B (les deux grands faders de droite) ──────────
    // Demande que Setup > OSC > Fader Bank 1 soit configure sur
    // « Master Playback » pour avoir un effet ; GO/STOP marchent sans ca
    // car ils passent par les touches, pas par le fader.
    pub const MASTER_FADER_1: &str = "/eos/user/1/fader/1/1";
    pub const MASTER_FADER_2: &str = "/eos/user/1/fader/1/2";

    pub const KEY_GO: &str = "/eos/key/go_0";
    pub const KEY_STOP: &str = "/eos/key/stop";

    // ── Touches generiques /eos/key/<nom> ──────────────────────────────────
    // Arg : nombre, 1.0 = press, 0.0 = release (section "Supported OSC Input").
    // Noms confirmes dans le document officiel "Eos OSC Keys" (liste
    // exhaustive des cles OSC <-> macro interne ETC).
    pub const KEY_PREFIXE: &str = "/eos/key/";

    // Boutons de PAGE D'ENCODEUR (bandeau au-dessus des encodeurs sur
    // Ion/Ion XE) : ils changent la categorie CIA actuellement affichee, ce
    // qui determine quels parametres ("Active Parameters") apparaissent sur
    // les 4 encodeurs physiques. Noms confirmes dans "Eos OSC Keys.pdf" :
    // suffixe interne `_CAT` (INTENSITY_CAT, FOCUS_CAT, COLOR_CAT,
    // SHUTTER_CAT, IMAGE_CAT, FORM_CAT) -- PAS le prefixe
    // `encoder_category_*` (ENCODER_CAT_OTHER/SPARE/BEAM/GRAPHIC/SHUTTER),
    // teste sans aucun effet sur les encodeurs malgre son nom trompeur.
    pub const KEY_ENC_CAT_INTENSITY: &str = "intensity";
    pub const KEY_ENC_CAT_FOCUS: &str = "focus";
    pub const KEY_ENC_CAT_COLOR: &str = "color";
    pub const KEY_ENC_CAT_SHUTTER: &str = "shutter";
    pub const KEY_ENC_CAT_IMAGE: &str = "image";
    pub const KEY_ENC_CAT_FORM: &str = "form";

    // ── Encoders (4 boutons rotatifs au-dessus de l'ecran) ─────────────────
    // Confirme par le document officiel, section "Active Parameters" :
    //
    // - EN SORTIE (feedback, Eos -> magic-osc), section "Implicit OSC
    //   Output" : `/eos/out/active/wheel/<1..4>` avec DEUX arguments
    //   separes (string = nom du parametre courant, nombre = valeur
    //   courante). Envoye automatiquement des que la selection ou le
    //   parametre assigne change. Label vide = encodeur actuellement
    //   inassigne.
    //
    // - EN ENTREE (commande, magic-osc -> Eos), section "Wheel" (PAS
    //   "Switch" : Switch est une methode distincte pour des parametres a
    //   bascule, pas pour les encodeurs rotatifs continus -- erreur de la
    //   version precedente) : `/eos/active/wheel/<1..4>` avec un delta
    //   flottant. Eos route en interne vers le parametre actuellement
    //   affecte a cet index : on adresse par index physique (1-based),
    //   jamais par nom.
    pub const OUT_ACTIVE_WHEEL_PREFIXE: &str = "/eos/out/active/wheel/";
    pub const ACTIVE_WHEEL_PREFIXE: &str = "/eos/active/wheel/";

    // ── Level Wheel ────────────────────────────────────────────────────────
    pub const WHEEL_LEVEL: &str = "/eos/wheel/level";
}
