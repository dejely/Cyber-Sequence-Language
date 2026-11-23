use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cmsc124-interpreter"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn explicit_parse_command_prints_only_the_ast() {
    let output = run(&["--parse", "tests/lab2/precedence.csl"], "");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"1.0 2.0 3.0 * + 4.0 -\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn file_modes_distinguish_execution_evaluation_and_ast_printing() {
    // Even .src files now execute their contents instead of a hardcoded greeting.
    for extension in ["csl", "src"] {
        let path = std::env::temp_dir().join(format!(
            "cmsc124-script-modes-{}.{extension}",
            std::process::id()
        ));
        std::fs::write(&path, "2 + 3;").unwrap();
        let file = path.to_str().unwrap();
        for (args, expected) in [
            (vec![file], ""),
            (vec!["--eval", file], "5\n"),
            (vec!["--evaluate", file], "5\n"),
            (vec!["--parse", file], "2.0 3.0 +\n"),
        ] {
            let output = run(&args, "");
            assert!(output.status.success());
            assert_eq!(output.stdout, expected.as_bytes());
            assert!(output.stderr.is_empty());
        }
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn repl_bindings_survive_scanning_parsing_and_runtime_errors() {
    for args in [vec![], vec!["--repl"]] {
        let output = run(
            &args,
            concat!(
                "var value = 10;\n",
                "value = value + 2\n",
                "#\n",
                "var value = ;\n",
                "value = 1 / 0\n",
                "var value = 1 / 0;\n",
                "value\n",
                "var first = missing;\n",
                "first\n",
                "missing = 3\n",
                "missing\n",
                "var value = value + 1;\n",
                "print value;\n",
            ),
        );
        assert!(output.status.success());
        assert_eq!(output.stdout, b"> > 12\n> > > > > 12\n> > > > > > 13\n> ");
        assert_eq!(
            output.stderr,
            concat!(
                "line 1: Unexpected character '#'.\n",
                "[line 1] Error at ';': Expect expression.\n",
                "[line 1] Runtime error at '/': Division by zero.\n",
                "[line 1] Runtime error at '/': Division by zero.\n",
                "[line 1] Runtime error at 'missing': Undefined variable 'missing'.\n",
                "[line 1] Runtime error at 'first': Undefined variable 'first'.\n",
                "[line 1] Runtime error at 'missing': Undefined variable 'missing'.\n",
                "[line 1] Runtime error at 'missing': Undefined variable 'missing'.\n",
            )
            .as_bytes()
        );
    }
}

#[test]
fn repl_keeps_completed_assignment_effects_when_a_later_operation_fails() {
    let output = run(
        &[],
        concat!(
            "var value = 1;\n",
            "missing = (value = 3)\n",
            "value\n",
            "var value = (value = 5) + (1 / 0);\n",
            "value\n",
            "missing\n",
        ),
    );
    assert!(output.status.success());
    assert_eq!(output.stdout, b"> > > 3\n> > 5\n> > ");
    assert_eq!(
        output.stderr,
        concat!(
            "[line 1] Runtime error at 'missing': Undefined variable 'missing'.\n",
            "[line 1] Runtime error at '/': Division by zero.\n",
            "[line 1] Runtime error at 'missing': Undefined variable 'missing'.\n",
        )
        .as_bytes()
    );
}

#[test]
fn rejected_repl_declarations_have_no_effect_and_sessions_are_separate() {
    let output = run(&[], "var value = 1; 2;\nvalue\nvar value = 7\nvalue\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"> > > > > ");
    assert_eq!(
        output.stderr,
        concat!(
            "[line 1] Error at '2': Expect end of statement.\n",
            "[line 1] Runtime error at 'value': Undefined variable 'value'.\n",
            "[line 1] Error at end: Expect ';' after variable declaration.\n",
            "[line 1] Runtime error at 'value': Undefined variable 'value'.\n",
        )
        .as_bytes()
    );
    let first = run(&[], "var value = 7;\nvalue\n");
    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, b"> > 7\n> ");
    let second = run(&[], "value\n");
    assert!(second.status.success());
    assert_eq!(second.stdout, b"> > ");
    assert_eq!(
        second.stderr,
        b"[line 1] Runtime error at 'value': Undefined variable 'value'.\n"
    );
}

#[test]
fn expression_modes_parse_assignment_and_report_undefined_names_at_runtime() {
    let parsed = run(&["--parse"], "a = b = 1 + 2\n");
    assert!(parsed.status.success());
    assert_eq!(parsed.stdout, b"> a b 1.0 2.0 + = =\n> ");
    assert!(parsed.stderr.is_empty());
    let evaluated = run(&["--eval"], "missing\n42\n");
    assert!(evaluated.status.success());
    assert_eq!(evaluated.stdout, b"> > 42\n> ");
    assert_eq!(
        evaluated.stderr,
        b"[line 1] Runtime error at 'missing': Undefined variable 'missing'.\n"
    );
}

#[test]
fn program_repl_prints_statements_and_expressions_and_recovers() {
    for args in [vec![], vec!["--repl"]] {
        let output = run(
            &args,
            "print \"banana\";\nprint 1 / 0;\nprint 2\nprint 2; 3;\n6 * 7\n",
        );
        assert!(output.status.success());
        assert_eq!(output.stdout, b"> banana\n> > > > 42\n> ");
        assert_eq!(
            output.stderr,
            concat!(
                "[line 1] Runtime error at '/': Division by zero.\n",
                "[line 1] Error at end: Expect ';' after value.\n",
                "[line 1] Error at '3': Expect end of statement.\n",
            )
            .as_bytes()
        );
    }
}

#[test]
fn failed_files_exit_65_with_exact_diagnostics_and_no_partial_stdout() {
    for (fixture, diagnostic) in [
        (
            "tests/lab2/recovery.csl",
            "[line 1] Error at ';': Expect expression.\n\
             [line 2] Error at ';': Expect expression.\n\
             [line 4] Error at ';': Expect ')' after expression.\n",
        ),
        (
            "tests/lab2/scanner_error.csl",
            "line 2: Unexpected character '#'.\nline 3: Unexpected character '?'.\n",
        ),
        (
            "tests/lab2/trailing_input.csl",
            "[line 1] Error at '2': Expect ';' between expressions.\n",
        ),
    ] {
        let output = run(&["--parse", fixture], "");
        assert_eq!(output.status.code(), Some(65), "{fixture}");
        assert!(output.stdout.is_empty(), "{fixture}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            diagnostic,
            "{fixture}"
        );
    }
}

#[test]
fn empty_file_is_a_static_error() {
    let output = run(&["--parse", "tests/lab1/empty.csl"], "");
    assert_eq!(output.status.code(), Some(65));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"[line 1] Error at end: Expect expression.\n"
    );
}

