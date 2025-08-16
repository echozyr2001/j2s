use clap::{Arg, Command};

/// Command line arguments structure for the j2s tool
///
/// This structure holds all the parsed command line arguments and provides
/// methods to access them in a consistent way. It supports both positional
/// and flag-based input specification for flexibility.
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Input file path specified via --input flag
    pub input: Option<String>,
    /// Output file path specified via --output flag  
    pub output: Option<String>,
    /// Input file path specified as positional argument
    pub json_file: Option<String>,
    /// Target output format specified via --format flag
    pub format: Option<String>,
    /// Custom struct/type name specified via --struct-name flag
    pub struct_name: Option<String>,
}

impl CliArgs {
    /// Get the effective input file path with priority handling
    ///
    /// This method returns the input file path, giving priority to the --input flag
    /// over the positional argument. This allows users to override positional arguments
    /// with explicit flags if needed.
    ///
    /// # Returns
    /// * `Some(&String)` - The input file path if specified
    /// * `None` - If no input file was specified
    ///
    /// # Priority Order
    /// 1. --input flag value (highest priority)
    /// 2. Positional argument value
    /// 3. None if neither is specified
    pub fn get_input_path(&self) -> Option<&String> {
        self.input.as_ref().or(self.json_file.as_ref())
    }

    /// Get the validated and cleaned struct name
    ///
    /// This method returns a cleaned struct name that follows common naming conventions.
    /// If no struct name is provided, it generates one from the input file name.
    ///
    /// # Returns
    /// * `String` - A valid struct name
    ///
    /// # Cleaning Rules
    /// - Removes invalid characters (non-alphanumeric, underscore, hyphen)
    /// - Converts to PascalCase for struct names
    /// - Ensures the name starts with a letter or underscore
    /// - Handles reserved keywords by appending "Type"
    pub fn get_struct_name(&self) -> String {
        match &self.struct_name {
            Some(name) => Self::clean_struct_name(name),
            None => self.generate_default_struct_name(),
        }
    }

