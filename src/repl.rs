use std::io::{ self, Write };

use crate::scanner::Scanner;

pub fn run_prompt() -> io::Result<()> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut source = String::new();
        // Ctrl Z or C ends the REPL
        if io::stdin().read_line(&mut source)? == 0 {
            break;
        }

        match Scanner::new(source).scan_tokens() {
            Ok(tokens) => {
                for token in tokens {
                    println!("{token}");
                }
            }
            Err(message) => eprintln!("{message}"),
        }
    }

    Ok(())
}
