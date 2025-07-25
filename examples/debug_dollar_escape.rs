use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Testing dollar sign escaping ===");
    
    // Test 1: Single dollar sign
    let html1 = r#"
        <template>
            <p itemprop="priceInfo">Price: ${price}</p>
        </template>
    "#;
    
    let template1 = HtmlTemplate::from_str(html1, None).unwrap();
    let data = json!({
        "price": 19.99,
        "priceInfo": "placeholder"
    });
    
    let result1 = template1.render(&data).unwrap();
    println!("Test 1 - Single $:");
    println!("HTML: <p itemprop=\"priceInfo\">Price: ${{price}}</p>");
    println!("Result: {}", result1.trim());
    println!("Expected: Price: 19.99");
    println!();
    
    // Test 2: Double dollar sign  
    let html2 = r#"
        <template>
            <p itemprop="priceInfo">Price: $${price}</p>
        </template>
    "#;
    
    let template2 = HtmlTemplate::from_str(html2, None).unwrap();
    let result2 = template2.render(&data).unwrap();
    println!("Test 2 - Double $$:");
    println!("HTML: <p itemprop=\"priceInfo\">Price: $${{price}}</p>");
    println!("Result: {}", result2.trim());
    println!("Expected: Price: $19.99");
}