//! # Code Generation Utilities
//!
//! This module provides common utility functions used across different language generators.
//! It includes naming convention converters, identifier sanitizers, and other helper functions
//! that are shared between multiple generators.

use std::collections::HashSet;

/// Utility for converting between different naming conventions
///
/// This struct provides methods to convert identifiers between various naming conventions
/// commonly used in different programming languages.
pub struct NameConverter;

impl NameConverter {
    /// Convert a string to PascalCase (UpperCamelCase)
    ///
    /// This is commonly used for type names in Go, TypeScript, and other languages.
    /// Examples: "user_name" -> "UserName", "api-key" -> "ApiKey"
    ///
    /// # Arguments
    /// * `input` - The string to convert
    ///
    /// # Returns
    /// * `String` - The PascalCase version of the input
    pub fn to_pascal_case(input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }

        input
            .split(|c: char| c == '_' || c == '-' || c == ' ')
            .filter(|s| !s.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect()
    }

    /// Convert a string to camelCase
    ///
    /// This is commonly used for field names in TypeScript and other languages.
    /// Examples: "user_name" -> "userName", "api-key" -> "apiKey"
    ///
    /// # Arguments
    /// * `input` - The string to convert
    ///
    /// # Returns
    /// * `String` - The camelCase version of the input
    pub fn to_camel_case(input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }

        let pascal = Self::to_pascal_case(input);
        if pascal.is_empty() {
            return pascal;
        }

        let mut chars = pascal.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Convert a string to snake_case
    ///
    /// This is commonly used for field names in Rust and Python.
    /// Examples: "UserName" -> "user_name", "APIKey" -> "api_key"
    ///
    /// # Arguments
    /// * `input` - The string to convert
    ///
    /// # Returns
    /// * `String` - The snake_case version of the input
    pub fn to_snake_case(input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                // Check if the next character is lowercase (indicating a word boundary)
                // or if this is not the start of an acronym
                if chars.peek().map_or(true, |&next| next.is_lowercase()) {
                    result.push('_');
                }
            }

            if ch == '-' || ch == ' ' {
                result.push('_');
            } else {
                result.push(ch.to_lowercase().next().unwrap_or(ch));
            }
        }

        // Clean up multiple underscores
        while result.contains("__") {
            result = result.replace("__", "_");
        }

        result.trim_matches('_').to_string()
    }

    /// Convert a string to kebab-case
    ///
    /// This is sometimes used in configuration files and URLs.
    /// Examples: "UserName" -> "user-name", "APIKey" -> "api-key"
    ///
    /// # Arguments
    /// * `input` - The string to convert
    ///
    /// # Returns
    /// * `String` - The kebab-case version of the input
    pub fn to_kebab_case(input: &str) -> String {
        Self::to_snake_case(input).replace('_', "-")
    }

    /// Sanitize an identifier to ensure it's valid for the target language
    ///
    /// This method handles reserved keywords and invalid characters to ensure
    /// the resulting identifier is valid in the target language.
    ///
    /// # Arguments
    /// * `input` - The identifier to sanitize
    /// * `keywords` - Set of reserved keywords for the target language
    ///
    /// # Returns
    /// * `String` - A sanitized identifier that's safe to use
    pub fn sanitize_identifier(input: &str, keywords: &HashSet<String>) -> String {
        if input.is_empty() {
            return "field".to_string();
        }

        // Remove or replace invalid characters
        let mut sanitized = String::new();
        let mut chars = input.chars();

        // Ensure the first character is valid (letter or underscore)
        if let Some(first) = chars.next() {
            if first.is_alphabetic() || first == '_' {
                sanitized.push(first);
            } else if first.is_numeric() {
                sanitized.push('_');
                sanitized.push(first);
            } else {
                sanitized.push('_');
            }
        }

        // Process remaining characters
        for ch in chars {
            if ch.is_alphanumeric() || ch == '_' {
                sanitized.push(ch);
            } else {
                sanitized.push('_');
            }
        }

        // Handle reserved keywords
        if keywords.contains(&sanitized.to_lowercase()) {
            sanitized.push('_');
        }

        // Ensure we don't have an empty result
        if sanitized.is_empty() {
            sanitized = "field".to_string();
        }

        sanitized
    }

    /// Generate a struct name from a JSON field name or file path
    ///
    /// This method creates appropriate struct/type names from various inputs,
    /// handling common patterns and ensuring the result follows naming conventions.
    ///
    /// # Arguments
    /// * `input` - The input string (field name, file path, etc.)
    /// * `default` - Default name to use if input is empty or invalid
    ///
    /// # Returns
    /// * `String` - A suitable struct/type name
    pub fn generate_struct_name(input: &str, default: &str) -> String {
        if input.is_empty() {
            return Self::to_pascal_case(default);
        }

        // Extract filename from path if needed
        let name = if input.contains('/') || input.contains('\\') {
            input
                .split(['/', '\\'])
                .last()
                .unwrap_or(input)
                .split('.')
                .next()
                .unwrap_or(input)
        } else {
            input.split('.').next().unwrap_or(input)
        };

        if name.is_empty() {
            Self::to_pascal_case(default)
        } else {
            Self::to_pascal_case(name)
        }
    }
}

