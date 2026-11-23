//! CLI entry point for scripts, the earlier expression modes, and the REPL.

mod environment;
mod evaluator;
mod expr;
mod function;
mod parser;
mod repl;
mod scanner;
mod stmt;
mod token;
mod value;
mod watch;

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
        [filename, options @ ..]
            if !filename.starts_with('-')
                && options.first().is_none_or(|option| {
                    matches!(option.as_str(), "--grant" | "--rank" | "--policy")
                }) =>
        {
            (filename, FileMode::Execute)
        }
        _ => {
            return Err(
                "Usage: ./run [--repl | --parse [file] | --eval [file] | --tokenize <file> | <file> [--grant <target> <capability>]... [--rank <0..9>] [--policy <file>]]".into(),
            );
        }
    };
    let options = if matches!(mode, FileMode::Execute) {
        watch::Options::parse(&args[2..])?
    } else {
        watch::Options::default()
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
    let mut interpreter = evaluator::Interpreter::default();
    if matches!(mode, FileMode::Execute) {
        match Parser::new(tokens).parse_program() {
            Ok(program) => {
                if !program.watches.is_empty() || !program.abilities.is_empty() {
                    if !program.statements.is_empty() {
                        eprintln!("A watch file accepts only watch and ability declarations.");
                        std::process::exit(65);
                    }
                    let input = std::io::stdin().lock();
                    let output = std::io::stdout().lock();
                    if let Err(error) =
                        watch::replay(&program.watches, &program.abilities, options, input, output)
                    {
                        eprintln!("{error}");
                        std::process::exit(70);
                    }
                    return Ok(());
                }
                for statement in program.statements {
                    if let Err(message) = interpreter.execute(&statement) {
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
                        match interpreter.evaluate(&expression) {
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
