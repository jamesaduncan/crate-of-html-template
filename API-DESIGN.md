# Rust HTML-Template API Design

A Rust implementation of the HTML templating library that uses microdata attributes (itemprop/itemtype/itemscope) for data binding. This implementation uses the `dom_query` crate for HTML parsing and manipulation to maintain API equivalence with the JavaScript version.

## Basic Usage

The Rust version uses strongly-typed structs with serde for JSON serialization and dom_query for DOM manipulation. The API is designed to be ergonomic while maintaining Rust's safety guarantees.

### Basic template structure

```rust
use html_template::HtmlTemplate;
use dom_query::Document;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <span itemprop="foo"></span>
        </template>
    "#;
    
    // Parse HTML document
    let doc = Document::from(html);
    let template_element = doc.select("template").first().unwrap();
    
    // Create template from DOM element
    let tmpl = HtmlTemplate::from_element(template_element, "span").unwrap();
    
    let element = tmpl.render(&json!({ "foo": "bar" })).unwrap();
    println!("{}", element.outer_html());
    // <span itemprop="foo">bar</span>
}
```

### Using derive macros for type safety

```rust
use html_template::{HtmlTemplate, Renderable};
use dom_query::Document;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Renderable)]
struct Person {
    name: String,
    email: String,
}

fn main() {
    let html = r#"
        <template>
            <div>
                <span itemprop="name"></span>
                <a href="mailto:${email}" itemprop="email"></a>
            </div>
        </template>
    "#;
    
    let doc = Document::from(html);
    let template = doc.select("template").first().unwrap();
    
    let tmpl = HtmlTemplate::from_element(template, "div").unwrap();
    
    let person = Person {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    
    let element = tmpl.render(&person).unwrap();
    println!("{}", element.outer_html());
}
```

## Array-like things

Arrays and vectors are handled naturally:

```rust
use html_template::HtmlTemplate;
use dom_query::Document;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <ul><li itemprop="foo[]"></li></ul>
        </template>
    "#;
    
    let doc = Document::from(html);
    let template = doc.select("template").first().unwrap();
    
    let tmpl = HtmlTemplate::from_element(template, "ul").unwrap();
    
    let element = tmpl.render(&json!({ 
        "foo": ["bar", "baz", "boo"] 
    })).unwrap();
    
    println!("{}", element.outer_html());
    // <ul><li itemprop="foo">bar</li><li itemprop="foo">baz</li><li itemprop="foo">boo</li></ul>
}
```

### Strongly typed arrays

```rust
#[derive(Serialize, Deserialize)]
struct TodoList {
    items: Vec<TodoItem>,
}

#[derive(Serialize, Deserialize)]
struct TodoItem {
    title: String,
    completed: bool,
}

let todos = TodoList {
    items: vec![
        TodoItem { title: "Buy milk".into(), completed: false },
        TodoItem { title: "Walk dog".into(), completed: true },
    ],
};
```

## Nested Objects

```rust
use html_template::HtmlTemplate;
use dom_query::Document;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    profile: Profile,
}

#[derive(Serialize, Deserialize)]
struct Profile {
    name: String,
    bio: String,
}

fn main() {
    let html = r#"
        <template>
            <div itemprop="profile" itemscope>
                <h1 itemprop="name"></h1>
                <p itemprop="bio"></p>
            </div>
        </template>
    "#;
    
    let doc = Document::from(html);
    let template = doc.select("template").first().unwrap();
    
    let tmpl = HtmlTemplate::from_element(template, "div").unwrap();
    
    let user = User {
        profile: Profile {
            name: "Jane Doe".to_string(),
            bio: "Rust developer".to_string(),
        },
    };
    
    let element = tmpl.render(&user).unwrap();
}
```

## Attributes

Attribute templating with type safety:

