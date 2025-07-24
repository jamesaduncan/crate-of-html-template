use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use html_template::{HtmlTemplate, HtmlTemplateBuilder, StreamingRenderer, TemplateConfig};
use serde_json::json;

fn create_simple_template() -> HtmlTemplate {
    let html = r#"
        <template>
            <div class="item">
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
                <span itemprop="author"></span>
            </div>
        </template>
    "#;
    
    HtmlTemplate::from_str(html, Some("div.item")).unwrap()
}

fn create_complex_template() -> HtmlTemplate {
    let html = r#"
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
            </article>
        </template>
    "#;
    
    HtmlTemplate::from_str(html, Some("article")).unwrap()
}

fn create_form_template_with_handlers() -> HtmlTemplate {
    let html = r#"
        <template>
            <form class="contact-form">
                <input type="text" name="name" itemprop="name" />
                <input type="email" name="email" itemprop="email" />
                <select name="country" itemprop="country">
                    <option value="us">United States</option>
                    <option value="uk">United Kingdom</option>
                    <option value="ca">Canada</option>
                    <option value="de">Germany</option>
                </select>
                <textarea name="message" itemprop="message"></textarea>
                <input type="checkbox" name="newsletter" itemprop="newsletter" />
                <meta name="form-id" itemprop="formId" />
            </form>
        </template>
    "#;
    
    HtmlTemplateBuilder::new()
        .from_str(html)
        .with_selector("form.contact-form")
        .with_default_handlers()
        .build()
        .unwrap()
}

fn generate_simple_data() -> serde_json::Value {
    json!({
        "title": "Simple Test Article",
        "description": "This is a simple test description for benchmarking purposes.",
        "author": "Benchmark Author"
    })
}

fn generate_complex_data() -> serde_json::Value {
    json!({
        "headline": "Complex Benchmark Article",
        "datePublished": "2024-01-15T10:30:00Z",
        "description": "This is a comprehensive benchmark article with nested data structures.",
        "articleBody": "This article contains extensive content for benchmarking rendering performance. It includes multiple paragraphs, complex nested structures, and various data types to simulate real-world usage scenarios.",
        "author": {
            "name": "Complex Author",
            "jobTitle": "Senior Benchmark Engineer",
            "url": "https://example.com/author/complex-author"
        },
        "keywords": [
            {"name": "benchmark"},
            {"name": "performance"},
            {"name": "rust"},
            {"name": "html-template"},
            {"name": "microdata"}
        ],
        "comments": [
            {
                "author": {"name": "Commenter One"},
                "dateCreated": "2024-01-16T09:15:00Z",
                "text": "Great article! Very informative and well-written."
            },
            {
                "author": {"name": "Commenter Two"},
                "dateCreated": "2024-01-16T14:22:00Z",
                "text": "I learned a lot from this. Thanks for sharing these insights."
            },
            {
                "author": {"name": "Commenter Three"},
                "dateCreated": "2024-01-17T08:45:00Z",
                "text": "The performance benchmarks are particularly interesting. Would love to see more analysis."
            }
        ]
    })
}

fn generate_form_data() -> serde_json::Value {
    json!({
        "name": "John Doe",
        "email": "john.doe@example.com",
        "country": "uk",
        "message": "This is a test message for the contact form benchmark.",
        "newsletter": "checked",
        "formId": "contact-form-2024-001"
    })
}

fn bench_simple_rendering(c: &mut Criterion) {
    let template = create_simple_template();
    let data = generate_simple_data();
    
    c.bench_function("render_simple", |b| {
        b.iter(|| {
            template.render(black_box(&data)).unwrap()
        })
    });
}

fn bench_complex_rendering(c: &mut Criterion) {
    let template = create_complex_template();
    let data = generate_complex_data();
    
    c.bench_function("render_complex", |b| {
        b.iter(|| {
            template.render(black_box(&data)).unwrap()
        })
    });
}

fn bench_form_rendering_with_handlers(c: &mut Criterion) {
    let template = create_form_template_with_handlers();
    let data = generate_form_data();
    
    c.bench_function("render_form_with_handlers", |b| {
        b.iter(|| {
            template.render(black_box(&data)).unwrap()
        })
    });
}

