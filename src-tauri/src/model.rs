use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthKind {
    #[default]
    Password,
    Key,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub auth: AuthKind,
    /// Jamais renvoyé au frontend. `None` à l'enregistrement = conserver l'existant, `Some("")` = effacer.
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub has_password: bool,
    /// Chemin d'une clé privée sur cette machine.
    #[serde(default)]
    pub key_path: Option<String>,
    #[serde(default)]
    pub key_passphrase: Option<String>,
    #[serde(default)]
    pub has_passphrase: bool,
    #[serde(default)]
    pub startup_command: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub last_connected: Option<u64>,
}

fn default_port() -> u16 {
    22
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Group {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Macro {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub commands: Vec<String>,
    /// Délai entre deux commandes, en millisecondes.
    #[serde(default = "default_delay")]
    pub delay_ms: u32,
    /// Vide = disponible sur tous les serveurs.
    #[serde(default)]
    pub server_ids: Vec<String>,
}

fn default_delay() -> u32 {
    150
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: String,
    pub splash: bool,
    pub font_size: u32,
    pub line_height: f32,
    pub scrollback: u32,
    pub cursor_style: String,
    pub cursor_blink: bool,
    pub copy_on_select: bool,
    pub auto_reconnect: bool,
    pub reconnect_attempts: u32,
    pub reconnect_delay: u32,
    /// 0 = désactivé
    pub keepalive: u32,
    pub connect_timeout: u32,
    pub confirm_close: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: "fr".into(),
            splash: true,
            font_size: 14,
            line_height: 1.15,
            scrollback: 10000,
            cursor_style: "bar".into(),
            cursor_blink: true,
            copy_on_select: false,
            auto_reconnect: true,
            reconnect_attempts: 5,
            reconnect_delay: 3,
            keepalive: 30,
            connect_timeout: 15,
            confirm_close: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Db {
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub servers: Vec<Server>,
    #[serde(default)]
    pub macros: Vec<Macro>,
    #[serde(default)]
    pub settings: Settings,
    /// "hôte:port" -> empreinte SHA256
    #[serde(default)]
    pub known_hosts: HashMap<String, String>,
}

impl Server {
    pub fn sanitized(&self) -> Server {
        let mut s = self.clone();
        s.has_password = self.password.as_deref().is_some_and(|p| !p.is_empty());
        s.has_passphrase = self.key_passphrase.as_deref().is_some_and(|p| !p.is_empty());
        s.password = None;
        s.key_passphrase = None;
        s
    }
}
