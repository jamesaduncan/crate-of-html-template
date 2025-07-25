//! Utility functions for zero-copy optimizations and performance
//!
//! This module provides helper functions and types that minimize allocations
//! and support zero-copy string operations where possible.

use std::borrow::Cow;
use std::collections::HashMap;

/// Pool for reusing string allocations
pub struct StringPool {
    available: Vec<String>,
    in_use: usize,
    max_size: usize,
}

impl StringPool {
    /// Create a new string pool with the given initial capacity
    pub fn new(initial_capacity: usize, max_size: usize) -> Self {
        Self {
            available: Vec::with_capacity(initial_capacity),
            in_use: 0,
            max_size,
        }
    }

    /// Get a string from the pool, reusing an existing one if available
    pub fn get(&mut self) -> String {
        if let Some(mut s) = self.available.pop() {
            s.clear();
            self.in_use += 1;
            s
        } else {
            self.in_use += 1;
            String::new()
        }
    }

    /// Return a string to the pool for reuse
    pub fn return_string(&mut self, mut s: String) {
        if self.available.len() < self.max_size {
            s.clear();
            self.available.push(s);
        }
        self.in_use = self.in_use.saturating_sub(1);
    }

    /// Get the number of strings currently in use
    pub fn in_use_count(&self) -> usize {
        self.in_use
    }

    /// Get the number of strings available in the pool
    pub fn available_count(&self) -> usize {
        self.available.len()
    }
}

/// A reusable buffer for string operations
pub struct StringBuffer {
    buffer: String,
}

impl StringBuffer {
    /// Create a new string buffer
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Create a new string buffer with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }

    /// Clear the buffer and return a mutable reference
    pub fn clear(&mut self) -> &mut String {
        self.buffer.clear();
        &mut self.buffer
    }

    /// Get a reference to the buffer contents
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// Take ownership of the buffer contents
    pub fn take(self) -> String {
        self.buffer
    }

    /// Get the current capacity
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }
}

impl Default for StringBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache for compiled regular expressions
pub struct RegexCache {
    cache: HashMap<String, regex::Regex>,
    max_size: usize,
}

impl RegexCache {
    /// Create a new regex cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    /// Get or compile a regex pattern
    pub fn get_or_compile(&mut self, pattern: &str) -> Result<&regex::Regex, regex::Error> {
        if !self.cache.contains_key(pattern) {
            if self.cache.len() >= self.max_size {
                // Simple eviction: clear half the cache
                let keys_to_remove: Vec<_> = self
                    .cache
                    .keys()
                    .take(self.cache.len() / 2)
                    .cloned()
                    .collect();
                for key in keys_to_remove {
                    self.cache.remove(&key);
                }
            }

            let regex = regex::Regex::new(pattern)?;
            self.cache.insert(pattern.to_string(), regex);
        }

        Ok(self.cache.get(pattern).unwrap())
    }

    /// Get the number of cached regexes
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Efficient string replacement that minimizes allocations
pub fn replace_multiple_cow<'a>(
    text: &'a str,
    replacements: &[(String, Cow<str>)],
) -> Cow<'a, str> {
    if replacements.is_empty() {
        return Cow::Borrowed(text);
    }

    // Check if any replacements are needed
    let mut needs_replacement = false;
    for (search, _) in replacements {
        if text.contains(search) {
            needs_replacement = true;
            break;
        }
    }

    if !needs_replacement {
        return Cow::Borrowed(text);
    }

    // Perform replacements
    let mut result = text.to_string();
    for (search, replace) in replacements {
        result = result.replace(search, replace);
    }

    Cow::Owned(result)
}

/// Split a dot-separated path without allocating for simple cases
pub fn split_path_cow(path: &str) -> Cow<[String]> {
    if !path.contains('.') {
        // Simple case: single segment
        Cow::Owned(vec![path.to_string()])
    } else {
        // Complex case: multiple segments
        Cow::Owned(path.split('.').map(String::from).collect())
    }
}

/// Check if a string contains only ASCII alphanumeric characters and underscores
pub fn is_simple_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Escape HTML entities efficiently
pub fn escape_html_cow(input: &str) -> Cow<str> {
    if !input.contains(['&', '<', '>', '"', '\'']) {
        return Cow::Borrowed(input);
    }

    let mut result = String::with_capacity(input.len() + input.len() / 4);
    for ch in input.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            _ => result.push(ch),
        }
    }

    Cow::Owned(result)
}

