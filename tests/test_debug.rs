use html_template::HtmlTemplate;
use serde_json::json;

#[test]
fn test_debug_nested_array() {
    let html = r#"
        <template>
            <ul>
                <li itemprop="items[]">
                    <span itemprop="name"></span>
                </li>
            </ul>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("ul")).unwrap();
    let data = json!({
        "items": [
            {"name": "Item 1"},
            {"name": "Item 2"}
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);

    assert!(result.contains("Item 1"));
    assert!(result.contains("Item 2"));
}
