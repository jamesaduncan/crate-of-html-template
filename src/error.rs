use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Render error: {0}")]
    RenderError(String),
    
    #[error("Selector error: {0}")]
    SelectorError(String),
    
    #[error("Constraint error: {0}")]
    ConstraintError(String),
    
    #[error("DOM error: {0}")]
    DomError(String),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = Error::ParseError("Failed to parse template".to_string());
        assert_eq!(err.to_string(), "Parse error: Failed to parse template");
        
        let err = Error::RenderError("Failed to render".to_string());
        assert_eq!(err.to_string(), "Render error: Failed to render");
        
        let err = Error::DomError("DOM manipulation failed".to_string());
        assert_eq!(err.to_string(), "DOM error: DOM manipulation failed");
    }
    
    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::JsonError(_)));
    }
    
    #[test]
    fn test_error_from_io() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::IoError(_)));
    }
}