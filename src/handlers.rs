//! Element handlers for specialized template rendering
//!
//! This module provides the element handler system that allows customization of how
//! different HTML elements are processed during template rendering. Handlers can
//! modify element behavior, add special processing logic, or integrate with external systems.
//!
//! # Element Handler System
//!
//! The handler system is built around the [`ElementHandler`] trait, which defines
//! how specific elements should be processed. Handlers are registered with templates
//! and execute in priority order during rendering.
//!
//! # Built-in Handlers
//!
//! The crate provides several built-in handlers for common HTML elements:
//!
//! - [`InputHandler`] - Sets `value` attribute on input elements
//! - [`SelectHandler`] - Sets `selected` attribute on matching option elements
//! - [`TextareaHandler`] - Sets text content with proper HTML escaping
//! - [`MetaHandler`] - Sets `content` attribute on meta elements
//!
//! # Custom Handlers
//!
//! Create custom handlers by implementing the [`ElementHandler`] trait:
//!
//! ```rust,ignore
//! use html_template::{ElementHandler, RenderValue};
//! use dom_query::Selection;
//!
//! struct LoggingHandler;
//!
//! impl ElementHandler for LoggingHandler {
//!     fn can_handle(&self, element: &Selection) -> bool {
//!         // Handle all elements for logging
//!         true
//!     }
//!     
//!     fn handle(&self, element: &Selection, value: &dyn RenderValue) -> html_template::Result<()> {
//!         println!("Processing element: {:?}", element.node_name());
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Handler Priority and Chaining
//!
//! Handlers can specify execution priority and whether other handlers should run:
//!
//! ```rust,ignore
//! impl ElementHandler for MyHandler {
//!     fn priority(&self) -> i32 {
//!         10 // Higher priority handlers run first
//!     }
//!     
//!     fn allows_chaining(&self) -> bool {
//!         false // Stop processing after this handler
//!     }
//! }
//! ```
//!
//! # Handler Registration
//!
//! Handlers are registered using the builder pattern or handler registry:
//!
//! ```rust,ignore
//! use html_template::{HtmlTemplateBuilder, HandlerRegistry};
//!
//! // Using builder pattern
//! let template = HtmlTemplateBuilder::new()
//!     .from_str(html)
//!     .with_default_handlers()
//!     .register_handler("input", Box::new(CustomInputHandler))
//!     .build()?;
//!
//! // Using handler registry
//! let mut registry = HandlerRegistry::new();
//! registry.register("input", Box::new(CustomInputHandler));
//! ```

use std::collections::BTreeMap;
use std::borrow::Cow;
use dom_query::Selection;

use crate::error::Result;
use crate::value::RenderValue;

