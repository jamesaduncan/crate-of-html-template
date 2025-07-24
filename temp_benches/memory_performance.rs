use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use html_template::{CacheMode, HtmlTemplate, StreamingRenderer, TemplateConfig};
use serde_json::json;
use std::collections::HashMap;

fn bench_memory_template_caching(c: &mut Criterion) {
    let html = r#"
        <template>
            <article>
                <h1 itemprop="title"></h1>
                <p itemprop="content"></p>
                <div itemprop="author" itemscope>
                    <span itemprop="name"></span>
                </div>
            </article>
        </template>
    "#;

    let mut group = c.benchmark_group("memory_template_caching");

    // Test memory usage with different cache modes
    group.bench_function("no_cache_repeated_creation", |b| {
        b.iter(|| {
            let mut templates = Vec::new();
            for i in 0..100 {
                // Each template is compiled separately, no caching
                let template = HtmlTemplate::from_str_with_config(
                    black_box(html),
                    black_box(Some("article")),
                    black_box(TemplateConfig::no_caching())
                ).unwrap();
                templates.push(template);

                // Use the template to prevent optimization
                let data = json!({"title": format!("Title {}", i), "content": "Content", "author": {"name": "Author"}});
                let _result = templates[i].render(&data).unwrap();
            }
            templates
        })
    });

    group.bench_function("aggressive_cache_repeated_creation", |b| {
        b.iter(|| {
            let mut templates = Vec::new();
            for i in 0..100 {
                // Templates should be cached and reused
                let template = HtmlTemplate::from_str_with_config(
                    black_box(html),
                    black_box(Some("article")),
                    black_box(TemplateConfig::aggressive_caching())
                ).unwrap();
                templates.push(template);

                // Use the template to prevent optimization
                let data = json!({"title": format!("Title {}", i), "content": "Content", "author": {"name": "Author"}});
                let _result = templates[i].render(&data).unwrap();
            }
            templates
        })
    });

    group.finish();
}

fn bench_memory_large_dataset_rendering(c: &mut Criterion) {
    let html = r#"
        <template>
            <div class="item">
                <h2 itemprop="title"></h2>
                <p itemprop="description"></p>
                <span itemprop="metadata"></span>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div.item")).unwrap();

    let mut group = c.benchmark_group("memory_large_dataset");

    // Test memory usage with different dataset sizes
    for dataset_size in [100, 500, 1000, 2000].iter() {
        let dataset = generate_large_dataset(*dataset_size);

        group.bench_with_input(
            BenchmarkId::new("regular_rendering", dataset_size),
            &dataset,
            |b, dataset| {
                b.iter(|| {
                    let mut results = Vec::new();
                    for data in black_box(dataset) {
                        results.push(template.render(data).unwrap());
                    }
                    results // This accumulates all results in memory
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("streaming_rendering", dataset_size),
            &dataset,
            |b, dataset| {
                b.iter(|| {
                    let streaming_renderer = StreamingRenderer::new(&template).unwrap();
                    let mut stream = streaming_renderer
                        .render_iter(black_box(dataset.clone()))
                        .unwrap();

                    let mut total_size = 0;
                    while let Some(chunk) = stream.next_chunk().unwrap() {
                        total_size += chunk.len();
                        // In streaming, we process chunks immediately rather than accumulating
                        black_box(chunk); // Simulate processing without accumulating
                    }
                    total_size
                })
            },
        );
    }

    group.finish();
}

fn generate_large_dataset(size: usize) -> Vec<serde_json::Value> {
    let mut dataset = Vec::new();
    for i in 0..size {
        dataset.push(json!({
            "title": format!("Item {} - A comprehensive title for memory testing purposes", i),
            "description": format!("This is a detailed description for item number {}. It contains multiple sentences to simulate real-world content that would consume more memory during rendering operations. The description includes various details about the item to make it realistic.", i),
            "metadata": format!("Category: {}, Tags: benchmark,memory,performance,test-{}, Created: 2024-01-15, Modified: 2024-01-16", i % 10, i)
        }));
    }
    dataset
}

fn bench_memory_string_allocation_patterns(c: &mut Criterion) {
    let html_with_many_variables = r#"
        <template>
            <div class="item">
                <h1>${title} - ${subtitle} (${category}, ${date})</h1>
                <p>${description} by ${author.name} from ${author.company}</p>
                <div>Tags: ${tags.0}, ${tags.1}, ${tags.2}, ${tags.3}, ${tags.4}</div>
                <div>Meta: ${metadata.created} | ${metadata.modified} | ${metadata.version}</div>
            </div>
        </template>
    "#;

    let html_without_variables = r#"
        <template>
            <div class="item">
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
                <div itemprop="tags"></div>
                <div itemprop="metadata"></div>
            </div>
        </template>
    "#;

    let template_with_vars =
        HtmlTemplate::from_str(html_with_many_variables, Some("div.item")).unwrap();
    let template_without_vars =
        HtmlTemplate::from_str(html_without_variables, Some("div.item")).unwrap();

    let data_with_vars = json!({
        "title": "Memory Allocation Test",
        "subtitle": "String Interpolation Impact",
        "category": "Performance",
        "date": "2024-01-15",
        "description": "Testing memory allocation patterns with variable interpolation",
        "author": {
            "name": "Memory Tester",
            "company": "Performance Labs"
        },
        "tags": ["memory", "performance", "strings", "allocation", "benchmark"],
        "metadata": {
            "created": "2024-01-15T10:00:00Z",
            "modified": "2024-01-15T15:30:00Z",
            "version": "1.0.0"
        }
    });

    let data_without_vars = json!({
        "title": "Memory Allocation Test - String Interpolation Impact (Performance, 2024-01-15)",
        "description": "Testing memory allocation patterns with variable interpolation by Memory Tester from Performance Labs",
        "tags": "memory, performance, strings, allocation, benchmark",
        "metadata": "2024-01-15T10:00:00Z | 2024-01-15T15:30:00Z | 1.0.0"
    });

    let mut group = c.benchmark_group("memory_string_allocation");

    group.bench_function("with_variable_interpolation", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for _ in 0..100 {
                results.push(
                    template_with_vars
                        .render(black_box(&data_with_vars))
                        .unwrap(),
                );
            }
            results
        })
    });

    group.bench_function("without_variable_interpolation", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for _ in 0..100 {
                results.push(
                    template_without_vars
                        .render(black_box(&data_without_vars))
                        .unwrap(),
                );
            }
            results
        })
    });

    group.finish();
}

