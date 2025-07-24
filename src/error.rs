use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse error: {0}")]
    ParseError(Cow<'static, str>),
    
    #[error("Render error: {0}")]
    RenderError(Cow<'static, str>),
    
    #[error("Selector error: {0}")]
    SelectorError(Cow<'static, str>),
    
    #[error("Constraint error: {0}")]
    ConstraintError(Cow<'static, str>),
    
    #[error("DOM error: {0}")]
    DomError(Cow<'static, str>),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Error {
    /// Create a parse error with a static string
    pub fn parse_static(msg: &'static str) -> Self {
        Error::ParseError(Cow::Borrowed(msg))
    }
    
    /// Create a parse error with an owned string
    pub fn parse_owned(msg: String) -> Self {
        Error::ParseError(Cow::Owned(msg))
    }
    
    /// Create a render error with a static string
    pub fn render_static(msg: &'static str) -> Self {
        Error::RenderError(Cow::Borrowed(msg))
    }
    
    /// Create a render error with an owned string
    pub fn render_owned(msg: String) -> Self {
        Error::RenderError(Cow::Owned(msg))
    }
    
    /// Create a DOM error with a static string
    pub fn dom_static(msg: &'static str) -> Self {
        Error::DomError(Cow::Borrowed(msg))
    }
    
    /// Create a DOM error with an owned string
    pub fn dom_owned(msg: String) -> Self {
        Error::DomError(Cow::Owned(msg))
    }
    
    /// Create a constraint error with a static string
    pub fn constraint_static(msg: &'static str) -> Self {
        Error::ConstraintError(Cow::Borrowed(msg))
    }
    
    /// Create a constraint error with an owned string
    pub fn constraint_owned(msg: String) -> Self {
        Error::ConstraintError(Cow::Owned(msg))
    }
    
    /// Create a selector error with a static string
    pub fn selector_static(msg: &'static str) -> Self {
        Error::SelectorError(Cow::Borrowed(msg))
    }
    
    /// Create a selector error with an owned string
    pub fn selector_owned(msg: String) -> Self {
        Error::SelectorError(Cow::Owned(msg))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = Error::parse_static("Failed to parse template");
        assert_eq!(err.to_string(), "Parse error: Failed to parse template");
        
        let err = Error::render_static("Failed to render");
        assert_eq!(err.to_string(), "Render error: Failed to render");
        
        let err = Error::dom_static("DOM manipulation failed");
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