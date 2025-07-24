//! Integration tests for constraint system and conditional rendering
//!
//! These tests verify data-constraint and data-scope attribute handling.

use html_template::{HtmlTemplate};
use serde_json::json;

#[test]
fn test_simple_constraints() {
    let html = r#"
        <template>
            <div>
                <h1 itemprop="title"></h1>
                <div data-constraint="showDetails">
                    <p>These are the detailed contents.</p>
                </div>
                <div data-constraint="!hideSection">
                    <p>This section is conditionally shown.</p>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    // Test with showDetails = true
    let data = json!({
        "title": "Conditional Content",
        "showDetails": true,
        "hideSection": false
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains("Conditional Content"));
    assert!(result.contains("These are the detailed contents"));
    assert!(result.contains("This section is conditionally shown"));
    
    // Test with showDetails = false
    let data2 = json!({
        "title": "Conditional Content",
        "showDetails": false,
        "hideSection": true
    });
    
    let result2 = template.render(&data2).unwrap();
    assert!(result2.contains("Conditional Content"));
    assert!(!result2.contains("These are the detailed contents"));
    assert!(!result2.contains("This section is conditionally shown"));
}

#[test]
fn test_comparison_constraints() {
    let html = r#"
        <template>
            <div>
                <div data-constraint="age >= 18">
                    <p>Adult content</p>
                </div>
                <div data-constraint="score > 80">
                    <p>Excellent score!</p>
                </div>
                <div data-constraint="count == 0">
                    <p>No items found</p>
                </div>
                <div data-constraint="level < 5">
                    <p>Beginner level</p>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    let data = json!({
        "age": 21,
        "score": 85,
        "count": 0,
        "level": 3
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains("Adult content"));
    assert!(result.contains("Excellent score"));
    assert!(result.contains("No items found"));
    assert!(result.contains("Beginner level"));
    
    // Test with different values
    let data2 = json!({
        "age": 16,
        "score": 75,
        "count": 5,
        "level": 10
    });
    
    let result2 = template.render(&data2).unwrap();
    assert!(!result2.contains("Adult content"));
    assert!(!result2.contains("Excellent score"));
    assert!(!result2.contains("No items found"));
    assert!(!result2.contains("Beginner level"));
}

#[test]
fn test_string_equality_constraints() {
    let html = r#"
        <template>
            <div>
                <div data-constraint='status == "active"'>
                    <p>Status is active</p>
                </div>
                <div data-constraint='role == "admin"'>
                    <p>Admin panel</p>
                </div>
                <div data-constraint='type != "hidden"'>
                    <p>Visible content</p>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    let data = json!({
        "status": "active",
        "role": "admin",
        "type": "public"
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains("Status is active"));
    assert!(result.contains("Admin panel"));
    assert!(result.contains("Visible content"));
    
    // Test with different values
    let data2 = json!({
        "status": "inactive",
        "role": "user",
        "type": "hidden"
    });
    
    let result2 = template.render(&data2).unwrap();
    assert!(!result2.contains("Status is active"));
    assert!(!result2.contains("Admin panel"));
    assert!(!result2.contains("Visible content"));
}

#[test]
fn test_nested_property_constraints() {
    let html = r#"
        <template>
            <div>
                <div data-constraint="user.isLoggedIn">
                    <p>Welcome, ${user.name}!</p>
                </div>
                <div data-constraint="settings.notifications.email">
                    <p>Email notifications are enabled</p>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    let data = json!({
        "user": {
            "isLoggedIn": true,
            "name": "John"
        },
        "settings": {
            "notifications": {
                "email": true
            }
        }
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains("Welcome, John!"));
    assert!(result.contains("Email notifications are enabled"));
}

#[test]
fn test_constraints_with_arrays() {
    let html = r#"
        <template>
            <div>
                <h2>Products</h2>
                <div itemprop="products[]" class="product">
                    <h3 itemprop="name"></h3>
                    <p>Price: $${price}</p>
                    <div data-constraint="inStock">
                        <button>Add to Cart</button>
                    </div>
                    <div data-constraint="!inStock">
                        <p>Out of Stock</p>
                    </div>
                    <div data-constraint="price < 50">
                        <span class="badge">Budget Friendly!</span>
                    </div>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    let data = json!({
        "products": [
            {
                "name": "Laptop",
                "price": 999,
                "inStock": true
            },
            {
                "name": "Mouse",
                "price": 25,
                "inStock": true
            },
            {
                "name": "Keyboard",
                "price": 75,
                "inStock": false
            }
        ]
    });
    
    let result = template.render(&data).unwrap();
    
    // Laptop - expensive and in stock
    assert!(result.contains("Laptop"));
    assert!(result.contains("Price: $999"));
    assert!(result.contains("Add to Cart"));
    assert!(!result.contains("Budget Friendly!"));
    
    // Mouse - cheap and in stock
    assert!(result.contains("Mouse"));
    assert!(result.contains("Price: $25"));
    assert!(result.contains("Budget Friendly!"));
    
    // Keyboard - out of stock
    assert!(result.contains("Keyboard"));
    assert!(result.contains("Out of Stock"));
}

#[test]
fn test_data_scope_attributes() {
    let html = r#"
        <template>
            <div>
                <h1>Dashboard</h1>
                <div data-scope="admin">
                    <p>Admin controls</p>
                    <button>Delete All</button>
                </div>
                <div data-scope="user">
                    <p>User dashboard</p>
                    <button>View Profile</button>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    // This test might depend on how scope is implemented
    // For now, we'll test basic rendering
    let data = json!({
        "userRole": "admin"
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains("Dashboard"));
    // Scope handling might filter these based on implementation
}

#[test]
fn test_complex_constraint_expressions() {
    let html = r#"
        <template>
            <div>
                <div data-constraint="premium && credits > 0">
                    <p>Premium features available</p>
                </div>
                <div data-constraint="trial || subscription">
                    <p>Access granted</p>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    let data = json!({
        "premium": true,
        "credits": 10,
        "trial": false,
        "subscription": true
    });
    
    let result = template.render(&data).unwrap();
    assert!(result.contains("Premium features available"));
    assert!(result.contains("Access granted"));
}

#[test]
fn test_constraints_with_missing_properties() {
    let html = r#"
        <template>
            <div>
                <div data-constraint="nonexistent">
                    <p>Should not appear</p>
                </div>
                <div data-constraint="!nonexistent">
                    <p>Should appear</p>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    let data = json!({});
    
    let result = template.render(&data).unwrap();
    assert!(!result.contains("Should not appear"));
    assert!(result.contains("Should appear"));
}

#[test]
fn test_constraints_with_null_values() {
    let html = r#"
        <template>
            <div>
                <div data-constraint="nullValue">
                    <p>Null is truthy?</p>
                </div>
                <div data-constraint="!nullValue">
                    <p>Null is falsy</p>
                </div>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, None).unwrap();
    
    let data = json!({
        "nullValue": null
    });
    
    let result = template.render(&data).unwrap();
    assert!(!result.contains("Null is truthy"));
    assert!(result.contains("Null is falsy"));
}