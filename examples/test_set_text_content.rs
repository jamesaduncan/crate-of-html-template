use dom_query::Document;
use html_template::node_ext::NodeExt;

fn main() {
    println!("=== Testing set_text_content behavior ===\n");
    
    let html = r#"<div class="product">
        <p>Price: ${price}
            <span data-constraint="true">Special!</span>
        </p>
    </div>"#;
    
    println!("Original HTML:\n{}\n", html);
    
    let doc = Document::from(html);
    
    // Before modification
    println!("Before modification:");
    println!("  Total elements: {}", doc.select("*").length());
    println!("  Elements with data-constraint: {}", doc.select("[data-constraint]").length());
    
    // Find the p element and set its text content
    let p_elements = doc.select("p");
    if let Some(p) = p_elements.nodes().first() {
        println!("\nOriginal p content: {:?}", p.html());
        p.set_text_content("Price: $100");
        println!("After set_text_content: {:?}", p.html());
    }
    
    // After modification
    println!("\nAfter modification:");
    println!("  Total elements: {}", doc.select("*").length());
    println!("  Elements with data-constraint: {}", doc.select("[data-constraint]").length());
}