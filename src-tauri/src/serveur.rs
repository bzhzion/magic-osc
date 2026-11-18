//! Serveur WebSocket local pour les clients mobiles (etape ulterieure).
//!
//! Etape 1 : squelette seulement. L'idee est que lorsqu'on fera une app
//! smartphone / tablette, elle se connectera a ce WebSocket pour :
//! - lire la liste des associations,
//! - declencher a distance une action OSC (comme un surcouche MIDI depuis
//!   le telephone),
//! - suivre en direct l'activite (temoin qui clignote quand une association
//!   se declenche sur l'ordinateur).
//!
//! Pas d'ouverture de port tant qu'on n'active pas explicitement cette
//! fonction dans les reglages. Par defaut magic-osc ne tourne pas en
//! service, l'utilisateur doit activer « piloter depuis mon telephone ».
//!
//! Le protocole d'appairage (clef pre-partagee, QR code) sera calque sur
//! celui d'oyant-desktop (voir `appairage.rs` la-bas) : TLS avec certificat
//! auto-signe epingle, defi/reponse Ed25519, generation du certificat a
//! l'installation. Pas de dependance supplementaire a prevoir avant cette
//! etape.
