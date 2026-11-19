mod error;
mod model;
mod sftp;
mod ssh;
mod store;
mod transfer;

use error::Result;
use model::*;
use serde::{Deserialize, Serialize};
use ssh::{Cmd, Connected, Sessions};
use std::path::PathBuf;
use store::Store;
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, Manager, State};

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ---------- État de l'application ----------

#[derive(Serialize)]
struct AppState {
    groups: Vec<Group>,
    servers: Vec<Server>,
    macros: Vec<Macro>,
    settings: Settings,
    data_dir: String,
    version: String,
}

fn snapshot(store: &Store) -> AppState {
    let db = store.db.lock().unwrap();
    AppState {
        groups: db.groups.clone(),
        servers: db.servers.iter().map(Server::sanitized).collect(),
        macros: db.macros.clone(),
        settings: db.settings.clone(),
        data_dir: store.dir().to_string_lossy().into_owned(),
        version: env!("CARGO_PKG_VERSION").into(),
    }
}

#[tauri::command]
fn get_state(store: State<Store>) -> AppState {
    snapshot(&store)
}

#[tauri::command]
fn save_server(store: State<Store>, mut server: Server) -> Result<Server> {
    if server.host.trim().is_empty() || server.username.trim().is_empty() {
        return Err("L'hôte et l'utilisateur sont obligatoires".into());
    }
    if server.id.is_empty() {
        server.id = new_id();
    }
    server.host = server.host.trim().to_string();
    if server.name.trim().is_empty() {
        server.name = server.host.clone();
    }
    store.update(|db| {
        if let Some(old) = db.servers.iter().find(|s| s.id == server.id) {
            if server.password.is_none() {
                server.password = old.password.clone();
            }
            if server.key_passphrase.is_none() {
                server.key_passphrase = old.key_passphrase.clone();
            }
            server.last_connected = old.last_connected;
        }
        server.password = server.password.take().filter(|p| !p.is_empty());
        server.key_passphrase = server.key_passphrase.take().filter(|p| !p.is_empty());
        match db.servers.iter().position(|s| s.id == server.id) {
            Some(i) => db.servers[i] = server.clone(),
            None => db.servers.push(server.clone()),
        }
        server.sanitized()
    })
}

#[tauri::command]
fn delete_server(store: State<Store>, id: String) -> Result<()> {
    store.update(|db| {
        db.servers.retain(|s| s.id != id);
        for m in &mut db.macros {
            m.server_ids.retain(|s| *s != id);
        }
    })
}

#[tauri::command]
fn save_group(store: State<Store>, mut group: Group) -> Result<Group> {
    if group.name.trim().is_empty() {
        return Err("Le nom du groupe est obligatoire".into());
    }
    if group.id.is_empty() {
        group.id = new_id();
    }
    store.update(|db| {
        match db.groups.iter().position(|g| g.id == group.id) {
            Some(i) => db.groups[i] = group.clone(),
            None => db.groups.push(group.clone()),
        }
        group
    })
}

#[tauri::command]
fn delete_group(store: State<Store>, id: String) -> Result<()> {
    store.update(|db| {
        db.groups.retain(|g| g.id != id);
        for s in &mut db.servers {
            if s.group_id.as_deref() == Some(&id) {
                s.group_id = None;
            }
        }
    })
}

#[tauri::command]
fn save_macro(store: State<Store>, mut m: Macro) -> Result<Macro> {
    if m.name.trim().is_empty() {
        return Err("Le nom de la macro est obligatoire".into());
    }
    m.commands.retain(|c| !c.trim().is_empty());
    if m.commands.is_empty() {
        return Err("Une macro doit contenir au moins une commande".into());
    }
    if m.id.is_empty() {
        m.id = new_id();
    }
    store.update(|db| {
        match db.macros.iter().position(|x| x.id == m.id) {
            Some(i) => db.macros[i] = m.clone(),
            None => db.macros.push(m.clone()),
        }
        m
    })
}

#[tauri::command]
fn delete_macro(store: State<Store>, id: String) -> Result<()> {
    store.update(|db| db.macros.retain(|m| m.id != id))
}