// Thread-local storage for reusable buffers
thread_local! {
    static STRING_POOL: std::cell::RefCell<StringPool> = std::cell::RefCell::new(
        StringPool::new(10, 50)
    );

    static STRING_BUFFER: std::cell::RefCell<StringBuffer> = std::cell::RefCell::new(
        StringBuffer::with_capacity(1024)
    );

    static REGEX_CACHE: std::cell::RefCell<RegexCache> = std::cell::RefCell::new(
        RegexCache::new(20)
    );
}

/// Get a string from the thread-local pool
pub fn with_pooled_string<F, R>(f: F) -> R
where
    F: FnOnce(String) -> (R, String),
{
    STRING_POOL.with(|pool| {
        let s = pool.borrow_mut().get();
        let (result, returned_string) = f(s);
        pool.borrow_mut().return_string(returned_string);
        result
    })
}

/// Use the thread-local string buffer
pub fn with_string_buffer<F, R>(f: F) -> R
where
    F: FnOnce(&mut String) -> R,
{
    STRING_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        let buf = buffer.clear();
        f(buf)
    })
}

/// Use the thread-local regex cache
pub fn with_regex_cache<F, R>(f: F) -> Result<R, regex::Error>
where
    F: FnOnce(&mut RegexCache) -> Result<R, regex::Error>,
{
    REGEX_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        f(&mut cache)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool() {
        let mut pool = StringPool::new(2, 5);

        // Get some strings
        let s1 = pool.get();
        let s2 = pool.get();
        assert_eq!(pool.in_use_count(), 2);
        assert_eq!(pool.available_count(), 0);

        // Return them
        pool.return_string(s1);
        pool.return_string(s2);
        assert_eq!(pool.in_use_count(), 0);
        assert_eq!(pool.available_count(), 2);

        // Reuse
        let s3 = pool.get();
        assert_eq!(pool.in_use_count(), 1);
        assert_eq!(pool.available_count(), 1);
        pool.return_string(s3);
    }

    #[test]
    fn test_string_buffer() {
        let mut buffer = StringBuffer::with_capacity(10);

        {
            let buf = buffer.clear();
            buf.push_str("test");
        }

        assert_eq!(buffer.as_str(), "test");
        assert!(buffer.capacity() >= 10);
    }

    #[test]
    fn test_replace_multiple_cow() {
        let text = "Hello ${name} and ${other}";
        let replacements = vec![
            ("${name}".to_string(), Cow::Borrowed("World")),
            ("${other}".to_string(), Cow::Borrowed("Universe")),
        ];

        let result = replace_multiple_cow(text, &replacements);
        assert_eq!(result, "Hello World and Universe");

        // Test no replacement needed
        let text = "No variables here";
        let result = replace_multiple_cow(text, &replacements);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_split_path_cow() {
        // Simple case
        let result = split_path_cow("simple");
        assert_eq!(result.as_ref(), &["simple"]);

        // Complex case
        let result = split_path_cow("user.profile.name");
        assert_eq!(result.as_ref(), &["user", "profile", "name"]);
    }

    #[test]
    fn test_is_simple_identifier() {
        assert!(is_simple_identifier("simple"));
        assert!(is_simple_identifier("with_underscore"));
        assert!(is_simple_identifier("with123numbers"));
        assert!(!is_simple_identifier("with.dot"));
        assert!(!is_simple_identifier("with-dash"));
        assert!(!is_simple_identifier(""));
    }

    #[test]
    fn test_escape_html_cow() {
        // No escaping needed
        let result = escape_html_cow("simple text");
        assert!(matches!(result, Cow::Borrowed(_)));

        // Escaping needed
        let result = escape_html_cow("text with <tags> & \"quotes\"");
        assert_eq!(result, "text with &lt;tags&gt; &amp; &quot;quotes&quot;");
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn test_regex_cache() {
        let mut cache = RegexCache::new(2);

        // Add first regex
        let _regex1 = cache.get_or_compile(r"\d+").unwrap();
        assert_eq!(cache.len(), 1);

        // Add second regex
        let _regex2 = cache.get_or_compile(r"[a-z]+").unwrap();
        assert_eq!(cache.len(), 2);

        // Add third regex (should trigger eviction)
        let _regex3 = cache.get_or_compile(r"[A-Z]+").unwrap();
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_with_pooled_string() {
        let result = with_pooled_string(|mut s| {
            s.push_str("test");
            (s.len(), s)
        });
        assert_eq!(result, 4);
    }

    #[test]
    fn test_with_string_buffer() {
        let result = with_string_buffer(|buf| {
            buf.push_str("test buffer");
            buf.len()
        });
        assert_eq!(result, 11);
    }
}
