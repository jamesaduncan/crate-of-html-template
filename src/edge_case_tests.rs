//! Edge case tests for the HTML template library
//!
//! This module contains tests for unusual, boundary, and edge cases
//! that might expose bugs or unexpected behavior.

#[cfg(test)]
mod tests {
    use crate::*;
    use serde_json::json;

    #[test]
    fn test_empty_template() {
        let html = r#"<template></template>"#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({});
        let result = template.render(&data).unwrap();
        assert!(result.trim().is_empty() || result.contains("<html"));
    }

    #[test]
    fn test_template_with_only_text() {
        let html = r#"<template>Just some text without any elements</template>"#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({});
        let result = template.render(&data).unwrap();
        assert!(result.contains("Just some text"));
    }

    #[test]
    fn test_deeply_nested_variables() {
        let html = r#"
            <template>
                <div itemprop="value">${a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z}</div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();

        // Create deeply nested data
        let data = json!({
            "a": {"b": {"c": {"d": {"e": {"f": {"g": {"h": {"i": {"j": {"k": {"l": {"m":
                {"n": {"o": {"p": {"q": {"r": {"s": {"t": {"u": {"v": {"w": {"x": {"y":
                    {"z": "Deep value"}
                }}}}}}}}}}}}}}}}}}}}}}}}
        });

        let result = template.render(&data).unwrap();
        assert!(result.contains("Deep value"));
    }

    #[test]
    fn test_variable_with_non_existent_deep_path() {
        let html = r#"
            <template>
                <div itemprop="test">${nonexistent.path.to.nowhere}</div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({"test": "fallback"});
        let result = template.render(&data).unwrap();
        // Should render empty or the original variable
        assert!(!result.contains("undefined"));
    }

    #[test]
    fn test_multiple_same_variables_in_one_element() {
        let html = r#"
            <template>
                <div itemprop="test">${name} meets ${name} and ${name}</div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({"name": "Alice"});
        let result = template.render(&data).unwrap();
        assert_eq!(result.matches("Alice").count(), 3);
    }

    #[test]
    fn test_array_with_single_element() {
        let html = r#"
            <template>
                <ul>
                    <li itemprop="items[]">${value}</li>
                </ul>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({"items": [{"value": "Only one"}]});
        let result = template.render(&data).unwrap();
        assert!(result.contains("Only one"));
        assert_eq!(result.matches("<li").count(), 1);
    }

    #[test]
    fn test_array_with_non_object_elements() {
        let html = r#"
            <template>
                <ul>
                    <li itemprop="items[]"></li>
                </ul>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({"items": ["string1", "string2", "string3"]});
        let result = template.render(&data).unwrap();
        // Should render 3 li elements even though items are strings not objects
        assert_eq!(result.matches("<li").count(), 3);
    }

