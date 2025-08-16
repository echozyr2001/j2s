//! # Language-Specific Code Generators
//!
//! This module contains the implementations of code generators for different programming
//! languages. Each language has its own submodule that implements the `CodeGenerator` trait
//! with language-specific logic for type mapping, naming conventions, and code formatting.
//!
//! ## Supported Languages
//!
//! - **Go**: Generates Go structs with JSON tags and proper naming conventions
//! - **Rust**: Generates Rust structs with serde derive macros and Option types
//! - **TypeScript**: Generates TypeScript interfaces with optional properties
//! - **Python**: Generates Python dataclasses with type annotations
//!
//! ## Adding New Languages
//!
//! To add support for a new language:
//!
//! 1. Create a new module file (e.g., `java.rs`)
//! 2. Implement the `CodeGenerator` trait for your language
//! 3. Add the module to this file
//! 4. Update the `GeneratorFactory` to include your new generator
//! 5. Add appropriate tests

pub mod go;
pub mod python;
pub mod rust;
pub mod typescript;