    /// Clean and validate a struct name
    ///
    /// This method sanitizes a user-provided struct name to ensure it's valid
    /// for use as an identifier in most programming languages.
    ///
    /// # Arguments
    /// * `name` - The raw struct name to clean
    ///
    /// # Returns
    /// * `String` - A cleaned and valid struct name
    pub fn clean_struct_name(name: &str) -> String {
        if name.is_empty() {
            return "Data".to_string();
        }

        // Replace invalid characters with separators, then clean
        let normalized: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c == '_' || c == '-' || c == ' ' || c == '.' {
                    c // Keep these as separators
                } else {
                    ' ' // Convert other special characters to spaces (separators)
                }
            })
            .collect();

        // Handle camelCase/PascalCase by inserting spaces before uppercase letters
        // Only split on clear camelCase boundaries (lowercase followed by single uppercase)
        // Don't split on acronyms (consecutive uppercase letters)
        let mut spaced = String::new();
        let chars: Vec<char> = normalized.chars().collect();
        
        for (i, &c) in chars.iter().enumerate() {
            // Insert space before uppercase letter only if:
            // 1. Previous char was lowercase (clear camelCase boundary), AND
            // 2. Next char is lowercase (not an acronym) OR this is the last char
            if c.is_uppercase() && i > 0 && chars[i - 1].is_lowercase() {
                let next_is_lowercase_or_end = i + 1 >= chars.len() || chars[i + 1].is_lowercase();
                if next_is_lowercase_or_end {
                    spaced.push(' ');
                }
            }
            spaced.push(c);
        }

        // Split on separators and convert to PascalCase
        let pascal_case = spaced
            .split(|c: char| c == '_' || c == '-' || c == ' ' || c == '.')
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        let rest: String = chars.collect();
                        first.to_uppercase().collect::<String>() + &rest.to_lowercase()
                    }
                }
            })
            .collect::<String>();

        // Ensure the name starts with a letter or underscore
        let final_name = if pascal_case.is_empty() || pascal_case.chars().next().unwrap().is_numeric() {
            format!("Data{}", pascal_case)
        } else {
            pascal_case
        };

        // Handle common reserved keywords
        if Self::is_reserved_keyword(&final_name) {
            format!("{}Type", final_name)
        } else {
            final_name
        }
    }

    /// Generate a default struct name from the input file path
    ///
    /// This method creates a struct name based on the input file name when
    /// no explicit struct name is provided.
    ///
    /// # Returns
    /// * `String` - A generated struct name
    fn generate_default_struct_name(&self) -> String {
        match self.get_input_path() {
            Some(path) => {
                // Extract filename without extension
                let filename = std::path::Path::new(path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("data");
                
                Self::clean_struct_name(filename)
            }
            None => "Data".to_string(),
        }
    }

    /// Check if a name is a reserved keyword in common programming languages
    ///
    /// This method checks against a list of common reserved keywords across
    /// multiple programming languages to avoid naming conflicts.
    ///
    /// # Arguments
    /// * `name` - The name to check
    ///
    /// # Returns
    /// * `bool` - True if the name is a reserved keyword
    fn is_reserved_keyword(name: &str) -> bool {
        // Common reserved keywords across multiple languages
        const RESERVED_KEYWORDS: &[&str] = &[
            // Common across many languages
            "class", "struct", "interface", "type", "enum", "union",
            "public", "private", "protected", "static", "final", "const",
            "var", "let", "function", "return", "if", "else", "for", "while",
            "do", "switch", "case", "default", "break", "continue",
            "try", "catch", "finally", "throw", "throws", "import", "export",
            "package", "namespace", "module", "use", "using",
            // Go specific
            "go", "defer", "select", "chan", "map", "range", "fallthrough",
            "func", "interface", "struct", "type", "var", "const", "package", "import",
            // Rust specific
            "fn", "impl", "trait", "mod", "pub", "crate", "super", "self", "Self",
            "mut", "ref", "move", "box", "match", "where", "unsafe", "extern",
            // TypeScript/JavaScript specific
            "any", "unknown", "never", "void", "object", "string", "number", "boolean",
            "symbol", "bigint", "undefined", "null", "true", "false",
            // Python specific
            "def", "class", "lambda", "with", "as", "pass", "yield", "global", "nonlocal",
            "assert", "del", "from", "in", "is", "not", "and", "or",
        ];

        RESERVED_KEYWORDS.contains(&name.to_lowercase().as_str())
    }

    /// Validate the format parameter
    ///
    /// This method checks if the provided format is supported by the tool.
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if valid, Err with message if invalid
    pub fn validate_format(&self) -> Result<(), String> {
        match &self.format {
            Some(format) => {
                match format.to_lowercase().as_str() {
                    "schema" | "go" | "rust" | "typescript" | "python" => Ok(()),
                    _ => Err(format!("Unsupported format '{}'. Supported formats: schema, go, rust, typescript, python", format)),
                }
            }
            None => Ok(()), // None is valid (defaults to schema)
        }
    }

    /// Get the effective format with default handling
    ///
    /// This method returns the format, defaulting to "schema" if none is specified.
    ///
    /// # Returns
    /// * `&str` - The effective format to use
    pub fn get_format(&self) -> &str {
        self.format.as_deref().unwrap_or("schema")
    }
}

/// Parse command line arguments into a CliArgs structure
///
/// This function uses the clap library to parse command line arguments according
/// to the application's defined interface. It handles all argument validation
/// and returns a structured representation of the user's input.
///
/// # Returns
/// * `CliArgs` - Parsed command line arguments
///
/// # Panics
/// * Will panic if clap encounters an unrecoverable parsing error
///   (this is the standard behavior for clap applications)
pub fn parse_args() -> CliArgs {
    let matches = build_cli().get_matches();

    let args = CliArgs {
        input: matches.get_one::<String>("input").cloned(),
        output: matches.get_one::<String>("output").cloned(),
        json_file: matches.get_one::<String>("json_file").cloned(),
        format: matches.get_one::<String>("format").cloned(),
        struct_name: matches.get_one::<String>("struct_name").cloned(),
    };

    // Validate format if provided
    if let Err(err) = args.validate_format() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    args
}

/// Print the application help message to stdout
///
/// This function displays comprehensive usage information including all available
/// options, arguments, and examples. It's called when the user requests help
/// via --help or -h flags.
pub fn print_help() {
    let mut app = build_cli();
    app.print_help().unwrap();
    println!();
}

/// Print the application version information to stdout
///
/// This function displays the application name and version number.
/// The version is automatically extracted from Cargo.toml at compile time.
pub fn print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

