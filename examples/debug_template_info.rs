use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
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
    
    // Try simple test first - single level array
    let simple_data = json!({
        "categories": [
            {"name": "Test Category", "description": "Test Description"}
        ]
    });
    
    println!("Simple array test (no nested items):");
    let simple_result = template.render(&simple_data).unwrap();
    println!("{}", simple_result);
    
    // Now test with nested arrays
    let complex_data = json!({
        "categories": [
            {
                "name": "Electronics",
                "description": "Latest gadgets",
                "items": [
                    {"title": "Laptop", "price": "$999"}
                ]
            }
        ]
    });
    
    println!("\nComplex nested array test:");
    let complex_result = template.render(&complex_data).unwrap();
    println!("{}", complex_result);
    
    // Check what happens when we test the items array directly
    let items_only = json!({
        "items": [
            {"title": "Laptop", "price": "$999"}
        ]
    });
    
    let items_html = r#"
        <template>
            <ul>
                <li itemprop="items[]">
                    <strong itemprop="title"></strong> - <span itemprop="price"></span>
                </li>
            </ul>
        </template>
    "#;
    
    let items_template = HtmlTemplate::from_str(items_html, None).unwrap();
    
    println!("\nItems array test (isolated):");
    let items_result = items_template.render(&items_only).unwrap();
    println!("{}", items_result);
}