#[test]
fn parse_repl_continues_after_parser_and_scanner_errors() {
    let output = run(&["--parse"], "1 +\n#\n(2 + 3) * 4\nnil;\n");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"> > > 2.0 3.0 + 4.0 *\n> nil\n> ");
    assert_eq!(
        output.stderr,
        b"[line 1] Error at end: Expect expression.\nline 1: Unexpected character '#'.\n"
    );
}

#[test]
fn default_and_explicit_repl_print_values_and_recover() {
    for args in [vec![], vec!["--repl"], vec!["--eval"]] {
        let output = run(&args, "1 +\n#\n(2 + 3) * 4\nnil;\n");
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, b"> > > 20\n> nil\n> ");
        assert_eq!(
            output.stderr,
            b"[line 1] Error at end: Expect expression.\nline 1: Unexpected character '#'.\n"
        );
    }
}

#[test]
fn repl_rejects_trailing_input_and_recovers_on_the_next_line() {
    let output = run(&["--parse"], "1 2\n42\n");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"> > 42.0\n> ");
    assert_eq!(
        output.stderr,
        b"[line 1] Error at '2': Expect end of expression.\n"
    );
}

#[test]
fn tokenizer_and_lab_zero_commands_preserve_existing_output() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (args, expected) in [
        (
            vec!["--tokenize", "tests/lab1/coverage/categories.csl"],
            "tests/lab1/coverage/categories.expected",
        ),
        (vec!["tests/lab0/hello.src"], "tests/lab0/hello.expected"),
    ] {
        let output = run(&args, "");
        assert_eq!(output.status.code(), Some(0));
        let mut expected_stdout = std::fs::read(root.join(expected)).unwrap();
        if !expected_stdout.ends_with(b"\n") {
            expected_stdout.push(b'\n');
        }
        assert_eq!(output.stdout, expected_stdout);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn invalid_arguments_and_missing_files_return_operational_errors() {
    for args in [
        vec!["--tokenize"],
        vec!["--unknown"],
        vec!["--parse", "tests/lab2/precedence.csl", "extra"],
        vec!["--parse", "tests/lab2/does-not-exist.csl"],
    ] {
        let output = run(&args, "");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn evaluation_repl_handles_literals_precedence_and_associativity() {
    let output = run(
        &["--eval"],
        "42\n3.5\n\"banana\"\ntrue\nfalse\nnil\n1 + 2 * 3\n(1 + 2) * 3\n10 - 3 - 2\n-8 / 2\n7 / 2\n\"ba\" + \"nana\"\n!nil\n2 < 3\n2 == 2\n2 != \"2\"\n",
    );
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, b"> 42\n> 3.5\n> banana\n> true\n> false\n> nil\n> 7\n> 9\n> 5\n> -4\n> 3.5\n> banana\n> true\n> true\n> true\n> true\n> ");
}

#[test]
fn evaluation_repl_recovers_after_runtime_errors() {
    let output = run(&["--eval"], "1 + true\n1 / 0\n8 / -0\n6 * 7\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"> > > > 42\n> ");
    let errors = String::from_utf8(output.stderr).unwrap();
    assert!(errors.contains("Operands must be two numbers or two strings."));
    assert_eq!(errors.matches("Division by zero.").count(), 2);
}

#[test]
fn evaluation_files_print_values_and_use_exit_70_for_runtime_errors() {
    let path = std::env::temp_dir().join(format!("cmsc124-evaluation-{}.csl", std::process::id()));
    std::fs::write(&path, "1 + 2 * 3; (1 + 2) * 3; 7 / 2;").unwrap();
    let output = run(&["--eval", path.to_str().unwrap()], "");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"7\n9\n3.5\n");
    assert!(output.stderr.is_empty());
    std::fs::write(&path, "1 / 0").unwrap();
    let output = run(&["--eval", path.to_str().unwrap()], "");
    std::fs::remove_file(path).unwrap();
    assert_eq!(output.status.code(), Some(70));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"[line 1] Runtime error at '/': Division by zero.\n"
    );
}

#[test]
fn repl_restores_all_scopes_after_success_and_nested_runtime_errors() {
    for args in [vec![], vec!["--repl"]] {
        let output = run(
            &args,
            concat!(
                "var value = 10;\n",
                "{ var value = 20; { value = 21; print value; } print value; }\n",
                "value\n",
                "{ value = 11; var value = 99; { var local = 1; 1 / 0; } value = 100; }\n",
                "value\n",
                "local\n",
                "{ var value = value + 1; print value; }\n",
                "value\n",
                "var after = value + 1;\n",
                "after\n",
            ),
        );
        assert!(output.status.success());
        assert_eq!(
            output.stdout,
            b"> > 21\n21\n> 10\n> > 11\n> > 12\n> 11\n> > 12\n> "
        );
        assert_eq!(
            output.stderr,
            concat!(
                "[line 1] Runtime error at '/': Division by zero.\n",
                "[line 1] Runtime error at 'local': Undefined variable 'local'.\n",
            )
            .as_bytes()
        );
    }
}

#[test]
fn malformed_repl_blocks_have_no_effect_and_recover_on_the_next_input() {
    let output = run(
        &[],
        concat!(
            "var value = 1;\n",
            "{ value = 9; { var local = ; } print value; }\n",
            "value\n",
            "{ var local = 3;\n",
            "local\n",
            "{}\n",
            "{ print value; } 2;\n",
            "value\n",
        ),
    );
    assert!(output.status.success());
    assert_eq!(output.stdout, b"> > > 1\n> > > > > 1\n> ");
    assert_eq!(
        output.stderr,
        concat!(
            "[line 1] Error at ';': Expect expression.\n",
            "[line 1] Error at end: Expect '}' after block.\n",
            "[line 1] Runtime error at 'local': Undefined variable 'local'.\n",
            "[line 1] Error at '2': Expect end of statement.\n",
        )
        .as_bytes()
    );
}

#[test]
fn program_recovery_preserves_block_boundaries_and_statement_starters() {
    let path =
        std::env::temp_dir().join(format!("cmsc124-block-recovery-{}.csl", std::process::id()));
    std::fs::write(
        &path,
        concat!(
            "print \"must not run\";\n",
            "{ var a = ; { print ; } var b = 2 }\n",
            "var = 3;\n",
            "}\n",
            "print ;\n",
            "var a = 1 print 2;\n",
            "{ var b = 1\n",
        ),
    )
    .unwrap();
    let output = run(&[path.to_str().unwrap()], "");
    std::fs::remove_file(path).unwrap();
    assert_eq!(output.status.code(), Some(65));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        concat!(
            "[line 2] Error at ';': Expect expression.\n",
            "[line 2] Error at ';': Expect expression.\n",
            "[line 2] Error at '}': Expect ';' after variable declaration.\n",
            "[line 3] Error at '=': Expect variable name.\n",
            "[line 4] Error at '}': Expect expression.\n",
            "[line 5] Error at ';': Expect expression.\n",
            "[line 6] Error at 'print': Expect ';' after variable declaration.\n",
            "[line 8] Error at end: Expect ';' after variable declaration.\n",
            "[line 8] Error at end: Expect '}' after block.\n",
        )
        .as_bytes()
    );
}

