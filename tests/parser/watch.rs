use crate::parser::{Parser, Program};
use crate::scanner::Scanner;
use crate::stmt::{Stmt, WatchSequence};

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
    let fixture = include_str!("../watch/inline_alerts.csl");
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
    let fixture = include_str!("../watch/invalid_clauses.csl");
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
