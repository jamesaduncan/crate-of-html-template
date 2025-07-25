use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    // Simpler nested test to isolate the issue
    let html = r#"
        <template>
            <div class="company">
                <h1 itemprop="name"></h1>
                <div itemprop="departments[]" class="department">
                    <h2 itemprop="name"></h2>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "name": "Tech Corp",
        "departments": [
            {
                "name": "Engineering"
            },
            {
                "name": "Marketing"
            }
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Result:\n{}", result);
    
    // Count departments
    let dept_count = result.matches(r#"class="department""#).count();
    println!("\nFound {} department elements", dept_count);
    
    // Check what names appear
    println!("Contains 'Tech Corp': {}", result.contains("Tech Corp"));
    println!("Contains 'Engineering': {}", result.contains("Engineering"));
    println!("Contains 'Marketing': {}", result.contains("Marketing"));
    
    // Look at h2 elements specifically
    let h2_count = result.matches("<h2").count();
    println!("Found {} h2 elements", h2_count);
}