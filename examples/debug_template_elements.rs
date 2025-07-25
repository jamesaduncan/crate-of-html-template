use html_template::HtmlTemplate;

fn main() {
    let html = r#"
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

    println!("=== Template Structure Debug ===");
    println!("HTML:");
    println!("{}", html);
    
    match HtmlTemplate::from_str(html, None) {
        Ok(template) => {
            println!("\nParsed template successfully");
            // We can't access internal fields, but we can see what renders
        }
        Err(e) => {
            println!("Error parsing template: {}", e);
        }
    }
    
    // Let's test with simpler structures to understand the issue
    println!("\n=== Testing simpler structure ===");
    let simple_html = r#"
        <template>
            <div>
                <h2 itemprop="name">Initial Name</h2>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(simple_html, None).unwrap();
    let data = serde_json::json!({
        "name": "Test Name"
    });
    
    let result = template.render(&data).unwrap();
    println!("Simple result: {}", result);
    println!("Contains 'Test Name': {}", result.contains("Test Name"));
}