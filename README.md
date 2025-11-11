# varsubst

High-performance variable substitution library and CLI tool written in Rust, inspired by `envsubst` but with **O(n) single-pass parsing** and additional features.

## Features

- ** High Performance**: O(n) time complexity with single-pass parsing
  - Unlike [coreos/envsubst-rs](https://github.com/coreos/envsubst-rs) which scans the string N times (once per variable)
  - Processes the entire string in a single scan using a state machine
- ** Multiple Syntax Support**:
  - `${VAR}`: Standard brace-delimited variables (always supported)
  - `$VAR`: Short form variables (optional, enable with `short_syntax` feature)
- ** Escape Sequences**: Support for `\$`, `\{`, `\}` (enabled by default)
- ** Feature Flags**: Modular functionality via Cargo features
- ** Well Tested**: Comprehensive test suite covering all features
- ** Both Library and CLI**: Use as a library or standalone tool

## Installation

### As a CLI tool

```bash
cargo install --path . --features cli
```

Note: The CLI is an optional feature to keep the library lightweight. When used as a library, clap is not included as a dependency.

### As a library

Add to your `Cargo.toml`:

```toml
[dependencies]
varsubst = "0.1"
```

## Usage

### Library Usage

#### Basic Example

```rust
use varsubst::substitute;
use std::collections::HashMap;

let mut vars = HashMap::new();
vars.insert("NAME", "World");
vars.insert("COUNT", "42");

let result = substitute("Hello ${NAME}! Count: ${COUNT}", &vars).unwrap();
assert_eq!(result, "Hello World! Count: 42");
```

#### With Escape Sequences (default)

```rust
use varsubst::substitute;
use std::collections::HashMap;

let vars = HashMap::new();
let result = substitute(r"Price: \${PRICE}", &vars).unwrap();
assert_eq!(result, "Price: ${PRICE}");
```

#### With Short Syntax (optional feature)

```toml
[dependencies]
varsubst = { version = "0.1", features = ["short_syntax"] }
```

```rust
use varsubst::substitute;
use std::collections::HashMap;

let mut vars = HashMap::new();
vars.insert("USER", "alice");
vars.insert("HOME", "/home/alice");

let result = substitute("User: $USER, Home: $HOME", &vars).unwrap();
assert_eq!(result, "User: alice, Home: /home/alice");
```

#### From Environment Variables

```rust
use varsubst::substitute_from_env;

std::env::set_var("MY_VAR", "test");
let result = substitute_from_env("Value: ${MY_VAR}").unwrap();
assert_eq!(result, "Value: test");
```

### CLI Usage

#### Basic Usage

```bash
# From stdin
echo "Hello ${USER}!" | varsubst

# From file
varsubst template.txt

# With custom variables
varsubst -v NAME=Alice -v AGE=30 template.txt

# Without environment variables
varsubst --no-env -v NAME=Alice template.txt

# Save to file
varsubst template.txt -o output.txt

# Fail on undefined variables
varsubst -f template.txt
```

#### Examples

```bash
# Create a template
cat > template.txt << 'EOF'
User: ${USER}
Home: ${HOME}
Custom: ${CUSTOM_VAR}
EOF

# Substitute with environment and custom variables
export CUSTOM_VAR="my value"
varsubst template.txt

# Output:
# User: your_username
# Home: /home/your_username
# Custom: my value
```

#### CLI Options

```
Usage: varsubst [OPTIONS] [FILE]

Arguments:
  [FILE]  Input file (or stdin if not specified)

Options:
  -o, --output <FILE>       Output file (or stdout if not specified)
  -v, --var <KEY=VALUE>     Define variables (format: KEY=VALUE)
      --no-env              Don't use environment variables
  -f, --fail-on-undefined   Fail if undefined variables are found
  -h, --help                Print help
  -V, --version             Print version
```

## Cargo Features

```toml
[features]
default = ["escape"]

# Support $X (short variable syntax without braces)
short_syntax = []

# Support escape sequences (\$, \{, \})
escape = []

# CLI binary (optional, includes clap for command-line interface)
cli = ["dep:clap"]
```

**Library Size**: The library without CLI feature is only ~63KB, while the CLI binary with clap is ~950KB.

### Feature Combinations

```bash
# Default (with escape)
cargo build

# With all features
cargo build --features short_syntax

# Minimal (no features)
cargo build --no-default-features

# Only short syntax (no escape)
cargo build --no-default-features --features short_syntax
```

## Performance Comparison

### varsubst vs envsubst-rs

| Scenario | envsubst-rs | varsubst | Improvement |
|----------|-------------|----------|-------------|
| Time Complexity | O(n × m) | O(n) | Linear vs Quadratic |
| String Scans | M (# of vars) | 1 | M× faster |
| Memory | Multiple passes | Single pass | More efficient |

**Example**: With 10 variables in a 1KB file:
- `envsubst-rs`: Scans 10KB total
- `varsubst`: Scans 1KB total (10× faster)

### Benchmark Results

Run benchmarks:
```bash
cargo bench --bench substitution
```

**Performance highlights** (on Apple Silicon):

| Scenario | Time | Description |
|----------|------|-------------|
| Single variable | ~198 ns | `Hello ${VAR}!` |
| Fast path (no vars) | **~29 ns** | Pure text, 7× faster |
| 10 variables | ~450 ns | Linear scaling |
| 100 variables | ~3.5 µs | Still blazing fast |
| 1KB template | ~2.1 µs | ~476 MB/s throughput |
| Real-world config | ~672 ns | 8 variables, 200 chars |

**Key optimizations**:
- ✅ Preprocessed lookup table: O(k·m) → O(k)
- ✅ Fast path for plain text: ~7× speedup
- ✅ Single-pass parsing: O(n) time complexity

See [OPTIMIZATION_RESULTS.md](OPTIMIZATION_RESULTS.md) for detailed analysis.

## Variable Naming Rules

Variable names must:
- Start with a letter (`a-z`, `A-Z`) or underscore (`_`)
- Contain only letters, digits, and underscores
- Be non-empty

Valid examples:
- `${VAR}`
- `${my_var}`
- `${_private}`
- `${VAR123}`

Invalid examples:
- `${}` (empty)
- `${123VAR}` (starts with digit)
- `${MY-VAR}` (contains hyphen)

## Error Handling

The library provides detailed error types:

```rust
pub enum SubstError {
    UnclosedBrace { position: usize },
    InvalidVarName { name: String, position: usize },
}
```

Example:

```rust
use varsubst::substitute;
use std::collections::HashMap;

let vars = HashMap::new();
match substitute("Hello ${NAME", &vars) {
    Err(varsubst::SubstError::UnclosedBrace { position }) => {
        println!("Unclosed brace at position {}", position);
    }
    _ => {}
}
```

## Testing

```bash
# Run all tests with default features
cargo test

# Test with all features
cargo test --all-features

# Test without default features
cargo test --no-default-features

# Test specific feature
cargo test --features short_syntax
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is dual-licensed under MIT OR Apache-2.0.

## Comparison with envsubst-rs

| Feature | envsubst-rs | varsubst |
|---------|-------------|----------|
| `${VAR}` syntax | ✅ | ✅ |
| `$VAR` syntax | ❌ | ✅ (feature) |
| Escape sequences | ❌ | ✅ (default) |
| Time complexity | O(n × m) | O(n) |
| Single-pass parsing | ❌ | ✅ |
| CLI tool | ❌ | ✅ |
| Error positions | ❌ | ✅ |

## Examples Directory

Check out the [examples](examples/) directory for more usage examples:

```bash
# Run library example
cargo run --example basic

# Run CLI example
cargo run -- examples/template.txt -v NAME=Alice
```

## Acknowledgments

Inspired by:
- [coreos/envsubst-rs](https://github.com/coreos/envsubst-rs)
- GNU `envsubst` utility
