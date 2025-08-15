//! # j2s - JSON to Schema Tool
//!
//! A command-line tool for generating JSON Schema files from JSON input files.
//!
//! ## Overview
//!
//! j2s analyzes the structure of JSON data and creates corresponding JSON Schema
//! definitions following the JSON Schema Draft 2020-12 specification. It provides
//! intelligent type inference, handles complex nested structures, and offers
//! performance optimizations for large files.
//!
//! ## Features
//!
//! - **Automatic Type Inference**: Detects and maps JSON types to appropriate schema types
//! - **Nested Structure Support**: Handles deeply nested objects and arrays
//! - **Performance Optimized**: Includes progress indication and sampling for large files
//! - **Comprehensive Error Handling**: Provides detailed error messages and suggestions
//! - **Flexible CLI**: Supports multiple input methods and output customization
//!
//! ## Usage Examples
//!
//! ```bash
//! # Basic usage
//! j2s data.json
//!
//! # Custom output path
//! j2s data.json --output my-schema.json
//!
//! # Using input flag
//! j2s --input data.json --output schema.json
//! ```
//!
//! ## Performance Characteristics
//!
//! - Files up to 100MB are supported
//! - Large files (>10MB) show progress indicators  
//! - Very large arrays (>10k items) use sampling for performance
//! - Recursion depth is limited to prevent stack overflow
//!
//! ## Module Organization
//!
//! - `cli`: Command-line interface and argument parsing
//! - `file_ops`: File I/O operations with enhanced error handling
//! - `schema_generator`: Core JSON Schema generation logic
//! - `error`: Comprehensive error types and handling

mod cli;
mod error;
mod file_ops;
mod schema_generator;

use cli::{parse_args, print_help, print_version};
use error::{J2sError, Result};
use file_ops::{generate_output_path, read_json_file, write_schema_file};
use schema_generator::{generate_schema, generate_schema_with_progress};

/// Main entry point for the j2s application
///
/// This function orchestrates the complete JSON to Schema conversion process:
/// 1. Parse command line arguments
/// 2. Validate input and generate output paths
/// 3. Read and validate the JSON input file
/// 4. Generate the JSON Schema with progress indication for large files
/// 5. Write the schema to the output file with verification
///
/// # Returns
/// * `Result<()>` - Success or a detailed error with user guidance
///
/// # Error Handling
/// The function provides comprehensive error handling with:
/// - Detailed error messages explaining what went wrong
/// - Helpful tips and suggestions for resolving issues
/// - Appropriate exit codes for different error types
///
/// # Performance Features
/// - Progress indication for large files (>100KB)
/// - File size warnings and limits
/// - Memory-efficient processing
fn main() -> Result<()> {
    // Parse command line arguments
    let args = parse_args();

    // Handle special cases first
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    if std::env::args().any(|arg| arg == "--version" || arg == "-V") {
        print_version();
        return Ok(());
    }

    // Get input file path
    let input_path = match args.get_input_path() {
        Some(path) => path,
        None => {
            eprintln!("Error: No input file specified.");
            eprintln!("Usage: j2s <input.json> [--output <output.json>]");
            eprintln!("       j2s --input <input.json> [--output <output.json>]");
            eprintln!("       j2s --help");
            return Err(J2sError::argument_error("No input file specified"));
        }
    };

    // Generate output path
    let output_path = generate_output_path(input_path, args.output.as_deref());

    // Provide user feedback about what we're doing
    println!("📖 Reading JSON file: {input_path}");

    // Read JSON file
    let json_content = match read_json_file(input_path) {
        Ok(content) => {
            let file_size = content.len();
            if file_size > 1_000_000 {
                println!(
                    "📊 Processing large file ({:.1} MB)...",
                    file_size as f64 / 1_000_000.0
                );
            }
            content
        }
        Err(e) => {
            eprintln!("❌ Error reading input file: {e}");
            eprintln!("💡 Tip: Make sure the file exists and you have read permissions");
            return Err(e);
        }
    };

    // Parse JSON content
    println!("🔍 Parsing JSON content...");
    let json_value: serde_json::Value = match serde_json::from_str(&json_content) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("❌ Error parsing JSON: {e}");
            eprintln!("💡 Tip: Check that your JSON file has valid syntax");
            eprintln!("   Common issues: missing quotes, trailing commas, unescaped characters");
            return Err(J2sError::json_error(format!(
                "Failed to parse JSON from {input_path}: {e}"
            )));
        }
    };

    // Generate schema with progress indication for large files
    println!("⚙️  Generating JSON Schema...");
    let schema = if json_content.len() > 100_000 {
        // Use progress indication for large files
        generate_schema_with_progress(&json_value, true)
    } else {
        generate_schema(&json_value)
    };

    // Serialize schema to JSON
    let schema_json = match serde_json::to_string_pretty(&schema) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("❌ Error serializing schema: {e}");
            eprintln!("💡 Tip: This is likely an internal error. Please report this issue.");
            return Err(J2sError::schema_error(format!(
                "Failed to serialize schema: {e}"
            )));
        }
    };

    // Write schema file
    println!("💾 Writing schema to: {output_path}");
    match write_schema_file(&output_path, &schema_json) {
        Ok(()) => {
            println!("✅ Successfully generated schema file: {output_path}");
            let schema_size = schema_json.len();
            println!("📈 Schema size: {:.1} KB", schema_size as f64 / 1000.0);
        }
        Err(e) => {
            eprintln!("❌ Error writing schema file: {e}");
            eprintln!("💡 Tip: Check that you have write permissions to the output directory");
            return Err(e);
        }
    }

    Ok(())
}
