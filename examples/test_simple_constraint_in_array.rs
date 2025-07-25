use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Testing simple constraint in array ===");
    
    let html = r#"
        <template>
            <div>
                <div itemprop="items[]">
                    <p itemprop="name"></p>
                    <div data-constraint="active">
                        <span>ACTIVE</span>
                    </div>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "items": [
            {"name": "Item 1", "active": true},
            {"name": "Item 2", "active": false}
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);
    
    println!("\nExpected: Item 1 should show ACTIVE, Item 2 should not");
}