use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    // Test array rendering with variables in attributes
    let html = r#"
        <template>
            <div class="users">
                <div itemprop="users[]">
                    <span>Name: ${name}</span>
                    <a href="mailto:${email}">Email</a>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();

    let data = json!({
        "users": [
            {
                "name": "Alice",
                "email": "alice@example.com"
            }
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);

    // Check results
    println!("\nChecks:");
    println!(
        "Name substitution: {}",
        if result.contains("Name: Alice") {
            "✓"
        } else {
            "✗"
        }
    );
    println!(
        "Email substitution: {}",
        if result.contains("mailto:alice@example.com") {
            "✓"
        } else {
            "✗"
        }
    );

    // Find what we actually got for the href
    if let Some(href_start) = result.find("href=\"") {
        let href_end = result[href_start + 6..].find("\"").unwrap_or(50);
        println!(
            "Actual href: {}",
            &result[href_start + 6..href_start + 6 + href_end]
        );
    }
}
