//! Core template types and configurations
//!
//! This module contains the main template structures, configuration options,
//! and fundamental types used throughout the library.
//!
//! # Key Types
//!
//! - [`HtmlTemplate`] - The main template type for rendering HTML
//! - [`TemplateConfig`] - Configuration options for template behavior
//! - [`CacheMode`] - Cache strategies for improved performance
//! - [`CompiledTemplate`] - Internal compiled template representation
//!
//! # Usage
//!
//! Most users will primarily interact with [`HtmlTemplate`] and [`TemplateConfig`]:
//!
//! ```rust,ignore
//! use html_template::{HtmlTemplate, TemplateConfig, CacheMode};
//! use serde_json::json;
//!
//! let config = TemplateConfig::default()
//!     .with_cache_mode(CacheMode::Aggressive)
//!     .with_zero_copy(true);
//!
//! let template = HtmlTemplate::from_str_with_config(html, Some("div"), config)?;
//! let result = template.render(&data)?;
//! ```

use dom_query::Document;
use std::collections::HashMap;
use std::sync::Arc;

use crate::cache::{get_global_cache, TemplateCache, TemplateCacheKey};
use crate::error::{Error, Result};
use crate::handlers::{ElementHandler, HandlerRegistry};
use crate::value::RenderValue;

/// Main template type for HTML rendering with microdata support
///
/// `HtmlTemplate` represents a compiled HTML template that can be rendered multiple times
/// with different data. Templates use microdata attributes (`itemprop`, `itemscope`, `itemtype`)
/// for data binding and support advanced features like array handling, nested objects,
/// constraints, and custom element handlers.
///
/// # Creating Templates
///
/// Templates can be created from strings, files, or using the builder pattern:
///
/// ```rust,ignore
/// use html_template::{HtmlTemplate, HtmlTemplateBuilder};
/// use serde_json::json;
///
/// // From string
/// let template = HtmlTemplate::from_str(html, Some("div"))?;
///
/// // From file  
/// let template = HtmlTemplate::from_file("template.html")?;
///
/// // Using builder
/// let template = HtmlTemplateBuilder::new()
///     .from_str(html)
///     .with_selector("article")
///     .with_default_handlers()
///     .build()?;
/// ```
///
/// # Rendering Data
///
/// Templates can render any type implementing [`RenderValue`]:
///
/// ```rust,ignore
/// use serde_json::json;
///
/// let data = json!({
///     "title": "Hello World",
///     "items": [{"name": "Item 1"}, {"name": "Item 2"}]
/// });
///
/// let result = template.render(&data)?;
/// ```
///
/// # Thread Safety
///
/// `HtmlTemplate` is thread-safe and can be shared across threads. The internal
/// compiled template is reference-counted, making cloning cheap.
///
/// # Performance
///
/// Templates are compiled once and can be rendered multiple times efficiently.
/// Enable caching and zero-copy optimizations for better performance:
///
/// ```rust,ignore
/// let config = TemplateConfig::default()
///     .with_cache_mode(CacheMode::Aggressive)
///     .with_zero_copy(true);
///
/// let template = HtmlTemplate::from_str_with_config(html, None, config)?;
/// ```
pub struct HtmlTemplate {
    pub(crate) compiled: Arc<CompiledTemplate>,
    pub(crate) config: TemplateConfig,
    pub(crate) handlers: HashMap<String, Box<dyn ElementHandler>>,
    pub(crate) handler_registry: Option<HandlerRegistry>,
}

impl Clone for HtmlTemplate {
    fn clone(&self) -> Self {
        Self {
            compiled: self.compiled.clone(),
            config: self.config.clone(),
            // Note: handlers are not cloned as ElementHandler doesn't implement Clone
            // This means cached templates will have empty handlers
            handlers: HashMap::new(),
            handler_registry: None,
        }
    }
}

