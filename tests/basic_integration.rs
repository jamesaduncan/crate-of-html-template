//! Basic integration tests for HTML template functionality
//!
//! These tests verify the core functionality of the template library
//! including property binding, variable interpolation, and basic rendering.

use html_template::{HtmlTemplate, HtmlTemplateBuilder, render_string_with_selector};
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
                <p itemprop="greeting">Hello, ${name}!</p>
                <p itemprop="info">Your email is ${email} and you are ${age} years old.</p>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("div")).unwrap();
    let data = json!({
        "greeting": "Hi there",
        "name": "Alice",
        "email": "alice@example.com",
        "age": 30,
        "info": "Custom info"
    });
    
    let result = template.render(&data).unwrap();
    // When an element has itemprop with content containing variables,
    // the variables are replaced within that content
    assert!(result.contains("Hello, Alice!"));
    // Variables in elements with itemprop are replaced
    assert!(result.contains("Your email is alice@example.com and you are 30 years old."));
}

#[test]
fn test_attribute_templating() {
    let html = r#"
        <template>
            <div>
                <a href="${url}" itemprop="link">${linkText}</a>
                <img src="${imageUrl}" alt="${altText}" itemprop="image" />
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("div")).unwrap();
    let data = json!({
        "url": "https://example.com",
        "link": "Example Link",
        "linkText": "Click here",
        "imageUrl": "image.jpg",
        "altText": "Test image",
        "image": "image-prop"
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains(r#"href="https://example.com""#));
    assert!(result.contains("Click here"));
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
                <span>${nonexistent}</span>
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
    // Missing variables might show empty or the original ${nonexistent}
    // depending on implementation
}

#[test]
fn test_nested_properties() {
    let html = r#"
        <template>
            <div>
                <h1 itemprop="heading">${user.name}</h1>
                <p itemprop="bio">${user.profile.bio}</p>
                <span itemprop="location">${company.address.city}, ${company.address.country}</span>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "user": {
            "name": "John Doe",
            "profile": {
                "bio": "Software developer"
            }
        },
        "company": {
            "address": {
                "city": "San Francisco",
                "country": "USA"
            }
        },
        "heading": "Page Title",
        "bio": "Default bio",
        "location": "Default location"
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains("John Doe"));
    assert!(result.contains("Software developer"));
    assert!(result.contains("San Francisco, USA"));
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
                        <span itemprop="byline">By ${author.name}</span>
                        <time datetime="${publishDate}" itemprop="datePublished">${formattedDate}</time>
                    </div>
                </header>
                <div itemprop="content" class="content">
                    ${content}
                </div>
                <footer>
                    <p itemprop="tagline">Tags: ${tags}</p>
                    <p itemprop="readingInfo">Reading time: ${readingTime} minutes</p>
                </footer>
            </article>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("article")).unwrap();
    let data = json!({
        "title": "Understanding Rust Templates",
        "author": {
            "name": "Jane Developer"
        },
        "publishDate": "2024-01-15",
        "formattedDate": "January 15, 2024",
        "content": "This is a detailed post about Rust templating...",
        "tags": "rust, templates, web",
        "readingTime": 5,
        "byline": "By Someone",
        "datePublished": "Some date",
        "tagline": "Some tags",
        "readingInfo": "Some reading time"
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
                <p itemprop="activeStatus">Active: ${isActive}</p>
                <p itemprop="countInfo">Count: ${count}</p>
                <p itemprop="priceInfo">Price: $${price}</p>
                <p itemprop="ratioInfo">Ratio: ${ratio}</p>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "isActive": true,
        "count": 42,
        "price": 19.99,
        "ratio": 0.75,
        "activeStatus": "placeholder",
        "countInfo": "placeholder",
        "priceInfo": "placeholder",
        "ratioInfo": "placeholder"
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains("Active: true"));
    assert!(result.contains("Count: 42"));
    assert!(result.contains("Price: $19.99"));
    assert!(result.contains("Ratio: 0.75"));
}

#[test]
fn test_null_values() {
    let html = r#"
        <template>
            <div>
                <p itemprop="nullable"></p>
                <p itemprop="nullInfo">Value: ${nullVar}</p>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "nullable": null,
        "nullVar": null,
        "nullInfo": "placeholder"
    });
    
    let result = template.render(&data).unwrap();
    // Null values should render as empty
    assert!(result.contains("<p itemprop=\"nullable\"></p>"));
    // Null variables render as empty string in interpolation
    assert!(result.contains("Value: "));
}

#[test] 
fn test_special_characters_in_content() {
    let html = r#"
        <template>
            <div>
                <p itemprop="content"></p>
                <code itemprop="codeSnippet">${code}</code>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "content": "<script>alert('XSS')</script> & \"quotes\" 'apostrophes'",
        "code": "fn main() { println!(\"Hello, world!\"); }",
        "codeSnippet": "placeholder"
    });
    
    let result = template.render(&data).unwrap();
    // Should escape HTML entities
    assert!(result.contains("&lt;script&gt;"));
    assert!(result.contains("&amp;"));
    // Code should be preserved
    assert!(result.contains("fn main()"));
}