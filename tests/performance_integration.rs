//! Integration tests for performance characteristics
//!
//! These tests verify that the template system performs adequately
//! with various data sizes and complexity levels.

use html_template::{HtmlTemplate, HtmlTemplateBuilder};
use serde_json::json;
use std::time::Instant;

#[test]
fn test_large_array_rendering_performance() {
    let html = r#"
        <template>
            <div class="container">
                <div itemprop="items[]" class="item">
                    <h3 itemprop="name"></h3>
                    <p itemprop="description"></p>
                    <span itemprop="price"></span>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div.container")).unwrap();

    // Generate a large dataset
    let mut items = Vec::new();
    for i in 0..1000 {
        items.push(json!({
            "name": format!("Item {}", i),
            "description": format!("Description for item number {}", i),
            "price": format!("${}.99", i % 100 + 10)
        }));
    }

    let data = json!({
        "items": items
    });

    let start = Instant::now();
    let result = template.render(&data).unwrap();
    let duration = start.elapsed();

    // Should complete within reasonable time (adjust threshold as needed)
    assert!(
        duration.as_millis() < 5000,
        "Large array rendering took too long: {:?}",
        duration
    );

    // Verify correctness
    assert!(result.contains("Item 0"));
    assert!(result.contains("Item 999"));
    assert_eq!(result.matches(r#"class="item""#).count(), 1000);

    println!("Rendered 1000 items in {:?}", duration);
}

#[test]
fn test_deeply_nested_structure_performance() {
    let html = r#"
        <template>
            <div class="org">
                <h1 itemprop="name"></h1>
                <div itemprop="departments[]" class="dept">
                    <h2 itemprop="name"></h2>
                    <div itemprop="teams[]" class="team">
                        <h3 itemprop="name"></h3>
                        <div itemprop="members[]" class="member">
                            <span itemprop="name"></span>
                            <span itemprop="role"></span>
                        </div>
                    </div>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div.org")).unwrap();

    // Create deeply nested structure
    let mut departments = Vec::new();
    for d in 0..5 {
        let mut teams = Vec::new();
        for t in 0..4 {
            let mut members = Vec::new();
            for m in 0..10 {
                members.push(json!({
                    "name": format!("Person {}{}_{}", d, t, m),
                    "role": format!("Role {}", m % 3)
                }));
            }
            teams.push(json!({
                "name": format!("Team {}{}", d, t),
                "members": members
            }));
        }
        departments.push(json!({
            "name": format!("Department {}", d),
            "teams": teams
        }));
    }

    let data = json!({
        "name": "Test Organization",
        "departments": departments
    });

    let start = Instant::now();
    let result = template.render(&data).unwrap();
    let duration = start.elapsed();

    // Should handle nested structure efficiently
    assert!(
        duration.as_millis() < 2000,
        "Nested structure rendering took too long: {:?}",
        duration
    );

    // Verify structure (5 depts * 4 teams * 10 members = 200 members total)
    assert_eq!(result.matches(r#"class="dept""#).count(), 5);
    assert_eq!(result.matches(r#"class="team""#).count(), 20);
    assert_eq!(result.matches(r#"class="member""#).count(), 200);

    println!("Rendered nested structure (5x4x10) in {:?}", duration);
}

#[test]
fn test_template_compilation_performance() {
    let html = r#"
        <template>
            <div class="complex">
                <h1 itemprop="title"></h1>
                <div itemprop="sections[]" class="section">
                    <h2 itemprop="heading"></h2>
                    <p itemprop="content"></p>
                    <div data-constraint='type == "special"'>
                        <span class="special-marker">Special Section</span>
                    </div>
                    <ul>
                        <li itemprop="items[]">
                            <strong>${name}</strong> - ${value}
                        </li>
                    </ul>
                </div>
                <footer>
                    <p>Total sections: ${sectionCount}</p>
                </footer>
            </div>
        </template>
    "#;

    // Test compilation time
    let start = Instant::now();
    let template = HtmlTemplate::from_str(html, Some("div.complex")).unwrap();
    let compilation_time = start.elapsed();

    // Compilation should be fast
    assert!(
        compilation_time.as_millis() < 100,
        "Template compilation took too long: {:?}",
        compilation_time
    );

    // Test that compiled template works correctly
    let data = json!({
        "title": "Performance Test",
        "sections": [
            {
                "heading": "Section 1",
                "content": "Content 1",
                "type": "normal",
                "items": [{"name": "Item A", "value": "Value A"}]
            },
            {
                "heading": "Section 2",
                "content": "Content 2",
                "type": "special",
                "items": [{"name": "Item B", "value": "Value B"}]
            }
        ],
        "sectionCount": 2
    });

    let result = template.render(&data).unwrap();
    assert!(result.contains("Performance Test"));
    assert!(result.contains("Special Section"));

    println!("Template compilation took {:?}", compilation_time);
}

#[test]
fn test_repeated_rendering_performance() {
    let html = r#"
        <template>
            <div class="item">
                <h2 itemprop="title"></h2>
                <p itemprop="description"></p>
                <span>${timestamp}</span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div.item")).unwrap();

    // Test rendering the same template multiple times
    let start = Instant::now();
    for i in 0..100 {
        let data = json!({
            "title": format!("Item {}", i),
            "description": format!("Description {}", i),
            "timestamp": format!("2024-01-{:02}", i % 30 + 1)
        });

        let result = template.render(&data).unwrap();
        assert!(result.contains(&format!("Item {}", i)));
    }
    let total_time = start.elapsed();

    // Should handle repeated rendering efficiently
    assert!(
        total_time.as_millis() < 1000,
        "100 repeated renders took too long: {:?}",
        total_time
    );

    let avg_time = total_time.as_micros() / 100;
    println!("Average render time: {}μs", avg_time);
}

#[test]
fn test_memory_usage_with_large_templates() {
    // Create a template with many different element types
    let html = r#"
        <template>
            <div class="document">
                <header>
                    <h1 itemprop="title"></h1>
                    <nav>
                        <a href="${link1}" itemprop="nav1">${nav1}</a>
                        <a href="${link2}" itemprop="nav2">${nav2}</a>
                        <a href="${link3}" itemprop="nav3">${nav3}</a>
                    </nav>
                </header>
                
                <main>
                    <article itemprop="articles[]" class="article">
                        <h2 itemprop="headline"></h2>
                        <div class="meta">
                            <time datetime="${publishDate}" itemprop="datePublished">${formattedDate}</time>
                            <span itemprop="author"></span>
                        </div>
                        <div itemprop="content"></div>
                        
                        <section itemprop="comments[]" class="comment">
                            <h4 itemprop="author"></h4>
                            <p itemprop="text"></p>
                            <time itemprop="date"></time>
                        </section>
                    </article>
                </main>
                
                <aside>
                    <div itemprop="widgets[]" class="widget">
                        <h3 itemprop="title"></h3>
                        <ul>
                            <li itemprop="items[]">${name}: ${value}</li>
                        </ul>
                    </div>
                </aside>
                
                <footer>
                    <p>${copyright}</p>
                    <div itemprop="links[]">
                        <a href="${url}" itemprop="text">${text}</a>
                    </div>
                </footer>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div.document")).unwrap();

    // Create substantial test data
    let mut articles = Vec::new();
    for i in 0..50 {
        let mut comments = Vec::new();
        for j in 0..5 {
            comments.push(json!({
                "author": format!("Commenter {}", j),
                "text": format!("Comment {} on article {}", j, i),
                "date": "2024-01-15"
            }));
        }

        articles.push(json!({
            "headline": format!("Article {}", i),
            "publishDate": "2024-01-15",
            "formattedDate": "January 15, 2024",
            "author": format!("Author {}", i % 10),
            "content": format!("Content for article {} with substantial text content that simulates real article length.", i),
            "comments": comments
        }));
    }

    let mut widgets = Vec::new();
    for i in 0..10 {
        let mut items = Vec::new();
        for j in 0..8 {
            items.push(json!({
                "name": format!("Item {}", j),
                "value": format!("Value {}", j)
            }));
        }
        widgets.push(json!({
            "title": format!("Widget {}", i),
            "items": items
        }));
    }

    let data = json!({
        "title": "Large Document Test",
        "link1": "/page1", "nav1": "Page 1",
        "link2": "/page2", "nav2": "Page 2",
        "link3": "/page3", "nav3": "Page 3",
        "articles": articles,
        "widgets": widgets,
        "copyright": "© 2024 Test Corp",
        "links": [
            {"url": "/about", "text": "About"},
            {"url": "/contact", "text": "Contact"}
        ]
    });

    let start = Instant::now();
    let result = template.render(&data).unwrap();
    let duration = start.elapsed();

    // Should handle large complex templates efficiently
    assert!(
        duration.as_millis() < 3000,
        "Large template rendering took too long: {:?}",
        duration
    );

    // Verify structure
    assert_eq!(result.matches(r#"class="article""#).count(), 50);
    assert_eq!(result.matches(r#"class="comment""#).count(), 250); // 50 articles * 5 comments
    assert_eq!(result.matches(r#"class="widget""#).count(), 10);

    // Verify content size is reasonable (not excessive memory usage)
    let result_size = result.len();
    assert!(result_size > 50000, "Result seems too small"); // Should have substantial content
    assert!(result_size < 5000000, "Result seems too large"); // But not excessive

    println!(
        "Large template rendered in {:?}, output size: {} bytes",
        duration, result_size
    );
}

#[test]
fn test_constraint_evaluation_performance() {
    let html = r#"
        <template>
            <div class="items">
                <div itemprop="items[]" class="item">
                    <h3 itemprop="name"></h3>
                    <div data-constraint="active">
                        <span class="active-badge">Active</span>
                    </div>
                    <div data-constraint="priority > 5">
                        <span class="high-priority">High Priority</span>
                    </div>
                    <div data-constraint='category == "important"'>
                        <span class="important">Important</span>
                    </div>
                    <div data-constraint="score >= 80 && active">
                        <span class="excellent">Excellent</span>
                    </div>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div.items")).unwrap();

    // Create data with many items having various constraint conditions
    let mut items = Vec::new();
    for i in 0..500 {
        items.push(json!({
            "name": format!("Item {}", i),
            "active": i % 3 == 0,
            "priority": i % 10,
            "category": if i % 4 == 0 { "important" } else { "normal" },
            "score": i % 100
        }));
    }

    let data = json!({
        "items": items
    });

    let start = Instant::now();
    let result = template.render(&data).unwrap();
    let duration = start.elapsed();

    // Constraint evaluation should be reasonably fast
    assert!(
        duration.as_millis() < 2000,
        "Constraint evaluation took too long: {:?}",
        duration
    );

    // Verify some constraints were evaluated correctly
    assert!(result.contains("Active"));
    assert!(result.contains("High Priority"));
    assert!(result.contains("Important"));

    println!("Constraint evaluation for 500 items took {:?}", duration);
}
