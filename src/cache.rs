//! Caching system for templates, compiled templates, and external resources
//!
//! This module provides multiple cache types to improve performance by avoiding
//! repeated parsing, compilation, and network requests.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

use crate::error::Result;
use crate::types::*;

/// Cache entry with expiration support
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
    pub access_count: usize,
    pub last_accessed: Instant,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry with optional expiration
    pub fn new(value: T, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            expires_at: ttl.map(|duration| now + duration),
            access_count: 0,
            last_accessed: now,
        }
    }

    /// Check if this entry has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Instant::now() > expires_at
        } else {
            false
        }
    }

    /// Mark this entry as accessed
    pub fn mark_accessed(&mut self) {
        self.access_count += 1;
        self.last_accessed = Instant::now();
    }

    /// Get the age of this entry
    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.created_at)
    }
}

/// Cache eviction strategy
#[derive(Debug, Clone, Copy)]
pub enum EvictionStrategy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In, First Out
    FIFO,
    /// Random eviction
    Random,
}

/// Generic cache implementation with configurable eviction
pub struct Cache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    max_size: usize,
    default_ttl: Option<Duration>,
    eviction_strategy: EvictionStrategy,
    hits: usize,
    misses: usize,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new cache with the given configuration
    pub fn new(
        max_size: usize,
        default_ttl: Option<Duration>,
        eviction_strategy: EvictionStrategy,
    ) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            default_ttl,
            eviction_strategy,
            hits: 0,
            misses: 0,
        }
    }

    /// Insert a value into the cache
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert_with_ttl(key, value, self.default_ttl)
    }

    /// Insert a value with a specific TTL
    pub fn insert_with_ttl(&mut self, key: K, value: V, ttl: Option<Duration>) -> Option<V> {
        // Remove expired entries first
        self.cleanup_expired();

        // If at capacity, evict based on strategy
        if self.entries.len() >= self.max_size && !self.entries.contains_key(&key) {
            self.evict_one();
        }

        let entry = CacheEntry::new(value, ttl);
        self.entries
            .insert(key, entry)
            .map(|old_entry| old_entry.value)
    }

    /// Get a value from the cache
    pub fn get(&mut self, key: &K) -> Option<V> {
        // Remove expired entries first
        self.cleanup_expired();

        if let Some(entry) = self.entries.get_mut(key) {
            if entry.is_expired() {
                self.entries.remove(key);
                self.misses += 1;
                None
            } else {
                entry.mark_accessed();
                self.hits += 1;
                Some(entry.value.clone())
            }
        } else {
            self.misses += 1;
            None
        }
    }

    /// Check if a key exists in the cache (without affecting access statistics)
    pub fn contains_key(&self, key: &K) -> bool {
        if let Some(entry) = self.entries.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Remove a specific key from the cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|entry| entry.value)
    }

    /// Clear all entries from the cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
            entry_count: self.entries.len(),
            max_size: self.max_size,
        }
    }

    /// Remove expired entries
    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| {
            if let Some(expires_at) = entry.expires_at {
                now <= expires_at
            } else {
                true
            }
        });
    }

    /// Evict one entry based on the eviction strategy
    fn evict_one(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let key_to_remove = match self.eviction_strategy {
            EvictionStrategy::LRU => {
                // Find least recently used
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.last_accessed)
                    .map(|(k, _)| k.clone())
            }
            EvictionStrategy::LFU => {
                // Find least frequently used
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.access_count)
                    .map(|(k, _)| k.clone())
            }
            EvictionStrategy::FIFO => {
                // Find oldest entry
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.created_at)
                    .map(|(k, _)| k.clone())
            }
            EvictionStrategy::Random => {
                // Random selection (simple approach using first key)
                self.entries.keys().next().cloned()
            }
        };

        if let Some(key) = key_to_remove {
            self.entries.remove(&key);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
    pub entry_count: usize,
    pub max_size: usize,
}

