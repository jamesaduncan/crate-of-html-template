use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Testing simple constraint ===");
    
    // Simple test case
    let html = r#"
        <template>
            <div>
                <p>Price: <span itemprop="price"></span></p>
                <div data-constraint="price < 50">
                    <span>Budget Friendly!</span>
                </div>
                <div data-constraint="price > 100">
                    <span>Expensive!</span>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();

    // Test with price = 999 (should show "Expensive!" but not "Budget Friendly!")
    let data = json!({
        "price": 999
    });

    let result = template.render(&data).unwrap();
    println!("Result with price=999:\n{}", result);
    
    println!("\nChecking content:");
    println!("Contains 'Budget Friendly!': {}", result.contains("Budget Friendly!"));
    println!("Contains 'Expensive!': {}", result.contains("Expensive!"));
    
    // Test with price = 25 (should show "Budget Friendly!" but not "Expensive!")
    let data2 = json!({
        "price": 25
    });

    let result2 = template.render(&data2).unwrap();
    println!("\n\nResult with price=25:\n{}", result2);
    
    println!("\nChecking content:");
    println!("Contains 'Budget Friendly!': {}", result2.contains("Budget Friendly!"));
    println!("Contains 'Expensive!': {}", result2.contains("Expensive!"));
}