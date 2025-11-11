use std::collections::HashMap;
use varsubst::substitute;

fn main() {
    // Example 1: Basic substitution
    println!("=== Example 1: Basic Substitution ===");
    let mut vars = HashMap::new();
    vars.insert("NAME", "Alice");
    vars.insert("AGE", "30");
    vars.insert("CITY", "New York");

    let template = "Name: ${NAME}, Age: ${AGE}, City: ${CITY}";
    match substitute(template, &vars) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 2: Undefined variables
    println!("\n=== Example 2: Undefined Variables ===");
    let template = "Hello ${NAME}, your score is ${SCORE}";
    match substitute(template, &vars) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 3: Empty variables
    println!("\n=== Example 3: Empty Variables ===");
    vars.insert("EMPTY", "");
    let template = "Before${EMPTY}After";
    match substitute(template, &vars) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 4: Multiple occurrences
    println!("\n=== Example 4: Multiple Occurrences ===");
    let template = "${NAME} likes ${NAME}'s job in ${CITY}";
    match substitute(template, &vars) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 5: Adjacent variables
    println!("\n=== Example 5: Adjacent Variables ===");
    vars.insert("FIRST", "John");
    vars.insert("LAST", "Doe");
    let template = "Full name: ${FIRST}${LAST}";
    match substitute(template, &vars) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 6: With escape sequences (if escape feature is enabled)
    #[cfg(feature = "escape")]
    {
        println!("\n=== Example 6: Escape Sequences ===");
        let template = r"Price: \$${AGE}, Literal: \${NAME\}";
        match substitute(template, &vars) {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    // Example 7: With short syntax (if short_syntax feature is enabled)
    #[cfg(feature = "short_syntax")]
    {
        println!("\n=== Example 7: Short Syntax ===");
        let template = "Name: $NAME, Age: $AGE";
        match substitute(template, &vars) {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    // Example 8: Error handling
    println!("\n=== Example 8: Error Handling ===");
    let template = "Unclosed: ${NAME";
    match substitute(template, &vars) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 9: From environment
    println!("\n=== Example 9: From Environment ===");
    std::env::set_var("MY_ENV_VAR", "from environment");
    match varsubst::substitute_from_env("Value: ${MY_ENV_VAR}") {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
