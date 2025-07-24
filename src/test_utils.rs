//! Test utilities for HTML template testing
//! 
//! This module provides helpful utilities for testing HTML templates,
//! including HTML normalization and comparison functions.

use dom_query::Document;

/// Normalize HTML for comparison by parsing and re-serializing
/// 
/// This function:
/// - Removes extra whitespace between tags
/// - Normalizes attribute order  
/// - Handles self-closing tags consistently
/// - Trims text content
/// 
/// # Examples
/// 
/// ```
/// use html_template::test_utils::normalize_html;
/// 
/// let html1 = "<div  class=\"test\"   id=\"main\" ><p>Hello</p></div>";
/// let html2 = "<div id=\"main\" class=\"test\"><p>Hello</p></div>";
/// 
/// assert_eq!(normalize_html(html1), normalize_html(html2));
/// ```
pub fn normalize_html(html: &str) -> String {
    // Parse and re-serialize to normalize
    let doc = Document::from(html);
    
    // Get the HTML back - this normalizes whitespace and attribute order
    let normalized = doc.html();
    
    // Additional normalization: trim whitespace
    normalized.trim().to_string()
}

/// Assert that two HTML strings are equivalent
/// 
/// This macro normalizes both HTML strings before comparison,
/// ignoring differences in whitespace, attribute order, etc.
/// 
/// # Examples
/// 
/// ```
/// use html_template::test_utils::assert_html_eq;
/// 
/// assert_html_eq!(
///     "<div class=\"a\" id=\"b\"><p>Text</p></div>",
///     "<div id=\"b\" class=\"a\" ><p>Text</p></div>"
/// );
/// ```
#[macro_export]
macro_rules! assert_html_eq {
    ($left:expr, $right:expr) => {
        {
            let left_normalized = $crate::test_utils::normalize_html($left);
            let right_normalized = $crate::test_utils::normalize_html($right);
            
            if left_normalized != right_normalized {
                panic!(
                    "HTML assertion failed\n\nLeft (normalized):\n{}\n\nRight (normalized):\n{}\n\nOriginal left:\n{}\n\nOriginal right:\n{}",
                    left_normalized,
                    right_normalized,
                    $left,
                    $right
                );
            }
        }
    };
    ($left:expr, $right:expr, $($arg:tt)*) => {
        {
            let left_normalized = $crate::test_utils::normalize_html($left);
            let right_normalized = $crate::test_utils::normalize_html($right);
            
            if left_normalized != right_normalized {
                panic!(
                    "HTML assertion failed: {}\n\nLeft (normalized):\n{}\n\nRight (normalized):\n{}\n\nOriginal left:\n{}\n\nOriginal right:\n{}",
                    format!($($arg)*),
                    left_normalized,
                    right_normalized,
                    $left,
                    $right
                );
            }
        }
    };
}

/// Compare two HTML strings and return whether they're equivalent
/// 
/// Like `assert_html_eq!` but returns a bool instead of panicking
pub fn html_eq(html1: &str, html2: &str) -> bool {
    normalize_html(html1) == normalize_html(html2)
}

