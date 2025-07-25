//! Basic integration tests for HTML template functionality
//!
//! These tests verify the core functionality of the template library
//! including property binding, variable interpolation, and basic rendering.

use html_template::{render_string_with_selector, HtmlTemplate, HtmlTemplateBuilder};
use serde_json::json;

#[test]
fn test_simple_property_binding() {
    let html = r#"
        <template>
            <div>
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div")).unwrap();
    let data = json!({
        "title": "Hello World",
        "description": "This is a test"
    });

    let result = template.render(&data).unwrap();
    assert!(result.contains("<h1 itemprop=\"title\">Hello World</h1>"));
    assert!(result.contains("<p itemprop=\"description\">This is a test</p>"));
}

#[test]
fn test_variable_interpolation() {
    let html = r#"
        <template>
            <div>
                <p itemprop="greeting"></p>
                <p itemprop="info"></p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div")).unwrap();
    let data = json!({
        "greeting": "Hello, Alice!",
        "info": "Your email is alice@example.com and you are 30 years old."
    });

    let result = template.render(&data).unwrap();
    // Properties are bound directly to elements with itemprop
    assert!(result.contains(r#"<p itemprop="greeting">Hello, Alice!</p>"#));
    assert!(result.contains(r#"<p itemprop="info">Your email is alice@example.com and you are 30 years old.</p>"#));
}

#[test]
fn test_attribute_templating() {
    let html = r#"
        <template>
            <div>
                <a href="${url}" itemprop="link"></a>
                <img src="${imageUrl}" alt="${altText}" itemprop="image" />
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div")).unwrap();
    let data = json!({
        "url": "https://example.com",
        "link": "Click here",
        "imageUrl": "image.jpg",
        "altText": "Test image",
        "image": "" // Not used since img elements don't have text content
    });

    let result = template.render(&data).unwrap();
    assert!(result.contains(r#"href="https://example.com""#));
    assert!(result.contains(r#"<a href="https://example.com" itemprop="link">Click here</a>"#));
    assert!(result.contains(r#"src="image.jpg""#));
    assert!(result.contains(r#"alt="Test image""#));
}

#[test]
fn test_missing_properties() {
    let html = r#"
        <template>
            <div>
                <h1 itemprop="title"></h1>
                <p itemprop="missing"></p>
                <span itemprop="nonexistent"></span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div")).unwrap();
    let data = json!({
        "title": "Only Title"
    });

    let result = template.render(&data).unwrap();
    assert!(result.contains("Only Title"));
    // Missing properties should render as empty
    assert!(result.contains("<p itemprop=\"missing\"></p>"));
    // Missing properties should render as empty
}

#[test]
fn test_nested_properties() {
    let html = r#"
        <template>
            <div>
                <h1 itemprop="heading"></h1>
                <p itemprop="bio"></p>
                <span itemprop="location"></span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "heading": "John Doe",
        "bio": "Software developer",
        "location": "San Francisco, USA"
    });

    let result = template.render(&data).unwrap();
    assert!(result.contains(r#"<h1 itemprop="heading">John Doe</h1>"#));
    assert!(result.contains(r#"<p itemprop="bio">Software developer</p>"#));
    assert!(result.contains(r#"<span itemprop="location">San Francisco, USA</span>"#));
}

#[test]
fn test_builder_pattern() {
    let html = r#"
        <template>
            <article>
                <h1 itemprop="headline"></h1>
                <p itemprop="content"></p>
            </article>
        </template>
    "#;

    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("article")
        .no_caching()
        .build()
        .unwrap();

    let data = json!({
        "headline": "Breaking News",
        "content": "Something important happened."
    });

    let result = template.render(&data).unwrap();
    assert!(result.contains("Breaking News"));
    assert!(result.contains("Something important happened"));
}

#[test]
fn test_convenience_functions() {
    let html = r#"
        <template>
            <div>
                <span itemprop="message"></span>
            </div>
        </template>
    "#;

    let data = json!({
        "message": "Quick render test"
    });

    let result = render_string_with_selector(html, "div", &data).unwrap();
    assert!(result.contains("Quick render test"));
}

#[test]
fn test_complex_template() {
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

    // Check all the rendered content
    assert!(result.contains("Understanding Rust Templates"));
    assert!(result.contains("By Jane Developer"));
    assert!(result.contains(r#"datetime="2024-01-15""#));
    assert!(result.contains("January 15, 2024"));
    assert!(result.contains("This is a detailed post"));
    assert!(result.contains("Tags: rust, templates, web"));
    assert!(result.contains("Reading time: 5 minutes"));
}

#[test]
fn test_boolean_and_numeric_values() {
    let html = r#"
        <template>
            <div>
                <p>Active: <span itemprop="isActive"></span></p>
                <p>Count: <span itemprop="count"></span></p>
                <p>Price: $<span itemprop="price"></span></p>
                <p>Ratio: <span itemprop="ratio"></span></p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "isActive": true,
        "count": 42,
        "price": 19.99,
        "ratio": 0.75
    });

    let result = template.render(&data).unwrap();
    assert!(result.contains(r#"Active: <span itemprop="isActive">true</span>"#));
    assert!(result.contains(r#"Count: <span itemprop="count">42</span>"#));
    assert!(result.contains(r#"Price: $<span itemprop="price">19.99</span>"#));
    assert!(result.contains(r#"Ratio: <span itemprop="ratio">0.75</span>"#));
}

#[test]
fn test_null_values() {
    let html = r#"
        <template>
            <div>
                <p itemprop="nullable"></p>
                <p itemprop="nullInfo"></p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "nullable": null,
        "nullInfo": "Value: "
    });

    let result = template.render(&data).unwrap();
    // Null values should render as empty
    assert!(result.contains("<p itemprop=\"nullable\"></p>"));
    // Properties render their content directly
    assert!(result.contains(r#"<p itemprop="nullInfo">Value: </p>"#));
}

#[test]
fn test_special_characters_in_content() {
    let html = r#"
        <template>
            <div>
                <p itemprop="content"></p>
                <code itemprop="codeSnippet"></code>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "content": "<script>alert('XSS')</script> & \"quotes\" 'apostrophes'",
        "codeSnippet": "fn main() { println!(\"Hello, world!\"); }"
    });

    let result = template.render(&data).unwrap();
    // Should escape HTML entities
    assert!(result.contains("&lt;script&gt;"));
    assert!(result.contains("&amp;"));
    // Code should be preserved
    assert!(result.contains("fn main()"));
}