```rust
use html_template::HtmlTemplate;
use dom_query::Document;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Link {
    url: String,
    name: String,
}

fn main() {
    let html = r#"
        <template>
            <a href="${url}" itemprop="name"></a>        
        </template>
    "#;

    let doc = Document::from(html);
    let template = doc.select("template").first().unwrap();
    
    let tmpl = HtmlTemplate::from_element(template, "a").unwrap();

    let link = Link {
        url: "https://example.com".to_string(),
        name: "Example URL".to_string(),
    };

    let element = tmpl.render(&link).unwrap();
    // <a href="https://example.com">Example URL</a>
}
```

## Special Elements

The Rust version handles special elements through a trait system:

```rust
use html_template::{HtmlTemplate, ElementHandler};
use dom_query::Document;

// Built-in handlers for:
// - input elements -> set value attribute
// - select elements -> set selected option
// - meta elements -> set content attribute
// - textarea -> set text content

// Custom handlers can be registered:
let doc = Document::from(html);
let template = doc.select("template").first().unwrap();

let tmpl = HtmlTemplate::from_element(template, None)
    .register_handler::<CustomVideoHandler>()
    .build()
    .unwrap();
```

## Microdata Types with Schema.org

Templates can work with Schema.org types (or any microdata types) by matching on the itemtype attribute:

```rust
use html_template::HtmlTemplate;
use dom_query::Document;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <div itemscope itemtype="https://schema.org/Person">
                <h1 itemprop="name"></h1>
            </div>
            <li itemscope itemtype="https://schema.org/Action" itemprop="name"></li>
        </template>
    "#;
    
    let doc = Document::from(html);
    let template = doc.select("template").first().unwrap();
    
    // When using microdata types, the selector can be omitted
    let tmpl = HtmlTemplate::from_element(template, None).unwrap();
    
    // Using json! macro with JSON-LD format
    let element = tmpl.render(&json!({
        "@type": "Person",
        "@context": "https://schema.org",
        "name": "John Doe"
    })).unwrap();
    
    // Or using strongly typed structs
    #[derive(Serialize, Deserialize)]
    struct Person {
        #[serde(rename = "@context")]
        context: String,
        #[serde(rename = "@type")]
        type_: String,
        name: String,
    }
    
    let person = Person {
        context: "https://schema.org".into(),
        type_: "Person".into(),
        name: "John Doe".into(),
    };
    
    let element = tmpl.render(&person).unwrap();
}
```

### Rendering Arrays of Schema.org Types

```rust
use html_template::HtmlTemplate;
use dom_query::Document;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Person {
    #[serde(rename = "@context")]
    context: String,
    #[serde(rename = "@type")]
    type_: String,
    name: String,
}

let doc = Document::from(html);
let template = doc.select("template").first().unwrap();
let tmpl = HtmlTemplate::from_element(template, None).unwrap();

let people = vec![
    Person {
        context: "https://schema.org".into(),
        type_: "Person".into(),
        name: "John Doe".into(),
    },
    Person {
        context: "https://schema.org".into(),
        type_: "Person".into(),
        name: "Jane Doe".into(),
    },
];

let elements = tmpl.render(&people).unwrap();
// Returns Vec<Element> when input is an array
```

## Constraints and Scopes

The Rust version handles constraints that are defined in the HTML:

```rust
use html_template::HtmlTemplate;
use dom_query::Document;
use serde_json::json;

let html = r#"
    <template>
        <div itemscope itemtype="https://schema.org/Person">
            <h1 itemprop="name"></h1>
            <h2>Jobs</h2>
            <ul>
                <li itemscope itemtype="https://schema.org/Action" data-scope="agent">
                    <span itemprop="name"></span>
                </li>
            </ul>
        </div>
    </template>
"#;

let doc = Document::from(html);
let template = doc.select("template").first().unwrap();

// Constraints are automatically parsed from data-scope and data-constraint attributes
let tmpl = HtmlTemplate::from_element(template, None).unwrap();

// Render with linked data
let data = vec![
    json!({
        "@type": "Action",
        "@context": "https://schema.org",
        "name": "Take the bins out",
        "agent": "#johndoe"
    }),
    json!({
        "@type": "Person",
        "@context": "https://schema.org",
        "@id": "johndoe",
        "name": "John Doe"
    })
];

let elements = tmpl.render(&data).unwrap();
```

