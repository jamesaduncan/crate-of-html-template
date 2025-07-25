use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Testing constraint preservation ===");
    
    // Test without array first
    let html = r#"
        <template>
            <div>
                <p itemprop="name"></p>
                <div data-constraint="active">
                    <span>ACTIVE</span>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "name": "Test",
        "active": true
    });

    let result = template.render(&data).unwrap();
    println!("Non-array result:\n{}", result);
    println!("Contains 'ACTIVE': {}", result.contains("ACTIVE"));
    
    // Now test with array
    let html2 = r#"
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

    let template2 = HtmlTemplate::from_str(html2, None).unwrap();
    let data2 = json!({
        "items": [
            {"name": "Item 1", "active": true}
        ]
    });

    let result2 = template2.render(&data2).unwrap();
    println!("\n\nArray result:\n{}", result2);
    println!("Contains 'ACTIVE': {}", result2.contains("ACTIVE"));
}