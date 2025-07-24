//! # HTML Template - A Rust HTML Templating Library
//! 
//! `html-template` is a powerful HTML templating library that uses microdata attributes 
//! (itemprop, itemtype, itemscope) for clean, semantic data binding. It supports nested 
//! data structures, arrays, variable substitution, streaming rendering, caching, and more.
//! 
//! ## Features
//! 
//! - **Microdata-based templating**: Uses standard HTML microdata attributes for data binding
//! - **Type-safe rendering**: Compile-time guarantees with derive macros and trait system
//! - **High performance**: Streaming rendering, zero-copy optimizations, and aggressive caching
//! - **Flexible API**: Builder pattern, direct constructors, and fluent configuration
//! - **Cross-document support**: Render templates using data from external sources
//! - **Advanced features**: Constraint system, Schema.org integration, custom element handlers
//! 
//! ## Quick Start
//! 
//! ### Basic Usage
//! 
//! ```rust,ignore
//! use html_template::{HtmlTemplate, Result};
//! use serde_json::json;
//! 
//! # fn example() -> Result<()> {
//! let html = r#"
//!     <template>
//!         <div>
//!             <h1 itemprop="title"></h1>
//!             <p itemprop="description"></p>
//!         </div>
//!     </template>
//! "#;
//! 
//! let template = HtmlTemplate::from_str(html, Some("div"))?;
//! 
//! let data = json!({
//!     "title": "Hello World",
//!     "description": "This is a microdata-powered template!"
//! });
//! 
//! let rendered = template.render(&data)?;
//! println!("{}", rendered);
//! # Ok(())
//! # }
//! ```
//! 
//! ### Using the Builder Pattern
//! 
//! ```rust,ignore
//! use html_template::{HtmlTemplateBuilder, TemplateConfig, CacheMode};
//! use serde_json::json;
//! 
//! # fn builder_example() -> html_template::Result<()> {
//! let template = HtmlTemplateBuilder::new()
//!     .from_str(html)
//!     .with_selector("article")
//!     .with_caching(CacheMode::Aggressive)
//!     .with_zero_copy(true)
//!     .build()?;
//! 
//! let data = json!({"title": "Builder Pattern", "content": "Easy to use!"});
//! let result = template.render(&data)?;
//! # Ok(())
//! # }
//! ```
//! 
//! ### Derive Macro for Type Safety
//! 
//! ```rust,ignore
//! use html_template::{Renderable, RenderValue};
//! 
//! #[derive(Renderable)]
//! struct Article {
//!     title: String,
//!     #[renderable(rename = "articleBody")]
//!     content: String,
//!     #[renderable(skip)]
//!     internal_id: u64,
//! }
//! 
//! let article = Article {
//!     title: "Type-Safe Templates".to_string(),
//!     content: "No more runtime errors!".to_string(),
//!     internal_id: 12345,
//! };
//! 
//! let result = template.render(&article)?;
//! ```
//! 
//! ### Streaming for Large Datasets
//! 
//! ```rust,ignore
//! use html_template::{StreamingRenderer, HtmlTemplate};
//! 
//! # fn streaming_example() -> html_template::Result<()> {
//! let renderer = StreamingRenderer::new(&template, &handlers, 1000);
//! let mut stream = renderer.render_owned(large_dataset)?;
//! 
//! while let Some(chunk) = stream.next_chunk()? {
//!     println!("{}", chunk);
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! ## Template Syntax
//! 
//! HTML templates use standard microdata attributes:
//! 
//! - `itemprop="propertyName"` - Binds data property to element
//! - `itemscope` - Creates nested object scope
//! - `itemtype="SchemaType"` - Specifies Schema.org type for validation
//! - `data-constraint="expression"` - Conditional rendering
//! - `${variable}` - Variable substitution in text and attributes
//! 
//! ### Array Handling
//! 
//! ```html
//! <template>
//!     <ul>
//!         <li itemprop="items[]"></li>
//!     </ul>
//! </template>
//! ```
//! 
//! ### Nested Objects
//! 
//! ```html
//! <template>
//!     <article itemscope>
//!         <h1 itemprop="headline"></h1>
//!         <div itemprop="author" itemscope>
//!             <span itemprop="name"></span>
//!             <span itemprop="email"></span>
//!         </div>
//!     </article>
//! </template>
//! ```

// Public modules - these are part of the stable API
pub mod error;
pub mod types;
pub mod value;
pub mod handlers;
pub mod streaming;
pub mod cache;
pub mod cross_document;
pub mod builder;

// Internal modules - implementation details, may change without notice
#[doc(hidden)]
pub mod parser;
#[doc(hidden)]
pub mod node_ext;
#[doc(hidden)]
pub mod compiler;
#[doc(hidden)]
pub mod renderer;
#[doc(hidden)]
pub mod constraints;
#[doc(hidden)]
pub mod microdata;
#[doc(hidden)]
pub mod utils;