#[tauri::command]
fn save_settings(store: State<Store>, settings: Settings) -> Result<()> {
    store.update(|db| db.settings = settings)
}

#[tauri::command]
fn forget_host_key(store: State<Store>, host: String, port: u16) -> Result<()> {
    store.update(|db| {
        db.known_hosts.remove(&format!("{host}:{port}"));
    })
}

// ---------- SSH ----------

#[derive(Deserialize)]
struct ConnectReq {
    server_id: Option<String>,
    /// Connexion rapide : serveur non enregistré.
    adhoc: Option<Server>,
    /// Mot de passe (ou phrase de passe de la clé) saisi à la volée.
    secret: Option<String>,
    cols: u32,
    rows: u32,
}

#[tauri::command]
async fn ssh_connect(
    app: AppHandle,
    store: State<'_, Store>,
    sessions: State<'_, Sessions>,
    req: ConnectReq,
    on_data: Channel<InvokeResponseBody>,
) -> Result<Connected> {
    let mut server = match (&req.server_id, req.adhoc) {
        (Some(id), _) => store
            .db
            .lock()
            .unwrap()
            .servers
            .iter()
            .find(|s| s.id == *id)
            .cloned()
            .ok_or("Serveur introuvable")?,
        (None, Some(s)) => s,
        _ => return Err("Aucun serveur indiqué".into()),
    };
    if let Some(secret) = req.secret.filter(|s| !s.is_empty()) {
        match server.auth {
            AuthKind::Password => server.password = Some(secret),
            AuthKind::Key => server.key_passphrase = Some(secret),
        }
    }
    let host_id = format!("{}:{}", server.host, server.port);
    let expected = store.db.lock().unwrap().known_hosts.get(&host_id).cloned();
    let (ka, to) = {
        let db = store.db.lock().unwrap();
        (db.settings.keepalive, db.settings.connect_timeout)
    };
    let info = ssh::open(&server, expected, ka, to).await?;

    let id = new_id();
    let startup = server.startup_command.clone();
    let app2 = app.clone();
    let session = ssh::start_shell(app, id.clone(), info.handle, req.cols, req.rows, startup, on_data).await?;
    sessions.0.lock().unwrap().insert(id.clone(), session.clone());

    // Détection du système en arrière-plan (ne retarde pas l'ouverture du terminal).
    if let Some(sid) = req.server_id.clone() {
        tokio::spawn(async move {
            let Some((os, name)) = ssh::detect_os(&session.handle).await else { return };
            let store = app2.state::<Store>();
            let changed = store
                .update(|db| {
                    let Some(s) = db.servers.iter_mut().find(|s| s.id == sid) else { return false };
                    let changed = s.os.as_deref() != Some(&os) || s.os_name.as_deref() != Some(&name);
                    s.os = Some(os);
                    s.os_name = Some(name);
                    changed
                })
                .unwrap_or(false);
            if changed {
                let _ = tauri::Emitter::emit(&app2, "servers-changed", ());
            }
        });
    }

    let _ = store.update(|db| {
        if info.new_host {
            db.known_hosts.insert(host_id, info.fingerprint.clone());
        }
        if let Some(s) = db.servers.iter_mut().find(|s| Some(&s.id) == req.server_id.as_ref()) {
            s.last_connected = Some(now());
        }
    });
    Ok(Connected { session_id: id, fingerprint: info.fingerprint, new_host: info.new_host })
}

#[tauri::command]
fn ssh_write(sessions: State<Sessions>, id: String, data: String) -> Result<()> {
    let s = sessions.get(&id)?;
    s.tx.send(Cmd::Write(data.into_bytes())).map_err(|_| "Session fermée".into())
}

#[tauri::command]
fn ssh_resize(sessions: State<Sessions>, id: String, cols: u32, rows: u32) -> Result<()> {
    let s = sessions.get(&id)?;
    let _ = s.tx.send(Cmd::Resize(cols, rows));
    Ok(())
}

#[tauri::command]
async fn ssh_close(sessions: State<'_, Sessions>, id: String) -> Result<()> {
    let removed = sessions.0.lock().unwrap().remove(&id);
    if let Some(s) = removed {
        let _ = s.tx.send(Cmd::Close);
        let _ = s.handle.disconnect(russh::Disconnect::ByApplication, "", "fr").await;
    }
    Ok(())
}

