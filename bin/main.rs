use rust_lisp::interpreter::*;
use std::env;

fn main() {
    // Read command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        // Interactive REPL.
        repl();
    } else {
        // Interpret a file.
        let path = &args[1];
        file(path);
    }
}
