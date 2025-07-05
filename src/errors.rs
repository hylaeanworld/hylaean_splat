use thiserror::Error;

#[derive(Error, Debug)]
pub enum HylaeanError {
    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },
    
    #[error("Unsupported format: {format}")]
    UnsupportedFormat { format: String },
    
    #[error("Conversion failed: {source_format} -> {target_format}")]
    ConversionFailed { source_format: String, target_format: String },
    
    #[error("Installation failed for tool: {tool}")]
    InstallationFailed { tool: String },
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sled::Error),
    
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    #[error("Invalid path: {path}")]
    InvalidPath { path: String },
    
    #[error("Tool execution failed: {tool} - {message}")]
    ToolExecutionFailed { tool: String, message: String },
    
    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

pub type Result<T> = std::result::Result<T, HylaeanError>;