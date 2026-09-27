use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run_with_input(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cmsc124-interpreter"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start interpreter");
    let mut stdin = child.stdin.take().expect("open stdin");
    stdin.write_all(input.as_bytes()).expect("write input");
    drop(stdin);
    child
        .wait_with_output()
        .expect("collect interpreter output")
}

#[test]
fn default_repl_recovers_after_an_invalid_line() {
    let result = run_with_input(&[], "var first;\n#\nprint first;\n");
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        concat!(
            "> Token(type=VAR, lexeme=var, literal=null, line=1)\n",
            "Token(type=IDENTIFIER, lexeme=first, literal=null, line=1)\n",
            "Token(type=SEMICOLON, lexeme=;, literal=null, line=1)\n",
            "Token(type=EOF, lexeme=, literal=null, line=2)\n",
            "> > Token(type=PRINT, lexeme=print, literal=null, line=1)\n",
            "Token(type=IDENTIFIER, lexeme=first, literal=null, line=1)\n",
            "Token(type=SEMICOLON, lexeme=;, literal=null, line=1)\n",
            "Token(type=EOF, lexeme=, literal=null, line=2)\n",
            "> "
        )
    );
    assert_eq!(result.stderr, b"line 1: Unexpected character '#'.\n");
}

#[test]
fn default_repl_exits_cleanly_at_immediate_eof() {
    let result = run_with_input(&[], "");
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(result.stdout, b"> ");
    assert!(result.stderr.is_empty());
}

#[test]
fn explicit_repl_alias_matches_default_including_final_line_without_newline() {
    let default = run_with_input(&[], "#\n42");
    let alias = run_with_input(&["--repl"], "#\n42");
    assert_eq!(default.status.code(), Some(0));
    assert_eq!(alias.status.code(), Some(0));
    assert_eq!(default.stdout, alias.stdout);
    assert_eq!(default.stderr, alias.stderr);
    assert_eq!(
        default.stdout,
        concat!(
            "> > Token(type=NUMBER, lexeme=42, literal=42, line=1)\n",
            "Token(type=EOF, lexeme=, literal=null, line=1)\n",
            "> "
        )
        .as_bytes()
    );
    assert_eq!(default.stderr, b"line 1: Unexpected character '#'.\n");
}

#[test]
fn lab0_file_invocation_preserves_the_greeting() {
    let result = run_with_input(&["tests/lab0/hello.src"], "");
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(result.stdout, b"Hello, JM & Dejel!\n");
    assert!(result.stderr.is_empty());
}

#[test]
fn numeric_overflow_rejects_file_without_stdout() {
    let result = run_with_input(
        &[
            "--tokenize",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/lab1/numeric_overflow.csl"
            ),
        ],
        "",
    );
    assert_eq!(result.status.code(), Some(65));
    assert!(result.stdout.is_empty());
    assert_eq!(result.stderr, b"line 1: Numeric literal out of range.\n");
}

#[test]
fn numeric_overflow_reports_following_errors_in_order() {
    let result = run_with_input(
        &[
            "--tokenize",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/lab1/numeric_overflow_recovery.csl"
            ),
        ],
        "",
    );
    assert_eq!(result.status.code(), Some(65));
    assert!(result.stdout.is_empty());
    assert_eq!(
        result.stderr,
        concat!(
            "line 2: Numeric literal out of range.\n",
            "line 2: Unexpected character '#'.\n",
            "line 3: Unexpected character '?'.\n"
        )
        .as_bytes()
    );
}

fn tokenize_fixture(name: &str) -> Output {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/lab1")
        .join(name);
    run_with_input(&["--tokenize", path.to_str().expect("fixture path")], "")
}

#[test]
fn lexical_rejections_report_exact_lines_and_no_stdout() {
    for (fixture, diagnostic) in [
        (
            "invalid.csl",
            "line 2: Unexpected character '#'.\nline 3: Unexpected character '?'.\n",
        ),
        ("unterminated.csl", "line 1: Unterminated string.\n"),
        (
            "mixed_errors.csl",
            concat!(
                "line 2: Unexpected character '#'.\n",
                "line 3: Unexpected character '?'.\n",
                "line 4: Unterminated string.\n"
            ),
        ),
    ] {
        let result = tokenize_fixture(fixture);
        assert_eq!(result.status.code(), Some(65), "{fixture}");
        assert!(result.stdout.is_empty(), "{fixture}");
        assert_eq!(result.stderr, diagnostic.as_bytes(), "{fixture}");
    }
}

#[test]
fn every_scanner_fixture_has_repeatable_output_and_correct_streams() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lab1");
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .expect("read fixtures")
        .map(|entry| entry.expect("fixture entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "csl"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    for path in paths {
        let name = path.file_name().unwrap().to_str().unwrap();
        let first = tokenize_fixture(name);
        let second = tokenize_fixture(name);
        let exit_path = path.with_extension("exit");
        let expected_exit = if exit_path.exists() {
            std::fs::read_to_string(exit_path)
                .unwrap()
                .trim()
                .parse::<i32>()
                .unwrap()
        } else {
            0
        };
        assert_eq!(first.status.code(), Some(expected_exit), "{name}");
        assert_eq!(second.status.code(), Some(expected_exit), "{name}");
        assert_eq!(first.stdout, second.stdout, "stdout changed: {name}");
        assert_eq!(first.stderr, second.stderr, "stderr changed: {name}");
        if expected_exit == 0 {
            assert!(first.stderr.is_empty(), "unexpected diagnostic: {name}");
        } else {
            assert!(first.stdout.is_empty(), "tokens leaked: {name}");
            assert!(!first.stderr.is_empty(), "missing diagnostic: {name}");
        }
    }
}
