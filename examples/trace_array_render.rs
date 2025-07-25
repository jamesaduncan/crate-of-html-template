use dom_query::Document;
use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <article itemprop="users[]" class="user">
                <p>Email: <a href="mailto:${email}" itemprop="email"></a></p>
            </article>
        </template>
    "#;

    // First, let's manually trace what should happen
    let doc = Document::from(html);
    let template_elem = doc.select("template");

    // Get template contents
    if let Some(template_node) = template_elem.nodes().first() {
        let template_contents_id = template_node
            .query(|node| node.as_element().and_then(|elem| elem.template_contents))
            .flatten();

        if let Some(contents_id) = template_contents_id {
            let contents_node = dom_query::Node::new(contents_id, template_node.tree);
            let inner_html = contents_node.inner_html();
            println!("Template contents:\n{}", inner_html);

            // Parse the contents
            let content_doc = Document::from(inner_html.as_ref());

            // Find the article
            let article = content_doc.select("article");
            println!("\nArticle HTML:\n{}", article.html());

            // Clone the article HTML (this is what happens during array rendering)
            let article_html = article.html();
            let cloned_doc = Document::from(article_html.as_ref());

            // Now find elements with itemprop in the cloned doc
            let email_elem = cloned_doc.select("a[itemprop='email']");
            println!("\nEmail element in clone: {}", email_elem.html());
            println!("Email href: {:?}", email_elem.attr("href"));

            // What happens when we process the p element?
            let p_elem = cloned_doc.select("p");
            println!("\nP element: {}", p_elem.html());
            println!("P text: '{}'", p_elem.text());
            println!("P inner HTML: {}", p_elem.inner_html());
        }
    }

    println!("\n=== Now with HtmlTemplate ===\n");

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "users": [
            {
                "email": "alice@example.com"
            }
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);
}
