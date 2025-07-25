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

    let template = HtmlTemplate::from_str(html, Some("article")).unwrap();
    
    let data = json!({
        "content": "This is a detailed post about Rust templating..."
    });
    
    let result = template.render(&data).unwrap();
    println!("Result: {}", result);
    
    // Check if content is in the result
    if result.contains("This is a detailed post") {
        println!("✓ Content was properly bound!");
    } else {
        println!("✗ Content was NOT bound!");
        
        // Show what's actually in the div
        let start = result.find("<div itemprop=\"content\"").unwrap_or(0);
        let end = result[start..].find("</div>").unwrap_or(result.len() - start) + start + 6;
        println!("Div content: {}", &result[start..end]);
    }
}