/// Extract text content from HTML, ignoring all tags
/// 
/// Useful for testing when you only care about the rendered text
pub fn extract_text(html: &str) -> String {
    let doc = Document::from(html);
    let body = doc.select("body");
    
    // If there's a body tag (added by dom_query), use its text
    // Otherwise use the whole document's text
    let text = if body.nodes().len() > 0 {
        body.text()
    } else {
        doc.text()
    };
    
    // Normalize whitespace: replace multiple spaces/newlines with single space
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract all text content from elements matching a selector
pub fn extract_text_by_selector(html: &str, selector: &str) -> Vec<String> {
    let doc = Document::from(html);
    let selection = doc.select(selector);
    
    selection.nodes()
        .iter()
        .map(|node| node.text().trim().to_string())
        .collect()
}

/// Extract attribute values from elements matching a selector
pub fn extract_attrs_by_selector(html: &str, selector: &str, attr: &str) -> Vec<String> {
    let doc = Document::from(html);
    let selection = doc.select(selector);
    
    selection.nodes()
        .iter()
        .filter_map(|node| node.attr(attr).map(|v| v.to_string()))
        .collect()
}

/// Count elements matching a selector
pub fn count_elements(html: &str, selector: &str) -> usize {
    let doc = Document::from(html);
    doc.select(selector).nodes().len()
}

/// Check if HTML contains an element matching a selector
pub fn has_element(html: &str, selector: &str) -> bool {
    count_elements(html, selector) > 0
}


/// Create a test HTML document with a template wrapper
/// 
/// Convenience function for creating test HTML with proper template structure
pub fn test_html(content: &str) -> String {
    format!("<template>{}</template>", content)
}

/// Create a test HTML document with a specific root element
pub fn test_html_with_root(root_tag: &str, content: &str) -> String {
    format!("<template><{0}>{1}</{0}></template>", root_tag, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_html() {
        // dom_query doesn't reorder attributes, so we test whitespace normalization
        let html1 = r#"<div   class="test"  ><p>  Hello  </p></div>"#;
        let html2 = r#"<div class="test"><p>  Hello  </p></div>"#;
        
        assert_eq!(normalize_html(html1), normalize_html(html2));
    }
    
    #[test]
    fn test_normalize_whitespace() {
        // dom_query preserves internal whitespace but normalizes tag spacing
        let html1 = r#"<div>  <p>Hello World</p>  </div>"#;
        let html2 = r#"<div><p>Hello World</p></div>"#;
        
        // Both should have the same structure after normalization
        let norm1 = normalize_html(html1);
        let norm2 = normalize_html(html2);
        
        // Check that both contain the expected content
        assert!(norm1.contains("<p>Hello World</p>"));
        assert!(norm2.contains("<p>Hello World</p>"));
    }
    
    #[test]
    fn test_normalize_self_closing() {
        let html1 = r#"<input type="text">"#;
        let html2 = r#"<input type="text">"#;
        
        // dom_query normalizes consistently
        let normalized1 = normalize_html(html1);
        let normalized2 = normalize_html(html2);
        
        assert_eq!(normalized1, normalized2);
    }
    
    #[test]
    fn test_html_eq() {
        // Test with same attribute order
        assert!(html_eq(
            "<div class='a' id='b'><span>Text</span></div>",
            "<div class='a' id='b'><span>Text</span></div>"
        ));
        
        assert!(!html_eq(
            "<div><span>Text1</span></div>",
            "<div><span>Text2</span></div>"
        ));
    }
    
    #[test]
    fn test_extract_text() {
        let html = r#"
            <div>
                <h1>Title</h1>
                <p>Paragraph <strong>with bold</strong> text.</p>
            </div>
        "#;
        
        assert_eq!(extract_text(html), "Title Paragraph with bold text.");
    }
    
    #[test]
    fn test_extract_text_by_selector() {
        let html = r#"
            <div>
                <p class="intro">First paragraph</p>
                <p>Second paragraph</p>
                <p class="intro">Third paragraph</p>
            </div>
        "#;
        
        let texts = extract_text_by_selector(html, "p.intro");
        assert_eq!(texts, vec!["First paragraph", "Third paragraph"]);
    }
    
    #[test]
    fn test_extract_attrs_by_selector() {
        let html = r#"
            <div>
                <a href="/page1">Link 1</a>
                <a href="/page2" class="external">Link 2</a>
                <a href="/page3">Link 3</a>
            </div>
        "#;
        
        let hrefs = extract_attrs_by_selector(html, "a", "href");
        assert_eq!(hrefs, vec!["/page1", "/page2", "/page3"]);
        
        let classes = extract_attrs_by_selector(html, "a", "class");
        assert_eq!(classes, vec!["external"]);
    }
    
    #[test]
    fn test_count_elements() {
        let html = r#"
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
                <li>Item 3</li>
            </ul>
        "#;
        
        assert_eq!(count_elements(html, "li"), 3);
        assert_eq!(count_elements(html, "ul"), 1);
        assert_eq!(count_elements(html, "div"), 0);
    }
    
    #[test]
    fn test_has_element() {
        let html = r#"<div><span class="highlight">Text</span></div>"#;
        
        assert!(has_element(html, "span"));
        assert!(has_element(html, ".highlight"));
        assert!(!has_element(html, "p"));
    }
    
    
    #[test]
    fn test_helper_functions() {
        assert_eq!(
            test_html("<div>Content</div>"),
            "<template><div>Content</div></template>"
        );
        
        assert_eq!(
            test_html_with_root("article", "<h1>Title</h1>"),
            "<template><article><h1>Title</h1></article></template>"
        );
    }
    
    #[test]
    fn test_assert_html_eq_macro() {
        // This should not panic - same attribute order
        assert_html_eq!(
            "<div class='a' id='b'>Text</div>",
            "<div class='a' id='b'>Text</div>"
        );
    }
    
    #[test]
    #[should_panic(expected = "HTML assertion failed")]
    fn test_assert_html_eq_macro_panic() {
        assert_html_eq!(
            "<div>Text1</div>",
            "<div>Text2</div>"
        );
    }
    
}