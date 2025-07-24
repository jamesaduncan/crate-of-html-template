use std::borrow::Cow;
use serde::Serialize;
use serde_json::Value as JsonValue;

pub trait RenderValue {
    fn get_property(&self, path: &[String]) -> Option<Cow<str>>;
    fn is_array(&self) -> bool;
    fn as_array(&self) -> Option<Vec<&dyn RenderValue>>;
    fn get_type(&self) -> Option<&str>;
    fn get_id(&self) -> Option<&str>;
}

impl RenderValue for JsonValue {
    fn get_property(&self, path: &[String]) -> Option<Cow<str>> {
        if path.is_empty() {
            return match self {
                JsonValue::String(s) => Some(Cow::Borrowed(s.as_str())),
                JsonValue::Number(n) => Some(Cow::Owned(n.to_string())),
                JsonValue::Bool(b) => Some(Cow::Owned(b.to_string())),
                JsonValue::Null => None,
                _ => None,
            };
        }
        
        let mut current = self;
        for segment in path.iter() {
            // Handle array access like items[0]
            if let Some(array_match) = parse_array_access(segment) {
                let (prop_name, index) = array_match;
                
                // First get the property
                current = current.get(&prop_name)?;
                
                // Then access the array index
                if let JsonValue::Array(arr) = current {
                    current = arr.get(index)?;
                } else {
                    return None;
                }
            } else {
                current = current.get(segment)?;
            }
        }
        
        match current {
            JsonValue::String(s) => Some(Cow::Borrowed(s.as_str())),
            JsonValue::Number(n) => Some(Cow::Owned(n.to_string())),
            JsonValue::Bool(b) => Some(Cow::Owned(b.to_string())),
            JsonValue::Null => None,
            _ => None,
        }
    }
    
    fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }
    
    fn as_array(&self) -> Option<Vec<&dyn RenderValue>> {
        match self {
            JsonValue::Array(arr) => {
                Some(arr.iter().map(|v| v as &dyn RenderValue).collect())
            }
            _ => None,
        }
    }
    
    fn get_type(&self) -> Option<&str> {
        match self {
            JsonValue::Object(obj) => {
                obj.get("@type")
                    .and_then(|v| v.as_str())
            }
            _ => None,
        }
    }
    
    fn get_id(&self) -> Option<&str> {
        match self {
            JsonValue::Object(obj) => {
                obj.get("@id")
                    .and_then(|v| v.as_str())
            }
            _ => None,
        }
    }
}

impl RenderValue for String {
    fn get_property(&self, path: &[String]) -> Option<Cow<str>> {
        if path.is_empty() {
            Some(Cow::Borrowed(self.as_str()))
        } else {
            None
        }
    }
    
    fn is_array(&self) -> bool {
        false
    }
    
    fn as_array(&self) -> Option<Vec<&dyn RenderValue>> {
        None
    }
    
    fn get_type(&self) -> Option<&str> {
        None
    }
    
    fn get_id(&self) -> Option<&str> {
        None
    }
}

impl RenderValue for &str {
    fn get_property(&self, path: &[String]) -> Option<Cow<str>> {
        if path.is_empty() {
            Some(Cow::Borrowed(*self))
        } else {
            None
        }
    }
    
    fn is_array(&self) -> bool {
        false
    }
    
    fn as_array(&self) -> Option<Vec<&dyn RenderValue>> {
        None
    }
    
    fn get_type(&self) -> Option<&str> {
        None
    }
    
    fn get_id(&self) -> Option<&str> {
        None
    }
}

macro_rules! impl_render_value_for_number {
    ($($t:ty),*) => {
        $(
            impl RenderValue for $t {
                fn get_property(&self, path: &[String]) -> Option<Cow<str>> {
                    if path.is_empty() {
                        Some(Cow::Owned(self.to_string()))
                    } else {
                        None
                    }
                }
                
                fn is_array(&self) -> bool {
                    false
                }
                
                fn as_array(&self) -> Option<Vec<&dyn RenderValue>> {
                    None
                }
                
                fn get_type(&self) -> Option<&str> {
                    None
                }
                
                fn get_id(&self) -> Option<&str> {
                    None
                }
            }
        )*
    };
}

impl_render_value_for_number!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);

impl<T: RenderValue> RenderValue for Vec<T> {
    fn get_property(&self, path: &[String]) -> Option<Cow<str>> {
        if path.is_empty() {
            return None;
        }
        
        // Handle array index access
        if path.len() == 1 {
            if let Ok(index) = path[0].parse::<usize>() {
                return self.get(index)?.get_property(&[]);
            }
        }
        
        None
    }
    
    fn is_array(&self) -> bool {
        true
    }
    
    fn as_array(&self) -> Option<Vec<&dyn RenderValue>> {
        Some(self.iter().map(|v| v as &dyn RenderValue).collect())
    }
    
