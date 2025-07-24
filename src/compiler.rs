use std::sync::Arc;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::Result;
use crate::types::*;
use crate::parser::Parser;

static ARRAY_INDEX_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+)\[(\d+)\]").expect("Invalid array index regex")
});

pub struct Compiler;

impl Compiler {
    pub fn compile(html: &str, root_selector: Option<&str>) -> Result<Arc<CompiledTemplate>> {
        let parser = Parser::new(html)?;
        let mut template = parser.parse_template(root_selector)?;
        
        // Optimize the template structure
        Self::optimize_template(&mut template)?;
        
        // Build lookup tables for faster rendering
        Self::build_lookup_tables(&mut template)?;
        
        Ok(Arc::new(template))
    }
    
    pub fn compile_from_template(template: CompiledTemplate) -> Arc<CompiledTemplate> {
        Arc::new(template)
    }
    
    fn optimize_template(template: &mut CompiledTemplate) -> Result<()> {
        // Sort elements by selector specificity for faster matching
        template.elements.sort_by(|a, b| {
            Self::selector_specificity(&a.selector)
                .cmp(&Self::selector_specificity(&b.selector))
        });
        
        // Pre-process variable paths
        for element in &mut template.elements {
            for property in &mut element.properties {
                for variable in &mut property.variables {
                    variable.path = Self::optimize_variable_path(&variable.path);
                }
            }
        }
        
        Ok(())
    }
    
    fn build_lookup_tables(_template: &mut CompiledTemplate) -> Result<()> {
        // In a full implementation, we would build various lookup tables:
        // - Property name to element index mapping
        // - Selector to element mapping
        // - Constraint dependency graph
        
        Ok(())
    }
    
    fn selector_specificity(selector: &str) -> u32 {
        let mut specificity = 0;
        
        // Count IDs (weight: 100)
        specificity += selector.matches('#').count() as u32 * 100;
        
        // Count classes and attributes (weight: 10)
        specificity += selector.matches('.').count() as u32 * 10;
        specificity += selector.matches('[').count() as u32 * 10;
        
        // Count element types (weight: 1)
        // Simple heuristic: count lowercase sequences at the start
        if let Some(first_char) = selector.chars().next() {
            if first_char.is_lowercase() {
                specificity += 1;
            }
        }
        
        specificity
    }
    
    fn optimize_variable_path(path: &[String]) -> Vec<String> {
        // Pre-process array indices and other optimizations
        path.iter()
            .map(|segment| {
                if let Some(caps) = ARRAY_INDEX_REGEX.captures(segment) {
                    // Store array accesses in an optimized format
                    format!("{}[{}]", &caps[1], &caps[2])
                } else {
                    segment.clone()
                }
            })
            .collect()
    }
}

/// Pre-compiled template cache to avoid re-parsing
pub struct TemplateCache {
    cache: HashMap<String, Arc<CompiledTemplate>>,
}

impl TemplateCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    pub fn get_or_compile(
        &mut self,
        key: &str,
        html: &str,
        root_selector: Option<&str>,
    ) -> Result<Arc<CompiledTemplate>> {
        if let Some(compiled) = self.cache.get(key) {
            return Ok(Arc::clone(compiled));
        }
        
        let compiled = Compiler::compile(html, root_selector)?;
        self.cache.insert(key.to_string(), Arc::clone(&compiled));
        Ok(compiled)
    }
    
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compile_simple_template() {
        let html = r#"
            <template>
                <div>
                    <h1 itemprop="title">${title}</h1>
                    <p itemprop="content">${content}</p>
                </div>
            </template>
        "#;
        
        let compiled = Compiler::compile(html, Some("div")).unwrap();
        assert_eq!(compiled.root_selector, Some("div".to_string()));
        assert_eq!(compiled.elements.len(), 2);
    }
    
    #[test]
    fn test_selector_specificity() {
        assert_eq!(Compiler::selector_specificity("div"), 1);
        assert_eq!(Compiler::selector_specificity(".class"), 10);
        assert_eq!(Compiler::selector_specificity("#id"), 100);
        assert_eq!(Compiler::selector_specificity("div.class#id"), 111);
        assert_eq!(Compiler::selector_specificity("[itemprop=\"test\"]"), 10);
    }
    
    #[test]
    fn test_optimize_variable_path() {
        let path = vec!["users".to_string(), "0".to_string(), "name".to_string()];
        let optimized = Compiler::optimize_variable_path(&path);
        assert_eq!(optimized, vec!["users", "0", "name"]);
        
        let path = vec!["items[0]".to_string(), "value".to_string()];
        let optimized = Compiler::optimize_variable_path(&path);
        assert_eq!(optimized, vec!["items[0]", "value"]);
    }
    
    #[test]
    fn test_template_cache() {
        let mut cache = TemplateCache::new();
        
        let html = r#"<template><div itemprop="test"></div></template>"#;
        
        // First call should compile
        let compiled1 = cache.get_or_compile("key1", html, None).unwrap();
        
        // Second call should return cached version
        let compiled2 = cache.get_or_compile("key1", html, None).unwrap();
        
        // Should be the same Arc
        assert!(Arc::ptr_eq(&compiled1, &compiled2));
        
        // Different key should compile new template
        let compiled3 = cache.get_or_compile("key2", html, None).unwrap();
        assert!(!Arc::ptr_eq(&compiled1, &compiled3));
        
        // Clear cache
        cache.clear();
        let compiled4 = cache.get_or_compile("key1", html, None).unwrap();
        assert!(!Arc::ptr_eq(&compiled1, &compiled4));
    }
    
    #[test]
    fn test_compile_with_nested_structure() {
        let html = r#"
            <template>
                <article>
                    <header itemprop="header" itemscope>
                        <h1 itemprop="title">${title}</h1>
                        <time itemprop="date">${date}</time>
                    </header>
                    <div itemprop="content">
                        ${content}
                    </div>
                </article>
            </template>
        "#;
        
        let compiled = Compiler::compile(html, Some("article")).unwrap();
        
        // Should have parsed all elements with itemprop
        assert_eq!(compiled.elements.len(), 4); // header, title, date, content
        
        // Check that header is marked as scope
        let header = compiled.elements.iter()
            .find(|e| e.properties.iter().any(|p| p.name == "header"))
            .unwrap();
        assert!(header.is_scope);
    }
}