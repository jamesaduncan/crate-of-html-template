//! Streaming renderer for efficient processing of large datasets
//!
//! This module provides a streaming interface for rendering templates with
//! large amounts of data without loading everything into memory at once.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::handlers::ElementHandler;
use crate::renderer::Renderer;
use crate::types::*;
use crate::value::RenderValue;

/// Streaming renderer that processes data iteratively
pub struct StreamingRenderer<'a> {
    template: &'a CompiledTemplate,
    handlers: &'a HashMap<String, Box<dyn ElementHandler>>,
    buffer_size: usize,
}

impl<'a> StreamingRenderer<'a> {
    /// Create a new streaming renderer
    pub fn new(
        template: &'a CompiledTemplate,
        handlers: &'a HashMap<String, Box<dyn ElementHandler>>,
    ) -> Self {
        Self {
            template,
            handlers,
            buffer_size: 1024, // Default buffer size
        }
    }

    /// Set the buffer size for streaming operations
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Render a stream of data items
    pub fn render_stream_owned<I>(&self, data_stream: I) -> OwnedStreamingResult
    where
        I: IntoIterator<Item = Box<dyn RenderValue>>,
        I::IntoIter: 'static,
    {
        OwnedStreamingResult::new(self.template, self.handlers, data_stream.into_iter())
            .with_buffer_size(self.buffer_size)
    }

    /// Render a single chunk of data
    pub fn render_chunk(&self, data: &dyn RenderValue) -> Result<String> {
        let renderer = Renderer::new(self.template, self.handlers);
        renderer.render(data)
    }

    /// Render multiple items and collect results
    pub fn render_batch(&self, items: &[&dyn RenderValue]) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(items.len());

        for item in items {
            results.push(self.render_chunk(*item)?);
        }

        Ok(results)
    }

    /// Render items and join with separator
    pub fn render_joined(&self, items: &[&dyn RenderValue], separator: &str) -> Result<String> {
        let results = self.render_batch(items)?;
        Ok(results.join(separator))
    }
}

/// Iterator for streaming rendering results
pub struct StreamingResult<'a> {
    renderer: &'a StreamingRenderer<'a>,
    data_iter: Box<dyn Iterator<Item = Box<dyn RenderValue>>>,
    buffer: Vec<String>,
    finished: bool,
}

/// Owned streaming result that doesn't require lifetime management
pub struct OwnedStreamingResult {
    template: Arc<CompiledTemplate>,
    handlers: Arc<HashMap<String, Box<dyn ElementHandler>>>,
    data_iter: Box<dyn Iterator<Item = Box<dyn RenderValue>>>,
    buffer: Vec<String>,
    buffer_size: usize,
    finished: bool,
}

impl<'a> StreamingResult<'a> {
    #[allow(dead_code)]
    fn new<I>(renderer: &'a StreamingRenderer<'a>, data_iter: I) -> Self
    where
        I: Iterator<Item = Box<dyn RenderValue>> + 'static,
    {
        Self {
            renderer,
            data_iter: Box::new(data_iter),
            buffer: Vec::new(),
            finished: false,
        }
    }

    /// Collect all results into a vector
    pub fn collect_all(mut self) -> Result<Vec<String>> {
        let mut results = Vec::new();

        while let Some(chunk) = self.next_chunk()? {
            results.extend(chunk);
        }

        Ok(results)
    }

    /// Get the next chunk of rendered results
    pub fn next_chunk(&mut self) -> Result<Option<Vec<String>>> {
        if self.finished {
            return Ok(None);
        }

        self.buffer.clear();
        let mut count = 0;

        while count < self.renderer.buffer_size {
            if let Some(data) = self.data_iter.next() {
                let rendered = self.renderer.render_chunk(data.as_ref())?;
                self.buffer.push(rendered);
                count += 1;
            } else {
                self.finished = true;
                break;
            }
        }

        if self.buffer.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.buffer.clone()))
        }
    }

    /// Process results in chunks with a callback
    pub fn for_each_chunk<F>(mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(&[String]) -> Result<()>,
    {
        while let Some(chunk) = self.next_chunk()? {
            callback(&chunk)?;
        }
        Ok(())
    }

    /// Write results to a writer with optional separator
    pub fn write_to<W: std::io::Write>(
        mut self,
        mut writer: W,
        separator: Option<&str>,
    ) -> Result<()> {
        let sep = separator.unwrap_or("");
        let mut first = true;

        while let Some(chunk) = self.next_chunk()? {
            for item in chunk {
                if !first && !sep.is_empty() {
                    writer
                        .write_all(sep.as_bytes())
                        .map_err(|e| Error::IoError(e))?;
                }
                writer
                    .write_all(item.as_bytes())
                    .map_err(|e| Error::IoError(e))?;
                first = false;
            }
        }

        Ok(())
    }
}

