use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <div class="categories">
                <section itemprop="categories[]" class="category">
                    <h2 itemprop="name"></h2>
                    <p itemprop="description"></p>
                    <ul>
                        <li itemprop="items[]">
                            <strong itemprop="title"></strong> - <span itemprop="price"></span>
                        </li>
                    </ul>
                </section>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    // Test with sample data
    let data = json!({
        "categories": [
            {
                "name": "Electronics",
                "description": "Latest gadgets and devices",
                "items": [
                    {"title": "Laptop", "price": "$999"},
                    {"title": "Phone", "price": "$599"}
                ]
            }
        ]
    });
    
    let result = template.render(&data).unwrap();
    println!("Rendered template:");
    println!("{}", result);
}