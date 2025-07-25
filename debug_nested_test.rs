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
    let data = json!({
        "categories": [
            {
                "name": "Electronics",
                "description": "Latest gadgets and devices",
                "items": [
                    {"title": "Laptop", "price": "$999"},
                    {"title": "Phone", "price": "$699"}
                ]
            },
            {
                "name": "Books",
                "description": "Educational and fiction books",
                "items": [
                    {"title": "Rust Programming", "price": "$45"}
                ]
            }
        ]
    });

    let result = template.render(&data).unwrap();
    println!("Rendered output:");
    println!("{}", result);
    
    println!("\nLooking for 'Laptop': {}", result.contains("Laptop"));
    println!("Looking for 'Electronics': {}", result.contains("Electronics"));
    println!("Number of <li> elements: {}", result.matches("<li").count());
    println!("Number of category sections: {}", result.matches(r#"class="category""#).count());
}