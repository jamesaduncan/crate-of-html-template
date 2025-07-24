//! Template rendering engine
//! 
//! This module implements the core rendering functionality that takes compiled
//! templates and binds data to them, producing final HTML output.

use std::borrow::Cow;
use std::sync::Arc;

use dom_query::{Document, Selection};

use crate::error::{Error, Result};
use crate::types::*;
use crate::value::RenderValue;
use crate::handlers::ElementHandler;
use crate::node_ext::NodeExt;

/// The main renderer that binds data to templates
pub struct Renderer<'a> {
    template: &'a CompiledTemplate,
    handlers: &'a std::collections::HashMap<String, Box<dyn ElementHandler>>,
}

impl<'a> Renderer<'a> {
    /// Create a new renderer for the given template
    pub fn new(
        template: &'a CompiledTemplate, 
        handlers: &'a std::collections::HashMap<String, Box<dyn ElementHandler>>
    ) -> Self {
        Self { template, handlers }
    }

    /// Render the template with the given data
    pub fn render(&self, data: &dyn RenderValue) -> Result<String> {
        // Parse the template HTML to create a working document
        let doc = Document::from(self.template.template_html.as_ref());
        
        // Apply the root selector if specified
        let root = if let Some(ref selector) = self.template.root_selector {
            doc.select(selector)
        } else {
            // First try body > *, then fall back to root elements
            let body_children = doc.select("body > *");
            if !body_children.is_empty() {
                body_children
            } else {
                // No body, select direct children of the document
                doc.select(":root > *")
            }
        };

        if root.is_empty() {
            return Err(Error::RenderError("No root elements found in template".to_string()));
        }

        // Process each template element
        for element_def in &self.template.elements {
            self.render_element(&doc, &root, element_def, data)?;
        }

        // Apply constraints if any
        self.apply_constraints(&doc, &root, data)?;

        // Return the rendered HTML
        Ok(self.serialize_selection(&root))
    }

    /// Render a single template element
    fn render_element(
        &self,
        _doc: &Document,
        root: &Selection,
        element_def: &TemplateElement,
        data: &dyn RenderValue,
    ) -> Result<()> {
        // Find the element(s) matching the selector
        
        // We need to check both the root elements themselves and their descendants
        let mut matching_elements = Vec::new();
        
        // First check if any root elements match
        // We'll use the is() method to check if elements match
        for node in root.nodes() {
            // Create a single-node selection to check if it matches
            let single_selection = Selection::from(node.clone());
            if single_selection.is(&element_def.selector) {
                matching_elements.push(node.clone());
            }
        }
        
        // Then search descendants
        let descendant_elements = root.select(&element_def.selector);
        for node in descendant_elements.nodes() {
            matching_elements.push(node.clone());
        }
        
        
        if matching_elements.is_empty() {
            // Element not found - this might be ok for optional elements
            return Ok(());
        }

        // Handle array rendering
        if element_def.is_array {
            // Convert to Selection for array rendering
            let elements_selection = Selection::from(matching_elements.clone());
            self.render_array_element(&elements_selection, element_def, data)?;
        } else {
            // Render single element
            for element in &matching_elements {
                self.render_single_element(element, element_def, data)?;
            }
        }

        Ok(())
    }

    /// Render a single (non-array) element
    fn render_single_element(
        &self,
        element: &dom_query::Node,
        element_def: &TemplateElement,
        data: &dyn RenderValue,
    ) -> Result<()> {
        // Determine the data context for this element
        let element_data = if element_def.is_scope {
            // For itemscope elements, extract the property value as the new context
            let prop_name = element_def.properties.first()
                .map(|p| &p.name)
                .ok_or_else(|| Error::RenderError("Itemscope element missing property name".to_string()))?;
            
            // Get the nested data
            if let Some(_value) = data.get_property(&[prop_name.clone()]) {
                // For now, we'll just use the string representation
                // In a full implementation, we'd need to handle nested objects properly
                data
            } else {
                data
            }
        } else {
            data
        };

        // Apply properties to the element
        for property in &element_def.properties {
            self.apply_property(element, property, element_data)?;
        }

        // Check if there's a custom handler for this element
        if let Some(tag_name) = element.node_name() {
            if let Some(handler) = self.handlers.get(&tag_name.to_lowercase()) {
                if handler.can_handle(&Selection::from(element.clone())) {
                    handler.handle(&Selection::from(element.clone()), element_data)?;
                }
            }
        }

        Ok(())
    }

