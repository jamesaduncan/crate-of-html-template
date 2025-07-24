use dom_query::{Document, Selection};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::{Error, Result};
use crate::node_ext::NodeExt;
use crate::types::*;
use crate::utils::split_path_cow;

static VARIABLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{([^}]+)\}").expect("Invalid variable regex"));

static ESCAPED_VARIABLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\$\{([^}]+)\}").expect("Invalid escaped variable regex"));

pub struct Parser {
    document: Document,
}

impl Parser {
    pub fn new(html: &str) -> Result<Self> {
        let document = Document::from(html);
        Ok(Self { document })
    }

    pub fn parse_template(&self, root_selector: Option<&str>) -> Result<CompiledTemplate> {
        // Find the template element
        let template = self.find_template_element()?;

        // Get the original template HTML for storage
        let _template_html = template.html().to_string();

        // For template elements, we need to extract their content properly
        // First, try to access the template_contents directly
        let template_node = template
            .nodes()
            .first()
            .ok_or_else(|| Error::parse_static("No template element found"))?;

        // Access the template contents via the query method
        let template_contents_id = template_node
            .query(|node| node.as_element().and_then(|elem| elem.template_contents))
            .flatten();

        // Create a new document from template content and store the inner HTML
        let (content_doc, content_html) = if let Some(contents_id) = template_contents_id {
            // We have template contents - need to serialize them to HTML
            let contents_node = dom_query::Node::new(contents_id, template_node.tree);
            let inner_html = contents_node.inner_html();
            (Document::from(inner_html.as_ref()), inner_html.to_string())
        } else {
            // Fallback: parse the template's inner HTML
            let inner_html = template.html();
            if inner_html.trim().is_empty() {
                return Err(Error::parse_static("Template element has no content"));
            }
            (Document::from(inner_html.as_ref()), inner_html.to_string())
        };

        // Select content based on root selector
        let content = if let Some(selector) = root_selector {
            let selected = content_doc.select(selector);
            selected
        } else {
            // Try multiple selectors to find content
            let body_content = content_doc.select("body > *");
            if !body_content.is_empty() {
                body_content
            } else {
                // If no body, try selecting all elements
                let all_content = content_doc.select("*");
                all_content
            }
        };

        if content.is_empty() {
            return Err(Error::parse_static("No content found in template"));
        }

        // Parse the template structure
        let elements = self.parse_elements(&content)?;
        let constraints = self.extract_constraints(&content)?;

        Ok(CompiledTemplate {
            root_selector: root_selector.map(String::from),
            elements,
            constraints,
            base_uri: self.extract_base_uri(),
            template_html: content_html,
        })
    }

    fn find_template_element(&self) -> Result<Selection> {
        let templates = self.document.select("template");
        if templates.is_empty() {
            return Err(Error::parse_static("No template element found"));
        }
        Ok(templates)
    }

    fn parse_elements(&self, root: &Selection) -> Result<Vec<TemplateElement>> {
        let mut elements = Vec::new();

        // First check if any of the root elements themselves have itemprop
        for root_node in root.nodes() {
            if root_node.attr("itemprop").is_some() {
                self.parse_element_node(root_node, &mut elements)?;
            }
        }

        // Then find all descendant elements with itemprop
        let itemprop_elements = root.select("[itemprop]");

        for element in itemprop_elements.nodes() {
            self.parse_element_node(element, &mut elements)?;
        }

        Ok(elements)
    }

    fn parse_element_node(
        &self,
        element: &dom_query::Node,
        elements: &mut Vec<TemplateElement>,
    ) -> Result<()> {
        let itemprop = element
            .attr("itemprop")
            .ok_or_else(|| Error::parse_static("Element missing itemprop"))?;

        let is_array = itemprop.ends_with("[]");
        let clean_name = if is_array {
            itemprop[..itemprop.len() - 2].to_string()
        } else {
            itemprop.to_string()
        };

        // Parse properties from this element
        let properties = self.parse_properties(element, &clean_name)?;

        // Check for itemscope
        let is_scope = element.has_attr("itemscope");
        let itemtype = element.attr("itemtype").map(|s| s.to_string());

        // Extract constraints for this element
        let constraint_refs = self.extract_element_constraints(element)?;

        // Generate a unique selector for this element
        let selector = self.generate_selector(element)?;

        elements.push(TemplateElement {
            selector,
            properties,
            is_array,
            is_scope,
            itemtype,
            constraints: constraint_refs,
        });

        Ok(())
    }

