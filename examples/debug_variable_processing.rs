use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    // Test with array
    let html = r#"
        <template>
            <div itemprop="users[]">
                <a href="mailto:${email}" itemprop="email"></a>
            </div>
        </template>
    "#;

    println!("Template HTML: {}", html);

    let template = HtmlTemplate::from_str(html, None).unwrap();

    let data = json!({
        "users": [{
            "email": "test@example.com"
        }]
    });

    let result = template.render(&data).unwrap();
    println!("\nRendered result: {}", result);

    // Check the result
    if result.contains("mailto:test@example.com") {
        println!("✓ Variable substitution worked!");
    } else {
        println!("✗ Variable substitution failed");
        if result.contains("mailto:") {
            println!("  Found 'mailto:' but variable not substituted");
            // Find what we got
            if let Some(start) = result.find("mailto:") {
                let end = result[start..].find("\"").unwrap_or(20);
                println!("  Found: {}", &result[start..start + end]);
            }
        }
    }
}
