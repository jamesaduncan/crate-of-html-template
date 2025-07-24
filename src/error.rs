//! Error handling for html-template
//!
//! This module provides a comprehensive error type system for all html-template operations.
//! All errors implement the standard `std::error::Error` trait and provide detailed
//! context information for debugging.
//!
//! # Error Types
//!
//! - [`Error::ParseError`] - HTML parsing and template extraction errors
//! - [`Error::RenderError`] - Template rendering and data binding errors  
//! - [`Error::SelectorError`] - CSS selector parsing and matching errors
//! - [`Error::ConstraintError`] - Constraint evaluation and processing errors
//! - [`Error::DomError`] - DOM manipulation and traversal errors
//! - [`Error::JsonError`] - JSON parsing and serialization errors
//! - [`Error::HttpError`] - HTTP client and networking errors
//! - [`Error::IoError`] - File system and I/O errors
//!
//! # Usage
//!
//! All public functions in this crate return `Result<T, Error>` for error handling:
//!
//! ```rust,ignore
//! use html_template::{HtmlTemplate, Error};
//!
//! match HtmlTemplate::from_str(html, Some("div")) {
//!     Ok(template) => println!("Template created successfully"),
//!     Err(Error::ParseError(msg)) => println!("Parse error: {}", msg),
//!     Err(Error::SelectorError(msg)) => println!("Selector error: {}", msg),
//!     Err(err) => println!("Other error: {}", err),
//! }
//! ```
//!
//! # Memory Efficiency
//!
//! Error messages use `Cow<'static, str>` to avoid unnecessary allocations
//! when using static error messages, while still supporting owned strings
//! for dynamic error context.

use std::borrow::Cow;
use thiserror::Error;

