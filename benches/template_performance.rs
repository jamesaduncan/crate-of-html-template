use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use html_template::{HtmlTemplate, HtmlTemplateBuilder, TemplateConfig, CacheMode};

fn generate_template_html() -> String {
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
                
                <div class="related">
                    <div itemprop="relatedArticles[]" itemscope>
                        <h4 itemprop="headline"></h4>
                        <span itemprop="author"></span>
                    </div>
                </div>
            </article>
        </template>
    "#.to_string()
}

fn bench_template_compilation_no_cache(c: &mut Criterion) {
    let html = generate_template_html();
    let config = TemplateConfig::no_caching();
    
    c.bench_function("compile_template_no_cache", |b| {
        b.iter(|| {
            HtmlTemplate::from_str_with_config(
                black_box(&html),
                black_box(Some("article")),
                black_box(config.clone())
            ).unwrap()
        })
    });
}

fn bench_template_compilation_with_cache(c: &mut Criterion) {
    let html = generate_template_html();
    let config = TemplateConfig::aggressive_caching();
    
    c.bench_function("compile_template_with_cache", |b| {
        b.iter(|| {
            HtmlTemplate::from_str_with_config(
                black_box(&html),
                black_box(Some("article")),
                black_box(config.clone())
            ).unwrap()
        })
    });
}

fn bench_template_builder_simple(c: &mut Criterion) {
    let html = generate_template_html();
    
    c.bench_function("builder_simple", |b| {
        b.iter(|| {
            HtmlTemplateBuilder::new()
                .from_str(black_box(&html))
                .with_selector(black_box("article"))
                .build()
                .unwrap()
        })
    });
}

fn bench_template_builder_with_handlers(c: &mut Criterion) {
    let html = r#"
        <template>
            <form class="contact-form">
                <input type="text" name="name" itemprop="name" />
                <input type="email" name="email" itemprop="email" />
                <select name="topic" itemprop="topic">
                    <option value="general">General</option>
                    <option value="support">Support</option>
                </select>
                <textarea name="message" itemprop="message"></textarea>
                <meta name="form-id" itemprop="formId" />
            </form>
        </template>
    "#;
    
    c.bench_function("builder_with_handlers", |b| {
        b.iter(|| {
            HtmlTemplateBuilder::new()
                .from_str(black_box(html))
                .with_selector(black_box("form.contact-form"))
                .with_default_handlers()
                .build()
                .unwrap()
        })
    });
}

fn bench_template_compilation_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_scale");
    
    for element_count in [10, 50, 100, 200].iter() {
        let html = generate_scale_template(*element_count);
        
        group.bench_with_input(
            BenchmarkId::new("elements", element_count),
            &html,
            |b, html| {
                b.iter(|| {
                    HtmlTemplate::from_str(
                        black_box(html),
                        black_box(Some("div.scale-container"))
                    ).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

fn generate_scale_template(element_count: usize) -> String {
    let mut html = String::from(r#"<template><div class="scale-container">"#);
    
    for i in 0..element_count {
        html.push_str(&format!(
            r#"<div class="item-{}" itemprop="item{}">
                <span itemprop="name">Item {}</span>
                <span itemprop="value">${{value{}}}</span>
                <div itemprop="nested{}" itemscope>
                    <span itemprop="nestedName">Nested {}</span>
                </div>
            </div>"#,
            i, i, i, i, i, i
        ));
    }
    
    html.push_str("</div></template>");
    html
}

fn bench_cache_hit_vs_miss(c: &mut Criterion) {
    let html = generate_template_html();
    let config = TemplateConfig::aggressive_caching();
    
    // Pre-warm the cache
    let _template = HtmlTemplate::from_str_with_config(&html, Some("article"), config.clone()).unwrap();
    
    let mut group = c.benchmark_group("cache_hit_vs_miss");
    
    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            // This should hit the cache
            HtmlTemplate::from_str_with_config(
                black_box(&html),
                black_box(Some("article")),
                black_box(config.clone())
            ).unwrap()
        })
    });
    
    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            // Use different selector to ensure cache miss
            let selector = format!("article-{}", criterion::black_box_drop(0));
            HtmlTemplate::from_str_with_config(
                black_box(&html),
                black_box(Some(&selector)),
                black_box(config.clone())
            ).unwrap_or_else(|_| {
                // Fallback if selector doesn't match
                HtmlTemplate::from_str_with_config(&html, None, config.clone()).unwrap()
            })
        })
    });
    
    group.finish();
}

fn bench_template_reuse_vs_recreation(c: &mut Criterion) {
    let html = generate_template_html();
    let template = HtmlTemplate::from_str(&html, Some("article")).unwrap();
    
    let mut group = c.benchmark_group("reuse_vs_recreation");
    
    // Simulate data for rendering
    let data = serde_json::json!({
        "headline": "Test Article",
        "datePublished": "2024-01-15",
        "description": "Test description",
        "articleBody": "Test content",
        "author": {
            "name": "Test Author",
            "jobTitle": "Writer"
        },
        "keywords": [
            {"name": "test"},
            {"name": "benchmark"}
        ],
        "relatedArticles": [
            {"headline": "Related 1", "author": "Author 1"},
            {"headline": "Related 2", "author": "Author 2"}
        ]
    });
    
    group.bench_function("template_reuse", |b| {
        b.iter(|| {
            // Reuse the same template instance
            template.render(black_box(&data)).unwrap()
        })
    });
    
    group.bench_function("template_recreation", |b| {
        b.iter(|| {
            // Create new template each time
            let new_template = HtmlTemplate::from_str(black_box(&html), black_box(Some("article"))).unwrap();
            new_template.render(black_box(&data)).unwrap()
        })
    });
    
    group.finish();
}

