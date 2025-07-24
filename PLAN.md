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

- [x] **1.1 Initialize Project**
  - [x] Run `cargo init --lib`
  - [x] Create workspace structure
  - [x] Set up initial `Cargo.toml` with all dependencies
  - [x] Create `macros` subdirectory for derive macros
  - [x] Configure workspace in root `Cargo.toml`

- [x] **1.2 Error Handling System**
  - [x] Create `src/error.rs`
  - [x] Define `Error` enum with all variants
  - [x] Implement `Display`, `Debug`, and `std::error::Error` traits
  - [x] Add `From` implementations for external errors
  - [x] Define `Result<T>` type alias
  - [x] Write unit tests for error conversions

- [x] **1.3 Core Types and Traits**
  - [x] Create `src/types.rs`
  - [x] Define `HtmlTemplate` struct
  - [x] Define `CompiledTemplate` struct
  - [x] Define `TemplateElement` struct
  - [x] Define `Property` and `PropertyTarget` types
  - [x] Define `Variable` struct
  - [x] Define `TemplateConfig` struct
  - [x] Define `CacheMode` enum
  - [x] Define `Constraint` and `ConstraintType` types
  - [x] Write unit tests for type construction

- [x] **1.4 RenderValue Trait**
  - [x] Create `src/value.rs`
  - [x] Define `RenderValue` trait
  - [x] Implement for `serde_json::Value`
  - [x] Implement for `String` and `&str`
  - [x] Implement for numeric types
  - [x] Implement for `Vec<T>` where `T: RenderValue`
  - [x] Write comprehensive tests

- [x] **1.5 Element Handler Trait**
  - [x] Define `ElementHandler` trait in `src/handlers.rs`
  - [x] Add async-trait support (decided to use sync instead)
  - [x] Define handler registration mechanism
  - [x] Write tests for trait implementation

### Phase 2: Template Parsing Engine

- [x] **2.1 Parser Module**
  - [x] Create `src/parser.rs`
  - [x] Implement template extraction from DOM
  - [x] Parse elements with `itemprop` attributes
  - [x] Detect array properties (ending with `[]`)
  - [x] Extract `itemscope` boundaries
  - [x] Extract `itemtype` values
  - [x] Parse `data-scope` attributes
  - [x] Parse `data-constraint` attributes
  - [x] Build element hierarchy
  - [x] Write unit tests for each parsing function

- [x] **2.2 Variable Parsing**
  - [x] Implement regex for `${variable}` extraction
  - [x] Parse simple variables (`${name}`)
  - [x] Parse nested paths (`${user.name}`)
  - [x] Parse array access (`${items[0]}`)
  - [x] Parse complex paths (`${users[0].address.street}`)
  - [x] Handle escaping (`$${literal}`) - Note: Not implemented as not in spec
  - [x] Extract variables from text content
  - [x] Extract variables from attributes
  - [x] Write comprehensive tests

- [x] **2.3 Template Compilation**
  - [x] Create `src/compiler.rs`
  - [x] Implement `compile` function
  - [x] Pre-compile CSS selectors
  - [x] Build property lookup tables
  - [x] Optimize variable paths
  - [x] Create element index
  - [x] Build constraint dependency graph
  - [x] Write performance benchmarks - Note: Basic tests written, full benchmarks in Phase 8.5

### Phase 3: Data Binding and Rendering

- [x] **3.1 Basic Renderer**
  - [x] Create `src/renderer.rs`
  - [x] Implement `render` method
  - [x] Handle single element rendering
  - [x] Apply text content bindings
  - [x] Apply attribute bindings
  - [x] Handle missing properties gracefully
  - [x] Write tests for basic rendering

- [x] **3.2 Array Rendering**
  - [x] Implement array detection
  - [x] Clone template elements for arrays
  - [x] Handle array property binding
  - [x] Remove array indicators from output
  - [x] Support nested arrays
  - [x] Write tests for array scenarios

- [x] **3.3 Nested Object Rendering**
  - [x] Handle `itemscope` elements
  - [x] Implement scoped data extraction
  - [x] Support recursive rendering
  - [x] Maintain proper data context
  - [x] Write tests for nested objects

- [x] **3.4 Variable Substitution**
  - [x] Implement variable resolution
  - [x] Support all path formats
  - [x] Handle undefined variables
  - [x] Implement default values (empty string)
  - [x] Write comprehensive tests

### Phase 4: Advanced Features

- [x] **4.1 Constraint System**
  - [x] Create `src/constraints.rs`
  - [x] Parse constraint expressions
  - [x] Implement scope resolution
  - [x] Handle `@id` references (basic support)
  - [x] Support JSON-LD references (basic)
  - [x] Evaluate constraints during rendering
  - [x] Write tests for constraint logic

- [x] **4.2 Cross-Document Rendering**
  - [x] Implement `render_from_element`
  - [x] Extract microdata from DOM elements
  - [x] Preserve source document base URI (basic implementation)
  - [x] Handle `itemid` generation
  - [ ] Support async document fetching (deferred to Phase 5)
  - [ ] Implement proper URI resolution (deferred to Phase 5)
  - [x] Write integration tests