#[test]
fn repl_executes_conditionals_as_statements_without_echoing() {
    let output = run(
        &[],
        concat!(
            "var x = 1;\n",
            "if (x > 0) print \"positive\"; else print \"other\";\n",
            "if (false) print \"hidden\";\n",
            "if (x) { x = x + 1; }\n",
            "x\n",
        ),
    );
    assert!(output.status.success());
    assert_eq!(output.stdout, b"> > positive\n> > > 2\n> ");
    assert!(output.stderr.is_empty());
}

#[test]
fn program_recovery_restarts_at_if_statements() {
    let path = std::env::temp_dir().join(format!("cmsc124-if-recovery-{}.csl", std::process::id()));
    std::fs::write(&path, "var a = 1\nif (a) print ;\nif a print 1;\n").unwrap();
    let output = run(&[path.to_str().unwrap()], "");
    std::fs::remove_file(path).unwrap();
    assert_eq!(output.status.code(), Some(65));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        concat!(
            "[line 2] Error at 'if': Expect ';' after variable declaration.\n",
            "[line 2] Error at ';': Expect expression.\n",
            "[line 3] Error at 'a': Expect '(' after 'if'.\n",
        )
        .as_bytes()
    );
}

#[test]
fn repl_restores_scope_after_a_runtime_error_inside_a_loop_body() {
    let output = run(
        &[],
        concat!(
            "var i = 0;\n",
            "while (i < 3) { var local = i; i = i + 1; if (i == 2) 1 / 0; }\n",
            "i\n",
            "local\n",
            "while (i < 5) i = i + 1;\n",
            "i\n",
        ),
    );
    assert!(output.status.success());
    assert_eq!(output.stdout, b"> > > 2\n> > > 5\n> ");
    assert_eq!(
        output.stderr,
        concat!(
            "[line 1] Runtime error at '/': Division by zero.\n",
            "[line 1] Runtime error at 'local': Undefined variable 'local'.\n",
        )
        .as_bytes()
    );
}