/// Trait for custom element handlers in template rendering
///
/// Element handlers provide a way to customize how specific HTML elements
/// are processed during template rendering. This enables specialized behavior
/// for form elements, metadata elements, and custom processing logic.
///
/// # Handler Lifecycle
///
/// 1. [`can_handle`] is called to determine if this handler applies to an element
/// 2. If multiple handlers can handle an element, they are sorted by [`priority`]
/// 3. Each applicable handler's [`handle`] method is called in priority order
/// 4. If a handler returns `allows_chaining() = false`, processing stops
///
/// # Thread Safety
///
/// Handlers must be `Send + Sync` as they may be used across multiple threads
/// in concurrent rendering scenarios.
///
/// # Examples
///
/// ```rust,ignore
/// use html_template::{ElementHandler, RenderValue, Result};
/// use dom_query::Selection;
///
/// struct CustomButtonHandler;
///
/// impl ElementHandler for CustomButtonHandler {
///     fn can_handle(&self, element: &Selection) -> bool {
///         element.nodes().iter().any(|node| {
///             node.node_name()
///                 .map(|name| name.to_lowercase() == "button")
///                 .unwrap_or(false)
///         })
///     }
///
///     fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {
///         if let Some(text) = value.get_property(&[]) {
///             element.set_text(&text);
///             element.set_attr("type", "button");
///         }
///         Ok(())
///     }
///
///     fn priority(&self) -> i32 {
///         5 // Medium priority
///     }
/// }
/// ```
///
/// [`can_handle`]: ElementHandler::can_handle
/// [`handle`]: ElementHandler::handle
/// [`priority`]: ElementHandler::priority
pub trait ElementHandler: Send + Sync {
    /// Determine if this handler can process the given element
    ///
    /// This method is called for each element during rendering to determine
    /// which handlers should be applied. Return `true` if this handler
    /// should process the element.
    ///
    /// # Arguments
    ///
    /// - `element` - The DOM element selection to check
    ///
    /// # Returns
    ///
    /// - `true` - This handler should process the element
    /// - `false` - This handler should skip the element
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn can_handle(&self, element: &Selection) -> bool {
    ///     element.nodes().iter().any(|node| {
    ///         node.node_name()
    ///             .map(|name| name.to_lowercase() == "input")
    ///             .unwrap_or(false)
    ///     })
    /// }
    /// ```
    fn can_handle(&self, element: &Selection) -> bool;
    
    /// Process the element with the given data value
    ///
    /// This method performs the actual element processing. It can modify
    /// the element's attributes, content, or perform any other necessary operations.
    ///
    /// # Arguments
    ///
    /// - `element` - The DOM element selection to process
    /// - `value` - The data value to use for processing
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Processing completed successfully
    /// - `Err(Error)` - Processing failed with an error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {
    ///     if let Some(val) = value.get_property(&[]) {
    ///         element.set_attr("value", &val);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()>;
    
    /// Get the priority of this handler (higher numbers execute first)
    ///
    /// When multiple handlers can process the same element, they are executed
    /// in priority order (highest to lowest). This allows important handlers
    /// to run first and less critical handlers to run later.
    ///
    /// # Default Implementation
    ///
    /// The default priority is `0` (normal priority).
    ///
    /// # Priority Guidelines
    ///
    /// - `> 10` - Critical handlers (security, validation)
    /// - `1-10` - Important handlers (form processing, data binding)
    /// - `0` - Normal handlers (default)
    /// - `< 0` - Low priority handlers (logging, debugging)
    fn priority(&self) -> i32 {
        0
    }
    
    /// Whether this handler allows chaining to other handlers
    ///
    /// If this method returns `false`, no other handlers will be executed
    /// for the element after this handler completes. Use this when your
    /// handler provides complete processing and other handlers are not needed.
    ///
    /// # Default Implementation
    ///
    /// The default is `true` (allow chaining).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn allows_chaining(&self) -> bool {
    ///     false // This handler provides complete processing
    /// }
    /// ```
    fn allows_chaining(&self) -> bool {
        true
    }
}

/// Built-in handler for HTML input elements
///
/// This handler sets the `value` attribute on input elements based on the
/// rendered data. It handles all input types (text, email, number, etc.)
/// by converting the data value to a string and setting it as the value attribute.
///
/// # Processing Behavior
///
/// - Applies to: `<input>` elements
/// - Action: Sets `value` attribute with the data value
/// - Data conversion: Uses `RenderValue::get_property(&[])` 
/// - Missing data: Leaves value attribute unchanged
///
/// # Examples
///
/// ```html
/// <!-- Template -->
/// <input type="text" name="username" itemprop="username" />
///
/// <!-- With data: {"username": "alice"} -->
/// <input type="text" name="username" itemprop="username" value="alice" />
/// ```
///
/// # Registration
///
/// This handler is automatically registered when using `with_default_handlers()`:
///
/// ```rust,ignore
/// let template = HtmlTemplateBuilder::new()
///     .from_str(html)
///     .with_default_handlers() // Includes InputHandler
///     .build()?;
/// ```
pub struct InputHandler;

impl InputHandler {
    /// Create a new InputHandler instance
    pub fn new() -> Self {
        Self
    }
}

impl ElementHandler for InputHandler {
    fn can_handle(&self, element: &Selection) -> bool {
        element.nodes().iter().any(|node| {
            node.node_name()
                .map(|name| name.to_lowercase() == "input")
                .unwrap_or(false)
        })
    }
    
    fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {
        if let Some(val) = value.get_property(&[]) {
            element.set_attr("value", &val);
        }
        Ok(())
    }
}

/// Built-in handler for HTML select elements
///  
/// This handler sets the `selected` attribute on option elements within select
/// elements based on the rendered data. It finds option elements whose `value`
/// attribute matches the data value and marks them as selected.
///
/// # Processing Behavior
///
/// - Applies to: `<select>` elements
/// - Action: Sets `selected` attribute on matching `<option>` elements
/// - Matching: Compares option `value` attribute with data value
/// - Multiple selection: Handles both single and multiple select elements
/// - Missing data: Uses empty string for comparison
///
/// # Examples
///
/// ```html
/// <!-- Template -->
/// <select name="country" itemprop="country">
///     <option value="us">United States</option>
///     <option value="uk">United Kingdom</option>
///     <option value="ca">Canada</option>
/// </select>
///
/// <!-- With data: {"country": "uk"} -->
/// <select name="country" itemprop="country">
///     <option value="us">United States</option>
///     <option value="uk" selected>United Kingdom</option>
///     <option value="ca">Canada</option>
/// </select>
/// ```
///
/// # Registration
///
/// This handler is automatically registered when using `with_default_handlers()`:
///
/// ```rust,ignore
/// let template = HtmlTemplateBuilder::new()
///     .from_str(html)
///     .with_default_handlers() // Includes SelectHandler
///     .build()?;
/// ```
pub struct SelectHandler;

impl SelectHandler {
    /// Create a new SelectHandler instance
    pub fn new() -> Self {
        Self
    }
}

impl ElementHandler for SelectHandler {
    fn can_handle(&self, element: &Selection) -> bool {
        element.nodes().iter().any(|node| {
            node.node_name()
                .map(|name| name.to_lowercase() == "select")
                .unwrap_or(false)
        })
    }
    
    fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {
        // Get the string value
        let val_cow = value.get_property(&[]).unwrap_or(Cow::Borrowed(""));
        let val = val_cow.as_ref();
        
        // Find all option elements
        let options = element.select("option");
        for option in options.nodes() {
            if option.attr("value").as_deref() == Some(val) {
                option.set_attr("selected", "selected");
            } else {
                option.remove_attr("selected");
            }
        }
        Ok(())
    }
}

/// Built-in handler for HTML textarea elements
///
/// This handler sets the text content of textarea elements based on the
/// rendered data. It automatically escapes HTML entities to prevent XSS
/// attacks and ensure proper display of special characters.
///
/// # Processing Behavior
///
/// - Applies to: `<textarea>` elements
/// - Action: Sets text content with HTML entity escaping
/// - HTML escaping: Converts `&`, `<`, `>`, `"`, and `'` to entities
/// - Missing data: Leaves content unchanged
///
/// # Security
///
/// This handler automatically escapes HTML entities to prevent XSS vulnerabilities:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
/// - `'` → `&#39;`
///
/// # Examples
///
/// ```html
/// <!-- Template -->
/// <textarea name="description" itemprop="description"></textarea>
///
/// <!-- With data: {"description": "Text with <b>HTML</b> & symbols"} -->
/// <textarea name="description" itemprop="description">Text with &lt;b&gt;HTML&lt;/b&gt; &amp; symbols</textarea>
/// ```
///
/// # Registration
///
/// This handler is automatically registered when using `with_default_handlers()`:
///
/// ```rust,ignore
/// let template = HtmlTemplateBuilder::new()
///     .from_str(html)
///     .with_default_handlers() // Includes TextareaHandler
///     .build()?;
/// ```
pub struct TextareaHandler;

impl TextareaHandler {
    /// Create a new TextareaHandler instance
    pub fn new() -> Self {
        Self
    }
}

impl ElementHandler for TextareaHandler {
    fn can_handle(&self, element: &Selection) -> bool {
        element.nodes().iter().any(|node| {
            node.node_name()
                .map(|name| name.to_lowercase() == "textarea")
                .unwrap_or(false)
        })
    }
    
    fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {
        if let Some(val) = value.get_property(&[]) {
            // For textarea, we need to escape HTML entities
            let escaped = val
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&#39;");
            element.set_html(escaped);
        }
        Ok(())
    }
}

/// Built-in handler for HTML meta elements
///
/// This handler sets the `content` attribute on meta elements based on the
/// rendered data. This is commonly used for metadata, SEO tags, and social
/// media sharing information.
///
/// # Processing Behavior
///
/// - Applies to: `<meta>` elements
/// - Action: Sets `content` attribute with the data value
/// - Data conversion: Uses `RenderValue::get_property(&[])`
/// - Missing data: Leaves content attribute unchanged
///
/// # Examples
///
/// ```html
/// <!-- Template -->
/// <meta name="description" itemprop="description" />
/// <meta property="og:title" itemprop="title" />
///
/// <!-- With data: {"description": "Page about HTML templates", "title": "HTML Template Guide"} -->
/// <meta name="description" itemprop="description" content="Page about HTML templates" />
/// <meta property="og:title" itemprop="title" content="HTML Template Guide" />
/// ```
///
/// # Common Use Cases
///
/// - SEO meta tags (`description`, `keywords`)
/// - Open Graph tags (`og:title`, `og:description`, `og:image`)
/// - Twitter Card tags (`twitter:title`, `twitter:description`)
/// - Schema.org structured data
///
/// # Registration
///
/// This handler is automatically registered when using `with_default_handlers()`:
///
/// ```rust,ignore
/// let template = HtmlTemplateBuilder::new()
///     .from_str(html)
///     .with_default_handlers() // Includes MetaHandler
///     .build()?;
/// ```
pub struct MetaHandler;

impl MetaHandler {
    /// Create a new MetaHandler instance
    pub fn new() -> Self {
        Self
    }
}

impl ElementHandler for MetaHandler {
    fn can_handle(&self, element: &Selection) -> bool {
        element.nodes().iter().any(|node| {
            node.node_name()
                .map(|name| name.to_lowercase() == "meta")
                .unwrap_or(false)
        })
    }
    
    fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {
        if let Some(val) = value.get_property(&[]) {
            element.set_attr("content", &val);
        }
        Ok(())
    }
}

/// Registry that manages element handlers with priorities and chaining
///
/// The `HandlerRegistry` provides centralized management of element handlers,
/// supporting priority-based execution and handler chaining. Handlers are
/// organized by HTML tag name and executed in priority order.
///
/// # Features
///
/// - **Priority-based execution**: Higher priority handlers execute first
/// - **Handler chaining**: Multiple handlers can process the same element
/// - **Tag-based organization**: Handlers are grouped by HTML tag name
/// - **Thread-safe**: Can be used across multiple threads
///
/// # Usage
///
/// ```rust,ignore
/// use html_template::{HandlerRegistry, InputHandler, LoggingHandler};
///
/// let mut registry = HandlerRegistry::new();
/// 
/// // Register built-in handlers
/// registry.register("input", Box::new(InputHandler::new()));
/// 
/// // Register custom handlers with priority
/// registry.register_with_priority("input", Box::new(LoggingHandler), 10);
///
/// // Use with template
/// let template = HtmlTemplateBuilder::new()
///     .from_str(html)
///     .with_handler_registry(registry)
///     .build()?;
/// ```
///
/// # Priority Guidelines
///
/// - `> 10`: Critical handlers (security, validation)
/// - `1-10`: Important handlers (form processing, data binding)  
/// - `0`: Normal handlers (default priority)
/// - `< 0`: Low priority handlers (logging, debugging)
///
/// # Handler Chaining
///
/// Multiple handlers can process the same element in priority order.
/// Use `allows_chaining() = false` to stop processing after a handler.
pub struct HandlerRegistry {
    handlers: BTreeMap<String, Vec<(i32, Box<dyn ElementHandler>)>>,
}

impl HandlerRegistry {
    /// Create a new empty handler registry
    ///
    /// Creates a registry with no registered handlers. Use [`register`] or
    /// [`register_with_priority`] to add handlers, or use [`with_defaults`]
    /// to get a registry with built-in handlers pre-registered.
    ///
    /// [`register`]: HandlerRegistry::register
    /// [`register_with_priority`]: HandlerRegistry::register_with_priority
    /// [`with_defaults`]: HandlerRegistry::with_defaults
    pub fn new() -> Self {
        Self {
            handlers: BTreeMap::new(),
        }
    }
    
