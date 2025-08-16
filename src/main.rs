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
mod codegen;
mod error;
mod file_ops;
mod schema_generator;

use cli::{parse_args, print_help, print_version};
use codegen::{factory::GeneratorFactory, generator::GenerationOptions};
use error::{J2sError, Result};
use file_ops::{generate_code_output_path, generate_output_path, read_json_file, write_code_file, write_schema_file};
use schema_generator::{generate_schema, generate_schema_with_progress};

/// Main entry point for the j2s application
///
/// This function orchestrates the complete JSON to Schema/Code conversion process:
/// 1. Parse command line arguments
/// 2. Validate input and generate output paths
/// 3. Read and validate the JSON input file
/// 4. Generate the output based on the specified format (schema or code)
/// 5. Write the output to the appropriate file with verification
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
            eprintln!("Usage: j2s <input.json> [--output <output.json>] [--format <format>]");
            eprintln!("       j2s --input <input.json> [--output <output.json>] [--format <format>]");
            eprintln!("       j2s --help");
            return Err(J2sError::argument_error("No input file specified"));
        }
    };

    // Get the target format (defaults to "schema" for backward compatibility)
    let format = args.get_format();
    
    // Provide user feedback about the selected format
    if format != "schema" {
        println!("🎯 Target format: {} ({})", format, 
                 match format {
                     "go" => "Go language structs",
                     "rust" => "Rust structs with serde",
                     "typescript" => "TypeScript interfaces",
                     "python" => "Python dataclasses",
                     _ => "Unknown format"
                 });
    }

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

    // Generate output based on format
    match format {
        "schema" => {
            // Generate JSON Schema (backward compatibility)
            generate_schema_output(&json_value, input_path, &args, &json_content)
        }
        _ => {
            // Generate code for the specified language
            generate_code_output(&json_value, input_path, &args, format)
        }
    }
}

/// Generate JSON Schema output (maintains backward compatibility)
fn generate_schema_output(
    json_value: &serde_json::Value,
    input_path: &str,
    args: &cli::CliArgs,
    json_content: &str,
) -> Result<()> {
    // Generate output path for schema
    let output_path = generate_output_path(input_path, args.output.as_deref());

    // Generate schema with progress indication for large files
    println!("⚙️  Generating JSON Schema...");
    let schema = if json_content.len() > 100_000 {
        // Use progress indication for large files
        generate_schema_with_progress(json_value, true)
    } else {
        generate_schema(json_value)
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

/// Generate code output for the specified programming language
fn generate_code_output(
    json_value: &serde_json::Value,
    input_path: &str,
    args: &cli::CliArgs,
    format: &str,
) -> Result<()> {
    // Generate output path for code
    let output_path = generate_code_output_path(input_path, args.output.as_deref(), format);

    // Create code generator for the target language
    println!("🔧 Creating {} code generator...", format);
    let generator = match GeneratorFactory::create_generator(format) {
        Ok(generator) => generator,
        Err(e) => {
            eprintln!("❌ Error creating code generator: {e}");
            eprintln!("💡 Tip: Supported formats are:");
            eprintln!("   • go        - Go language structs with JSON tags");
            eprintln!("   • rust      - Rust structs with serde annotations");
            eprintln!("   • typescript - TypeScript interfaces");
            eprintln!("   • python    - Python dataclasses with type hints");
            eprintln!("   • schema    - JSON Schema (default)");
            eprintln!("📖 Example: j2s data.json --format go --struct-name User");
            return Err(e);
        }
    };

    // Prepare generation options
    let struct_name = args.get_struct_name();
    let options = GenerationOptions::new()
        .with_struct_name(struct_name)
        .with_comments(true)
        .with_optional_fields(true);

    // Validate options with the generator
    if let Err(e) = generator.validate_options(&options) {
        eprintln!("❌ Invalid generation options: {e}");
        eprintln!("💡 Tip: Check your --struct-name parameter and other options");
        return Err(e);
    }

    // Generate code with progress indication for large files
    let file_size = json_value.to_string().len();
    if file_size > 100_000 {
        println!("⚙️  Generating {} code for large file ({:.1} KB)...", 
                 generator.language_name(), file_size as f64 / 1000.0);
        println!("📊 Processing complex JSON structure...");
    } else {
        println!("⚙️  Generating {} code...", generator.language_name());
    }
    
    let generated_code = match generator.generate(json_value, &options) {
        Ok(code) => {
            if file_size > 100_000 {
                println!("✨ Code generation completed successfully!");
            }
            code
        },
        Err(e) => {
            eprintln!("❌ Error generating {} code: {e}", generator.language_name());
            eprintln!("💡 Troubleshooting tips:");
            eprintln!("   • Check that your JSON structure is valid");
            eprintln!("   • Ensure the JSON is not too deeply nested (max 10 levels)");
            eprintln!("   • Try with a simpler JSON structure first");
            eprintln!("   • Check that field names don't contain special characters");
            if args.struct_name.is_some() {
                eprintln!("   • Verify your --struct-name parameter is valid");
            }
            return Err(e);
        }
    };

    // Write code file
    println!("💾 Writing {} code to: {output_path}", generator.language_name());
    match write_code_file(&output_path, &generated_code, format) {
        Ok(()) => {
            println!("✅ Successfully generated {} file: {output_path}", generator.language_name());
            let code_size = generated_code.len();
            println!("📈 Code size: {:.1} KB", code_size as f64 / 1000.0);
            
            // Provide language-specific usage hints
            print_usage_hints(format, &output_path);
        }
        Err(e) => {
            eprintln!("❌ Error writing {} file: {e}", generator.language_name());
            eprintln!("💡 Troubleshooting tips:");
            eprintln!("   • Check that you have write permissions to the output directory");
            eprintln!("   • Ensure the output directory exists");
            eprintln!("   • Verify there's enough disk space available");
            eprintln!("   • Check if the file is currently open in another application");
            if let Some(output) = &args.output {
                eprintln!("   • Verify the output path is valid: {}", output);
            }
            return Err(e);
        }
    }

    Ok(())
}

/// Print language-specific usage hints for the generated code
fn print_usage_hints(format: &str, output_path: &str) {
    match format {
        "go" => {
            println!("💡 Usage hints for Go:");
            println!("   • Add to your Go module: go mod tidy");
            println!("   • Import in your code: import \"encoding/json\"");
            println!("   • Use json.Unmarshal() to parse JSON into your struct");
        }
        "rust" => {
            println!("💡 Usage hints for Rust:");
            println!("   • Add to Cargo.toml: serde = {{ version = \"1.0\", features = [\"derive\"] }}");
            println!("   • Add to Cargo.toml: serde_json = \"1.0\"");
            println!("   • Use serde_json::from_str() to parse JSON into your struct");
        }
        "typescript" => {
            println!("💡 Usage hints for TypeScript:");
            println!("   • Import in your code: import {{ YourInterface }} from './{}'", 
                     std::path::Path::new(output_path).file_stem().unwrap().to_str().unwrap());
            println!("   • Use JSON.parse() with type assertion: JSON.parse(data) as YourInterface");
        }
        "python" => {
            println!("💡 Usage hints for Python:");
            println!("   • Import in your code: from {} import YourClass", 
                     std::path::Path::new(output_path).file_stem().unwrap().to_str().unwrap());
            println!("   • Use json.loads() and create instance: YourClass(**json.loads(data))");
        }
        _ => {}
    }
}
