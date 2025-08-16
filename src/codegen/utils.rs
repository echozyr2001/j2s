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
    /// Get reserved keywords for a specific language
    ///
    /// Returns a HashSet of reserved keywords that should be avoided when
    /// generating identifiers for the specified language.
    ///
    /// # Arguments
    /// * `language` - The target language ("go", "rust", "typescript", "python")
    ///
    /// # Returns
    /// * `HashSet<String>` - Set of reserved keywords for the language
    pub fn get_reserved_keywords(language: &str) -> HashSet<String> {
        let keywords = match language {
            "go" => vec![
                "break", "case", "chan", "const", "continue", "default", "defer", "else",
                "fallthrough", "for", "func", "go", "goto", "if", "import", "interface",
                "map", "package", "range", "return", "select", "struct", "switch", "type",
                "var", "bool", "byte", "complex64", "complex128", "error", "float32",
                "float64", "int", "int8", "int16", "int32", "int64", "rune", "string",
                "uint", "uint8", "uint16", "uint32", "uint64", "uintptr", "true", "false",
                "iota", "nil", "append", "cap", "close", "complex", "copy", "delete",
                "imag", "len", "make", "new", "panic", "print", "println", "real", "recover"
            ],
            "rust" => vec![
                "as", "break", "const", "continue", "crate", "else", "enum", "extern",
                "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
                "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                "super", "trait", "true", "type", "unsafe", "use", "where", "while",
                "async", "await", "dyn", "abstract", "become", "box", "do", "final",
                "macro", "override", "priv", "typeof", "unsized", "virtual", "yield",
                "try", "union", "raw"
            ],
            "typescript" => vec![
                "break", "case", "catch", "class", "const", "continue", "debugger",
                "default", "delete", "do", "else", "enum", "export", "extends", "false",
                "finally", "for", "function", "if", "import", "in", "instanceof", "new",
                "null", "return", "super", "switch", "this", "throw", "true", "try",
                "typeof", "var", "void", "while", "with", "as", "implements", "interface",
                "let", "package", "private", "protected", "public", "static", "yield",
                "any", "boolean", "constructor", "declare", "get", "module", "require",
                "number", "set", "string", "symbol", "type", "from", "of", "namespace",
                "abstract", "async", "await", "is", "keyof", "readonly", "unique",
                "infer", "never", "object", "unknown"
            ],
            "python" => vec![
                "False", "None", "True", "and", "as", "assert", "async", "await", "break",
                "class", "continue", "def", "del", "elif", "else", "except", "finally",
                "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal",
                "not", "or", "pass", "raise", "return", "try", "while", "with", "yield",
                "match", "case", "type", "int", "float", "str", "bool", "list", "dict",
                "tuple", "set", "frozenset", "bytes", "bytearray", "memoryview", "range",
                "enumerate", "zip", "map", "filter", "sorted", "reversed", "len", "sum",
                "min", "max", "abs", "round", "pow", "divmod", "isinstance", "issubclass",
                "hasattr", "getattr", "setattr", "delattr", "callable", "iter", "next",
                "open", "print", "input", "repr", "str", "chr", "ord", "hex", "oct", "bin"
            ],
            _ => vec![]
        };
        
        keywords.into_iter().map(String::from).collect()
    }

    /// Sanitize an identifier for a specific language
    ///
    /// This method handles reserved keywords and invalid characters to ensure
    /// the resulting identifier is valid in the target language.
    ///
    /// # Arguments
    /// * `input` - The identifier to sanitize
    /// * `language` - The target language
    ///
    /// # Returns
    /// * `String` - A sanitized identifier that's safe to use
    pub fn sanitize_identifier_for_language(input: &str, language: &str) -> String {
        let keywords = Self::get_reserved_keywords(language);
        Self::sanitize_identifier(input, &keywords)
    }

    /// Convert field name to appropriate naming convention for the language
    ///
    /// This method automatically applies the correct naming convention based on
    /// the target language and sanitizes the result.
    ///
    /// # Arguments
    /// * `input` - The field name to convert
    /// * `language` - The target language
    ///
    /// # Returns
    /// * `String` - The converted and sanitized field name
    pub fn convert_field_name(input: &str, language: &str) -> String {
        // First clean the input to handle special characters
        let cleaned = if Self::has_special_characters(input) {
            Self::clean_string(input)
        } else {
            input.to_string()
        };
        
        let converted = match language {
            "go" => Self::to_pascal_case(&cleaned),
            "typescript" => Self::to_camel_case(&cleaned),
            "rust" | "python" => Self::to_snake_case(&cleaned),
            _ => cleaned,
        };
        
        Self::sanitize_identifier_for_language(&converted, language)
    }

    /// Convert type name to appropriate naming convention for the language
    ///
    /// This method automatically applies the correct naming convention for type names
    /// based on the target language and sanitizes the result.
    ///
    /// # Arguments
    /// * `input` - The type name to convert
    /// * `language` - The target language
    ///
    /// # Returns
    /// * `String` - The converted and sanitized type name
    pub fn convert_type_name(input: &str, language: &str) -> String {
        // First clean the input to handle special characters
        let cleaned = if Self::has_special_characters(input) {
            Self::clean_string(input)
        } else {
            input.to_string()
        };
        
        let converted = match language {
            "go" | "rust" | "typescript" | "python" => Self::to_pascal_case(&cleaned),
            _ => cleaned,
        };
        
        Self::sanitize_identifier_for_language(&converted, language)
    }

    /// Check if a string contains special characters that need handling
    ///
    /// # Arguments
    /// * `input` - The string to check
    ///
    /// # Returns
    /// * `bool` - True if the string contains special characters
    pub fn has_special_characters(input: &str) -> bool {
        input.chars().any(|c| !c.is_alphanumeric() && c != '_')
    }

    /// Clean a string by removing or replacing problematic characters
    ///
    /// # Arguments
    /// * `input` - The string to clean
    ///
    /// # Returns
    /// * `String` - The cleaned string
    pub fn clean_string(input: &str) -> String {
        input
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else if c == '-' || c == ' ' || c == '.' {
                    '_'
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }
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

        // First, split on common separators
        let parts: Vec<&str> = input
            .split(|c: char| c == '_' || c == '-' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();

        let mut result = String::new();
        
        for part in parts {
            // For each part, handle camelCase by splitting on uppercase letters
            let mut current_word = String::new();
            let mut prev_was_lower = false;
            
            for c in part.chars() {
                if c.is_uppercase() && prev_was_lower && !current_word.is_empty() {
                    // Found a camelCase boundary, capitalize the current word and start a new one
                    result.push_str(&Self::capitalize_word(&current_word));
                    current_word.clear();
                }
                current_word.push(c);
                prev_was_lower = c.is_lowercase();
            }
            
            // Add the final word
            if !current_word.is_empty() {
                result.push_str(&Self::capitalize_word(&current_word));
            }
        }
        
        result
    }
    
    /// Helper function to capitalize a single word
    fn capitalize_word(word: &str) -> String {
        if word.is_empty() {
            return String::new();
        }
        
        let mut chars = word.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
        }
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

        // Clean up multiple underscores but preserve leading underscore if needed
        while sanitized.contains("__") {
            sanitized = sanitized.replace("__", "_");
        }
        
        // Only trim trailing underscores, preserve leading underscore for numeric starts
        sanitized = sanitized.trim_end_matches('_').to_string();

        // Handle reserved keywords
        if keywords.contains(&sanitized.to_lowercase()) {
            sanitized.push('_');
        }

        // Ensure we don't have an empty result or just underscores
        if sanitized.is_empty() || sanitized.chars().all(|c| c == '_') {
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

    #[test]
    fn test_get_reserved_keywords() {
        let go_keywords = NameConverter::get_reserved_keywords("go");
        assert!(go_keywords.contains("struct"));
        assert!(go_keywords.contains("type"));
        assert!(go_keywords.contains("func"));
        assert!(!go_keywords.contains("random_word"));

        let rust_keywords = NameConverter::get_reserved_keywords("rust");
        assert!(rust_keywords.contains("struct"));
        assert!(rust_keywords.contains("impl"));
        assert!(rust_keywords.contains("fn"));

        let ts_keywords = NameConverter::get_reserved_keywords("typescript");
        assert!(ts_keywords.contains("interface"));
        assert!(ts_keywords.contains("type"));
        assert!(ts_keywords.contains("class"));

        let py_keywords = NameConverter::get_reserved_keywords("python");
        assert!(py_keywords.contains("class"));
        assert!(py_keywords.contains("def"));
        assert!(py_keywords.contains("import"));

        let unknown_keywords = NameConverter::get_reserved_keywords("unknown");
        assert!(unknown_keywords.is_empty());
    }

    #[test]
    fn test_sanitize_identifier_for_language() {
        // Test Go keywords
        assert_eq!(NameConverter::sanitize_identifier_for_language("struct", "go"), "struct_");
        assert_eq!(NameConverter::sanitize_identifier_for_language("type", "go"), "type_");
        assert_eq!(NameConverter::sanitize_identifier_for_language("valid", "go"), "valid");

        // Test Rust keywords
        assert_eq!(NameConverter::sanitize_identifier_for_language("impl", "rust"), "impl_");
        assert_eq!(NameConverter::sanitize_identifier_for_language("fn", "rust"), "fn_");
        assert_eq!(NameConverter::sanitize_identifier_for_language("valid", "rust"), "valid");

        // Test TypeScript keywords
        assert_eq!(NameConverter::sanitize_identifier_for_language("interface", "typescript"), "interface_");
        assert_eq!(NameConverter::sanitize_identifier_for_language("class", "typescript"), "class_");
        assert_eq!(NameConverter::sanitize_identifier_for_language("valid", "typescript"), "valid");

        // Test Python keywords
        assert_eq!(NameConverter::sanitize_identifier_for_language("class", "python"), "class_");
        assert_eq!(NameConverter::sanitize_identifier_for_language("def", "python"), "def_");
        assert_eq!(NameConverter::sanitize_identifier_for_language("valid", "python"), "valid");
    }

    #[test]
    fn test_convert_field_name() {
        // Test Go (PascalCase)
        assert_eq!(NameConverter::convert_field_name("user_name", "go"), "UserName");
        assert_eq!(NameConverter::convert_field_name("api_key", "go"), "ApiKey");
        assert_eq!(NameConverter::convert_field_name("type", "go"), "Type_"); // Reserved keyword

        // Test Rust (snake_case)
        assert_eq!(NameConverter::convert_field_name("UserName", "rust"), "user_name");
        assert_eq!(NameConverter::convert_field_name("APIKey", "rust"), "api_key");
        assert_eq!(NameConverter::convert_field_name("impl", "rust"), "impl_"); // Reserved keyword

        // Test TypeScript (camelCase)
        assert_eq!(NameConverter::convert_field_name("user_name", "typescript"), "userName");
        assert_eq!(NameConverter::convert_field_name("api_key", "typescript"), "apiKey");
        assert_eq!(NameConverter::convert_field_name("class", "typescript"), "class_"); // Reserved keyword

        // Test Python (snake_case)
        assert_eq!(NameConverter::convert_field_name("UserName", "python"), "user_name");
        assert_eq!(NameConverter::convert_field_name("APIKey", "python"), "api_key");
        assert_eq!(NameConverter::convert_field_name("class", "python"), "class_"); // Reserved keyword
    }

    #[test]
    fn test_convert_type_name() {
        // All languages use PascalCase for type names
        assert_eq!(NameConverter::convert_type_name("user_data", "go"), "UserData");
        assert_eq!(NameConverter::convert_type_name("user_data", "rust"), "UserData");
        assert_eq!(NameConverter::convert_type_name("user_data", "typescript"), "UserData");
        assert_eq!(NameConverter::convert_type_name("user_data", "python"), "UserData");

        // Test with reserved keywords
        assert_eq!(NameConverter::convert_type_name("type", "go"), "Type_");
        assert_eq!(NameConverter::convert_type_name("struct", "rust"), "Struct_");
        assert_eq!(NameConverter::convert_type_name("interface", "typescript"), "Interface_");
        assert_eq!(NameConverter::convert_type_name("class", "python"), "Class_");
    }

    #[test]
    fn test_has_special_characters() {
        assert!(!NameConverter::has_special_characters("simple"));
        assert!(!NameConverter::has_special_characters("user_name"));
        assert!(!NameConverter::has_special_characters("UserName123"));
        
        assert!(NameConverter::has_special_characters("user-name"));
        assert!(NameConverter::has_special_characters("user name"));
        assert!(NameConverter::has_special_characters("user.name"));
        assert!(NameConverter::has_special_characters("user@name"));
        assert!(NameConverter::has_special_characters("user#name"));
    }

    #[test]
    fn test_clean_string() {
        assert_eq!(NameConverter::clean_string("simple"), "simple");
        assert_eq!(NameConverter::clean_string("user_name"), "user_name");
        assert_eq!(NameConverter::clean_string("user-name"), "user_name");
        assert_eq!(NameConverter::clean_string("user name"), "user_name");
        assert_eq!(NameConverter::clean_string("user.name"), "user_name");
        assert_eq!(NameConverter::clean_string("user@name#test"), "user_name_test");
        assert_eq!(NameConverter::clean_string("_leading_underscore_"), "leading_underscore");
        assert_eq!(NameConverter::clean_string("123numbers"), "123numbers");
        assert_eq!(NameConverter::clean_string(""), "");
    }

    #[test]
    fn test_complex_naming_scenarios() {
        // Test complex field names with special characters and keywords
        assert_eq!(
            NameConverter::convert_field_name("user-profile.data", "go"),
            "UserProfileData"
        );
        assert_eq!(
            NameConverter::convert_field_name("api@key#value", "rust"),
            "api_key_value"
        );
        assert_eq!(
            NameConverter::convert_field_name("class-name", "typescript"),
            "className"
        );
        assert_eq!(
            NameConverter::convert_field_name("def-value", "python"),
            "def_value"
        );

        // Test type names with complex inputs
        assert_eq!(
            NameConverter::convert_type_name("user-profile.data", "go"),
            "UserProfileData"
        );
        assert_eq!(
            NameConverter::convert_type_name("api@response#data", "rust"),
            "ApiResponseData"
        );
    }

    #[test]
    fn test_edge_cases() {
        // Empty strings
        assert_eq!(NameConverter::convert_field_name("", "go"), "field");
        assert_eq!(NameConverter::convert_type_name("", "rust"), "field");

        // Only special characters
        assert_eq!(NameConverter::convert_field_name("@#$", "go"), "field");
        assert_eq!(NameConverter::convert_type_name("@#$", "rust"), "field");

        // Numbers at start
        assert_eq!(NameConverter::convert_field_name("123test", "go"), "_123test");
        assert_eq!(NameConverter::convert_type_name("123test", "rust"), "_123test");

        // Mixed case with numbers
        assert_eq!(NameConverter::convert_field_name("user123Name", "rust"), "user123_name");
        assert_eq!(NameConverter::convert_field_name("API2Key", "rust"), "api2_key");
    }
}