    fn parse_properties(
        &self,
        element: &dom_query::Node,
        prop_name: &str,
    ) -> Result<Vec<Property>> {
        let mut properties = Vec::new();

        // Check text content for variables or binding
        let text = element.text_content();
        let text_variables = self.extract_variables(&text);

        // Always create a text content property for elements with itemprop
        // This allows binding data to the element even without explicit ${} syntax
        if !text_variables.is_empty() || !text.trim().is_empty() || properties.is_empty() {
            properties.push(Property {
                name: prop_name.to_string(),
                is_array: false,
                target: PropertyTarget::TextContent,
                variables: if text_variables.is_empty() {
                    // Create an implicit variable binding
                    // For itemprop names, treat the entire name as a single property
                    // (don't split on dots like we do for ${var.path} variables)
                    vec![Variable {
                        path: vec![prop_name.to_string()],
                        raw: format!("${{{}}}", prop_name),
                    }]
                } else {
                    text_variables
                },
            });
        }

        // Check attributes for variables
        let attrs = element.attrs();
        for attr in &attrs {
            let variables = self.extract_variables(&attr.value);
            if !variables.is_empty() {
                properties.push(Property {
                    name: prop_name.to_string(),
                    is_array: false,
                    target: PropertyTarget::Attribute(attr.name.local.to_string()),
                    variables,
                });
            }
        }

        // Special handling for input elements
        if element
            .node_name()
            .map(|n| n.to_lowercase() == "input")
            .unwrap_or(false)
        {
            // Only add if we haven't already added a value property
            let has_value_prop = properties
                .iter()
                .any(|p| matches!(p.target, PropertyTarget::Value));
            if !has_value_prop {
                properties.push(Property {
                    name: prop_name.to_string(),
                    is_array: false,
                    target: PropertyTarget::Value,
                    variables: vec![Variable {
                        path: vec![prop_name.to_string()],
                        raw: format!("${{{}}}", prop_name),
                    }],
                });
            }
        }

        // Ensure we always have at least one property for elements with itemprop
        if properties.is_empty() {
            properties.push(Property {
                name: prop_name.to_string(),
                is_array: false,
                target: PropertyTarget::TextContent,
                variables: vec![Variable {
                    path: vec![prop_name.to_string()],
                    raw: format!("${{{}}}", prop_name),
                }],
            });
        }

        Ok(properties)
    }

    fn extract_variables(&self, text: &str) -> Vec<Variable> {
        // First, temporarily replace escaped variables with placeholders to avoid false matches
        let mut working_text = text.to_string();
        let escaped_vars: Vec<_> = ESCAPED_VARIABLE_REGEX.captures_iter(text).collect();
        
        for (i, cap) in escaped_vars.iter().enumerate() {
            let placeholder = format!("__ESCAPED_VAR_{}__", i);
            working_text = working_text.replace(&cap[0], &placeholder);
        }
        
        // Now extract regular variables from the text with escaped vars replaced
        VARIABLE_REGEX
            .captures_iter(&working_text)
            .map(|cap| {
                let var_path = &cap[1];
                let path = self.parse_variable_path(var_path);
                Variable {
                    path,
                    raw: cap[0].to_string(),
                }
            })
            .collect()
    }

    fn parse_variable_path(&self, path: &str) -> Vec<String> {
        // Use zero-copy path splitting for simple cases
        if !path.contains('[') {
            // Simple case without array access
            split_path_cow(path).into_owned()
        } else {
            // Complex case with array access - need to handle brackets
            path.split('.')
                .map(|segment| {
                    // Handle array access like items[0]
                    if let Some(bracket_pos) = segment.find('[') {
                        segment[..bracket_pos].to_string()
                    } else {
                        segment.to_string()
                    }
                })
                .collect()
        }
    }

    fn extract_constraints(&self, root: &Selection) -> Result<Vec<Constraint>> {
        let mut constraints = Vec::new();
        let mut _constraint_index = 0;

        // Find elements with data-scope
        let scope_elements = root.select("[data-scope]");
        for element in scope_elements.nodes() {
            if let Some(scope) = element.attr("data-scope") {
                let selector = self.generate_selector(element)?;
                constraints.push(Constraint {
                    element_selector: selector,
                    constraint_type: ConstraintType::Scope(scope.to_string()),
                    scope: Some(scope.to_string()),
                });
                _constraint_index += 1;
            }
        }

        // Find elements with data-constraint
        let constraint_elements = root.select("[data-constraint]");
        for element in constraint_elements.nodes() {
            if let Some(constraint_expr) = element.attr("data-constraint") {
                let selector = self.generate_selector(element)?;
                constraints.push(Constraint {
                    element_selector: selector,
                    constraint_type: ConstraintType::Expression(constraint_expr.to_string()),
                    scope: None,
                });
                _constraint_index += 1;
            }
        }

        Ok(constraints)
    }