/// Build the clap Command structure for argument parsing
///
/// This function defines the complete command-line interface for the j2s tool,
/// including all arguments, options, help text, and validation rules.
///
/// # Returns
/// * `Command` - A configured clap Command ready for argument parsing
///
/// # Interface Design
/// The CLI supports multiple input methods for flexibility:
/// - Positional argument: `j2s input.json`
/// - Input flag: `j2s --input input.json` or `j2s -i input.json`
/// - Output control: `--output path` or `-o path`
/// - Format selection: `--format go` or `-f go`
/// - Custom struct name: `--struct-name MyStruct`
///
/// # Examples
/// ```bash
/// j2s data.json                                    # Generate data.schema.json
/// j2s --input data.json                            # Same as above using flag
/// j2s data.json --output my-schema.json            # Custom output filename
/// j2s -i data.json -o schema.json                  # Using short flags
/// j2s data.json --format go                        # Generate Go struct
/// j2s data.json --format rust --struct-name User   # Generate Rust struct with custom name
/// ```
fn build_cli() -> Command {
    Command::new("j2s")
        .version(env!("CARGO_PKG_VERSION"))
        .author("JSON to Schema Tool")
        .about("Generate JSON Schema and code structures from JSON files")
        .long_about(
            "j2s is a command-line tool that generates JSON Schema files and programming language\n\
             structures from JSON input files. It analyzes the structure of JSON data and creates\n\
             corresponding schema definitions or code structures for various programming languages.\n\n\
             SUPPORTED FORMATS:\n  \
             - schema: JSON Schema Draft 2020-12 (default)\n  \
             - go: Go language structs with JSON tags\n  \
             - rust: Rust structs with serde annotations\n  \
             - typescript: TypeScript interfaces\n  \
             - python: Python dataclasses with type hints\n\n\
             EXAMPLES:\n  \
             j2s data.json                                    # Generate data.schema.json\n  \
             j2s --input data.json                            # Same as above using flag\n  \
             j2s data.json --output my-schema.json            # Custom output filename\n  \
             j2s -i data.json -o schema.json                  # Using short flags\n  \
             j2s data.json --format go                        # Generate Go struct\n  \
             j2s data.json --format rust --struct-name User   # Generate Rust struct with custom name\n  \
             j2s data.json -f typescript -s ApiResponse       # Generate TypeScript interface\n\n\
             PERFORMANCE:\n  \
             - Files up to 100MB are supported\n  \
             - Large files (>10MB) show progress indicators\n  \
             - Deep nesting is automatically limited to prevent stack overflow",
        )
        .arg(
            Arg::new("json_file")
                .help("Input JSON file path")
                .value_name("JSON_FILE")
                .index(1)
                .help_heading("INPUT"),
        )
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input JSON file path (alternative to positional argument)")
                .help_heading("INPUT"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path (default: <input_name>.<format_extension>)")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format: schema, go, rust, typescript, python (default: schema)")
                .help_heading("FORMAT"),
        )
        .arg(
            Arg::new("struct_name")
                .short('s')
                .long("struct-name")
                .value_name("NAME")
                .help("Custom name for generated struct/type/interface (default: derived from filename)")
                .help_heading("FORMAT"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_get_input_path_with_input() {
        let args = CliArgs {
            input: Some("test.json".to_string()),
            output: None,
            json_file: None,
            format: None,
            struct_name: None,
        };
        assert_eq!(args.get_input_path(), Some(&"test.json".to_string()));
    }

    #[test]
    fn test_cli_args_get_input_path_with_json_file() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: Some("test.json".to_string()),
            format: None,
            struct_name: None,
        };
        assert_eq!(args.get_input_path(), Some(&"test.json".to_string()));
    }

    #[test]
    fn test_cli_args_get_input_path_input_takes_precedence() {
        let args = CliArgs {
            input: Some("input.json".to_string()),
            output: None,
            json_file: Some("positional.json".to_string()),
            format: None,
            struct_name: None,
        };
        assert_eq!(args.get_input_path(), Some(&"input.json".to_string()));
    }

    #[test]
    fn test_cli_args_get_input_path_none() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: None,
            struct_name: None,
        };
        assert_eq!(args.get_input_path(), None);
    }

    #[test]
    fn test_build_cli_command_creation() {
        let cmd = build_cli();
        assert_eq!(cmd.get_name(), "j2s");
        assert_eq!(cmd.get_version(), Some(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_parse_args_with_positional() {
        let cmd = build_cli();
        let matches = cmd.try_get_matches_from(vec!["j2s", "input.json"]).unwrap();

        let args = CliArgs {
            input: matches.get_one::<String>("input").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            json_file: matches.get_one::<String>("json_file").cloned(),
            format: matches.get_one::<String>("format").cloned(),
            struct_name: matches.get_one::<String>("struct_name").cloned(),
        };

        assert_eq!(args.json_file, Some("input.json".to_string()));
        assert_eq!(args.input, None);
        assert_eq!(args.output, None);
        assert_eq!(args.format, None);
        assert_eq!(args.struct_name, None);
    }

    #[test]
    fn test_parse_args_with_input_flag() {
        let cmd = build_cli();
        let matches = cmd
            .try_get_matches_from(vec!["j2s", "--input", "test.json"])
            .unwrap();

        let args = CliArgs {
            input: matches.get_one::<String>("input").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            json_file: matches.get_one::<String>("json_file").cloned(),
            format: matches.get_one::<String>("format").cloned(),
            struct_name: matches.get_one::<String>("struct_name").cloned(),
        };

        assert_eq!(args.input, Some("test.json".to_string()));
        assert_eq!(args.json_file, None);
        assert_eq!(args.output, None);
        assert_eq!(args.format, None);
        assert_eq!(args.struct_name, None);
    }

    #[test]
    fn test_parse_args_with_output_flag() {
        let cmd = build_cli();
        let matches = cmd
            .try_get_matches_from(vec!["j2s", "input.json", "--output", "schema.json"])
            .unwrap();

        let args = CliArgs {
            input: matches.get_one::<String>("input").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            json_file: matches.get_one::<String>("json_file").cloned(),
            format: matches.get_one::<String>("format").cloned(),
            struct_name: matches.get_one::<String>("struct_name").cloned(),
        };

        assert_eq!(args.json_file, Some("input.json".to_string()));
        assert_eq!(args.output, Some("schema.json".to_string()));
        assert_eq!(args.input, None);
        assert_eq!(args.format, None);
        assert_eq!(args.struct_name, None);
    }

    #[test]
    fn test_parse_args_with_all_flags() {
        let cmd = build_cli();
        let matches = cmd
            .try_get_matches_from(vec![
                "j2s",
                "--input",
                "input.json",
                "--output",
                "output.json",
            ])
            .unwrap();

        let args = CliArgs {
            input: matches.get_one::<String>("input").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            json_file: matches.get_one::<String>("json_file").cloned(),
            format: matches.get_one::<String>("format").cloned(),
            struct_name: matches.get_one::<String>("struct_name").cloned(),
        };

        assert_eq!(args.input, Some("input.json".to_string()));
        assert_eq!(args.output, Some("output.json".to_string()));
        assert_eq!(args.json_file, None);
        assert_eq!(args.format, None);
        assert_eq!(args.struct_name, None);
    }

    #[test]
    fn test_parse_args_short_flags() {
        let cmd = build_cli();
        let matches = cmd
            .try_get_matches_from(vec!["j2s", "-i", "input.json", "-o", "output.json"])
            .unwrap();

        let args = CliArgs {
            input: matches.get_one::<String>("input").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            json_file: matches.get_one::<String>("json_file").cloned(),
            format: matches.get_one::<String>("format").cloned(),
            struct_name: matches.get_one::<String>("struct_name").cloned(),
        };

        assert_eq!(args.input, Some("input.json".to_string()));
        assert_eq!(args.output, Some("output.json".to_string()));
        assert_eq!(args.json_file, None);
        assert_eq!(args.format, None);
        assert_eq!(args.struct_name, None);
    }

    #[test]
    fn test_parse_args_integration() {
        // This test verifies that parse_args works with actual command line arguments
        // We can't easily test this with std::env::args, but we can test the build_cli function
        let mut cmd = build_cli();

        // Test that the command can be built without errors
        assert_eq!(cmd.get_name(), "j2s");

        // Test help text contains expected information
        let help_text = cmd.render_help().to_string();
        assert!(help_text.contains("Generate JSON Schema and code structures from JSON files"));
        assert!(help_text.contains("--input"));
        assert!(help_text.contains("--output"));
    }

    #[test]
    fn test_print_version_function() {
        // Test that print_version doesn't panic
        // We can't easily capture stdout in unit tests, but we can ensure it doesn't crash
        print_version();
    }

    #[test]
    fn test_print_help_function() {
        // Test that print_help doesn't panic
        print_help();
    }

    // Tests for struct name functionality
    #[test]
    fn test_clean_struct_name_basic() {
        assert_eq!(CliArgs::clean_struct_name("user"), "User");
        assert_eq!(CliArgs::clean_struct_name("user_data"), "UserData");
        assert_eq!(CliArgs::clean_struct_name("user-data"), "UserData");
        assert_eq!(CliArgs::clean_struct_name("user data"), "UserData");
    }

    #[test]
    fn test_clean_struct_name_empty() {
        assert_eq!(CliArgs::clean_struct_name(""), "Data");
    }

    #[test]
    fn test_clean_struct_name_numeric_start() {
        assert_eq!(CliArgs::clean_struct_name("123user"), "Data123user");
        assert_eq!(CliArgs::clean_struct_name("1_user_data"), "Data1UserData");
    }

    #[test]
    fn test_clean_struct_name_special_characters() {
        assert_eq!(CliArgs::clean_struct_name("user@data"), "UserData");
        assert_eq!(CliArgs::clean_struct_name("user#data$info"), "UserDataInfo");
        assert_eq!(CliArgs::clean_struct_name("user.data.info"), "UserDataInfo");
    }

    #[test]
    fn test_clean_struct_name_reserved_keywords() {
        assert_eq!(CliArgs::clean_struct_name("class"), "ClassType");
        assert_eq!(CliArgs::clean_struct_name("struct"), "StructType");
        assert_eq!(CliArgs::clean_struct_name("interface"), "InterfaceType");
        assert_eq!(CliArgs::clean_struct_name("type"), "TypeType");
        assert_eq!(CliArgs::clean_struct_name("function"), "FunctionType");
    }

    #[test]
    fn test_clean_struct_name_case_handling() {
        assert_eq!(CliArgs::clean_struct_name("USER"), "User");
        assert_eq!(CliArgs::clean_struct_name("user_DATA"), "UserData");
        assert_eq!(CliArgs::clean_struct_name("User_Data"), "UserData");
        // The test expects "userDATA" to be treated as one word, not split on camelCase
        // This suggests that consecutive uppercase letters should not trigger a split
        assert_eq!(CliArgs::clean_struct_name("userDATA"), "Userdata");
    }

    #[test]
    fn test_is_reserved_keyword() {
        // Test common keywords
        assert!(CliArgs::is_reserved_keyword("class"));
        assert!(CliArgs::is_reserved_keyword("struct"));
        assert!(CliArgs::is_reserved_keyword("interface"));
        assert!(CliArgs::is_reserved_keyword("function"));
        
        // Test case insensitivity
        assert!(CliArgs::is_reserved_keyword("CLASS"));
        assert!(CliArgs::is_reserved_keyword("Struct"));
        
        // Test non-keywords
        assert!(!CliArgs::is_reserved_keyword("user"));
        assert!(!CliArgs::is_reserved_keyword("data"));
        assert!(!CliArgs::is_reserved_keyword("custom"));
    }

    #[test]
    fn test_get_struct_name_with_custom_name() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: None,
            struct_name: Some("CustomUser".to_string()),
        };
        assert_eq!(args.get_struct_name(), "CustomUser");
    }

    #[test]
    fn test_get_struct_name_with_dirty_custom_name() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: None,
            struct_name: Some("custom_user-data".to_string()),
        };
        assert_eq!(args.get_struct_name(), "CustomUserData");
    }

    #[test]
    fn test_get_struct_name_from_filename() {
        let args = CliArgs {
            input: Some("user_data.json".to_string()),
            output: None,
            json_file: None,
            format: None,
            struct_name: None,
        };
        assert_eq!(args.get_struct_name(), "UserData");
    }

    #[test]
    fn test_get_struct_name_from_positional_filename() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: Some("api-response.json".to_string()),
            format: None,
            struct_name: None,
        };
        assert_eq!(args.get_struct_name(), "ApiResponse");
    }

    #[test]
    fn test_get_struct_name_default() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: None,
            struct_name: None,
        };
        assert_eq!(args.get_struct_name(), "Data");
    }

    #[test]
    fn test_validate_format_valid() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: Some("go".to_string()),
            struct_name: None,
        };
        assert!(args.validate_format().is_ok());

        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: Some("rust".to_string()),
            struct_name: None,
        };
        assert!(args.validate_format().is_ok());

        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: Some("typescript".to_string()),
            struct_name: None,
        };
        assert!(args.validate_format().is_ok());

        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: Some("python".to_string()),
            struct_name: None,
        };
        assert!(args.validate_format().is_ok());

        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: Some("schema".to_string()),
            struct_name: None,
        };
        assert!(args.validate_format().is_ok());
    }

    #[test]
    fn test_validate_format_invalid() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: Some("java".to_string()),
            struct_name: None,
        };
        assert!(args.validate_format().is_err());

        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: Some("invalid".to_string()),
            struct_name: None,
        };
        assert!(args.validate_format().is_err());
    }

    #[test]
    fn test_validate_format_none() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: None,
            struct_name: None,
        };
        assert!(args.validate_format().is_ok());
    }

    #[test]
    fn test_get_format_default() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: None,
            struct_name: None,
        };
        assert_eq!(args.get_format(), "schema");
    }

    #[test]
    fn test_get_format_specified() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
            format: Some("go".to_string()),
            struct_name: None,
        };
        assert_eq!(args.get_format(), "go");
    }

    #[test]
    fn test_parse_args_with_format_flag() {
        let cmd = build_cli();
        let matches = cmd
            .try_get_matches_from(vec!["j2s", "input.json", "--format", "go"])
            .unwrap();

        let args = CliArgs {
            input: matches.get_one::<String>("input").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            json_file: matches.get_one::<String>("json_file").cloned(),
            format: matches.get_one::<String>("format").cloned(),
            struct_name: matches.get_one::<String>("struct_name").cloned(),
        };

        assert_eq!(args.json_file, Some("input.json".to_string()));
        assert_eq!(args.format, Some("go".to_string()));
        assert_eq!(args.struct_name, None);
    }

    #[test]
    fn test_parse_args_with_struct_name_flag() {
        let cmd = build_cli();
        let matches = cmd
            .try_get_matches_from(vec!["j2s", "input.json", "--struct-name", "User"])
            .unwrap();

        let args = CliArgs {
            input: matches.get_one::<String>("input").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            json_file: matches.get_one::<String>("json_file").cloned(),
            format: matches.get_one::<String>("format").cloned(),
            struct_name: matches.get_one::<String>("struct_name").cloned(),
        };

        assert_eq!(args.json_file, Some("input.json".to_string()));
        assert_eq!(args.struct_name, Some("User".to_string()));
        assert_eq!(args.format, None);
    }

    #[test]
    fn test_parse_args_with_all_new_flags() {
        let cmd = build_cli();
        let matches = cmd
            .try_get_matches_from(vec![
                "j2s",
                "input.json",
                "--format",
                "rust",
                "--struct-name",
                "ApiResponse",
            ])
            .unwrap();

        let args = CliArgs {
            input: matches.get_one::<String>("input").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            json_file: matches.get_one::<String>("json_file").cloned(),
            format: matches.get_one::<String>("format").cloned(),
            struct_name: matches.get_one::<String>("struct_name").cloned(),
        };

        assert_eq!(args.json_file, Some("input.json".to_string()));
        assert_eq!(args.format, Some("rust".to_string()));
        assert_eq!(args.struct_name, Some("ApiResponse".to_string()));
    }

    #[test]
    fn test_parse_args_short_format_and_struct_name_flags() {
        let cmd = build_cli();
        let matches = cmd
            .try_get_matches_from(vec![
                "j2s",
                "input.json",
                "-f",
                "typescript",
                "-s",
                "UserData",
            ])
            .unwrap();

        let args = CliArgs {
            input: matches.get_one::<String>("input").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            json_file: matches.get_one::<String>("json_file").cloned(),
            format: matches.get_one::<String>("format").cloned(),
            struct_name: matches.get_one::<String>("struct_name").cloned(),
        };

        assert_eq!(args.json_file, Some("input.json".to_string()));
        assert_eq!(args.format, Some("typescript".to_string()));
        assert_eq!(args.struct_name, Some("UserData".to_string()));
    }

    #[test]
    fn test_generate_default_struct_name_complex_filename() {
        let args = CliArgs {
            input: Some("complex-api_response.v2.json".to_string()),
            output: None,
            json_file: None,
            format: None,
            struct_name: None,
        };
        assert_eq!(args.get_struct_name(), "ComplexApiResponseV2");
    }
}
