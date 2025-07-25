use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <article class="blog-post">
                <div itemprop="content" class="content">
                </div>
            </article>
        </template>
    "#;

    println!("Creating template from HTML...");
    let template = HtmlTemplate::from_str(html, Some("article")).unwrap();
    
    println!("Template created successfully");
    
    let data = json!({
        "content": "This is a detailed post about Rust templating..."
    });
    
    println!("Rendering template...");
    let result = template.render(&data).unwrap();
    println!("Result: {}", result);
    
    // Try a simpler case
    println!("\n--- Testing simpler case ---");
    let simple_html = r#"
        <template>
            <div>
                <span itemprop="message"></span>
            </div>
        </template>
    "#;
    
    let simple_template = HtmlTemplate::from_str(simple_html, Some("div")).unwrap();
    let simple_data = json!({"message": "Hello World"});
    let simple_result = simple_template.render(&simple_data).unwrap();
    println!("Simple result: {}", simple_result);
    
    if simple_result.contains("Hello World") {
        println!("✓ Simple case works!");
    } else {
        println!("✗ Simple case also fails!");
    }
}