fn bench_memory_array_rendering_scale(c: &mut Criterion) {
    let html = r#"
        <template>
            <div class="container">
                <div itemprop="items[]" class="item">
                    <h3 itemprop="title"></h3>
                    <p itemprop="description"></p>
                    <div itemprop="tags[]" class="tag">
                        <span itemprop="name"></span>
                    </div>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div.container")).unwrap();

    let mut group = c.benchmark_group("memory_array_scale");

    // Test memory usage with different array sizes
    for array_size in [10, 50, 100, 200].iter() {
        let data = generate_array_data(*array_size);

        group.bench_with_input(
            BenchmarkId::new("array_size", array_size),
            &data,
            |b, data| b.iter(|| template.render(black_box(data)).unwrap()),
        );
    }

    group.finish();
}

fn generate_array_data(item_count: usize) -> serde_json::Value {
    let mut items = Vec::new();

    for i in 0..item_count {
        let mut tags = Vec::new();
        for j in 0..5 {
            tags.push(json!({"name": format!("tag-{}-{}", i, j)}));
        }

        items.push(json!({
            "title": format!("Array Item {} - Memory Test", i),
            "description": format!("This is item number {} in the array memory test. It contains sample content to measure memory usage during array rendering operations.", i),
            "tags": tags
        }));
    }

    json!({"items": items})
}

fn bench_memory_nested_structure_depth(c: &mut Criterion) {
    let html = r#"
        <template>
            <div class="root">
                <div itemprop="level1" itemscope>
                    <span itemprop="name"></span>
                    <div itemprop="level2" itemscope>
                        <span itemprop="name"></span>
                        <div itemprop="level3" itemscope>
                            <span itemprop="name"></span>
                            <div itemprop="level4" itemscope>
                                <span itemprop="name"></span>
                                <div itemprop="level5" itemscope>
                                    <span itemprop="name"></span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("div.root")).unwrap();

    let mut group = c.benchmark_group("memory_nested_depth");

    // Test memory usage with different nesting depths
    for depth in [1, 2, 3, 4, 5].iter() {
        let data = generate_deeply_nested_data(*depth);

        group.bench_with_input(BenchmarkId::new("depth", depth), &data, |b, data| {
            b.iter(|| template.render(black_box(data)).unwrap())
        });
    }

    group.finish();
}

fn generate_deeply_nested_data(depth: usize) -> serde_json::Value {
    let mut current = json!({
        "name": format!("Level {} - Deeply nested data for memory testing with longer content", depth)
    });

    for level in (1..depth).rev() {
        current = json!({
            "name": format!("Level {} - Nested structure level with detailed content for memory analysis", level),
            format!("level{}", level + 1): current
        });
    }

    json!({"level1": current})
}

