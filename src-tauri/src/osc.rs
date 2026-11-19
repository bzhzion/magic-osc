//! Emission ET reception OSC (UDP).
//!
//! `rosc` fait l'encodage/decodage. La socket d'envoi et la socket d'ecoute
//! du feedback sont la MEME socket : Eos repond au port d'ou vient la
//! requete (comportement standard d'un serveur OSC UDP), donc ouvrir une
//! deuxieme socket a part pour ecouter ne recevrait jamais rien.
//!
//! Sources (Eos Family Show Control User Guide, Rev C, chapitres "Active
//! Parameters" et "Implicit OSC Output") :
//! - Eos n'envoie son feedback QUE si « Transmit » (UDP TX) est coche cote
//!   console, dans Setup > System > Show Control > OSC. « Receive » seul
//!   (necessaire pour GO/STOP/faders) ne suffit PAS a recevoir ce feedback.
//! - `/eos/reset` force Eos a renvoyer IMMEDIATEMENT tout son etat implicite
//!   courant (wheels actifs, commande en cours...), au lieu d'attendre le
//!   prochain changement. Envoye juste apres connexion.
//! - Feedback des 4 wheels contextuels : `/eos/out/active/wheel/<1..4>`,
//!   DEUX arguments separes (string = nom du parametre, nombre = valeur
//!   courante), PAS une seule chaine "label [valeur]" comme le laissait
//!   supposer un essai precedent base sur du code tiers non officiel.

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

pub static EMETTEUR: Mutex<Option<Emetteur>> = Mutex::new(None);
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

    // /eos/reset : force Eos a renvoyer tout de suite son etat implicite
    // courant (wheels actifs...) sans attendre un changement. Sans ca,
    // l'UI resterait vide jusqu'a ce que l'operateur touche quelque chose
    // sur la console.
    if let Err(e) = emetteur.envoyer("/eos/reset", vec![]) {
        eprintln!("[osc] /eos/reset a echoue : {e}");
    }

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
    valeur: Option<f64>,
}

#[derive(Serialize, Clone)]
struct FeedbackNomSub {
    n: u8,
    libelle: String,
}

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

fn osc_type_vers_f64(v: &OscType) -> Option<f64> {
    match v {
        OscType::Int(i) => Some(*i as f64),
        OscType::Float(f) => Some(*f as f64),
        OscType::Double(d) => Some(*d),
        OscType::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn traiter_message_entrant(m: &OscMessage, app: &AppHandle) {
    let segments: Vec<&str> = m.addr.split('/').filter(|s| !s.is_empty()).collect();

    // /eos/out/active/wheel/<1..4> : DEUX arguments [nom (string), valeur
    // (nombre)], format officiel confirme (Eos Family Show Control User
    // Guide, section "Active Parameters").
    if segments.len() == 5
        && segments[0] == "eos"
        && segments[1] == "out"
        && segments[2] == "active"
        && segments[3] == "wheel"
    {
        if let Ok(index) = segments[4].parse::<u8>() {
            let libelle = match m.args.first() {
                Some(OscType::String(s)) => s.clone(),
                _ => String::new(),
            };
            let valeur = m.args.get(1).and_then(osc_type_vers_f64);
            let evt = FeedbackEncodeur { index, libelle, valeur };
            let _ = app.emit("eos-wheel-feedback", evt);
        }
        return;
    }

    // /eos/out/get/sub/<n>/list/<index>/<count> : reponse a /eos/get/sub/<n>.
    // args[2] = label (string). Voir Appendix: Advanced OSC, bloc "SUB (1 OF 2)".
    if segments.len() == 8
        && segments[0] == "eos"
        && segments[1] == "out"
        && segments[2] == "get"
        && segments[3] == "sub"
        && segments[5] == "list"
    {
        if let Ok(n) = segments[4].parse::<u8>() {
            if let Some(OscType::String(label)) = m.args.get(2) {
                let evt = FeedbackNomSub { n, libelle: label.clone() };
                let _ = app.emit("eos-sub-name-feedback", evt);
            }
        }
    }
}
