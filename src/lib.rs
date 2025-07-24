//! HTML templating library using microdata attributes
//! 
//! This crate provides a powerful HTML templating system that uses microdata
//! attributes (itemprop, itemtype, itemscope) for data binding. It supports
//! nested data structures, arrays, variable substitution, and more.
//! 
//! # Examples
//! 
//! ```rust,ignore
//! use html_template::HtmlTemplate;
//! use serde_json::json;
//! 
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let html = r#"
//!     <template>
//!         <div>
//!             <h1 itemprop="title"></h1>
//!             <p itemprop="description"></p>
//!         </div>
//!     </template>
//! "#;
//! 
//! // API not yet implemented - use parser and compiler directly
//! let template = HtmlTemplate::from_str(html, "div")?;
//! 
//! let data = json!({
//!     "title": "Hello World",
//!     "description": "This is a test"
//! });
//! 
//! let rendered = template.render(&data)?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod types;
pub mod value;
pub mod handlers;
pub mod parser;
pub mod node_ext;
pub mod compiler;
pub mod renderer;
pub mod constraints;

// Re-export commonly used types
pub use error::{Error, Result};
pub use types::{HtmlTemplate, TemplateConfig, CacheMode};
pub use value::RenderValue;
pub use handlers::ElementHandler;

// Re-export derive macro when implemented
#[cfg(feature = "derive")]
pub use html_template_macros::Renderable;