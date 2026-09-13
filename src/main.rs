//! CLI entry point for tokenizing source files.

mod scanner;
mod token;

use scanner::Scanner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Preserve the Lab 0 greeting.
    if args.get(1).map(String::as_str) != Some("--tokenize") {
        println!("Hello, JM & Dejel!");
        return Ok(());
    }

    let filename = args.get(2).ok_or("Usage: ./run --tokenize <file>")?;
    let source = std::fs::read_to_string(filename)?;

    // Finish scanning before printing, so an error leaves stdout empty.
    let tokens = match Scanner::new(source).scan_tokens() {
        Ok(tokens) => tokens,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(65);
        }
    };
    for token in tokens {
        println!("{token}");
    }

    Ok(())
}
