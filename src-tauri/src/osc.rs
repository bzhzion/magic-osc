//! Emission OSC (UDP a l'etape 1, TCP ajoute plus tard pour Allen & Heath).
//!
//! `rosc` fait l'encodage. Nous tenons la socket UDP ouverte : rouvrir une
//! socket a chaque envoi ajoute 20-50 ms de latence Windows (pile reseau),
//! inacceptable pour du MIDI learn qui doit sentir instantane sous le doigt.

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use std::net::{SocketAddr, UdpSocket};

pub struct Emetteur {
    socket: UdpSocket,
    destination: SocketAddr,
}

impl Emetteur {
    /// Ouvre une socket UDP liee au port 0 (l'OS choisit un port libre) et
    /// se prepare a parler a `destination`.
    pub fn nouveau(destination: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        Ok(Self {
            socket,
            destination,
        })
    }

    /// Envoie un message OSC avec son adresse et ses arguments. L'ordre des
    /// arguments est preserve. `rosc` encode sans allouer plus d'un buffer.
    pub fn envoyer(&self, adresse: &str, args: Vec<OscType>) -> std::io::Result<()> {
        let message = OscMessage {
            addr: adresse.to_string(),
            args,
        };
        let paquet = OscPacket::Message(message);
        let octets = encoder::encode(&paquet).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}"))
        })?;
        self.socket.send_to(&octets, self.destination)?;
        Ok(())
    }
}