#[test]
fn annotated_programs_preserve_execution_and_runtime_errors() {
    let output = run(&["tests/annotations/valid.csl"], "");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"banana\n5\n2\n5\nnil\n3\n3\n");
    assert!(output.stderr.is_empty());

    let output = run(&["tests/annotations/runtime_error.csl"], "");
    assert_eq!(output.status.code(), Some(70));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"[line 1] Runtime error at 'missing': Undefined variable 'missing'.\n"
    );
}

#[test]
fn repl_declares_functions_silently_and_echoes_call_results() {
    let output = run(
        &[],
        concat!(
            "fun greet(name) { print \"hi \" + name; }\n",
            "greet(\"ana\")\n",
            "greet\n",
        ),
    );
    assert!(output.status.success());
    assert_eq!(output.stdout, b"> > hi ana\nnil\n> <fn greet>\n> ");
    assert!(output.stderr.is_empty());
}

#[test]
fn repl_restores_the_caller_scope_after_a_runtime_error_inside_a_call() {
    let output = run(
        &[],
        concat!(
            "var x = \"global\";\n",
            "fun fail(x) { var local = 1; print x / 0; }\n",
            "fail(4)\n",
            "x\n",
            "local\n",
            "fail(1, 2)\n",
            "x\n",
        ),
    );
    assert!(output.status.success());
    assert_eq!(output.stdout, b"> > > > global\n> > > global\n> ");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        concat!(
            "[line 1] Runtime error at '/': Division by zero.\n",
            "[line 1] Runtime error at 'local': Undefined variable 'local'.\n",
            "[line 1] Runtime error at ')': Expected 1 arguments but got 2.\n",
        ),
    );
}

