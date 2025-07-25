use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Debugging nested array processing ===");
    
    // Test with the exact same data structure as the failing test
    let html = r#"
        <template>
            <div class="categories">
                <section itemprop="categories[]" class="category">
                    <h2 itemprop="name"></h2>
                    <p itemprop="description"></p>
                    <ul>
                        <li itemprop="items[]">
                            <strong itemprop="title"></strong> - <span itemprop="price"></span>
                        </li>
                    </ul>
                </section>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "categories": [
            {
                "name": "Electronics",
                "description": "Latest gadgets and devices",
                "items": [
                    {"title": "Laptop", "price": "$999"},
                    {"title": "Phone", "price": "$699"}
                ]
            }
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Rendered output:");
    println!("{}", result);
    
    // Check what we're looking for
    println!("\nChecking content:");
    println!("Contains 'Electronics': {}", result.contains("Electronics"));
    println!("Contains 'Latest gadgets': {}", result.contains("Latest gadgets"));
    println!("Contains 'Laptop': {}", result.contains("Laptop"));
    println!("Contains '$999': {}", result.contains("$999"));
    
    // Check structure
    println!("Number of <li> elements: {}", result.matches("<li").count());
    println!("Number of category sections: {}", result.matches(r#"class="category""#).count());
    
    // Let's also check if we can see the raw structure
    println!("\nRaw structure analysis:");
    println!("Contains 'items[]': {}", result.contains("items[]"));
    println!("Contains '<li': {}", result.contains("<li"));
}