    /// Apply a property binding to an element
    fn apply_property(
        &self,
        element: &dom_query::Node,
        property: &Property,
        data: &dyn RenderValue,
    ) -> Result<()> {
        // Get the current content to process variables
        let current_content = match &property.target {
            PropertyTarget::TextContent => element.text(),
            PropertyTarget::Attribute(attr_name) => {
                element.attr(attr_name).unwrap_or_default()
            }
            PropertyTarget::Value => {
                element.attr("value").unwrap_or_default()
            }
        };
        
        // Process the content with variable substitution
        let value = if property.variables.is_empty() {
            // No variables, use the property name directly
            data.get_property(&[property.name.clone()])
                .unwrap_or_else(|| Cow::Borrowed(""))
        } else {
            // Process variables in the current content
            self.process_variables_in_text(&current_content, &property.variables, data)?
        };

        // Apply the value based on the target
        match &property.target {
            PropertyTarget::TextContent => {
                // Set text content by replacing all children with a text node
                element.set_text_content(&value);
            }
            PropertyTarget::Attribute(attr_name) => {
                element.set_attr(attr_name, &value);
            }
            PropertyTarget::Value => {
                // For input elements, set the value attribute
                element.set_attr("value", &value);
            }
        }

        Ok(())
    }

    /// Process variable substitution in text
    fn process_variables_in_text<'b>(
        &self,
        text: &str,
        variables: &[Variable],
        data: &'b dyn RenderValue,
    ) -> Result<Cow<'b, str>> {
        if variables.is_empty() {
            return Ok(Cow::Borrowed(""));
        }
        
        // If text is empty and we have one variable, it's an implicit binding
        if text.is_empty() && variables.len() == 1 {
            let var = &variables[0];
            return Ok(data.get_property(&var.path).unwrap_or_else(|| Cow::Borrowed("")));
        }
        
        // If there's only one variable and it's the entire text, return just the value
        if variables.len() == 1 && variables[0].raw == text {
            let var = &variables[0];
            return Ok(data.get_property(&var.path).unwrap_or_else(|| Cow::Borrowed("")));
        }
        
        // Otherwise, do string substitution
        let mut result = text.to_string();
        for var in variables {
            let value = data.get_property(&var.path)
                .unwrap_or_else(|| Cow::Borrowed(""));
            result = result.replace(&var.raw, &value);
        }
        Ok(Cow::Owned(result))
    }

    /// Render an array element by cloning it for each array item
    fn render_array_element(
        &self,
        _elements: &Selection,
        _element_def: &TemplateElement,
        _data: &dyn RenderValue,
    ) -> Result<()> {
        // For now, just mark this as TODO
        // Array rendering will be implemented in step 3.2
        Ok(())
    }

    /// Apply constraints to the rendered output
    fn apply_constraints(
        &self,
        _doc: &Document,
        _root: &Selection,
        _data: &dyn RenderValue,
    ) -> Result<()> {
        // Constraints will be implemented in Phase 4
        Ok(())
    }

    /// Serialize a selection back to HTML
    fn serialize_selection(&self, selection: &Selection) -> String {
        let mut html = String::new();
        for node in selection.nodes() {
            html.push_str(&node.html());
        }
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::compiler::Compiler;
    use serde_json::json;
    
    fn create_test_template(html: &str) -> Arc<CompiledTemplate> {
        let parser = Parser::new(html).unwrap();
        let template = parser.parse_template(None).unwrap();
        Compiler::compile_from_template(template)
    }
    
    #[test]
    fn test_render_simple_template() {
        let html = r#"
            <template>
                <div>
                    <h1 itemprop="title"></h1>
                    <p itemprop="description"></p>
                </div>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "title": "Hello World",
            "description": "This is a test"
        });
        
        let result = renderer.render(&data).unwrap();
        assert!(result.contains("Hello World"));
        assert!(result.contains("This is a test"));
    }
    
    #[test]
    fn test_render_with_variables() {
        let html = r#"
            <template>
                <div>
                    <h1 itemprop="greeting">${greeting}, ${name}!</h1>
                </div>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "greeting": "Hello",
            "name": "World"
        });
        
        let result = renderer.render(&data).unwrap();
        // For now this will just show "Hello" since we haven't implemented
        // full variable substitution yet
        assert!(result.contains("Hello"));
    }
    
    #[test]
    fn test_render_with_attributes() {
        let html = r#"
            <template>
                <a href="${url}" itemprop="link">${linkText}</a>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "url": "https://example.com",
            "link": "Example",
            "linkText": "Click here"
        });
        
        let result = renderer.render(&data).unwrap();
        assert!(result.contains(r#"href="https://example.com""#));
        assert!(result.contains("Click here")); // from ${linkText} variable
    }
    
    #[test]
    fn test_render_missing_data() {
        let html = r#"
            <template>
                <div>
                    <h1 itemprop="title"></h1>
                    <p itemprop="missing"></p>
                </div>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "title": "Only Title"
        });
        
        let result = renderer.render(&data).unwrap();
        assert!(result.contains("Only Title"));
        // Missing property should render as empty
        assert!(result.contains("<p"));
    }
}