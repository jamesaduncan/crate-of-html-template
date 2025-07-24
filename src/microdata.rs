//! Microdata extraction from DOM elements
//! 
//! This module provides functionality to extract microdata values from
//! DOM elements for cross-document rendering.

use std::collections::HashMap;
use dom_query::{Document, Selection};
use serde_json::{Value as JsonValue, Map};

use crate::error::{Error, Result};
use crate::value::RenderValue;
use crate::node_ext::NodeExt;

/// Extract microdata from a DOM element
pub fn extract_microdata(element: &dom_query::Node) -> Result<JsonValue> {
    // Check if this is a microdata item
    if !element.has_attr("itemscope") {
        // If not itemscope, just extract the value
        return Ok(extract_value(element));
    }
    
    // Create object for this item
    let mut item = Map::new();
    
    // Add @type if present
    if let Some(itemtype) = element.attr("itemtype") {
        item.insert("@type".to_string(), JsonValue::String(itemtype.to_string()));
    }
    
    // Add @id if present
    if let Some(itemid) = element.attr("itemid") {
        item.insert("@id".to_string(), JsonValue::String(itemid.to_string()));
    }
    
    // Find all properties within this item
    let properties = find_properties(element);
    
    // Group properties by name
    let mut property_map: HashMap<String, Vec<JsonValue>> = HashMap::new();
    
    for (name, value) in properties {
        property_map.entry(name).or_insert_with(Vec::new).push(value);
    }
    
    // Add properties to item
    for (name, values) in property_map {
        if values.len() == 1 {
            item.insert(name, values.into_iter().next().unwrap());
        } else {
            item.insert(name, JsonValue::Array(values));
        }
    }
    
    Ok(JsonValue::Object(item))
}

/// Find all properties within an item
fn find_properties(item: &dom_query::Node) -> Vec<(String, JsonValue)> {
    let mut properties = Vec::new();
    
    // Create a selection from the item
    let item_selection = Selection::from(item.clone());
    
    // Find all elements with itemprop within this item
    // but not within nested itemscope elements
    let prop_elements = item_selection.select("[itemprop]");
    
    for element in prop_elements.nodes() {
        // Check if this property belongs to a nested item
        if is_nested_item(&element, item) {
            continue;
        }
        
        if let Some(itemprop) = element.attr("itemprop") {
            let value = if element.has_attr("itemscope") {
                // Nested item - extract recursively
                extract_microdata(&element).unwrap_or(JsonValue::Null)
            } else {
                extract_value(&element)
            };
            
            // Handle multiple property names
            for prop_name in itemprop.split_whitespace() {
                properties.push((prop_name.to_string(), value.clone()));
            }
        }
    }
    
    properties
}

/// Check if an element belongs to a nested itemscope
fn is_nested_item(element: &dom_query::Node, root_item: &dom_query::Node) -> bool {
    let mut current = element.parent();
    
    while let Some(parent) = current {
        // If we reach the root item, this is not nested
        // Compare using the id field of NodeRef
        if parent.id == root_item.id {
            return false;
        }
        
        // If we find an itemscope before reaching root, this is nested
        if parent.has_attr("itemscope") {
            return true;
        }
        
        current = parent.parent();
    }
    
    false
}

/// Extract value from an element based on its type
fn extract_value(element: &dom_query::Node) -> JsonValue {
    // Check for special elements
    if let Some(tag_name) = element.node_name() {
        match tag_name.to_lowercase().as_str() {
            // Meta elements use content attribute
            "meta" => {
                if let Some(content) = element.attr("content") {
                    return JsonValue::String(content.to_string());
                }
            }
            // Link elements use href
            "link" | "a" | "area" => {
                if let Some(href) = element.attr("href") {
                    return JsonValue::String(href.to_string());
                }
            }
            // Image elements use src
            "img" | "audio" | "video" | "source" | "track" | "embed" => {
                if let Some(src) = element.attr("src") {
                    return JsonValue::String(src.to_string());
                }
            }
            // Object elements use data
            "object" => {
                if let Some(data) = element.attr("data") {
                    return JsonValue::String(data.to_string());
                }
            }
            // Time elements use datetime if available
            "time" => {
                if let Some(datetime) = element.attr("datetime") {
                    return JsonValue::String(datetime.to_string());
                }
            }
            // Data elements use value if available
            "data" | "meter" => {
                if let Some(value) = element.attr("value") {
                    return JsonValue::String(value.to_string());
                }
            }
            _ => {}
        }
    }
    
    // Default to text content
    JsonValue::String(element.text().to_string())
}