fn bench_memory_template_instance_reuse(c: &mut Criterion) {
    let html = r#"
        <template>
            <article>
                <h1 itemprop="headline"></h1>
                <p itemprop="content"></p>
                <div itemprop="author" itemscope>
                    <span itemprop="name"></span>
                    <span itemprop="title"></span>
                </div>
            </article>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, Some("article")).unwrap();

    let mut group = c.benchmark_group("memory_template_reuse");

    group.bench_function("single_template_many_renders", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..100 {
                let data = json!({
                    "headline": format!("Article {}", i),
                    "content": format!("Content for article number {} in the memory reuse test.", i),
                    "author": {
                        "name": format!("Author {}", i % 10),
                        "title": "Writer"
                    }
                });
                results.push(template.render(black_box(&data)).unwrap());
            }
            results
        })
    });

    group.bench_function("many_templates_single_render", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..100 {
                // Create new template each time - should use more memory
                let new_template = HtmlTemplate::from_str(black_box(html), black_box(Some("article"))).unwrap();
                let data = json!({
                    "headline": format!("Article {}", i),
                    "content": format!("Content for article number {} in the memory reuse test.", i),
                    "author": {
                        "name": format!("Author {}", i % 10),
                        "title": "Writer"
                    }
                });
                results.push(new_template.render(black_box(&data)).unwrap());
            }
            results
        })
    });

    group.finish();
}

fn bench_memory_zero_copy_optimization(c: &mut Criterion) {
    let html = r#"
        <template>
            <div class="content">
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
                <div itemprop="metadata"></div>
                <div itemprop="tags[]" class="tag">
                    <span itemprop="name"></span>
                </div>
            </div>
        </template>
    "#;

    let template_zero_copy = HtmlTemplate::from_str_with_config(
        html,
        Some("div.content"),
        TemplateConfig::default().with_zero_copy(true),
    )
    .unwrap();

    let template_no_zero_copy = HtmlTemplate::from_str_with_config(
        html,
        Some("div.content"),
        TemplateConfig::default().with_zero_copy(false),
    )
    .unwrap();

    // Create data with reusable string references
    let title = "Zero Copy Memory Test Article";
    let description = "This article tests the memory impact of zero-copy optimizations in the html-template library.";
    let metadata = "Category: Performance, Date: 2024-01-15, Author: Memory Tester";

    let data = json!({
        "title": title,
        "description": description,
        "metadata": metadata,
        "tags": [
            {"name": "zero-copy"},
            {"name": "memory"},
            {"name": "optimization"},
            {"name": "performance"}
        ]
    });

    let mut group = c.benchmark_group("memory_zero_copy");

    group.bench_function("zero_copy_enabled", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for _ in 0..100 {
                results.push(template_zero_copy.render(black_box(&data)).unwrap());
            }
            results
        })
    });

    group.bench_function("zero_copy_disabled", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for _ in 0..100 {
                results.push(template_no_zero_copy.render(black_box(&data)).unwrap());
            }
            results
        })
    });

    group.finish();
}

fn bench_memory_cache_efficiency(c: &mut Criterion) {
    let templates_html = [
        (
            r#"<template><div itemprop="content"></div></template>"#,
            "div",
        ),
        (
            r#"<template><article><h1 itemprop="title"></h1></article></template>"#,
            "article",
        ),
        (
            r#"<template><section><p itemprop="text"></p></section></template>"#,
            "section",
        ),
        (
            r#"<template><span itemprop="label"></span></template>"#,
            "span",
        ),
        (
            r#"<template><header><h2 itemprop="heading"></h2></header></template>"#,
            "header",
        ),
    ];

    let mut group = c.benchmark_group("memory_cache_efficiency");

    group.bench_function("cache_disabled_multiple_templates", |b| {
        b.iter(|| {
            let mut templates = Vec::new();
            // Create multiple instances of the same templates without caching
            for _ in 0..20 {
                for (html, selector) in &templates_html {
                    let template = HtmlTemplate::from_str_with_config(
                        black_box(html),
                        black_box(Some(selector)),
                        black_box(TemplateConfig::no_caching())
                    ).unwrap();
                    templates.push(template);
                }
            }

            // Use all templates to prevent optimization
            let mut results = Vec::new();
            for template in &templates {
                let data = json!({"content": "test", "title": "test", "text": "test", "label": "test", "heading": "test"});
                results.push(template.render(&data).unwrap());
            }
            results
        })
    });

    group.bench_function("cache_enabled_multiple_templates", |b| {
        b.iter(|| {
            let mut templates = Vec::new();
            // Create multiple instances of the same templates with caching
            for _ in 0..20 {
                for (html, selector) in &templates_html {
                    let template = HtmlTemplate::from_str_with_config(
                        black_box(html),
                        black_box(Some(selector)),
                        black_box(TemplateConfig::aggressive_caching())
                    ).unwrap();
                    templates.push(template);
                }
            }

            // Use all templates to prevent optimization
            let mut results = Vec::new();
            for template in &templates {
                let data = json!({"content": "test", "title": "test", "text": "test", "label": "test", "heading": "test"});
                results.push(template.render(&data).unwrap());
            }
            results
        })
    });

    group.finish();
}

criterion_group!(
    memory_benches,
    bench_memory_template_caching,
    bench_memory_large_dataset_rendering,
    bench_memory_string_allocation_patterns,
    bench_memory_array_rendering_scale,
    bench_memory_nested_structure_depth,
    bench_memory_template_instance_reuse,
    bench_memory_zero_copy_optimization,
    bench_memory_cache_efficiency
);

criterion_main!(memory_benches);