## Other Input Sources

### From DOM Elements

Templates can render from other microdata elements in the document:

```rust
use html_template::HtmlTemplate;
use dom_query::Document;

fn main() {
    let html = r#"
        <ul>
            <li itemscope itemtype="https://schema.org/Person">
                <span itemprop="name">John Doe</span>
            </li>
            <li itemscope itemtype="https://schema.org/Person">
                <span itemprop="name">Jane Doe</span>
            </li>
        </ul>
        <template>        
            <div itemscope itemtype="https://schema.org/Person">
                <h1 itemprop="name"></h1>
            </div>        
        </template>
    "#;
    
    let doc = Document::from(html);
    let template = doc.select("template").first().unwrap();
    let source_list = doc.select("ul").first().unwrap();
    
    let tmpl = HtmlTemplate::from_element(template, None).unwrap();
    
    // Render from DOM elements
    let elements = tmpl.render_from_element(&source_list).unwrap();
    
    for elem in elements {
        println!("{}", elem.outer_html());
    }
    // <div itemscope itemtype="https://schema.org/Person">
    //   <h1>John Doe</h1>
    // </div>
    // <div itemscope itemtype="https://schema.org/Person">
    //   <h1>Jane Doe</h1>
    // </div>
}
```

### With itemid attributes

The itemid is generated using the source element's base URI, not the rendering document's base URI. This preserves the original document context:

```rust
let html_with_ids = r#"
    <base href="https://example.com/people">
    <ul>
        <li id="johndoe" itemscope itemtype="https://schema.org/Person">
            <span itemprop="name">John Doe</span>
        </li>
        <li id="janedoe" itemscope itemtype="https://schema.org/Person">
            <span itemprop="name">Jane Doe</span>
        </li>
    </ul>
    <template>        
        <div itemscope itemtype="https://schema.org/Person">
            <h1 itemprop="name"></h1>
        </div>        
    </template>
"#;

let doc = Document::from(html_with_ids);
let template = doc.select("template").first().unwrap();
let source_list = doc.select("ul").first().unwrap();

let tmpl = HtmlTemplate::from_element(template, None).unwrap();

// The library uses each source element's base URI for itemid generation
let elements = tmpl.render_from_element(&source_list).unwrap();

// Rendered elements will have itemid attributes like:
// <div itemscope itemtype="https://schema.org/Person" itemid="https://example.com/people#johndoe">
```

### Cross-Document Rendering

When rendering microdata from external documents, the itemid preserves the source document's base URI:

```rust
use html_template::HtmlTemplate;
use dom_query::Document;

async fn render_from_external_source() -> Result<Vec<Element>, Error> {
    // Current document might be at https://myapp.com/dashboard
    let template_html = r#"
        <template id="person-template">        
            <div itemscope itemtype="https://schema.org/Person">
                <h1 itemprop="name"></h1>
            </div>        
        </template>
    "#;
    
    let template_doc = Document::from(template_html);
    let template = template_doc.select("template").first().unwrap();
    
    // Fetch microdata from another domain
    let response = reqwest::get("https://external-api.com/data/people.html").await?;
    let external_html = response.text().await?;
    
    // Parse the external document - it has its own base URI
    let external_doc = Document::from(&external_html);
    
    // The external HTML might have: <base href="https://external-api.com/data/">
    // and contains: <li id="person123" itemscope itemtype="https://schema.org/Person">
    
    let tmpl = HtmlTemplate::from_element(template, None)?;
    let source_elements = external_doc.select("ul").first().unwrap();
    
    // When rendering, the library preserves the source element's base URI
    let elements = tmpl.render_from_element(&source_elements)?;
    
    // The itemid uses the source element's baseURI
    let itemid = elements[0].attr("itemid").unwrap();
    assert_eq!(itemid, "https://external-api.com/data/#person123");
    // NOT "https://myapp.com/dashboard#person123"
    
    Ok(elements)
}
```

