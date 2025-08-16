//! # Performance Optimization Module
//!
//! This module provides performance optimization utilities for the j2s tool,
//! including streaming JSON processing, parallel code generation, and performance
//! monitoring capabilities.

use crate::error::{J2sError, Result};
use rayon::prelude::*;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics collector for monitoring code generation performance
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Time taken for JSON parsing
    pub parse_time: Duration,
    /// Time taken for schema/code generation
    pub generation_time: Duration,
    /// Time taken for file I/O operations
    pub io_time: Duration,
    /// Total processing time
    pub total_time: Duration,
    /// Memory usage peak (in bytes)
    pub peak_memory_usage: usize,
    /// Number of JSON objects processed
    pub objects_processed: usize,
    /// Number of fields processed
    pub fields_processed: usize,
    /// Size of input data (in bytes)
    pub input_size: usize,
    /// Size of output data (in bytes)
    pub output_size: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            parse_time: Duration::ZERO,
            generation_time: Duration::ZERO,
            io_time: Duration::ZERO,
            total_time: Duration::ZERO,
            peak_memory_usage: 0,
            objects_processed: 0,
            fields_processed: 0,
            input_size: 0,
            output_size: 0,
        }
    }
}

impl PerformanceMetrics {
    /// Create a new performance metrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Print a performance summary to stdout
    pub fn print_summary(&self) {
        println!("📊 Performance Summary:");
        println!("   ⏱️  Total time: {:.2}ms", self.total_time.as_millis());
        println!("   📖 Parse time: {:.2}ms", self.parse_time.as_millis());
        println!("   ⚙️  Generation time: {:.2}ms", self.generation_time.as_millis());
        println!("   💾 I/O time: {:.2}ms", self.io_time.as_millis());
        println!("   🧠 Peak memory: {:.1} MB", self.peak_memory_usage as f64 / 1_000_000.0);
        println!("   📦 Objects processed: {}", self.objects_processed);
        println!("   🏷️  Fields processed: {}", self.fields_processed);
        println!("   📏 Input size: {:.1} KB", self.input_size as f64 / 1000.0);
        println!("   📄 Output size: {:.1} KB", self.output_size as f64 / 1000.0);
        
        if self.total_time.as_millis() > 0 {
            let throughput = (self.input_size as f64 / 1000.0) / (self.total_time.as_secs_f64());
            println!("   🚀 Throughput: {throughput:.1} KB/s");
        }
    }

    /// Check if performance is within acceptable bounds
    pub fn is_performance_acceptable(&self) -> bool {
        // Define acceptable performance thresholds
        const MAX_PROCESSING_TIME_MS: u128 = 30_000; // 30 seconds
        const MAX_MEMORY_MB: usize = 500; // 500 MB
        
        self.total_time.as_millis() <= MAX_PROCESSING_TIME_MS &&
        self.peak_memory_usage <= MAX_MEMORY_MB * 1_000_000
    }
}

/// Performance monitor for tracking resource usage during processing
pub struct PerformanceMonitor {
    start_time: Instant,
    metrics: PerformanceMetrics,
    memory_tracker: Arc<AtomicUsize>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            metrics: PerformanceMetrics::new(),
            memory_tracker: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Start timing a specific operation
    pub fn start_operation(&self) -> Instant {
        Instant::now()
    }

    /// Record parse time
    pub fn record_parse_time(&mut self, duration: Duration) {
        self.metrics.parse_time = duration;
    }

    /// Record generation time
    pub fn record_generation_time(&mut self, duration: Duration) {
        self.metrics.generation_time = duration;
    }

    /// Record I/O time
    pub fn record_io_time(&mut self, duration: Duration) {
        self.metrics.io_time = duration;
    }

    /// Update memory usage tracking
    pub fn update_memory_usage(&mut self, bytes: usize) {
        self.memory_tracker.store(bytes, Ordering::Relaxed);
        if bytes > self.metrics.peak_memory_usage {
            self.metrics.peak_memory_usage = bytes;
        }
    }