#[test]
fn repl_recovers_from_call_depth_and_preserves_scope_after_early_return() {
    let output = run(
        &[],
        concat!(
            "var label = \"global\";\n",
            "fun find() { var label = \"local\"; while (true) { return label; } }\n",
            "fun recurse() { return recurse(); }\n",
            "find()\n",
            "label\n",
            "recurse()\n",
            "label\n",
            "len(42)\n",
            "len(\"ok\")\n",
        ),
    );

    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        b"> > > > local\n> global\n> > global\n> > 2\n> "
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        concat!(
            "[line 1] Runtime error at ')': Maximum call depth (128) exceeded.\n",
            "[line 1] Runtime error at ')': len expects a string.\n",
        ),
    );
}

#[test]
fn annotation_errors_reject_the_entire_file_before_execution() {
    let path = std::env::temp_dir().join(format!("cmsc124-annotations-{}.csl", std::process::id()));
    std::fs::write(&path, "print \"must not run\";\n@entry var main;").unwrap();
    let output = run(&[path.to_str().unwrap()], "");
    std::fs::remove_file(path).unwrap();
    assert_eq!(output.status.code(), Some(65));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"[line 2] Error at 'entry': Unknown annotation '@entry'.\n"
    );
}

#[test]
fn annotation_repl_keeps_metadata_silent_and_recovers_after_errors() {
    for args in [vec![], vec!["--repl"]] {
        let output = run(
            &args,
            concat!(
                "@deprecated @doc(\"Old fruit\") var fruit = \"banana\";\n",
                "fruit\n",
                "@doc(fruit = \"mango\") var invalid;\n",
                "fruit\n",
                "@deprecated\n",
                "@doc(\"Stock\") var stock = 2;\n",
                "stock\n",
                "{ @deprecated var fruit = \"mango\"; 1 / 0; }\n",
                "fruit\n",
            ),
        );
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(
            output.stdout,
            b"> > banana\n> > banana\n> > > 2\n> > banana\n> "
        );
        assert_eq!(
            output.stderr,
            concat!(
                "[line 1] Error at 'fruit': Expect string literal annotation argument.\n",
                "[line 1] Error at end: Annotations must precede a 'var' declaration.\n",
                "[line 1] Runtime error at '/': Division by zero.\n",
            )
            .as_bytes()
        );
    }
}

#[test]
fn annotations_remain_declarations_in_expression_only_modes() {
    for mode in ["--parse", "--eval", "--evaluate"] {
        let output = run(&[mode, "examples/annotations.csl"], "");
        assert_eq!(output.status.code(), Some(65));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("Error at '@': Expect expression.")
        );
    }
}

#[test]
fn parse_mode_prints_calls_and_logical_operators_in_postfix() {
    let output = run(&["--parse"], "f(1)(2, a or b and c)\nf()\n");
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        b"> f 1.0 call/1 2.0 a b c and or call/2\n> f call/0\n> "
    );
}

