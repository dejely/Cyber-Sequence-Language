use super::Parser;
use crate::stmt::{AbilityDecl, WatchDecl, WatchSequence};
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
        let alerts = self.alert_sequence()?;
        self.consume(
            &TokenType::RightBrace,
            "Expect '}' after ability declaration.",
        )?;
        Ok(AbilityDecl {
            name,
            target,
            capability,
            alerts,
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
        Ok(alerts)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Program;
    use crate::scanner::Scanner;
    use crate::stmt::Stmt;

    fn parse(source: &str) -> Result<Program, String> {
        Parser::new(Scanner::new(source.into()).scan_tokens().unwrap()).parse_program()
    }

    fn watch(name: &str, window: &str, threshold: &str) -> String {
        format!(
            r#"watch {name} {{ source "events"; count "kind" by "group" within {window} >= {threshold}; sequence {{ alert "hit"; }} }}"#
        )
    }

    #[test]
    fn preserves_watch_fields_alert_order_and_script_statements() {
        let fixture = include_str!("../../tests/watch/inline_alerts.csl");
        let program = parse(fixture).unwrap();
        assert!(matches!(
            program.statements.as_slice(),
            [Stmt::Var { .. }, Stmt::Print(_)]
        ));
        let [rule] = program.watches.as_slice() else {
            panic!("expected one watch");
        };
        assert_eq!(rule.name.lexeme(), "failedSignIns");
        assert_eq!(rule.name.line(), 2);
        assert_eq!(rule.source, "signins");
        assert_eq!(rule.kind, "failed_signin");
        assert_eq!(rule.group_field, "account");
        assert_eq!(rule.window, 60);
        assert_eq!(rule.threshold, 3);
        let WatchSequence::Inline(alerts) = &rule.sequence else {
            panic!("expected inline sequence");
        };
        assert_eq!(
            alerts,
            &["Review repeated sign-in failures", "Check the account"]
        );
    }

    #[test]
    fn rejects_duplicate_abilities_and_invalid_grant_names() {
        let fixture = include_str!("../../examples/review-signins.csl");
        let declaration = fixture.split_once("ability reviewSignIns").unwrap().1;
        let duplicate = format!("{fixture}\nability reviewSignIns{declaration}");
        assert!(
            parse(&duplicate)
                .unwrap_err()
                .contains("Duplicate ability name.")
        );
        assert!(
            parse(&format!("{{ ability reviewSignIns{declaration} }}"))
                .unwrap_err()
                .contains("'ability' declarations must be at the top level.")
        );

        for (original, invalid, label) in [
            ("target \"local-report\";", "target \"  \";", "target name"),
            (
                "capability \"report:review\";",
                "capability \"--grant\";",
                "capability name",
            ),
        ] {
            let program = fixture.replace(original, invalid);
            assert!(
                parse(&program)
                    .unwrap_err()
                    .contains(&format!("Invalid {label}."))
            );
        }
    }

    #[test]
    fn reports_each_invalid_watch_at_the_offending_token() {
        let fixture = include_str!("../../tests/watch/invalid_clauses.csl");
        assert_eq!(
            parse(fixture).unwrap_err(),
            concat!(
                "[line 4] Error at '0': The window in seconds must be a positive 32-bit unsigned integer.\n",
                "[line 9] Error at 'source': Expect 'count' after the source clause."
            )
        );
    }

    #[test]
    fn rejects_invalid_numbers_without_rounding() {
        for number in ["0", "-1", "1.5", "1.0000000000000000001", "4294967296"] {
            let window = parse(&watch("rule", number, "1")).unwrap_err();
            assert!(window.contains("The window in seconds must be a positive"));
            let threshold = parse(&watch("rule", "1", number)).unwrap_err();
            assert!(threshold.contains("The count threshold must be a positive"));
        }
        let program = parse(&watch("rule", "4294967295", "1.0")).unwrap();
        assert_eq!(program.watches[0].window, u32::MAX);
        assert_eq!(program.watches[0].threshold, 1);
    }

    #[test]
    fn requires_complete_top_level_watches_with_unique_names() {
        let valid = watch("rule", "1", "2");
        assert_eq!(
            parse(&format!("{valid}\n{valid}")).unwrap_err(),
            "[line 2] Error at 'rule': Duplicate watch name."
        );
        assert_eq!(
            parse(&format!("{valid}\n{}", watch("other", "1", "2")))
                .unwrap()
                .watches
                .len(),
            2
        );
        for (fragment, diagnostic) in [
            (r#"source "events";"#, "Expect 'source'"),
            (
                r#"count "kind" by "group" within 1 >= 2;"#,
                "Expect 'count'",
            ),
            (r#"sequence { alert "hit"; }"#, "Expect 'sequence'"),
            (r#"alert "hit";"#, "Expect 'alert'"),
        ] {
            let error = parse(&valid.replace(fragment, "")).unwrap_err();
            assert!(error.contains(diagnostic), "{error}");
        }
        let nested = parse(&format!("{{ {valid} }}")).unwrap_err();
        assert!(nested.contains("'watch' declarations must be at the top level."));
    }
}