- [x] **4.3 Derive Macros**
  - [x] Create `macros/Cargo.toml`
  - [x] Set up proc-macro crate
  - [x] Implement `Renderable` derive
  - [x] Support field attributes
  - [x] Generate `RenderValue` implementation
  - [ ] Handle nested structs (deferred - requires trait bounds)
  - [ ] Support generic types (deferred - requires trait bounds)
  - [x] Write macro tests

### Phase 5: Performance Optimizations

- [x] **5.1 Streaming Renderer**
  - [x] Create `src/streaming.rs`
  - [x] Define `StreamingRenderer` struct
  - [x] Implement iterator-based rendering
  - [x] Handle backpressure through chunked processing
  - [x] Support async streams (with async feature flag)
  - [x] Write performance tests

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

- [ ] **5.4 Advanced Cross-Document Features**
  - [ ] Support async document fetching (deferred from Phase 4.2)
  - [ ] Implement proper URI resolution (deferred from Phase 4.2)
  - [ ] Add HTTP client integration for remote templates
  - [ ] Implement document caching for remote resources
  - [ ] Add retry logic and error handling for network requests
  - [ ] Write tests for async document operations

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

- [ ] **6.3 Advanced Derive Macro Features**
  - [ ] Handle nested structs in derive macro (requires trait bounds)
  - [ ] Support generic types in derive macro (requires trait bounds)
  - [ ] Implement automatic nested property access
  - [ ] Add support for complex nested rendering
  - [ ] Write tests for nested struct scenarios
  - [ ] Write tests for generic type scenarios

- [ ] **6.4 Public API**
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
  - [ ] Benchmark template compilation performance
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
4. **Week 4**: Complete Phase 4 items 4.1-4.3 (Constraints, Cross-doc, Macros)
5. **Week 5**: Complete Phase 5 (Performance Optimizations including deferred async features)
6. **Week 6**: Complete Phase 6 (API Surface including advanced derive features)
7. **Week 7**: Complete Phase 7 (Element Handlers)
8. **Week 8**: Complete Phase 8 (Testing & Documentation)

## Progress Tracking

Current Status:
- **Started**: 2025-07-24
- **Current Phase**: Phase 5 Performance Optimizations
- **Last Completed Task**: 5.1 Streaming Renderer
- **Next Task**: 5.2 Zero-Copy Optimizations
- **Blockers**: None

### Implementation Notes from Phase 5.1 (Complete):
- Streaming renderer implemented with `StreamingRenderer` and `OwnedStreamingResult` structs
- Supports efficient chunked processing of large datasets without loading all data into memory
- Configurable buffer size for controlling memory usage vs processing efficiency  
- Iterator-based API that processes data items one chunk at a time
- `OwnedStreamingResult` avoids lifetime issues by owning template and data
- Comprehensive API: `collect_all()`, `next_chunk()`, `for_each_chunk()`, `write_to()`
- Async streaming support with feature flag (uses futures crate)
- Full test suite covering all streaming scenarios including chunked processing
- Integration with main `HtmlTemplate` API through convenience methods
- Note: Element handlers not supported in streaming mode (empty handlers map used)

### Implementation Notes from Phase 4.3 (Complete):
- Derive macro `#[derive(Renderable)]` implemented with comprehensive attribute support
- Supports field renaming with `#[renderable(rename = "newName")]`
- Field skipping with `#[renderable(skip)]` for sensitive data
- Automatic ID field detection and `#[renderable(id)]` attribute
- Generates complete `RenderValue` trait implementation
- Handles String, numeric, Option, and Vec field types
- Array detection for structs containing Vec fields
- Integration with template rendering system
- Comprehensive test suite and working example
- Note: Nested structs and generics deferred to Phase 6.3 (require trait bounds)

### Implementation Notes from Phase 4.2 (Complete):
- Cross-document rendering implemented with microdata extraction
- `render_from_element`, `render_from_html`, and `render_from_document` methods added
- Full microdata extraction supporting nested objects, arrays, and special elements
- Handles @type and @id attributes from itemtype and itemid
- Special element value extraction (meta[content], time[datetime], link[href], etc.)
- Comprehensive integration tests for cross-document scenarios
- Note: Async document fetching and URI resolution deferred to Phase 5.4
- Note: Array content population has known issue (tracked separately)

### Implementation Notes from Phase 4.1:
- Constraint system implemented with expression parser
- Supports equality, inequality, and comparison operators
- Property existence checks with truthy/falsy evaluation
- Basic scope constraint support
- Constraints are evaluated during rendering
- Elements that fail constraints are removed from output
- Comprehensive tests for constraint evaluation

### Implementation Notes from Phase 3 (Complete):
- Implemented basic rendering engine with full variable substitution
- Handle both root elements and descendants when matching selectors
- Support implicit variable binding (empty elements with itemprop get data)
- Process variables in both text content and attributes
- Missing data renders as empty strings
- Element finding works correctly for both root and nested elements
- Array rendering fully implemented with DOM cloning
- Arrays render by parsing template HTML for each item and processing all nested elements
- Variable processing handles elements without itemprop attributes
- Nested object rendering implemented with proper scope tracking
- `itemscope` elements change data context for their children
- Scoped children are rendered with the nested object data
- All rendering tests passing including complex nested scenarios

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