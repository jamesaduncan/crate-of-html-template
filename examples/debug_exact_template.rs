use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    // Exact template from the failing test
    let html = r#"
        <template>
            <article class="blog-post">
                <header>
                    <h1 itemprop="title"></h1>
                    <div class="meta">
                        <span itemprop="byline"></span>
                        <time datetime="${publishDate}" itemprop="datePublished"></time>
                    </div>
                </header>
                <div itemprop="content" class="content">
                </div>
                <footer>
                    <p itemprop="tagline"></p>
                    <p itemprop="readingInfo"></p>
                </footer>
            </article>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("article")).unwrap();
    let data = json!({
        "title": "Understanding Rust Templates",
        "byline": "By Jane Developer", 
        "publishDate": "2024-01-15",
        "datePublished": "January 15, 2024",
        "content": "This is a detailed post about Rust templating...",
        "tagline": "Tags: rust, templates, web",
        "readingInfo": "Reading time: 5 minutes"
    });

    let result = template.render(&data).unwrap();
    println!("Rendered result:");
    println!("{}", result);
    
    // Check each property individually
    println!("\n=== Property checks ===");
    println!("title: {}", result.contains("Understanding Rust Templates"));
    println!("byline: {}", result.contains("By Jane Developer"));
    println!("publishDate (attr): {}", result.contains(r#"datetime="2024-01-15""#));
    println!("datePublished: {}", result.contains("January 15, 2024"));
    println!("content: {}", result.contains("This is a detailed post"));
    println!("tagline: {}", result.contains("Tags: rust, templates, web"));
    println!("readingInfo: {}", result.contains("Reading time: 5 minutes"));
}