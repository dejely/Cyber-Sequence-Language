use std::io::{self, Write};

use crate::parser::Parser;
use crate::scanner::Scanner;

pub fn run_prompt() -> io::Result<()> {
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
            .and_then(|tokens| Parser::new(tokens).parse());
        match result {
            Ok(expression) => println!("{expression}"),
            Err(message) => eprintln!("{message}"),
        }
    }

    Ok(())
}
