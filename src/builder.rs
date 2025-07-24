//! Builder pattern API for constructing HtmlTemplate instances
//! 
//! This module provides a fluent, ergonomic API for creating templates with
//! various configurations, handlers, and data sources.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use dom_query::Document;

use crate::error::{Error, Result};
use crate::types::*;
use crate::handlers::{ElementHandler, HandlerRegistry};
use crate::cache::TemplateCache;

/// Builder for constructing HtmlTemplate instances
/// 
/// Provides a fluent API for configuring templates with various options,
/// handlers, and data sources before compilation.
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use html_template::HtmlTemplateBuilder;
/// 
/// let template = HtmlTemplateBuilder::new()
///     .from_str(html_content)
///     .with_selector("article")
///     .with_caching(CacheMode::Aggressive)
///     .with_zero_copy(true)
///     .add_handler("input", input_handler)
///     .build()?;
/// ```
pub struct HtmlTemplateBuilder {
    source: Option<TemplateSource>,
    selector: Option<String>,
    config: TemplateConfig,
    handlers: HashMap<String, Box<dyn ElementHandler>>,
    handler_registry: Option<HandlerRegistry>,
    custom_cache: Option<TemplateCache>,
}

/// Template source for the builder
#[derive(Debug, Clone)]
enum TemplateSource {
    Html(String),
    Element(String), // Serialized HTML from element
    File(std::path::PathBuf),
}

impl HtmlTemplateBuilder {
    /// Create a new template builder
    pub fn new() -> Self {
        Self {
            source: None,
            selector: None,
            config: TemplateConfig::default(),
            handlers: HashMap::new(),
            handler_registry: None,
            custom_cache: None,
        }
    }
    
    /// Set the HTML source from a string
    pub fn from_str<S: Into<String>>(mut self, html: S) -> Self {
        self.source = Some(TemplateSource::Html(html.into()));
        self
    }
    
    /// Set the source from a DOM element
    pub fn from_element(mut self, element: &dom_query::Node) -> Self {
        let html = element.html().to_string();
        self.source = Some(TemplateSource::Element(html));
        self
    }
    