fn bench_rendering_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering_scale");
    
    let template = create_complex_template();
    
    // Test with different amounts of array data
    for item_count in [1, 10, 50, 100].iter() {
        let data = generate_scaled_data(*item_count);
        
        group.bench_with_input(
            BenchmarkId::new("items", item_count),
            &data,
            |b, data| {
                b.iter(|| {
                    template.render(black_box(data)).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

fn generate_scaled_data(item_count: usize) -> serde_json::Value {
    let mut keywords = Vec::new();
    let mut comments = Vec::new();
    
    for i in 0..item_count {
        keywords.push(json!({"name": format!("keyword-{}", i)}));
        comments.push(json!({
            "author": {"name": format!("Commenter {}", i)},
            "dateCreated": "2024-01-15T10:00:00Z",
            "text": format!("This is comment number {} for benchmarking purposes.", i)
        }));
    }
    
    json!({
        "headline": "Scaled Benchmark Article",
        "datePublished": "2024-01-15T10:30:00Z",
        "description": "Article with scaled data for performance testing.",
        "articleBody": "This article tests rendering performance with varying amounts of array data.",
        "author": {
            "name": "Scale Test Author",
            "jobTitle": "Performance Engineer",
            "url": "https://example.com/author/scale-test"
        },
        "keywords": keywords,
        "comments": comments
    })
}

fn bench_streaming_vs_regular_rendering(c: &mut Criterion) {
    let template = create_simple_template();
    
    // Generate large dataset
    let mut large_dataset = Vec::new();
    for i in 0..1000 {
        large_dataset.push(json!({
            "title": format!("Article {}", i),
            "description": format!("Description for article number {} in the benchmark dataset.", i),
            "author": format!("Author {}", i % 10)
        }));
    }
    
    let mut group = c.benchmark_group("streaming_vs_regular");
    
    group.bench_function("regular_rendering", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for data in black_box(&large_dataset) {
                results.push(template.render(data).unwrap());
            }
            results
        })
    });
    
    group.bench_function("streaming_rendering", |b| {
        b.iter(|| {
            let streaming_renderer = StreamingRenderer::new(&template).unwrap();
            let mut stream = streaming_renderer.render_iter(black_box(large_dataset.clone())).unwrap();
            
            let mut results = Vec::new();
            while let Some(chunk) = stream.next_chunk().unwrap() {
                results.push(chunk);
            }
            results
        })
    });
    
    group.finish();
}

fn bench_variable_interpolation_impact(c: &mut Criterion) {
    let html_with_vars = r#"
        <template>
            <div class="article">
                <h1 itemprop="title">${title} - ${subtitle}</h1>
                <p itemprop="description">${description} by ${author.name}</p>
                <div itemprop="meta">Published on ${publishDate} in ${category}</div>
                <div itemprop="content">${content}</div>
            </div>
        </template>
    "#;
    
    let html_without_vars = r#"
        <template>
            <div class="article">
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
                <div itemprop="meta"></div>
                <div itemprop="content"></div>
            </div>
        </template>
    "#;
    
    let template_with_vars = HtmlTemplate::from_str(html_with_vars, Some("div.article")).unwrap();
    let template_without_vars = HtmlTemplate::from_str(html_without_vars, Some("div.article")).unwrap();
    
    let data = json!({
        "title": "Variable Interpolation Test",
        "subtitle": "Performance Impact",
        "description": "Testing the performance impact of variable interpolation",
        "author": {"name": "Test Author"},
        "publishDate": "2024-01-15",
        "category": "Benchmarks",
        "content": "This is the article content for variable interpolation testing."
    });
    
    let mut group = c.benchmark_group("variable_interpolation");
    
    group.bench_function("with_variables", |b| {
        b.iter(|| {
            template_with_vars.render(black_box(&data)).unwrap()
        })
    });
    
    group.bench_function("without_variables", |b| {
        b.iter(|| {
            template_without_vars.render(black_box(&data)).unwrap()
        })
    });
    
    group.finish();
}

fn bench_constraint_evaluation_impact(c: &mut Criterion) {
    let html_with_constraints = r#"
        <template>
            <div class="article">
                <div data-constraint="featured" itemprop="featuredBadge">Featured</div>
                <div data-constraint="author.verified" itemprop="verifiedBadge">Verified Author</div>
                <div data-constraint="publishDate > '2024-01-01'" itemprop="recentBadge">Recent</div>
                <div data-constraint="tags.length > 0" itemprop="taggedBadge">Tagged</div>
                <h1 itemprop="title"></h1>
                <p itemprop="content"></p>
            </div>
        </template>
    "#;
    
    let html_without_constraints = r#"
        <template>
            <div class="article">
                <div itemprop="featuredBadge">Featured</div>
                <div itemprop="verifiedBadge">Verified Author</div>
                <div itemprop="recentBadge">Recent</div>
                <div itemprop="taggedBadge">Tagged</div>
                <h1 itemprop="title"></h1>
                <p itemprop="content"></p>
            </div>
        </template>
    "#;
    
    let template_with_constraints = HtmlTemplate::from_str(html_with_constraints, Some("div.article")).unwrap();
    let template_without_constraints = HtmlTemplate::from_str(html_without_constraints, Some("div.article")).unwrap();
    
    let data = json!({
        "title": "Constraint Evaluation Test",
        "content": "Testing the performance impact of constraint evaluation during rendering.",
        "featured": true,
        "author": {"verified": true},
        "publishDate": "2024-02-01",
        "tags": ["benchmark", "performance"],
        "featuredBadge": "Featured Content",
        "verifiedBadge": "Verified Author Content",
        "recentBadge": "Recent Content",
        "taggedBadge": "Tagged Content"
    });
    
    let mut group = c.benchmark_group("constraint_evaluation");
    
    group.bench_function("with_constraints", |b| {
        b.iter(|| {
            template_with_constraints.render(black_box(&data)).unwrap()
        })
    });
    
    group.bench_function("without_constraints", |b| {
        b.iter(|| {
            template_without_constraints.render(black_box(&data)).unwrap()
        })
    });
    
    group.finish();
}

fn bench_nested_data_depth_impact(c: &mut Criterion) {
    let html = r#"
        <template>
            <div class="nested-test">
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
    
    let template = HtmlTemplate::from_str(html, Some("div.nested-test")).unwrap();
    
    let mut group = c.benchmark_group("nested_depth");
    
    for depth in [1, 2, 3, 4, 5].iter() {
        let data = generate_nested_data(*depth);
        
        group.bench_with_input(
            BenchmarkId::new("depth", depth),
            &data,
            |b, data| {
                b.iter(|| {
                    template.render(black_box(data)).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

fn generate_nested_data(depth: usize) -> serde_json::Value {
    let mut current = json!({"name": format!("Level {}", depth)});
    
    for level in (1..depth).rev() {
        current = json!({
            "name": format!("Level {}", level),
            format!("level{}", level + 1): current
        });
    }
    
    json!({"level1": current})
}

fn bench_zero_copy_impact(c: &mut Criterion) {
    let html = generate_complex_data_template();
    let data = generate_complex_data();
    
    let template_zero_copy = HtmlTemplate::from_str_with_config(
        &html,
        Some("article"),
        TemplateConfig::default().with_zero_copy(true)
    ).unwrap();
    
    let template_no_zero_copy = HtmlTemplate::from_str_with_config(
        &html,
        Some("article"),
        TemplateConfig::default().with_zero_copy(false)
    ).unwrap();
    
    let mut group = c.benchmark_group("zero_copy_impact");
    
    group.bench_function("zero_copy_enabled", |b| {
        b.iter(|| {
            template_zero_copy.render(black_box(&data)).unwrap()
        })
    });
    
    group.bench_function("zero_copy_disabled", |b| {
        b.iter(|| {
            template_no_zero_copy.render(black_box(&data)).unwrap()
        })
    });
    
    group.finish();
}

fn generate_complex_data_template() -> String {
    r#"
        <template>
            <article>
                <h1 itemprop="headline"></h1>
                <p itemprop="description"></p>
                <div itemprop="author" itemscope>
                    <span itemprop="name"></span>
                    <span itemprop="jobTitle"></span>
                </div>
                <div itemprop="keywords[]" class="tag">
                    <span itemprop="name"></span>
                </div>
            </article>
        </template>
    "#.to_string()
}

criterion_group!(
    rendering_benches,
    bench_simple_rendering,
    bench_complex_rendering,
    bench_form_rendering_with_handlers,
    bench_rendering_scale,
    bench_streaming_vs_regular_rendering,
    bench_variable_interpolation_impact,
    bench_constraint_evaluation_impact,
    bench_nested_data_depth_impact,
    bench_zero_copy_impact
);

criterion_main!(rendering_benches);