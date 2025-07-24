//! Advanced features examples for html-template
//!
//! This example demonstrates advanced features including constraints, 
//! cross-document rendering, custom handlers, and streaming.
//!
//! Run with: cargo run --example advanced_features

use html_template::{HtmlTemplate, HtmlTemplateBuilder, StreamingRenderer};
use serde_json::json;

fn main() -> html_template::Result<()> {
    println!("=== HTML Template Advanced Features ===\n");

    // Example 1: Constraint-Based Conditional Rendering
    constraint_rendering()?;
    
    // Example 2: Cross-Document Rendering
    cross_document_rendering()?;
    
    // Example 3: Custom Element Handlers
    custom_handlers()?;
    
    // Example 4: Streaming Large Datasets
    streaming_example()?;
    
    // Example 5: Template Configuration
    template_configuration()?;

    Ok(())
}

fn constraint_rendering() -> html_template::Result<()> {
    println!("1. Constraint-Based Conditional Rendering");
    println!("==========================================");
    
    let html = r#"
        <template>
            <div class="user-dashboard">
                <header>
                    <h1>Welcome, <span itemprop="name"></span>!</h1>
                </header>
                
                <div data-constraint="isAdmin" class="admin-panel">
                    <h2>Admin Controls</h2>
                    <button>Manage Users</button>
                    <button>System Settings</button>
                </div>
                
                <div data-constraint="!isAdmin" class="user-panel">
                    <h2>Your Dashboard</h2>
                    <p>Account Type: <span itemprop="accountType"></span></p>
                </div>
                
                <div data-constraint='accountType == "premium"' class="premium-features">
                    <h3>Premium Features</h3>
                    <ul>
                        <li>Advanced Analytics</li>
                        <li>Priority Support</li>
                        <li>Custom Themes</li>
                    </ul>
                </div>
                
                <div data-constraint="credits > 0" class="credits-info">
                    <p>Available Credits: <span itemprop="credits"></span></p>
                </div>
                
                <div data-constraint="credits <= 0" class="no-credits">
                    <p class="warning">You have no credits remaining.</p>
                    <button>Purchase Credits</button>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("div.user-dashboard"))?;
    
    // Example 1: Admin user
    let admin_data = json!({
        "name": "Alice Admin",
        "isAdmin": true,
        "accountType": "premium",
        "credits": 150
    });
    
    println!("Admin User:");
    let result = template.render(&admin_data)?;
    println!("{}", result);
    println!("\n" + &"-".repeat(30) + "\n");
    
    // Example 2: Regular premium user
    let premium_data = json!({
        "name": "Bob Premium",
        "isAdmin": false,
        "accountType": "premium", 
        "credits": 25
    });
    
    println!("Premium User:");
    let result = template.render(&premium_data)?;
    println!("{}", result);
    println!("\n" + &"-".repeat(30) + "\n");
    
    // Example 3: Basic user with no credits
    let basic_data = json!({
        "name": "Charlie Basic",
        "isAdmin": false,
        "accountType": "basic",
        "credits": 0
    });
    
    println!("Basic User (No Credits):");
    let result = template.render(&basic_data)?;
    println!("{}", result);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}

fn cross_document_rendering() -> html_template::Result<()> {
    println!("2. Cross-Document Rendering");
    println!("===========================");
    
    // Simulate external HTML document with microdata
    let source_html = r#"
        <article itemscope itemtype="http://schema.org/Article">
            <h1 itemprop="headline">The Future of Web Development</h1>
            <meta itemprop="datePublished" content="2024-01-15">
            <div itemprop="author" itemscope itemtype="http://schema.org/Person">
                <span itemprop="name">Dr. Sarah Tech</span>
                <span itemprop="jobTitle">Lead Developer</span>
            </div>
            <div itemprop="articleBody">
                Web development continues to evolve with new frameworks and approaches...
            </div>
        </article>
    "#;
    
    let template_html = r#"
        <template>
            <div class="article-summary">
                <h2 itemprop="headline"></h2>
                <div class="meta">
                    <span>Published: <time itemprop="datePublished"></time></span>
                </div>
                <div itemprop="author" itemscope class="author">
                    <strong itemprop="name"></strong>
                    <em itemprop="jobTitle"></em>
                </div>
                <div class="content">
                    <p itemprop="articleBody"></p>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(template_html, Some("div.article-summary"))?;
    
    // Extract data from source HTML and render with template
    let results = template.render_from_html(source_html)?;
    
    println!("Source HTML (with microdata):");
    println!("{}", source_html.trim());
    println!("\nTemplate HTML:");
    println!("{}", template_html.trim());
    println!("\nRendered Result:");
    if !results.is_empty() {
        println!("{}", results[0]);
    }
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}

