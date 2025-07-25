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

    println!("Data has {} users", data["users"].as_array().unwrap().len());

    let result = template.render(&data).unwrap();

    // Count how many times each user appears
    let alice_count = result.matches("Alice").count();
    let bob_count = result.matches("Bob").count();

    println!("Alice appears {} times", alice_count);
    println!("Bob appears {} times", bob_count);

    // Count article elements
    let article_count = result.matches(r#"class="user""#).count();
    println!("Found {} article elements", article_count);

    println!("\nFull output:\n{}", result);
}