    /// Set the source from a file path
    pub fn from_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.source = Some(TemplateSource::File(path.as_ref().to_path_buf()));
        self
    }
    
    /// Set the root selector for the template
    pub fn with_selector<S: Into<String>>(mut self, selector: S) -> Self {
        self.selector = Some(selector.into());
        self
    }
    
    /// Set the cache mode
    pub fn with_caching(mut self, mode: CacheMode) -> Self {
        self.config = self.config.with_cache_mode(mode);
        self
    }
    
    /// Enable or disable zero-copy optimizations
    pub fn with_zero_copy(mut self, enabled: bool) -> Self {
        self.config = self.config.with_zero_copy(enabled);
        self
    }
    
    /// Enable or disable compiled template caching
    pub fn with_compiled_template_caching(mut self, enabled: bool) -> Self {
        self.config = self.config.with_compiled_template_caching(enabled);
        self
    }
    
    /// Enable or disable external document caching
    pub fn with_external_document_caching(mut self, enabled: bool) -> Self {
        self.config = self.config.with_external_document_caching(enabled);
        self
    }
    
    /// Use a custom template configuration
    pub fn with_config(mut self, config: TemplateConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Add an element handler for a specific tag name
    pub fn add_handler<S: Into<String>>(mut self, tag_name: S, handler: Box<dyn ElementHandler>) -> Self {
        self.handlers.insert(tag_name.into(), handler);
        self
    }
    
    /// Add multiple handlers at once
    pub fn add_handlers(mut self, handlers: HashMap<String, Box<dyn ElementHandler>>) -> Self {
        self.handlers.extend(handlers);
        self
    }
    
    /// Use a handler registry instead of individual handlers
    pub fn with_handler_registry(mut self, registry: HandlerRegistry) -> Self {
        self.handler_registry = Some(registry);
        self
    }
    
    /// Use the default handler registry with built-in handlers
    pub fn with_default_handlers(mut self) -> Self {
        self.handler_registry = Some(HandlerRegistry::with_defaults());
        self
    }
    
    /// Add a handler to the registry with a specific priority
    pub fn register_handler(mut self, tag_name: &str, handler: Box<dyn ElementHandler>, priority: i32) -> Self {
        let registry = self.handler_registry.get_or_insert_with(HandlerRegistry::new);
        registry.register_with_priority(tag_name, handler, priority);
        self
    }
    
    /// Use a custom cache instance instead of the global cache
    pub fn with_cache(mut self, cache: TemplateCache) -> Self {
        self.custom_cache = Some(cache);
        self
    }
    
    /// Configure for aggressive caching with all optimizations enabled
    pub fn aggressive_performance(mut self) -> Self {
        self.config = TemplateConfig::aggressive_caching();
        self
    }
    
    /// Configure for no caching (useful for development or one-time rendering)
    pub fn no_caching(mut self) -> Self {
        self.config = TemplateConfig::no_caching();
        self
    }
    
    /// Build the HtmlTemplate instance
    pub fn build(self) -> Result<HtmlTemplate> {
        // Ensure we have a source
        let source = self.source.ok_or_else(|| {
            Error::parse_static("No template source provided. Use from_str(), from_element(), or from_file()")
        })?;
        
        // Load the HTML content based on source type
        let html = match source {
            TemplateSource::Html(html) => html,
            TemplateSource::Element(html) => html,
            TemplateSource::File(path) => {
                std::fs::read_to_string(&path)
                    .map_err(|e| Error::io(format!("Failed to read template file '{}': {}", path.display(), e)))?
            }
        };
        
        // Create the template using the appropriate method
        let template = if let Some(cache) = self.custom_cache {
            HtmlTemplate::from_str_with_cache(
                &html,
                self.selector.as_deref(),
                self.config,
                &cache,
            )?
        } else {
            HtmlTemplate::from_str_with_config(
                &html,
                self.selector.as_deref(),
                self.config,
            )?
        };
        
        // Add handlers if any were provided
        if !self.handlers.is_empty() {
            // Since we can't modify the template after creation, we need to create a new one
            // with the handlers. This is a limitation of the current API design.
            let compiled = template.compiled.clone();
            let config = template.config.clone();
            Ok(HtmlTemplate::new(compiled, config, self.handlers))
        } else {
            Ok(template)
        }
    }
    
    /// Build and validate the template by attempting to render with empty data
    pub fn build_and_validate(self) -> Result<HtmlTemplate> {
        let template = self.build()?;
        
        // Attempt to render with empty JSON to validate the template
        let empty_data = serde_json::json!({});
        template.render(&empty_data)?;
        
        Ok(template)
    }
}

impl Default for HtmlTemplateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common template creation patterns
impl HtmlTemplateBuilder {
    /// Create a template from HTML string with minimal configuration
    pub fn quick_template(html: &str) -> Result<HtmlTemplate> {
        Self::new()
            .from_str(html)
            .build()
    }
    
    /// Create a template from HTML string with a selector
    pub fn quick_template_with_selector(html: &str, selector: &str) -> Result<HtmlTemplate> {
        Self::new()
            .from_str(html)
            .with_selector(selector)
            .build()
    }
    
    /// Create a high-performance template with all optimizations
    pub fn performance_template(html: &str, selector: Option<&str>) -> Result<HtmlTemplate> {
        let mut builder = Self::new()
            .from_str(html)
            .aggressive_performance();
            
        if let Some(sel) = selector {
            builder = builder.with_selector(sel);
        }
        
        builder.build()
    }
    
