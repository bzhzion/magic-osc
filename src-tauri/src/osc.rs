//! Emission OSC (UDP pour l'etape 1, TCP ajoute plus tard pour Allen & Heath).
//!
//! `rosc` fait l'encodage. Nous tenons la socket UDP ouverte : rouvrir une
//! socket a chaque envoi ajoute 20-50 ms de latence Windows (pile reseau),
//! inacceptable pour une console lumiere ou chaque tick de l'encoder doit
//! sentir instantane sous le doigt.
//!
//! L'`Emetteur` global est partage entre les commandes Tauri via un
//! `Mutex`. Un changement de destination (host/port) remplace la socket
//! sous ce mutex, sans que l'appelant ait a s'en occuper.

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Mutex;

pub struct Emetteur {
    socket: UdpSocket,
    destination: SocketAddr,
}

impl Emetteur {
    pub fn nouveau(destination: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        Ok(Self { socket, destination })
    }

    pub fn envoyer(&self, adresse: &str, args: Vec<OscType>) -> std::io::Result<()> {
        let paquet = OscPacket::Message(OscMessage {
            addr: adresse.to_string(),
            args,
        });
        let octets = encoder::encode(&paquet).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}"))
        })?;
        self.socket.send_to(&octets, self.destination)?;
        Ok(())
    }
}

/// Tampon partage entre toutes les commandes Tauri. Initialise au premier
/// appel a `regler_destination`. Avant ca, les commandes d'envoi refusent
/// (la console n'a pas d'adresse).
pub static EMETTEUR: Mutex<Option<Emetteur>> = Mutex::new(None);

pub fn regler_destination(hote: &str, port: u16) -> Result<(), String> {
    let adresse = (hote, port)
        .to_socket_addrs()
        .map_err(|e| format!("adresse invalide : {e}"))?
        .next()
        .ok_or_else(|| "aucune adresse resolue".to_string())?;
    let emetteur = Emetteur::nouveau(adresse).map_err(|e| e.to_string())?;
    let mut tampon = EMETTEUR.lock().unwrap();
    *tampon = Some(emetteur);
    Ok(())
}

pub fn envoyer(adresse: &str, args: Vec<OscType>) -> Result<(), String> {
    let tampon = EMETTEUR.lock().unwrap();
    let emetteur = tampon
        .as_ref()
        .ok_or_else(|| "destination OSC non reglee".to_string())?;
    emetteur.envoyer(adresse, args).map_err(|e| e.to_string())
}