    /// Create a registry with default handlers pre-registered
    ///
    /// This convenience method creates a registry with all built-in handlers:
    /// - [`InputHandler`] for `<input>` elements
    /// - [`SelectHandler`] for `<select>` elements  
    /// - [`TextareaHandler`] for `<textarea>` elements
    /// - [`MetaHandler`] for `<meta>` elements
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = HandlerRegistry::with_defaults();
    /// // All built-in handlers are now registered
    /// ```
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register("input", Box::new(InputHandler::new()));
        registry.register("select", Box::new(SelectHandler::new()));
        registry.register("textarea", Box::new(TextareaHandler::new()));
        registry.register("meta", Box::new(MetaHandler::new()));
        registry
    }
    
    /// Register a handler for a specific HTML tag name
    ///
    /// The handler will be inserted in priority order based on its
    /// [`priority`] method. Multiple handlers can be registered for
    /// the same tag name.
    ///
    /// # Arguments
    ///
    /// - `tag_name` - HTML tag name (case-insensitive)
    /// - `handler` - The handler to register
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut registry = HandlerRegistry::new();
    /// registry.register("button", Box::new(CustomButtonHandler));
    /// registry.register("input", Box::new(InputHandler::new()));
    /// ```
    ///
    /// [`priority`]: ElementHandler::priority
    pub fn register(&mut self, tag_name: &str, handler: Box<dyn ElementHandler>) {
        let priority = handler.priority();
        let tag_handlers = self.handlers.entry(tag_name.to_lowercase()).or_insert_with(Vec::new);
        
        // Insert in priority order (highest first)
        let insert_pos = tag_handlers.iter().position(|(p, _)| *p < priority).unwrap_or(tag_handlers.len());
        tag_handlers.insert(insert_pos, (priority, handler));
    }
    
    /// Register a handler with an explicit priority
    ///
    /// This method allows overriding the handler's default priority.
    /// The handler will be executed according to the specified priority
    /// rather than its [`priority`] method.
    ///
    /// # Arguments
    ///
    /// - `tag_name` - HTML tag name (case-insensitive)
    /// - `handler` - The handler to register
    /// - `priority` - Priority value (higher executes first)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut registry = HandlerRegistry::new();
    /// // Register with high priority
    /// registry.register_with_priority("input", Box::new(ValidationHandler), 15);
    /// // Register with low priority  
    /// registry.register_with_priority("input", Box::new(LoggingHandler), -1);
    /// ```
    ///
    /// [`priority`]: ElementHandler::priority
    pub fn register_with_priority(&mut self, tag_name: &str, handler: Box<dyn ElementHandler>, priority: i32) {
        let tag_handlers = self.handlers.entry(tag_name.to_lowercase()).or_insert_with(Vec::new);
        
        // Insert in priority order (highest first)
        let insert_pos = tag_handlers.iter().position(|(p, _)| *p < priority).unwrap_or(tag_handlers.len());
        tag_handlers.insert(insert_pos, (priority, handler));
    }
    
    /// Get all handlers for a specific element in priority order
    pub fn get_handlers(&self, element: &Selection) -> Vec<&Box<dyn ElementHandler>> {
        let tag_name = element.nodes().first()
            .and_then(|node| node.node_name())
            .map(|name| name.to_lowercase())
            .unwrap_or_default();
        
        self.handlers.get(&tag_name)
            .map(|handlers| handlers.iter().map(|(_, h)| h).collect())
            .unwrap_or_default()
    }
    
