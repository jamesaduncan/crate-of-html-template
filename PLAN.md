# HTML Template Crate Implementation Plan

This document contains the comprehensive implementation plan for the `html-template` Rust crate, including all technical decisions, implementation details, and considerations needed to resume work at any point.

## Project Overview

A Rust implementation of an HTML templating library that uses microdata attributes (itemprop/itemtype/itemscope) for data binding. The library provides type-safe, performant HTML template rendering with support for:

- Microdata-based template binding
- Array handling with element cloning
- Nested object structures
- Variable substitution in text and attributes
- Cross-document rendering
- Streaming and zero-copy optimizations
- Custom element handlers
- Derive macros for type safety

## Core Design Principles

1. **Complete Implementation**: No half-measures - every feature is fully implemented
2. **Type Safety**: Leverage Rust's type system for compile-time guarantees
3. **Performance**: Include optimizations from the start (streaming, zero-copy, caching)
4. **Extensibility**: Trait-based system for custom element handlers
5. **API Compatibility**: Mirror the JavaScript version's capabilities while being idiomatic Rust

## Technical Architecture

### Module Structure

```
src/
├── lib.rs              # Public API exports, crate documentation
├── error.rs            # Error types and implementations
├── types.rs            # Core types, traits, and enums
├── parser.rs           # HTML parsing and template extraction
├── compiler.rs         # Template compilation and optimization
├── renderer.rs         # Rendering engine implementation
├── value.rs            # RenderValue trait and implementations
├── handlers.rs         # Element handler trait and built-in handlers
├── constraints.rs      # Constraint parsing and evaluation
├── builder.rs          # Builder pattern API
├── cache.rs            # Caching implementations
├── streaming.rs        # Streaming renderer
├── utils.rs            # Internal utilities
├── test_utils.rs       # Public testing utilities
└── macros/
    └── lib.rs          # Derive macro implementations
```

### Key Dependencies

```toml
[dependencies]
dom_query = "0.7"                                    # HTML parsing and DOM manipulation
serde = { version = "1.0", features = ["derive"] }  # Serialization framework
serde_json = "1.0"                                   # JSON support
reqwest = { version = "0.11", features = ["json"] } # HTTP client for cross-document
async-trait = "0.1"                                  # Async trait support
thiserror = "1.0"                                    # Error derive macros
once_cell = "1.0"                                    # Lazy statics for caching
regex = "1.0"                                        # Variable parsing
indexmap = "2.0"                                     # Ordered hashmaps

[dependencies.html-template-macros]
path = "macros"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }      # Async runtime for tests
pretty_assertions = "1.4"                            # Better test assertions
criterion = "0.5"                                    # Benchmarking
```

### Core Types and Traits

```rust
// Primary template type
pub struct HtmlTemplate {
    compiled: Arc<CompiledTemplate>,
    config: TemplateConfig,
    handlers: HashMap<String, Box<dyn ElementHandler>>,
}

// Compiled template for performance
pub struct CompiledTemplate {
    root_selector: Option<String>,
    elements: Vec<TemplateElement>,
    constraints: Vec<Constraint>,
    base_uri: Option<String>,
}

// Template element representation
pub struct TemplateElement {
    selector: String,
    properties: Vec<Property>,
    is_array: bool,
    is_scope: bool,
    itemtype: Option<String>,
    constraints: Vec<ConstraintRef>,
}

// Property binding
pub struct Property {
    name: String,
    is_array: bool,
    target: PropertyTarget,
    variables: Vec<Variable>,
}

// Where to bind the property
pub enum PropertyTarget {
    TextContent,
    Attribute(String),
    Value, // for input elements
}

// Variable in templates
pub struct Variable {
    path: Vec<String>, // e.g., ["user", "name"]
    raw: String,       // original ${...} string
}

// Configuration
pub struct TemplateConfig {
    cache_mode: CacheMode,
    zero_copy: bool,
}

// Main trait for renderable values
pub trait RenderValue {
    fn get_property(&self, path: &[String]) -> Option<Cow<str>>;
    fn is_array(&self) -> bool;
    fn as_array(&self) -> Option<Vec<&dyn RenderValue>>;
    fn get_type(&self) -> Option<&str>;
    fn get_id(&self) -> Option<&str>;
}

// Element handler trait
#[async_trait]
pub trait ElementHandler: Send + Sync {
    fn can_handle(&self, element: &Element) -> bool;
    async fn handle(&self, element: &mut Element, value: &dyn RenderValue) -> Result<()>;
}
```

