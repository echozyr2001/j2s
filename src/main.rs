mod cli;
mod file_ops;
mod schema_generator;
mod error;

use cli::{parse_args, print_help, print_version};
use error::{J2sError, Result};
use file_ops::{read_json_file, write_schema_file, generate_output_path};
use schema_generator::generate_schema;

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
    println!("Reading JSON file: {}", input_path);
    
    // Read JSON file
    let json_content = match read_json_file(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            return Err(e);
        }
    };
    
    // Parse JSON content
    println!("Parsing JSON content...");
    let json_value: serde_json::Value = match serde_json::from_str(&json_content) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            return Err(J2sError::json_error(format!("Failed to parse JSON from {}: {}", input_path, e)));
        }
    };
    
    // Generate schema
    println!("Generating JSON Schema...");
    let schema = generate_schema(&json_value);
    
    // Serialize schema to JSON
    let schema_json = match serde_json::to_string_pretty(&schema) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error serializing schema: {}", e);
            return Err(J2sError::schema_error(format!("Failed to serialize schema: {}", e)));
        }
    };
    
    // Write schema file
    println!("Writing schema to: {}", output_path);
    match write_schema_file(&output_path, &schema_json) {
        Ok(()) => {
            println!("✓ Successfully generated schema file: {}", output_path);
        }
        Err(e) => {
            eprintln!("Error writing schema file: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
