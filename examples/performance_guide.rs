//! Performance Guide for html-template
//!
//! This example demonstrates performance optimization techniques and best practices
//! for the html-template library. Run with: cargo run --example performance_guide

use html_template::{
    HtmlTemplate, HtmlTemplateBuilder, TemplateConfig, CacheMode, 
    StreamingRenderer, TemplateCache, CacheConfig
};
use serde_json::json;
use std::time::Instant;

fn main() -> html_template::Result<()> {
    println!("=== HTML Template Performance Guide ===\n");

    // 1. Template Compilation and Caching
    template_caching_performance()?;
    
    // 2. Zero-Copy Optimizations
    zero_copy_optimizations()?;
    
    // 3. Streaming for Large Datasets
    streaming_performance()?;
    
    // 4. Configuration Tuning  
    configuration_tuning()?;
    
    // 5. Memory Usage Optimization
    memory_optimization()?;
    
    // 6. Benchmarking and Profiling Tips
    benchmarking_tips()?;

    Ok(())
}

fn template_caching_performance() -> html_template::Result<()> {
    println!("1. Template Compilation and Caching");
    println!("===================================");
    
    let html = r#"
        <template>
            <div class="item">
                <h2 itemprop="title"></h2>
                <p itemprop="description"></p>
                <span class="meta">
                    <time itemprop="date"></time>
                    <span itemprop="author"></span>
                </span>
            </div>  
        </template>
    "#;
    
    // Without caching - template is compiled each time
    println!("Performance without caching:");
    let start = Instant::now();
    for i in 0..100 {
        let config = TemplateConfig::no_caching();
        let template = HtmlTemplate::from_str_with_config(html, Some("div.item"), config)?;
        let data = json!({
            "title": format!("Item {}", i),
            "description": "Performance test description",
            "date": "2024-01-15",
            "author": "Performance Tester"
        });
        let _result = template.render(&data)?;
    }
    let no_cache_duration = start.elapsed();
    
    // With aggressive caching - template compiled once and reused
    println!("Performance with aggressive caching:");
    let start = Instant::now();
    let config = TemplateConfig::aggressive_caching();
    for i in 0..100 {
        let template = HtmlTemplate::from_str_with_config(html, Some("div.item"), config.clone())?;
        let data = json!({
            "title": format!("Item {}", i),
            "description": "Performance test description", 
            "date": "2024-01-15",
            "author": "Performance Tester"
        });
        let _result = template.render(&data)?;
    }
    let cached_duration = start.elapsed();
    
    println!("Without caching: {:?}", no_cache_duration);
    println!("With caching:    {:?}", cached_duration);
    println!("Speedup:         {:.2}x", no_cache_duration.as_secs_f64() / cached_duration.as_secs_f64());
    println!();
    
    // Best Practice: Reuse template instances
    println!("Best Practice - Reuse template instances:");
    let start = Instant::now();
    let config = TemplateConfig::aggressive_caching();
    let template = HtmlTemplate::from_str_with_config(html, Some("div.item"), config)?;
    
    for i in 0..100 {
        let data = json!({
            "title": format!("Item {}", i),
            "description": "Performance test description",
            "date": "2024-01-15", 
            "author": "Performance Tester"
        });
        let _result = template.render(&data)?;
    }
    let reuse_duration = start.elapsed();
    
    println!("Template reuse:  {:?}", reuse_duration);
    println!("Additional speedup: {:.2}x", cached_duration.as_secs_f64() / reuse_duration.as_secs_f64());
    println!("\n" + &"=".repeat(60) + "\n");
    
    Ok(())
}