This behavior ensures that microdata maintains its semantic integrity and correct identification regardless of where it's being rendered. The dom_query crate preserves the document context, allowing the html_template library to correctly resolve base URIs for each element.

## Builder Pattern API

The Rust version can use a fluent builder pattern for configuration:

```rust
use html_template::{HtmlTemplate, CacheMode};
use dom_query::Document;

let doc = Document::from(html);
let template = doc.select("template").first().unwrap();

let tmpl = HtmlTemplate::builder()
    .from_element(template)
    .selector("div.content")
    .cache_mode(CacheMode::Aggressive)
    .register_handler::<CustomHandler>()
    .build()?;
```

## Error Handling

Rust's Result type provides explicit error handling:

```rust
use html_template::{HtmlTemplate, Error};
use dom_query::Document;
use serde_json::Value;

fn render_template(html: &str, data: &Value) -> Result<String, Error> {
    let doc = Document::from(html);
    let template = doc.select("template").first()
        .ok_or(Error::ParseError("No template element found".into()))?;
    
    let tmpl = HtmlTemplate::from_element(template, None)
        .map_err(|e| Error::ParseError(e.to_string()))?;
    
    let element = tmpl.render(data)
        .map_err(|e| Error::RenderError(e.to_string()))?;
    
    Ok(element.outer_html())
}

// Error types
pub enum Error {
    ParseError(String),
    RenderError(String),
    SelectorError(String),
    ConstraintError(String),
    DomQueryError(dom_query::Error),
}

impl From<dom_query::Error> for Error {
    fn from(err: dom_query::Error) -> Self {
        Error::DomQueryError(err)
    }
}
```

## Performance Features

```rust
// Pre-compiled templates
let compiled = tmpl.compile()?;
let rendered = compiled.render(&data)?;

// Streaming render for large datasets
let stream = tmpl.render_stream(data_iterator);
for element in stream {
    let element = element?;
    // Process each element
}

// Zero-copy rendering where possible
let tmpl = HtmlTemplate::from_str(html)
    .zero_copy(true)
    .build()?;
```

## Testing Utilities

```rust
#[cfg(test)]
mod tests {
    use html_template::test_utils::{assert_html_eq, normalize_html};
    
    #[test]
    fn test_person_render() {
        let tmpl = HtmlTemplate::from_str(TEMPLATE).build().unwrap();
        let rendered = tmpl.render(&person).unwrap();
        
        assert_html_eq!(
            rendered.outer_html(),
            r#"<div itemscope itemtype="https://schema.org/Person">
                <h1>John Doe</h1>
            </div>"#
        );
    }
}
```

## Cargo.toml Dependencies

```toml
[dependencies]
html-template = "0.1.0"
dom_query = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
pretty_assertions = "1.4"
```

## Usage Example

```rust
use html_template::HtmlTemplate;
use dom_query::Document;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct PageData {
    title: String,
    posts: Vec<Post>,
}

#[derive(Serialize, Deserialize)]
struct Post {
    title: String,
    author: String,
    content: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template_html = std::fs::read_to_string("template.html")?;
    
    // Parse the HTML document
    let doc = Document::from(&template_html);
    let template = doc.select("template").first()
        .ok_or("No template element found")?;
    
    // Create the template renderer
    let tmpl = HtmlTemplate::from_element(template, "main")?;
    
    let data = PageData {
        title: "My Blog".to_string(),
        posts: vec![
            Post {
                title: "First Post".to_string(),
                author: "Jane".to_string(),
                content: "Hello world!".to_string(),
            },
        ],
    };
    
    let rendered = tmpl.render(&data)?;
    println!("{}", rendered.outer_html());
    
    Ok(())
}
```

## Alternative API: Direct from File

```rust
use html_template::HtmlTemplate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse template in one step
    let tmpl = HtmlTemplate::from_file("template.html", "main")?;
    
    let data = /* ... */;
    let rendered = tmpl.render(&data)?;
    
    Ok(())
}
```