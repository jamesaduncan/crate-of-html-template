use dom_query::Document;
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

    // Parse template to check structure
    let doc = Document::from(html);
    let template_root = doc.select("template");
    println!("Template HTML:\n{}\n", template_root.html());

    // Check if we can find the anchor tag
    let anchor = doc.select("a[itemprop='email']");
    println!("Found anchor tag: {}", anchor.length() > 0);
    if anchor.length() > 0 {
        println!("Anchor HTML: {}", anchor.html());
        println!("Anchor href: {:?}", anchor.attr("href"));
    }

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
    println!("\nRendered result:\n{}", result);

    // Check the rendered output
    let result_doc = Document::from(result.as_str());
    let rendered_anchor = result_doc.select("a");
    println!("\nFound anchor in result: {}", rendered_anchor.length() > 0);
    if rendered_anchor.length() > 0 {
        println!("Rendered anchor HTML: {}", rendered_anchor.html());
        println!("Rendered anchor href: {:?}", rendered_anchor.attr("href"));
    }
}
