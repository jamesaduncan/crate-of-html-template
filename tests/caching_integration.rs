use html_template::*;
use serde_json::json;

#[test]
fn test_template_caching_integration() {
    // Initialize global cache
    cache::init_global_cache();
    
    // Create a template with caching enabled
    let html = r#"
        <template>
            <div>
                <h1 itemprop="title">${title}</h1>
                <p itemprop="description">${description}</p>
            </div>
        </template>
    "#;
    
    let config = TemplateConfig::new().with_cache_mode(CacheMode::Normal);
    
    // Create template first time - should compile and cache
    let template1 = HtmlTemplate::from_str_with_config(html, Some("div"), config.clone()).unwrap();
    
    // Create same template again - should come from cache
    let template2 = HtmlTemplate::from_str_with_config(html, Some("div"), config.clone()).unwrap();
    
    // Test that both templates work correctly
    let data = json!({
        "title": "Test Title",
        "description": "Test Description"
    });
    
    let result1 = template1.render(&data).unwrap();
    let result2 = template2.render(&data).unwrap();
    
    assert!(result1.contains("Test Title"));
    assert!(result1.contains("Test Description"));
    assert_eq!(result1, result2);
    
    // Check cache statistics
    let stats = cache::get_global_cache_stats();
    assert!(stats.compiled_templates.entry_count > 0);
    
    // Test no-cache mode
    let config_no_cache = TemplateConfig::no_caching();
    let template3 = HtmlTemplate::from_str_with_config(html, Some("div"), config_no_cache).unwrap();
    let result3 = template3.render(&data).unwrap();
    
    assert_eq!(result1, result3);
}

#[test]
fn test_cache_eviction_and_statistics() {
    // Create a custom cache with small size for testing eviction
    let cache_config = CacheConfig {
        compiled_cache_size: 2,
        ..Default::default()
    };
    
    let cache = TemplateCache::with_config(cache_config);
    
    let html1 = "<template><div itemprop=\"test\">Template 1</div></template>";
    let html2 = "<template><div itemprop=\"test\">Template 2</div></template>";
    let html3 = "<template><div itemprop=\"test\">Template 3</div></template>";
    
    let config = TemplateConfig::new().with_cache_mode(CacheMode::Normal);
    
    // Create templates that should trigger eviction
    let _template1 = HtmlTemplate::from_str_with_cache(html1, None, config.clone(), &cache).unwrap();
    let _template2 = HtmlTemplate::from_str_with_cache(html2, None, config.clone(), &cache).unwrap();
    let _template3 = HtmlTemplate::from_str_with_cache(html3, None, config.clone(), &cache).unwrap(); // Should evict template1
    
    // Check that cache size is respected
    let stats = cache.get_stats();
    assert!(stats.compiled_templates.entry_count <= 2);
}

#[test]
fn test_cache_configuration_options() {
    // Test different cache configurations
    let aggressive = TemplateConfig::aggressive_caching();
    assert_eq!(aggressive.cache_mode(), CacheMode::Aggressive);
    assert!(aggressive.cache_compiled_templates());
    assert!(aggressive.cache_external_documents());
    assert!(aggressive.zero_copy());
    
    let no_cache = TemplateConfig::no_caching();
    assert_eq!(no_cache.cache_mode(), CacheMode::None);
    assert!(!no_cache.cache_compiled_templates());
    assert!(!no_cache.cache_external_documents());
    assert!(no_cache.zero_copy()); // Zero-copy should still be enabled
    
    let default = TemplateConfig::default();
    assert_eq!(default.cache_mode(), CacheMode::Normal);
    assert!(default.cache_compiled_templates());
    assert!(default.cache_external_documents());
    assert!(default.zero_copy());
}

#[test]
fn test_cache_hit_miss_statistics() {
    let cache = TemplateCache::new();
    
    let html = "<template><div itemprop=\"data\">${value}</div></template>";
    let key = cache::TemplateCacheKey::new(html, None);
    
    // First access should be a miss
    let _template1 = cache.get_or_compile_template(&key, || {
        crate::compiler::Compiler::compile(html, None)
    }).unwrap();
    
    // Second access should be a hit
    let _template2 = cache.get_or_compile_template(&key, || {
        panic!("Should not compile again - should come from cache")
    }).unwrap();
    
    let stats = cache.get_stats();
    assert!(stats.compiled_templates.hits > 0);
    assert!(stats.compiled_templates.hit_rate > 0.0);
}