fn custom_handlers() -> html_template::Result<()> {
    println!("3. Custom Element Handlers");
    println!("==========================");
    
    let html = r#"
        <template>
            <div class="form-container">
                <h2>Contact Form</h2>
                <form>
                    <div class="field">
                        <label>Name:</label>
                        <input type="text" name="name" itemprop="name" />
                    </div>
                    
                    <div class="field">
                        <label>Email:</label>
                        <input type="email" name="email" itemprop="email" />
                    </div>
                    
                    <div class="field">
                        <label>Country:</label>
                        <select name="country" itemprop="country">
                            <option value="us">United States</option>
                            <option value="uk">United Kingdom</option>
                            <option value="ca">Canada</option>
                            <option value="de">Germany</option>
                        </select>
                    </div>
                    
                    <div class="field">
                        <label>Message:</label>
                        <textarea name="message" itemprop="message"></textarea>
                    </div>
                    
                    <div class="field">
                        <label>
                            <input type="checkbox" name="newsletter" itemprop="newsletter" />
                            Subscribe to newsletter
                        </label>
                    </div>
                </form>
            </div>
        </template>
    "#;
    
    // Build template with default handlers for form elements
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("div.form-container")
        .with_default_handlers()  // Enables input, select, textarea handlers
        .build()?;
    
    let data = json!({
        "name": "John Smith",
        "email": "john.smith@example.com",
        "country": "uk",  // This will be selected in the dropdown
        "message": "Hello! I'm interested in your services.",
        "newsletter": "checked"  // This will check the checkbox
    });
    
    let result = template.render(&data)?;
    
    println!("Form with custom handlers (notice selected/checked attributes):");
    println!("{}", result);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}

fn streaming_example() -> html_template::Result<()> {
    println!("4. Streaming Large Datasets");
    println!("===========================");
    
    let html = r#"
        <template>
            <div class="data-row">
                <span class="id" itemprop="id"></span>
                <span class="name" itemprop="name"></span>
                <span class="value" itemprop="value"></span>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("div.data-row"))?;
    
    // Create a large dataset
    let mut large_dataset = Vec::new();
    for i in 0..100 {
        large_dataset.push(json!({
            "id": i,
            "name": format!("Item {}", i),
            "value": format!("Value {}", i * 10)
        }));
    }
    
    println!("Processing {} items with streaming renderer...", large_dataset.len());
    
    // Use streaming renderer for large datasets
    let streaming_renderer = StreamingRenderer::new(&template)?;
    let mut stream = streaming_renderer.render_iter(large_dataset)?;
    
    let mut count = 0;
    let mut total_size = 0;
    
    // Process stream in chunks
    while let Some(chunk) = stream.next_chunk()? {
        count += 1;
        total_size += chunk.len();
        
        // Only print first few chunks for demo
        if count <= 3 {
            println!("Chunk {}: {}", count, chunk.trim());
        } else if count == 4 {
            println!("... (remaining {} chunks)", stream.remaining_count());
        }
    }
    
    println!("\nStreaming complete!");
    println!("Total chunks: {}", count);
    println!("Total output size: {} bytes", total_size);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}

fn template_configuration() -> html_template::Result<()> {
    println!("5. Template Configuration");
    println!("=========================");
    
    let html = r#"
        <template>
            <div class="config-demo">
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
                <ul>
                    <li itemprop="items[]" class="item">
                        <strong itemprop="name"></strong>: <span itemprop="value"></span>
                    </li>
                </ul>
            </div>
        </template>
    "#;
    
    use html_template::{TemplateConfig, CacheMode};
    
    // Create template with custom configuration
    let config = TemplateConfig::default()
        .with_cache_mode(CacheMode::Aggressive)
        .with_zero_copy(true);
    
    let template = HtmlTemplate::from_str_with_config(html, Some("div.config-demo"), config)?;
    
    let data = json!({
        "title": "Configuration Demo",
        "description": "This template uses aggressive caching and zero-copy optimizations.",
        "items": [
            {"name": "Cache Mode", "value": "Aggressive"},
            {"name": "Zero Copy", "value": "Enabled"},
            {"name": "Performance", "value": "Optimized"}
        ]
    });
    
    let result = template.render(&data)?;
    
    println!("Template with custom configuration:");
    println!("- Cache Mode: Aggressive");
    println!("- Zero Copy: Enabled");
    println!("- Optimizations: Active");
    println!("\nResult:");
    println!("{}", result);
    println!("\n" + &"=".repeat(50) + "\n");
    
    Ok(())
}