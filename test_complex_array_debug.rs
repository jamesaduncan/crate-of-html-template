use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <div class="users">
                <article itemprop="users[]" class="user">
                    <h3 itemprop="name"></h3>
                    <p>Email: <a href="mailto:${email}" itemprop="email"></a></p>
                    <p>Age: <span itemprop="age"></span></p>
                    <div class="tags">
                        Tags: <span itemprop="tags"></span>
                    </div>
                </article>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "users": [
            {
                "name": "Alice",
                "email": "alice@example.com",
                "age": 30,
                "tags": "developer, rust"
            }
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);
    
    // Check specific line
    if result.contains(r#"href="mailto:alice@example.com""#) {
        println!("✓ href attribute found correctly");
    } else {
        println!("✗ href attribute NOT found");
        println!("Looking for: href=\"mailto:alice@example.com\"");
        
        // Try to find what we actually have
        if let Some(start) = result.find("href=") {
            let snippet = &result[start..std::cmp::min(start + 50, result.len())];
            println!("Found href: {}", snippet);
        }
    }
}