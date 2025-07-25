use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Testing attribute preservation in arrays ===");
    
    let html = r#"
        <template>
            <div>
                <div itemprop="items[]">
                    <p data-test="hello">Item: <span itemprop="name"></span></p>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "items": [
            {"name": "First"},
            {"name": "Second"}
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);
    
    println!("\nChecking content:");
    println!("Contains 'data-test=\"hello\"': {}", result.contains("data-test=\"hello\""));
}