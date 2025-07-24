use std::collections::BTreeMap;
use std::borrow::Cow;
use dom_query::Selection;

use crate::error::Result;
use crate::value::RenderValue;

pub trait ElementHandler: Send + Sync {
    fn can_handle(&self, element: &Selection) -> bool;
    fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()>;
    
    /// Get the priority of this handler (higher numbers execute first)
    fn priority(&self) -> i32 {
        0
    }
    
    /// Whether this handler allows chaining to other handlers
    fn allows_chaining(&self) -> bool {
        true
    }
}

pub struct InputHandler;

impl InputHandler {
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

pub struct SelectHandler;

impl SelectHandler {
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

pub struct TextareaHandler;

impl TextareaHandler {
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

pub struct MetaHandler;

impl MetaHandler {
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

/// Handler registry that manages handlers with priorities and chaining
pub struct HandlerRegistry {
    handlers: BTreeMap<String, Vec<(i32, Box<dyn ElementHandler>)>>,
}

impl HandlerRegistry {
    /// Create a new handler registry
    pub fn new() -> Self {
        Self {
            handlers: BTreeMap::new(),
        }
    }
    
    /// Create a registry with default handlers
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register("input", Box::new(InputHandler::new()));
        registry.register("select", Box::new(SelectHandler::new()));
        registry.register("textarea", Box::new(TextareaHandler::new()));
        registry.register("meta", Box::new(MetaHandler::new()));
        registry
    }
    
    /// Register a handler for a specific tag name
    pub fn register(&mut self, tag_name: &str, handler: Box<dyn ElementHandler>) {
        let priority = handler.priority();
        let tag_handlers = self.handlers.entry(tag_name.to_lowercase()).or_insert_with(Vec::new);
        
        // Insert in priority order (highest first)
        let insert_pos = tag_handlers.iter().position(|(p, _)| *p < priority).unwrap_or(tag_handlers.len());
        tag_handlers.insert(insert_pos, (priority, handler));
    }
    
    /// Register a handler with a specific priority
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