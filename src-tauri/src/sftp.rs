use crate::error::Result;
use crate::ssh::Session;
use russh_sftp::client::SftpSession;
use russh_sftp::protocol::{FileType, OpenFlags};
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn get(session: &Arc<Session>) -> Result<Arc<SftpSession>> {
    let mut slot = session.sftp.lock().await;
    if let Some(s) = slot.as_ref() {
        return Ok(s.clone());
    }
    let chan = session.handle.channel_open_session().await?;
    chan.request_subsystem(true, "sftp").await?;
    let sftp = Arc::new(SftpSession::new(chan.into_stream()).await?);
    *slot = Some(sftp.clone());
    Ok(sftp)
}

#[derive(Serialize)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub mtime: Option<u32>,
    pub mode: Option<u32>,
}

#[derive(Serialize)]
pub struct Listing {
    pub path: String,
    pub entries: Vec<Entry>,
}

pub async fn list(sftp: &SftpSession, path: &str) -> Result<Listing> {
    let path = sftp.canonicalize(if path.is_empty() { "." } else { path }).await?;
    let mut entries = Vec::new();
    for e in sftp.read_dir(path.clone()).await? {
        let name = e.file_name();
        if name == "." || name == ".." {
            continue;
        }
        let md = e.metadata();
        let mut ft = e.file_type();
        let is_symlink = ft == FileType::Symlink;
        if is_symlink {
            // Suivre le lien pour savoir s'il pointe vers un dossier.
            if let Ok(t) = sftp.metadata(e.path()).await {
                ft = t.file_type();
            }
        }
        entries.push(Entry {
            name,
            is_dir: ft == FileType::Dir,
            is_symlink,
            size: md.size.unwrap_or(0),
            mtime: md.mtime,
            mode: md.permissions.map(|p| p & 0o7777),
        });
    }
    Ok(Listing { path, entries })
}

#[derive(Serialize, Clone)]
pub struct Progress {
    pub id: String,
    pub name: String,
    pub done: u64,
    pub total: u64,
    /// "running" | "done" | "error"
    pub state: String,
    pub error: Option<String>,
}

pub struct Tracker {
    app: AppHandle,
    id: String,
    name: String,
    total: u64,
    done: u64,
    last: Instant,
}

impl Tracker {
    pub fn new(app: AppHandle, id: String, name: String, total: u64) -> Self {
        let t = Tracker { app, id, name, total, done: 0, last: Instant::now() - Duration::from_secs(1) };
        t.emit("running", None);
        t
    }
    fn emit(&self, state: &str, error: Option<String>) {
        let _ = self.app.emit(
            "transfer",
            Progress { id: self.id.clone(), name: self.name.clone(), done: self.done, total: self.total, state: state.into(), error },
        );
    }
    fn add(&mut self, n: u64) {
        self.done += n;
        if self.last.elapsed() > Duration::from_millis(120) {
            self.last = Instant::now();
            self.emit("running", None);
        }
    }
    pub fn finish(&self, res: &Result<()>) {
        match res {
            Ok(()) => self.emit("done", None),
            Err(e) => self.emit("error", Some(e.0.clone())),
        }
    }
}

const CHUNK: usize = 128 * 1024;

async fn copy_down(sftp: &SftpSession, remote: &str, local: &Path, t: &mut Tracker) -> Result<()> {
    let mut src = sftp.open(remote).await?;
    let mut dst = tokio::fs::File::create(local).await?;
    let mut buf = vec![0u8; CHUNK];
    loop {
        let n = src.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        dst.write_all(&buf[..n]).await?;
        t.add(n as u64);
    }
    dst.flush().await?;
    Ok(())
}

async fn copy_up(sftp: &SftpSession, local: &Path, remote: &str, t: &mut Tracker) -> Result<()> {
    let mut src = tokio::fs::File::open(local).await?;
    let mut dst = sftp.create(remote).await?;
    let mut buf = vec![0u8; CHUNK];
    loop {
        let n = src.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        dst.write_all(&buf[..n]).await?;
        t.add(n as u64);
    }
    dst.shutdown().await?;
    Ok(())
}

fn join(base: &str, name: &str) -> String {
    if base.ends_with('/') { format!("{base}{name}") } else { format!("{base}/{name}") }
}

/// Taille totale d'un arbre local (pour la progression).
fn local_size(p: &Path) -> u64 {
    let Ok(md) = std::fs::metadata(p) else { return 0 };
    if md.is_file() {
        return md.len();
    }
    std::fs::read_dir(p).map(|rd| rd.flatten().map(|e| local_size(&e.path())).sum()).unwrap_or(0)
}

