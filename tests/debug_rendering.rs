//! Debug test to understand what's happening with cross-document rendering

use html_template::*;
use serde_json::json;

#[test]
fn debug_basic_rendering() {
    // Simple template
    let template_html = r#"
        <template>
            <div class="person-card">
                <h2 itemprop="name"></h2>
                <p class="email" itemprop="email"></p>
            </div>
        </template>
    "#;
    
    // Create template
    let template = HtmlTemplate::from_str(template_html, Some("div.person-card")).unwrap();
    
    // Simple data
    let data = json!({
        "name": "John Doe",
        "email": "john@example.com"
    });
    
    // Render with direct data
    let output = template.render(&data).unwrap();
    println!("Direct render output:\n{}", output);
    
    // Now test microdata extraction and rendering
    let source_html = r#"
        <div itemscope itemtype="https://schema.org/Person">
            <span itemprop="name">John Doe</span>
            <span itemprop="email">john@example.com</span>
        </div>
    "#;
    
    // Extract microdata
    let microdata_items = html_template::microdata::extract_microdata_from_html(source_html).unwrap();
    println!("Extracted microdata: {:#?}", microdata_items);
    
    // Render using microdata
    let cross_doc_results = template.render_from_html(source_html).unwrap();
    println!("Cross-document render output:\n{:#?}", cross_doc_results);
    
    if let Some(output) = cross_doc_results.first() {
        println!("First result:\n{}", output);
    }
}