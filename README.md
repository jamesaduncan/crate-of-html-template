# html-template

A Rust HTML templating library using microdata attributes for data binding with support for arrays, constraints, and streaming.

[![Crates.io](https://img.shields.io/crates/v/html-template.svg)](https://crates.io/crates/html-template)
[![Documentation](https://docs.rs/html-template/badge.svg)](https://docs.rs/html-template)
[![License](https://img.shields.io/crates/l/html-template.svg)](LICENSE)

## Features

- **Microdata-based templating**: Uses HTML5 microdata attributes (`itemprop`, `itemscope`, `itemtype`) for data binding
- **Array support**: Automatic cloning of template elements for array data
- **Nested objects**: Support for nested data structures with `itemscope`
- **Constraint system**: Conditional rendering with `data-constraint` expressions
- **Variable substitution**: Replace `${variable}` syntax in text and attributes
- **Performance optimized**: Template compilation with caching support
- **Streaming support**: Process large datasets efficiently with streaming API
- **Thread-safe**: Templates can be shared across threads
- **Schema.org integration**: Built-in support for Schema.org vocabulary
- **Zero-copy optimizations**: Minimize allocations where possible

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
html-template = "0.1"
```

### Basic Usage

```rust
use html_template::HtmlTemplate;
use serde_json::json;

let html = r#"
<template>
    <div>
        <h1 itemprop="title">${title}</h1>
        <p itemprop="content">${content}</p>
    </div>
</template>
"#;

let template = HtmlTemplate::from_str(html, Some("div"))?;
let data = json!({
    "title": "Hello World",
    "content": "This is a test."
});

let result = template.render(&data)?;
println!("{}", result);
```

### Array Rendering

```rust
use html_template::HtmlTemplate;
use serde_json::json;

let html = r#"
<template>
    <ul>
        <li itemprop="items[]">
            <span itemprop="name">${name}</span>
        </li>
    </ul>
</template>
"#;

let template = HtmlTemplate::from_str(html, Some("li"))?;
let data = json!({
    "items": [
        {"name": "Item 1"},
        {"name": "Item 2"},
        {"name": "Item 3"}
    ]
});

let result = template.render(&data)?;
// Renders three <li> elements
```

### Constraints

```rust
use html_template::HtmlTemplate;
use serde_json::json;

let html = r#"
<template>
    <div>
        <h1 itemprop="title">${title}</h1>
        <p itemprop="description" data-constraint="hasDescription">${description}</p>
        <span data-constraint="!hasDescription">No description available</span>
    </div>
</template>
"#;

let template = HtmlTemplate::from_str(html, Some("div"))?;
let data = json!({
    "title": "Test Article",
    "hasDescription": true,
    "description": "This article has a description."
});

let result = template.render(&data)?;
```

### Nested Objects

```rust
use html_template::HtmlTemplate;
use serde_json::json;

let html = r#"
<template>
    <article itemscope itemtype="http://schema.org/Article">
        <h1 itemprop="headline">${headline}</h1>
        <div itemprop="author" itemscope itemtype="http://schema.org/Person">
            <span itemprop="name">${author.name}</span>
            <span itemprop="email">${author.email}</span>
        </div>
    </article>
</template>
"#;

let template = HtmlTemplate::from_str(html, Some("article"))?;
let data = json!({
    "headline": "Breaking News",
    "author": {
        "name": "Jane Doe",
        "email": "jane@example.com"
    }
});

let result = template.render(&data)?;
```

## Advanced Features

### Performance Configuration

```rust
use html_template::{HtmlTemplate, TemplateConfig, CacheMode};

let config = TemplateConfig::default()
    .with_cache_mode(CacheMode::Aggressive)
    .with_zero_copy(true);

let template = HtmlTemplate::from_str_with_config(html, Some("div"), config)?;
```

### Streaming for Large Datasets

```rust
use html_template::HtmlTemplate;
use serde_json::json;

let template = HtmlTemplate::from_str(html, Some("div"))?;

// Create data stream
let data_stream = vec![
    Box::new(json!({"name": "Item 1"})) as Box<dyn RenderValue>,
    Box::new(json!({"name": "Item 2"})) as Box<dyn RenderValue>,
    // ... many more items
];

let streaming_result = template.render_stream(data_stream);
let results = streaming_result.collect_all()?;
```

### Custom Data Types

Implement the `RenderValue` trait for your custom types:

```rust
use html_template::RenderValue;
use std::borrow::Cow;

struct MyData {
    name: String,
    count: i32,
}

impl RenderValue for MyData {
    fn get_property(&self, path: &[String]) -> Option<Cow<str>> {
        match path.get(0)?.as_str() {
            "name" => Some(Cow::Borrowed(&self.name)),
            "count" => Some(Cow::Owned(self.count.to_string())),
            _ => None,
        }
    }
    
    fn is_array(&self) -> bool { false }
    fn as_array(&self) -> Option<Vec<&dyn RenderValue>> { None }
    fn get_type(&self) -> Option<&str> { None }
    fn get_id(&self) -> Option<&str> { None }
}
```

## Template Syntax

### Microdata Attributes

- `itemprop="property"` - Bind data property to element
- `itemprop="property[]"` - Bind array property (clones element for each item)
- `itemscope` - Define object boundary for nested data
- `itemtype="http://schema.org/Type"` - Schema.org type information

### Variable Substitution

- `${variable}` - Replace with data property value
- `${nested.property}` - Access nested object properties
- `${array[0]}` - Access array elements by index

### Constraints

- `data-constraint="property"` - Show if property exists and is truthy
- `data-constraint="!property"` - Show if property doesn't exist or is falsy
- `data-constraint="property == 'value'"` - Show if property equals value
- `data-constraint="property > 5"` - Show if property is greater than 5

## Performance

The library is optimized for performance with several key features:

- **Template compilation**: Templates are parsed once and reused
- **Caching**: Compiled templates can be cached globally or per-instance
- **Zero-copy optimizations**: Minimize string allocations where possible
- **Streaming support**: Process large datasets without loading everything into memory
- **Thread-safe sharing**: Templates can be shared across threads efficiently

Benchmarks show significant performance improvements with caching enabled:

```
Template compilation (no cache):     ~100μs
Template compilation (with cache):   ~1.5μs (60x faster)
Template rendering:                  ~10-50μs depending on complexity
```

## Architecture

The library is structured around several core components:

- **Parser**: Extracts microdata and variables from HTML templates
- **Compiler**: Converts parsed templates into efficient internal representation
- **Renderer**: Applies data to compiled templates to generate output
- **Cache**: Stores compiled templates for reuse
- **Value trait**: Provides unified interface for different data types
- **Constraints**: Evaluates conditional expressions for element visibility

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.