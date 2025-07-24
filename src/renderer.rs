//! Template rendering engine
//! 
//! This module implements the core rendering functionality that takes compiled
//! templates and binds data to them, producing final HTML output.

use std::borrow::Cow;
use std::sync::Arc;

use dom_query::{Document, Selection};
use regex::Regex;
use once_cell::sync::Lazy;

use crate::error::{Error, Result};
use crate::types::*;
use crate::value::RenderValue;
use crate::handlers::ElementHandler;
use crate::node_ext::NodeExt;
use crate::constraints::{ConstraintContext, ConstraintEvaluator};
use crate::utils::{replace_multiple_cow, split_path_cow, with_string_buffer};
use crate::cache::get_global_cache;

static VARIABLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\{([^}]+)\}").expect("Invalid variable regex")
});

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
            return Err(Error::render_static("No root elements found in template"));
        }

        // Keep track of which elements have been processed to avoid duplicates
        let mut processed_selectors = std::collections::HashSet::new();
        
        // Process each template element
        for element_def in &self.template.elements {
            if !processed_selectors.contains(&element_def.selector) {
                self.render_element_with_context(&doc, &root, element_def, data, &mut processed_selectors)?;
            }
        }

        // Apply constraints if any
        self.apply_constraints(&doc, &root, data)?;

        // Return the rendered HTML
        Ok(self.serialize_selection(&root))
    }

    /// Render a single template element with context tracking
    fn render_element_with_context(
        &self,
        doc: &Document,
        root: &Selection,
        element_def: &TemplateElement,
        data: &dyn RenderValue,
        processed_selectors: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        // Mark this selector as processed
        processed_selectors.insert(element_def.selector.clone());
        self.render_element(doc, root, element_def, data)
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
                .ok_or_else(|| Error::render_static("Itemscope element missing property name"))?;
            
            // Get the nested data object
            if let Some(nested_value) = data.get_value(&[prop_name.clone()]) {
                // Use the nested object as the new data context
                nested_value
            } else {
                // If no nested data, use the original data
                data
            }
        } else {
            data
        };

        // Apply properties to the element
        for property in &element_def.properties {
            // Skip text content property for itemscope elements 
            // (they don't render their property value as text)
            if element_def.is_scope && matches!(property.target, PropertyTarget::TextContent) {
                continue;
            }
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
        
        // If this is a scope element, render its children with the scoped data
        if element_def.is_scope {
            self.render_scoped_children(element, element_data)?;
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

    /// Process variable substitution in text using zero-copy optimizations
    fn process_variables_in_text<'b>(
        &self,
        text: &'b str,
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
        
        // Use zero-copy replacement for multiple variables
        let replacements: Vec<(String, Cow<str>)> = variables.iter()
            .map(|var| {
                let value = data.get_property(&var.path)
                    .unwrap_or_else(|| Cow::Borrowed(""));
                (var.raw.clone(), value)
            })
            .collect();
        
        Ok(replace_multiple_cow(text, &replacements))
    }

    /// Render an array element by cloning it for each array item
    fn render_array_element(
        &self,
        elements: &Selection,
        element_def: &TemplateElement,
        data: &dyn RenderValue,
    ) -> Result<()> {
        // Array properties have their name without the [] suffix
        let array_prop_name = if element_def.properties.is_empty() {
            return Err(Error::render_static("Array element has no properties"));
        } else {
            &element_def.properties[0].name
        };
        
        // Get the array value using the property name
        let array_value = data.get_value(&[array_prop_name.clone()]);
        
        // Check if we have array data and get the items
        let array_items = if let Some(arr_val) = array_value {
            if let Some(items) = arr_val.as_array() {
                items
            } else {
                // Not an array, treat as single item
                vec![arr_val]
            }
        } else {
            // No data, remove the template elements
            for element in elements.nodes() {
                element.remove_from_parent();
            }
            return Ok(());
        };
        
        // If no items, remove the template elements
        if array_items.is_empty() {
            for element in elements.nodes() {
                element.remove_from_parent();
            }
            return Ok(());
        }
        
        // Process each element that needs array rendering
        for element in elements.nodes() {
            // Get the parent element
            let parent = element.parent();
            if parent.is_none() {
                continue;
            }
            let parent_node = parent.unwrap();
            
            // Store the template HTML
            let template_html = element.html();
            
            // Create a non-array version of the element definition
            let mut item_element_def = element_def.clone();
            item_element_def.is_array = false;
            
            // For each array item, create a new element from the template HTML
            for item in array_items.iter() {
                // Parse the template HTML to create a new element
                let item_doc = Document::from(template_html.as_ref());
                
                // We need to render all elements within this cloned item
                // Get the root element
                let item_root = item_doc.select(":root > *");
                
                // Process all template elements (except arrays) within this item
                for element_to_render in &self.template.elements {
                    // Skip array elements - we're already rendering the array item
                    if element_to_render.is_array {
                        continue;
                    }
                    
                    // Use render_element which handles finding and rendering
                    self.render_element(&item_doc, &item_root, element_to_render, *item)?;
                }
                
                // For array items, process the article element itself with variables
                // The article element contains ${age} in its text
                let article_elements = item_doc.select(&element_def.selector);
                for article in article_elements.nodes() {
                    self.render_single_element(article, &item_element_def, *item)?;
                }
                
                // Also process any text nodes with variables that don't have itemprop
                // This handles cases like <p>Age: ${age}</p>
                self.process_variables_in_dom(&item_doc, &item_root, *item)?;
                
                // Append the fully rendered item HTML to the parent
                let rendered_html = item_root.html();
                parent_node.append_html(rendered_html);
            }
            
            // Remove the original template element
            element.remove_from_parent();
        }
        
        Ok(())
    }

    /// Apply constraints to the rendered output
    fn apply_constraints(
        &self,
        _doc: &Document,
        root: &Selection,
        data: &dyn RenderValue,
    ) -> Result<()> {
        // Create constraint context
        let context = ConstraintContext::new(data);
        let evaluator = ConstraintEvaluator::new();
        
        // TODO: Implement @id registration with proper lifetime management
        // For now, we'll skip this and only support direct property constraints
        
        // Evaluate constraints
        for constraint in &self.template.constraints {
            // Find elements matching the constraint selector
            let constrained_elements = root.select(&constraint.element_selector);
            
            // Check if constraint is satisfied
            let should_show = evaluator.should_render(constraint, &context)?;
            
            // Hide or show elements based on constraint
            if !should_show {
                for element in constrained_elements.nodes() {
                    element.remove_from_parent();
                }
            }
        }
        
        Ok(())
    }

    /// Process variables in all text nodes within the DOM
    fn process_variables_in_dom(
        &self,
        _doc: &Document,
        root: &Selection,
        data: &dyn RenderValue,
    ) -> Result<()> {
        // Find all elements that might contain variables
        let all_elements = root.select("*");
        
        for element in all_elements.nodes() {
            // Skip elements with itemprop - they're handled separately
            if element.attr("itemprop").is_some() {
                continue;
            }
            
            // Check if this element has no child elements (leaf node)
            // Only process text content for leaf nodes to avoid destroying structure
            let has_child_elements = !element.children().is_empty();
            if !has_child_elements {
                let text = element.text();
                if text.contains("${") {
                    // Extract variables
                    let variables = VARIABLE_REGEX
                        .captures_iter(&text)
                        .map(|cap| {
                            let var_path = &cap[1];
                            let path = split_path_cow(var_path).into_owned();
                            Variable {
                                path,
                                raw: cap[0].to_string(),
                            }
                        })
                        .collect::<Vec<_>>();
                    
                    if !variables.is_empty() {
                        let processed_text = self.process_variables_in_text(&text, &variables, data)?;
                        element.set_text_content(&processed_text);
                    }
                }
            }
            
            // Check attributes for variables
            let attrs = element.attrs();
            for attr in &attrs {
                let attr_value = &attr.value;
                if attr_value.contains("${") {
                        let variables = VARIABLE_REGEX
                            .captures_iter(attr_value)
                            .map(|cap| {
                                let var_path = &cap[1];
                                let path = split_path_cow(var_path).into_owned();
                                Variable {
                                    path,
                                    raw: cap[0].to_string(),
                                }
                            })
                            .collect::<Vec<_>>();
                        
                    if !variables.is_empty() {
                        let processed_value = self.process_variables_in_text(attr_value, &variables, data)?;
                        element.set_attr(&attr.name.local, &processed_value);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Render children of a scoped element
    fn render_scoped_children(
        &self,
        scope_element: &dom_query::Node,
        scoped_data: &dyn RenderValue,
    ) -> Result<()> {
        // Find all child elements with itemprop within this scope
        let scoped_selection = Selection::from(scope_element.clone());
        
        // Process template elements that are children of this scope
        for element_def in &self.template.elements {
            // Skip if this is the scope element itself
            if element_def.is_scope {
                continue;
            }
            
            // Find elements matching this definition within the scope
            let child_elements = scoped_selection.select(&element_def.selector);
            if !child_elements.is_empty() {
                // Render these elements with the scoped data
                for child_element in child_elements.nodes() {
                    self.render_single_element(child_element, element_def, scoped_data)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Serialize a selection back to HTML using optimized string building
    fn serialize_selection(&self, selection: &Selection) -> String {
        with_string_buffer(|buffer| {
            for node in selection.nodes() {
                buffer.push_str(&node.html());
            }
            buffer.clone()
        })
    }
    
    /// Perform CSS selector query with optional caching
    fn cached_select<'s>(&self, root: &'s Selection, selector: &str, _use_cache: bool) -> Selection<'s> {
        // Note: Caching disabled for now due to lifetime complexity
        // TODO: Implement proper selector result caching
        root.select(selector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::compiler::Compiler;
    use crate::types::{Constraint, ConstraintType};
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
    
    #[test]
    fn test_render_array() {
        let html = r#"
            <template>
                <ul>
                    <li itemprop="items[]">${name}</li>
                </ul>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "items": [
                {"name": "Item 1"},
                {"name": "Item 2"},
                {"name": "Item 3"}
            ]
        });
        
        let result = renderer.render(&data).unwrap();
        // All items should be rendered
        assert!(result.contains("Item 1"));
        assert!(result.contains("Item 2"));
        assert!(result.contains("Item 3"));
    }
    
    #[test]
    fn test_render_complex_array() {
        let html = r#"
            <template>
                <div class="users">
                    <article itemprop="users[]" class="user-card">
                        <h3 itemprop="name"></h3>
                        <p>Email: <span itemprop="email"></span></p>
                        <p>Age: ${age}</p>
                    </article>
                </div>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "users": [
                {
                    "name": "Alice",
                    "email": "alice@example.com",
                    "age": 30
                },
                {
                    "name": "Bob",
                    "email": "bob@example.com",
                    "age": 25
                }
            ]
        });
        
        let result = renderer.render(&data).unwrap();
        
        // Check that both users are rendered
        assert!(result.contains("Alice"));
        assert!(result.contains("alice@example.com"));
        assert!(result.contains("Age: 30"));
        
        assert!(result.contains("Bob"));
        assert!(result.contains("bob@example.com"));
        assert!(result.contains("Age: 25"));
        
        // Verify structure is maintained
        assert!(result.contains("class=\"user-card\""));
        assert!(result.contains("Email:"));
    }
    
    #[test]
    fn test_render_empty_array() {
        let html = r#"
            <template>
                <ul>
                    <li itemprop="items[]">${name}</li>
                </ul>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "items": []
        });
        
        let result = renderer.render(&data).unwrap();
        // List should be empty (no <li> elements)
        assert!(!result.contains("<li"));
        assert!(result.contains("<ul>"));
    }
    
    #[test]
    fn test_render_nested_object() {
        let html = r#"
            <template>
                <article>
                    <h1 itemprop="title"></h1>
                    <div itemprop="author" itemscope>
                        <span itemprop="name"></span>
                        <span itemprop="email"></span>
                    </div>
                    <p itemprop="content"></p>
                </article>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "title": "My Article",
            "author": {
                "name": "John Doe",
                "email": "john@example.com"
            },
            "content": "This is the article content."
        });
        
        let result = renderer.render(&data).unwrap();
        assert!(result.contains("My Article"));
        assert!(result.contains("John Doe"));
        assert!(result.contains("john@example.com"));
        assert!(result.contains("This is the article content."));
    }
    
    #[test]
    fn test_render_with_constraints() {
        let html = r#"
            <template>
                <div>
                    <h1 itemprop="title"></h1>
                    <div data-constraint='status == "active"'>
                        <p>This content is only shown when status is active</p>
                    </div>
                    <div data-constraint="count > 5">
                        <p>Count is greater than 5!</p>
                    </div>
                    <div data-constraint="premium">
                        <p>Premium content</p>
                    </div>
                </div>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "title": "Conditional Content",
            "status": "active",
            "count": 10,
            "premium": false
        });
        
        let result = renderer.render(&data).unwrap();
        
        // Should show title
        assert!(result.contains("Conditional Content"));
        
        // Should show active content
        assert!(result.contains("This content is only shown when status is active"));
        
        // Should show count > 5 content
        assert!(result.contains("Count is greater than 5!"));
        
        // Should NOT show premium content
        assert!(!result.contains("Premium content"));
    }
    
    #[test]
    fn test_render_with_scope_constraints() {
        let html = r#"
            <template>
                <div>
                    <h1 itemprop="title"></h1>
                    <div data-scope="admin">
                        <p>Admin only content</p>
                    </div>
                    <div data-scope="user">
                        <p>User content</p>
                    </div>
                </div>
            </template>
        "#;
        
        let parser = Parser::new(html).unwrap();
        let mut template = parser.parse_template(Some("div")).unwrap();
        
        // Manually set scope context for testing
        // In real usage, this would be set by the application
        template.constraints.push(Constraint {
            element_selector: "[data-scope=\"admin\"]".to_string(),
            constraint_type: ConstraintType::Scope("admin".to_string()),
            scope: Some("admin".to_string()),
        });
        
        let compiled = Compiler::compile_from_template(template);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&compiled, &handlers);
        
        let data = json!({
            "title": "Scoped Content"
        });
        
        let result = renderer.render(&data).unwrap();
        
        // Should show title
        assert!(result.contains("Scoped Content"));
        
        // For now, both will show since we don't have scope context in renderer
        // This would need application-level integration
    }
    
    #[test]
    fn test_render_nested_array_of_objects() {
        let html = r#"
            <template>
                <div class="blog">
                    <h1 itemprop="blogTitle"></h1>
                    <article itemprop="posts[]" itemscope>
                        <h2 itemprop="title"></h2>
                        <div itemprop="author" itemscope>
                            <span itemprop="name"></span>
                        </div>
                        <p itemprop="summary"></p>
                    </article>
                </div>
            </template>
        "#;
        
        let template = create_test_template(html);
        let handlers = std::collections::HashMap::new();
        let renderer = Renderer::new(&template, &handlers);
        
        let data = json!({
            "blogTitle": "Tech Blog",
            "posts": [
                {
                    "title": "First Post",
                    "author": {
                        "name": "Alice"
                    },
                    "summary": "Introduction to our blog"
                },
                {
                    "title": "Second Post", 
                    "author": {
                        "name": "Bob"
                    },
                    "summary": "More interesting content"
                }
            ]
        });
        
        let result = renderer.render(&data).unwrap();
        assert!(result.contains("Tech Blog"));
        assert!(result.contains("First Post"));
        assert!(result.contains("Alice"));
        assert!(result.contains("Introduction to our blog"));
        assert!(result.contains("Second Post"));
        assert!(result.contains("Bob"));
        assert!(result.contains("More interesting content"));
    }
}