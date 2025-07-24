//! Troubleshooting Guide for html-template
//!
//! This example demonstrates solutions to common issues and debugging techniques
//! for the html-template library. Run with: cargo run --example troubleshooting_guide

use html_template::{CacheMode, Error, HtmlTemplate, HtmlTemplateBuilder, TemplateConfig};
use serde_json::json;

fn main() -> html_template::Result<()> {
    println!("=== HTML Template Troubleshooting Guide ===\n");

    // 1. Template Parsing Issues
    template_parsing_issues()?;

    // 2. Data Binding Problems
    data_binding_issues()?;

    // 3. Rendering Errors
    rendering_issues()?;

    // 4. Performance Problems
    performance_issues()?;

    // 5. Memory and Threading Issues
    memory_and_threading_issues()?;

    // 6. Debugging Techniques
    debugging_techniques()?;

    Ok(())
}

fn template_parsing_issues() -> html_template::Result<()> {
    println!("1. Template Parsing Issues");
    println!("==========================");

    // Issue 1: Malformed HTML
    println!("Problem: Malformed HTML");
    let bad_html = r#"<div>Unclosed div"#;

    match HtmlTemplate::from_str(bad_html, None) {
        Ok(_) => println!("✓ Template parsed successfully (dom_query is forgiving)"),
        Err(e) => println!("✗ Parse error: {}", e),
    }

    println!("Solution: Use well-formed HTML with proper closing tags");
    let good_html = r#"<div>Properly closed div</div>"#;
    let _template = HtmlTemplate::from_str(good_html, None)?;
    println!("✓ Fixed template parses successfully\n");

    // Issue 2: Missing itemprop attributes
    println!("Problem: Elements without itemprop don't bind data");
    let no_itemprop_html = r#"
        <template>
            <div>
                <h1>Title will be empty</h1>
                <p>Description will be empty</p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(no_itemprop_html, Some("div"))?;
    let data = json!({"title": "My Title", "description": "My Description"});
    let result = template.render(&data)?;
    println!("Result without itemprop: {}", result.trim());

    println!("Solution: Add itemprop attributes to bind data");
    let with_itemprop_html = r#"
        <template>
            <div>
                <h1 itemprop="title">Title will be replaced</h1>
                <p itemprop="description">Description will be replaced</p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(with_itemprop_html, Some("div"))?;
    let result = template.render(&data)?;
    println!("Result with itemprop: {}", result.trim());

    // Issue 3: Wrong selector
    println!("\nProblem: CSS selector doesn't match template structure");
    let html = r#"
        <template>
            <article>
                <h1 itemprop="title"></h1>
            </article>
        </template>
    "#;

    // Wrong selector
    match HtmlTemplate::from_str(html, Some("div")) {
        Ok(template) => {
            let result = template.render(&json!({"title": "Test"}))?;
            println!("Wrong selector result: '{}'", result.trim());
        }
        Err(e) => println!("Error with wrong selector: {}", e),
    }

    // Correct selector
    let template = HtmlTemplate::from_str(html, Some("article"))?;
    let result = template.render(&json!({"title": "Test"}))?;
    println!("Correct selector result: {}", result.trim());

    println!("\n" + &"=".repeat(60) + "\n");
    Ok(())
}

fn data_binding_issues() -> html_template::Result<()> {
    println!("2. Data Binding Problems");
    println!("========================");

    // Issue 1: Property names don't match
    println!("Problem: Property names in template don't match data");
    let html = r#"
        <template>
            <div>
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div"))?;

    // Wrong property names in data
    let wrong_data = json!({"heading": "My Title", "content": "My Description"});
    let result = template.render(&wrong_data)?;
    println!("With wrong property names: {}", result.trim());

    // Correct property names
    let correct_data = json!({"title": "My Title", "description": "My Description"});
    let result = template.render(&correct_data)?;
    println!("With correct property names: {}", result.trim());

    // Issue 2: Variable interpolation only works in itemprop elements
    println!("\nProblem: Variables ${} only work in elements with itemprop");
    let variable_html = r#"
        <template>
            <div>
                <h1>Hello, ${name}!</h1>
                <p itemprop="greeting">Welcome, ${name}!</p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(variable_html, Some("div"))?;
    let data = json!({"greeting": "Welcome message", "name": "Alice"});
    let result = template.render(&data)?;
    println!("Variable interpolation result: {}", result.trim());
    println!("Note: Variable in h1 not replaced, but in p it is");

    // Issue 3: Array handling
    println!("\nProblem: Array syntax [] not used correctly");
    let array_html = r#"
        <template>
            <ul>
                <li itemprop="items">Item template</li>
            </ul>
        </template>
    "#;

    let template = HtmlTemplate::from_str(array_html, Some("ul"))?;
    let array_data = json!({"items": ["Item 1", "Item 2", "Item 3"]});
    let result = template.render(&array_data)?;
    println!("Without array syntax: {}", result.trim());

    let array_html_fixed = r#"
        <template>
            <ul>
                <li itemprop="items[]">Item template</li>
            </ul>
        </template>
    "#;

    let template = HtmlTemplate::from_str(array_html_fixed, Some("ul"))?;
    let result = template.render(&array_data)?;
    println!("With array syntax []: {}", result.trim());

    println!("\n" + &"=".repeat(60) + "\n");
    Ok(())
}