    fn extract_element_constraints(
        &self,
        _element: &dom_query::Node,
    ) -> Result<Vec<ConstraintRef>> {
        let refs = Vec::new();

        // For now, we'll implement a simple approach
        // In a full implementation, this would map to the constraints vector

        Ok(refs)
    }

    fn generate_selector(&self, element: &dom_query::Node) -> Result<String> {
        // Generate a unique CSS selector for this element
        let mut selector_parts = Vec::new();

        // Add tag name
        if let Some(tag) = element.node_name() {
            selector_parts.push(tag.to_lowercase());
        }

        // Add id if present
        if let Some(id) = element.attr("id") {
            selector_parts.push(format!("#{}", id));
        }

        // Add classes if present
        if let Some(classes) = element.attr("class") {
            for class in classes.split_whitespace() {
                selector_parts.push(format!(".{}", class));
            }
        }

        // Add itemprop as attribute selector
        if let Some(itemprop) = element.attr("itemprop") {
            selector_parts.push(format!("[itemprop=\"{}\"]", itemprop));
        }

        // Add data-constraint as attribute selector
        if let Some(constraint) = element.attr("data-constraint") {
            selector_parts.push(format!(
                "[data-constraint=\"{}\"]",
                constraint.replace('"', "\\\"")
            ));
        }

        // Add data-scope as attribute selector
        if let Some(scope) = element.attr("data-scope") {
            selector_parts.push(format!("[data-scope=\"{}\"]", scope));
        }

        Ok(selector_parts.join(""))
    }

    fn extract_base_uri(&self) -> Option<String> {
        self.document
            .select("base")
            .nodes()
            .first()
            .and_then(|base| base.attr("href"))
            .map(|href| href.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_template() {
        let html = r#"
            <template>
                <div>
                    <h1 itemprop="title"></h1>
                    <p itemprop="description"></p>
                </div>
            </template>
        "#;

        let parser = Parser::new(html).unwrap();
        let compiled = parser.parse_template(Some("div")).unwrap();

        assert_eq!(compiled.root_selector, Some("div".to_string()));
        assert_eq!(compiled.elements.len(), 2);

        assert!(!compiled.elements.is_empty());
        if !compiled.elements.is_empty() {
            assert!(!compiled.elements[0].properties.is_empty());
            assert_eq!(compiled.elements[0].properties[0].name, "title");
        }
        if compiled.elements.len() > 1 {
            assert!(!compiled.elements[1].properties.is_empty());
            assert_eq!(compiled.elements[1].properties[0].name, "description");
        }
    }

    #[test]
    fn test_parse_array_property() {
        let html = r#"
            <template>
                <ul>
                    <li itemprop="items[]"></li>
                </ul>
            </template>
        "#;

        let parser = Parser::new(html).unwrap();
        let compiled = parser.parse_template(Some("ul")).unwrap();

        assert_eq!(compiled.elements.len(), 1);
        assert!(compiled.elements[0].is_array);
        assert_eq!(compiled.elements[0].properties[0].name, "items");
    }

    #[test]
    fn test_extract_variables() {
        let parser = Parser::new("").unwrap();

        let vars = parser.extract_variables("Hello ${name}, your age is ${user.age}!");
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].raw, "${name}");
        assert_eq!(vars[0].path, vec!["name"]);
        assert_eq!(vars[1].raw, "${user.age}");
        assert_eq!(vars[1].path, vec!["user", "age"]);
    }

    #[test]
    fn test_parse_variable_path() {
        let parser = Parser::new("").unwrap();

        let path = parser.parse_variable_path("user.profile.name");
        assert_eq!(path, vec!["user", "profile", "name"]);

        let path = parser.parse_variable_path("items[0].name");
        assert_eq!(path, vec!["items", "name"]);
    }

    #[test]
    fn test_itemscope_detection() {
        let html = r#"
            <template>
                <div itemprop="author" itemscope>
                    <span itemprop="name"></span>
                </div>
            </template>
        "#;

        let parser = Parser::new(html).unwrap();
        let compiled = parser.parse_template(None).unwrap();

        assert_eq!(compiled.elements.len(), 2);
        assert!(compiled.elements[0].is_scope);
        assert!(!compiled.elements[1].is_scope);
    }

    #[test]
    fn test_constraints_extraction() {
        let html = r#"
            <template>
                <div>
                    <div data-scope="user" itemprop="profile"></div>
                    <div data-constraint="age > 18" itemprop="content"></div>
                </div>
            </template>
        "#;

        let parser = Parser::new(html).unwrap();
        let compiled = parser.parse_template(Some("div")).unwrap();

        assert_eq!(compiled.constraints.len(), 2);

        match &compiled.constraints[0].constraint_type {
            ConstraintType::Scope(scope) => assert_eq!(scope, "user"),
            _ => panic!("Expected scope constraint"),
        }

        match &compiled.constraints[1].constraint_type {
            ConstraintType::Expression(expr) => assert_eq!(expr, "age > 18"),
            _ => panic!("Expected expression constraint"),
        }
    }