/// Extract microdata from a document
pub fn extract_microdata_from_document(doc: &Document) -> Result<Vec<JsonValue>> {
    let mut items = Vec::new();
    
    // Find all top-level microdata items (itemscope without itemprop)
    let top_items = doc.select("[itemscope]:not([itemprop])");
    
    for item in top_items.nodes() {
        if let Ok(data) = extract_microdata(&item) {
            items.push(data);
        }
    }
    
    Ok(items)
}

/// Extract microdata from HTML string
pub fn extract_microdata_from_html(html: &str) -> Result<Vec<JsonValue>> {
    let doc = Document::from(html);
    extract_microdata_from_document(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_extract_simple_microdata() {
        let html = r#"
            <div itemscope itemtype="https://schema.org/Person">
                <span itemprop="name">John Doe</span>
                <span itemprop="email">john@example.com</span>
            </div>
        "#;
        
        let doc = Document::from(html);
        let items = extract_microdata_from_document(&doc).unwrap();
        
        assert_eq!(items.len(), 1);
        let item = &items[0];
        
        assert_eq!(item["@type"], "https://schema.org/Person");
        assert_eq!(item["name"], "John Doe");
        assert_eq!(item["email"], "john@example.com");
    }
    
    #[test]
    fn test_extract_nested_microdata() {
        let html = r#"
            <div itemscope itemtype="https://schema.org/Article">
                <h1 itemprop="headline">Article Title</h1>
                <div itemprop="author" itemscope itemtype="https://schema.org/Person">
                    <span itemprop="name">Jane Smith</span>
                    <span itemprop="email">jane@example.com</span>
                </div>
                <p itemprop="articleBody">Article content here.</p>
            </div>
        "#;
        
        let doc = Document::from(html);
        let items = extract_microdata_from_document(&doc).unwrap();
        
        assert_eq!(items.len(), 1);
        let article = &items[0];
        
        assert_eq!(article["@type"], "https://schema.org/Article");
        assert_eq!(article["headline"], "Article Title");
        assert_eq!(article["articleBody"], "Article content here.");
        
        // Check nested author
        let author = &article["author"];
        assert_eq!(author["@type"], "https://schema.org/Person");
        assert_eq!(author["name"], "Jane Smith");
        assert_eq!(author["email"], "jane@example.com");
    }
    
    #[test]
    fn test_extract_special_elements() {
        let html = r#"
            <div itemscope>
                <meta itemprop="datePublished" content="2024-01-01">
                <a itemprop="url" href="https://example.com">Link</a>
                <img itemprop="image" src="image.jpg" alt="Image">
                <time itemprop="dateModified" datetime="2024-01-02">Jan 2</time>
            </div>
        "#;
        
        let doc = Document::from(html);
        let items = extract_microdata_from_document(&doc).unwrap();
        
        assert_eq!(items.len(), 1);
        let item = &items[0];
        
        assert_eq!(item["datePublished"], "2024-01-01");
        assert_eq!(item["url"], "https://example.com");
        assert_eq!(item["image"], "image.jpg");
        assert_eq!(item["dateModified"], "2024-01-02");
    }
    
    #[test]
    fn test_multiple_property_names() {
        let html = r#"
            <div itemscope>
                <span itemprop="name nickname">John "Johnny" Doe</span>
            </div>
        "#;
        
        let doc = Document::from(html);
        let items = extract_microdata_from_document(&doc).unwrap();
        
        assert_eq!(items.len(), 1);
        let item = &items[0];
        
        assert_eq!(item["name"], "John \"Johnny\" Doe");
        assert_eq!(item["nickname"], "John \"Johnny\" Doe");
    }
    
    #[test]
    fn test_array_properties() {
        let html = r#"
            <div itemscope>
                <span itemprop="tag">rust</span>
                <span itemprop="tag">html</span>
                <span itemprop="tag">template</span>
            </div>
        "#;
        
        let doc = Document::from(html);
        let items = extract_microdata_from_document(&doc).unwrap();
        
        assert_eq!(items.len(), 1);
        let item = &items[0];
        
        let tags = item["tag"].as_array().unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0], "rust");
        assert_eq!(tags[1], "html");
        assert_eq!(tags[2], "template");
    }
    
    #[test]
    fn test_cross_document_data_extraction() {
        let source_html = r#"
            <article itemscope itemtype="https://schema.org/Article">
                <h1 itemprop="headline">How to Use Rust Templates</h1>
                <div itemprop="author" itemscope itemtype="https://schema.org/Person">
                    <span itemprop="name">Jane Developer</span>
                    <span itemprop="email">jane@example.com</span>
                </div>
                <time itemprop="datePublished" datetime="2024-01-15">January 15, 2024</time>
                <div itemprop="articleBody">
                    <p>This article explains how to use Rust templates effectively.</p>
                </div>
            </article>
        "#;
        
        let doc = Document::from(source_html);
        let items = extract_microdata_from_document(&doc).unwrap();
        
        assert_eq!(items.len(), 1);
        let article = &items[0];
        
        assert_eq!(article["@type"], "https://schema.org/Article");
        assert_eq!(article["headline"], "How to Use Rust Templates");
        assert_eq!(article["datePublished"], "2024-01-15");
        assert!(article["articleBody"].as_str().unwrap().contains("This article explains how to use Rust templates effectively."));
        
        // Check nested author
        let author = &article["author"];
        assert!(author.is_object());
        assert_eq!(author["@type"], "https://schema.org/Person");
        assert_eq!(author["name"], "Jane Developer");
        assert_eq!(author["email"], "jane@example.com");
    }
    
    #[test]
    fn test_itemid_extraction() {
        let html = r#"
            <div itemscope itemtype="https://schema.org/Person" itemid="https://example.com/users/123">
                <span itemprop="name">John Doe</span>
                <span itemprop="email">john@example.com</span>
            </div>
        "#;
        
        let doc = Document::from(html);
        let items = extract_microdata_from_document(&doc).unwrap();
        
        assert_eq!(items.len(), 1);
        let person = &items[0];
        
        assert_eq!(person["@type"], "https://schema.org/Person");
        assert_eq!(person["@id"], "https://example.com/users/123");
        assert_eq!(person["name"], "John Doe");
        assert_eq!(person["email"], "john@example.com");
    }
    
    #[test]
    fn test_complex_nested_structure() {
        let html = r#"
            <div itemscope itemtype="https://schema.org/Recipe">
                <h1 itemprop="name">Chocolate Chip Cookies</h1>
                <div itemprop="author" itemscope itemtype="https://schema.org/Person">
                    <span itemprop="name">Chef Alice</span>
                </div>
                <div itemprop="nutrition" itemscope itemtype="https://schema.org/NutritionInformation">
                    <span itemprop="calories">150</span>
                    <span itemprop="fatContent">8g</span>
                </div>
                <ul>
                    <li itemprop="recipeIngredient">2 cups flour</li>
                    <li itemprop="recipeIngredient">1 cup sugar</li>
                    <li itemprop="recipeIngredient">1/2 cup butter</li>
                </ul>
            </div>
        "#;
        
        let doc = Document::from(html);
        let items = extract_microdata_from_document(&doc).unwrap();
        
        assert_eq!(items.len(), 1);
        let recipe = &items[0];
        
        assert_eq!(recipe["@type"], "https://schema.org/Recipe");
        assert_eq!(recipe["name"], "Chocolate Chip Cookies");
        
        // Check nested author
        let author = &recipe["author"];
        assert_eq!(author["@type"], "https://schema.org/Person");
        assert_eq!(author["name"], "Chef Alice");
        
        // Check nested nutrition
        let nutrition = &recipe["nutrition"];
        assert_eq!(nutrition["@type"], "https://schema.org/NutritionInformation");
        assert_eq!(nutrition["calories"], "150");
        assert_eq!(nutrition["fatContent"], "8g");
        
        // Check array of ingredients
        let ingredients = recipe["recipeIngredient"].as_array().unwrap();
        assert_eq!(ingredients.len(), 3);
        assert_eq!(ingredients[0], "2 cups flour");
        assert_eq!(ingredients[1], "1 cup sugar");
        assert_eq!(ingredients[2], "1/2 cup butter");
    }
}