/// Template cache key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplateCacheKey {
    pub html: String,
    pub root_selector: Option<String>,
}

impl TemplateCacheKey {
    pub fn new(html: &str, root_selector: Option<&str>) -> Self {
        Self {
            html: html.to_string(),
            root_selector: root_selector.map(|s| s.to_string()),
        }
    }
}

/// External document cache key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentCacheKey {
    pub url: String,
    pub headers: Vec<(String, String)>,
}

impl DocumentCacheKey {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            headers: Vec::new(),
        }
    }

    pub fn with_headers(url: &str, headers: Vec<(String, String)>) -> Self {
        let mut sorted_headers = headers;
        sorted_headers.sort();
        Self {
            url: url.to_string(),
            headers: sorted_headers,
        }
    }
}

/// Cached external document
#[derive(Debug, Clone)]
pub struct CachedDocument {
    pub content: String,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

/// Template cache manager
pub struct TemplateCache {
    parsed_templates: Arc<RwLock<Cache<TemplateCacheKey, HtmlTemplate>>>,
    compiled_templates: Arc<RwLock<Cache<TemplateCacheKey, Arc<CompiledTemplate>>>>,
    external_documents: Arc<RwLock<Cache<DocumentCacheKey, CachedDocument>>>,
    // Note: CSS selector caching disabled due to lifetime issues with dom_query::Selection
    // css_selectors: Arc<RwLock<Cache<String, dom_query::Selection>>>,
}

impl TemplateCache {
    /// Create a new template cache with default settings
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new template cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            parsed_templates: Arc::new(RwLock::new(Cache::new(
                config.template_cache_size,
                config.template_ttl,
                config.eviction_strategy,
            ))),
            compiled_templates: Arc::new(RwLock::new(Cache::new(
                config.compiled_cache_size,
                config.compiled_ttl,
                config.eviction_strategy,
            ))),
            external_documents: Arc::new(RwLock::new(Cache::new(
                config.document_cache_size,
                config.document_ttl,
                config.eviction_strategy,
            ))),
            // css_selectors disabled
        }
    }

    /// Get or create a parsed template
    pub fn get_or_parse_template<F>(
        &self,
        key: &TemplateCacheKey,
        parser: F,
    ) -> Result<HtmlTemplate>
    where
        F: FnOnce() -> Result<HtmlTemplate>,
    {
        // Try to get from cache first
        if let Ok(mut cache) = self.parsed_templates.write() {
            if let Some(template) = cache.get(key) {
                return Ok(template);
            }
        }

        // Not in cache, parse it
        let template = parser()?;

        // Store in cache
        if let Ok(mut cache) = self.parsed_templates.write() {
            cache.insert(key.clone(), template.clone());
        }

        Ok(template)
    }

    /// Get or create a compiled template
    pub fn get_or_compile_template<F>(
        &self,
        key: &TemplateCacheKey,
        compiler: F,
    ) -> Result<Arc<CompiledTemplate>>
    where
        F: FnOnce() -> Result<Arc<CompiledTemplate>>,
    {
        // Try to get from cache first
        if let Ok(mut cache) = self.compiled_templates.write() {
            if let Some(template) = cache.get(key) {
                return Ok(template);
            }
        }

        // Not in cache, compile it
        let template = compiler()?;

        // Store in cache
        if let Ok(mut cache) = self.compiled_templates.write() {
            cache.insert(key.clone(), template.clone());
        }

        Ok(template)
    }

    /// Get or fetch an external document
    pub fn get_or_fetch_document<F>(
        &self,
        key: &DocumentCacheKey,
        fetcher: F,
    ) -> Result<CachedDocument>
    where
        F: FnOnce() -> Result<CachedDocument>,
    {
        // Try to get from cache first
        if let Ok(mut cache) = self.external_documents.write() {
            if let Some(document) = cache.get(key) {
                return Ok(document);
            }
        }

        // Not in cache, fetch it
        let document = fetcher()?;

        // Store in cache
        if let Ok(mut cache) = self.external_documents.write() {
            cache.insert(key.clone(), document.clone());
        }

        Ok(document)
    }

    // CSS selector caching methods disabled due to lifetime issues
    // TODO: Implement CSS selector caching with proper lifetime management

    /// Clear all caches
    pub fn clear_all(&self) {
        if let Ok(mut cache) = self.parsed_templates.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.compiled_templates.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.external_documents.write() {
            cache.clear();
        }
        // css_selectors disabled
    }

    /// Get comprehensive cache statistics
    pub fn get_stats(&self) -> TemplateCacheStats {
        let parsed_stats = self
            .parsed_templates
            .read()
            .map(|cache| cache.stats())
            .unwrap_or_default();
        let compiled_stats = self
            .compiled_templates
            .read()
            .map(|cache| cache.stats())
            .unwrap_or_default();
        let document_stats = self
            .external_documents
            .read()
            .map(|cache| cache.stats())
            .unwrap_or_default();
        let selector_stats = CacheStats::default(); // css_selectors disabled

        TemplateCacheStats {
            parsed_templates: parsed_stats,
            compiled_templates: compiled_stats,
            external_documents: document_stats,
            css_selectors: selector_stats,
        }
    }
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub template_cache_size: usize,
    pub compiled_cache_size: usize,
    pub document_cache_size: usize,
    pub selector_cache_size: usize,
    pub template_ttl: Option<Duration>,
    pub compiled_ttl: Option<Duration>,
    pub document_ttl: Option<Duration>,
    pub selector_ttl: Option<Duration>,
    pub eviction_strategy: EvictionStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            template_cache_size: 100,
            compiled_cache_size: 200,
            document_cache_size: 50,
            selector_cache_size: 500,
            template_ttl: Some(Duration::from_secs(3600)), // 1 hour
            compiled_ttl: Some(Duration::from_secs(7200)), // 2 hours
            document_ttl: Some(Duration::from_secs(1800)), // 30 minutes
            selector_ttl: Some(Duration::from_secs(3600)), // 1 hour
            eviction_strategy: EvictionStrategy::LRU,
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
            entry_count: 0,
            max_size: 0,
        }
    }
}

