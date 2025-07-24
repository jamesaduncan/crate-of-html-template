//! Regression tests to prevent previously fixed issues from reoccurring
//!
//! These tests capture specific bug fixes and edge cases that were problematic
//! in earlier versions of the library.

use html_template::{HtmlTemplate, HtmlTemplateBuilder};
use serde_json::json;

#[test]
fn test_regression_empty_array_handling() {
    // Regression test: Empty arrays should not crash or produce invalid HTML
    let html = r#"
        <template>
            <div class="container">
                <ul>
                    <li itemprop="items[]">${name}</li>
                </ul>
                <p>Count: ${count}</p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div.container")).unwrap();
    let data = json!({
        "items": [],
        "count": 0
    });

    let result = template.render(&data).unwrap();

    // Should not contain any <li> elements for empty array
    assert!(!result.contains("<li"));
    // Should still contain the container structure
    assert!(result.contains("<ul>"));
    assert!(result.contains("</ul>"));
    // Non-array properties should still work
    assert!(result.contains("Count: 0"));
}

#[test]
fn test_regression_null_property_handling() {
    // Regression test: Null properties should render as empty, not crash
    let html = r#"
        <template>
            <div>
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
                <span itemprop="optional"></span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "title": "Test Title",
        "description": null,
        "optional": null
    });

    let result = template.render(&data).unwrap();

    assert!(result.contains("Test Title"));
    // Null properties should render as empty elements
    assert!(result.contains(r#"<p itemprop="description"></p>"#));
    assert!(result.contains(r#"<span itemprop="optional"></span>"#));
}

#[test]
fn test_regression_special_characters_escaping() {
    // Regression test: HTML entities should be properly escaped
    let html = r#"
        <template>
            <div>
                <p itemprop="content"></p>
                <div itemprop="code"></div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "content": "<script>alert('xss')</script> & \"quotes\" 'apostrophes'",
        "code": "if (x < y && y > z) { return true; }"
    });

    let result = template.render(&data).unwrap();

    // HTML should be escaped
    assert!(result.contains("&lt;script&gt;"));
    assert!(result.contains("&amp;"));
    assert!(result.contains("&quot;"));
    // But structure should be preserved
    assert!(result.contains("if (x"));
}

#[test]
fn test_regression_nested_itemscope_data_isolation() {
    // Regression test: Nested itemscope should properly isolate data contexts
    let html = r#"
        <template>
            <article itemscope>
                <h1 itemprop="title"></h1>
                <div itemprop="author" itemscope>
                    <span itemprop="name"></span>
                    <span itemprop="email"></span>
                </div>
                <p itemprop="content"></p>
            </article>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("article")).unwrap();
    let data = json!({
        "title": "Article Title",
        "author": {
            "name": "John Doe",
            "email": "john@example.com"
        },
        "content": "Article content",
        // These shouldn't interfere with nested scope
        "name": "Wrong Name",
        "email": "wrong@email.com"
    });

    let result = template.render(&data).unwrap();

    assert!(result.contains("Article Title"));
    assert!(result.contains("Article content"));
    // Should use nested author data, not root-level name/email
    assert!(result.contains("John Doe"));
    assert!(result.contains("john@example.com"));
    assert!(!result.contains("Wrong Name"));
    assert!(!result.contains("wrong@email.com"));
}

#[test]
fn test_regression_malformed_variable_syntax() {
    // Regression test: Malformed variables should be handled gracefully
    let html = r#"
        <template>
            <div>
                <p itemprop="content1">${valid}</p>
                <p itemprop="content2">${}</p>
                <p itemprop="content3">${unclosed</p>
                <p itemprop="content4">$valid_without_braces</p>
                <p itemprop="content5">${deeply.nested.property}</p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "valid": "Valid content",
        "content1": "replaced1",
        "content2": "replaced2",
        "content3": "replaced3",
        "content4": "replaced4",
        "content5": "replaced5",
        "deeply": {
            "nested": {
                "property": "Deep value"
            }
        }
    });

    let result = template.render(&data).unwrap();

    // Valid variables should work
    assert!(result.contains("Valid content"));
    assert!(result.contains("Deep value"));

    // Properties should still be replaced even with malformed variables in content
    assert!(result.contains("replaced1"));
    assert!(result.contains("replaced2"));
    assert!(result.contains("replaced3"));
    assert!(result.contains("replaced4"));
    assert!(result.contains("replaced5"));
}

#[test]
fn test_regression_array_property_name_collision() {
    // Regression test: Array properties shouldn't collide with non-array properties
    let html = r#"
        <template>
            <div>
                <h1 itemprop="items"></h1>
                <ul>
                    <li itemprop="items[]">${name}</li>
                </ul>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "items": [
            {"name": "Array Item 1"},
            {"name": "Array Item 2"}
        ]
    });

    let result = template.render(&data).unwrap();

    // The non-array itemprop="items" should get some representation of the array
    // while itemprop="items[]" should iterate through array items
    assert_eq!(result.matches("<li").count(), 2);
    assert!(result.contains("Array Item 1"));
    assert!(result.contains("Array Item 2"));
}