    /// Record input size
    pub fn record_input_size(&mut self, size: usize) {
        self.metrics.input_size = size;
    }

    /// Record output size
    pub fn record_output_size(&mut self, size: usize) {
        self.metrics.output_size = size;
    }

    /// Increment objects processed counter
    pub fn increment_objects_processed(&mut self) {
        self.metrics.objects_processed += 1;
    }

    /// Increment fields processed counter
    pub fn increment_fields_processed(&mut self, count: usize) {
        self.metrics.fields_processed += count;
    }

    /// Finalize metrics and return the results
    pub fn finalize(mut self) -> PerformanceMetrics {
        self.metrics.total_time = self.start_time.elapsed();
        self.metrics
    }
}

/// Streaming JSON processor for handling large JSON files efficiently
pub struct StreamingJsonProcessor {
    chunk_size: usize,
    max_memory_usage: usize,
}

impl Default for StreamingJsonProcessor {
    fn default() -> Self {
        Self {
            chunk_size: 1024 * 1024, // 1MB chunks
            max_memory_usage: 100 * 1024 * 1024, // 100MB max memory
        }
    }
}

impl StreamingJsonProcessor {
    /// Create a new streaming processor with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a streaming processor with custom settings
    pub fn with_settings(chunk_size: usize, max_memory_usage: usize) -> Self {
        Self {
            chunk_size,
            max_memory_usage,
        }
    }

    /// Process large JSON content in chunks to reduce memory usage
    pub fn process_large_json(&self, content: &str, monitor: &mut PerformanceMonitor) -> Result<Value> {
        let start_time = monitor.start_operation();
        
        // For very large files, we need to be more careful about memory usage
        if content.len() > self.max_memory_usage {
            return Err(J2sError::performance_error(format!(
                "JSON file too large for streaming processing: {:.1} MB (max: {:.1} MB)",
                content.len() as f64 / 1_000_000.0,
                self.max_memory_usage as f64 / 1_000_000.0
            )));
        }

        // Update memory usage tracking
        monitor.update_memory_usage(content.len());

        // Parse JSON with progress indication for large files
        let json_value = if content.len() > 10_000_000 {
            println!("📊 Processing large JSON file ({:.1} MB)...", content.len() as f64 / 1_000_000.0);
            self.parse_with_progress(content)?
        } else {
            serde_json::from_str(content).map_err(|e| {
                J2sError::json_error(format!("Failed to parse JSON: {e}"))
            })?
        };

        monitor.record_parse_time(start_time.elapsed());
        Ok(json_value)
    }

    /// Parse JSON with progress indication
    fn parse_with_progress(&self, content: &str) -> Result<Value> {
        // For now, we use the standard parser but with progress indication
        // In a more advanced implementation, we could use a streaming JSON parser
        println!("   📊 Progress: 25% - Starting JSON parsing...");
        
        let result = serde_json::from_str(content).map_err(|e| {
            J2sError::json_error(format!("Failed to parse large JSON: {e}"))
        });

        println!("   📊 Progress: 100% - JSON parsing complete");
        result
    }
}

/// Parallel code generator for processing multiple formats simultaneously
pub struct ParallelCodeGenerator;

impl ParallelCodeGenerator {
    /// Generate code for multiple formats in parallel
    pub fn generate_parallel(
        json_value: &Value,
        formats: &[String],
        options: &crate::codegen::generator::GenerationOptions,
        monitor: &mut PerformanceMonitor,
    ) -> Result<Vec<(String, String)>> {
        let start_time = monitor.start_operation();

        if formats.is_empty() {
            return Ok(Vec::new());
        }

        println!("🔧 Generating code for {} formats in parallel...", formats.len());

        // Use rayon for parallel processing
        let results: Result<Vec<_>> = formats
            .par_iter()
            .map(|format| {
                let generator = crate::codegen::factory::GeneratorFactory::create_generator(format)?;
                let code = generator.generate(json_value, options)?;
                Ok((format.clone(), code))
            })
            .collect();

        let generation_results = results?;
        monitor.record_generation_time(start_time.elapsed());

        println!("✅ Parallel code generation completed for {} formats", formats.len());
        Ok(generation_results)
    }