/// Comprehensive template cache statistics
#[derive(Debug, Clone)]
pub struct TemplateCacheStats {
    pub parsed_templates: CacheStats,
    pub compiled_templates: CacheStats,
    pub external_documents: CacheStats,
    pub css_selectors: CacheStats,
}

impl TemplateCacheStats {
    /// Calculate overall hit rate across all caches
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.parsed_templates.hits
            + self.compiled_templates.hits
            + self.external_documents.hits
            + self.css_selectors.hits;
        let total_requests = total_hits
            + self.parsed_templates.misses
            + self.compiled_templates.misses
            + self.external_documents.misses
            + self.css_selectors.misses;

        if total_requests > 0 {
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }

    /// Get total memory usage estimate (rough)
    pub fn total_entries(&self) -> usize {
        self.parsed_templates.entry_count
            + self.compiled_templates.entry_count
            + self.external_documents.entry_count
            + self.css_selectors.entry_count
    }
}

/// Global cache instance - thread-safe initialization
static GLOBAL_CACHE: OnceLock<TemplateCache> = OnceLock::new();

/// Initialize the global cache with default settings
pub fn init_global_cache() {
    init_global_cache_with_config(CacheConfig::default());
}

/// Initialize the global cache with custom configuration
pub fn init_global_cache_with_config(config: CacheConfig) {
    let _ = GLOBAL_CACHE.set(TemplateCache::with_config(config));
}

/// Get the global cache instance
pub fn get_global_cache() -> &'static TemplateCache {
    GLOBAL_CACHE.get_or_init(|| TemplateCache::with_config(CacheConfig::default()))
}

