use std::io::{self, Write};

use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::stmt::Stmt;

pub enum PromptMode {
    Program,
    Parse,
    Evaluate,
}

pub fn run_prompt(mode: PromptMode) -> io::Result<()> {
    let mut interpreter = crate::evaluator::Interpreter::default();
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut source = String::new();
        // EOF ends the REPL (Ctrl-D on Unix).
        if io::stdin().read_line(&mut source)? == 0 {
            break;
        }

        // read_line includes the line ending; it is not part of the entered expression.
        let result = Scanner::new(source.trim_end_matches(['\r', '\n']).to_string())
            .scan_tokens()
            .and_then(|tokens| {
                let mut parser = Parser::new(tokens);
                if matches!(mode, PromptMode::Program) {
                    parser.parse_repl_statement()
                } else {
                    parser.parse().map(Stmt::Expression)
                }
            });
        match result {
            Ok(Stmt::Expression(expression)) => {
                if matches!(mode, PromptMode::Parse) {
                    println!("{expression}");
                } else {
                    match interpreter.evaluate(&expression) {
                        Ok(value) => println!("{value}"),
                        Err(message) => eprintln!("{message}"),
                    }
                }
            }
            Ok(statement) => {
                if let Err(message) = interpreter.execute(&statement) {
                    eprintln!("{message}");
                }
            }
            Err(message) => eprintln!("{message}"),
        }
    }

    Ok(())
}