#[test]
fn test_regression_constraint_with_missing_properties() {
    // Regression test: Constraints referencing missing properties should not crash
    let html = r#"
        <template>
            <div>
                <h1 itemprop="title"></h1>
                <div data-constraint="nonexistent">
                    <p>Should not appear</p>
                </div>
                <div data-constraint="!nonexistent">
                    <p>Should appear</p>
                </div>
                <div data-constraint='missing == "value"'>
                    <p>Also should not appear</p>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "title": "Test Title"
    });

    let result = template.render(&data).unwrap();

    assert!(result.contains("Test Title"));
    assert!(!result.contains("Should not appear"));
    assert!(result.contains("Should appear"));
    assert!(!result.contains("Also should not appear"));
}

#[test]
fn test_regression_unicode_content_handling() {
    // Regression test: Unicode content should be preserved correctly
    let html = r#"
        <template>
            <div>
                <h1 itemprop="title"></h1>
                <p itemprop="content"></p>
                <span itemprop="emoji"></span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "title": "测试标题 العنوان Заголовок",
        "content": "Content with émojis: 🚀 🎉 ✨ and símböls: ñáéíóú",
        "emoji": "🔥"
    });

    let result = template.render(&data).unwrap();

    assert!(result.contains("测试标题"));
    assert!(result.contains("العنوان"));
    assert!(result.contains("Заголовок"));
    assert!(result.contains("🚀"));
    assert!(result.contains("ñáéíóú"));
    assert!(result.contains("🔥"));
}

#[test]
fn test_regression_whitespace_preservation() {
    // Regression test: Significant whitespace should be preserved
    let html = r#"
        <template>
            <pre itemprop="code"></pre>
            <div>
                <span itemprop="spaced">  spaced content  </span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "code": "function test() {\n    return true;\n}",
        "spaced": "  keep  spaces  "
    });

    let result = template.render(&data).unwrap();

    // Newlines and indentation should be preserved in code
    assert!(result.contains("{\n    return"));
    // Leading/trailing spaces should be preserved
    assert!(result.contains("  keep  spaces  "));
}

#[test]
fn test_regression_deep_nesting_stack_overflow() {
    // Regression test: Very deep nesting should not cause stack overflow
    let html = r#"
        <template>
            <div>
                <span itemprop="deep">${a.b.c.d.e.f.g.h.i.j}</span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "deep": "replaced",
        "a": {
            "b": {
                "c": {
                    "d": {
                        "e": {
                            "f": {
                                "g": {
                                    "h": {
                                        "i": {
                                            "j": "deep value"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let result = template.render(&data).unwrap();
    assert!(result.contains("deep value"));
}

#[test]
fn test_regression_boolean_constraint_evaluation() {
    // Regression test: Boolean values in constraints should work correctly
    let html = r#"
        <template>
            <div>
                <div data-constraint="isActive">
                    <p>Active</p>
                </div>
                <div data-constraint="!isHidden">
                    <p>Visible</p>
                </div>
                <div data-constraint="enabled == true">
                    <p>Enabled</p>
                </div>
                <div data-constraint="disabled == false">
                    <p>Not disabled</p>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "isActive": true,
        "isHidden": false,
        "enabled": true,
        "disabled": false
    });

    let result = template.render(&data).unwrap();

    assert!(result.contains("Active"));
    assert!(result.contains("Visible"));
    assert!(result.contains("Enabled"));
    assert!(result.contains("Not disabled"));
}

#[test]
fn test_regression_array_index_access() {
    // Regression test: Arrays should handle various access patterns
    let html = r#"
        <template>
            <div>
                <ul>
                    <li itemprop="items[]" class="item">
                        <span itemprop="name"></span>
                        <span>Position: ${index}</span>
                    </li>
                </ul>
                <p>First: ${first}</p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "items": [
            {"name": "Item A", "index": 0},
            {"name": "Item B", "index": 1},
            {"name": "Item C", "index": 2}
        ],
        "first": "First item outside array"
    });

    let result = template.render(&data).unwrap();

    assert!(result.contains("Item A"));
    assert!(result.contains("Position: 0"));
    assert!(result.contains("Item C"));
    assert!(result.contains("Position: 2"));
    assert!(result.contains("First item outside array"));
    assert_eq!(result.matches(r#"class="item""#).count(), 3);
}

#[test]
fn test_regression_handler_element_modification() {
    // Regression test: Element handlers should not interfere with template structure
    let html = r#"
        <template>
            <form>
                <select itemprop="country">
                    <option value="us">United States</option>
                    <option value="uk">United Kingdom</option>
                </select>
                <input type="text" itemprop="name">
                <textarea itemprop="bio"></textarea>
            </form>
        </template>
    "#;

    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("form")
        .with_default_handlers()
        .build()
        .unwrap();

    let data = json!({
        "country": "uk",
        "name": "John Doe",
        "bio": "Software developer"
    });

    let result = template.render(&data).unwrap();

    // Handlers should enhance, not break, the template
    assert!(result.contains("United States"));
    assert!(result.contains("United Kingdom"));
    assert!(result.contains(r#"value="John Doe""#));
    assert!(result.contains("Software developer"));

    // Structure should be maintained
    assert!(result.contains("<form>"));
    assert!(result.contains("</form>"));
    assert!(result.contains("<select"));
    assert!(result.contains("<input"));
    assert!(result.contains("<textarea"));
}
