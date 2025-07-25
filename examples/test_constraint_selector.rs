use dom_query::Document;

fn main() {
    println!("=== Testing data-constraint selector ===\n");
    
    let html = r#"<div class="product">
        <h3>Name</h3>
        <div data-constraint="true">
            <button>Add to Cart</button>
        </div>
        <div data-constraint="false">
            <p>Out of Stock</p>
        </div>
    </div>"#;
    
    println!("HTML:\n{}\n", html);
    
    let doc = Document::from(html);
    
    // Test various selectors
    println!("Total elements (*): {}", doc.select("*").length());
    println!("All divs: {}", doc.select("div").length());
    
    // Try different attribute selectors
    println!("\nAttribute selector tests:");
    println!("[data-constraint]: {}", doc.select("[data-constraint]").length());
    println!("div[data-constraint]: {}", doc.select("div[data-constraint]").length());
    println!("*[data-constraint]: {}", doc.select("*[data-constraint]").length());
    
    // Check if we can find divs and then check their attributes
    println!("\nManual attribute check:");
    let divs = doc.select("div");
    for (i, div) in divs.nodes().iter().enumerate() {
        println!("Div {}: class={:?}, data-constraint={:?}", 
            i, 
            div.attr("class"),
            div.attr("data-constraint")
        );
    }
    
    // Try escaped attribute selector
    println!("\nEscaped selector tests:");
    println!("[data\\-constraint]: {}", doc.select("[data\\-constraint]").length());
    println!("div[data\\-constraint]: {}", doc.select("div[data\\-constraint]").length());
}