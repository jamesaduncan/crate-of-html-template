use std::sync::Arc;
use std::collections::HashMap;
use dom_query::Document;

use crate::handlers::ElementHandler;
use crate::error::{Error, Result};
use crate::value::RenderValue;
use crate::cache::{TemplateCache, TemplateCacheKey, get_global_cache};

pub struct HtmlTemplate {
    pub(crate) compiled: Arc<CompiledTemplate>,
    pub(crate) config: TemplateConfig,
    pub(crate) handlers: HashMap<String, Box<dyn ElementHandler>>,
}

impl Clone for HtmlTemplate {
    fn clone(&self) -> Self {
        Self {
            compiled: self.compiled.clone(),
            config: self.config.clone(),
            // Note: handlers are not cloned as ElementHandler doesn't implement Clone
            // This means cached templates will have empty handlers
            handlers: HashMap::new(),
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
        }
    }
    
    /// Create a template from HTML string and selector
    pub fn from_str(html: &str, selector: Option<&str>) -> Result<Self> {
        Self::from_str_with_config(html, selector, TemplateConfig::default())
    }
    
    /// Create a template from HTML string with custom configuration
    pub fn from_str_with_config(html: &str, selector: Option<&str>, config: TemplateConfig) -> Result<Self> {
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
        
        Ok(Self::new(compiled, config, std::collections::HashMap::new()))
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
        
        Ok(Self::new(compiled, config, std::collections::HashMap::new()))
    }
    
    /// Render the template with the given data
    pub fn render(&self, data: &dyn RenderValue) -> Result<String> {
        let renderer = crate::renderer::Renderer::new(&self.compiled, &self.handlers);
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
    pub(crate) base_uri: Option<String>,
    pub(crate) template_html: String,
}

#[derive(Debug, Clone)]
pub struct TemplateElement {
    pub(crate) selector: String,
    pub(crate) properties: Vec<Property>,
    pub(crate) is_array: bool,
    pub(crate) is_scope: bool,
    pub(crate) itemtype: Option<String>,
    pub(crate) constraints: Vec<ConstraintRef>,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub(crate) name: String,
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
    pub fn cache_mode(&self) -> CacheMode { self.cache_mode }
    pub fn zero_copy(&self) -> bool { self.zero_copy }
    pub fn cache_compiled_templates(&self) -> bool { self.cache_compiled_templates }
    pub fn cache_external_documents(&self) -> bool { self.cache_external_documents }
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
    pub(crate) scope: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConstraintType {
    Scope(String),
    Expression(String),
}

#[derive(Debug, Clone)]
pub struct ConstraintRef {
    pub(crate) constraint_index: usize,
}

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
}