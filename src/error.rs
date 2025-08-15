use thiserror::Error;

#[derive(Debug, Error)]
pub enum J2sError {
    #[error("File operation failed: {0}")]
    File(String),
    
    #[error("JSON parsing failed: {0}")]
    Json(String),
    
    #[error("Schema generation failed: {0}")]
    Schema(String),
    
    #[error("Invalid arguments: {0}")]
    Argument(String),
}

pub type Result<T> = std::result::Result<T, J2sError>;

impl From<std::io::Error> for J2sError {
    fn from(err: std::io::Error) -> Self {
        J2sError::File(err.to_string())
    }
}

impl From<serde_json::Error> for J2sError {
    fn from(err: serde_json::Error) -> Self {
        J2sError::Json(err.to_string())
    }
}