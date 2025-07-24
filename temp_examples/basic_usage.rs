//! Basic usage examples for html-template
//!
//! This example demonstrates the fundamental usage patterns of the html-template library,
//! including basic property binding, arrays, nested objects, and variable interpolation.
//!
//! Run with: cargo run --example basic_usage

use html_template::{HtmlTemplate, HtmlTemplateBuilder, render_string_with_selector};
use serde_json::json;

fn main() -> html_template::Result<()> {
    println!("=== HTML Template Basic Usage Examples ===\n");

    // Example 1: Simple Property Binding
    simple_property_binding()?;
    
    // Example 2: Array Handling
    array_handling()?;
    
    // Example 3: Nested Objects
    nested_objects()?;
    
    // Example 4: Variable Interpolation
    variable_interpolation()?;
    
    // Example 5: Using the Builder Pattern
    builder_pattern()?;
    
    // Example 6: Convenience Functions
    convenience_functions()?;

    Ok(())
}

fn simple_property_binding() -> html_template::Result<()> {
    println!("1. Simple Property Binding");
    println!("=========================");
    
    let html = r#"
        <template>
            <div class="card">
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
                <span class="author">By: <em itemprop="author"></em></span>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("div.card"))?;
    
    let data = json!({
        "title": "Welcome to HTML Templates",
        "description": "This demonstrates basic property binding using microdata attributes.",
        "author": "The Template Engine"
    });
    
    let result = template.render(&data)?;
    println!("Template:");
    println!("{}", html.trim());
    println!("\nData:");
    println!("{}", serde_json::to_string_pretty(&data)?);
    println!("\nResult:");
    println!("{}", result);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}

fn array_handling() -> html_template::Result<()> {
    println!("2. Array Handling");
    println!("=================");
    
    let html = r#"
        <template>
            <div class="product-list">
                <h2>Products</h2>
                <div itemprop="products[]" class="product">
                    <h3 itemprop="name"></h3>
                    <p class="price">$<span itemprop="price"></span></p>
                    <p itemprop="description"></p>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("div.product-list"))?;
    
    let data = json!({
        "products": [
            {
                "name": "Laptop Pro",
                "price": "1299.99",
                "description": "High-performance laptop for professionals"
            },
            {
                "name": "Wireless Mouse",
                "price": "29.99", 
                "description": "Ergonomic wireless mouse with precision tracking"
            },
            {
                "name": "Mechanical Keyboard",
                "price": "149.99",
                "description": "Premium mechanical keyboard with RGB lighting"
            }
        ]
    });
    
    let result = template.render(&data)?;
    println!("Template:");
    println!("{}", html.trim());
    println!("\nResult:");
    println!("{}", result);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}

fn nested_objects() -> html_template::Result<()> {
    println!("3. Nested Objects with itemscope");
    println!("================================");
    
    let html = r#"
        <template>
            <article class="blog-post" itemscope>
                <header>
                    <h1 itemprop="headline"></h1>
                    <div class="meta">
                        <time itemprop="datePublished"></time>
                    </div>
                </header>
                
                <div itemprop="author" itemscope class="author-info">
                    <img itemprop="image" alt="Author photo" />
                    <div class="author-details">
                        <h3 itemprop="name"></h3>
                        <p itemprop="jobTitle"></p>
                        <a href="#" itemprop="email"></a>
                    </div>
                </div>
                
                <div class="content">
                    <p itemprop="articleBody"></p>
                </div>
            </article>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("article.blog-post"))?;
    
    let data = json!({
        "headline": "Building Modern Web Applications",
        "datePublished": "2024-01-15",
        "articleBody": "In this article, we explore the latest techniques for building scalable web applications...",
        "author": {
            "name": "Jane Developer",
            "jobTitle": "Senior Software Engineer",
            "email": "jane@example.com",
            "image": "https://example.com/jane-photo.jpg"
        }
    });
    
    let result = template.render(&data)?;
    println!("Template (simplified):");
    println!("  <article itemscope>");
    println!("    <h1 itemprop=\"headline\"></h1>");
    println!("    <div itemprop=\"author\" itemscope>");
    println!("      <h3 itemprop=\"name\"></h3>");
    println!("      <p itemprop=\"jobTitle\"></p>");
    println!("    </div>");
    println!("  </article>");
    println!("\nResult:");
    println!("{}", result);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}

fn variable_interpolation() -> html_template::Result<()> {
    println!("4. Variable Interpolation");
    println!("=========================");
    
    let html = r#"
        <template>
            <div class="greeting-card">
                <h1 itemprop="greeting">Hello, ${name}!</h1>
                <p itemprop="message">Welcome to ${platform}. You have ${count} new notifications.</p>
                <a href="${profileUrl}" itemprop="profileLink">View Profile</a>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("div.greeting-card"))?;
    
    let data = json!({
        "greeting": "Custom greeting placeholder",
        "message": "Custom message placeholder", 
        "profileLink": "Custom link placeholder",
        "name": "Alice",
        "platform": "HTML Template Engine",
        "count": 5,
        "profileUrl": "https://example.com/profile/alice"
    });
    
    let result = template.render(&data)?;
    println!("Template:");
    println!("{}", html.trim());
    println!("\nResult:");
    println!("{}", result);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}

fn builder_pattern() -> html_template::Result<()> {
    println!("5. Builder Pattern");
    println!("==================");
    
    let html = r#"
        <template>
            <form class="contact-form">
                <input type="text" name="name" itemprop="name" placeholder="Your name" />
                <input type="email" name="email" itemprop="email" placeholder="Your email" />
                <select name="topic" itemprop="topic">
                    <option value="general">General Inquiry</option>
                    <option value="support">Technical Support</option>
                    <option value="sales">Sales Question</option>
                </select>
                <textarea name="message" itemprop="message" placeholder="Your message"></textarea>
            </form>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("form.contact-form")
        .with_default_handlers()  // Enables form element handling
        .build()?;
    
    let data = json!({
        "name": "John Doe",
        "email": "john@example.com", 
        "topic": "support",
        "message": "I need help with template configuration."
    });
    
    let result = template.render(&data)?;
    println!("Using builder pattern with form handlers:");
    println!("Result:");
    println!("{}", result);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}

fn convenience_functions() -> html_template::Result<()> {
    println!("6. Convenience Functions");
    println!("========================");
    
    let html = r#"
        <template>
            <div class="quick-render">
                <h2 itemprop="title"></h2>
                <p itemprop="content"></p>
            </div>
        </template>
    "#;
    
    let data = json!({
        "title": "Quick Rendering",
        "content": "This uses the convenience function for one-off rendering."
    });
    
    // Using convenience function - no need to create template instance
    let result = render_string_with_selector(html, "div.quick-render", &data)?;
    
    println!("Using render_string_with_selector convenience function:");
    println!("Result:");
    println!("{}", result);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}