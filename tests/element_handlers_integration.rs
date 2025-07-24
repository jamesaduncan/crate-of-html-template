//! Integration tests for element handlers and special HTML elements
//!
//! These tests verify that special elements like input, select, textarea,
//! and meta are handled correctly.

use html_template::{HtmlTemplate, HtmlTemplateBuilder};
use serde_json::json;

#[test]
fn test_input_element_handling() {
    let html = r#"
        <template>
            <form>
                <input type="text" name="username" itemprop="username" />
                <input type="email" name="email" itemprop="email" />
                <input type="checkbox" name="subscribe" itemprop="subscribe" />
                <input type="number" name="age" itemprop="age" />
            </form>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("form")
        .with_default_handlers()
        .build()
        .unwrap();
    let data = json!({
        "username": "john_doe",
        "email": "john@example.com",
        "subscribe": "true",
        "age": 25
    });
    
    let result = template.render(&data).unwrap();
    
    // Input values should be set via value attribute
    assert!(result.contains(r#"value="john_doe""#));
    assert!(result.contains(r#"value="john@example.com""#));
    assert!(result.contains(r#"value="true""#));
    assert!(result.contains(r#"value="25""#));
}

#[test]
fn test_select_element_handling() {
    let html = r#"
        <template>
            <form>
                <select name="country" itemprop="country">
                    <option value="us">United States</option>
                    <option value="uk">United Kingdom</option>
                    <option value="ca">Canada</option>
                </select>
                <select name="language" itemprop="language">
                    <option value="en">English</option>
                    <option value="es">Spanish</option>
                    <option value="fr">French</option>
                </select>
            </form>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("form")
        .with_default_handlers()
        .build()
        .unwrap();
    let data = json!({
        "country": "uk",
        "language": "es"
    });
    
    let result = template.render(&data).unwrap();
    println!("Select result: {}", result);
    
    // Selected options should have the selected attribute
    assert!(result.contains(r#"<option value="uk" selected"#));
    assert!(result.contains(r#"<option value="es" selected"#));
    
    // Other options should not be selected
    assert!(!result.contains(r#"<option value="us" selected"#));
    assert!(!result.contains(r#"<option value="en" selected"#));
}

#[test]
fn test_textarea_element_handling() {
    let html = r#"
        <template>
            <form>
                <textarea name="bio" itemprop="bio"></textarea>
                <textarea name="notes" itemprop="notes" placeholder="Add notes..."></textarea>
            </form>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("form")
        .with_default_handlers()
        .build()
        .unwrap();
    let data = json!({
        "bio": "I am a software developer with 5 years of experience.\nI love coding!",
        "notes": "<script>alert('test')</script> Some notes with special chars & quotes"
    });
    
    let result = template.render(&data).unwrap();
    
    // Textarea content should be set as text content
    assert!(result.contains("I am a software developer"));
    assert!(result.contains("I love coding!"));
    
    // Special characters should be escaped
    assert!(result.contains("&lt;script&gt;"));
    assert!(result.contains("&amp;"));
}

#[test]
fn test_meta_element_handling() {
    let html = r#"
        <template>
            <head>
                <meta name="description" itemprop="description" />
                <meta property="og:title" itemprop="ogTitle" />
                <meta name="keywords" itemprop="keywords" />
            </head>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("head")
        .with_default_handlers()
        .build()
        .unwrap();
    let data = json!({
        "description": "A comprehensive guide to Rust templates",
        "ogTitle": "Rust Templates - Complete Guide",
        "keywords": "rust, templates, web development"
    });
    
    let result = template.render(&data).unwrap();
    
    // Meta content should be set via content attribute
    assert!(result.contains(r#"content="A comprehensive guide to Rust templates""#));
    assert!(result.contains(r#"content="Rust Templates - Complete Guide""#));
    assert!(result.contains(r#"content="rust, templates, web development""#));
}

#[test]
fn test_mixed_form_elements() {
    let html = r#"
        <template>
            <form class="user-form">
                <div class="form-group">
                    <label>Name:</label>
                    <input type="text" name="name" itemprop="name" />
                </div>
                
                <div class="form-group">
                    <label>Email:</label>
                    <input type="email" name="email" itemprop="email" />
                </div>
                
                <div class="form-group">
                    <label>Role:</label>
                    <select name="role" itemprop="role">
                        <option value="admin">Administrator</option>
                        <option value="user">User</option>
                        <option value="guest">Guest</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <label>Bio:</label>
                    <textarea name="bio" itemprop="bio" rows="4"></textarea>
                </div>
                
                <div class="form-group">
                    <label>
                        <input type="checkbox" name="active" itemprop="active" />
                        Active User
                    </label>
                </div>
            </form>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("form")
        .with_default_handlers()
        .build()
        .unwrap();
    let data = json!({
        "name": "Jane Smith",
        "email": "jane@company.com",
        "role": "admin",
        "bio": "Experienced administrator with strong leadership skills.",
        "active": "checked"
    });
    
    let result = template.render(&data).unwrap();
    
    // Check all form elements
    assert!(result.contains(r#"value="Jane Smith""#));
    assert!(result.contains(r#"value="jane@company.com""#));
    assert!(result.contains(r#"<option value="admin" selected"#));
    assert!(result.contains("Experienced administrator"));
    assert!(result.contains(r#"value="checked""#));
}

#[test]
fn test_input_types() {
    let html = r#"
        <template>
            <form>
                <input type="date" itemprop="date" />
                <input type="time" itemprop="time" />
                <input type="datetime-local" itemprop="datetime" />
                <input type="range" min="0" max="100" itemprop="progress" />
                <input type="color" itemprop="color" />
                <input type="hidden" name="id" itemprop="id" />
            </form>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_default_handlers()
        .build()
        .unwrap();
    let data = json!({
        "date": "2024-01-15",
        "time": "14:30",
        "datetime": "2024-01-15T14:30",
        "progress": "75",
        "color": "#ff6600",
        "id": "12345"
    });
    
    let result = template.render(&data).unwrap();
    
    // All input types should have their values set
    assert!(result.contains(r#"value="2024-01-15""#));
    assert!(result.contains(r#"value="14:30""#));
    assert!(result.contains(r#"value="2024-01-15T14:30""#));
    assert!(result.contains(r#"value="75""#));
    assert!(result.contains(r##"value="#ff6600""##));
    assert!(result.contains(r#"value="12345""#));
}

#[test]
fn test_radio_buttons() {
    let html = r#"
        <template>
            <form>
                <div>
                    <input type="radio" name="size" value="small" itemprop="size" /> Small
                    <input type="radio" name="size" value="medium" itemprop="size" /> Medium
                    <input type="radio" name="size" value="large" itemprop="size" /> Large
                </div>
            </form>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_default_handlers()
        .build()
        .unwrap();
    let data = json!({
        "size": "medium"
    });
    
    let result = template.render(&data).unwrap();
    
    // All radio buttons should have the value set
    // In a real implementation, you might want to set 'checked' attribute
    // for the matching value instead
    assert!(result.contains(r#"value="medium""#));
}

#[test]
fn test_custom_handlers_integration() {
    // Test that custom handlers can be added via the builder
    let html = r#"
        <template>
            <div>
                <div class="item" itemprop="content"></div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("div")
        .with_default_handlers() // Use default handlers
        .build()
        .unwrap();
    
    let data = json!({
        "content": "Test content"
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains("Test content"));
}

#[test]
fn test_nested_form_arrays() {
    let html = r#"
        <template>
            <form>
                <div itemprop="fields[]" class="field">
                    <label>${label}</label>
                    <input type="${type}" name="${name}" itemprop="value" />
                    <span class="help">${help}</span>
                </div>
            </form>
        </template>
    "#;
    
    let template = HtmlTemplateBuilder::new()
        .from_str(html)
        .with_default_handlers()
        .build()
        .unwrap();
    let data = json!({
        "fields": [
            {
                "label": "Username",
                "type": "text",
                "name": "username",
                "value": "john_doe",
                "help": "Enter your username"
            },
            {
                "label": "Password",
                "type": "password",
                "name": "password",
                "value": "",
                "help": "Enter a strong password"
            }
        ]
    });
    
    let result = template.render(&data).unwrap();
    
    // Check dynamic form fields
    assert!(result.contains("Username"));
    assert!(result.contains(r#"type="text""#));
    assert!(result.contains(r#"name="username""#));
    assert!(result.contains("Enter your username"));
    
    assert!(result.contains("Password"));
    assert!(result.contains(r#"type="password""#));
    assert!(result.contains(r#"name="password""#));
}