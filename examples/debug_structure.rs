use dom_query::Document;

fn main() {
    let html = r#"<p>Email: <a href="mailto:${email}" itemprop="email"></a></p>"#;

    let doc = Document::from(html);

    // Check the p element
    let p = doc.select("p");
    println!("P element HTML: {}", p.html());
    println!("P element text: '{}'", p.text());
    println!("P element inner HTML: {}", p.inner_html());

    // Check the anchor
    let a = doc.select("a");
    println!("\nA element HTML: {}", a.html());
    println!("A element text: '{}'", a.text());
    println!("A element href: {:?}", a.attr("href"));

    // Test what happens when we process the template
    println!("\n--- Processing test ---");

    // Process href attribute
    a.set_attr("href", "mailto:alice@example.com");
    println!("After setting href: {}", a.html());

    // Process text content
    a.set_text("alice@example.com");
    println!("After setting text: {}", a.html());

    println!("\nFinal P element: {}", p.html());
}
