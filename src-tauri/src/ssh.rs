use crate::error::{Error, Result};
use crate::model::{AuthKind, Server};
use russh::client::{self, Handle};
use russh::keys::{load_secret_key, HashAlg, PrivateKeyWithHashAlg, PublicKeyOrCertificate};
use russh::{ChannelMsg, Disconnect};
use russh_sftp::client::SftpSession;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

pub enum Cmd {
    Write(Vec<u8>),
    Resize(u32, u32),
    Close,
}

pub struct Session {
    pub handle: Handle<Client>,
    pub tx: mpsc::UnboundedSender<Cmd>,
    pub sftp: tokio::sync::Mutex<Option<Arc<SftpSession>>>,
}

#[derive(Default)]
pub struct Sessions(pub Mutex<HashMap<String, Arc<Session>>>);

impl Sessions {
    pub fn get(&self, id: &str) -> Result<Arc<Session>> {
        self.0.lock().unwrap().get(id).cloned().ok_or_else(|| "Session introuvable ou fermée".into())
    }
}

#[derive(Clone, Default)]
pub struct HostKeyState {
    pub seen: Arc<Mutex<Option<String>>>,
    pub mismatch: Arc<Mutex<bool>>,
}

pub struct Client {
    expected: Option<String>,
    state: HostKeyState,
}

impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(&mut self, key: &PublicKeyOrCertificate) -> std::result::Result<bool, Self::Error> {
        let fp = match key {
            PublicKeyOrCertificate::PublicKey { key, .. } => key.fingerprint(HashAlg::Sha256).to_string(),
            PublicKeyOrCertificate::Certificate(c) => c.public_key().fingerprint(HashAlg::Sha256).to_string(),
        };
        *self.state.seen.lock().unwrap() = Some(fp.clone());
        match &self.expected {
            Some(exp) if *exp != fp => {
                *self.state.mismatch.lock().unwrap() = true;
                Ok(false)
            }
            _ => Ok(true),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Closed {
    pub id: String,
    pub clean: bool,
}

#[derive(Serialize, Clone)]
pub struct Phase {
    pub attempt: String,
    pub phase: String,
}

#[derive(Serialize)]
pub struct Connected {
    pub session_id: String,
    pub fingerprint: String,
    pub new_host: bool,
}

pub struct ConnectInfo {
    pub handle: Handle<Client>,
    pub fingerprint: String,
    pub new_host: bool,
}

/// Ouvre la connexion TCP + SSH et authentifie. Ne crée pas de shell.
pub async fn open(
    server: &Server,
    expected_fp: Option<String>,
    keepalive: u32,
    timeout: u32,
    phase: &(dyn Fn(&str) + Send + Sync),
) -> Result<ConnectInfo> {
    let cfg = Arc::new(client::Config {
        keepalive_interval: (keepalive > 0).then(|| Duration::from_secs(keepalive as u64)),
        keepalive_max: 4,
        inactivity_timeout: None,
        nodelay: true,
        ..Default::default()
    });
    let state = HostKeyState::default();
    let handler = Client { expected: expected_fp.clone(), state: state.clone() };
    let addr = (server.host.as_str(), server.port);
    let mut handle = match tokio::time::timeout(Duration::from_secs(timeout.max(3) as u64), client::connect(cfg, addr, handler)).await {
        Err(_) => return Err(format!("Délai dépassé en se connectant à {}:{}", server.host, server.port).into()),
        Ok(Err(e)) => {
            if *state.mismatch.lock().unwrap() {
                return Err(format!(
                    "HOST_KEY_CHANGED: l'empreinte de {} a changé (attendue {}, reçue {}). Connexion refusée.",
                    server.host,
                    expected_fp.unwrap_or_default(),
                    state.seen.lock().unwrap().clone().unwrap_or_default()
                )
                .into());
            }
            return Err(Error::from(format!("Connexion impossible : {e}")));
        }
        Ok(Ok(h)) => h,
    };

    phase("auth");
    let ok = match server.auth {
        AuthKind::Password => {
            let pw = server.password.clone().unwrap_or_default();
            handle.authenticate_password(server.username.clone(), pw).await?.success()
        }
        AuthKind::Key => {
            let path = server.key_path.clone().filter(|p| !p.is_empty()).ok_or("Aucune clé privée configurée")?;
            let pass = server.key_passphrase.clone().filter(|p| !p.is_empty());
            let key = load_secret_key(&path, pass.as_deref()).map_err(|e| Error::from(format!("Lecture de la clé impossible ({path}) : {e}")))?;
            let hash = handle.best_supported_rsa_hash().await.ok().flatten().flatten();
            handle.authenticate_publickey(server.username.clone(), PrivateKeyWithHashAlg::new(Arc::new(key), hash)).await?.success()
        }
    };
    if !ok {
        let _ = handle.disconnect(Disconnect::ByApplication, "", "fr").await;
        return Err("Authentification refusée (identifiants ou clé incorrects)".into());
    }
    let fingerprint = state.seen.lock().unwrap().clone().unwrap_or_default();
    Ok(ConnectInfo { handle, fingerprint, new_host: expected_fp.is_none() })
}

/// Démarre le shell interactif et la tâche qui relaie les données.
pub async fn start_shell(
    app: AppHandle,
    id: String,
    handle: Handle<Client>,
    cols: u32,
    rows: u32,
    startup: Option<String>,
    on_data: Channel<InvokeResponseBody>,
) -> Result<Arc<Session>> {
    let mut chan = handle.channel_open_session().await?;
    chan.request_pty(true, "xterm-256color", cols, rows, 0, 0, &[]).await?;
    chan.request_shell(true).await?;
    if let Some(cmd) = startup.filter(|c| !c.trim().is_empty()) {
        chan.data_bytes(format!("{cmd}\n").into_bytes()).await?;
    }
    let (tx, mut rx) = mpsc::unbounded_channel::<Cmd>();
    let session = Arc::new(Session { handle, tx, sftp: tokio::sync::Mutex::new(None) });
    let sid = id.clone();
    tokio::spawn(async move {
        // Vrai si le serveur a terminé le shell lui-même (commande `exit`) : pas une coupure réseau.
        let mut clean = false;
        loop {
            tokio::select! {
                msg = chan.wait() => match msg {
                    Some(ChannelMsg::Data { data }) => { let _ = on_data.send(InvokeResponseBody::Raw(data.to_vec())); }
                    Some(ChannelMsg::ExtendedData { data, .. }) => { let _ = on_data.send(InvokeResponseBody::Raw(data.to_vec())); }
                    Some(ChannelMsg::ExitStatus { .. }) => { clean = true; }
                    Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => break,
                    _ => {}
                },
                cmd = rx.recv() => match cmd {
                    Some(Cmd::Write(b)) => { if chan.data_bytes(b).await.is_err() { break; } }
                    Some(Cmd::Resize(c, r)) => { let _ = chan.window_change(c, r, 0, 0).await; }
                    Some(Cmd::Close) | None => { clean = true; let _ = chan.close().await; break; }
                }
            }
        }
        let _ = app.emit("ssh-closed", Closed { id: sid, clean });
    });
    Ok(session)
}

/// Détecte le système du serveur via /etc/os-release (canal exec séparé du shell).
/// Retourne (clé d'icône, nom lisible).
pub async fn detect_os(handle: &Handle<Client>) -> Option<(String, String)> {
    let chan = handle.channel_open_session().await.ok()?;
    chan.exec(
        true,
        "sh -c 'cat /etc/os-release 2>/dev/null || cat /usr/lib/os-release 2>/dev/null; echo UNAME=$(uname -s 2>/dev/null)'",
    )
    .await
    .ok()?;
    let mut chan = chan;
    let mut out = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(6), async {
        while let Some(msg) = chan.wait().await {
            match msg {
                ChannelMsg::Data { data } => out.extend_from_slice(&data),
                ChannelMsg::Eof | ChannelMsg::Close => break,
                _ => {}
            }
        }
    })
    .await;
    let _ = chan.close().await;
    parse_os(&String::from_utf8_lossy(&out))
}

const KNOWN: &[&str] = &[
    "debian", "ubuntu", "fedora", "arch", "centos", "rhel", "alpine", "opensuse", "linuxmint", "raspberrypi", "rocky",
    "alma", "kali", "manjaro", "gentoo", "nixos", "popos", "freebsd", "apple", "openwrt", "zorin", "elementary",
];

fn normalize(id: &str) -> Option<&'static str> {
    let id = id.trim().to_lowercase();
    let k = match id.as_str() {
        "arch" | "archlinux" | "endeavouros" | "garuda" | "artix" => "arch",
        "rhel" | "redhat" => "rhel",
        "rocky" => "rocky",
        "almalinux" | "alma" => "alma",
        "linuxmint" | "mint" => "linuxmint",
        "raspbian" => "raspberrypi",
        "pop" | "pop_os" => "popos",
        "kali" => "kali",
        "alpine" => "alpine",
        "nixos" => "nixos",
        "openwrt" => "openwrt",
        s if s.starts_with("opensuse") || s == "sles" || s == "suse" => "opensuse",
        s => return KNOWN.iter().copied().find(|k| *k == s),
    };
    Some(k)
}

