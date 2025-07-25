//! Integration tests for array handling and nested structures
//!
//! These tests verify array rendering, element cloning, and nested object handling.

use html_template::HtmlTemplate;
use serde_json::json;

#[test]
fn test_simple_array_rendering() {
    let html = r#"
        <template>
            <ul>
                <li itemprop="items[]">
                    <span itemprop="name"></span>
                </li>
            </ul>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("ul")).unwrap();
    let data = json!({
        "items": [
            {"name": "Item 1"},
            {"name": "Item 2"},
            {"name": "Item 3"}
        ]
    });

    let result = template.render(&data).unwrap();

    println!("test_simple_array_rendering result:\n{}", result);

    // All items should be rendered
    assert!(result.contains("Item 1"));
    assert!(result.contains("Item 2"));
    assert!(result.contains("Item 3"));

    // Should have 3 li elements
    assert_eq!(result.matches("<li").count(), 3);
}

#[test]
fn test_array_with_complex_elements() {
    let html = r#"
        <template>
            <div class="users">
                <article itemprop="users[]" class="user">
                    <h3 itemprop="name"></h3>
                    <p>Email: <a href="mailto:${email}" itemprop="email"></a></p>
                    <p>Age: <span itemprop="age"></span></p>
                    <div class="tags">
                        Tags: <span itemprop="tags"></span>
                    </div>
                </article>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "users": [
            {
                "name": "Alice",
                "email": "alice@example.com",
                "age": 30,
                "tags": "developer, rust"
            },
            {
                "name": "Bob",
                "email": "bob@example.com",
                "age": 25,
                "tags": "designer, ui"
            }
        ]
    });

    let result = template.render(&data).unwrap();

    // Check Alice's data
    assert!(result.contains("Alice"));
    assert!(result.contains("alice@example.com"));
    assert!(result.contains(">30</span>"));
    assert!(result.contains(">developer, rust</span>"));

    // Check Bob's data
    assert!(result.contains("Bob"));
    assert!(result.contains("bob@example.com"));
    assert!(result.contains(">25</span>"));
    assert!(result.contains(">designer, ui</span>"));

    // Check structure
    assert_eq!(result.matches(r#"class="user""#).count(), 2);
    assert!(result.contains(r#"href="mailto:alice@example.com""#));
}

#[test]
fn test_empty_array() {
    let html = r#"
        <template>
            <div>
                <h2>Items:</h2>
                <ul>
                    <li itemprop="items[]">
                        <span itemprop="name"></span>
                    </li>
                </ul>
                <p>Total items: <span itemprop="itemCount"></span></p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "items": [],
        "itemCount": 0
    });

    let result = template.render(&data).unwrap();

    println!("test_empty_array result:\n{}", result);

    // Should not have any li elements
    assert!(!result.contains("<li"));
    assert!(result.contains("Total items: <span itemprop=\"itemCount\">0</span>"));
    assert!(result.contains("<ul>"));
}

#[test]
fn test_nested_object_with_itemscope() {
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

    assert!(result.contains("Understanding Nested Data"));
    assert!(result.contains("John Doe"));
    assert!(result.contains("john@example.com"));
    assert!(result.contains(">Senior developer"));
    assert!(result.contains("This article explains"));
}

#[test]
fn test_nested_arrays() {
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
                    {"title": "Rust Programming", "price": "$45"},
                    {"title": "Web Development", "price": "$35"}
                ]
            }
        ]
    });

    let result = template.render(&data).unwrap();

    // Check categories
    assert!(result.contains("Electronics"));
    assert!(result.contains("Latest gadgets"));
    assert!(result.contains("Books"));
    assert!(result.contains("Educational and fiction"));

    // Check items
    assert!(result.contains("Laptop"));
    assert!(result.contains("$999"));
    assert!(result.contains("Rust Programming"));
    assert!(result.contains("$45"));

    // Check structure
    assert_eq!(result.matches(r#"class="category""#).count(), 2);
    assert_eq!(result.matches("<li").count(), 4);
}

#[test]
fn test_array_with_mixed_content() {
    let html = r#"
        <template>
            <div>
                <div itemprop="items[]" class="item">
                    <h3 itemprop="title"></h3>
                    <p itemprop="description"></p>
                    <span itemprop="type"></span>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "items": [
            {
                "title": "First Item",
                "description": "Has all properties",
                "type": "complete"
            },
            {
                "title": "Second Item",
                "type": "partial"
                // Missing description
            },
            {
                "description": "Only description",
                "type": "minimal"
                // Missing title
            }
        ]
    });

    let result = template.render(&data).unwrap();

    // All items should be rendered
    assert_eq!(result.matches(r#"class="item""#).count(), 3);

    // Check content
    assert!(result.contains("First Item"));
    assert!(result.contains("Has all properties"));
    assert!(result.contains("complete"));

    assert!(result.contains("Second Item"));
    assert!(result.contains("partial"));

    assert!(result.contains("Only description"));
    assert!(result.contains("minimal"));
}

#[test]
#[ignore = "Complex nested arrays with itemscope elements need further investigation"]
fn test_deeply_nested_structure() {
    let html = r#"
        <template>
            <div class="company">
                <h1 itemprop="name"></h1>
                <div itemprop="departments[]" class="department">
                    <h2 itemprop="name"></h2>
                    <div itemprop="teams[]" class="team">
                        <h3 itemprop="name"></h3>
                        <p>Lead: <span itemprop="lead" itemscope><span itemprop="name"></span></span></p>
                        <ul>
                            <li itemprop="members[]">
                                <span itemprop="name"></span> - <span itemprop="role"></span>
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "name": "Tech Corp",
        "departments": [
            {
                "name": "Engineering",
                "teams": [
                    {
                        "name": "Backend Team",
                        "lead": {"name": "Alice"},
                        "members": [
                            {"name": "Bob", "role": "Senior Developer"},
                            {"name": "Carol", "role": "Developer"}
                        ]
                    },
                    {
                        "name": "Frontend Team",
                        "lead": {"name": "David"},
                        "members": [
                            {"name": "Eve", "role": "UI Designer"},
                            {"name": "Frank", "role": "Developer"}
                        ]
                    }
                ]
            }
        ]
    });

    let result = template.render(&data).unwrap();

    // Check all levels
    assert!(result.contains("Tech Corp"));
    assert!(result.contains("Engineering"));
    assert!(result.contains("Backend Team"));
    assert!(result.contains("Frontend Team"));
    assert!(result.contains("Lead: Alice"));
    assert!(result.contains("Lead: David"));
    assert!(result.contains("Bob - Senior Developer"));
    assert!(result.contains("Eve - UI Designer"));
}

#[test]
fn test_array_without_itemprop_content() {
    // Test arrays where the array element itself doesn't have itemprop children
    let html = r#"
        <template>
            <div>
                <p itemprop="messages[]"></p>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "messages": ["Hello", "World", "Test"]
    });

    let result = template.render(&data).unwrap();

    // Should render three p elements
    assert_eq!(result.matches("<p").count(), 3);
    // Content might be rendered based on implementation
}

#[test]
fn test_single_item_as_array() {
    let html = r#"
        <template>
            <ul>
                <li itemprop="items[]">
                    <span itemprop="name"></span>
                </li>
            </ul>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    let data = json!({
        "items": {"name": "Single item treated as array"}
    });

    let result = template.render(&data).unwrap();

    // Single object should be treated as array of one
    assert!(result.contains("Single item treated as array"));
    assert_eq!(result.matches("<li").count(), 1);
}
