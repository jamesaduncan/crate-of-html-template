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
            },
            {
                "name": "Bob",
                "email": "bob@example.com",
                "age": 25,
                "tags": "designer, ui"
            }
        ]
    });

    let result = template.render(&data).unwrap();

    // Let's check what the test expects vs what we have
    println!("Checking test assertions:");
    println!("result.contains(\"Alice\"): {}", result.contains("Alice"));
    println!(
        "result.contains(\"alice@example.com\"): {}",
        result.contains("alice@example.com")
    );
    println!(
        "result.contains(\"Age: 30\"): {}",
        result.contains("Age: 30")
    );
    println!(
        "result.contains(\"Tags: developer, rust\"): {}",
        result.contains("Tags: developer, rust")
    );

    // Find what we actually have for age
    if let Some(age_idx) = result.find("Age:") {
        let snippet = &result[age_idx..std::cmp::min(age_idx + 50, result.len())];
        println!("\nActual Age snippet: {}", snippet);
    }

    // The href check
    println!("\nChecking href:");
    println!(
        "result.contains(r#\"href=\"mailto:alice@example.com\"\"#): {}",
        result.contains(r#"href="mailto:alice@example.com""#)
    );
}