fn zero_copy_optimizations() -> html_template::Result<()> {
    println!("2. Zero-Copy Optimizations");
    println!("==========================");
    
    let html = r#"
        <template>
            <article>
                <h1 itemprop="headline">${title}</h1>
                <div class="content" itemprop="articleBody">${content}</div>
                <div class="metadata">
                    <span>By: <strong itemprop="author">${author}</strong></span>
                    <span>Published: <time itemprop="datePublished">${date}</time></span>
                </div>
            </article>
        </template>
    "#;
    
    // Zero-copy configuration minimizes string allocations
    let config = TemplateConfig::default()
        .with_zero_copy(true)
        .with_cache_mode(CacheMode::Aggressive);
    
    let template = HtmlTemplate::from_str_with_config(html, Some("article"), config)?;
    
    // Performance tip: Use string slices when possible
    let data = json!({
        "title": "Zero-Copy Performance Guide",
        "content": "This demonstrates efficient string handling with minimal allocations",
        "author": "Performance Expert", 
        "date": "2024-01-15"
    });
    
    let start = Instant::now();
    for _ in 0..1000 {
        let _result = template.render(&data)?;
    }
    let duration = start.elapsed();
    
    println!("Zero-copy rendering (1000 iterations): {:?}", duration);
    println!("Performance: {:.2} renders/ms", 1000.0 / duration.as_millis() as f64);
    
    println!("\nZero-copy optimization techniques:");
    println!("- Use TemplateConfig::with_zero_copy(true)");
    println!("- Reuse template instances across renders");
    println!("- Prefer string slices over owned strings in data");
    println!("- Enable aggressive caching for repeated templates");
    println!("- Use streaming for large datasets");
    println!("\n" + &"=".repeat(60) + "\n");
    
    Ok(())
}

fn streaming_performance() -> html_template::Result<()> {
    println!("3. Streaming for Large Datasets");
    println!("===============================");
    
    let html = r#"
        <template>
            <tr class="data-row">
                <td itemprop="id"></td>
                <td itemprop="name"></td>
                <td itemprop="value"></td>
                <td itemprop="status"></td>
            </tr>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(html, Some("tr.data-row"))?;
    
    // Create large dataset
    let mut large_dataset = Vec::new();
    for i in 0..10000 {
        large_dataset.push(json!({
            "id": i,
            "name": format!("Item {}", i),
            "value": format!("Value {}", i * 10),
            "status": if i % 2 == 0 { "active" } else { "inactive" }
        }));
    }
    
    println!("Dataset size: {} items", large_dataset.len());
    
    // Standard rendering - loads all into memory
    println!("Standard rendering:");
    let start = Instant::now();
    let mut all_results = Vec::new();
    for data in &large_dataset {
        all_results.push(template.render(data)?);
    }
    let standard_duration = start.elapsed();
    let standard_memory = all_results.iter().map(|s| s.len()).sum::<usize>();
    
    println!("Time: {:?}", standard_duration);
    println!("Memory: ~{} KB", standard_memory / 1024);
    
    // Streaming rendering - processes in chunks
    println!("\nStreaming rendering:");
    let start = Instant::now();
    let streaming_renderer = StreamingRenderer::new(&template)?;
    let mut stream = streaming_renderer.render_iter(large_dataset.clone())?;
    
    let mut chunk_count = 0;
    let mut total_size = 0;
    
    while let Some(chunk) = stream.next_chunk()? {
        chunk_count += 1;
        total_size += chunk.len();
        // In practice, you would write chunk to file/network here
    }
    let streaming_duration = start.elapsed();
    
    println!("Time: {:?}", streaming_duration);
    println!("Chunks processed: {}", chunk_count);
    println!("Total output: ~{} KB", total_size / 1024);
    println!("Memory efficiency: Constant memory usage vs {} KB peak", standard_memory / 1024);
    
    println!("\nWhen to use streaming:");
    println!("- Large datasets (>1000 items)");
    println!("- Memory-constrained environments");
    println!("- Real-time processing requirements");
    println!("- Network streaming scenarios");
    println!("\n" + &"=".repeat(60) + "\n");
    
    Ok(())
}