    fn get_type(&self) -> Option<&str> {
        None
    }
    
    fn get_id(&self) -> Option<&str> {
        None
    }
}

// Generic implementation for any Serialize type
pub struct SerializeWrapper<'a, T: Serialize> {
    value: &'a T,
    cached_json: once_cell::sync::OnceCell<JsonValue>,
}

impl<'a, T: Serialize> SerializeWrapper<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self {
            value,
            cached_json: once_cell::sync::OnceCell::new(),
        }
    }
    
    fn as_json(&self) -> &JsonValue {
        self.cached_json.get_or_init(|| {
            serde_json::to_value(self.value).unwrap_or(JsonValue::Null)
        })
    }
}

impl<'a, T: Serialize> RenderValue for SerializeWrapper<'a, T> {
    fn get_property(&self, path: &[String]) -> Option<Cow<str>> {
        self.as_json().get_property(path)
    }
    
    fn is_array(&self) -> bool {
        self.as_json().is_array()
    }
    
    fn as_array(&self) -> Option<Vec<&dyn RenderValue>> {
        // This is a bit inefficient but necessary for the trait
        None
    }
    
    fn get_type(&self) -> Option<&str> {
        self.as_json().get_type()
    }
    
    fn get_id(&self) -> Option<&str> {
        self.as_json().get_id()
    }
}

fn parse_array_access(segment: &str) -> Option<(String, usize)> {
    if let Some(bracket_pos) = segment.find('[') {
        if segment.ends_with(']') {
            let prop_name = segment[..bracket_pos].to_string();
            let index_str = &segment[bracket_pos + 1..segment.len() - 1];
            if let Ok(index) = index_str.parse::<usize>() {
                return Some((prop_name, index));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_json_value_simple_property() {
        let data = json!({
            "name": "John Doe",
            "age": 30
        });
        
        assert_eq!(
            data.get_property(&["name".to_string()]).unwrap(),
            "John Doe"
        );
        assert_eq!(
            data.get_property(&["age".to_string()]).unwrap(),
            "30"
        );
    }
    
    #[test]
    fn test_json_value_nested_property() {
        let data = json!({
            "user": {
                "profile": {
                    "name": "Jane Doe"
                }
            }
        });
        
        let path = vec!["user".to_string(), "profile".to_string(), "name".to_string()];
        assert_eq!(
            data.get_property(&path).unwrap(),
            "Jane Doe"
        );
    }
    
    #[test]
    fn test_json_value_array() {
        let data = json!(["a", "b", "c"]);
        
        assert!(data.is_array());
        let array = data.as_array().unwrap();
        assert_eq!(array.len(), 3);
    }
    
    #[test]
    fn test_json_value_array_access() {
        let data = json!({
            "items": ["first", "second", "third"]
        });
        
        let path = vec!["items[0]".to_string()];
        assert_eq!(
            data.get_property(&path).unwrap(),
            "first"
        );
        
        let path = vec!["items[1]".to_string()];
        assert_eq!(
            data.get_property(&path).unwrap(),
            "second"
        );
    }
    
    #[test]
    fn test_json_value_type_and_id() {
        let data = json!({
            "@type": "Person",
            "@id": "john-doe",
            "name": "John Doe"
        });
        
        assert_eq!(data.get_type().unwrap(), "Person");
        assert_eq!(data.get_id().unwrap(), "john-doe");
    }
    
    #[test]
    fn test_string_render_value() {
        let s = String::from("Hello");
        assert_eq!(s.get_property(&[]).unwrap(), "Hello");
        assert!(s.get_property(&["any".to_string()]).is_none());
        assert!(!s.is_array());
    }
    
    #[test]
    fn test_number_render_value() {
        let n: i32 = 42;
        assert_eq!(n.get_property(&[]).unwrap(), "42");
        
        let f: f64 = 3.14;
        assert_eq!(f.get_property(&[]).unwrap(), "3.14");
    }
    
    #[test]
    fn test_vec_render_value() {
        let v = vec!["a", "b", "c"];
        assert!(v.is_array());
        
        let array = v.as_array().unwrap();
        assert_eq!(array.len(), 3);
        
        assert_eq!(v.get_property(&["0".to_string()]).unwrap(), "a");
        assert_eq!(v.get_property(&["1".to_string()]).unwrap(), "b");
    }
    
    #[test]
    fn test_parse_array_access() {
        assert_eq!(
            parse_array_access("items[0]"),
            Some(("items".to_string(), 0))
        );
        assert_eq!(
            parse_array_access("users[42]"),
            Some(("users".to_string(), 42))
        );
        assert_eq!(parse_array_access("plain"), None);
        assert_eq!(parse_array_access("invalid["), None);
        assert_eq!(parse_array_access("invalid[abc]"), None);
    }
}