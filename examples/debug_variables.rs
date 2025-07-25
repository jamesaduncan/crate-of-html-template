use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Testing variable replacement ===");
    
    let html = r#"
        <template>
            <div>
                <p itemprop="activeStatus">Active: ${isActive}</p>
                <p itemprop="countInfo">Count: ${count}</p>
                <p itemprop="priceInfo">Price: $${price}</p>
                <p itemprop="ratioInfo">Ratio: ${ratio}</p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "isActive": true,
        "count": 42,
        "price": 19.99,
        "ratio": 0.75,
        "activeStatus": "placeholder",
        "countInfo": "placeholder",
        "priceInfo": "placeholder",
        "ratioInfo": "placeholder"
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);
    
    println!("\nChecking content:");
    println!("Contains 'Active: true': {}", result.contains("Active: true"));
    println!("Contains 'Count: 42': {}", result.contains("Count: 42"));
    println!("Contains 'Price: $19.99': {}", result.contains("Price: $19.99"));
    println!("Contains 'Ratio: 0.75': {}", result.contains("Ratio: 0.75"));
}