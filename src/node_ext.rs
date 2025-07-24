use dom_query::Node;
use std::collections::HashMap;

/// Extension trait for dom_query::Node to provide helper methods
pub trait NodeExt {
    fn text_content(&self) -> String;
    fn attrs(&self) -> Option<HashMap<String, String>>;
    fn first_element_child(&self) -> Option<Node>;
}

impl<'a> NodeExt for Node<'a> {
    fn text_content(&self) -> String {
        self.text().to_string()
    }
    
    fn attrs(&self) -> Option<HashMap<String, String>> {
        // Use the query method to access element data
        self.query(|node| {
            if let Some(element) = node.as_element() {
                if element.attrs.is_empty() {
                    None
                } else {
                    let mut map = HashMap::new();
                    for attr in &element.attrs {
                        map.insert(
                            attr.name.local.to_string(),
                            attr.value.to_string(),
                        );
                    }
                    Some(map)
                }
            } else {
                None
            }
        }).flatten()
    }
    
    fn first_element_child(&self) -> Option<Node<'a>> {
        self.first_element_child()
    }
}