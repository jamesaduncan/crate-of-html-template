use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <div>
                <p>Active: <span itemprop="isActive"></span></p>
                <p>Count: <span itemprop="count"></span></p>
                <p>Price: $<span itemprop="price"></span></p>
                <p>Ratio: <span itemprop="ratio"></span></p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "isActive": true,
        "count": 42,
        "price": 19.99,
        "ratio": 0.75
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);
    
    println!("\nChecking content:");
    println!("Contains 'Active: true': {}", result.contains("Active: true"));
    println!("Contains 'Count: 42': {}", result.contains("Count: 42"));
    println!("Contains 'Price: $19.99': {}", result.contains("Price: $19.99"));
    println!("Contains 'Ratio: 0.75': {}", result.contains("Ratio: 0.75"));
}