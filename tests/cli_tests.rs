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