#[test]
fn script_files_validate_watches_before_execution() {
    let mixed = run(&["tests/watch/inline_alerts.csl"], "");
    assert_eq!(mixed.status.code(), Some(65));
    assert!(mixed.stdout.is_empty());
    assert_eq!(
        mixed.stderr,
        b"A watch file accepts only watch and ability declarations.\n"
    );

    let invalid = run(&["tests/watch/invalid_clauses.csl"], "");
    assert_eq!(invalid.status.code(), Some(65));
    assert!(invalid.stdout.is_empty());
    assert_eq!(
        invalid.stderr,
        concat!(
            "[line 4] Error at '0': The window in seconds must be a positive 32-bit unsigned integer.\n",
            "[line 9] Error at 'source': Expect 'count' after the source clause.\n"
        )
        .as_bytes()
    );
}

#[test]
fn watch_replay_tracks_rule_and_group_windows_and_emits_ordered_alerts() {
    use serde_json::json;

    let output = run(
        &["tests/watch/replay.csl"],
        include_str!("watch/replay.jsonl"),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let actual: Vec<serde_json::Value> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let mut expected = vec![json!({
        "rule": "otherSource", "source": "backup", "kind": "failed_signin",
        "group_key": "ana", "timestamp": -10, "count": 1, "step": "alert",
        "message": "Backup source"
    })];
    for (group, timestamp) in [("ana", 61), ("bob", 71), ("ana", 123)] {
        for (rule, message) in [
            ("failedSignIns", "Review sign-ins"),
            ("failedSignIns", "Check the account"),
            ("corroborate", "Record crossing"),
        ] {
            expected.push(json!({
                "rule": rule, "source": "signins", "kind": "failed_signin",
                "group_key": group, "timestamp": timestamp, "count": 3,
                "step": "alert", "message": message
            }));
        }
    }
    assert_eq!(actual, expected);
}

#[test]
fn watch_replay_infers_the_only_source_and_keeps_alerts_before_an_input_error() {
    let valid_input = include_str!("../examples/signins.jsonl");
    let args = ["examples/failed-signins.csl"];
    let valid = run(&args, valid_input);
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    let alert: serde_json::Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(alert["timestamp"], 40);
    assert_eq!(alert["source"], "signins");

    let invalid = run(&args, &format!("{valid_input}\n{{invalid json}}\n"));
    assert_eq!(invalid.status.code(), Some(70));
    assert_eq!(invalid.stdout, valid.stdout);
    assert!(
        String::from_utf8(invalid.stderr)
            .unwrap()
            .starts_with("Event input line 5:")
    );
}

#[test]
fn watch_replay_rejects_invalid_events_with_their_input_line() {
    let args = ["tests/watch/replay.csl"];
    for (event, message) in [
        (
            r#"{"kind":"failed_signin","timestamp":2}"#,
            "must name its source",
        ),
        (
            r#"{"source":"unknown","kind":"failed_signin","timestamp":2}"#,
            "unknown event source",
        ),
        (
            r#"{"source":"signins","kind":"","timestamp":2}"#,
            "event kind must be nonempty",
        ),
        (
            r#"{"source":"signins","kind":"failed_signin","timestamp":1}"#,
            "received timestamp 1 after 2",
        ),
        (
            r#"{"source":"signins","kind":"failed_signin","timestamp":2.5}"#,
            "expected i64",
        ),
        (
            r#"{"source":"signins","kind":"failed_signin","timestamp":2,"fields":{"account":8}}"#,
            "expected a string",
        ),
        (
            r#"["signins","failed_signin",2,{"account":"ana"}]"#,
            "expected a JSON event object",
        ),
    ] {
        let input = format!(
            "{{\"source\":\"signins\",\"kind\":\"failed_signin\",\"timestamp\":2}}\n\n{event}\n"
        );
        let output = run(&args, &input);
        assert_eq!(output.status.code(), Some(70), "{event}");
        assert!(output.stdout.is_empty(), "{event}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.starts_with("Event input line 3:"), "{stderr}");
        assert!(stderr.contains(message), "{stderr}");
    }
}

#[test]
fn ability_replay_resolves_forward_references_and_preserves_step_order() {
    let output = run(
        &[
            "examples/review-signins.csl",
            "--grant",
            "another-target",
            "report:review",
            "--grant",
            "local-report",
            "report:review",
        ],
        include_str!("../examples/signins.jsonl"),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let records: Vec<serde_json::Value> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["message"], "Review repeated sign-in failures");
    assert_eq!(records[1]["message"], "Check the account");
    for record in records {
        assert_eq!(record["rule"], "failedSignIns");
        assert_eq!(record["timestamp"], 40);
    }
}

#[test]
fn ability_replay_denies_missing_or_mismatched_grants_before_reading_events() {
    let file = "examples/review-signins.csl";
    for args in [
        vec![file],
        vec![
            file,
            "--grant",
            "local-report",
            "report:other",
            "--grant",
            "another-target",
            "report:review",
        ],
    ] {
        let output = run(&args, "not an event\n");
        assert_eq!(output.status.code(), Some(70));
        assert!(output.stdout.is_empty());
        let diagnostic = String::from_utf8(output.stderr).unwrap();
        assert!(diagnostic.contains(
            "Ability 'reviewSignIns' requires grant \"report:review\" for target \"local-report\"."
        ));
        assert!(!diagnostic.contains("Event input line"));
    }

    for args in [
        vec![file, "--grant", "local-report"],
        vec![file, "--grant", "local-report", ""],
        vec![file, "--grant", "   ", "report:review"],
        vec![file, "--grant", "local-report", "--grant"],
    ] {
        let output = run(&args, "not an event\n");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("Expected --grant")
        );
    }
}

#[test]
fn ability_replay_reports_unknown_references_before_emitting_inline_alerts() {
    let file = std::env::temp_dir().join(format!("csl-unknown-ability-{}.csl", std::process::id()));
    let inline = include_str!("../examples/failed-signins.csl");
    let named = include_str!("../examples/review-signins.csl")
        .replace("watch failedSignIns", "watch namedWatch")
        .replace("sequence reviewSignIns;", "sequence missing;");
    std::fs::write(&file, format!("{inline}\n{named}")).unwrap();
    let output = run(
        &[file.to_str().unwrap()],
        include_str!("../examples/signins.jsonl"),
    );
    std::fs::remove_file(file).unwrap();
    assert_eq!(output.status.code(), Some(70));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("Unknown ability 'missing'.")
    );
}

