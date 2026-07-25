use std::fmt;
use std::io;

/// Application-wide error type.
#[derive(Debug)]
pub enum AppError {
    /// File I/O error
    Io(io::Error),
    /// Serialization/deserialization error
    Serde(serde_json::Error),
    /// ImGui/Dear ImGui runtime error
    #[allow(dead_code)]
    ImGui(String),
    /// Graph validation error
    #[allow(dead_code)]
    Graph(String),
    /// Window/GPU initialization error
    Init(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "I/O error: {e}"),
            AppError::Serde(e) => write!(f, "Serialization error: {e}"),
            AppError::ImGui(msg) => write!(f, "ImGui error: {msg}"),
            AppError::Graph(msg) => write!(f, "Graph error: {msg}"),
            AppError::Init(msg) => write!(f, "Initialization error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            AppError::Serde(e) => Some(e),
            _ => None,
        }
    }
}

// From impls for ergonomic ? operator usage
impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serde(e)
    }
}
