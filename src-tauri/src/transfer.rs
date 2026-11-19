//! Export / import portable, sans aucun cloud : un fichier `.termomix`.
use crate::error::{Error, Result};
use crate::model::*;
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use serde::{Deserialize, Serialize};

const MAGIC: &str = "termomix-export";

#[derive(Serialize, Deserialize)]
struct Envelope {
    format: String,
    version: u32,
    encrypted: bool,
    #[serde(default)]
    kdf: Option<Kdf>,
    #[serde(default)]
    nonce: Option<String>,
    /// Si chiffré : base64 du texte chiffré. Sinon : la base en clair.
    #[serde(default)]
    ciphertext: Option<String>,
    #[serde(default)]
    data: Option<Db>,
}

#[derive(Serialize, Deserialize)]
struct Kdf {
    algo: String,
    salt: String,
    m_kib: u32,
    t: u32,
    p: u32,
}

fn derive(pass: &str, salt: &[u8], m: u32, t: u32, p: u32) -> Result<[u8; 32]> {
    let params = Params::new(m, t, p, Some(32)).map_err(|e| Error(e.to_string()))?;
    let mut out = [0u8; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(pass.as_bytes(), salt, &mut out)
        .map_err(|e| Error(e.to_string()))?;
    Ok(out)
}

pub fn export(db: &Db, passphrase: Option<&str>, include_secrets: bool) -> Result<Vec<u8>> {
    let mut data = db.clone();
    if !include_secrets {
        for s in &mut data.servers {
            s.password = None;
            s.key_passphrase = None;
        }
    }
    let env = match passphrase.filter(|p| !p.is_empty()) {
        None => {
            if include_secrets {
                return Err("Une phrase secrète est obligatoire pour exporter les mots de passe".into());
            }
            Envelope { format: MAGIC.into(), version: 1, encrypted: false, kdf: None, nonce: None, ciphertext: None, data: Some(data) }
        }
        Some(pass) => {
            let mut salt = [0u8; 16];
            let nonce_salt = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            salt.copy_from_slice(&ChaCha20Poly1305::generate_key(&mut OsRng)[..16]);
            let (m, t, p) = (65536, 3, 1);
            let key = derive(pass, &salt, m, t, p)?;
            let ct = ChaCha20Poly1305::new(Key::from_slice(&key))
                .encrypt(&nonce_salt, serde_json::to_vec(&data)?.as_ref())
                .map_err(|_| Error::from("Chiffrement impossible"))?;
            Envelope {
                format: MAGIC.into(),
                version: 1,
                encrypted: true,
                kdf: Some(Kdf { algo: "argon2id".into(), salt: B64.encode(salt), m_kib: m, t, p }),
                nonce: Some(B64.encode(nonce_salt)),
                ciphertext: Some(B64.encode(ct)),
                data: None,
            }
        }
    };
    Ok(serde_json::to_vec_pretty(&env)?)
}

pub fn read_envelope(bytes: &[u8], passphrase: Option<&str>) -> Result<Db> {
    let env: Envelope = serde_json::from_slice(bytes).map_err(|_| Error::from("Ce fichier n'est pas un export Termomix valide"))?;
    if env.format != MAGIC {
        return Err("Ce fichier n'est pas un export Termomix valide".into());
    }
    if !env.encrypted {
        return env.data.ok_or_else(|| "Export vide".into());
    }
    let pass = passphrase.filter(|p| !p.is_empty()).ok_or_else(|| Error::from("PASSPHRASE_REQUIRED"))?;
    let kdf = env.kdf.ok_or("Export invalide")?;
    let salt = B64.decode(kdf.salt)?;
    let key = derive(pass, &salt, kdf.m_kib, kdf.t, kdf.p)?;
    let nonce = B64.decode(env.nonce.ok_or("Export invalide")?)?;
    if nonce.len() != 12 {
        return Err("Export invalide".into());
    }
    let ct = B64.decode(env.ciphertext.ok_or("Export invalide")?)?;
    let plain = ChaCha20Poly1305::new(Key::from_slice(&key))
        .decrypt(Nonce::from_slice(&nonce), ct.as_ref())
        .map_err(|_| Error::from("Phrase secrète incorrecte ou fichier altéré"))?;
    Ok(serde_json::from_slice(&plain)?)
}

/// Fusionne `inc` dans `db` (par identifiant). Retourne (serveurs, macros) ajoutés ou mis à jour.
pub fn merge(db: &mut Db, inc: Db) -> (usize, usize) {
    fn upsert<T>(list: &mut Vec<T>, item: T, id: impl Fn(&T) -> &str) {
        match list.iter().position(|x| id(x) == id(&item)) {
            Some(i) => list[i] = item,
            None => list.push(item),
        }
    }
    let (ns, nm) = (inc.servers.len(), inc.macros.len());
    for g in inc.groups {
        upsert(&mut db.groups, g, |x| &x.id);
    }
    for mut s in inc.servers {
        // Un export sans secrets ne doit pas effacer les secrets déjà présents.
        if let Some(old) = db.servers.iter().find(|o| o.id == s.id) {
            if s.password.is_none() {
                s.password = old.password.clone();
            }
            if s.key_passphrase.is_none() {
                s.key_passphrase = old.key_passphrase.clone();
            }
        }
        upsert(&mut db.servers, s, |x| &x.id);
    }
    for m in inc.macros {
        upsert(&mut db.macros, m, |x| &x.id);
    }
    for (k, v) in inc.known_hosts {
        db.known_hosts.entry(k).or_insert(v);
    }
    (ns, nm)
}
