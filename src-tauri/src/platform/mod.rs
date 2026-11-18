//! Specificites par OS (demarrage auto, raccourcis globaux, integration tray).
//!
//! Etape 1 : ce module n'expose rien, il est deja en place pour que les
//! branchements des etapes suivantes aient un endroit coherent ou vivre
//! plutot que d'etre semes dans `main.rs`.

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(windows)]
mod windows;
