use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    println!("=== Testing exact edge case test scenario ===");
    
    let html = r#"
        <template>
            <div>
                <article itemprop="categories[]">
                    <h3 itemprop="name"></h3>
                    <ul>
                        <li itemprop="items[]">${title}</li>
                    </ul>
                </article>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "categories": [
            {
                "name": "Category 1",
                "items": [
                    {"title": "Item 1.1"},
                    {"title": "Item 1.2"}
                ]
            },
            {
                "name": "Category 2",
                "items": [
                    {"title": "Item 2.1"}
                ]
            }
        ]
    });
    
    println!("Data: {}", serde_json::to_string_pretty(&data).unwrap());
    
    let result = template.render(&data).unwrap();
    println!("\nResult:\n{}", result);
    
    println!("\nChecking content:");
    println!("Contains 'Category 1': {}", result.contains("Category 1"));
    println!("Contains 'Category 2': {}", result.contains("Category 2"));
    println!("Contains 'Item 1.1': {}", result.contains("Item 1.1"));
    println!("Contains 'Item 1.2': {}", result.contains("Item 1.2"));
    println!("Contains 'Item 2.1': {}", result.contains("Item 2.1"));
    
    println!("\nCounts:");
    println!("Number of <article> elements: {}", result.matches("<article").count());
    println!("Number of <li> elements: {}", result.matches("<li").count());
}