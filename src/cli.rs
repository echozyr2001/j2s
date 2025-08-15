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

    CliArgs {
        input: matches.get_one::<String>("input").cloned(),
        output: matches.get_one::<String>("output").cloned(),
        json_file: matches.get_one::<String>("json_file").cloned(),
    }
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
///
/// # Examples
/// ```bash
/// j2s data.json                           # Basic usage
/// j2s --input data.json                   # Using input flag
/// j2s data.json --output schema.json      # Custom output path
/// j2s -i data.json -o schema.json         # Short flags
/// ```
fn build_cli() -> Command {
    Command::new("j2s")
        .version(env!("CARGO_PKG_VERSION"))
        .author("JSON to Schema Tool")
        .about("Generate JSON Schema from JSON files")
        .long_about(
            "j2s is a command-line tool that generates JSON Schema files from JSON input files.\n\
             It analyzes the structure of JSON data and creates corresponding schema definitions\n\
             following JSON Schema Draft 2020-12 specification.\n\n\
             EXAMPLES:\n  \
             j2s data.json                           # Generate data.schema.json\n  \
             j2s --input data.json                   # Same as above using flag\n  \
             j2s data.json --output my-schema.json   # Custom output filename\n  \
             j2s -i data.json -o schema.json         # Using short flags\n\n\
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
                .help("Output schema file path (default: <input_name>.schema.json)")
                .help_heading("OUTPUT"),
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
        };
        assert_eq!(args.get_input_path(), Some(&"test.json".to_string()));
    }

    #[test]
    fn test_cli_args_get_input_path_with_json_file() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: Some("test.json".to_string()),
        };
        assert_eq!(args.get_input_path(), Some(&"test.json".to_string()));
    }

    #[test]
    fn test_cli_args_get_input_path_input_takes_precedence() {
        let args = CliArgs {
            input: Some("input.json".to_string()),
            output: None,
            json_file: Some("positional.json".to_string()),
        };
        assert_eq!(args.get_input_path(), Some(&"input.json".to_string()));
    }

    #[test]
    fn test_cli_args_get_input_path_none() {
        let args = CliArgs {
            input: None,
            output: None,
            json_file: None,
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
        };

        assert_eq!(args.json_file, Some("input.json".to_string()));
        assert_eq!(args.input, None);
        assert_eq!(args.output, None);
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
        };

        assert_eq!(args.input, Some("test.json".to_string()));
        assert_eq!(args.json_file, None);
        assert_eq!(args.output, None);
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
        };

        assert_eq!(args.json_file, Some("input.json".to_string()));
        assert_eq!(args.output, Some("schema.json".to_string()));
        assert_eq!(args.input, None);
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
        };

        assert_eq!(args.input, Some("input.json".to_string()));
        assert_eq!(args.output, Some("output.json".to_string()));
        assert_eq!(args.json_file, None);
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
        };

        assert_eq!(args.input, Some("input.json".to_string()));
        assert_eq!(args.output, Some("output.json".to_string()));
        assert_eq!(args.json_file, None);
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
        assert!(help_text.contains("Generate JSON Schema from JSON files"));
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
}
