use super::Parser;
use crate::stmt::{AbilityDecl, AbilityStep, WatchDecl, WatchSequence};
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
        let sequence = if self.check(&TokenType::LeftBrace) {
            WatchSequence::Inline(self.alert_sequence()?)
        } else {
            let reference = self
                .consume(
                    &TokenType::Identifier,
                    "Expect a sequence block or ability name.",
                )?
                .clone();
            self.consume(&TokenType::Semicolon, "Expect ';' after ability name.")?;
            WatchSequence::Named(reference)
        };
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
            sequence,
        })
    }

    pub(super) fn ability_declaration(&mut self) -> Result<AbilityDecl, String> {
        self.advance(); // ability
        let name = self
            .consume(&TokenType::Identifier, "Expect an ability name.")?
            .clone();
        self.consume(&TokenType::LeftBrace, "Expect '{' after ability name.")?;

        self.consume(
            &TokenType::Target,
            "Expect 'target' as the first ability clause.",
        )?;
        let target = self.ability_input("target name")?;
        self.consume(&TokenType::Semicolon, "Expect ';' after target name.")?;

        self.consume(
            &TokenType::Require,
            "Expect 'require' after the target clause.",
        )?;
        self.consume(
            &TokenType::Capability,
            "Expect 'capability' after 'require'.",
        )?;
        let capability = self.ability_input("capability name")?;
        self.consume(&TokenType::Semicolon, "Expect ';' after capability name.")?;

        self.consume(
            &TokenType::Sequence,
            "Expect 'sequence' after the capability clause.",
        )?;
        let steps = self.ability_sequence()?;
        self.consume(
            &TokenType::RightBrace,
            "Expect '}' after ability declaration.",
        )?;
        Ok(AbilityDecl {
            name,
            target,
            capability,
            steps,
        })
    }

    fn ability_input(&mut self, label: &str) -> Result<String, String> {
        let input = self.watch_string(label)?;
        if !crate::stmt::valid_grant_name(&input) {
            return Err(Self::error_at(
                self.previous(),
                &format!("Invalid {label}."),
            ));
        }
        Ok(input)
    }

    fn alert_sequence(&mut self) -> Result<Vec<String>, String> {
        self.consume(&TokenType::LeftBrace, "Expect '{' after 'sequence'.")?;
        let mut alerts = Vec::new();
        loop {
            self.consume(&TokenType::Alert, "Expect 'alert' in the sequence.")?;
            alerts.push(self.alert_message()?);
            if !self.check(&TokenType::Alert) {
                break;
            }
        }
        self.consume(
            &TokenType::RightBrace,
            "Expect 'alert' or '}' after alert step.",
        )?;
        Ok(alerts)
    }

    fn ability_sequence(&mut self) -> Result<Vec<AbilityStep>, String> {
        self.consume(&TokenType::LeftBrace, "Expect '{' after 'sequence'.")?;
        let mut steps = Vec::new();
        loop {
            steps.push(self.ability_step()?);
            if self.match_token(&[TokenType::RightBrace]) {
                return Ok(steps);
            }
        }
    }

    fn ability_step(&mut self) -> Result<AbilityStep, String> {
        let step = match self.peek().token_type() {
            TokenType::Alert => {
                self.advance();
                return self.alert_message().map(AbilityStep::Alert);
            }
            TokenType::Inspect => AbilityStep::Inspect,
            TokenType::Isolate => AbilityStep::Isolate,
            TokenType::Execute => AbilityStep::Execute,
            _ => return Err(self.error("Expect 'alert', 'inspect', 'isolate', or 'execute'.")),
        };
        self.advance();
        self.consume(&TokenType::Semicolon, "Expect ';' after ability step.")?;
        Ok(step)
    }

    fn alert_message(&mut self) -> Result<String, String> {
        let message = self.watch_string("alert message")?;
        self.consume(&TokenType::Semicolon, "Expect ';' after alert message.")?;
        Ok(message)
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

    pub(super) fn at_domain_declaration(&self) -> bool {
        matches!(
            self.peek().token_type(),
            TokenType::Watch | TokenType::Ability
        )
    }

    pub(super) fn synchronize_domain_declaration(&mut self, start: usize) {
        // A failed domain declaration has not consumed its closing brace.
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
        // A new declaration can still be parsed if a closing brace is missing.
        while !self.is_at_end() && !self.at_domain_declaration() {
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
