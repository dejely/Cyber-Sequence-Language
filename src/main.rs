//! CLI entry point for scanning files, parsing expressions, and the REPL.

mod expr;
mod parser;
mod repl;
mod scanner;
mod token;

use parser::Parser;
use scanner::Scanner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let (filename, parse) = match &args[1..] {
        [] => {
            repl::run_prompt()?;
            return Ok(());
        }
        [flag] if flag == "--repl" || flag == "--parse" => {
            repl::run_prompt()?;
            return Ok(());
        }
        [flag, filename] if flag == "--tokenize" => (filename, false),
        [flag, filename] if flag == "--parse" => (filename, true),
        [filename] if !filename.starts_with('-') => {
            // Preserve the original Lab 0 fixture invocation.
            if std::path::Path::new(filename)
                .extension()
                .and_then(|ext| ext.to_str())
                == Some("src")
            {
                println!("Hello, JM & Dejel!");
                return Ok(());
            }
            (filename, true)
        }
        _ => {
            return Err(
                "Usage: ./run [--repl | --parse [file] | --tokenize <file> | <file>]".into(),
            );
        }
    };
    // No arguments starts the prompt; keep --repl as an explicit alias.
    if args.len() == 1 || args.get(1).map(String::as_str) == Some("--repl") {
        repl::run_prompt()?;
        return Ok(());
    }

    if args.get(1).map(String::as_str) == Some("--parse") {
        repl::run_prompt()?;
        return Ok(()); // place holder
    }

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
    if parse {
        // Print only after every expression succeeds, keeping failed files silent.
        match Parser::new(tokens).parse_all() {
            Ok(expressions) => {
                for expression in expressions {
                    println!("{expression}");
                }
            }
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(65);
            }
        }
    } else {
        for token in tokens {
            println!("{token}");
        }
    }

    Ok(())
}