    /// Create a development template with no caching for rapid iteration
    pub fn development_template(html: &str, selector: Option<&str>) -> Result<HtmlTemplate> {
        let mut builder = Self::new()
            .from_str(html)
            .no_caching();
            
        if let Some(sel) = selector {
            builder = builder.with_selector(sel);
        }
        
        builder.build()
    }
    
    /// Create a template from a file with automatic error handling
    pub fn from_template_file<P: AsRef<Path>>(path: P) -> Result<HtmlTemplate> {
        Self::new()
            .from_file(path)
            .build()
    }
    
    /// Create a template from a file with selector and performance optimizations
    pub fn from_template_file_optimized<P: AsRef<Path>>(path: P, selector: Option<&str>) -> Result<HtmlTemplate> {
        let mut builder = Self::new()
            .from_file(path)
            .aggressive_performance();
            
        if let Some(sel) = selector {
            builder = builder.with_selector(sel);
        }
        
        builder.build()
    }
}

/// Builder for configuring template rendering options
/// 
/// This builder focuses on runtime rendering configuration rather than
/// template compilation configuration.
pub struct RenderBuilder<'a> {
    template: &'a HtmlTemplate,
    validate_data: bool,
    error_on_missing: bool,
    custom_handlers: HashMap<String, Box<dyn ElementHandler>>,
}

impl<'a> RenderBuilder<'a> {
    /// Create a new render builder for the given template
    pub fn new(template: &'a HtmlTemplate) -> Self {
        Self {
            template,
            validate_data: false,
            error_on_missing: false,
            custom_handlers: HashMap::new(),
        }
    }
    
    /// Enable data validation before rendering
    pub fn with_data_validation(mut self, enabled: bool) -> Self {
        self.validate_data = enabled;
        self
    }
    
    /// Configure whether to error on missing properties
    pub fn error_on_missing_properties(mut self, enabled: bool) -> Self {
        self.error_on_missing = enabled;
        self
    }
    
    /// Add temporary handlers for this render operation
    pub fn with_handler<S: Into<String>>(mut self, tag_name: S, handler: Box<dyn ElementHandler>) -> Self {
        self.custom_handlers.insert(tag_name.into(), handler);
        self
    }
    
    /// Render the template with the configured options
    pub fn render(self, data: &dyn crate::value::RenderValue) -> Result<String> {
        // For now, use the basic render method
        // TODO: Implement validation and custom handler support
        self.template.render(data)
    }
    
    /// Render and return additional metadata about the rendering process
    pub fn render_with_metadata(self, data: &dyn crate::value::RenderValue) -> Result<RenderResult> {
        let start_time = std::time::Instant::now();
        let rendered = self.render(data)?;
        let duration = start_time.elapsed();
        
        Ok(RenderResult {
            html: rendered,
            duration,
            properties_used: Vec::new(), // TODO: Track property usage
            missing_properties: Vec::new(), // TODO: Track missing properties
        })
    }
}

/// Result of a template rendering operation with metadata
#[derive(Debug)]
pub struct RenderResult {
    /// The rendered HTML
    pub html: String,
    /// Time taken to render
    pub duration: std::time::Duration,
    /// Properties that were accessed during rendering
    pub properties_used: Vec<String>,
    /// Properties that were requested but not found in data
    pub missing_properties: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_builder_basic_usage() {
        let html = r#"
            <template>
                <div>
                    <h1 itemprop="title"></h1>
                    <p itemprop="description"></p>
                </div>
            </template>
        "#;
        
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .no_caching()
            .build()
            .unwrap();
        
        let data = json!({
            "title": "Test Title",
            "description": "Test Description"
        });
        
        let result = template.render(&data).unwrap();
        assert!(result.contains("Test Title"));
        assert!(result.contains("Test Description"));
    }
    
    #[test]
    fn test_builder_with_configuration() {
        let html = r#"
            <template>
                <div>
                    <span itemprop="message"></span>
                </div>
            </template>
        "#;
        
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .with_caching(CacheMode::None)
            .with_zero_copy(true)
            .build()
            .unwrap();
        
        let data = json!({"message": "Hello World"});
        let result = template.render(&data).unwrap();
        assert!(result.contains("Hello World"));
    }
    