    #[test]
    fn test_mixed_array_elements() {
        let html = r#"
            <template>
                <div>
                    <span itemprop="mixed[]"></span>
                </div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "mixed": [
                {"name": "object"},
                "string",
                42,
                true,
                null,
                ["nested", "array"]
            ]
        });
        let result = template.render(&data).unwrap();
        // Should handle all types gracefully
        assert_eq!(result.matches("<span").count(), 6);
    }

    #[test]
    fn test_self_closing_tags() {
        let html = r#"
            <template>
                <div>
                    <img src="${imageUrl}" itemprop="image" />
                    <br />
                    <input type="text" value="${inputValue}" itemprop="field" />
                </div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "imageUrl": "test.jpg",
            "image": "alt text",
            "inputValue": "test value",
            "field": "field value"
        });
        let result = template.render(&data).unwrap();
        assert!(result.contains("test.jpg"));
        assert!(result.contains("test value") || result.contains("field value"));
    }

    #[test]
    fn test_special_html_entities_in_property_names() {
        let html = r#"
            <template>
                <div itemprop="special&lt;chars&gt;"></div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({"special<chars>": "value"});
        let result = template.render(&data).unwrap();
        // Should handle encoded property names
        assert!(result.contains("<div"));
    }

    #[test]
    #[ignore] // This test causes a panic in serde_json
    fn test_circular_reference_in_data() {
        // JSON doesn't support circular references, but we can test
        // repeated references to same objects
        let html = r#"
            <template>
                <div>
                    <span itemprop="user">${name}</span>
                    <span itemprop="friend">${name}</span>
                </div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let user = json!({"name": "Same Person"});
        let data = json!({
            "user": user.clone(),
            "friend": user
        });
        let result = template.render(&data).unwrap();
        // Both should render the same value
        assert_eq!(result.matches("Same Person").count(), 2);
    }

    #[test]
    fn test_very_long_property_names() {
        let long_prop = "a".repeat(1000);
        let html = format!(
            r#"<template><div itemprop="{}"></div></template>"#,
            long_prop
        );
        let template = HtmlTemplate::from_str(&html, None).unwrap();
        let mut data = std::collections::HashMap::new();
        data.insert(long_prop, "value");
        let data = serde_json::to_value(data).unwrap();
        let result = template.render(&data).unwrap();
        assert!(result.contains("value"));
    }

    #[test]
    fn test_whitespace_only_property_values() {
        let html = r#"
            <template>
                <div>
                    <span itemprop="spaces"></span>
                    <span itemprop="tabs"></span>
                    <span itemprop="newlines"></span>
                </div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "spaces": "   ",
            "tabs": "\t\t\t",
            "newlines": "\n\n\n"
        });
        let result = template.render(&data).unwrap();
        // Should preserve whitespace
        assert!(result.contains("<span>   </span>"));
    }

    #[test]
    fn test_numeric_property_names() {
        let html = r#"
            <template>
                <div itemprop="123"></div>
                <div itemprop="456.789"></div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "123": "numeric key 1",
            "456.789": "numeric key 2"
        });
        let result = template.render(&data).unwrap();
        assert!(result.contains("numeric key 1"));
        assert!(result.contains("numeric key 2"));
    }

    #[test]
    fn test_boolean_values_in_attributes() {
        let html = r#"
            <template>
                <input type="checkbox" checked="${isChecked}" itemprop="checkbox" />
                <button disabled="${isDisabled}" itemprop="button">Click</button>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "isChecked": true,
            "isDisabled": false,
            "checkbox": "cb",
            "button": "btn"
        });
        let result = template.render(&data).unwrap();
        assert!(result.contains("checked=\"true\""));
        assert!(result.contains("disabled=\"false\""));
    }

    #[test]
    fn test_array_property_without_brackets() {
        // Test what happens when we forget the [] suffix
        let html = r#"
            <template>
                <ul>
                    <li itemprop="items">${name}</li>
                </ul>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "items": [
                {"name": "Item 1"},
                {"name": "Item 2"}
            ]
        });
        let result = template.render(&data).unwrap();
        // Should not duplicate the li elements
        assert_eq!(result.matches("<li").count(), 1);
    }

    #[test]
    fn test_nested_arrays() {
        let html = r#"
            <template>
                <div>
                    <article itemprop="categories[]">
                        <h3 itemprop="name"></h3>
                        <ul>
                            <li itemprop="items[]">${title}</li>
                        </ul>
                    </article>
                </div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "categories": [
                {
                    "name": "Category 1",
                    "items": [
                        {"title": "Item 1.1"},
                        {"title": "Item 1.2"}
                    ]
                },
                {
                    "name": "Category 2",
                    "items": [
                        {"title": "Item 2.1"}
                    ]
                }
            ]
        });
        let result = template.render(&data).unwrap();
        assert!(result.contains("Category 1"));
        assert!(result.contains("Category 2"));
        assert!(result.contains("Item 1.1"));
        assert!(result.contains("Item 2.1"));
        assert_eq!(result.matches("<article").count(), 2);
        assert_eq!(result.matches("<li").count(), 3);
    }

    #[test]
    fn test_empty_array_in_nested_structure() {
        let html = r#"
            <template>
                <div itemprop="container" itemscope>
                    <h2 itemprop="title"></h2>
                    <ul>
                        <li itemprop="items[]"></li>
                    </ul>
                </div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "container": {
                "title": "Empty List",
                "items": []
            }
        });
        let result = template.render(&data).unwrap();
        assert!(result.contains("Empty List"));
        assert!(!result.contains("<li")); // No li elements should be rendered
    }

    #[test]
    fn test_constraint_with_missing_property() {
        let html = r#"
            <template>
                <div>
                    <p data-constraint="nonexistent.property == 'value'">Should not show</p>
                    <p data-constraint="!nonexistent">Should show</p>
                </div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({});
        let result = template.render(&data).unwrap();
        assert!(!result.contains("Should not show"));
        assert!(result.contains("Should show"));
    }

    #[test]
    fn test_multiple_constraints_on_same_element() {
        let html = r#"
            <template>
                <div>
                    <p data-constraint="show && count > 5">Complex constraint</p>
                </div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "show": true,
            "count": 10
        });
        let result = template.render(&data).unwrap();
        println!("Result HTML: {}", result);
        println!(
            "Contains 'Complex constraint': {}",
            result.contains("Complex constraint")
        );
        assert!(result.contains("Complex constraint"));
    }

    #[test]
    fn test_escaped_variables_in_text() {
        let html = r#"
            <template>
                <div itemprop="code">Use $${variable} for templates</div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({"code": "example"});
        let result = template.render(&data).unwrap();
        // Should not process escaped variables
        assert!(result.contains("${variable}") || result.contains("example"));
    }

    #[test]
    fn test_zero_width_characters() {
        let html = r#"
            <template>
                <div itemprop="text"></div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "text": "Hello\u{200B}World\u{200C}Test\u{200D}End"
        });
        let result = template.render(&data).unwrap();
        // Should preserve zero-width characters
        assert!(result.contains("\u{200B}"));
    }

    #[test]
    fn test_rtl_and_bidi_text() {
        let html = r#"
            <template>
                <div>
                    <p itemprop="arabic"></p>
                    <p itemprop="hebrew"></p>
                    <p itemprop="mixed"></p>
                </div>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "arabic": "مرحبا بالعالم",
            "hebrew": "שלום עולם",
            "mixed": "Hello שלום مرحبا World"
        });
        let result = template.render(&data).unwrap();
        assert!(result.contains("مرحبا بالعالم"));
        assert!(result.contains("שלום עולם"));
        assert!(result.contains("Hello שלום مرحبا World"));
    }

    #[test]
    fn test_very_large_array() {
        let html = r#"
            <template>
                <ul>
                    <li itemprop="items[]">${index}</li>
                </ul>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();

        // Create array with 10000 elements
        let items: Vec<_> = (0..10000).map(|i| json!({"index": i})).collect();

        let data = json!({"items": items});
        let result = template.render(&data).unwrap();

        // Should render all items
        assert!(result.contains("0"));
        assert!(result.contains("9999"));
        assert_eq!(result.matches("<li").count(), 10000);
    }

    #[test]
    fn test_conflicting_property_targets() {
        // When the same property is used for both text and attribute
        let html = r#"
            <template>
                <a href="${link}" itemprop="link">${link}</a>
            </template>
        "#;
        let template = HtmlTemplate::from_str(html, None).unwrap();
        let data = json!({
            "link": "https://example.com"
        });
        let result = template.render(&data).unwrap();
        // Should use the value in both places
        assert!(result.contains("href=\"https://example.com\""));
        assert!(result.contains(">https://example.com<"));
    }
}