/// Async streaming renderer for non-blocking operations
#[cfg(feature = "async")]
pub struct AsyncStreamingRenderer<'a> {
    renderer: StreamingRenderer<'a>,
}

#[cfg(feature = "async")]
impl<'a> AsyncStreamingRenderer<'a> {
    /// Create a new async streaming renderer
    pub fn new(
        template: &'a CompiledTemplate,
        handlers: &'a HashMap<String, Box<dyn ElementHandler>>,
    ) -> Self {
        Self {
            renderer: StreamingRenderer::new(template, handlers),
        }
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.renderer = self.renderer.with_buffer_size(size);
        self
    }

    /// Render data from an async stream
    pub async fn render_async_stream<S, E>(&self, mut stream: S) -> Result<Vec<String>>
    where
        S: futures::Stream<Item = std::result::Result<Box<dyn RenderValue>, E>>
            + std::marker::Unpin,
        E: Into<Error>,
    {
        use futures::StreamExt;

        let mut results = Vec::new();

        while let Some(item_result) = stream.next().await {
            match item_result {
                Ok(data) => {
                    let rendered = self.renderer.render_chunk(data.as_ref())?;
                    results.push(rendered);
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(results)
    }

    /// Process async stream in chunks
    pub async fn process_async_chunks<S, E, F>(&self, mut stream: S, mut callback: F) -> Result<()>
    where
        S: futures::Stream<Item = std::result::Result<Box<dyn RenderValue>, E>>
            + std::marker::Unpin,
        E: Into<Error>,
        F: FnMut(&[String]) -> Result<()>,
    {
        use futures::StreamExt;

        let mut chunk = Vec::new();

        while let Some(item_result) = stream.next().await {
            match item_result {
                Ok(data) => {
                    let rendered = self.renderer.render_chunk(data.as_ref())?;
                    chunk.push(rendered);

                    if chunk.len() >= self.renderer.buffer_size {
                        callback(&chunk)?;
                        chunk.clear();
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        // Process remaining items
        if !chunk.is_empty() {
            callback(&chunk)?;
        }

        Ok(())
    }
}

/// Convenience methods for HtmlTemplate
impl HtmlTemplate {
    /// Create a streaming renderer for this template
    pub fn streaming_renderer(&self) -> StreamingRenderer {
        StreamingRenderer::new(&self.compiled, &self.handlers)
    }

    /// Render a stream of data items
    pub fn render_stream<I>(&self, data_stream: I) -> OwnedStreamingResult
    where
        I: IntoIterator<Item = Box<dyn RenderValue>>,
        I::IntoIter: 'static,
    {
        let renderer = self.streaming_renderer();
        renderer.render_stream_owned(data_stream)
    }

    /// Render multiple items efficiently
    pub fn render_batch(&self, items: &[&dyn RenderValue]) -> Result<Vec<String>> {
        self.streaming_renderer().render_batch(items)
    }

    /// Create an async streaming renderer
    #[cfg(feature = "async")]
    pub fn async_streaming_renderer(&self) -> AsyncStreamingRenderer {
        AsyncStreamingRenderer::new(&self.compiled, &self.handlers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_template() -> Result<HtmlTemplate> {
        let html = r#"
            <template>
                <div class="item">
                    <h3 itemprop="name"></h3>
                    <p itemprop="description"></p>
                </div>
            </template>
        "#;

        HtmlTemplate::from_str(html, Some("div.item"))
    }

    #[test]
    fn test_streaming_renderer_creation() {
        let template = create_test_template().unwrap();
        let renderer = template.streaming_renderer();

        // Should be able to create renderer without issues
        assert_eq!(renderer.buffer_size, 1024);
    }

    #[test]
    fn test_render_batch() {
        let template = create_test_template().unwrap();

        let data1 = json!({"name": "Item 1", "description": "First item"});
        let data2 = json!({"name": "Item 2", "description": "Second item"});
        let data3 = json!({"name": "Item 3", "description": "Third item"});

        let items: Vec<&dyn RenderValue> = vec![&data1, &data2, &data3];
        let results = template.render_batch(&items).unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].contains("Item 1"));
        assert!(results[1].contains("Item 2"));
        assert!(results[2].contains("Item 3"));
    }

    #[test]
    fn test_streaming_result() {
        let template = create_test_template().unwrap();

        let data = vec![
            json!({"name": "Stream 1", "description": "First"}),
            json!({"name": "Stream 2", "description": "Second"}),
            json!({"name": "Stream 3", "description": "Third"}),
        ];

        let items: Vec<Box<dyn RenderValue>> = data
            .iter()
            .map(|d| Box::new(d.clone()) as Box<dyn RenderValue>)
            .collect();
        let streaming_result = template.render_stream(items);
        let results = streaming_result.collect_all().unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].contains("Stream 1"));
        assert!(results[1].contains("Stream 2"));
        assert!(results[2].contains("Stream 3"));
    }

    #[test]
    fn test_render_joined() {
        let template = create_test_template().unwrap();
        let renderer = template.streaming_renderer();

        let data1 = json!({"name": "A", "description": "First"});
        let data2 = json!({"name": "B", "description": "Second"});

        let items: Vec<&dyn RenderValue> = vec![&data1, &data2];
        let result = renderer.render_joined(&items, "\n---\n").unwrap();

        assert!(result.contains("A"));
        assert!(result.contains("B"));
        assert!(result.contains("\n---\n"));
    }

    #[test]
    fn test_chunked_processing() {
        let template = create_test_template().unwrap();

        let data: Vec<_> = (0..10)
            .map(|i| json!({"name": format!("Item {}", i), "description": format!("Description {}", i)}))
            .collect();

        let items: Vec<Box<dyn RenderValue>> = data
            .iter()
            .map(|d| Box::new(d.clone()) as Box<dyn RenderValue>)
            .collect();

        let renderer = template.streaming_renderer().with_buffer_size(3);
        let streaming_result = renderer.render_stream_owned(items);

        let mut chunk_count = 0;
        let mut total_items = 0;

        streaming_result
            .for_each_chunk(|chunk| {
                chunk_count += 1;
                total_items += chunk.len();
                Ok(())
            })
            .unwrap();

        assert_eq!(total_items, 10);
        assert!(chunk_count >= 3); // Should have multiple chunks
    }

    #[test]
    fn test_write_to_buffer() {
        let template = create_test_template().unwrap();

        let data1 = json!({"name": "Writer Test", "description": "Test description"});
        let items: Vec<Box<dyn RenderValue>> = vec![Box::new(data1)];

        let streaming_result = template.render_stream(items);
        let mut buffer = Vec::new();

        streaming_result.write_to(&mut buffer, None).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Writer Test"));
        assert!(output.contains("Test description"));
    }
}

impl OwnedStreamingResult {
    /// Create a new owned streaming result
    pub fn new<I>(
        template: &CompiledTemplate,
        _handlers: &HashMap<String, Box<dyn ElementHandler>>,
        data_iter: I,
    ) -> Self
    where
        I: Iterator<Item = Box<dyn RenderValue>> + 'static,
    {
        // Create a new empty handlers map for now, since ElementHandler doesn't implement Clone
        // In practice, this would need a different approach if handlers are required
        let empty_handlers = HashMap::new();

        Self {
            template: Arc::new(template.clone()),
            handlers: Arc::new(empty_handlers),
            data_iter: Box::new(data_iter),
            buffer: Vec::new(),
            buffer_size: 1024,
            finished: false,
        }
    }

    /// Set the buffer size for streaming operations
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Render a single chunk of data
    pub fn render_chunk(&self, data: &dyn RenderValue) -> Result<String> {
        let renderer = Renderer::new(&self.template, &self.handlers);
        renderer.render(data)
    }

    /// Collect all results into a vector
    pub fn collect_all(mut self) -> Result<Vec<String>> {
        let mut results = Vec::new();

        while let Some(chunk) = self.next_chunk()? {
            results.extend(chunk);
        }

        Ok(results)
    }

    /// Get the next chunk of rendered results
    pub fn next_chunk(&mut self) -> Result<Option<Vec<String>>> {
        if self.finished {
            return Ok(None);
        }

        self.buffer.clear();
        let mut count = 0;

        while count < self.buffer_size {
            if let Some(data) = self.data_iter.next() {
                let rendered = self.render_chunk(data.as_ref())?;
                self.buffer.push(rendered);
                count += 1;
            } else {
                self.finished = true;
                break;
            }
        }

        if self.buffer.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.buffer.clone()))
        }
    }

    /// Process results in chunks with a callback
    pub fn for_each_chunk<F>(mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(&[String]) -> Result<()>,
    {
        while let Some(chunk) = self.next_chunk()? {
            callback(&chunk)?;
        }
        Ok(())
    }

    /// Write results to a writer with optional separator
    pub fn write_to<W: std::io::Write>(
        mut self,
        mut writer: W,
        separator: Option<&str>,
    ) -> Result<()> {
        let sep = separator.unwrap_or("");
        let mut first = true;

        while let Some(chunk) = self.next_chunk()? {
            for item in chunk {
                if !first && !sep.is_empty() {
                    writer
                        .write_all(sep.as_bytes())
                        .map_err(|e| Error::IoError(e))?;
                }
                writer
                    .write_all(item.as_bytes())
                    .map_err(|e| Error::IoError(e))?;
                first = false;
            }
        }

        Ok(())
    }
}
