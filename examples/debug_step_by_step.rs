use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Testing nested arrays step by step ===");
    
    // Test 1: Simple outer array only (should work)
    println!("\n1. Simple outer array test:");
    let simple_html = r#"
        <template>
            <div>
                <section itemprop="categories[]" class="category">
                    <h2 itemprop="name"></h2>
                </section>
            </div>
        </template>
    "#;
    
    let simple_template = HtmlTemplate::from_str(simple_html, None).unwrap();
    let simple_data = json!({
        "categories": [
            {"name": "Electronics"},
            {"name": "Books"}
        ]
    });
    
    let simple_result = simple_template.render(&simple_data).unwrap();
    println!("Result: {}", simple_result);
    println!("Contains 'Electronics': {}", simple_result.contains("Electronics"));
    println!("Contains 'Books': {}", simple_result.contains("Books"));
    
    // Test 2: Simple inner array only (should work)
    println!("\n2. Simple inner array test:");
    let inner_html = r#"
        <template>
            <ul>
                <li itemprop="items[]">
                    <strong itemprop="title"></strong>
                </li>
            </ul>
        </template>
    "#;
    
    let inner_template = HtmlTemplate::from_str(inner_html, None).unwrap();
    let inner_data = json!({
        "items": [
            {"title": "Laptop"},
            {"title": "Phone"}
        ]
    });
    
    let inner_result = inner_template.render(&inner_data).unwrap();
    println!("Result: {}", inner_result);
    println!("Contains 'Laptop': {}", inner_result.contains("Laptop"));
    println!("Contains 'Phone': {}", inner_result.contains("Phone"));
    
    // Test 3: Full nested array (the failing case)
    println!("\n3. Full nested array test:");
    let nested_html = r#"
        <template>
            <div>
                <section itemprop="categories[]" class="category">
                    <h2 itemprop="name"></h2>
                    <ul>
                        <li itemprop="items[]">
                            <strong itemprop="title"></strong>
                        </li>
                    </ul>
                </section>
            </div>
        </template>
    "#;
    
    let nested_template = HtmlTemplate::from_str(nested_html, None).unwrap();
    let nested_data = json!({
        "categories": [
            {
                "name": "Electronics",
                "items": [
                    {"title": "Laptop"},
                    {"title": "Phone"}
                ]
            }
        ]
    });
    
    let nested_result = nested_template.render(&nested_data).unwrap();
    println!("Result: {}", nested_result);
    println!("Contains 'Electronics': {}", nested_result.contains("Electronics"));
    println!("Contains 'Laptop': {}", nested_result.contains("Laptop"));
    println!("Contains 'Phone': {}", nested_result.contains("Phone"));
    
    if !nested_result.contains("Laptop") {
        println!("\n=== NESTED ARRAYS STILL NOT WORKING ===");
        println!("The nested items array is not being rendered!");
    } else {
        println!("\n=== SUCCESS! Nested arrays are working! ===");
    }
}