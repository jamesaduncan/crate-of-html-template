use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <article>
                <h1 itemprop="title"></h1>
                <div itemprop="author" itemscope>
                    <span itemprop="name"></span>
                    <span itemprop="email"></span>
                    <p>Bio: <span itemprop="bio"></span></p>
                </div>
                <p itemprop="content"></p>
            </article>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("article")).unwrap();
    let data = json!({
        "title": "Understanding Nested Data",
        "author": {
            "name": "John Doe",
            "email": "john@example.com",
            "bio": "Senior developer with 10 years experience"
        },
        "content": "This article explains nested data structures..."
    });

    let result = template.render(&data).unwrap();

    println!("Result:\n{}", result);

    println!("\nChecks:");
    println!(
        "contains 'Understanding Nested Data': {}",
        result.contains("Understanding Nested Data")
    );
    println!("contains 'John Doe': {}", result.contains("John Doe"));
    println!(
        "contains 'john@example.com': {}",
        result.contains("john@example.com")
    );
    println!(
        "contains 'Bio: Senior developer': {}",
        result.contains("Bio: Senior developer")
    );
    println!(
        "contains 'This article explains': {}",
        result.contains("This article explains")
    );

    // Find what's actually in the bio section
    if let Some(bio_start) = result.find("Bio:") {
        let bio_end = result[bio_start..].find("</p>").unwrap_or(100);
        println!(
            "\nActual bio section: {}",
            &result[bio_start..bio_start + bio_end]
        );
    }
}
