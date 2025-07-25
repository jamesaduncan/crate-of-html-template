use dom_query::Document;
use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    // Focus on just the problematic paragraph
    let html = r#"
        <template>
            <div itemprop="users[]">
                <p>Email: <a href="mailto:${email}" itemprop="email"></a></p>
            </div>
        </template>
    "#;

    // First, let's see what the parser finds
    let template = HtmlTemplate::from_str(html, None).unwrap();
    println!("Parsed template: {:#?}", template);

    // Now render
    let data = json!({
        "users": [
            {
                "email": "test@example.com"
            }
        ]
    });

    let result = template.render(&data).unwrap();
    println!("\nResult:\n{}", result);

    // Parse result to check structure
    let result_doc = Document::from(result.as_str());
    let p_elements = result_doc.select("p");
    println!("\nFound {} <p> elements", p_elements.length());

    if p_elements.length() > 0 {
        println!("P inner HTML: {}", p_elements.inner_html());
        println!("P text: '{}'", p_elements.text());

        let anchors = p_elements.select("a");
        println!("\nFound {} <a> elements inside <p>", anchors.length());
        if anchors.length() > 0 {
            println!("Anchor HTML: {}", anchors.html());
        }
    }
}