fn configuration_tuning() -> html_template::Result<()> {
    println!("4. Configuration Tuning");
    println!("=======================");
    
    let html = r#"
        <template>
            <div class="config-test">
                <h1 itemprop="title"></h1>
                <p itemprop="description"></p>
            </div>
        </template>
    "#;
    
    // Development configuration - optimized for flexibility
    let dev_config = TemplateConfig::default()
        .with_cache_mode(CacheMode::None)  // No caching for quick iteration
        .with_zero_copy(false)             // Easier debugging
        .with_compiled_template_caching(false)
        .with_external_document_caching(false);
    
    // Production configuration - optimized for performance  
    let prod_config = TemplateConfig::aggressive_caching();
    
    let data = json!({
        "title": "Configuration Test",
        "description": "Testing different configuration settings"
    });
    
    // Benchmark development config
    let start = Instant::now();
    for _ in 0..100 {
        let template = HtmlTemplate::from_str_with_config(html, Some("div"), dev_config.clone())?;
        let _result = template.render(&data)?;
    }
    let dev_duration = start.elapsed();
    
    // Benchmark production config
    let start = Instant::now();
    for _ in 0..100 {
        let template = HtmlTemplate::from_str_with_config(html, Some("div"), prod_config.clone())?;
        let _result = template.render(&data)?;
    }
    let prod_duration = start.elapsed();
    
    println!("Development config: {:?}", dev_duration);
    println!("Production config:  {:?}", prod_duration);
    println!("Production speedup: {:.2}x", dev_duration.as_secs_f64() / prod_duration.as_secs_f64());
    
    println!("\nConfiguration recommendations:");
    println!("Development:");
    println!("  - CacheMode::None for quick iteration");
    println!("  - Zero-copy disabled for easier debugging");
    println!("  - Template caching disabled");
    
    println!("\nProduction:");
    println!("  - CacheMode::Aggressive for maximum performance");
    println!("  - Zero-copy enabled");
    println!("  - All caching enabled");
    
    println!("\nTesting/CI:");
    println!("  - CacheMode::Normal for balance");
    println!("  - Zero-copy enabled");
    println!("  - Moderate caching");
    println!("\n" + &"=".repeat(60) + "\n");
    
    Ok(())
}

fn memory_optimization() -> html_template::Result<()> {
    println!("5. Memory Usage Optimization");
    println!("============================");
    
    let html = r#"
        <template>
            <div class="memory-test">
                <h1 itemprop="title"></h1>
                <div itemprop="items[]" class="item">
                    <span itemprop="name"></span>
                    <span itemprop="value"></span>
                </div>
            </div>
        </template>
    "#;
    
    println!("Memory optimization techniques:");
    
    // 1. Template reuse
    println!("\n1. Template Reuse:");
    let template = HtmlTemplate::from_str(html, Some("div.memory-test"))?;
    println!("   ✓ Create template once, reuse for multiple renders");
    println!("   ✓ Avoids repeated compilation overhead");
    
    // 2. Efficient data structures
    println!("\n2. Efficient Data Structures:");
    let data = json!({
        "title": "Memory Test",
        "items": [
            {"name": "Item 1", "value": "Value 1"},
            {"name": "Item 2", "value": "Value 2"},
            {"name": "Item 3", "value": "Value 3"}
        ]
    });
    println!("   ✓ Use serde_json::Value for flexibility");
    println!("   ✓ Consider custom structs with #[derive(Renderable)] for type safety");
    
    // 3. Streaming for large data
    println!("\n3. Streaming for Large Data:");
    println!("   ✓ Use StreamingRenderer for datasets > 1000 items");
    println!("   ✓ Process data in chunks to maintain constant memory usage");
    println!("   ✓ Configure appropriate buffer sizes");
    
    // 4. Cache management
    println!("\n4. Cache Management:");
    let cache_config = CacheConfig::default()
        .with_template_capacity(100)    // Limit template cache size
        .with_document_capacity(50);    // Limit document cache size
    
    println!("   ✓ Set appropriate cache capacities");
    println!("   ✓ Use TTL for automatic cache expiration");
    println!("   ✓ Clear caches periodically in long-running applications");
    
    // 5. String handling
    println!("\n5. String Handling:");
    println!("   ✓ Enable zero-copy optimizations");
    println!("   ✓ Use Cow<str> in custom RenderValue implementations");
    println!("   ✓ Prefer string slices over owned strings when possible");
    
    // Example render to show it works
    let _result = template.render(&data)?;
    println!("\n   Template rendered successfully with {} items", 3);
    
    println!("\nMemory profiling tools:");
    println!("- Use `cargo build --release` for production builds");
    println!("- Profile with `valgrind --tool=massif` on Linux");
    println!("- Use `heaptrack` for heap profiling");
    println!("- Monitor with `cargo-profdata` and `cargo-pgo`");
    println!("\n" + &"=".repeat(60) + "\n");
    
    Ok(())
}

