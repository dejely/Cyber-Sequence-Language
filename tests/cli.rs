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
fn explicit_and_positional_file_commands_print_only_the_ast() {
    for args in [
        vec!["--parse", "tests/lab2/precedence.csl"],
        vec!["tests/lab2/precedence.csl"],
    ] {
        let output = run(&args, "");
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, b"(- (+ 1.0 (* 2.0 3.0)) 4.0)\n");
        assert!(output.stderr.is_empty());
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
fn every_repl_entrypoint_continues_after_parser_and_scanner_errors() {
    for args in [vec![], vec!["--repl"], vec!["--parse"]] {
        let output = run(&args, "1 +\n#\n(2 + 3) * 4\nnil;\n");
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(
            output.stdout,
            b"> > > (* (group (+ 2.0 3.0)) 4.0)\n> nil\n> "
        );
        assert_eq!(
            output.stderr,
            b"[line 1] Error at end: Expect expression.\nline 1: Unexpected character '#'.\n"
        );
    }
}

#[test]
fn repl_rejects_trailing_input_and_recovers_on_the_next_line() {
    let output = run(&["--repl"], "1 2\n42\n");
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
            vec!["--tokenize", "tests/lab1/categories.csl"],
            "tests/lab1/categories.expected",
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
