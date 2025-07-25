use dom_query::{Document, Selection};

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

    let doc = Document::from(html);
    let template = doc.select("template");

    println!("Template found: {}", template.length() > 0);
    println!("Template HTML: {}", template.html());
    println!("Template inner HTML: {}", template.inner_html());

    // Try to get template node
    if let Some(template_node) = template.nodes().first() {
        println!("\nTemplate node found");

        // Try to access template contents
        let template_contents_id = template_node
            .query(|node| node.as_element().and_then(|elem| elem.template_contents))
            .flatten();

        println!("Template contents ID: {:?}", template_contents_id);

        if let Some(contents_id) = template_contents_id {
            let contents_node = dom_query::Node::new(contents_id, template_node.tree);
            let inner_html = contents_node.inner_html();
            println!("Template contents inner HTML: {}", inner_html);

            // Parse and check
            let content_doc = Document::from(inner_html.as_ref());
            let anchor = content_doc.select("a[itemprop='email']");
            println!(
                "\nFound anchor in template contents: {}",
                anchor.length() > 0
            );
        }
    }
}
