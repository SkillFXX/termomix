//! Stockage local : un fichier unique chiffré (ChaCha20-Poly1305) avec une clé d'appareil.
use crate::error::{Error, Result};
use crate::model::*;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Store {
    dir: PathBuf,
    pub db: Mutex<Db>,
    key: [u8; 32],
}

fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn restrict(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

impl Store {
    pub fn open(dir: PathBuf) -> Result<Store> {
        std::fs::create_dir_all(&dir)?;
        let key_path = dir.join("device.key");
        let key: [u8; 32] = match std::fs::read(&key_path) {
            Ok(b) if b.len() == 32 => b.try_into().unwrap(),
            _ => {
                let k = ChaCha20Poly1305::generate_key(&mut OsRng);
                std::fs::write(&key_path, k)?;
                restrict(&key_path);
                k.into()
            }
        };
        let db_path = dir.join("termomix.db");
        let db = match std::fs::read(&db_path) {
            Ok(raw) => {
                if raw.len() < 12 {
                    return Err("Fichier de données corrompu".into());
                }
                let (nonce, ct) = raw.split_at(12);
                let plain = ChaCha20Poly1305::new(Key::from_slice(&key))
                    .decrypt(Nonce::from_slice(nonce), ct)
                    .map_err(|_| Error::from("Impossible de déchiffrer les données (clé d'appareil absente ou différente)"))?;
                serde_json::from_slice(&plain)?
            }
            Err(_) => Db::default(),
        };
        Ok(Store { dir, db: Mutex::new(db), key })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn save(&self) -> Result<()> {
        let db = self.db.lock().unwrap();
        let plain = serde_json::to_vec(&*db)?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ct = cipher.encrypt(&nonce, plain.as_ref()).map_err(|_| Error::from("Chiffrement impossible"))?;
        let mut out = nonce.to_vec();
        out.extend(ct);
        let p = self.dir.join("termomix.db");
        write_atomic(&p, &out)?;
        restrict(&p);
        Ok(())
    }

    /// Modifie la base puis la persiste.
    pub fn update<T>(&self, f: impl FnOnce(&mut Db) -> T) -> Result<T> {
        let out = f(&mut self.db.lock().unwrap());
        self.save()?;
        Ok(out)
    }
}