## Implementation Checklist

### Phase 1: Project Setup and Core Infrastructure

- [ ] **1.1 Initialize Project**
  - [ ] Run `cargo init --lib`
  - [ ] Create workspace structure
  - [ ] Set up initial `Cargo.toml` with all dependencies
  - [ ] Create `macros` subdirectory for derive macros
  - [ ] Configure workspace in root `Cargo.toml`

- [ ] **1.2 Error Handling System**
  - [ ] Create `src/error.rs`
  - [ ] Define `Error` enum with all variants
  - [ ] Implement `Display`, `Debug`, and `std::error::Error` traits
  - [ ] Add `From` implementations for external errors
  - [ ] Define `Result<T>` type alias
  - [ ] Write unit tests for error conversions

- [ ] **1.3 Core Types and Traits**
  - [ ] Create `src/types.rs`
  - [ ] Define `HtmlTemplate` struct
  - [ ] Define `CompiledTemplate` struct
  - [ ] Define `TemplateElement` struct
  - [ ] Define `Property` and `PropertyTarget` types
  - [ ] Define `Variable` struct
  - [ ] Define `TemplateConfig` struct
  - [ ] Define `CacheMode` enum
  - [ ] Define `Constraint` and `ConstraintType` types
  - [ ] Write unit tests for type construction

- [ ] **1.4 RenderValue Trait**
  - [ ] Create `src/value.rs`
  - [ ] Define `RenderValue` trait
  - [ ] Implement for `serde_json::Value`
  - [ ] Implement for `String` and `&str`
  - [ ] Implement for numeric types
  - [ ] Implement for `Vec<T>` where `T: RenderValue`
  - [ ] Write comprehensive tests

- [ ] **1.5 Element Handler Trait**
  - [ ] Define `ElementHandler` trait in `src/handlers.rs`
  - [ ] Add async-trait support
  - [ ] Define handler registration mechanism
  - [ ] Write tests for trait implementation

### Phase 2: Template Parsing Engine

- [ ] **2.1 Parser Module**
  - [ ] Create `src/parser.rs`
  - [ ] Implement template extraction from DOM
  - [ ] Parse elements with `itemprop` attributes
  - [ ] Detect array properties (ending with `[]`)
  - [ ] Extract `itemscope` boundaries
  - [ ] Extract `itemtype` values
  - [ ] Parse `data-scope` attributes
  - [ ] Parse `data-constraint` attributes
  - [ ] Build element hierarchy
  - [ ] Write unit tests for each parsing function

- [ ] **2.2 Variable Parsing**
  - [ ] Implement regex for `${variable}` extraction
  - [ ] Parse simple variables (`${name}`)
  - [ ] Parse nested paths (`${user.name}`)
  - [ ] Parse array access (`${items[0]}`)
  - [ ] Parse complex paths (`${users[0].address.street}`)
  - [ ] Handle escaping (`$${literal}`)
  - [ ] Extract variables from text content
  - [ ] Extract variables from attributes
  - [ ] Write comprehensive tests

- [ ] **2.3 Template Compilation**
  - [ ] Create `src/compiler.rs`
  - [ ] Implement `compile` function
  - [ ] Pre-compile CSS selectors
  - [ ] Build property lookup tables
  - [ ] Optimize variable paths
  - [ ] Create element index
  - [ ] Build constraint dependency graph
  - [ ] Write performance benchmarks

### Phase 3: Data Binding and Rendering

- [ ] **3.1 Basic Renderer**
  - [ ] Create `src/renderer.rs`
  - [ ] Implement `render` method
  - [ ] Handle single element rendering
  - [ ] Apply text content bindings
  - [ ] Apply attribute bindings
  - [ ] Handle missing properties gracefully
  - [ ] Write tests for basic rendering

- [ ] **3.2 Array Rendering**
  - [ ] Implement array detection
  - [ ] Clone template elements for arrays
  - [ ] Handle array property binding
  - [ ] Remove array indicators from output
  - [ ] Support nested arrays
  - [ ] Write tests for array scenarios

