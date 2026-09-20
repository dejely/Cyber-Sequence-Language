# CMSC 124 Progress Tracker

**Last repository verification:** 2026-09-20  
**Current milestone:** Lab 1 — scanner and tokenizer  
**Course reference:** [Lab 1 implementation notes](https://renscourses.netlify.app/articles/cmsc-124-lab1#implementation-notes)

This tracker records what the repository implements today. A checked item has
been verified against the source and the listed test command. An unchecked item
is planned, incomplete, or needs to be reconciled with the laboratory material.
When a future laboratory README or PDF is added, link it in the Future
Laboratories section first, then turn its requirements into checklist items
before marking implementation work complete.

## Lab 1 — Scanner and Tokenizer

### Build and command interface

- [x] Build the release executable with `./build.sh`.
- [x] Preserve the Lab 0 greeting when no tokenizer command is supplied.
- [x] Tokenize a source file with `./run --tokenize <file>`.
- [x] Print tokens in the fixture format: type, lexeme, literal, and line.
- [x] End a successful scan with exactly one `EOF` token.
- [x] Return exit code `65` for static scanning errors.
- [x] Provide `./run --repl` for tokenizing one input line at a time.

### Lexical categories

- [x] Recognize parentheses, braces, brackets, comma, dot, arithmetic
  operators, semicolon, and `@`.
- [x] Recognize `!`, `!=`, `=`, `==`, `>`, `>=`, `<`, and `<=`.
- [x] Recognize identifiers beginning with an ASCII letter or underscore.
- [x] Recognize integer and decimal number literals.
- [x] Recognize double-quoted strings, including strings that span lines.
- [x] Skip `//` line comments, including a comment that ends at EOF.
- [x] Track one-based source line numbers.
- [x] Recognize the core reserved words: `and`, `class`, `else`, `false`,
  `for`, `fun`, `if`, `nil`, `or`, `print`, `return`, `super`, `this`, `true`,
  `var`, and `while`.

### Error handling

- [x] Reject unexpected characters with their source line.
- [x] Report multiple unexpected characters in source order.
- [x] Reject unterminated strings with the line where the string began.
- [x] Keep standard output empty when scanning fails so fixture output remains
  deterministic.

### CSL extensions

- [x] Recognize `@` resource references at the token level.
- [x] Recognize the CSL keywords `watch`, `ability`, `target`, `source`,
  `sequence`, `require`, `capability`, `count`, `by`, `within`, `alert`,
  `inspect`, `isolate`, and `execute`.
- [ ] Define CSL statement grammar and an abstract syntax tree.
- [ ] Parse CSL sources into validated rules.
- [ ] Evaluate CSL event windows, counts, and grouping.
- [ ] Implement alert and response behavior for approved targets.
- [ ] Add an RFID credential access device (CAD) adapter that supplies an
  authorization result for enrolled credentials. Raw RFID access must remain
  outside the language runtime.
- [ ] Add tests for accepted and rejected RFID/CAD authorization results.

### Deferred interpreter work

- [ ] Parse expressions, declarations, blocks, and statements.
- [ ] Build an abstract syntax tree.
- [ ] Resolve names and scopes.
- [ ] Evaluate expressions and execute statements.
- [ ] Define runtime errors and their exit behavior.

### Verification recorded on 2026-09-20

- [x] `./build.sh`
- [x] `python3 run_tests.py tests/lab0` — 1/1 passed.
- [x] `python3 run_tests.py tests/lab1` — 8/8 passed.
- [x] `cargo test` — 1/1 passed.

The Rust build reports existing dead-code warnings for `Literal::Boolean` and
`Literal::Null`. They do not fail the Lab 1 fixtures, but they should be
reviewed when later parser or evaluator work needs those literal values.

## Future Laboratories

Add each laboratory below as soon as its README or PDF is available. Keep the
course material in the repository, preferably under `docs/labs/`, and use a
relative link so the tracker remains useful after cloning the project.

### Lab 2 and later

- [ ] Add the laboratory README or PDF under `docs/labs/`.
- [ ] Link the exact brief here and record its submission requirements.
- [ ] Translate each requirement into implementation and test checklist items.
- [ ] Record the build and test commands used to verify the laboratory.
- [ ] Update this tracker only after the source and tests support the status.

## Update procedure

1. Add the new lab material under `docs/labs/` and link it above.
2. Compare the brief with the current code before changing any checkbox.
3. Implement the missing requirement and add a focused test or fixture.
4. Run the lab's documented validation command, then record the result and
   verification date here.
