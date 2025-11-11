# varsubst

[![Crates.io](https://img.shields.io/crates/v/varsubst.svg)](https://crates.io/crates/varsubst)
[![Documentation](https://docs.rs/varsubst/badge.svg)](https://docs.rs/varsubst)
[![License](https://img.shields.io/crates/l/varsubst.svg)](https://github.com/AprilNEA/varsubst#license)
[![Downloads](https://img.shields.io/crates/d/varsubst.svg)](https://crates.io/crates/varsubst)

High-performance variable substitution (envsubst) library and CLI tool written in Rust with **O(n) single-pass parsing**.

## Features

- **High Performance**: O(n) time complexity with single-pass parsing
  - Unlike [coreos/envsubst-rs](https://github.com/coreos/envsubst-rs) which scans the string N times (once per variable)
  - Processes the entire string in a single scan using a state machine
- **Multiple Syntax Support**:
  - `${VAR}`: Standard brace-delimited variables (always supported)
  - `$VAR`: Short form variables (optional, enable with `short_syntax` feature)
- **Escape Sequences**: Support for `\$`, `\{`, `\}` (enabled by default)

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

## License

This project is dual-licensed under MIT OR Apache-2.0.

## Acknowledgments

Inspired by:
- [coreos/envsubst-rs](https://github.com/coreos/envsubst-rs)
- GNU `envsubst` utility