/// Téléverse un fichier ou un dossier (récursif) vers `remote_dir`.
pub async fn upload(sftp: &SftpSession, app: AppHandle, id: String, local: &Path, remote_dir: &str) -> Result<()> {
    let name = local.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "fichier".into());
    let total = tokio::task::spawn_blocking({
        let l = local.to_path_buf();
        move || local_size(&l)
    })
    .await
    .unwrap_or(0);
    let mut t = Tracker::new(app, id, name.clone(), total);
    let res = async {
        let mut stack = vec![(local.to_path_buf(), join(remote_dir, &name))];
        while let Some((lp, rp)) = stack.pop() {
            if tokio::fs::metadata(&lp).await?.is_dir() {
                let _ = sftp.create_dir(rp.clone()).await; // existe peut-être déjà
                let mut rd = tokio::fs::read_dir(&lp).await?;
                while let Some(e) = rd.next_entry().await? {
                    stack.push((e.path(), join(&rp, &e.file_name().to_string_lossy())));
                }
            } else {
                copy_up(sftp, &lp, &rp, &mut t).await?;
            }
        }
        Ok(())
    }
    .await;
    t.finish(&res);
    res
}

async fn remote_size(sftp: &SftpSession, path: &str, is_dir: bool) -> u64 {
    if !is_dir {
        return sftp.metadata(path).await.map(|m| m.len()).unwrap_or(0);
    }
    let mut total = 0;
    let mut stack = vec![path.to_string()];
    while let Some(d) = stack.pop() {
        if let Ok(rd) = sftp.read_dir(d.clone()).await {
            for e in rd {
                let n = e.file_name();
                if n == "." || n == ".." {
                    continue;
                }
                if e.file_type() == FileType::Dir {
                    stack.push(e.path());
                } else {
                    total += e.metadata().len();
                }
            }
        }
    }
    total
}

/// Télécharge un fichier ou dossier (récursif) distant dans `local_dir`.
pub async fn download(sftp: &SftpSession, app: AppHandle, id: String, remote: &str, is_dir: bool, local_dir: &Path) -> Result<()> {
    let name = remote.trim_end_matches('/').rsplit('/').next().unwrap_or("fichier").to_string();
    let total = remote_size(sftp, remote, is_dir).await;
    let mut t = Tracker::new(app, id, name.clone(), total);
    let res = async {
        let mut stack = vec![(remote.to_string(), local_dir.join(&name), is_dir)];
        while let Some((rp, lp, dir)) = stack.pop() {
            if dir {
                tokio::fs::create_dir_all(&lp).await?;
                for e in sftp.read_dir(rp.clone()).await? {
                    let n = e.file_name();
                    if n == "." || n == ".." {
                        continue;
                    }
                    stack.push((e.path(), lp.join(&n), e.file_type() == FileType::Dir));
                }
            } else {
                copy_down(sftp, &rp, &lp, &mut t).await?;
            }
        }
        Ok(())
    }
    .await;
    t.finish(&res);
    res
}

/// Suppression récursive.
pub async fn remove(sftp: &SftpSession, path: &str, is_dir: bool) -> Result<()> {
    if !is_dir {
        return Ok(sftp.remove_file(path).await?);
    }
    // Parcours en profondeur : on empile les dossiers pour les supprimer une fois vides.
    let mut order = Vec::new();
    let mut stack = vec![path.to_string()];
    while let Some(d) = stack.pop() {
        for e in sftp.read_dir(d.clone()).await? {
            let n = e.file_name();
            if n == "." || n == ".." {
                continue;
            }
            if e.file_type() == FileType::Dir {
                stack.push(e.path());
            } else {
                sftp.remove_file(e.path()).await?;
            }
        }
        order.push(d);
    }
    for d in order.into_iter().rev() {
        sftp.remove_dir(d).await?;
    }
    Ok(())
}

/// Limite d'édition en ligne (les gros fichiers se téléchargent).
pub const EDIT_MAX: u64 = 5 * 1024 * 1024;

pub async fn read_text(sftp: &SftpSession, path: &str) -> Result<String> {
    let size = sftp.metadata(path).await?.len();
    if size > EDIT_MAX {
        return Err(format!("FILE_TOO_LARGE:{size}").into());
    }
    let bytes = sftp.read(path).await?;
    String::from_utf8(bytes).map_err(|_| "NOT_TEXT".into())
}

pub async fn write_text(sftp: &SftpSession, path: &str, content: &str) -> Result<()> {
    // Le fichier existe déjà : ses permissions sont conservées, seul le contenu est remplacé.
    let mut f = sftp.open_with_flags(path, OpenFlags::WRITE | OpenFlags::TRUNCATE | OpenFlags::CREATE).await?;
    f.write_all(content.as_bytes()).await?;
    f.shutdown().await?;
    Ok(())
}
