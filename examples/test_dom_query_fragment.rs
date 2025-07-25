use dom_query::Document;

fn main() {
    println!("=== Testing dom_query fragment parsing ===\n");
    
    let fragment = r#"<div class="test">
        <p>Hello</p>
        <div data-constraint="true">Constraint</div>
    </div>"#;
    
    println!("Fragment HTML:\n{}\n", fragment);
    
    // Test 1: Using Document::from()
    println!("Test 1: Document::from(fragment)");
    let doc1 = Document::from(fragment);
    println!("Total elements: {}", doc1.select("*").length());
    println!("Body content: {:?}", doc1.select("body").html());
    println!("Divs found: {}", doc1.select("div").length());
    println!("Elements with data-constraint: {}", doc1.select("[data-constraint]").length());
    
    // Test 2: Wrapping in proper HTML
    println!("\nTest 2: Document::from with wrapped HTML");
    let wrapped_html = format!("<html><body>{}</body></html>", fragment);
    let doc2 = Document::from(wrapped_html.as_str());
    println!("Total elements: {}", doc2.select("*").length());
    println!("Body content: {:?}", doc2.select("body").html());
    println!("Divs found: {}", doc2.select("div").length());
    println!("Elements with data-constraint: {}", doc2.select("[data-constraint]").length());
    
    // Test 3: Using fragment() method if available
    println!("\nTest 3: Try creating from HTML directly");
    let full_html = format!("<!DOCTYPE html><html><head></head><body>{}</body></html>", fragment);
    let doc3 = Document::from(full_html.as_str());
    println!("Total elements: {}", doc3.select("*").length());
    println!("Body content: {:?}", doc3.select("body").html());
    println!("Divs found: {}", doc3.select("div").length());
    println!("Elements with data-constraint: {}", doc3.select("[data-constraint]").length());
}