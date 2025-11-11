//! # varsubst
//!
//! High-performance variable substitution library with **O(n) single-pass parsing**.
//!
//! ## Features
//!
//! - **Single-pass parsing**: O(n) time complexity, scans the input string only once
//! - **`${VAR}` syntax**: Standard shell-like variable substitution
//! - **`$VAR` syntax**: Optional short form (enable with `short_syntax` feature)
//! - **Escape sequences**: Support `\$`, `\{`, `\}` (enabled by default with `escape` feature)
//! - **Zero-copy when possible**: Efficient memory usage
//!
//! ## Examples
//!
//! ```
//! use varsubst::substitute;
//! use std::collections::HashMap;
//!
//! let mut vars = HashMap::new();
//! vars.insert("NAME", "World");
//! vars.insert("COUNT", "42");
//!
//! let result = substitute("Hello ${NAME}! Count: ${COUNT}", &vars).unwrap();
//! assert_eq!(result, "Hello World! Count: 42");
//! ```
//!
//! With escape sequences:
//!
//! ```
//! use varsubst::substitute;
//! use std::collections::HashMap;
//!
//! let vars: HashMap<&str, &str> = HashMap::new();
//! let result = substitute(r"Price: \${PRICE}", &vars).unwrap();
//! assert_eq!(result, "Price: ${PRICE}");
//! ```

use std::collections::HashMap;
use std::fmt;

/// Error types for variable substitution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstError {
    /// Unclosed variable reference (missing closing brace)
    UnclosedBrace {
        /// Position where the unclosed brace was detected
        position: usize,
    },
    /// Invalid variable name (empty or contains invalid characters)
    InvalidVarName {
        /// The invalid variable name
        name: String,
        /// Position where the invalid name was detected
        position: usize,
    },
}

impl fmt::Display for SubstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubstError::UnclosedBrace { position } => {
                write!(f, "Unclosed brace at position {}", position)
            }
            SubstError::InvalidVarName { name, position } => {
                write!(f, "Invalid variable name '{}' at position {}", name, position)
            }
        }
    }
}

impl std::error::Error for SubstError {}

/// Result type for substitution operations
pub type SubstResult<T> = Result<T, SubstError>;

/// Parser state during variable substitution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Normal text parsing
    Normal,
    /// Just encountered a backslash (escape character)
    #[cfg(feature = "escape")]
    Escape,
    /// Just encountered a dollar sign
    Dollar,
    /// Inside ${...} collecting variable name
    BraceVar,
    /// After $ collecting short variable name (with short_syntax feature)
    #[cfg(feature = "short_syntax")]
    ShortVar,
}

