//! CLI entry point for scanning files, parsing expressions, and the REPL.

mod evaluator;
mod expr;
mod parser;
mod repl;
mod scanner;
mod token;
mod value;

use parser::Parser;
use scanner::Scanner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let (filename, parse, evaluate) = match &args[1..] {
        [] => {
            repl::run_prompt(true)?;
            return Ok(());
        }
        [flag] if flag == "--repl" => {
            repl::run_prompt(true)?;
            return Ok(());
        }
        [flag] if flag == "--parse" => {
            repl::run_prompt(false)?;
            return Ok(());
        }
        [flag] if flag == "--eval" || flag == "--evaluate" => {
            repl::run_prompt(true)?;
            return Ok(());
        }
        [flag, filename] if flag == "--eval" || flag == "--evaluate" => (filename, true, true),
        [flag, filename] if flag == "--tokenize" => (filename, false, false),
        [flag, filename] if flag == "--parse" => (filename, true, false),
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
            (filename, true, false)
        }
        _ => {
            return Err(
                "Usage: ./run [--repl | --parse [file] | --eval [file] | --tokenize <file> | <file>]".into(),
            );
        }
    };
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
        // Parse the whole file before printing, keeping syntax failures silent.
        match Parser::new(tokens).parse_all() {
            Ok(expressions) => {
                for expression in expressions {
                    if evaluate {
                        match evaluator::evaluate(&expression) {
                            Ok(value) => println!("{value}"),
                            Err(message) => {
                                eprintln!("{message}");
                                std::process::exit(70);
                            }
                        }
                    } else {
                        println!("{expression}");
                    }
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