// ============================================================================
// Core API - Essential types for basic usage
// ============================================================================

/// Core template type and result types
pub use error::{Error, Result};
/// Main template struct and configuration
pub use types::{HtmlTemplate, TemplateConfig, CacheMode};
/// Trait for types that can be rendered in templates
pub use value::RenderValue;

// ============================================================================
// Builder API - Fluent template construction
// ============================================================================

/// Builder pattern for constructing templates
pub use builder::{HtmlTemplateBuilder, RenderBuilder, RenderResult};

// ============================================================================
// Advanced Features - Performance and extensibility
// ============================================================================

/// Custom element handlers for specialized rendering
pub use handlers::ElementHandler;

/// Streaming rendering for large datasets
pub use streaming::{StreamingRenderer, StreamingResult, OwnedStreamingResult};

/// Caching system for improved performance
pub use cache::{
    TemplateCache, CacheConfig, CacheStats, TemplateCacheStats, EvictionStrategy
};

/// Cross-document rendering with external data sources
pub use cross_document::{
    CrossDocumentRenderer, DocumentFetcher, CrossDocumentConfig, CrossDocumentTemplate,
    DataSource, CrossDocumentRequest, CrossDocumentResponse, CrossDocumentMetadata
};

// ============================================================================
// Derive Macro - Type-safe template rendering
// ============================================================================

/// Derive macro for automatic RenderValue implementation
/// 
/// Enable with the "derive" feature in your Cargo.toml:
/// ```toml
/// [dependencies]
/// html-template = { version = "0.1", features = ["derive"] }
/// ```
#[cfg(feature = "derive")]
pub use html_template_macros::Renderable;

// ============================================================================
// Convenience Functions - Quick template operations
// ============================================================================

/// Quick template rendering without explicit template construction
/// 
/// This is a convenience function for simple one-off template rendering.
/// For repeated rendering or advanced features, use [`HtmlTemplate`] directly.
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use html_template::render_string;
/// use serde_json::json;
/// 
/// let html = r#"<div itemprop="message"></div>"#;
/// let data = json!({"message": "Hello World"});
/// let result = render_string(html, &data)?;
/// ```
pub fn render_string(html: &str, data: &dyn RenderValue) -> Result<String> {
    let config = TemplateConfig::no_caching();
    let template = HtmlTemplate::from_str_with_config(html, None, config)?;
    template.render(data)
}

/// Quick template rendering with selector
/// 
/// Like [`render_string`] but allows specifying a CSS selector for the root element.
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use html_template::render_string_with_selector;
/// use serde_json::json;
/// 
/// let html = r#"<template><div itemprop="msg"></div></template>"#;
/// let data = json!({"msg": "Hello"});
/// let result = render_string_with_selector(html, "div", &data)?;
/// ```
pub fn render_string_with_selector(html: &str, selector: &str, data: &dyn RenderValue) -> Result<String> {
    let config = TemplateConfig::no_caching();
    let template = HtmlTemplate::from_str_with_config(html, Some(selector), config)?;
    template.render(data)
}

/// Quick template rendering from file
/// 
/// Convenience function to render a template directly from a file.
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use html_template::render_file;
/// use serde_json::json;
/// 
/// let data = json!({"title": "File Template"});
/// let result = render_file("template.html", &data)?;
/// ```
pub fn render_file<P: AsRef<std::path::Path>>(path: P, data: &dyn RenderValue) -> Result<String> {
    let config = TemplateConfig::no_caching();
    let template = HtmlTemplate::from_file_with_config(path, config)?;
    template.render(data)
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_render_string_convenience() {
        let html = r#"
            <template>
                <div itemprop="message"></div>
            </template>
        "#;
        let data = json!({"message": "Hello from convenience function!"});
        
        let result = render_string_with_selector(html, "div", &data).unwrap();
        assert!(result.contains("Hello from convenience function!"));
    }
    
    #[test] 
    fn test_render_string_with_selector_convenience() {
        let html = r#"
            <template>
                <div itemprop="message"></div>
            </template>
        "#;
        let data = json!({"message": "Hello with selector!"});
        
        let result = render_string_with_selector(html, "div", &data).unwrap();
        assert!(result.contains("Hello with selector!"));
    }
    
    #[test]
    fn test_render_file_convenience() {
        use std::io::Write;
        
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_convenience.html");
        
        let html_content = r#"
            <template>
                <div>
                    <span itemprop="content"></span>
                </div>
            </template>
        "#;
        
        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            file.write_all(html_content.as_bytes()).unwrap();
        }
        
        let data = json!({"content": "File convenience test"});
        let result = render_file(&file_path, &data).unwrap();
        
        assert!(result.contains("File convenience test"));
        
        // Clean up
        std::fs::remove_file(&file_path).ok();
    }
}