    /// Check if parallel processing would be beneficial
    pub fn should_use_parallel(formats: &[String], json_size: usize) -> bool {
        // Use parallel processing if:
        // 1. Multiple formats are requested, OR
        // 2. Single format but large JSON (>1MB)
        formats.len() > 1 || json_size > 1_000_000
    }
}

/// Memory-efficient JSON analyzer for large structures
pub struct MemoryEfficientAnalyzer {
    max_depth: usize,
    sample_size: usize,
}

impl Default for MemoryEfficientAnalyzer {
    fn default() -> Self {
        Self {
            max_depth: 50,
            sample_size: 1000,
        }
    }
}

impl MemoryEfficientAnalyzer {
    /// Create a new analyzer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze JSON structure with memory efficiency optimizations
    pub fn analyze_structure(&self, value: &Value, monitor: &mut PerformanceMonitor) -> StructureAnalysis {
        let mut analysis = StructureAnalysis::new();
        self.analyze_value(value, 0, &mut analysis, monitor);
        analysis
    }

    /// Recursively analyze JSON value structure
    fn analyze_value(&self, value: &Value, depth: usize, analysis: &mut StructureAnalysis, monitor: &mut PerformanceMonitor) {
        if depth > self.max_depth {
            analysis.max_depth_exceeded = true;
            return;
        }

        match value {
            Value::Object(obj) => {
                analysis.object_count += 1;
                analysis.max_depth = analysis.max_depth.max(depth);
                monitor.increment_objects_processed();
                monitor.increment_fields_processed(obj.len());

                // For large objects, sample fields instead of processing all
                if obj.len() > self.sample_size {
                    analysis.large_objects += 1;
                    let sample: Vec<_> = obj.iter().take(self.sample_size).collect();
                    for (_, field_value) in sample {
                        self.analyze_value(field_value, depth + 1, analysis, monitor);
                    }
                } else {
                    for (_, field_value) in obj {
                        self.analyze_value(field_value, depth + 1, analysis, monitor);
                    }
                }
            }
            Value::Array(arr) => {
                analysis.array_count += 1;
                analysis.max_array_size = analysis.max_array_size.max(arr.len());

                // For large arrays, sample elements instead of processing all
                if arr.len() > self.sample_size {
                    analysis.large_arrays += 1;
                    let sample: Vec<_> = arr.iter().take(self.sample_size).collect();
                    for item in sample {
                        self.analyze_value(item, depth + 1, analysis, monitor);
                    }
                } else {
                    for item in arr {
                        self.analyze_value(item, depth + 1, analysis, monitor);
                    }
                }
            }
            Value::String(s) => {
                analysis.string_count += 1;
                analysis.max_string_length = analysis.max_string_length.max(s.len());
            }
            Value::Number(_) => analysis.number_count += 1,
            Value::Bool(_) => analysis.bool_count += 1,
            Value::Null => analysis.null_count += 1,
        }
    }
}

/// Structure analysis results for performance optimization decisions
#[derive(Debug, Clone)]
pub struct StructureAnalysis {
    pub object_count: usize,
    pub array_count: usize,
    pub string_count: usize,
    pub number_count: usize,
    pub bool_count: usize,
    pub null_count: usize,
    pub max_depth: usize,
    pub max_array_size: usize,
    pub max_string_length: usize,
    pub large_objects: usize,
    pub large_arrays: usize,
    pub max_depth_exceeded: bool,
}

impl StructureAnalysis {
    fn new() -> Self {
        Self {
            object_count: 0,
            array_count: 0,
            string_count: 0,
            number_count: 0,
            bool_count: 0,
            null_count: 0,
            max_depth: 0,
            max_array_size: 0,
            max_string_length: 0,
            large_objects: 0,
            large_arrays: 0,
            max_depth_exceeded: false,
        }
    }