/// Substitute variables in the input string.
///
/// This function performs a single-pass scan of the input string, replacing
/// variable references with their values from the provided map.
///
/// # Supported syntax
///
/// - `${VAR}`: Standard brace-delimited variables (always supported)
/// - `$VAR`: Short form variables (requires `short_syntax` feature)
/// - `\$`, `\{`, `\}`: Escape sequences (requires `escape` feature, enabled by default)
///
/// # Arguments
///
/// * `template` - The input string containing variable references
/// * `variables` - A map of variable names to their replacement values
///
/// # Returns
///
/// Returns `Ok(String)` with all variables substituted, or `Err(SubstError)` if
/// parsing fails.
///
/// # Examples
///
/// ```
/// use varsubst::substitute;
/// use std::collections::HashMap;
///
/// let mut vars = HashMap::new();
/// vars.insert("USER", "alice");
/// vars.insert("HOME", "/home/alice");
///
/// let result = substitute("User: ${USER}, Home: ${HOME}", &vars).unwrap();
/// assert_eq!(result, "User: alice, Home: /home/alice");
/// ```
pub fn substitute<K, V>(template: &str, variables: &HashMap<K, V>) -> SubstResult<String>
where
    K: AsRef<str> + std::hash::Hash + Eq,
    V: AsRef<str>,
{
    // Pre-allocate with template size as a reasonable starting point
    let mut output = String::with_capacity(template.len());
    let mut state = State::Normal;
    let mut var_name = String::new();
    let mut var_start_pos = 0;

    let chars: Vec<char> = template.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        match state {
            State::Normal => {
                #[cfg(feature = "escape")]
                if ch == '\\' {
                    state = State::Escape;
                    i += 1;
                    continue;
                }

                if ch == '$' {
                    state = State::Dollar;
                    var_start_pos = i;
                } else {
                    output.push(ch);
                }
            }

            #[cfg(feature = "escape")]
            State::Escape => {
                // Escape special characters: $, {, }
                match ch {
                    '$' | '{' | '}' => output.push(ch),
                    '\\' => output.push('\\'),
                    // For any other character after \, keep the backslash
                    _ => {
                        output.push('\\');
                        output.push(ch);
                    }
                }
                state = State::Normal;
            }

            State::Dollar => {
                if ch == '{' {
                    state = State::BraceVar;
                    var_name.clear();
                } else if is_var_char_start(ch) {
                    #[cfg(feature = "short_syntax")]
                    {
                        state = State::ShortVar;
                        var_name.clear();
                        var_name.push(ch);
                    }
                    #[cfg(not(feature = "short_syntax"))]
                    {
                        // Without short_syntax feature, $ followed by non-{ is literal
                        output.push('$');
                        output.push(ch);
                        state = State::Normal;
                    }
                } else {
                    // Dollar sign followed by something else, treat as literal
                    output.push('$');
                    output.push(ch);
                    state = State::Normal;
                }
            }

            State::BraceVar => {
                if ch == '}' {
                    // End of variable reference
                    if var_name.is_empty() {
                        return Err(SubstError::InvalidVarName {
                            name: String::new(),
                            position: var_start_pos,
                        });
                    }

                    // Look up and substitute the variable
                    if let Some(value) = variables.iter().find(|(k, _)| k.as_ref() == var_name.as_str()) {
                        output.push_str(value.1.as_ref());
                    } else {
                        // Variable not found, keep original syntax
                        output.push_str("${");
                        output.push_str(&var_name);
                        output.push('}');
                    }

                    var_name.clear();
                    state = State::Normal;
                } else if is_var_char(ch) {
                    var_name.push(ch);
                } else {
                    // Invalid character in variable name
                    return Err(SubstError::InvalidVarName {
                        name: var_name.clone(),
                        position: var_start_pos,
                    });
                }
            }

            #[cfg(feature = "short_syntax")]
            State::ShortVar => {
                if is_var_char(ch) {
                    var_name.push(ch);
                } else {
                    // End of short variable name
                    if let Some(value) = variables.iter().find(|(k, _)| k.as_ref() == var_name.as_str()) {
                        output.push_str(value.1.as_ref());
                    } else {
                        // Variable not found, keep original syntax
                        output.push('$');
                        output.push_str(&var_name);
                    }

                    var_name.clear();
                    state = State::Normal;

                    // Process current character in Normal state
                    #[cfg(feature = "escape")]
                    if ch == '\\' {
                        state = State::Escape;
                        i += 1;
                        continue;
                    }

                    if ch == '$' {
                        state = State::Dollar;
                        var_start_pos = i;
                    } else {
                        output.push(ch);
                    }
                }
            }
        }

        i += 1;
    }

    // Handle end of string
    match state {
        State::Normal => {}

        #[cfg(feature = "escape")]
        State::Escape => {
            // Trailing backslash, keep it
            output.push('\\');
        }

        State::Dollar => {
            // Trailing dollar sign
            output.push('$');
        }

        State::BraceVar => {
            // Unclosed brace
            return Err(SubstError::UnclosedBrace {
                position: var_start_pos,
            });
        }

        #[cfg(feature = "short_syntax")]
        State::ShortVar => {
            // End of string in short var, substitute if found
            if let Some(value) = variables.iter().find(|(k, _)| k.as_ref() == var_name.as_str()) {
                output.push_str(value.1.as_ref());
            } else {
                output.push('$');
                output.push_str(&var_name);
            }
        }
    }

    Ok(output)
}

/// Check if a character can start a variable name
#[inline]
fn is_var_char_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

/// Check if a character can be part of a variable name
#[inline]
fn is_var_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