- [ ] **3.3 Nested Object Rendering**
  - [ ] Handle `itemscope` elements
  - [ ] Implement scoped data extraction
  - [ ] Support recursive rendering
  - [ ] Maintain proper data context
  - [ ] Write tests for nested objects

- [ ] **3.4 Variable Substitution**
  - [ ] Implement variable resolution
  - [ ] Support all path formats
  - [ ] Handle undefined variables
  - [ ] Implement default values
  - [ ] Write comprehensive tests

### Phase 4: Advanced Features

- [ ] **4.1 Constraint System**
  - [ ] Create `src/constraints.rs`
  - [ ] Parse constraint expressions
  - [ ] Implement scope resolution
  - [ ] Handle `@id` references
  - [ ] Support JSON-LD references
  - [ ] Evaluate constraints during rendering
  - [ ] Write tests for constraint logic

- [ ] **4.2 Cross-Document Rendering**
  - [ ] Implement `render_from_element`
  - [ ] Extract microdata from DOM elements
  - [ ] Preserve source document base URI
  - [ ] Handle `itemid` generation
  - [ ] Support async document fetching
  - [ ] Implement proper URI resolution
  - [ ] Write integration tests

- [ ] **4.3 Derive Macros**
  - [ ] Create `macros/Cargo.toml`
  - [ ] Set up proc-macro crate
  - [ ] Implement `Renderable` derive
  - [ ] Support field attributes
  - [ ] Generate `RenderValue` implementation
  - [ ] Handle nested structs
  - [ ] Support generic types
  - [ ] Write macro tests

### Phase 5: Performance Optimizations

- [ ] **5.1 Streaming Renderer**
  - [ ] Create `src/streaming.rs`
  - [ ] Define `StreamingRenderer` struct
  - [ ] Implement iterator-based rendering
  - [ ] Handle backpressure
  - [ ] Support async streams
  - [ ] Write performance tests

- [ ] **5.2 Zero-Copy Optimizations**
  - [ ] Use `Cow<str>` throughout
  - [ ] Minimize string allocations
  - [ ] Implement efficient cloning
  - [ ] Pool temporary allocations
  - [ ] Benchmark memory usage

- [ ] **5.3 Caching System**
  - [ ] Create `src/cache.rs`
  - [ ] Implement template cache
  - [ ] Add compiled template cache
  - [ ] Create selector cache
  - [ ] Add external document cache
  - [ ] Implement cache eviction
  - [ ] Write cache effectiveness tests

### Phase 6: API Surface

- [ ] **6.1 Builder Pattern**
  - [ ] Create `src/builder.rs`
  - [ ] Implement `HtmlTemplateBuilder`
  - [ ] Add `from_element` method
  - [ ] Add `from_str` method
  - [ ] Add `from_file` method
  - [ ] Add configuration methods
  - [ ] Add handler registration
  - [ ] Implement `build` method
  - [ ] Write builder tests

- [ ] **6.2 Direct Constructors**
  - [ ] Implement `HtmlTemplate::from_element`
  - [ ] Implement `HtmlTemplate::from_str`
  - [ ] Implement `HtmlTemplate::from_file`
  - [ ] Ensure API compatibility
  - [ ] Write tests for each constructor

- [ ] **6.3 Public API**
  - [ ] Design clean public API in `src/lib.rs`
  - [ ] Export necessary types
  - [ ] Hide implementation details
  - [ ] Add convenience methods
  - [ ] Document all public items

### Phase 7: Element Handlers

- [ ] **7.1 Built-in Handlers**
  - [ ] Implement `InputHandler`
  - [ ] Implement `SelectHandler`
  - [ ] Implement `TextareaHandler`
  - [ ] Implement `MetaHandler`
  - [ ] Register default handlers
  - [ ] Write tests for each handler

- [ ] **7.2 Custom Handler Support**
  - [ ] Implement handler registration system
  - [ ] Support handler priorities
  - [ ] Allow handler chaining
  - [ ] Document handler API
  - [ ] Create example custom handler

### Phase 8: Testing and Documentation

- [ ] **8.1 Test Utilities**
  - [ ] Create `src/test_utils.rs`
  - [ ] Implement `assert_html_eq`
  - [ ] Implement `normalize_html`
  - [ ] Add HTML comparison helpers
  - [ ] Export as public module
  - [ ] Write tests for utilities

