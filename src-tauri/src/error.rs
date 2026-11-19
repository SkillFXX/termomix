use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);

pub type Result<T> = std::result::Result<T, Error>;

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

macro_rules! from_err {
    ($($t:ty),*) => {$(
        impl From<$t> for Error {
            fn from(e: $t) -> Self { Error(e.to_string()) }
        }
    )*};
}

from_err!(
    std::io::Error,
    serde_json::Error,
    russh::Error,
    russh::keys::Error,
    russh_sftp::client::error::Error,
    base64::DecodeError,
    tauri::Error
);

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error(e.to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error(e)
    }
}
