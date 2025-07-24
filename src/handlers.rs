use dom_query::Selection;

use crate::error::Result;
use crate::value::RenderValue;

pub trait ElementHandler: Send + Sync {
    fn can_handle(&self, element: &Selection) -> bool;
    fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()>;
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
        if let Some(val) = value.get_property(&[]) {
            // Find all option elements
            let options = element.select("option");
            for option in options.nodes() {
                if option.attr("value").as_deref() == Some(val.as_ref()) {
                    option.set_attr("selected", "selected");
                } else {
                    option.remove_attr("selected");
                }
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
}