fn rendering_issues() -> html_template::Result<()> {
    println!("3. Rendering Errors");
    println!("===================");

    // Issue 1: Form elements not working correctly
    println!("Problem: Form elements need special handlers");
    let form_html = r#"
        <template>
            <form>
                <input type="text" name="username" itemprop="username" />
                <select name="country" itemprop="country">
                    <option value="us">United States</option>
                    <option value="uk">United Kingdom</option>
                </select>
                <textarea name="message" itemprop="message"></textarea>
            </form>
        </template>
    "#;

    // Without handlers
    let template = HtmlTemplate::from_str(form_html, Some("form"))?;
    let form_data = json!({
        "username": "alice",
        "country": "uk",
        "message": "Hello world"
    });
    let result = template.render(&form_data)?;
    println!("Without handlers: {}", result.trim());

    // With default handlers
    let template = HtmlTemplateBuilder::new()
        .from_str(form_html)
        .with_selector("form")
        .with_default_handlers()
        .build()?;
    let result = template.render(&form_data)?;
    println!("With default handlers: {}", result.trim());

    // Issue 2: Missing data handling
    println!("\nProblem: How missing data is handled");
    let html = r#"
        <template>
            <div>
                <h1 itemprop="title"></h1>
                <p itemprop="missing_property"></p>
                <span itemprop="optional_field">Default content</span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div"))?;
    let partial_data = json!({"title": "Present Title"});
    let result = template.render(&partial_data)?;
    println!("With missing properties: {}", result.trim());
    println!("Note: Missing properties render as empty, existing content is preserved");

    // Issue 3: Type conversion
    println!("\nProblem: Non-string data types");
    let numeric_html = r#"
        <template>
            <div>
                <span itemprop="count"></span>
                <span itemprop="price"></span>
                <span itemprop="active"></span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(numeric_html, Some("div"))?;
    let numeric_data = json!({
        "count": 42,
        "price": 19.99,
        "active": true
    });
    let result = template.render(&numeric_data)?;
    println!("With numeric data: {}", result.trim());
    println!("Note: Numbers and booleans are automatically converted to strings");

    println!("\n" + &"=".repeat(60) + "\n");
    Ok(())
}

fn performance_issues() -> html_template::Result<()> {
    println!("4. Performance Problems");
    println!("=======================");

    // Issue 1: Template recompilation
    println!("Problem: Creating new templates for each render");
    println!("❌ Bad practice:");
    println!(
        r#"
    for item in items {{
        let template = HtmlTemplate::from_str(html, Some("div"))?; // Recompiles each time!
        let result = template.render(&item)?;
    }}
    "#
    );

    println!("✅ Good practice:");
    println!(
        r#"
    let template = HtmlTemplate::from_str(html, Some("div"))?; // Compile once
    for item in items {{
        let result = template.render(&item)?; // Reuse template
    }}
    "#
    );

    // Issue 2: No caching
    println!("\nProblem: Not using caching for repeated templates");
    let html = r#"<div itemprop="message"></div>"#;

    // Demonstrate caching benefit
    let no_cache_config = TemplateConfig::no_caching();
    let cached_config = TemplateConfig::aggressive_caching();

    println!("Without caching: Each template compilation is independent");
    for i in 0..3 {
        let _template =
            HtmlTemplate::from_str_with_config(html, Some("div"), no_cache_config.clone())?;
        println!("  Template {} compiled", i + 1);
    }

    println!("With caching: Template compiled once, reused from cache");
    for i in 0..3 {
        let _template =
            HtmlTemplate::from_str_with_config(html, Some("div"), cached_config.clone())?;
        println!("  Template {} (cached)", i + 1);
    }

    // Issue 3: Large dataset handling
    println!("\nProblem: Loading large datasets into memory");
    println!("❌ Bad for large data:");
    println!(
        r#"
    let mut all_results = Vec::new();
    for item in large_dataset {{
        all_results.push(template.render(&item)?); // Accumulates in memory
    }}
    "#
    );

    println!("✅ Good for large data:");
    println!(
        r#"
    let streaming_renderer = StreamingRenderer::new(&template)?;
    let mut stream = streaming_renderer.render_iter(large_dataset)?;
    while let Some(chunk) = stream.next_chunk()? {{
        // Process chunk immediately, constant memory usage
    }}
    "#
    );

    println!("\n" + &"=".repeat(60) + "\n");
    Ok(())
}

