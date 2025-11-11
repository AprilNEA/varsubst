#[cfg(not(feature = "cli"))]
compile_error!("The binary requires the 'cli' feature. Use: cargo build --features cli");

use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::process;

/// High-performance variable substitution tool with single-pass parsing
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (or stdin if not specified)
    #[arg(value_name = "FILE")]
    input: Option<String>,

    /// Output file (or stdout if not specified)
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Define variables (format: KEY=VALUE)
    #[arg(short = 'v', long = "var", value_name = "KEY=VALUE")]
    variables: Vec<String>,

    /// Don't use environment variables (by default, environment variables are used)
    #[arg(long = "no-env")]
    no_env: bool,

    /// Fail if undefined variables are found
    #[arg(short = 'f', long = "fail-on-undefined")]
    fail_on_undefined: bool,
}

fn main() {
    let args = Args::parse();

    // Read input
    let input = match read_input(&args.input) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            process::exit(1);
        }
    };

    // Build variable map
    let mut vars: HashMap<String, String> = HashMap::new();

    // Add environment variables if requested (default behavior unless --no-env is specified)
    if !args.no_env {
        for (key, value) in std::env::vars() {
            vars.insert(key, value);
        }
    }

    // Add command-line variables (overrides environment)
    for var in &args.variables {
        match var.split_once('=') {
            Some((key, value)) => {
                vars.insert(key.to_string(), value.to_string());
            }
            None => {
                eprintln!("Invalid variable format: '{}' (expected KEY=VALUE)", var);
                process::exit(1);
            }
        }
    }

    // Perform substitution
    let result = match varsubst::substitute(&input, &vars) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Substitution error: {}", e);
            process::exit(1);
        }
    };

    // Check for undefined variables if requested
    if args.fail_on_undefined && result.contains("${") {
        eprintln!("Error: Undefined variables found in output");
        process::exit(1);
    }

    // Write output
    if let Err(e) = write_output(&args.output, &result) {
        eprintln!("Error writing output: {}", e);
        process::exit(1);
    }
}

/// Read input from file or stdin
fn read_input(path: &Option<String>) -> io::Result<String> {
    match path {
        Some(file_path) => fs::read_to_string(file_path),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

/// Write output to file or stdout
fn write_output(path: &Option<String>, content: &str) -> io::Result<()> {
    match path {
        Some(file_path) => fs::write(file_path, content),
        None => {
            print!("{}", content);
            Ok(())
        }
    }
}
