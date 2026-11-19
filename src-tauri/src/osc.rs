//! Emission ET reception OSC (UDP).
//!
//! `rosc` fait l'encodage/decodage.
//!
//! ⚠️ **La socket d'envoi et la socket d'ecoute du feedback sont DEUX
//! sockets distinctes, sur des ports differents.** Contrairement a une
//! reponse HTTP, Eos n'envoie jamais son feedback UDP vers le port depuis
//! lequel on vient de lui parler : il l'envoie vers une destination FIXE
//! configuree cote console (Setup > Show > ShowControl > « OSC TX IP
//! Address » / « OSC TX Port Number »), independamment de nos requetes.
//! Une premiere version de ce fichier clonait la socket d'envoi en
//! supposant un comportement "reply-to-sender" : aucun feedback n'arrivait
//! jamais, meme console bien configuree, parce qu'Eos visait un port que
//! magic-osc n'ecoutait pas. D'ou `PORT_ECOUTE`, fixe, a renseigner cote
//! console comme "OSC TX Port Number" (avec l'IP de la machine qui lance
//! magic-osc comme "OSC TX IP Address").
//!
//! Sources (Eos Family Show Control User Guide, Rev C, chapitres
//! "Configuring OSC", "Active Parameters" et "Implicit OSC Output") :
//! - « String RX » suffit pour GO/STOP/faders (on envoie, Eos execute).
//!   « String TX » + OSC TX IP/Port sont necessaires en plus pour tout
//!   feedback (wheels actifs, nom des subs...), car c'est Eos qui doit
//!   nous contacter cette fois.
//! - Ports recommandes par ETC : 8000 (OSC RX, cote console -- c'est le
//!   port auquel magic-osc envoie) et 8001 (OSC TX, cote console -- c'est
//!   le port sur lequel magic-osc doit ecouter : `PORT_ECOUTE`).
//! - `/eos/reset` force Eos a renvoyer IMMEDIATEMENT tout son etat implicite
//!   courant (wheels actifs, commande en cours...), au lieu d'attendre le
//!   prochain changement. Envoye juste apres connexion.
//! - Feedback des 4 wheels contextuels : `/eos/out/active/wheel/<1..4>`.
//!   Le Show Control User Guide annonce DEUX arguments (string, nombre),
//!   mais la capture reseau reelle (Eos Nomad, tcpdump) montre TROIS
//!   arguments, type tag OSC ",sif" (string, int, float) : le label
//!   embarque deja la valeur sous forme texte ("Intens  [69]"), l'int est
//!   un champ non identifie, et la vraie valeur numerique est le FLOAT en
//!   position 2. On lit position 2, pas position 1.

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use serde::Serialize;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Port fixe sur lequel magic-osc ecoute le feedback Eos. A renseigner
/// cote console comme « OSC TX Port Number » (Setup > Show > ShowControl),
/// avec l'IP de la machine qui lance magic-osc comme « OSC TX IP Address ».
pub const PORT_ECOUTE: u16 = 8001;

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
    // Socket DEDIEE a l'ecoute, sur le port fixe que la console doit viser
    // (voir le commentaire de PORT_ECOUTE en tete de fichier) -- surtout
    // pas un clone de la socket d'envoi, qui a un port ephemere qu'Eos ne
    // contactera jamais.
    let socket_ecoute = UdpSocket::bind(("0.0.0.0", PORT_ECOUTE))
        .map_err(|e| format!("impossible d'ecouter sur le port {PORT_ECOUTE} : {e}"))?;

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
                Ok((taille, expediteur)) => {
                    match rosc::decoder::decode_udp(&tampon[..taille]) {
                        Ok((_, paquet)) => traiter_paquet_entrant(&paquet, &app),
                        Err(e) => eprintln!(
                            "[osc] paquet recu de {expediteur} ({taille} octets) illisible : {e:?}"
                        ),
                    }
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    continue;
                }
                Err(e) => {
                    eprintln!("[osc] ecoute du feedback arretee : {e}");
                    break;
                }
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
    eprintln!("[osc] recu {} args={:?}", m.addr, m.args);
    let segments: Vec<&str> = m.addr.split('/').filter(|s| !s.is_empty()).collect();

    // /eos/out/active/wheel/<1..4> : 3 arguments reels (string, int, float),
    // voir le commentaire en tete de fichier.
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
            // Format reel observe (Eos Nomad, capture tcpdump) : 3 arguments
            // (string, int, float) type tag ",sif", PAS 2 comme le laissait
            // supposer le Show Control User Guide. La vraie valeur courante
            // est le FLOAT en position 2, pas l'int en position 1.
            let valeur = m.args.get(2).and_then(osc_type_vers_f64);
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
