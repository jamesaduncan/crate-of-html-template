use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use html_template::{HtmlTemplate, TemplateConfig, CacheMode};

fn generate_simple_html() -> String {
    r#"
        <template>
            <div class="item">
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
                <span itemprop="author"></span>
                <time itemprop="date"></time>
            </div>
        </template>
    "#.to_string()
}

fn generate_complex_html() -> String {
    r#"
        <template>
            <article itemscope itemtype="http://schema.org/Article">
                <header>
                    <h1 itemprop="headline"></h1>
                    <div class="meta">
                        <time itemprop="datePublished"></time>
                        <div itemprop="author" itemscope itemtype="http://schema.org/Person">
                            <span itemprop="name"></span>
                            <span itemprop="jobTitle"></span>
                            <a href="#" itemprop="url"></a>
                        </div>
                    </div>
                </header>
                
                <div class="content">
                    <p itemprop="description"></p>
                    <div itemprop="articleBody"></div>
                </div>
                
                <div class="tags">
                    <span itemprop="keywords[]" class="tag">
                        <span itemprop="name"></span>
                    </span>
                </div>
                
                <div class="comments">
                    <div itemprop="comments[]" itemscope itemtype="http://schema.org/Comment">
                        <div itemprop="author" itemscope>
                            <span itemprop="name"></span>
                        </div>
                        <time itemprop="dateCreated"></time>
                        <p itemprop="text"></p>
                    </div>
                </div>
                
                <div data-constraint="featured" class="featured-badge">
                    <span>Featured Article</span>
                </div>
                
                <div data-constraint="publishedDate > 2024-01-01" class="recent-badge">
                    <span>Recent</span>
                </div>
            </article>
        </template>
    "#.to_string()
}

fn bench_template_parsing_simple(c: &mut Criterion) {
    let html = generate_simple_html();
    let config = TemplateConfig::no_caching();
    let selector = "div";
    
    c.bench_function("parse_simple_template", |b| {
        b.iter(|| {
            HtmlTemplate::from_str_with_config(
                black_box(&html),
                black_box(Some(selector)),
                black_box(config.clone())
            ).unwrap()
        })
    });
}

fn bench_template_parsing_complex(c: &mut Criterion) {
    let html = generate_complex_html();
    let config = TemplateConfig::no_caching();
    let selector = "article";
    
    c.bench_function("parse_complex_template", |b| {
        b.iter(|| {
            HtmlTemplate::from_str_with_config(
                black_box(&html),
                black_box(Some(selector)),
                black_box(config.clone())
            ).unwrap()
        })
    });
}

fn bench_parsing_with_cache_modes(c: &mut Criterion) {
    let html = generate_complex_html();
    let selector = "article";
    
    let mut group = c.benchmark_group("parsing_cache_modes");
    
    let configs = vec![
        ("no_cache", TemplateConfig::no_caching()),
        ("normal_cache", TemplateConfig::default()),
        ("aggressive_cache", TemplateConfig::aggressive_caching()),
    ];
    
    for (cache_name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("cache_mode", cache_name),
            &(html.clone(), config),
            |b, (html, config)| {
                b.iter(|| {
                    HtmlTemplate::from_str_with_config(
                        black_box(html),
                        black_box(Some(selector)),
                        black_box(config.clone())
                    ).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

fn bench_parsing_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_scale");
    let selector = "div";
    
    // Test parsing performance with different template sizes
    for size in vec![10, 50, 100, 200] {
        let html = generate_scale_html(size);
        
        group.bench_with_input(
            BenchmarkId::new("elements", size),
            &html,
            |b, html| {
                b.iter(|| {
                    HtmlTemplate::from_str(
                        black_box(html),
                        black_box(Some(selector))
                    ).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

fn generate_scale_html(element_count: usize) -> String {
    let mut html = String::from(r#"<template><div class="scale-container">"#);
    
    for i in 0..element_count {
        html.push_str(&format!(
            r#"<div class="item-{}" itemprop="item{}">
                <span itemprop="name">Item {}</span>
                <span itemprop="value">{}</span>
            </div>"#,
            i, i, i, i * 10
        ));
    }
    
    html.push_str("</div></template>");
    html
}

fn bench_parsing_with_variables(c: &mut Criterion) {
    let html_with_vars = r#"
        <template>
            <div class="variable-test">
                <h1 itemprop="title">${title} - ${subtitle}</h1>
                <p itemprop="description">${description} by ${author.name}</p>
                <div itemprop="metadata">
                    Published on ${publishDate} in ${category.name}
                    Read time: ${readTime} minutes
                </div>
                <div itemprop="tags[]" class="tag">
                    <span>${name} (${count} posts)</span>
                </div>
            </div>
        </template>
    "#;
    
    let html_without_vars = r#"
        <template>
            <div class="no-variable-test">
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
                <div itemprop="metadata"></div>
                <div itemprop="tags[]" class="tag">
                    <span itemprop="name"></span>
                </div>
            </div>
        </template>
    "#;
    
    let mut group = c.benchmark_group("parsing_variables");
    let selector = "div";
    
    group.bench_function("with_variables", |b| {
        b.iter(|| {
            HtmlTemplate::from_str(
                black_box(html_with_vars),
                black_box(Some(selector))
            ).unwrap()
        })
    });
    
    group.bench_function("without_variables", |b| {
        b.iter(|| {
            HtmlTemplate::from_str(
                black_box(html_without_vars),
                black_box(Some(selector))
            ).unwrap()
        })
    });
    
    group.finish();
}

fn bench_parsing_with_constraints(c: &mut Criterion) {
    let html_with_constraints = r#"
        <template>
            <div class="constraint-test">
                <div data-constraint="featured" itemprop="featuredContent"></div>
                <div data-constraint="author.isVerified" itemprop="verifiedContent"></div>
                <div data-constraint="publishDate > 2024-01-01" itemprop="recentContent"></div>
                <div data-constraint="tags.length > 0" itemprop="taggedContent"></div>
                <div data-constraint="category == tech" itemprop="techContent"></div>
            </div>
        </template>
    "#;
    
    let html_without_constraints = r#"
        <template>
            <div class="no-constraint-test">
                <div itemprop="featuredContent"></div>
                <div itemprop="verifiedContent"></div>
                <div itemprop="recentContent"></div>
                <div itemprop="taggedContent"></div>
                <div itemprop="techContent"></div>
            </div>
        </template>
    "#;
    
    let mut group = c.benchmark_group("parsing_constraints");
    let selector = "div";
    
    group.bench_function("with_constraints", |b| {
        b.iter(|| {
            HtmlTemplate::from_str(
                black_box(html_with_constraints),
                black_box(Some(selector))
            ).unwrap()
        })
    });
    
    group.bench_function("without_constraints", |b| {
        b.iter(|| {
            HtmlTemplate::from_str(
                black_box(html_without_constraints),
                black_box(Some(selector))
            ).unwrap()
        })
    });
    
    group.finish();
}

criterion_group!(
    parsing_benches,
    bench_template_parsing_simple,
    bench_template_parsing_complex,
    bench_parsing_with_cache_modes,
    bench_parsing_scale,
    bench_parsing_with_variables,
    bench_parsing_with_constraints
);

criterion_main!(parsing_benches);