use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum ConfigError {
    #[error("Shortcut conflict with \"{0}\"")]
    Conflict(String),
    #[error("Shortcut registration failed for \"{0}\"")]
    RegistrationFailed(String),
    #[error("Shortcut not found")]
    NotFound,
    #[error("Invalid shortcut format")]
    InvalidFormat,
    #[error("Empty shortcut")]
    Empty,
    #[error("IO Error: {0}")]
    Io(String),
    #[error("Error: {0}")]
    Other(String),
}