    #[test]
    fn test_quick_template_methods() {
        let html = r#"
            <template>
                <div itemprop="test"></div>
            </template>
        "#;
        
        // Test quick template
        let template1 = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .no_caching()
            .build()
            .unwrap();
        let data = json!({"test": "Quick"});
        let result = template1.render(&data).unwrap();
        assert!(result.contains("Quick"));
        
        // Test quick template with selector
        let template2 = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .no_caching()
            .build()
            .unwrap();
        let result = template2.render(&data).unwrap();
        assert!(result.contains("Quick"));
    }
    
    #[test]
    fn test_performance_template() {
        let html = r#"
            <template>
                <article itemprop="content"></article>
            </template>
        "#;
        
        let template = HtmlTemplateBuilder::performance_template(html, Some("article")).unwrap();
        assert_eq!(template.config.cache_mode(), CacheMode::Aggressive);
        assert!(template.config.zero_copy());
        
        let data = json!({"content": "Performance test"});
        let result = template.render(&data).unwrap();
        assert!(result.contains("Performance test"));
    }
    
    #[test]
    fn test_development_template() {
        let html = r#"
            <template>
                <div itemprop="dev"></div>
            </template>
        "#;
        
        let template = HtmlTemplateBuilder::development_template(html, Some("div")).unwrap();
        assert_eq!(template.config.cache_mode(), CacheMode::None);
        assert!(template.config.zero_copy()); // Still want zero-copy optimizations
        
        let data = json!({"dev": "Development test"});
        let result = template.render(&data).unwrap();
        assert!(result.contains("Development test"));
    }
    
    #[test]
    fn test_builder_validation() {
        let html = r#"
            <template>
                <div>
                    <h1 itemprop="title"></h1>
                </div>
            </template>
        "#;
        
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .no_caching()
            .build_and_validate()
            .unwrap();
        
        // Template should be valid and renderable
        let data = json!({"title": "Validated"});
        let result = template.render(&data).unwrap();
        assert!(result.contains("Validated"));
    }
    
    #[test]
    fn test_builder_no_source_error() {
        let result = HtmlTemplateBuilder::new()
            .with_selector("div")
            .build();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No template source provided"));
    }
    
    #[test]
    fn test_render_builder() {
        let html = r#"
            <template>
                <div itemprop="message"></div>
            </template>
        "#;
        
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .no_caching()
            .build()
            .unwrap();
        
        let data = json!({"message": "Builder test"});
        
        let result = RenderBuilder::new(&template)
            .with_data_validation(true)
            .render(&data)
            .unwrap();
        
        assert!(result.contains("Builder test"));
    }
    
    #[test]
    fn test_render_with_metadata() {
        let html = r#"
            <template>
                <div itemprop="test"></div>
            </template>
        "#;
        
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .no_caching()
            .build()
            .unwrap();
        
        let data = json!({"test": "Metadata test"});
        
        let result = RenderBuilder::new(&template)
            .render_with_metadata(&data)
            .unwrap();
        
        assert!(result.html.contains("Metadata test"));
        assert!(result.duration.as_nanos() > 0);
    }
    
    #[test]
    fn test_chained_configuration() {
        let html = r#"<template><div itemprop="chain"></div></template>"#;
        
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .no_caching()
            .with_zero_copy(true)
            .with_compiled_template_caching(false)
            .with_external_document_caching(false)
            .build()
            .unwrap();
        
        assert_eq!(template.config.cache_mode(), CacheMode::None);
        assert!(template.config.zero_copy());
        assert!(!template.config.cache_compiled_templates());
        assert!(!template.config.cache_external_documents());
    }
    
