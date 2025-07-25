use serde_json::json;

// Mimic what RenderValue does
fn get_property(data: &serde_json::Value, path: &[String]) -> Option<String> {
    if path.is_empty() {
        return Some(data.to_string());
    }

    let mut current = data;
    for segment in path {
        match current {
            serde_json::Value::Object(map) => {
                current = map.get(segment)?;
            }
            _ => return None,
        }
    }

    match current {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => Some(current.to_string()),
    }
}

fn main() {
    let data = json!({
        "email": "alice@example.com"
    });

    // Test property access
    let email_path = vec!["email".to_string()];
    let value = get_property(&data, &email_path);
    println!("get_property(['email']): {:?}", value);

    // Test variable substitution
    let text = "mailto:${email}";
    if let Some(email_value) = value {
        let result = text.replace("${email}", &email_value);
        println!("Substituted: {}", result);
    }
}
