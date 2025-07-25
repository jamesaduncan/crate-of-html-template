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

    // Pretty print the output with line numbers
    println!("Rendered output:");
    println!("================");
    for (i, line) in result.lines().enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
    println!("================");

    // Check specific parts
    println!("\nChecks:");
    println!("Contains <a tag: {}", result.contains("<a"));
    println!("Contains href attribute: {}", result.contains("href="));
    println!("Contains mailto: {}", result.contains("mailto:"));
    println!(
        "Contains alice@example.com: {}",
        result.contains("alice@example.com")
    );
    println!(
        "Contains expected href: {}",
        result.contains(r#"href="mailto:alice@example.com""#)
    );

    // Find the anchor tag
    if let Some(start) = result.find("<a") {
        let end = result[start..]
            .find(">")
            .map(|e| start + e + 1)
            .unwrap_or(result.len());
        println!("\nAnchor tag found: {}", &result[start..end]);
    }
}
