use criterion::{black_box, criterion_group, criterion_main, Criterion};
use html_template::{HtmlTemplate, TemplateConfig};
use serde_json::json;

fn bench_simple_template_creation(c: &mut Criterion) {
    let html = "<template><div itemprop=\"content\"></div></template>";
    
    c.bench_function("simple_template_creation", |b| {
        b.iter(|| {
            HtmlTemplate::from_str(black_box(html), black_box(Some("div"))).unwrap()
        })
    });
}

fn bench_simple_template_rendering(c: &mut Criterion) {
    let html = "<template><div itemprop=\"content\"></div></template>";
    let template = HtmlTemplate::from_str(html, Some("div")).unwrap();
    let data = json!({"content": "Test content"});
    
    c.bench_function("simple_template_rendering", |b| {
        b.iter(|| {
            template.render(black_box(&data)).unwrap()
        })
    });
}

fn bench_template_with_caching(c: &mut Criterion) {
    let html = "<template><div itemprop=\"content\"></div></template>";
    let config_no_cache = TemplateConfig::no_caching();
    let config_with_cache = TemplateConfig::aggressive_caching();
    
    c.bench_function("template_no_cache", |b| {
        b.iter(|| {
            HtmlTemplate::from_str_with_config(
                black_box(html),
                black_box(Some("div")),
                black_box(config_no_cache.clone())
            ).unwrap()
        })
    });
    
    c.bench_function("template_with_cache", |b| {
        b.iter(|| {
            HtmlTemplate::from_str_with_config(
                black_box(html),
                black_box(Some("div")),
                black_box(config_with_cache.clone())
            ).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_simple_template_creation,
    bench_simple_template_rendering,
    bench_template_with_caching
);

criterion_main!(benches);