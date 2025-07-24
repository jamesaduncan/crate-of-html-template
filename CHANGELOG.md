# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-24

### Added

#### Core Features
- HTML templating using microdata attributes (`itemprop`, `itemscope`, `itemtype`)
- Variable substitution with `${variable}` syntax in text content and attributes
- Array support with automatic element cloning for `itemprop="property[]"`
- Nested object support with `itemscope` boundaries
- Schema.org integration with `itemtype` attributes
- Constraint system for conditional rendering with `data-constraint` expressions

#### Performance Features
- Template compilation with caching support (global and instance-level)
- Zero-copy optimizations to minimize allocations
- Thread-safe template sharing
- Streaming API for processing large datasets efficiently
- Multiple cache modes (None, Normal, Aggressive)

#### API Surface
- `HtmlTemplate` - Main template type with multiple creation methods
- `TemplateConfig` - Configuration options for performance and behavior
- `RenderValue` trait - Unified interface for data types
- Built-in implementations for JSON, strings, numbers, arrays
- `SerializeWrapper` for automatic serde integration

#### Advanced Features
- Custom element handlers for specialized rendering logic
- Constraint evaluation with comparison operators (`==`, `!=`, `>`, `<`, etc.)
- Logical operators in constraints (`&&`, `||`)
- Negation operator (`!`) for constraints
- Single and double-quoted string literals in constraints
- Microdata extraction from existing HTML documents

#### Data Types Support
- `serde_json::Value` with full JSON support
- Primitive types (String, numbers, booleans)
- Collections (Vec, HashMap)
- Custom types via `RenderValue` trait implementation

#### Streaming Support
- `StreamingRenderer` for batch processing
- `OwnedStreamingResult` for iterator-based processing
- Chunked processing with configurable buffer sizes
- Writer integration for direct output streaming
- Optional async support (behind feature flag)

#### Performance Optimizations
- Template caching with 60x performance improvement
- String pooling for memory efficiency  
- Regex caching to avoid recompilation
- Copy-on-write strings to minimize allocations
- Thread-local storage for reusable buffers

#### Documentation and Testing
- Comprehensive documentation with examples
- Full test coverage across all modules
- Performance benchmarks using Criterion
- Integration tests for real-world scenarios
- API documentation with usage examples

### Technical Details

#### Dependencies
- `dom_query` 0.7 - HTML parsing and DOM manipulation
- `serde` 1.0 - Serialization support  
- `serde_json` 1.0 - JSON data handling
- `regex` 1.0 - Pattern matching for constraints
- `once_cell` 1.0 - Thread-safe initialization
- `thiserror` 1.0 - Error handling
- `indexmap` 2.0 - Ordered maps
- Optional: `futures` 0.3 - Async streaming support

#### Features
- `default` - Core functionality only
- `derive` - Includes derive macros (future expansion)
- `async` - Enables async streaming support

#### Performance Benchmarks
- Template compilation: ~100μs without cache, ~1.5μs with cache
- Template rendering: ~10-50μs depending on complexity
- Memory usage: Optimized with string pooling and copy-on-write
- Thread safety: Templates can be shared across threads efficiently

### Internal Architecture

#### Module Structure
- `lib.rs` - Public API and re-exports
- `template.rs` - Core `HtmlTemplate` implementation
- `parser.rs` - HTML parsing and template extraction
- `compiler.rs` - Template compilation to internal representation
- `renderer.rs` - Data binding and HTML generation
- `value.rs` - `RenderValue` trait and implementations
- `constraints.rs` - Constraint evaluation system
- `cache.rs` - Template caching implementation
- `streaming.rs` - Streaming renderer for large datasets
- `handlers.rs` - Custom element handler system
- `microdata.rs` - Microdata extraction utilities
- `types.rs` - Core type definitions
- `error.rs` - Error types and handling
- `utils.rs` - Performance utilities and helpers

#### Error Handling
- Comprehensive error types with descriptive messages
- Graceful handling of malformed HTML and invalid data
- Context-aware error reporting for debugging

### Breaking Changes
- None (initial release)

### Migration Guide
- None (initial release)

### Known Issues
- Element handlers are not cloned with cached templates (empty handlers map used)
- Some complex constraint expressions may need parentheses for proper precedence
- Async streaming requires `async` feature flag

### Future Plans
- Derive macros for automatic `RenderValue` implementation
- Additional constraint operators and functions
- Template inheritance and composition features
- Hot-reloading for development workflows
- WASM support for browser usage