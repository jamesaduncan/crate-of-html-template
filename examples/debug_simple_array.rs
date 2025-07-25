use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Testing simple array only ===");
    
    let html = r#"
        <template>
            <div>
                <section itemprop="categories[]" class="category">
                    <h2 itemprop="name"></h2>
                </section>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "categories": [
            {"name": "Electronics"},
            {"name": "Books"}
        ]
    });
    
    println!("\nRendering with data: {}", serde_json::to_string_pretty(&data).unwrap());
    
    let result = template.render(&data).unwrap();
    println!("\nResult: {}", result);
    println!("\nContains 'Electronics': {}", result.contains("Electronics"));
    println!("Contains 'Books': {}", result.contains("Books"));
}