pub fn parse_os(text: &str) -> Option<(String, String)> {
    let mut id = None;
    let mut like = Vec::new();
    let mut pretty = None;
    let mut uname = None;
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        let v = v.trim().trim_matches('"').trim_matches('\'');
        match k.trim() {
            "ID" => id = Some(v.to_string()),
            "ID_LIKE" => like = v.split_whitespace().map(String::from).collect(),
            "PRETTY_NAME" => pretty = Some(v.to_string()),
            "UNAME" => uname = Some(v.to_string()),
            _ => {}
        }
    }
    let key = id
        .as_deref()
        .and_then(normalize)
        .or_else(|| like.iter().find_map(|l| normalize(l)))
        .or_else(|| match uname.as_deref() {
            Some("Darwin") => Some("apple"),
            Some("FreeBSD") => Some("freebsd"),
            _ => None,
        })
        .or_else(|| (id.is_some() || uname.as_deref() == Some("Linux")).then_some("linux"))?;
    let name = pretty.or(id).or(uname).unwrap_or_else(|| key.to_string());
    Some((key.to_string(), name))
}

#[cfg(test)]
mod tests {
    use super::parse_os;
    #[test]
    fn detects_distros() {
        let deb = "PRETTY_NAME=\"Debian GNU/Linux 12 (bookworm)\"\nID=debian\nUNAME=Linux\n";
        assert_eq!(parse_os(deb).unwrap().0, "debian");
        let mint = "NAME=\"Linux Mint\"\nID=linuxmint\nID_LIKE=\"ubuntu debian\"\nUNAME=Linux";
        assert_eq!(parse_os(mint).unwrap().0, "linuxmint");
        let derived = "ID=weirdos\nID_LIKE=\"rhel centos fedora\"\nUNAME=Linux";
        assert_eq!(parse_os(derived).unwrap().0, "rhel");
        assert_eq!(parse_os("UNAME=Darwin\n").unwrap().0, "apple");
        assert_eq!(parse_os("ID=unknownos\nUNAME=Linux").unwrap().0, "linux");
        assert!(parse_os("").is_none());
    }
}