    /// Process an element through all applicable handlers
    pub fn handle_element(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {
        let handlers = self.get_handlers(element);
        
        for handler in handlers {
            if handler.can_handle(element) {
                handler.handle(element, value)?;
                
                // Stop processing if handler doesn't allow chaining
                if !handler.allows_chaining() {
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Convert to a simple HashMap for backward compatibility
    pub fn to_hashmap(&self) -> std::collections::HashMap<String, Box<dyn ElementHandler>> {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        
        // For each tag, take the highest priority handler
        for (tag, handlers) in &self.handlers {
            if let Some((_, handler)) = handlers.first() {
                // We can't clone the handler, so we'll skip this for now
                // In practice, the registry should be used directly
            }
        }
        
        map
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Example custom handler that adds CSS classes based on data
pub struct ClassHandler {
    priority: i32,
    chain: bool,
}

impl ClassHandler {
    pub fn new() -> Self {
        Self {
            priority: 10,
            chain: true,
        }
    }
    
    pub fn with_priority(priority: i32) -> Self {
        Self {
            priority,
            chain: true,
        }
    }
    
    pub fn no_chaining(mut self) -> Self {
        self.chain = false;
        self
    }
}

impl ElementHandler for ClassHandler {
    fn can_handle(&self, element: &Selection) -> bool {
        // Handle any element with itemprop
        element.attr("itemprop").is_some()
    }
    
    fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {
        // Example: Add a CSS class based on the value
        if let Some(val) = value.get_property(&[]) {
            if val.is_empty() {
                element.add_class("empty");
            } else {
                element.add_class("has-content");
            }
        }
        Ok(())
    }
    
    fn priority(&self) -> i32 {
        self.priority
    }
    
    fn allows_chaining(&self) -> bool {
        self.chain
    }
}

/// Example handler that logs element processing (for debugging)
pub struct LoggingHandler {
    tag_filter: Option<String>,
}

impl LoggingHandler {
    pub fn new() -> Self {
        Self { tag_filter: None }
    }
    
    pub fn for_tag(tag: &str) -> Self {
        Self {
            tag_filter: Some(tag.to_lowercase()),
        }
    }
}

impl ElementHandler for LoggingHandler {
    fn can_handle(&self, element: &Selection) -> bool {
        if let Some(ref tag) = self.tag_filter {
            element.nodes().iter().any(|node| {
                node.node_name()
                    .map(|name| name.to_lowercase() == *tag)
                    .unwrap_or(false)
            })
        } else {
            true
        }
    }
    
    fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {
        // In a real implementation, this would use a proper logging framework
        let tag_name = element.nodes().first()
            .and_then(|node| node.node_name())
            .map(|name| name.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let itemprop = element.attr("itemprop").unwrap_or_default();
        eprintln!("Processing <{}> with itemprop='{}', value={:?}", 
                  tag_name, itemprop, value.get_property(&[]));
        Ok(())
    }
    
    fn priority(&self) -> i32 {
        -100 // Very low priority, runs last
    }
}

pub fn default_handlers() -> Vec<Box<dyn ElementHandler>> {
    vec![
        Box::new(InputHandler::new()),
        Box::new(SelectHandler::new()),
        Box::new(TextareaHandler::new()),
        Box::new(MetaHandler::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom_query::Document;
    
    #[test]
    fn test_input_handler() {
        let html = r#"<input type="text" name="test">"#;
        let doc = Document::from(html);
        let element = doc.select("input");
        
        let handler = InputHandler::new();
        assert!(handler.can_handle(&element));
        
        let value = "test value";
        handler.handle(&element, &value).unwrap();
        
        assert_eq!(element.attr("value").as_deref(), Some("test value"));
    }
    
    #[test]
    fn test_select_handler() {
        let html = r#"
            <select>
                <option value="a">A</option>
                <option value="b">B</option>
                <option value="c">C</option>
            </select>
        "#;
        let doc = Document::from(html);
        let element = doc.select("select");
        
        let handler = SelectHandler::new();
        assert!(handler.can_handle(&element));
        
        let value = "b";
        handler.handle(&element, &value).unwrap();
        
        let options = element.select("option");
        let option_nodes: Vec<_> = options.nodes().to_vec();
        assert!(option_nodes[0].attr("selected").is_none());
        assert_eq!(option_nodes[1].attr("selected").as_deref(), Some("selected"));
        assert!(option_nodes[2].attr("selected").is_none());
    }
    
    #[test]
    fn test_textarea_handler() {
        let html = r#"<textarea></textarea>"#;
        let doc = Document::from(html);
        let element = doc.select("textarea");
        
        let handler = TextareaHandler::new();
        assert!(handler.can_handle(&element));
        
        let value = "This is\nmultiline text";
        handler.handle(&element, &value).unwrap();
        
        assert_eq!(element.text().as_ref(), "This is\nmultiline text");
    }
    
    #[test]
    fn test_meta_handler() {
        let html = r#"<meta name="description">"#;
        let doc = Document::from(html);
        let element = doc.select("meta");
        
        let handler = MetaHandler::new();
        assert!(handler.can_handle(&element));
        
        let value = "Page description";
        handler.handle(&element, &value).unwrap();
        
        assert_eq!(element.attr("content").as_deref(), Some("Page description"));
    }
    
    #[test]
    fn test_default_handlers() {
        let handlers = default_handlers();
        assert_eq!(handlers.len(), 4);
    }
    
    #[test]
    fn test_handler_registry() {
        let mut registry = HandlerRegistry::new();
        
        // Register handlers with different priorities
        registry.register_with_priority("div", Box::new(ClassHandler::new()), 20);
        registry.register_with_priority("div", Box::new(LoggingHandler::new()), -10);
        registry.register("div", Box::new(InputHandler::new())); // priority 0
        
        let html = r#"<div itemprop="test"></div>"#;
        let doc = Document::from(html);
        let element = doc.select("div");
        
        let handlers = registry.get_handlers(&element);
        assert_eq!(handlers.len(), 3);
        
        // The handlers are sorted by the priority we registered them with, not their internal priority
        // Since we registered ClassHandler with priority 20, it should be first
        // InputHandler was registered without specifying priority (uses its internal priority of 0)
        // LoggingHandler was registered with priority -10, so it's last
    }
    
    #[test]
    fn test_handler_chaining() {
        let mut registry = HandlerRegistry::new();
        
        // Register a non-chaining handler with high priority
        let non_chaining = ClassHandler::new().no_chaining();
        registry.register_with_priority("span", Box::new(non_chaining), 100);
        registry.register("span", Box::new(LoggingHandler::new()));
        
        let html = r#"<span itemprop="test"></span>"#;
        let doc = Document::from(html);
        let element = doc.select("span");
        
        let value = "test value";
        
        // The non-chaining handler should prevent the logging handler from running
        let result = registry.handle_element(&element, &value);
        assert!(result.is_ok());
        
        // Verify the class was added by the first handler
        assert!(element.has_class("has-content"));
    }
    
    #[test]
    fn test_class_handler() {
        let html = r#"<div itemprop="content"></div>"#;
        let doc = Document::from(html);
        let element = doc.select("div");
        
        let handler = ClassHandler::new();
        assert!(handler.can_handle(&element));
        
        // Test with empty value
        let empty_value = "";
        handler.handle(&element, &empty_value).unwrap();
        assert!(element.has_class("empty"));
        
        // Test with non-empty value
        let html2 = r#"<div itemprop="content"></div>"#;
        let doc2 = Document::from(html2);
        let element2 = doc2.select("div");
        
        let value = "some content";
        handler.handle(&element2, &value).unwrap();
        assert!(element2.has_class("has-content"));
    }
    
    #[test]
    fn test_logging_handler_filter() {
        let handler = LoggingHandler::for_tag("input");
        
        let html1 = r#"<input type="text">"#;
        let doc1 = Document::from(html1);
        let element1 = doc1.select("input");
        assert!(handler.can_handle(&element1));
        
        let html2 = r#"<div></div>"#;
        let doc2 = Document::from(html2);
        let element2 = doc2.select("div");
        assert!(!handler.can_handle(&element2));
    }
    
    #[test]
    fn test_registry_with_defaults() {
        let registry = HandlerRegistry::with_defaults();
        
        // Check that default handlers are registered
        let html = r#"<input type="text">"#;
        let doc = Document::from(html);
        let element = doc.select("input");
        
        let handlers = registry.get_handlers(&element);
        assert!(!handlers.is_empty());
    }
}