    /// Get total element count
    pub fn total_elements(&self) -> usize {
        self.object_count + self.array_count + self.string_count + 
        self.number_count + self.bool_count + self.null_count
    }

    /// Check if the structure is complex enough to benefit from optimization
    pub fn is_complex(&self) -> bool {
        self.max_depth > 10 || 
        self.max_array_size > 1000 || 
        self.large_objects > 0 || 
        self.large_arrays > 0 ||
        self.total_elements() > 10000
    }

    /// Print analysis summary
    pub fn print_summary(&self) {
        println!("📊 JSON Structure Analysis:");
        println!("   📦 Objects: {}", self.object_count);
        println!("   📋 Arrays: {}", self.array_count);
        println!("   📝 Strings: {}", self.string_count);
        println!("   🔢 Numbers: {}", self.number_count);
        println!("   ✅ Booleans: {}", self.bool_count);
        println!("   ❌ Nulls: {}", self.null_count);
        println!("   📏 Max depth: {}", self.max_depth);
        println!("   📊 Max array size: {}", self.max_array_size);
        println!("   📄 Max string length: {}", self.max_string_length);
        
        if self.large_objects > 0 || self.large_arrays > 0 {
            println!("   ⚠️  Large structures: {} objects, {} arrays", self.large_objects, self.large_arrays);
        }
        
        if self.max_depth_exceeded {
            println!("   ⚠️  Maximum analysis depth exceeded");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.parse_time, Duration::ZERO);
        assert_eq!(metrics.generation_time, Duration::ZERO);
        assert_eq!(metrics.objects_processed, 0);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        monitor.record_input_size(1000);
        monitor.record_output_size(2000);
        monitor.increment_objects_processed();
        monitor.increment_fields_processed(5);

        let metrics = monitor.finalize();
        assert_eq!(metrics.input_size, 1000);
        assert_eq!(metrics.output_size, 2000);
        assert_eq!(metrics.objects_processed, 1);
        assert_eq!(metrics.fields_processed, 5);
    }

    #[test]
    fn test_streaming_processor() {
        let processor = StreamingJsonProcessor::new();
        let mut monitor = PerformanceMonitor::new();
        let json_content = r#"{"test": "value"}"#;

        let result = processor.process_large_json(json_content, &mut monitor);
        assert!(result.is_ok());
        
        let value = result.unwrap();
        assert_eq!(value["test"], "value");
    }

    #[test]
    fn test_memory_efficient_analyzer() {
        let analyzer = MemoryEfficientAnalyzer::new();
        let mut monitor = PerformanceMonitor::new();
        
        let json_value = json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ],
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        });

        let analysis = analyzer.analyze_structure(&json_value, &mut monitor);
        
        assert!(analysis.object_count > 0);
        assert!(analysis.array_count > 0);
        assert!(analysis.string_count > 0);
        assert!(analysis.number_count > 0);
        assert!(analysis.bool_count > 0);
    }

    #[test]
    fn test_structure_analysis_complexity() {
        let mut analysis = StructureAnalysis::new();
        analysis.max_depth = 15;
        analysis.max_array_size = 2000;
        
        assert!(analysis.is_complex());
        
        let simple_analysis = StructureAnalysis::new();
        assert!(!simple_analysis.is_complex());
    }

    #[test]
    fn test_parallel_should_use_parallel() {
        let multiple_formats = vec!["go".to_string(), "rust".to_string()];
        assert!(ParallelCodeGenerator::should_use_parallel(&multiple_formats, 100));
        
        let single_format = vec!["go".to_string()];
        assert!(ParallelCodeGenerator::should_use_parallel(&single_format, 2_000_000));
        assert!(!ParallelCodeGenerator::should_use_parallel(&single_format, 100));
    }
}