fn benchmarking_tips() -> html_template::Result<()> {
    println!("6. Benchmarking and Profiling Tips");
    println!("==================================");
    
    println!("Essential benchmarking practices:");
    
    println!("\n1. Use Criterion for Benchmarks:");
    println!(r#"   Add to Cargo.toml:
   [dev-dependencies]
   criterion = "0.5"
   
   [[bench]]
   name = "template_performance"
   harness = false"#);
    
    println!("\n2. Benchmark Different Scenarios:");
    println!("   ✓ Small templates (< 10 elements)");
    println!("   ✓ Medium templates (10-100 elements)");
    println!("   ✓ Large templates (> 100 elements)");
    println!("   ✓ Simple data (flat objects)");
    println!("   ✓ Complex data (nested objects, arrays)");
    println!("   ✓ Large datasets (1K+ items)");
    
    println!("\n3. Measure Key Metrics:");
    println!("   ✓ Template compilation time");
    println!("   ✓ Rendering time per template");
    println!("   ✓ Memory usage (peak and steady-state)");
    println!("   ✓ Cache hit rates");
    println!("   ✓ Throughput (renders/second)");
    
    println!("\n4. Profiling Tools:");
    println!("   ✓ `cargo flamegraph` - CPU profiling");
    println!("   ✓ `perf record` - Linux performance analysis");
    println!("   ✓ `cargo-profdata` - LLVM profiling");
    println!("   ✓ `valgrind --tool=callgrind` - Call graph analysis");
    
    println!("\n5. Performance Testing Example:");
    println!(r#"   use criterion::{{black_box, criterion_group, criterion_main, Criterion}};
   
   fn benchmark_template_render(c: &mut Criterion) {{
       let template = HtmlTemplate::from_str(HTML, Some("div")).unwrap();
       let data = json!({{"title": "Test", "items": generate_test_data()}});
       
       c.bench_function("template_render", |b| {{
           b.iter(|| template.render(black_box(&data)))
       }});
   }}"#);
    
    println!("\n6. Optimization Workflow:");
    println!("   1. Establish baseline benchmarks");
    println!("   2. Profile to identify bottlenecks");
    println!("   3. Apply targeted optimizations");
    println!("   4. Measure and validate improvements");
    println!("   5. Repeat until performance goals are met");
    
    println!("\n7. Common Performance Anti-patterns:");
    println!("   ✗ Creating new templates for each render");
    println!("   ✗ Using owned strings unnecessarily");
    println!("   ✗ Disabling caching in production");
    println!("   ✗ Not using streaming for large datasets");
    println!("   ✗ Ignoring memory usage growth");
    
    println!("\n8. Performance Goals by Use Case:");
    println!("   Web applications: < 1ms per page render");
    println!("   Report generation: > 1000 items/second");
    println!("   Real-time systems: < 100μs per template");
    println!("   Batch processing: Memory usage < 100MB");
    
    println!("\nExample benchmark command:");
    println!("   cargo bench --bench template_performance");
    println!("\n" + &"=".repeat(60) + "\n");
    
    Ok(())
}