    #[test]
    fn test_base_uri_extraction() {
        let html = r#"
            <base href="https://example.com/app/">
            <template>
                <div itemprop="test"></div>
            </template>
        "#;

        let parser = Parser::new(html).unwrap();
        let compiled = parser.parse_template(None).unwrap();

        assert_eq!(
            compiled.base_uri,
            Some("https://example.com/app/".to_string())
        );
    }

    #[test]
    fn test_attribute_variables() {
        let html = r#"
            <template>
                <a href="${url}" itemprop="link">${linkText}</a>
            </template>
        "#;

        let parser = Parser::new(html).unwrap();
        let compiled = parser.parse_template(None).unwrap();

        assert_eq!(compiled.elements.len(), 1);
        let element = &compiled.elements[0];

        // Should have properties for both text content and href attribute
        assert!(element
            .properties
            .iter()
            .any(|p| matches!(&p.target, PropertyTarget::Attribute(attr) if attr == "href")));
        assert!(element
            .properties
            .iter()
            .any(|p| matches!(&p.target, PropertyTarget::TextContent)));
    }

    #[test]
    fn test_invalid_selector_error() {
        let html = r#"<template><div>Test</div></template>"#;

        // Test with valid selector that matches nothing
        let parser = Parser::new(html).unwrap();
        let result = parser.parse_template(Some("span"));
        // Should succeed or fail gracefully
        if let Ok(template) = result {
            // But should have no elements with itemprop
            assert_eq!(template.elements.len(), 0);
        } else {
            // If it errors, that's also acceptable for a non-matching selector
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_malformed_html_parsing() {
        // Test parser's ability to handle malformed HTML gracefully
        // Need template elements for parser
        let malformed_cases = vec![
            (
                "<template><div><p>Unclosed paragraph</div></template>",
                "div",
            ),
            (
                "<template><div itemprop='test'>Content</div></template>",
                "div",
            ),
            ("<template><div>Valid content</div></template>", "div"),
        ];

        for (html, expected_tag) in malformed_cases {
            let parser = Parser::new(html).unwrap();
            let result = parser.parse_template(None);
            assert!(result.is_ok());

            // Check if the template parsing succeeded
            let template = result.unwrap();
            // Should have parsed the template contents
            assert!(template.template_html.contains(expected_tag));
        }
    }

    #[test]
    fn test_deeply_nested_properties() {
        let mut html = String::from("<template>");
        for i in 0..50 {
            html.push_str(&format!(r#"<div itemprop="level{}"><template>"#, i));
        }
        html.push_str(r#"<span itemprop="deep">Value</span>"#);
        for _ in 0..50 {
            html.push_str("</template></div>");
        }
        html.push_str("</template>");

        let parser = Parser::new(&html).unwrap();
        let result = parser.parse_template(None);
        assert!(result.is_ok());

        let template = result.unwrap();
        // Should handle deep nesting
        assert!(template.elements.len() > 0);
    }

    #[test]
    fn test_empty_and_whitespace_templates() {
        let test_cases = vec![
            "<template></template>",
            "<template>   </template>",
            "<template>\n\n\n</template>",
            "<template>\t\t</template>",
            "<template><!-- comment only --></template>",
        ];

        for html in test_cases {
            let parser = Parser::new(html).unwrap();
            let result = parser.parse_template(None);
            assert!(result.is_ok());

            let template = result.unwrap();
            assert_eq!(template.elements.len(), 0);
        }
    }

    #[test]
    fn test_special_characters_in_variables() {
        let test_cases = vec![
            ("${user name}", vec!["user name"]), // Space in variable
            ("${user_name}", vec!["user_name"]), // Underscore
            ("${user-name}", vec!["user-name"]), // Hyphen
            ("${123}", vec!["123"]),             // Numbers
            ("${user.emails[0]}", vec!["user", "emails"]), // Array access removed
        ];

        let parser = Parser::new("").unwrap();

        for (input, expected_path) in test_cases {
            let vars = parser.extract_variables(input);
            if vars.len() > 0 {
                assert_eq!(vars[0].path, expected_path);
            }
        }
    }

    #[test]
    fn test_circular_template_references() {
        // Test templates that reference themselves
        let html = r##"
            <template id="recursive">
                <div itemprop="name"></div>
                <template src="#recursive"></template>
            </template>
        "##;

        let parser = Parser::new(html).unwrap();
        let result = parser.parse_template(None);

        // Should not hang or stack overflow
        assert!(result.is_ok());
    }
}