// ---------- SFTP ----------

#[tauri::command]
async fn sftp_list(sessions: State<'_, Sessions>, id: String, path: String) -> Result<sftp::Listing> {
    let s = sessions.get(&id)?;
    let c = sftp::get(&s).await?;
    sftp::list(&c, &path).await
}

#[tauri::command]
async fn sftp_mkdir(sessions: State<'_, Sessions>, id: String, path: String) -> Result<()> {
    let c = sftp::get(&sessions.get(&id)?).await?;
    Ok(c.create_dir(path).await?)
}

#[tauri::command]
async fn sftp_rename(sessions: State<'_, Sessions>, id: String, from: String, to: String) -> Result<()> {
    let c = sftp::get(&sessions.get(&id)?).await?;
    Ok(c.rename(from, to).await?)
}

#[tauri::command]
async fn sftp_remove(sessions: State<'_, Sessions>, id: String, path: String, is_dir: bool) -> Result<()> {
    let c = sftp::get(&sessions.get(&id)?).await?;
    sftp::remove(&c, &path, is_dir).await
}

#[tauri::command]
async fn sftp_read_text(sessions: State<'_, Sessions>, id: String, path: String) -> Result<String> {
    let c = sftp::get(&sessions.get(&id)?).await?;
    sftp::read_text(&c, &path).await
}

#[tauri::command]
async fn sftp_write_text(sessions: State<'_, Sessions>, id: String, path: String, content: String) -> Result<()> {
    let c = sftp::get(&sessions.get(&id)?).await?;
    sftp::write_text(&c, &path, &content).await
}

#[tauri::command]
async fn sftp_upload(app: AppHandle, sessions: State<'_, Sessions>, id: String, local: String, remote_dir: String) -> Result<()> {
    let c = sftp::get(&sessions.get(&id)?).await?;
    sftp::upload(&c, app, new_id(), &PathBuf::from(local), &remote_dir).await
}

#[tauri::command]
async fn sftp_download(app: AppHandle, sessions: State<'_, Sessions>, id: String, remote: String, is_dir: bool, local_dir: String) -> Result<()> {
    let c = sftp::get(&sessions.get(&id)?).await?;
    sftp::download(&c, app, new_id(), &remote, is_dir, &PathBuf::from(local_dir)).await
}

// ---------- Export / import ----------

#[tauri::command]
fn export_data(store: State<Store>, path: String, passphrase: Option<String>, include_secrets: bool) -> Result<()> {
    let bytes = {
        let db = store.db.lock().unwrap();
        transfer::export(&db, passphrase.as_deref(), include_secrets)?
    };
    std::fs::write(path, bytes)?;
    Ok(())
}

#[derive(Serialize)]
struct ImportResult {
    servers: usize,
    macros: usize,
}

#[tauri::command]
fn import_data(store: State<Store>, path: String, passphrase: Option<String>, replace: bool) -> Result<ImportResult> {
    let bytes = std::fs::read(path)?;
    let inc = transfer::read_envelope(&bytes, passphrase.as_deref())?;
    store.update(|db| {
        if replace {
            let (s, m) = (inc.servers.len(), inc.macros.len());
            let known = std::mem::take(&mut db.known_hosts);
            *db = inc;
            for (k, v) in known {
                db.known_hosts.entry(k).or_insert(v);
            }
            ImportResult { servers: s, macros: m }
        } else {
            let (servers, macros) = transfer::merge(db, inc);
            ImportResult { servers, macros }
        }
    })
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Sessions::default())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            app.manage(Store::open(dir)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_server,
            delete_server,
            save_group,
            delete_group,
            save_macro,
            delete_macro,
            save_settings,
            forget_host_key,
            ssh_connect,
            ssh_write,
            ssh_resize,
            ssh_close,
            sftp_list,
            sftp_mkdir,
            sftp_rename,
            sftp_remove,
            sftp_read_text,
            sftp_write_text,
            sftp_upload,
            sftp_download,
            export_data,
            import_data,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de Termomix");
}