fn memory_and_threading_issues() -> html_template::Result<()> {
    println!("5. Memory and Threading Issues");
    println!("==============================");

    // Issue 1: Thread safety
    println!("Problem: Tests failing with concurrent access");
    println!("Solution: Run tests with single thread:");
    println!("  cargo test -- --test-threads=1");
    println!("This is due to global cache state in the current implementation");

    // Issue 2: Memory leaks
    println!("\nProblem: Memory usage growing over time");
    println!("Solutions:");
    println!("- Set cache capacity limits:");
    println!(
        r#"
    let config = TemplateConfig::default()
        .with_cache_mode(CacheMode::Normal); // Limits cache growth
    "#
    );

    println!("- Clear caches periodically in long-running applications");
    println!("- Use streaming for large datasets");
    println!("- Monitor memory usage with tools like valgrind");

    // Issue 3: String allocation
    println!("\nProblem: Excessive string allocations");
    println!("Solutions:");
    println!("- Enable zero-copy optimizations");
    println!("- Use Cow<str> in custom RenderValue implementations");
    println!("- Reuse data structures when possible");

    let html = r#"<div itemprop="message"></div>"#;
    let zero_copy_config = TemplateConfig::default().with_zero_copy(true);
    let template = HtmlTemplate::from_str_with_config(html, Some("div"), zero_copy_config)?;

    // Demonstrate efficient string usage
    let data = json!({"message": "Efficient string handling"});
    let _result = template.render(&data)?;
    println!("✓ Template uses zero-copy optimizations");

    println!("\n" + &"=".repeat(60) + "\n");
    Ok(())
}

fn debugging_techniques() -> html_template::Result<()> {
    println!("6. Debugging Techniques");
    println!("=======================");

    // Technique 1: Error inspection
    println!("1. Inspecting Errors");
    let bad_html = r#"<template><div itemprop="test">${invalid.path}</div></template>"#;

    match HtmlTemplate::from_str(bad_html, Some("div")) {
        Ok(template) => {
            let data = json!({"test": "value"});
            match template.render(&data) {
                Ok(_) => println!("Render succeeded"),
                Err(e) => {
                    println!("Render error: {}", e);
                    println!("Error type: {:?}", e);
                    // Check error chain
                    let mut source = e.source();
                    while let Some(err) = source {
                        println!("Caused by: {}", err);
                        source = err.source();
                    }
                }
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }

    // Technique 2: Template inspection
    println!("\n2. Template Structure Inspection");
    let html = r#"
        <template>
            <article itemscope itemtype="http://schema.org/Article">
                <h1 itemprop="headline"></h1>
                <div itemprop="author" itemscope>
                    <span itemprop="name"></span>
                </div>
                <div itemprop="tags[]" class="tag">
                    <span itemprop="name"></span>
                </div>
            </article>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("article"))?;
    println!("✓ Template parsed successfully");
    println!("  - Contains nested itemscope");
    println!("  - Has array property (tags[])");
    println!("  - Uses Schema.org Article type");

    // Technique 3: Data structure debugging
    println!("\n3. Data Structure Debugging");
    let data = json!({
        "headline": "Debug Article",
        "author": {"name": "Debug Author"},
        "tags": [
            {"name": "debugging"},
            {"name": "troubleshooting"}
        ]
    });

    println!("Data structure:");
    println!("{}", serde_json::to_string_pretty(&data)?);

    let result = template.render(&data)?;
    println!("\nRendered result:");
    println!("{}", result);

    // Technique 4: Step-by-step debugging
    println!("\n4. Step-by-step Debugging Process");
    println!("  1. Verify HTML structure is valid");
    println!("  2. Check CSS selector matches template elements");
    println!("  3. Ensure itemprop attributes are present");
    println!("  4. Verify data property names match itemprop values");
    println!("  5. Check array syntax [] is used for array properties");
    println!("  6. Test with minimal data first, then add complexity");
    println!("  7. Use pretty-printed JSON to inspect data structure");
    println!("  8. Enable debug logging if available");

    // Technique 5: Common debugging patterns
    println!("\n5. Common Debugging Patterns");
    println!("Minimal test case:");
    let minimal_html = r#"<div itemprop="test"></div>"#;
    let minimal_template = HtmlTemplate::from_str(minimal_html, None)?;
    let minimal_data = json!({"test": "working"});
    let minimal_result = minimal_template.render(&minimal_data)?;
    println!("Minimal case: '{}'", minimal_result.trim());

    println!("\nGradual complexity increase:");
    println!("  1. Start with single property");
    println!("  2. Add multiple properties");
    println!("  3. Add nested objects");
    println!("  4. Add arrays");
    println!("  5. Add variable interpolation");
    println!("  6. Add constraints");

    // Technique 6: Logging and monitoring
    println!("\n6. Logging and Monitoring");
    println!("Custom logging handler example:");
    println!(
        r#"
    struct LoggingHandler;
    
    impl ElementHandler for LoggingHandler {{
        fn can_handle(&self, element: &Selection) -> bool {{
            true // Log all elements
        }}
        
        fn handle(&self, element: &Selection, value: &dyn RenderValue) -> Result<()> {{
            println!("Processing: {{:?}}", element.node_name());
            Ok(())
        }}
        
        fn priority(&self) -> i32 {{ -10 }} // Low priority for logging
    }}
    "#
    );

    println!("\nUse environment variables for debug output:");
    println!("  RUST_LOG=debug cargo run");
    println!("  RUST_BACKTRACE=1 cargo run");

    println!("\n" + &"=".repeat(60) + "\n");
    Ok(())
}
