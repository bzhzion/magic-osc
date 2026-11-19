//! Emission ET reception OSC (UDP).
//!
//! `rosc` fait l'encodage/decodage. La socket d'envoi et la socket d'ecoute
//! du feedback sont la MEME socket : Eos repond au port d'ou vient la
//! requete (comportement standard d'un serveur OSC UDP), donc ouvrir une
//! deuxieme socket a part pour ecouter ne recevrait jamais rien. Rouvrir une
//! socket a chaque envoi ajoute par ailleurs 20-50 ms de latence Windows,
//! inacceptable pour un encodeur qui doit sentir instantane sous le doigt.
//!
//! Le feedback Eos (quel parametre est assigne a quel encodeur, et sa
//! valeur courante) arrive sur `/eos/out/active/wheel/<1..4>`. On le
//! retransmet au frontend via un evenement Tauri `eos-wheel-feedback`.

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use serde::Serialize;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

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
/// appel a `regler_destination`.
pub static EMETTEUR: Mutex<Option<Emetteur>> = Mutex::new(None);

/// Compteur de connexion : chaque appel a `regler_destination` l'incremente,
/// le thread d'ecoute en cours compare sa propre valeur a chaque reveil et
/// s'arrete si elle a change. Evite d'accumuler des threads d'ecoute morts
/// a chaque reconnexion (changer d'IP de console en cours de route).
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn regler_destination(hote: &str, port: u16, app: AppHandle) -> Result<(), String> {
    let adresse = (hote, port)
        .to_socket_addrs()
        .map_err(|e| format!("adresse invalide : {e}"))?
        .next()
        .ok_or_else(|| "aucune adresse resolue".to_string())?;
    let emetteur = Emetteur::nouveau(adresse).map_err(|e| e.to_string())?;
    let socket_ecoute = emetteur
        .socket
        .try_clone()
        .map_err(|e| format!("impossible de dupliquer la socket : {e}"))?;

    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    {
        let mut tampon = EMETTEUR.lock().unwrap();
        *tampon = Some(emetteur);
    }

    demarrer_ecoute_feedback(socket_ecoute, generation, app);
    Ok(())
}

pub fn envoyer(adresse: &str, args: Vec<OscType>) -> Result<(), String> {
    let tampon = EMETTEUR.lock().unwrap();
    let emetteur = tampon
        .as_ref()
        .ok_or_else(|| "destination OSC non reglee".to_string())?;
    emetteur.envoyer(adresse, args).map_err(|e| e.to_string())
}

#[derive(Serialize, Clone)]
struct FeedbackEncodeur {
    index: u8,
    libelle: String,
    valeur: Option<i64>,
}

/// Lance un thread qui lit en boucle la socket partagee avec l'emission et
/// retransmet au frontend tout feedback de roue active Eos. Timeout court
/// pour pouvoir verifier regulierement si une reconnexion a eu lieu
/// (`GENERATION` a change) et s'arreter proprement sans fermer violemment
/// la socket en cours d'usage.
fn demarrer_ecoute_feedback(socket: UdpSocket, generation: u64, app: AppHandle) {
    if let Err(e) = socket.set_read_timeout(Some(Duration::from_millis(400))) {
        eprintln!("[osc] set_read_timeout a echoue : {e}");
    }
    std::thread::spawn(move || {
        let mut tampon = [0u8; 4096];
        loop {
            if GENERATION.load(Ordering::SeqCst) != generation {
                break;
            }
            match socket.recv_from(&mut tampon) {
                Ok((taille, _)) => {
                    if let Ok((_, paquet)) = rosc::decoder::decode_udp(&tampon[..taille]) {
                        traiter_paquet_entrant(&paquet, &app);
                    }
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    continue;
                }
                Err(_) => break,
            }
        }
    });
}

fn traiter_paquet_entrant(paquet: &OscPacket, app: &AppHandle) {
    match paquet {
        OscPacket::Message(m) => traiter_message_entrant(m, app),
        OscPacket::Bundle(b) => {
            for contenu in &b.content {
                traiter_paquet_entrant(contenu, app);
            }
        }
    }
}

/// Feedback attendu : `/eos/out/active/wheel/<1..4>` avec un argument
/// string au format `"<label>  [<valeur>]"`. Label vide = encodeur
/// actuellement inassigne (aucun parametre sous ce numero sur la console).
fn traiter_message_entrant(m: &OscMessage, app: &AppHandle) {
    let segments: Vec<&str> = m.addr.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() != 5
        || segments[0] != "eos"
        || segments[1] != "out"
        || segments[2] != "active"
        || segments[3] != "wheel"
    {
        return;
    }
    let Ok(index) = segments[4].parse::<u8>() else { return };
    let Some(OscType::String(brut)) = m.args.first() else { return };
    let (libelle, valeur) = parser_label_valeur(brut);
    let evt = FeedbackEncodeur { index, libelle, valeur };
    let _ = app.emit("eos-wheel-feedback", evt);
}

fn parser_label_valeur(brut: &str) -> (String, Option<i64>) {
    if let Some(pos) = brut.find("  [") {
        let label = brut[..pos].to_string();
        let reste = &brut[pos + 3..];
        let valeur = reste.trim_end_matches(']').trim().parse::<i64>().ok();
        (label, valeur)
    } else {
        (brut.trim().to_string(), None)
    }
}
