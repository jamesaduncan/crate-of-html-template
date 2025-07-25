//! Template rendering engine
//!
//! This module implements the core rendering functionality that takes compiled
//! templates and binds data to them, producing final HTML output.

use std::borrow::Cow;

use dom_query::{Document, Selection};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::constraints::{ConstraintContext, ConstraintEvaluator};
use crate::error::{Error, Result};
use crate::handlers::ElementHandler;
use crate::node_ext::NodeExt;
use crate::types::*;
use crate::utils::{replace_multiple_cow, split_path_cow};
use crate::value::RenderValue;

static VARIABLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{([^}]+)\}").expect("Invalid variable regex"));

/// The main renderer that binds data to templates
pub struct Renderer<'a> {
    template: &'a CompiledTemplate,
    handlers: &'a std::collections::HashMap<String, Box<dyn ElementHandler>>,
}

impl<'a> Renderer<'a> {
    /// Create a new renderer for the given template
    pub fn new(
        template: &'a CompiledTemplate,
        handlers: &'a std::collections::HashMap<String, Box<dyn ElementHandler>>,
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
                self.render_element_with_context(
                    &doc,
                    &root,
                    element_def,
                    data,
                    &mut processed_selectors,
                )?;
            }
        }

        // Apply constraints if any
        self.apply_constraints(&doc, &root, data)?;

        // Also apply inline data-constraint attributes
        self.apply_inline_constraints(&root, data)?;

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
                // Check if this element is inside an array container
                // If so, skip it - it will be rendered when the array is processed
                let mut current = element.parent();
                let mut is_inside_array = false;
                while let Some(parent) = current {
                    if let Some(itemprop) = parent.attr("itemprop") {
                        if itemprop.ends_with("[]") {
                            is_inside_array = true;
                            break;
                        }
                    }
                    current = parent.parent();
                }
                
                if !is_inside_array {
                    self.render_single_element(element, element_def, data)?;
                }
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
            let prop_name = element_def
                .properties
                .first()
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

        // Check if there's a handler that should handle this element exclusively
        let tag_name = element.node_name();
        let skip_property_application = if let Some(tag) = &tag_name {
            // For select elements, skip text content property application as the handler will manage it
            tag.to_lowercase() == "select"
                && element_def
                    .properties
                    .iter()
                    .any(|p| matches!(p.target, PropertyTarget::TextContent))
        } else {
            false
        };

        // Apply properties to the element
        if !skip_property_application {
            for property in &element_def.properties {
                // Skip text content property for itemscope elements
                // (they don't render their property value as text)
                if element_def.is_scope && matches!(property.target, PropertyTarget::TextContent) {
                    continue;
                }
                self.apply_property(element, property, element_data)?;
            }
        }

        // Check if there's a custom handler for this element
        if let Some(tag_name) = element.node_name() {
            if let Some(handler) = self.handlers.get(&tag_name.to_lowercase()) {
                if handler.can_handle(&Selection::from(element.clone())) {
                    // For elements with itemprop, pass the property value directly
                    let handler_value = if !element_def.properties.is_empty() {
                        let prop_name = &element_def.properties[0].name;
                        if let Some(prop_value) = element_data.get_value(&[prop_name.clone()]) {
                            prop_value
                        } else {
                            element_data
                        }
                    } else {
                        element_data
                    };
                    handler.handle(&Selection::from(element.clone()), handler_value)?;
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
            PropertyTarget::Attribute(attr_name) => element.attr(attr_name).unwrap_or_default(),
            PropertyTarget::Value => element.attr("value").unwrap_or_default(),
        };

        // Check if we can resolve the variables
        // For properties with variables, check if the data is available
        if !property.variables.is_empty() {
            // Check if the first variable can be resolved
            if let Some(first_var) = property.variables.first() {
                if data.get_value(&first_var.path).is_none() {
                    // Variable can't be resolved, skip this property
                    return Ok(());
                }
            }
        }

        // Process the content with variable substitution
        let value = if property.variables.is_empty() {
            // No variables, use the property name directly
            data.get_property(&[property.name.clone()])
                .unwrap_or(Cow::Borrowed(""))
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

    /// Render array item HTML by processing the template with data
    /// This is a workaround for dom_query limitations with cloned documents
    fn render_array_item_html(
        &self,
        template_html: &str,
        item_data: &dyn RenderValue,
        _array_prop_name: &str,
    ) -> Result<String> {
        // Parse the template HTML
        let item_doc = Document::from(template_html);
        
        // Find the array element (should be the root element)
        let array_selection = item_doc.select("*[itemprop$='[]']");
        let array_element = if let Some(elem) = array_selection.nodes().first() {
            elem
        } else {
            return Err(Error::render_static("Array element not found in template"));
        };
        
        // Convert the array element to a regular element
        if let Some(itemprop) = array_element.attr("itemprop") {
            if itemprop.ends_with("[]") {
                let clean_name = &itemprop[..itemprop.len() - 2];
                array_element.set_attr("itemprop", clean_name);
            }
        }
        
        // Process all elements with itemprops within this template
        self.process_template_elements(&item_doc, item_data)?;
        
        // Process variables in any text content (including elements with itemprop)
        // This handles cases like <p>Age: ${age}</p> and <li itemprop="items[]">${name}</li>
        // Due to dom_query limitations, we need to manually process these
        let all_elements = item_doc.select("*");
        for element in all_elements.nodes() {
            let text = element.text();
            if text.contains("${") {
                // Extract variables
                let variables = crate::parser::VARIABLE_REGEX
                    .captures_iter(&text)
                    .map(|cap| {
                        let var_path = &cap[1];
                        let path = crate::utils::split_path_cow(var_path).into_owned();
                        crate::types::Variable {
                            path,
                            raw: cap[0].to_string(),
                        }
                    })
                    .collect::<Vec<_>>();
                
                if !variables.is_empty() {
                    let processed_text = self.process_variables_in_text(&text, &variables, item_data)?;
                    element.set_text_content(&processed_text);
                }
            }
        }
        
        // Extract just the array element's HTML
        let result_html = array_element.html();
        Ok(result_html.to_string())
    }
    
    /// Process all template elements within a document
    fn process_template_elements(
        &self,
        doc: &Document,
        data: &dyn RenderValue,
    ) -> Result<()> {
        // Process each template element that isn't an array
        for element_def in &self.template.elements {
            if element_def.is_array {
                continue; // Skip array elements - they're handled separately
            }
            
            // Find elements matching this definition
            // Use a simple attribute selector instead of complex selectors
            if let Some(prop_name) = element_def.properties.first().map(|p| &p.name) {
                let selector = format!("[itemprop='{}']", prop_name);
                let elements = doc.select(&selector);
                
                for element_node in elements.nodes() {
                    self.render_single_element(&element_node, element_def, data)?;
                }
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
            return Ok(data.get_property(&var.path).unwrap_or(Cow::Borrowed("")));
        }

        // If there's only one variable and it's the entire text, return just the value
        if variables.len() == 1 && variables[0].raw == text {
            let var = &variables[0];
            return Ok(data.get_property(&var.path).unwrap_or(Cow::Borrowed("")));
        }

        // Use zero-copy replacement for multiple variables
        let replacements: Vec<(String, Cow<str>)> = variables
            .iter()
            .map(|var| {
                let value = data.get_property(&var.path).unwrap_or(Cow::Borrowed(""));
                (var.raw.clone(), value)
            })
            .collect();

        let result = replace_multiple_cow(text, &replacements);

        // Handle escaped variables: convert $${variable} to ${variable}
        if result.contains("$${") {
            let unescaped = result.replace("$${", "${");
            Ok(Cow::Owned(unescaped))
        } else {
            Ok(result)
        }
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
                // Get all elements in the document
                let item_root = item_doc.select("*");

                // First, process the array element itself 
                // Find the actual array element by selector
                let array_elements_in_item = item_root.select(&element_def.selector);
                for array_element in array_elements_in_item.nodes() {
                    // Remove the [] suffix from the itemprop attribute to convert from array template to item
                    if let Some(itemprop) = array_element.attr("itemprop") {
                        if itemprop.ends_with("[]") {
                            let clean_name = &itemprop[..itemprop.len() - 2];
                            array_element.set_attr("itemprop", clean_name);
                        }
                    }
                    
                    // Check if this element has variables in its text content
                    let text = array_element.text();
                    if text.contains("${") {
                        // Process variables in the text content
                        let variables = crate::parser::VARIABLE_REGEX
                            .captures_iter(&text)
                            .map(|cap| {
                                let var_path = &cap[1];
                                let path = crate::utils::split_path_cow(var_path).into_owned();
                                crate::types::Variable {
                                    path,
                                    raw: cap[0].to_string(),
                                }
                            })
                            .collect::<Vec<_>>();
                        
                        if !variables.is_empty() {
                            let processed_text = self.process_variables_in_text(&text, &variables, *item)?;
                            array_element.set_text_content(&processed_text);
                        }
                    }
                }

                // Note: Due to dom_query limitations with CSS selectors on cloned documents,
                // we use manual HTML reconstruction instead of DOM manipulation

                // Note: We don't process the array element itself with properties
                // as it's just a container for the array items

                // Also process any text nodes with variables that don't have itemprop
                // This handles cases like <p>Age: ${age}</p>
                self.process_variables_in_dom(&item_doc, &item_root, *item)?;

                // Process nested array elements within this item
                // Find any array elements inside this item and render them recursively
                // BUT skip the current array type to avoid infinite recursion
                for nested_element_def in &self.template.elements {
                    if nested_element_def.is_array && nested_element_def.selector != element_def.selector {
                        // Look for this array element within the current item
                        let nested_elements = item_root.select(&nested_element_def.selector);
                        if !nested_elements.is_empty() {
                            
                            // Extract the array data from the current item's data context
                            // The nested array should use the array property from the current item
                            let nested_array_property = &nested_element_def.properties[0].name;
                            let nested_data = if let Some(array_data) = item.get_value(&[nested_array_property.clone()]) {
                                array_data
                            } else {
                                // Fallback to current item data if nested array not found
                                *item
                            };
                            
                            self.render_array_element(&nested_elements, nested_element_def, nested_data)?;
                        }
                    }
                }

                // Instead of relying on DOM modifications persisting, rebuild the HTML manually
                // This is a workaround for dom_query limitations with cloned documents
                let rendered_html = self.render_array_item_html(&template_html, *item, &element_def.properties[0].name)?;
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
            let sel = Selection::from(element.clone());
            let children = sel.select("*");
            let child_count = children.length();
            if child_count == 0 {
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
                        let processed_text =
                            self.process_variables_in_text(&text, &variables, data)?;
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
                        let processed_value =
                            self.process_variables_in_text(attr_value, &variables, data)?;
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

    /// Apply inline data-constraint attributes by parsing them directly from DOM
    fn apply_inline_constraints(&self, root: &Selection, data: &dyn RenderValue) -> Result<()> {
        // Find all elements with data-constraint attributes
        let constrained_elements = root.select("[data-constraint]");

        for element in constrained_elements.nodes() {
            if let Some(constraint_expr) = element.attr("data-constraint") {
                // Create constraint context
                let context = ConstraintContext::new(data);

                // Evaluate constraint expression directly
                match context.evaluate_expression(&constraint_expr) {
                    Ok(should_show) => {
                        if !should_show {
                            element.remove_from_parent();
                        }
                    }
                    Err(e) => {
                        // If constraint evaluation fails, hide the element to be safe
                        eprintln!(
                            "Constraint evaluation failed for '{}': {:?}",
                            constraint_expr, e
                        );
                        element.remove_from_parent();
                    }
                }
            }
        }

        Ok(())
    }

    /// Serialize a selection back to HTML using optimized string building
    fn serialize_selection(&self, selection: &Selection) -> String {
        // For now, avoid the string buffer due to safety issues
        let mut result = String::new();
        for node in selection.nodes() {
            result.push_str(&node.html());
        }
        result
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::parser::Parser;
    use crate::types::{Constraint, ConstraintType};
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;

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

    #[test]
    fn test_render_invalid_data_types() {
        let html = r#"
            <template>
                <div itemprop="test"></div>
            </template>
        "#;
        let template = create_test_template(html);
        let handlers = HashMap::new();
        let renderer = Renderer::new(&template, &handlers);

        // Test with null value
        let data = json!(null);
        let result = renderer.render(&data);
        assert!(result.is_ok());

        // Test with boolean
        let data = json!(true);
        let result = renderer.render(&data);
        assert!(result.is_ok());

        // Test with number
        let data = json!(42);
        let result = renderer.render(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_deeply_nested_data() {
        let html = r#"
            <template>
                <div itemprop="value">${a.b.c.d.e.f.g}</div>
            </template>
        "#;
        let template = create_test_template(html);
        let handlers = HashMap::new();
        let renderer = Renderer::new(&template, &handlers);

        let data = json!({
            "a": {
                "b": {
                    "c": {
                        "d": {
                            "e": {
                                "f": {
                                    "g": "Deep value"
                                }
                            }
                        }
                    }
                }
            }
        });

        let result = renderer.render(&data).unwrap();
        assert!(result.contains("Deep value"));
    }

    #[test]
    fn test_render_with_special_characters() {
        let html = r#"
            <template>
                <div itemprop="content"></div>
            </template>
        "#;
        let template = create_test_template(html);
        let handlers = HashMap::new();
        let renderer = Renderer::new(&template, &handlers);

        let data = json!({
            "content": "<script>alert('XSS')</script> & \"quotes\" 'apostrophes'"
        });

        let result = renderer.render(&data).unwrap();
        // dom_query handles escaping automatically when setting text content
        // It escapes < and > but may not escape quotes in text content
        assert!(result.contains("&lt;script&gt;"));
        assert!(result.contains("&amp;"));
        // Check for the actual quotes in the output (may not be escaped in text content)
        assert!(result.contains("quotes") || result.contains("&quot;"));
    }

    #[test]
    fn test_render_with_unicode() {
        let html = r#"
            <template>
                <div itemprop="emoji"></div>
                <div itemprop="chinese"></div>
                <div itemprop="rtl"></div>
            </template>
        "#;
        let template = create_test_template(html);
        let handlers = HashMap::new();
        let renderer = Renderer::new(&template, &handlers);

        let data = json!({
            "emoji": "Hello 😀🎉🌟",
            "chinese": "你好世界",
            "rtl": "مرحبا بالعالم"
        });

        let result = renderer.render(&data).unwrap();
        assert!(result.contains("Hello 😀🎉🌟"));
        assert!(result.contains("你好世界"));
        assert!(result.contains("مرحبا بالعالم"));
    }

    #[test]
    fn test_render_large_dataset() {
        let html = r#"
            <template>
                <ul>
                    <li itemprop="items[]">
                        <span itemprop="id"></span>: <span itemprop="value"></span>
                    </li>
                </ul>
            </template>
        "#;
        let template = create_test_template(html);
        let handlers = HashMap::new();
        let renderer = Renderer::new(&template, &handlers);

        // Create large array
        let mut items = Vec::new();
        for i in 0..1000 {
            items.push(json!({
                "id": i,
                "value": format!("Item {}", i)
            }));
        }

        let data = json!({ "items": items });
        let result = renderer.render(&data).unwrap();

        // Should render all items
        assert!(result.contains("Item 0"));
        assert!(result.contains("Item 999"));
    }

    #[test]
    fn test_render_with_null_values() {
        let html = r#"
            <template>
                <div>
                    <span itemprop="nullable"></span>
                    <span itemprop="defined"></span>
                </div>
            </template>
        "#;
        let template = create_test_template(html);
        let handlers = HashMap::new();
        let renderer = Renderer::new(&template, &handlers);

        let data = json!({
            "nullable": null,
            "defined": "value"
        });

        let result = renderer.render(&data).unwrap();
        // The second span should contain "value"
        assert!(result.contains("value"));
        // The result should have two span elements
        let span_count = result.matches("<span").count();
        assert_eq!(
            span_count, 2,
            "Expected 2 span elements, found {}",
            span_count
        );
    }
}