#[test]
fn ranked_abilities_return_trigger_data_and_keep_simulations_and_incidents_target_scoped() {
    use serde_json::json;

    let output = run(
        &[
            "tests/watch/ranked.csl",
            "--grant",
            "workstation",
            "report:review",
            "--rank",
            "1",
            "--grant",
            "other",
            "report:review",
            "--policy",
            "tests/watch/ranked_policy.json",
        ],
        include_str!("watch/ranked_events.jsonl"),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let records: Vec<serde_json::Value> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records[0]["step"], "inspection");
    assert_eq!(records[0]["target"], "workstation");
    assert_eq!(records[0]["rule"], "accountReview");
    assert_eq!(records[0]["group_key"], "ana");
    assert_eq!(records[0]["timestamp"], 61);
    assert_eq!(records[0]["count"], 3);
    let report = &records[1];
    assert_eq!(report["step"], "report");
    let events = report["events"].as_array().unwrap();
    assert_eq!(events.len(), 3);
    for (event, timestamp) in events.iter().zip([30, 60, 61]) {
        assert_eq!(event["timestamp"], timestamp);
        assert_eq!(event["fields"]["account"], "ana");
    }
    assert_eq!(records[2]["step"], "simulated_isolate");
    assert_eq!(records[2]["id"], "containment-1");
    assert_eq!(records[2]["reason"], "Account review");
    assert_eq!(records[3]["reason"], "Manual review");
    assert_eq!(records[4]["status"], "canceled");
    assert_eq!(records[5]["step"], "simulated_execute");
    assert_eq!(records[5]["id"], "response-2");
    assert_eq!(
        records[5]["response"]["steps"],
        json!(["Record for review"])
    );
    assert_eq!(records[6]["status"], "canceled");

    let changes: Vec<_> = records
        .iter()
        .filter(|record| record["id"] == "incident-3")
        .collect();
    assert_eq!(changes.first().unwrap()["status"], "open");
    let final_states: Vec<_> = changes[changes.len() - 3..]
        .iter()
        .map(|record| record["status"].as_str().unwrap())
        .collect();
    assert_eq!(final_states, ["in_review", "escalated", "closed"]);
    let evidence = changes.last().unwrap()["evidence"].as_array().unwrap();
    assert_eq!(
        evidence,
        &[
            report.clone(),
            json!(3.0),
            json!(true),
            json!({"label":"Workstation"}),
            json!({"network":"simulated"}),
            json!(true),
            json!(["review"]),
            json!({"target":"workstation", "name":"review","steps":["Record for review"]}),
            records[5].clone(),
            json!(true),
            {
                let mut result = records[5].clone();
                result["status"] = json!("canceled");
                result
            }
        ]
    );
    for (id, status) in [
        ("containment-4", "pending"),
        ("response-5", "pending"),
        ("incident-6", "open"),
    ] {
        let matching: Vec<_> = records.iter().filter(|record| record["id"] == id).collect();
        assert_eq!(matching.len(), 1, "{id}");
        assert_eq!(matching[0]["status"], status);
    }
    let other_target = records.last().unwrap();
    assert_eq!(other_target["id"], "incident-7");
    assert_eq!(other_target["target"], "other");
    assert_eq!(
        other_target["evidence"],
        json!([null, false, false, false, false, false, false])
    );
}

