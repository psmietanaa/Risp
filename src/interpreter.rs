use crate::eval::{eval, Environment, EvalResult};
use crate::lex::lex;
use crate::parse::parse;

use std::fs;
use std::io;
use std::io::Write;

/// Interactive REPL.
pub fn repl() {
    println!("Welcome to RustLisp!");
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        // Read user input
        io::stdin()
            .read_line(&mut input)
            .ok()
            .expect("Failed to read user input!");
        // Interpret the input
        match run_interpreter(&input.trim()) {
            EvalResult::Err(error) => println!("{}", error),
            _ => continue,
        }
    }
}

/// Interpret a file.
pub fn file(path: &str) {
    // Read file
    match fs::read_to_string(path) {
        Ok(content) => {
            // Print the returned result only if it is an error
            match run_interpreter(&content) {
                EvalResult::Err(error) => println!("{}", error),
                _ => return,
            }
        }
        Err(_) => println!("Unable to open the file!"),
    }
}

/// Lexes, parses, and evaluates the given program.
pub fn run_interpreter(program: &str) -> EvalResult {
    match lex(program) {
        Ok(tokens) => match parse(&tokens) {
            Ok(expr) => {
                let mut env = Environment::default();
                eval(expr.clone(), &mut env)
            }
            Err(error) => EvalResult::Err(format!("Parse error: {:?}", error)),
        },
        Err(error) => EvalResult::Err(format!("Lex error: {:?}", error)),
    }
}