fn bench_configuration_overhead(c: &mut Criterion) {
    let html = generate_template_html();
    
    let mut group = c.benchmark_group("configuration_overhead");
    
    let configs = [
        ("no_caching", TemplateConfig::no_caching()),
        ("default", TemplateConfig::default()),
        ("aggressive", TemplateConfig::aggressive_caching()),
        ("zero_copy_disabled", TemplateConfig::default().with_zero_copy(false)),
        ("zero_copy_enabled", TemplateConfig::default().with_zero_copy(true)),
    ];
    
    for (config_name, config) in configs.iter() {
        group.bench_with_input(
            BenchmarkId::new("config", config_name),
            &(html.clone(), config.clone()),
            |b, (html, config)| {
                b.iter(|| {
                    HtmlTemplate::from_str_with_config(
                        black_box(html),
                        black_box(Some("article")),
                        black_box(config.clone())
                    ).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

fn bench_template_complexity_impact(c: &mut Criterion) {
    let simple_html = r#"
        <template>
            <div itemprop="content"></div>
        </template>
    "#;
    
    let complex_html = generate_template_html();
    
    let very_complex_html = r#"
        <template>
            <div itemscope itemtype="http://schema.org/WebPage">
                <header>
                    <nav itemprop="navigation[]" itemscope>
                        <a href="#" itemprop="url"><span itemprop="name"></span></a>
                    </nav>
                </header>
                
                <main>
                    <article itemprop="mainEntity" itemscope itemtype="http://schema.org/Article">
                        <header>
                            <h1 itemprop="headline">${title} - ${subtitle}</h1>
                            <div class="meta">
                                <time itemprop="datePublished">${publishDate}</time>
                                <div itemprop="author" itemscope itemtype="http://schema.org/Person">
                                    <img itemprop="image" src="${author.avatar}" alt="${author.name}" />
                                    <span itemprop="name">${author.name}</span>
                                    <span itemprop="jobTitle">${author.title}</span>
                                    <a href="${author.url}" itemprop="url">${author.website}</a>
                                </div>
                            </div>
                        </header>
                        
                        <div class="content">
                            <div itemprop="description">${description}</div>
                            <div itemprop="articleBody">${content}</div>
                            
                            <div class="media">
                                <img itemprop="images[]" src="${url}" alt="${alt}" />
                            </div>
                        </div>
                        
                        <footer>
                            <div class="tags">
                                <span itemprop="keywords[]" class="tag" data-constraint="featured">
                                    <span itemprop="name">${name}</span>
                                    <span itemprop="count">${count}</span>
                                </span>
                            </div>
                            
                            <div class="related">
                                <div itemprop="relatedArticles[]" itemscope data-constraint="score > 0.5">
                                    <h4 itemprop="headline">${title}</h4>
                                    <span itemprop="author">${author}</span>
                                    <time itemprop="datePublished">${date}</time>
                                </div>
                            </div>
                            
                            <div class="comments">
                                <div itemprop="comments[]" itemscope itemtype="http://schema.org/Comment">
                                    <div itemprop="author" itemscope>
                                        <span itemprop="name">${author.name}</span>
                                        <img itemprop="image" src="${author.avatar}" />
                                    </div>
                                    <time itemprop="dateCreated">${createdAt}</time>
                                    <div itemprop="text">${comment}</div>
                                    <div itemprop="replies[]" itemscope data-constraint="approved">
                                        <span itemprop="author">${author}</span>
                                        <span itemprop="text">${text}</span>
                                    </div>
                                </div>
                            </div>
                        </footer>
                    </article>
                </main>
                
                <aside>
                    <div itemprop="sidebar[]" itemscope data-constraint="visible">
                        <h3 itemprop="title">${title}</h3>
                        <div itemprop="content">${content}</div>
                    </div>
                </aside>
            </div>
        </template>
    "#;
    
    let mut group = c.benchmark_group("template_complexity");
    
    group.bench_function("simple", |b| {
        b.iter(|| {
            HtmlTemplate::from_str(black_box(simple_html), black_box(Some("div"))).unwrap()
        })
    });
    
    group.bench_function("complex", |b| {
        b.iter(|| {
            HtmlTemplate::from_str(black_box(&complex_html), black_box(Some("article"))).unwrap()
        })
    });
    
    group.bench_function("very_complex", |b| {
        b.iter(|| {
            HtmlTemplate::from_str(black_box(very_complex_html), black_box(Some("div"))).unwrap()
        })
    });
    
    group.finish();
}

criterion_group!(
    template_benches,
    bench_template_compilation_no_cache,
    bench_template_compilation_with_cache,
    bench_template_builder_simple,
    bench_template_builder_with_handlers,
    bench_template_compilation_scale,
    bench_cache_hit_vs_miss,
    bench_template_reuse_vs_recreation,
    bench_configuration_overhead,
    bench_template_complexity_impact
);

criterion_main!(template_benches);