#[test]
fn authorized_keyword_steps_emit_in_order_and_enforce_rank() {
    let file = "tests/watch/simulation_steps.csl";
    let events = include_str!("../examples/signins.jsonl");
    let grants = ["--grant", "workstation", "report:review"];
    let output = run(&[&[file][..], &grants, &["--rank", "2"]].concat(), events);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let records: Vec<serde_json::Value> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let steps: Vec<_> = records
        .iter()
        .map(|record| record["step"].as_str().unwrap())
        .collect();
    assert_eq!(
        steps,
        [
            "inspection",
            "alert",
            "simulated_isolate",
            "simulated_execute"
        ]
    );
    for record in [&records[0], &records[2], &records[3]] {
        assert_eq!(record["target"], "workstation");
        assert_eq!(record["timestamp"], 40);
    }

    let denied = run(&[file, "--rank", "2"], "not an event\n");
    assert_eq!(denied.status.code(), Some(70));
    assert!(denied.stdout.is_empty());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("requires grant"));

    let ranks = [
        ("7", "inspect", 6),
        ("6", "isolate", 3),
        ("3", "execute", 2),
    ];
    for (rank, operation, needed) in ranks {
        let denied = run(&[&[file][..], &grants, &["--rank", rank]].concat(), events);
        assert_eq!(denied.status.code(), Some(70));
        assert!(
            String::from_utf8_lossy(&denied.stderr)
                .contains(&format!("{operation} requires Sequence {needed}"))
        );
    }
}

#[test]
fn response_calls_require_a_configured_default_and_sequence_one_gates_incidents() {
    let no_policy = run(
        &[
            "tests/watch/response_default.csl",
            "--grant",
            "workstation",
            "report:review",
            "--rank",
            "2",
        ],
        include_str!("../examples/signins.jsonl"),
    );
    assert_eq!(no_policy.status.code(), Some(70));
    assert!(no_policy.stdout.is_empty());
    assert!(String::from_utf8_lossy(&no_policy.stderr).contains("no approved default response"));

    let file = "tests/watch/ranked.csl";
    let events = include_str!("watch/ranked_events.jsonl");
    let args = [
        file,
        "--grant",
        "workstation",
        "report:review",
        "--grant",
        "other",
        "report:review",
        "--rank",
        "2",
        "--policy",
        "tests/watch/ranked_policy.json",
    ];
    let denied = run(&args, events);
    assert_eq!(denied.status.code(), Some(70));
    assert!(String::from_utf8_lossy(&denied.stderr).contains("openIncident requires Sequence 1"));
}
