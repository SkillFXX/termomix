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
pub async fn open(server: &Server, expected_fp: Option<String>) -> Result<ConnectInfo> {
    let cfg = Arc::new(client::Config {
        keepalive_interval: Some(Duration::from_secs(30)),
        keepalive_max: 4,
        inactivity_timeout: None,
        nodelay: true,
        ..Default::default()
    });
    let state = HostKeyState::default();
    let handler = Client { expected: expected_fp.clone(), state: state.clone() };
    let addr = (server.host.as_str(), server.port);
    let mut handle = match tokio::time::timeout(Duration::from_secs(15), client::connect(cfg, addr, handler)).await {
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
        loop {
            tokio::select! {
                msg = chan.wait() => match msg {
                    Some(ChannelMsg::Data { data }) => { let _ = on_data.send(InvokeResponseBody::Raw(data.to_vec())); }
                    Some(ChannelMsg::ExtendedData { data, .. }) => { let _ = on_data.send(InvokeResponseBody::Raw(data.to_vec())); }
                    Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => break,
                    _ => {}
                },
                cmd = rx.recv() => match cmd {
                    Some(Cmd::Write(b)) => { if chan.data_bytes(b).await.is_err() { break; } }
                    Some(Cmd::Resize(c, r)) => { let _ = chan.window_change(c, r, 0, 0).await; }
                    Some(Cmd::Close) | None => { let _ = chan.close().await; break; }
                }
            }
        }
        let _ = app.emit("ssh-closed", sid);
    });
    Ok(session)
}