/// Generate a timestamp string for use in generated code comments
///
/// This function creates a human-readable timestamp that can be included in
/// generated code to indicate when the code was created.
///
/// # Returns
/// * `String` - A formatted timestamp string
pub fn generate_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    // Format as ISO 8601 date-time
    let secs = now.as_secs();
    let days_since_epoch = secs / 86400;
    let seconds_today = secs % 86400;

    // Simple date calculation (approximate)
    let years_since_1970 = days_since_epoch / 365;
    let year = 1970 + years_since_1970;

    let hours = seconds_today / 3600;
    let minutes = (seconds_today % 3600) / 60;
    let seconds = seconds_today % 60;

    format!("{}-01-01T{:02}:{:02}:{:02}Z", year, hours, minutes, seconds)
}

/// Escape a string for safe inclusion in generated code comments
///
/// This function ensures that strings included in comments don't break
/// the comment syntax or contain potentially problematic characters.
///
/// # Arguments
/// * `input` - The string to escape
///
/// # Returns
/// * `String` - The escaped string safe for use in comments
pub fn escape_comment_string(input: &str) -> String {
    input
        .replace("*/", "* /")  // Prevent closing block comments
        .replace("//", "/ /")  // Prevent line comments in block comments
        .replace('\n', " ")    // Replace newlines with spaces
        .replace('\r', " ")    // Replace carriage returns with spaces
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(NameConverter::to_pascal_case("user_name"), "UserName");
        assert_eq!(NameConverter::to_pascal_case("api-key"), "ApiKey");
        assert_eq!(NameConverter::to_pascal_case("first name"), "FirstName");
        assert_eq!(NameConverter::to_pascal_case("simple"), "Simple");
        assert_eq!(NameConverter::to_pascal_case(""), "");
        assert_eq!(NameConverter::to_pascal_case("a_b_c"), "ABC");
        assert_eq!(NameConverter::to_pascal_case("user_id_value"), "UserIdValue");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(NameConverter::to_camel_case("user_name"), "userName");
        assert_eq!(NameConverter::to_camel_case("api-key"), "apiKey");
        assert_eq!(NameConverter::to_camel_case("first name"), "firstName");
        assert_eq!(NameConverter::to_camel_case("simple"), "simple");
        assert_eq!(NameConverter::to_camel_case(""), "");
        assert_eq!(NameConverter::to_camel_case("a_b_c"), "aBC");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(NameConverter::to_snake_case("UserName"), "user_name");
        assert_eq!(NameConverter::to_snake_case("APIKey"), "api_key");
        assert_eq!(NameConverter::to_snake_case("firstName"), "first_name");
        assert_eq!(NameConverter::to_snake_case("simple"), "simple");
        assert_eq!(NameConverter::to_snake_case(""), "");
        assert_eq!(NameConverter::to_snake_case("XMLHttpRequest"), "xml_http_request");
        assert_eq!(NameConverter::to_snake_case("user-name"), "user_name");
        assert_eq!(NameConverter::to_snake_case("first name"), "first_name");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(NameConverter::to_kebab_case("UserName"), "user-name");
        assert_eq!(NameConverter::to_kebab_case("APIKey"), "api-key");
        assert_eq!(NameConverter::to_kebab_case("firstName"), "first-name");
        assert_eq!(NameConverter::to_kebab_case("simple"), "simple");
        assert_eq!(NameConverter::to_kebab_case(""), "");
    }

    #[test]
    fn test_sanitize_identifier() {
        let mut keywords = HashSet::new();
        keywords.insert("type".to_string());
        keywords.insert("struct".to_string());

        assert_eq!(NameConverter::sanitize_identifier("valid", &keywords), "valid");
        assert_eq!(NameConverter::sanitize_identifier("type", &keywords), "type_");
        assert_eq!(NameConverter::sanitize_identifier("struct", &keywords), "struct_");
        assert_eq!(NameConverter::sanitize_identifier("123invalid", &keywords), "_123invalid");
        assert_eq!(NameConverter::sanitize_identifier("invalid-name", &keywords), "invalid_name");
        assert_eq!(NameConverter::sanitize_identifier("", &keywords), "field");
        assert_eq!(NameConverter::sanitize_identifier("valid_name", &keywords), "valid_name");
    }

    #[test]
    fn test_generate_struct_name() {
        assert_eq!(NameConverter::generate_struct_name("user_data", "Default"), "UserData");
        assert_eq!(NameConverter::generate_struct_name("/path/to/user_data.json", "Default"), "UserData");
        assert_eq!(NameConverter::generate_struct_name("C:\\path\\to\\user_data.json", "Default"), "UserData");
        assert_eq!(NameConverter::generate_struct_name("", "default_name"), "DefaultName");
        assert_eq!(NameConverter::generate_struct_name("simple", "Default"), "Simple");
        assert_eq!(NameConverter::generate_struct_name("file.json", "Default"), "File");
    }

    #[test]
    fn test_generate_timestamp() {
        let timestamp = generate_timestamp();
        assert!(timestamp.contains("T"));
        assert!(timestamp.ends_with("Z"));
        assert!(timestamp.len() > 10); // Should be a reasonable length
    }

    #[test]
    fn test_escape_comment_string() {
        assert_eq!(escape_comment_string("normal text"), "normal text");
        assert_eq!(escape_comment_string("text with */ comment"), "text with * / comment");
        assert_eq!(escape_comment_string("text with // comment"), "text with / / comment");
        assert_eq!(escape_comment_string("text\nwith\nnewlines"), "text with newlines");
        assert_eq!(escape_comment_string("text\rwith\rcarriage"), "text with carriage");
        assert_eq!(escape_comment_string("  spaced text  "), "spaced text");
        assert_eq!(escape_comment_string(""), "");
    }
}