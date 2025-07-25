use dom_query::Document;

fn main() {
    println!("=== Testing data-constraint with itemprop ===\n");
    
    // This matches the exact HTML we're getting in render_array_item_html
    let html = r#"<div itemprop="products[]" class="product">
                    <h3 itemprop="name"></h3>
                    <p>Price: $${price}</p>
                    <div data-constraint="inStock">
                        <button>Add to Cart</button>
                    </div>
                    <div data-constraint="!inStock">
                        <p>Out of Stock</p>
                    </div>
                    <div data-constraint="price < 50">
                        <span class="badge">Budget Friendly!</span>
                    </div>
                </div>"#;
    
    println!("HTML:\n{}\n", html);
    
    let doc = Document::from(html);
    
    // Test various selectors
    println!("Total elements (*): {}", doc.select("*").length());
    println!("All divs: {}", doc.select("div").length());
    println!("Elements with itemprop: {}", doc.select("[itemprop]").length());
    
    // Try attribute selectors
    println!("\nConstraint selector tests:");
    println!("[data-constraint]: {}", doc.select("[data-constraint]").length());
    
    // Check body content
    let body = doc.select("body");
    if let Some(body_node) = body.nodes().first() {
        println!("\nBody inner HTML length: {}", body_node.inner_html().len());
        
        // Try selecting from body
        let body_sel = dom_query::Selection::from(body_node.clone());
        println!("Constraints in body selection: {}", body_sel.select("[data-constraint]").length());
    }
    
    // Manual check
    println!("\nManual div check:");
    let divs = doc.select("div");
    for (i, div) in divs.nodes().iter().enumerate() {
        if let Some(dc) = div.attr("data-constraint") {
            println!("  Div {} has data-constraint: {:?}", i, dc);
        }
    }
}