/// Comprehensive error type for all html-template operations
///
/// This enum covers all possible error conditions that can occur during
/// template parsing, compilation, rendering, and related operations.
/// Each variant provides specific context about the error that occurred.
///
/// # Examples
///
/// ```rust,ignore
/// use html_template::{HtmlTemplate, Error};
///
/// let result = HtmlTemplate::from_str("<invalid html", None);
/// match result {
///     Err(Error::ParseError(msg)) => {
///         println!("Failed to parse template: {}", msg);
///     }
///     _ => {}
/// }
/// ```
#[derive(Error, Debug)]
pub enum Error {
    /// HTML parsing and template extraction errors
    ///
    /// Occurs when the HTML cannot be parsed, template elements are malformed,
    /// or microdata attributes are invalid.
    #[error("Parse error: {0}")]
    ParseError(Cow<'static, str>),

    /// Template rendering and data binding errors
    ///
    /// Occurs during template rendering when data binding fails, variable
    /// resolution encounters issues, or the rendering process cannot complete.
    #[error("Render error: {0}")]
    RenderError(Cow<'static, str>),

    /// CSS selector parsing and matching errors
    ///
    /// Occurs when CSS selectors are malformed or cannot be applied to
    /// the template's DOM structure.
    #[error("Selector error: {0}")]
    SelectorError(Cow<'static, str>),

    /// Constraint evaluation and processing errors
    ///
    /// Occurs when constraint expressions cannot be parsed or evaluated,
    /// or when constraint logic encounters invalid conditions.
    #[error("Constraint error: {0}")]
    ConstraintError(Cow<'static, str>),

    /// DOM manipulation and traversal errors
    ///
    /// Occurs when DOM operations fail, elements cannot be found or modified,
    /// or when DOM structure is invalid for the requested operation.
    #[error("DOM error: {0}")]
    DomError(Cow<'static, str>),

    /// JSON parsing and serialization errors
    ///
    /// Automatically converted from `serde_json::Error` when JSON operations fail.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HTTP client and networking errors
    ///
    /// Occurs during cross-document rendering when external resources
    /// cannot be fetched or network operations fail.
    #[error("HTTP error: {0}")]
    HttpError(Cow<'static, str>),

    /// File system and I/O errors
    ///
    /// Automatically converted from `std::io::Error` when file operations fail.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Error {
    /// Create a parse error with a static string
    ///
    /// Use this for compile-time known error messages to avoid allocations.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use html_template::Error;
    ///
    /// let error = Error::parse_static("Invalid template structure");
    /// ```
    pub fn parse_static(msg: &'static str) -> Self {
        Error::ParseError(Cow::Borrowed(msg))
    }

    /// Create a parse error with an owned string
    ///
    /// Use this for dynamic error messages that include runtime context.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use html_template::Error;
    ///
    /// let error = Error::parse_owned(format!("Invalid element: {}", element_name));
    /// ```
    pub fn parse_owned(msg: String) -> Self {
        Error::ParseError(Cow::Owned(msg))
    }

    /// Create a render error with a static string
    ///
    /// Use this for compile-time known error messages to avoid allocations.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use html_template::Error;
    ///
    /// let error = Error::render_static("Data binding failed");
    /// ```
    pub fn render_static(msg: &'static str) -> Self {
        Error::RenderError(Cow::Borrowed(msg))
    }

    /// Create a render error with an owned string
    ///
    /// Use this for dynamic error messages that include runtime context.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use html_template::Error;
    ///
    /// let error = Error::render_owned(format!("Missing property: {}", prop_name));
    /// ```
    pub fn render_owned(msg: String) -> Self {
        Error::RenderError(Cow::Owned(msg))
    }

    /// Create a DOM error with a static string
    ///
    /// Use this for compile-time known error messages to avoid allocations.
    pub fn dom_static(msg: &'static str) -> Self {
        Error::DomError(Cow::Borrowed(msg))
    }

    /// Create a DOM error with an owned string
    ///
    /// Use this for dynamic error messages that include runtime context.
    pub fn dom_owned(msg: String) -> Self {
        Error::DomError(Cow::Owned(msg))
    }

    /// Create a constraint error with a static string
    ///
    /// Use this for compile-time known error messages to avoid allocations.
    pub fn constraint_static(msg: &'static str) -> Self {
        Error::ConstraintError(Cow::Borrowed(msg))
    }

    /// Create a constraint error with an owned string
    ///
    /// Use this for dynamic error messages that include runtime context.
    pub fn constraint_owned(msg: String) -> Self {
        Error::ConstraintError(Cow::Owned(msg))
    }

    /// Create a selector error with a static string
    ///
    /// Use this for compile-time known error messages to avoid allocations.
    pub fn selector_static(msg: &'static str) -> Self {
        Error::SelectorError(Cow::Borrowed(msg))
    }

    /// Create a selector error with an owned string
    ///
    /// Use this for dynamic error messages that include runtime context.
    pub fn selector_owned(msg: String) -> Self {
        Error::SelectorError(Cow::Owned(msg))
    }

    /// Create a parse error (alias for compatibility)
    ///
    /// This is a convenience method that calls [`Error::parse_owned`].
    pub fn parse(msg: String) -> Self {
        Self::parse_owned(msg)
    }

    /// Create an IO error (alias for compatibility)
    ///
    /// This creates a generic IO error with the provided message.
    /// For more specific IO errors, use the standard `std::io::Error` constructors
    /// and let them be automatically converted.
    pub fn io(msg: String) -> Self {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, msg);
        Error::IoError(io_err)
    }
}

/// Result type alias for html-template operations
///
/// This is a convenience alias for `Result<T, Error>` that is used throughout
/// the html-template crate. All public functions return this Result type.
///
/// # Examples
///
/// ```rust,ignore
/// use html_template::{HtmlTemplate, Result};
///
/// fn create_template(html: &str) -> Result<HtmlTemplate> {
///     HtmlTemplate::from_str(html, None)
/// }
/// ```
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

    #[test]
    fn test_all_error_variants() {
        // Test all error variants with owned strings
        let parse_err = Error::parse_owned("parse error".to_string());
        assert_eq!(parse_err.to_string(), "Parse error: parse error");

        let render_err = Error::render_owned("render error".to_string());
        assert_eq!(render_err.to_string(), "Render error: render error");

        let dom_err = Error::dom_owned("dom error".to_string());
        assert_eq!(dom_err.to_string(), "DOM error: dom error");

        let io_err = Error::io("io error message".to_string());
        assert_eq!(io_err.to_string(), "IO error: io error message");
    }

    #[test]
    fn test_error_chaining() {
        // Test that errors can be properly converted and chained
        let original = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let wrapped: Error = original.into();

        // Verify the error message is preserved
        assert!(wrapped.to_string().contains("access denied"));
    }
}