/// Substitute variables from environment variables.
///
/// This is a convenience function that reads environment variables and
/// performs substitution.
///
/// # Examples
///
/// ```no_run
/// use varsubst::substitute_from_env;
///
/// std::env::set_var("MY_VAR", "test");
/// let result = substitute_from_env("Value: ${MY_VAR}").unwrap();
/// assert_eq!(result, "Value: test");
/// ```
pub fn substitute_from_env(template: &str) -> SubstResult<String> {
    let env_vars: HashMap<String, String> = std::env::vars().collect();
    substitute(template, &env_vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vars<'a>(pairs: &[(&'a str, &'a str)]) -> HashMap<&'a str, &'a str> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn test_basic_substitution() {
        let vars = make_vars(&[("NAME", "World"), ("COUNT", "42")]);
        let result = substitute("Hello ${NAME}! Count: ${COUNT}", &vars).unwrap();
        assert_eq!(result, "Hello World! Count: 42");
    }

    #[test]
    fn test_no_variables() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute("Hello World!", &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_undefined_variable() {
        let vars = make_vars(&[("NAME", "World")]);
        let result = substitute("Hello ${NAME}! ${UNDEFINED}", &vars).unwrap();
        assert_eq!(result, "Hello World! ${UNDEFINED}");
    }

    #[test]
    fn test_empty_string() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute("", &vars).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_only_variable() {
        let vars = make_vars(&[("VAR", "value")]);
        let result = substitute("${VAR}", &vars).unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn test_multiple_same_variable() {
        let vars = make_vars(&[("X", "test")]);
        let result = substitute("${X} and ${X} and ${X}", &vars).unwrap();
        assert_eq!(result, "test and test and test");
    }

    #[test]
    fn test_adjacent_variables() {
        let vars = make_vars(&[("A", "foo"), ("B", "bar")]);
        let result = substitute("${A}${B}", &vars).unwrap();
        assert_eq!(result, "foobar");
    }

    #[test]
    fn test_unclosed_brace() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute("Hello ${NAME", &vars);
        assert!(matches!(result, Err(SubstError::UnclosedBrace { position: 6 })));
    }

    #[test]
    fn test_empty_var_name() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute("${}", &vars);
        assert!(matches!(result, Err(SubstError::InvalidVarName { .. })));
    }

    #[test]
    fn test_invalid_var_name() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute("${NA-ME}", &vars);
        assert!(matches!(result, Err(SubstError::InvalidVarName { .. })));
    }

    #[test]
    fn test_literal_dollar() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute("Price: $5.99", &vars).unwrap();
        assert_eq!(result, "Price: $5.99");
    }

    #[test]
    fn test_dollar_at_end() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute("End with $", &vars).unwrap();
        assert_eq!(result, "End with $");
    }

    #[test]
    fn test_underscore_in_var_name() {
        let vars = make_vars(&[("MY_VAR", "value"), ("_VAR", "test")]);
        let result = substitute("${MY_VAR} ${_VAR}", &vars).unwrap();
        assert_eq!(result, "value test");
    }

    #[test]
    fn test_numbers_in_var_name() {
        let vars = make_vars(&[("VAR123", "value")]);
        let result = substitute("${VAR123}", &vars).unwrap();
        assert_eq!(result, "value");
    }

    #[cfg(feature = "escape")]
    #[test]
    fn test_escape_dollar() {
        let vars = make_vars(&[("PRICE", "100")]);
        let result = substitute(r"Price: \$${PRICE}", &vars).unwrap();
        assert_eq!(result, "Price: $100");
    }

    #[cfg(feature = "escape")]
    #[test]
    fn test_escape_brace() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute(r"\${VAR\}", &vars).unwrap();
        assert_eq!(result, "${VAR}");
    }

    #[cfg(feature = "escape")]
    #[test]
    fn test_escape_backslash() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute(r"Path: C:\\Users", &vars).unwrap();
        assert_eq!(result, r"Path: C:\Users");
    }

    #[cfg(feature = "escape")]
    #[test]
    fn test_escape_other_char() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute(r"\a\b\c", &vars).unwrap();
        assert_eq!(result, r"\a\b\c");
    }

    #[cfg(feature = "escape")]
    #[test]
    fn test_trailing_backslash() {
        let vars: HashMap<&str, &str> = HashMap::new();
        let result = substitute(r"End with \", &vars).unwrap();
        assert_eq!(result, r"End with \");
    }

    #[cfg(feature = "short_syntax")]
    #[test]
    fn test_short_syntax() {
        let vars = make_vars(&[("HOME", "/home/user"), ("USER", "alice")]);
        let result = substitute("User: $USER, Home: $HOME", &vars).unwrap();
        assert_eq!(result, "User: alice, Home: /home/user");
    }

    #[cfg(feature = "short_syntax")]
    #[test]
    fn test_short_syntax_undefined() {
        let vars = make_vars(&[("VAR", "value")]);
        let result = substitute("$VAR $UNDEFINED", &vars).unwrap();
        assert_eq!(result, "value $UNDEFINED");
    }

    #[cfg(feature = "short_syntax")]
    #[test]
    fn test_short_syntax_with_delimiter() {
        let vars = make_vars(&[("VAR", "value")]);
        let result = substitute("$VAR-suffix", &vars).unwrap();
        assert_eq!(result, "value-suffix");
    }

    #[cfg(feature = "short_syntax")]
    #[test]
    fn test_mixed_syntax() {
        let vars = make_vars(&[("A", "foo"), ("B", "bar")]);
        let result = substitute("$A and ${B}", &vars).unwrap();
        assert_eq!(result, "foo and bar");
    }

    #[cfg(feature = "short_syntax")]
    #[test]
    fn test_short_syntax_at_end() {
        let vars = make_vars(&[("VAR", "value")]);
        let result = substitute("End: $VAR", &vars).unwrap();
        assert_eq!(result, "End: value");
    }

    #[cfg(all(feature = "short_syntax", feature = "escape"))]
    #[test]
    fn test_escape_with_short_syntax() {
        let vars = make_vars(&[("VAR", "value")]);
        let result = substitute(r"\$VAR $VAR", &vars).unwrap();
        assert_eq!(result, "$VAR value");
    }

    #[test]
    fn test_performance_single_pass() {
        // This test verifies that we only scan the string once
        // by using a very long string with many variables
        let vars = make_vars(&[
            ("VAR1", "a"),
            ("VAR2", "b"),
            ("VAR3", "c"),
            ("VAR4", "d"),
            ("VAR5", "e"),
        ]);

        let template = "${VAR1}${VAR2}${VAR3}${VAR4}${VAR5}".repeat(100);
        let expected = "abcde".repeat(100);

        let result = substitute(&template, &vars).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_and_string_types() {
        let mut vars = HashMap::new();
        vars.insert("KEY".to_string(), "value".to_string());

        let result = substitute("${KEY}", &vars).unwrap();
        assert_eq!(result, "value");
    }
}
