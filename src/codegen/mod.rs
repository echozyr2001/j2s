//! # Code Generation Module
//!
//! This module provides the core infrastructure for generating code in multiple programming
//! languages from JSON data. It implements a plugin-based architecture where each language
//! has its own generator that implements the common `CodeGenerator` trait.
//!
//! ## Architecture
//!
//! The module is organized around the following key components:
//!
//! - **CodeGenerator Trait**: Defines the interface that all language generators must implement
//! - **GeneratorFactory**: Creates appropriate generators based on the target format
//! - **Language Generators**: Specific implementations for Go, Rust, TypeScript, and Python
//! - **Type System**: Common type definitions and mapping utilities
//!
//! ## Usage
//!
//! ```rust
//! use crate::codegen::{GeneratorFactory, GenerationOptions};
//!
//! let generator = GeneratorFactory::create_generator("go")?;
//! let options = GenerationOptions::default();
//! let code = generator.generate(&json_value, &options)?;
//! ```

pub mod comments;
pub mod factory;
pub mod generator;
pub mod languages;
pub mod types;
pub mod utils;

// Re-export main types for convenience
pub use factory::GeneratorFactory;
pub use generator::{CodeGenerator, GenerationOptions};
pub use types::{FieldDefinition, FieldType, StructDefinition};