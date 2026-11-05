//! CLI entry point for scripts, the earlier expression modes, and the REPL.

mod evaluator;
mod expr;
mod parser;
mod repl;
mod scanner;
mod stmt;
mod token;
mod value;

use parser::Parser;
use scanner::Scanner;

enum FileMode {
    Tokenize,
    Parse,
    Evaluate,
    Execute,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let (filename, mode) = match &args[1..] {
        [] => {
            repl::run_prompt(repl::PromptMode::Program)?;
            return Ok(());
        }
        [flag] if flag == "--repl" => {
            repl::run_prompt(repl::PromptMode::Program)?;
            return Ok(());
        }
        [flag] if flag == "--parse" => {
            repl::run_prompt(repl::PromptMode::Parse)?;
            return Ok(());
        }
        [flag] if flag == "--eval" || flag == "--evaluate" => {
            repl::run_prompt(repl::PromptMode::Evaluate)?;
            return Ok(());
        }
        [flag, filename] if flag == "--eval" || flag == "--evaluate" => {
            (filename, FileMode::Evaluate)
        }
        [flag, filename] if flag == "--tokenize" => (filename, FileMode::Tokenize),
        [flag, filename] if flag == "--parse" => (filename, FileMode::Parse),
        [filename] if !filename.starts_with('-') => (filename, FileMode::Execute),
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
    if matches!(mode, FileMode::Execute) {
        match Parser::new(tokens).parse_program() {
            Ok(statements) => {
                for statement in statements {
                    if let Err(message) = evaluator::execute(&statement) {
                        eprintln!("{message}");
                        std::process::exit(70);
                    }
                }
            }
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(65);
            }
        }
    } else if matches!(mode, FileMode::Parse | FileMode::Evaluate) {
        // Parse the whole file before printing, keeping syntax failures silent.
        match Parser::new(tokens).parse_all() {
            Ok(expressions) => {
                for expression in expressions {
                    if matches!(mode, FileMode::Evaluate) {
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