/// Clear the global cache
pub fn clear_global_cache() {
    get_global_cache().clear_all();
}

/// Get global cache statistics
pub fn get_global_cache_stats() -> TemplateCacheStats {
    get_global_cache().get_stats()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = Cache::new(3, None, EvictionStrategy::LRU);

        // Insert and retrieve
        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"nonexistent"), None);

        // Check statistics
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.entry_count, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = Cache::new(2, None, EvictionStrategy::LRU);

        cache.insert("key1", "value1");
        cache.insert("key2", "value2");
        cache.insert("key3", "value3"); // Should evict key1

        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.get(&"key2"), Some("value2"));
        assert_eq!(cache.get(&"key3"), Some("value3"));
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = Cache::new(10, None, EvictionStrategy::LRU);

        // Insert with very short TTL
        cache.insert_with_ttl("key1", "value1", Some(Duration::from_millis(1)));

        // Wait for expiration
        thread::sleep(Duration::from_millis(5));

        // Should be expired
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn test_cache_entry_lifecycle() {
        let entry = CacheEntry::new("test_value", Some(Duration::from_secs(60)));

        assert!(!entry.is_expired());
        assert!(entry.age() < Duration::from_secs(1));
        assert_eq!(entry.access_count, 0);
    }

    #[test]
    fn test_template_cache_key() {
        let key1 = TemplateCacheKey::new("<template>test</template>", Some("div"));
        let key2 = TemplateCacheKey::new("<template>test</template>", Some("div"));
        let key3 = TemplateCacheKey::new("<template>other</template>", Some("div"));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_document_cache_key() {
        let key1 = DocumentCacheKey::new("https://example.com/test");
        let key2 = DocumentCacheKey::with_headers(
            "https://example.com/test",
            vec![("accept".to_string(), "text/html".to_string())],
        );

        assert_ne!(key1, key2);

        let key3 = DocumentCacheKey::with_headers(
            "https://example.com/test",
            vec![("accept".to_string(), "text/html".to_string())],
        );
        assert_eq!(key2, key3);
    }

    #[test]
    fn test_template_cache_operations() {
        let cache = TemplateCache::new();
        let key = TemplateCacheKey::new("<template>test</template>", None);

        // Test that we can create cache instances
        assert!(cache.parsed_templates.read().is_ok());
        assert!(cache.compiled_templates.read().is_ok());
        assert!(cache.external_documents.read().is_ok());
        // css_selectors disabled
    }

    #[test]
    fn test_cache_config() {
        let config = CacheConfig::default();
        assert_eq!(config.template_cache_size, 100);
        assert_eq!(config.eviction_strategy as u8, EvictionStrategy::LRU as u8);

        let custom_config = CacheConfig {
            template_cache_size: 50,
            eviction_strategy: EvictionStrategy::FIFO,
            ..Default::default()
        };
        assert_eq!(custom_config.template_cache_size, 50);
    }

    #[test]
    fn test_cache_stats() {
        let stats = TemplateCacheStats {
            parsed_templates: CacheStats {
                hits: 10,
                misses: 5,
                hit_rate: 0.67,
                entry_count: 8,
                max_size: 100,
            },
            compiled_templates: CacheStats {
                hits: 20,
                misses: 10,
                hit_rate: 0.67,
                entry_count: 15,
                max_size: 200,
            },
            external_documents: CacheStats::default(),
            css_selectors: CacheStats::default(),
        };

        assert_eq!(stats.total_entries(), 23);
        assert!((stats.overall_hit_rate() - 0.67).abs() < 0.01);
    }

    #[test]
    fn test_global_cache() {
        // Test that global cache can be initialized and accessed
        init_global_cache();
        let cache = get_global_cache();

        // Clear any existing entries from other tests
        cache.clear_all();

        let stats = cache.get_stats();

        // Should have no entries after clearing
        assert_eq!(stats.total_entries(), 0);
    }
}
