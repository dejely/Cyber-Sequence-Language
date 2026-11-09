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
    let path =
        std::env::temp_dir().join(format!("cmsc124-if-recovery-{}.csl", std::process::id()));
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
