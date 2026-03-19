use std::fmt;
use std::io;

/// Unified error type for the kanban application
#[derive(Debug)]
pub enum KanbanError {
    /// IO errors (file operations, etc.)
    Io(io::Error),

    /// Configuration parsing errors
    ConfigParse(String),

    /// Task parsing errors
    TaskParse(String),

    /// Validation errors
    Validation(String),

    /// Not found errors (project, task, status, etc.)
    NotFound(String),

    /// Already exists errors
    AlreadyExists(String),

    /// Invalid operation errors
    InvalidOperation(String),

    /// Serialization/deserialization errors
    Serialization(String),
}

impl fmt::Display for KanbanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KanbanError::Io(err) => write!(f, "IO error: {}", err),
            KanbanError::ConfigParse(msg) => write!(f, "Config parse error: {}", msg),
            KanbanError::TaskParse(msg) => write!(f, "Task parse error: {}", msg),
            KanbanError::Validation(msg) => write!(f, "Validation error: {}", msg),
            KanbanError::NotFound(msg) => write!(f, "Not found: {}", msg),
            KanbanError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            KanbanError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            KanbanError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for KanbanError {}

impl From<io::Error> for KanbanError {
    fn from(err: io::Error) -> Self {
        KanbanError::Io(err)
    }
}

impl From<toml::de::Error> for KanbanError {
    fn from(err: toml::de::Error) -> Self {
        KanbanError::ConfigParse(err.to_string())
    }
}

impl From<toml::ser::Error> for KanbanError {
    fn from(err: toml::ser::Error) -> Self {
        KanbanError::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for KanbanError {
    fn from(err: serde_json::Error) -> Self {
        KanbanError::Serialization(err.to_string())
    }
}

// Conversion from String for backward compatibility during migration
impl From<String> for KanbanError {
    fn from(msg: String) -> Self {
        KanbanError::InvalidOperation(msg)
    }
}

impl From<&str> for KanbanError {
    fn from(msg: &str) -> Self {
        KanbanError::InvalidOperation(msg.to_string())
    }
}

// Conversion to String for backward compatibility
impl From<KanbanError> for String {
    fn from(err: KanbanError) -> Self {
        err.to_string()
    }
}

pub type Result<T> = std::result::Result<T, KanbanError>;