- [ ] **8.2 Unit Tests**
  - [ ] Achieve >90% code coverage
  - [ ] Test error conditions
  - [ ] Test edge cases
  - [ ] Add property-based tests
  - [ ] Run tests in CI

- [ ] **8.3 Integration Tests**
  - [ ] Create `tests/` directory
  - [ ] Test complete rendering scenarios
  - [ ] Test cross-document rendering
  - [ ] Test async operations
  - [ ] Test performance characteristics
  - [ ] Add regression tests

- [ ] **8.4 Documentation**
  - [ ] Write module documentation
  - [ ] Document all public APIs
  - [ ] Create usage examples
  - [ ] Write performance guide
  - [ ] Add troubleshooting section
  - [ ] Generate and review rustdoc

- [ ] **8.5 Benchmarks**
  - [ ] Create `benches/` directory
  - [ ] Benchmark parsing performance
  - [ ] Benchmark rendering performance
  - [ ] Benchmark memory usage
  - [ ] Compare with/without optimizations
  - [ ] Document performance characteristics

### Phase 9: Final Steps

- [ ] **9.1 Code Quality**
  - [ ] Run `cargo fmt`
  - [ ] Run `cargo clippy` with all lints
  - [ ] Fix all warnings
  - [ ] Review code for idiomaticity
  - [ ] Ensure consistent style

- [ ] **9.2 Release Preparation**
  - [ ] Update `Cargo.toml` metadata
  - [ ] Write comprehensive README
  - [ ] Create CHANGELOG
  - [ ] Add LICENSE files
  - [ ] Prepare for crates.io publication

## Implementation Order

The recommended order for implementation:

1. **Week 1**: Complete Phase 1 (Project Setup)
2. **Week 2**: Complete Phase 2 (Parsing Engine)
3. **Week 3**: Complete Phase 3 (Basic Rendering)
4. **Week 4**: Complete Phase 4 items 4.1-4.2 (Constraints & Cross-doc)
5. **Week 5**: Complete Phase 4 item 4.3 and Phase 5 (Macros & Performance)
6. **Week 6**: Complete Phases 6-7 (API & Handlers)
7. **Week 7**: Complete Phase 8 (Testing & Documentation)

## Progress Tracking

Current Status:
- **Started**: _(Date)_
- **Current Phase**: _Phase X_
- **Last Completed Task**: _Task X.Y_
- **Next Task**: _Task X.Y_
- **Blockers**: _None_

## Notes for Resuming Work

When resuming:
1. Check the last completed checkbox
2. Read the technical notes for current phase
3. Run `cargo test` to verify current state
4. Continue with next unchecked item
5. Update progress tracking section

## Derive Macro Implementation

Create separate crate `html-template-macros`:

```rust
#[proc_macro_derive(Renderable, attributes(renderable))]
pub fn derive_renderable(input: TokenStream) -> TokenStream {
    // Generate RenderValue implementation
}
```

Features:
- Automatic property extraction
- Support for nested structs
- Field renaming via attributes
- Array handling

## Critical Implementation Notes

### Variable Parsing

Use regex to extract variables:
```rust
lazy_static! {
    static ref VAR_REGEX: Regex = Regex::new(r"\$\{([^}]+)\}").unwrap();
}
```

### Property Path Resolution

Support:
- Simple paths: `name`
- Nested paths: `user.name`
- Array access: `items[0]`
- Mixed: `users[0].address.street`

### DOM Manipulation

Always use `dom_query` methods:
- Element cloning for arrays
- Attribute manipulation
- Text content updates
- Child element queries

### Memory Management

- Use `Arc` for shared compiled templates
- Clone only when necessary
- Pool temporary allocations
- Cache aggressively but with limits

## Progress Tracking

When resuming work:
1. Check which phase was last completed
2. Run existing tests to verify working state
3. Continue with next item in the phase
4. Update tests for new functionality
5. Document any API changes

## Next Steps

1. Create project structure
2. Implement error types
3. Define core traits
4. Begin parser implementation
5. Write initial tests

This plan provides everything needed to implement a complete, production-ready HTML templating library with all requested features.