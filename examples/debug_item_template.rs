use dom_query::Document;

fn main() {
    // Simulate what happens during array rendering
    let department_template = r#"<div itemprop="departments[]" class="department">
                    <h2 itemprop="name"></h2>
                </div>"#;
    
    let item_doc = Document::from(department_template);
    let item_root = item_doc.select(":root > *");
    
    println!("Item root HTML: {}", item_root.html());
    
    // Try to find h2 elements
    let h2_elements = item_root.select("h2[itemprop=\"name\"]");
    println!("Found {} h2 elements with selector h2[itemprop=\"name\"]", h2_elements.length());
    
    if h2_elements.length() > 0 {
        println!("H2 HTML: {}", h2_elements.html());
    }
    
    // Also try a broader search
    let all_h2 = item_root.select("h2");
    println!("Found {} h2 elements total", all_h2.length());
    
    let all_name_props = item_root.select("[itemprop=\"name\"]");
    println!("Found {} elements with itemprop=\"name\"", all_name_props.length());
}