impl std::fmt::Debug for HtmlTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlTemplate")
            .field("compiled", &self.compiled)
            .field("config", &self.config)
            .field("handlers", &self.handlers.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl HtmlTemplate {
    /// Create a new HtmlTemplate from compiled template
    pub fn new(
        compiled: Arc<CompiledTemplate>,
        config: TemplateConfig,
        handlers: std::collections::HashMap<String, Box<dyn ElementHandler>>,
    ) -> Self {
        Self {
            compiled,
            config,
            handlers,
            handler_registry: None,
        }
    }

    /// Create a new HtmlTemplate with HandlerRegistry
    pub fn new_with_registry(
        compiled: Arc<CompiledTemplate>,
        config: TemplateConfig,
        handler_registry: HandlerRegistry,
    ) -> Self {
        Self {
            compiled,
            config,
            handlers: HashMap::new(),
            handler_registry: Some(handler_registry),
        }
    }

    /// Create a template from HTML string and selector
    pub fn from_str(html: &str, selector: Option<&str>) -> Result<Self> {
        Self::from_str_with_config(html, selector, TemplateConfig::default())
    }

    /// Create a template from HTML string with custom configuration
    pub fn from_str_with_config(
        html: &str,
        selector: Option<&str>,
        config: TemplateConfig,
    ) -> Result<Self> {
        let compiled = if config.cache_mode != CacheMode::None {
            // Use caching
            let cache_key = TemplateCacheKey::new(html, selector);
            let cache = get_global_cache();

            cache.get_or_compile_template(&cache_key, || {
                crate::compiler::Compiler::compile(html, selector)
            })?
        } else {
            // Direct compilation without caching
            crate::compiler::Compiler::compile(html, selector)?
        };

        Ok(Self::new(
            compiled,
            config,
            std::collections::HashMap::new(),
        ))
    }

    /// Create a template from HTML string using a custom cache
    pub fn from_str_with_cache(
        html: &str,
        selector: Option<&str>,
        config: TemplateConfig,
        cache: &TemplateCache,
    ) -> Result<Self> {
        let compiled = if config.cache_mode != CacheMode::None {
            let cache_key = TemplateCacheKey::new(html, selector);
            cache.get_or_compile_template(&cache_key, || {
                crate::compiler::Compiler::compile(html, selector)
            })?
        } else {
            crate::compiler::Compiler::compile(html, selector)?
        };

        Ok(Self::new(
            compiled,
            config,
            std::collections::HashMap::new(),
        ))
    }

    /// Create a template from a DOM element
    pub fn from_element(element: &dom_query::Node) -> Result<Self> {
        Self::from_element_with_config(element, TemplateConfig::default())
    }

    /// Create a template from a DOM element with custom configuration
    pub fn from_element_with_config(
        element: &dom_query::Node,
        config: TemplateConfig,
    ) -> Result<Self> {
        // When creating from element, wrap it in a template tag to ensure proper parsing
        let html = format!("<template>{}</template>", element.html());
        // Use the element's tag name as selector to extract just this element
        let selector = element.node_name().map(|n| n.to_string());
        Self::from_str_with_config(&html, selector.as_deref(), config)
    }

    /// Create a template from a file path
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Self::from_file_with_config(path, TemplateConfig::default())
    }

    /// Create a template from a file path with custom configuration
    pub fn from_file_with_config<P: AsRef<std::path::Path>>(
        path: P,
        config: TemplateConfig,
    ) -> Result<Self> {
        let html = std::fs::read_to_string(&path).map_err(|e| {
            Error::io(format!(
                "Failed to read template file '{}': {}",
                path.as_ref().display(),
                e
            ))
        })?;
        Self::from_str_with_config(&html, None, config)
    }

    /// Create a template from a file path with selector
    pub fn from_file_with_selector<P: AsRef<std::path::Path>>(
        path: P,
        selector: &str,
    ) -> Result<Self> {
        Self::from_file_with_selector_and_config(path, selector, TemplateConfig::default())
    }

    /// Create a template from a file path with selector and custom configuration
    pub fn from_file_with_selector_and_config<P: AsRef<std::path::Path>>(
        path: P,
        selector: &str,
        config: TemplateConfig,
    ) -> Result<Self> {
        let html = std::fs::read_to_string(&path).map_err(|e| {
            Error::io(format!(
                "Failed to read template file '{}': {}",
                path.as_ref().display(),
                e
            ))
        })?;
        Self::from_str_with_config(&html, Some(selector), config)
    }

    /// Render the template with the given data
    pub fn render(&self, data: &dyn RenderValue) -> Result<String> {
        let renderer = if let Some(ref registry) = self.handler_registry {
            crate::renderer::Renderer::new_with_registry(&self.compiled, registry, &self.handlers)
        } else {
            crate::renderer::Renderer::new(&self.compiled, &self.handlers)
        };
        renderer.render(data)
    }

    /// Render template using microdata extracted from a DOM element
    pub fn render_from_element(&self, element: &dom_query::Node) -> Result<String> {
        let microdata = crate::microdata::extract_microdata(element)?;
        self.render(&microdata)
    }

    /// Render template using microdata extracted from HTML
    pub fn render_from_html(&self, html: &str) -> Result<Vec<String>> {
        let items = crate::microdata::extract_microdata_from_html(html)?;
        let mut results = Vec::new();

        for item in items {
            results.push(self.render(&item)?);
        }

        Ok(results)
    }

    /// Render template using microdata extracted from a document
    pub fn render_from_document(&self, doc: &Document) -> Result<Vec<String>> {
        let items = crate::microdata::extract_microdata_from_document(doc)?;
        let mut results = Vec::new();

        for item in items {
            results.push(self.render(&item)?);
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct CompiledTemplate {
    pub(crate) root_selector: Option<String>,
    pub(crate) elements: Vec<TemplateElement>,
    pub(crate) constraints: Vec<Constraint>,
    #[allow(dead_code)]
    pub(crate) base_uri: Option<String>,
    pub(crate) template_html: String,
}

#[derive(Debug, Clone)]
pub struct TemplateElement {
    pub(crate) selector: String,
    pub(crate) properties: Vec<Property>,
    pub(crate) is_array: bool,
    pub(crate) is_scope: bool,
    #[allow(dead_code)]
    pub(crate) itemtype: Option<String>,
    #[allow(dead_code)]
    pub(crate) constraints: Vec<ConstraintRef>,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) is_array: bool,
    pub(crate) target: PropertyTarget,
    pub(crate) variables: Vec<Variable>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyTarget {
    TextContent,
    Attribute(String),
    Value, // for input elements
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub(crate) path: Vec<String>,
    pub(crate) raw: String,
}

#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub(crate) cache_mode: CacheMode,
    pub(crate) zero_copy: bool,
    pub(crate) cache_compiled_templates: bool,
    pub(crate) cache_external_documents: bool,
}

impl TemplateConfig {
    /// Create a new template configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the cache mode
    pub fn with_cache_mode(mut self, mode: CacheMode) -> Self {
        self.cache_mode = mode;
        self
    }

    /// Enable or disable zero-copy optimizations
    pub fn with_zero_copy(mut self, enabled: bool) -> Self {
        self.zero_copy = enabled;
        self
    }

    /// Enable or disable compiled template caching
    pub fn with_compiled_template_caching(mut self, enabled: bool) -> Self {
        self.cache_compiled_templates = enabled;
        self
    }

    /// Enable or disable external document caching
    pub fn with_external_document_caching(mut self, enabled: bool) -> Self {
        self.cache_external_documents = enabled;
        self
    }

    /// Create configuration for aggressive caching
    pub fn aggressive_caching() -> Self {
        Self {
            cache_mode: CacheMode::Aggressive,
            zero_copy: true,
            cache_compiled_templates: true,
            cache_external_documents: true,
        }
    }

    /// Create configuration with no caching
    pub fn no_caching() -> Self {
        Self {
            cache_mode: CacheMode::None,
            zero_copy: true,
            cache_compiled_templates: false,
            cache_external_documents: false,
        }
    }

    // Accessors
    pub fn cache_mode(&self) -> CacheMode {
        self.cache_mode
    }
    pub fn zero_copy(&self) -> bool {
        self.zero_copy
    }
    pub fn cache_compiled_templates(&self) -> bool {
        self.cache_compiled_templates
    }
    pub fn cache_external_documents(&self) -> bool {
        self.cache_external_documents
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            cache_mode: CacheMode::Normal,
            zero_copy: true,
            cache_compiled_templates: true,
            cache_external_documents: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheMode {
    None,
    Normal,
    Aggressive,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub(crate) element_selector: String,
    pub(crate) constraint_type: ConstraintType,
    #[allow(dead_code)]
    pub(crate) scope: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConstraintType {
    Scope(String),
    Expression(String),
}

#[derive(Debug, Clone)]
pub struct ConstraintRef {
    #[allow(dead_code)]
    pub(crate) constraint_index: usize,
}

#[allow(dead_code)]
pub(crate) enum TemplateSource {
    Document(Document),
    Html(String),
    File(std::path::PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_config_default() {
        let config = TemplateConfig::default();
        assert_eq!(config.cache_mode(), CacheMode::Normal);
        assert_eq!(config.zero_copy(), true);
    }

    #[test]
    fn test_property_target_equality() {
        assert_eq!(PropertyTarget::TextContent, PropertyTarget::TextContent);
        assert_eq!(PropertyTarget::Value, PropertyTarget::Value);
        assert_eq!(
            PropertyTarget::Attribute("href".to_string()),
            PropertyTarget::Attribute("href".to_string())
        );
        assert_ne!(
            PropertyTarget::Attribute("href".to_string()),
            PropertyTarget::Attribute("src".to_string())
        );
    }

    #[test]
    fn test_variable_construction() {
        let var = Variable {
            path: vec!["user".to_string(), "name".to_string()],
            raw: "${user.name}".to_string(),
        };
        assert_eq!(var.path.len(), 2);
        assert_eq!(var.path[0], "user");
        assert_eq!(var.path[1], "name");
    }

    #[test]
    fn test_from_element() {
        let html = r#"<div itemprop="message"></div>"#;

        let doc = Document::from(html);
        let selection = doc.select("div");
        let element = selection.nodes().first().unwrap();

        let config = TemplateConfig::no_caching();
        let template = HtmlTemplate::from_element_with_config(&element, config).unwrap();

        let data = serde_json::json!({"message": "Hello from element"});
        let result = template.render(&data).unwrap();

        assert!(result.contains("Hello from element"));
    }

    #[test]
    fn test_from_element_with_config() {
        let html = r#"<div itemprop="test"></div>"#;

        let doc = Document::from(html);
        let selection = doc.select("div");
        let element = selection.nodes().first().unwrap();

        let config = TemplateConfig::no_caching();
        let template = HtmlTemplate::from_element_with_config(&element, config).unwrap();

        assert_eq!(template.config.cache_mode(), CacheMode::None);

        let data = serde_json::json!({"test": "Element config test"});
        let result = template.render(&data).unwrap();

        assert!(result.contains("Element config test"));
    }

    #[test]
    fn test_from_file() {
        use std::io::Write;

        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_template.html");

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

        let config = TemplateConfig::no_caching();
        let template = HtmlTemplate::from_file_with_config(&file_path, config).unwrap();

        let data = serde_json::json!({"content": "File content"});
        let result = template.render(&data).unwrap();

        assert!(result.contains("File content"));

        // Clean up
        std::fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_from_file_with_config() {
        use std::io::Write;

        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_template_config.html");

        let html_content = r#"
            <template>
                <article>
                    <h1 itemprop="title"></h1>
                </article>
            </template>
        "#;

        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            file.write_all(html_content.as_bytes()).unwrap();
        }

        let config = TemplateConfig::aggressive_caching();
        let template = HtmlTemplate::from_file_with_config(&file_path, config).unwrap();

        assert_eq!(template.config.cache_mode(), CacheMode::Aggressive);

        let data = serde_json::json!({"title": "File with config"});
        let result = template.render(&data).unwrap();

        assert!(result.contains("File with config"));

        // Clean up
        std::fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_from_file_with_selector() {
        use std::io::Write;

        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_template_selector.html");

        let html_content = r#"
            <template>
                <div class="container">
                    <p itemprop="paragraph"></p>
                </div>
            </template>
        "#;

        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            file.write_all(html_content.as_bytes()).unwrap();
        }

        let config = TemplateConfig::no_caching();
        let template =
            HtmlTemplate::from_file_with_selector_and_config(&file_path, "p", config).unwrap();

        let data = serde_json::json!({"paragraph": "Selector test"});
        let result = template.render(&data).unwrap();

        assert!(result.contains("Selector test"));

        // Clean up
        std::fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_from_file_with_selector_and_config() {
        use std::io::Write;

        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_template_full.html");

        let html_content = r#"
            <template>
                <section>
                    <div itemprop="content"></div>
                </section>
            </template>
        "#;

        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            file.write_all(html_content.as_bytes()).unwrap();
        }

        let config = TemplateConfig::no_caching();
        let template =
            HtmlTemplate::from_file_with_selector_and_config(&file_path, "div", config).unwrap();

        assert_eq!(template.config.cache_mode(), CacheMode::None);

        let data = serde_json::json!({"content": "Full method test"});
        let result = template.render(&data).unwrap();

        assert!(result.contains("Full method test"));

        // Clean up
        std::fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_from_file_nonexistent() {
        let result = HtmlTemplate::from_file("/nonexistent/file.html");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read template file"));
    }
}