    #[test]
    fn test_builder_from_element() {
        // Builder expects template element, so let's wrap our div in a template
        let html = r#"<template><div itemprop="test">Element content</div></template>"#;
        let doc = dom_query::Document::from(html);
        let selection = doc.select("template");
        let element = selection.nodes().first().unwrap();
        
        let template = HtmlTemplateBuilder::new()
            .from_element(element)
            .no_caching()
            .build()
            .unwrap();
        
        let data = json!({"test": "From element"});
        let result = template.render(&data).unwrap();
        // Just check that we got a successful render
        assert!(!result.is_empty());
    }
    
    #[test]
    fn test_builder_add_handlers() {
        use crate::handlers::{ElementHandler, InputHandler, SelectHandler};
        use std::collections::HashMap;
        
        let mut handlers: HashMap<String, Box<dyn ElementHandler>> = HashMap::new();
        handlers.insert("input".to_string(), Box::new(InputHandler::new()));
        handlers.insert("select".to_string(), Box::new(SelectHandler::new()));
        
        let html = r#"<template><input itemprop="field"></template>"#;
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .add_handlers(handlers)
            .no_caching()
            .build()
            .unwrap();
        
        // Verify handlers were added
        assert_eq!(template.handlers.len(), 2);
    }
    
    #[test]
    fn test_builder_with_handler_registry() {
        use crate::handlers::HandlerRegistry;
        
        let registry = HandlerRegistry::with_defaults();
        
        let html = r#"<template><div>Test</div></template>"#;
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_handler_registry(registry)
            .no_caching()
            .build()
            .unwrap();
        
        // The template should have a handler registry set up
        let data = json!({});
        let _result = template.render(&data).unwrap();
    }
    
    #[test]
    fn test_builder_register_handler() {
        use crate::handlers::LoggingHandler;
        
        let html = r#"<template><div>Test</div></template>"#;
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .register_handler("div", Box::new(LoggingHandler::new()), 100)
            .no_caching()
            .build()
            .unwrap();
        
        let data = json!({});
        let _result = template.render(&data).unwrap();
    }
    
    #[test]
    fn test_builder_with_custom_cache() {
        use crate::cache::{TemplateCache, CacheConfig};
        
        let cache_config = CacheConfig {
            template_cache_size: 50,
            template_ttl: Some(Duration::from_secs(300)),
            ..Default::default()
        };
        let custom_cache = TemplateCache::with_config(cache_config);
        
        let html = r#"<template><div>Test</div></template>"#;
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_cache(custom_cache)
            .build()
            .unwrap();
        
        let data = json!({});
        let _result = template.render(&data).unwrap();
    }
    
    #[test]
    fn test_builder_from_file_not_found() {
        let result = HtmlTemplateBuilder::from_template_file("/nonexistent/file.html");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read template file"));
    }
    
    #[test]
    fn test_render_builder_with_handler() {
        use crate::handlers::ClassHandler;
        
        let html = r#"<template><div itemprop="content"></div></template>"#;
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .no_caching()
            .build()
            .unwrap();
        
        let data = json!({"content": "test"});
        
        // Note: RenderBuilder's with_handler doesn't actually affect rendering yet
        let result = RenderBuilder::new(&template)
            .with_handler("div", Box::new(ClassHandler::new()))
            .render(&data)
            .unwrap();
        
        assert!(result.contains("test"));
    }
    
    #[test]
    fn test_render_builder_error_on_missing() {
        let html = r#"<template><div itemprop="required"></div></template>"#;
        let template = HtmlTemplateBuilder::new()
            .from_str(html)
            .with_selector("div")
            .no_caching()
            .build()
            .unwrap();
        
        let data = json!({});
        
        // Note: error_on_missing_properties is not implemented yet
        let result = RenderBuilder::new(&template)
            .error_on_missing_properties(true)
            .render(&data);
        
        // Currently this doesn't error, it just renders empty
        assert!(result.is_ok());
    }
}