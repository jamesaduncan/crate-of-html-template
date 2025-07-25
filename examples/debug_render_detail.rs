use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    // Simpler test case
    let html = r#"
        <template>
            <div itemprop="users[]">
                <a href="mailto:${email}" itemprop="email"></a>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    println!("Template parsed successfully");

    let data = json!({
        "users": [
            {
                "email": "alice@example.com"
            }
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);

    // Check if anchor exists
    if result.contains("<a") {
        println!("✓ Anchor tag preserved");
    } else {
        println!("✗ Anchor tag lost");
    }

    if result.contains("mailto:alice@example.com") {
        println!("✓ Href attribute rendered correctly");
    } else {
        println!("✗ Href attribute not rendered");
    }
}
