use super::Parser;
use crate::stmt::WatchDecl;
use crate::token::{Literal, TokenType};

impl Parser {
    pub(super) fn watch_declaration(&mut self) -> Result<WatchDecl, String> {
        self.advance(); // watch
        let name = self
            .consume(&TokenType::Identifier, "Expect a watch name.")?
            .clone();
        self.consume(&TokenType::LeftBrace, "Expect '{' after watch name.")?;

        self.consume(
            &TokenType::Source,
            "Expect 'source' as the first watch clause.",
        )?;
        let source = self.watch_string("source name")?;
        self.consume(&TokenType::Semicolon, "Expect ';' after source name.")?;

        self.consume(&TokenType::Count, "Expect 'count' after the source clause.")?;
        let kind = self.watch_string("event kind")?;
        self.consume(&TokenType::By, "Expect 'by' after event kind.")?;
        let group_field = self.watch_string("group field")?;
        self.consume(&TokenType::Within, "Expect 'within' after group field.")?;
        let window = self.watch_integer("window in seconds")?;
        self.consume(&TokenType::GreaterEqual, "Expect '>=' after window.")?;
        let threshold = self.watch_integer("count threshold")?;
        self.consume(&TokenType::Semicolon, "Expect ';' after count threshold.")?;

        self.consume(
            &TokenType::Sequence,
            "Expect 'sequence' after the count clause.",
        )?;
        self.consume(&TokenType::LeftBrace, "Expect '{' after 'sequence'.")?;
        let mut alerts = Vec::new();
        loop {
            self.consume(&TokenType::Alert, "Expect 'alert' in the watch sequence.")?;
            alerts.push(self.watch_string("alert message")?);
            self.consume(&TokenType::Semicolon, "Expect ';' after alert message.")?;
            if !self.check(&TokenType::Alert) {
                break;
            }
        }
        self.consume(
            &TokenType::RightBrace,
            "Expect 'alert' or '}' after alert step.",
        )?;
        self.consume(
            &TokenType::RightBrace,
            "Expect '}' after watch declaration.",
        )?;

        Ok(WatchDecl {
            name,
            source,
            kind,
            group_field,
            window,
            threshold,
            alerts,
        })
    }

    fn watch_string(&mut self, label: &str) -> Result<String, String> {
        let token = self.consume(&TokenType::String, &format!("Expect a string for {label}."))?;
        match token.literal() {
            Some(Literal::String(value)) if !value.is_empty() => Ok(value.clone()),
            _ => Err(Self::error_at(
                token,
                &format!("The {label} must be nonempty."),
            )),
        }
    }

    fn watch_integer(&mut self, label: &str) -> Result<u32, String> {
        let message = format!("The {label} must be a positive 32-bit unsigned integer.");
        let token = self.consume(&TokenType::Number, &message)?;
        // Use the original digits so a small fraction cannot round to an integer.
        let (whole, fraction) = token
            .lexeme()
            .split_once('.')
            .unwrap_or((token.lexeme(), ""));
        match whole.parse::<u32>() {
            Ok(value) if value > 0 && fraction.bytes().all(|digit| digit == b'0') => Ok(value),
            _ => Err(Self::error_at(token, &message)),
        }
    }

    pub(super) fn synchronize_watch(&mut self, start: usize) {
        // The watch parser can only fail before consuming its closing brace.
        let mut depth = 0;
        for token in &self.tokens[start..self.current] {
            match token.token_type() {
                TokenType::LeftBrace => depth += 1,
                TokenType::RightBrace => depth -= 1,
                _ => {}
            }
        }
        if depth == 0 {
            self.synchronize_statement(start, false);
            return;
        }
        // A new watch can still be parsed if a closing brace is missing.
        while !self.is_at_end() && !self.check(&TokenType::Watch) {
            match self.advance().token_type() {
                TokenType::LeftBrace => depth += 1,
                TokenType::RightBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                _ => {}
            }
        }
    }
}
