# Cyber Sequence Language (CSL)

## Creators

- Jemarco Briz ([Jmbriz123](https://github.com/Jmbriz123))
- Dejel Cyrus De Asis ([dejely](https://github.com/dejely))

## Overview

CSL is a proposed dynamically typed language for people describing authorized
monitoring and response sequences: watch an event source, group or count events
within a time window, require a capability, then describe an alert or inspection.
Its intended syntax combines explicit punctuation with domain words such as
`watch`, `source`, and `sequence`. This CMSC 124 prototype currently implements
lexical scanning only. Lab 1's deliverable is the scanner, diagnostics, tests,
and this specification; parsing and evaluation are later milestones. Event
adapters and optional RFID credential verification remain future extensions,
not dependencies of the three-month language prototype.

## Host language and build

- Host: Rust, edition 2024, with no runtime crate dependencies.
- Version metadata: [rust-toolchain.toml](rust-toolchain.toml) currently selects
  the moving `stable` channel. Rust 1.98.0 was used for local verification;
  an exact version pin is planned for Step 8 and is not implemented yet.
- Prerequisites: Rust/Cargo and Bash; Python 3 and curl for the shared test harness.
- From the repository root, build the release executable:

```bash
./build.sh
```

The executable is `target/release/cmsc124-interpreter`, invoked by `./run`.
Rebuild after source changes; `./run` does not build automatically. On Windows,
use Git Bash. Node/npm is optional: `npm run format` and `npm run format:check` are aliases
for `cargo fmt` and `cargo fmt --check`. VS Code uses the rust-analyzer extension
for Rust formatting on save; rustfmt is the shared formatter.

## Running it

| Command | Current behavior |
| --- | --- |
| `./run` | Starts the scanner REPL |
| `./run --repl` | Explicit alias for the scanner REPL |
| `./run --tokenize <file>` | Scans a UTF-8 source file and prints tokens |
| `./run tests/lab0/hello.src` | Prints `Hello, JM & Dejel!` for Lab 0 compatibility |
| `./run <file>` | Currently prints the legacy greeting; does not execute the file |
| `./run --parse <file>` | Placeholder: opens the scanner REPL and does not parse the file |
| `./run --eval <file>` | Not implemented; currently falls through to the legacy greeting |

Example file invocation:

```bash
./run --tokenize tests/lab1/coverage/categories.csl
```

The REPL prints `> `, scans one input line, and prints tokens or diagnostics.
Each line is independent, with line numbering starting at 1; `var x = 1;` does
not create a variable. A bad line such as `#` reports an error and returns to the
prompt. End input with EOF (Ctrl-D on Unix-like terminals). Multiline strings
must be scanned from files because the REPL does not accumulate input lines.

Clean file scans and normal REPL exit use code 0. Lexically rejected files use
65, with diagnostics on stderr and no stdout. Missing/unreadable input files or
missing `--tokenize` filenames currently use code 1 through Rust's error return.
Runtime exit 70 is reserved for later work and is not implemented.

## File extension

CSL source files use `.csl`, matching `tests/lab1/manifest.json`. The `.src`
extension in `tests/lab0/manifest.json` is a preserved legacy-test exception.

## Lexical structure

Scanning is handwritten, character by character, without regular expressions.
Patterns below describe the grammar; they are not regex-based implementation.
All token categories are defined in [src/token.rs](src/token.rs); scanning is in
[src/scanner.rs](src/scanner.rs).

### Keywords

The words below are recognized now. Their purposes describe intended semantics;
none of these keywords currently executes statements or CSL operations.

| Lexeme   | Token Type | Purpose                           |
| -------- | ---------- | --------------------------------- |
| `and`    | `AND`      | Logical conjunction               |
| `class`  | `CLASS`    | Declares a class                  |
| `else`   | `ELSE`     | Runs an alternate branch          |
| `false`  | `FALSE`    | Boolean false value               |
| `for`    | `FOR`      | Starts a `for` loop               |
| `fun`    | `FUN`      | Declares a function               |
| `if`     | `IF`       | Starts a conditional branch       |
| `nil`    | `NIL`      | Represents the absence of a value |
| `or`     | `OR`       | Logical disjunction               |
| `print`  | `PRINT`    | Prints a value                    |
| `return` | `RETURN`   | Returns from a function           |
| `super`  | `SUPER`    | Accesses a superclass method      |
| `this`   | `THIS`     | Refers to the current instance    |
| `true`   | `TRUE`     | Boolean true value                |
| `var`    | `VAR`      | Declares a variable               |
| `while`  | `WHILE`    | Starts a `while` loop             |
| `watch` | `WATCH` | declares an event-monitoring rule |
| `ability` | `ABILITY` | declares a named ability |
| `target` | `TARGET` | specifies the target of an ability |
| `source` | `SOURCE` | specifies an event or log source |
| `sequence` | `SEQUENCE` | groups operation steps |
| `require` | `REQUIRE` | declares a required capability |
| `capability` | `CAPABILITY` | introduces a capability reference |
| `count` | `COUNT` | counts matching events |
| `by` | `BY` | groups events by a field |
| `within` | `WITHIN` | sets a query time window |
| `alert` | `ALERT` | raises an alert |
| `inspect` | `INSPECT` | requests an inspection of an approved target |
| `isolate` | `ISOLATE` | requests isolation of an approved target |
| `execute` | `EXECUTE` | requests execution of declared, authorized operations |

### Operators

Arity describes the intended language operation; the scanner only emits tokens.
Precedence and associativity are deferred until the Lab 2 grammar.

| Operator | Token type | Category | Operands | Associativity | Precedence |
| --- | --- | --- | --- | --- | --- |
| `+` | `PLUS` | Arithmetic addition | Binary | Deferred | Deferred |
| `-` | `MINUS` | Subtraction / negation | Binary / unary | Deferred | Deferred |
| `*` | `STAR` | Arithmetic multiplication | Binary | Deferred | Deferred |
| `/` | `SLASH` | Arithmetic division | Binary | Deferred | Deferred |
| `=` | `EQUAL` | Assignment | Binary | Deferred | Deferred |
| `==` | `EQUAL_EQUAL` | Equality | Binary | Deferred | Deferred |
| `!=` | `BANG_EQUAL` | Inequality | Binary | Deferred | Deferred |
| `<` | `LESS` | Comparison | Binary | Deferred | Deferred |
| `<=` | `LESS_EQUAL` | Comparison | Binary | Deferred | Deferred |
| `>` | `GREATER` | Comparison | Binary | Deferred | Deferred |
| `>=` | `GREATER_EQUAL` | Comparison | Binary | Deferred | Deferred |
| `!` | `BANG` | Logical negation | Unary | Deferred | Deferred |
| `and` | `AND` | Logical conjunction | Binary | Deferred | Deferred |
| `or` | `OR` | Logical disjunction | Binary | Deferred | Deferred |

Longest valid matches win: `!=` emits one `BANG_EQUAL`, and `===` emits
`EQUAL_EQUAL` followed by `EQUAL`. A single `/` is an operator unless followed
by another slash beginning a comment.

Punctuation and single-character vocabulary (meanings beyond tokenization are
reserved for later stages):

| Lexeme | Token Type      | Purpose                                              |
| ------ | --------------- | ---------------------------------------------------- |
| `(`    | `LEFT_PAREN`    | Opens a grouped expression                           |
| `)`    | `RIGHT_PAREN`   | Closes a grouped expression                          |
| `{`    | `LEFT_BRACE`    | Starts a code block                                  |
| `}`    | `RIGHT_BRACE`   | Ends a code block                                    |
| `[`    | `LEFT_BRACKET`  | Opens an array or index expression                   |
| `]`    | `RIGHT_BRACKET` | Closes an array or index expression                  |
| `,`    | `COMMA`         | Separates values or arguments                        |
| `.`    | `DOT`           | Accesses a property or separates a fractional number |
| `+`    | `PLUS`          | Addition                                             |
| `-`    | `MINUS`         | Subtraction                                          |
| `*`    | `STAR`          | Multiplication                                       |
| `/`    | `SLASH`         | Division                                             |
| `;`    | `SEMICOLON`     | Terminates a statement                               |
| `@`    | `AT`            | Introduces a resource reference, such as `@host`     |

### Literals

| Kind | Syntax | Current token value / intended meaning |
| --- | --- | --- |
| Number | `42`, `3.14`; `[0-9]+(\.[0-9]+)?` | `NUMBER` with finite `f64` value |
| String | `"hello"`, `""`, or quoted multiline text | `STRING` with contents excluding quotes |
| Boolean | `true`, `false` | Keyword tokens without literal values; future Boolean values |
| Nil | `nil` | Keyword token without a literal value; future absence of a value |

Numbers use floating-point rounding; tiny values may round to zero. Conversion
failure or overflow to infinity is rejected. A number consisting of 400 `9`
digits is an example rejection. Exact decimal arithmetic, arbitrary precision,
and exponent notation are not supported as numeric literal forms.

The decimal point belongs to a number only if a digit follows it:

| Input | Tokens (excluding EOF) |
| --- | --- |
| `.5` | `DOT NUMBER(5)` |
| `3.` | `NUMBER(3) DOT` |
| `3.toString` | `NUMBER(3) DOT IDENTIFIER(toString)` |
| `-2` | `MINUS NUMBER(2)` |
| `123event` | `NUMBER(123) IDENTIFIER(event)`; no lexical error |

Strings allow actual newlines and Unicode contents. There is **no source escape
processing**: `"a\nb"` holds four characters (`a`, backslash, `n`, `b`), and a
backslash does not protect a following quote. A multiline string's token reports
its opening line; following tokens reflect newlines consumed within the string.

### Identifiers

The pattern is `[A-Za-z_][A-Za-z0-9_]*`: start with an ASCII letter or underscore,
then use ASCII letters, digits, or underscores. Names are case-sensitive.
`_host2`, `event123`, and `MyVariable` are identifiers. No explicit length limit
is imposed, beyond available resources.

The whole name is scanned before keyword lookup: `if` is reserved, while `iffy`
and `If` are identifiers. `user-name` is two identifiers separated by `MINUS`;
`123event` is the numeric/name sequence shown above. Neither is one identifier.

### Comments

`//` discards the remainder of a line, including a comment ending at EOF without
a newline. It may follow code: `x // note` retains only the identifier before
EOF. Inside strings, `"//"` and `"/* text */"` remain text. Block comments,
nested comments, and docstring comment syntax are not supported.

Lab 1 uses sidecars, so it does not need inline expectation comments. Its manifest
omits `comment_prefix`; if later labs use inline mode, their prefix should be `//`.

## Whitespace and termination

Spaces, tabs, CR, and LF outside strings emit no tokens. LF increments the source
line; CR is ignored, so CRLF counts once. Blank lines still advance positions.
Whitespace inside a string is retained. Empty input emits only `EOF` on line 1;
EOF has an empty lexeme and no literal.

The intended statement terminator is `;`, blocks use `{}`, grouping/calls use
`()`, and `[]` is reserved for arrays/indexing. For example, `{ print x; }` and
`items[0]` tokenize today but are not parsed. Newlines do not terminate statements
at the scanner level. `@auth_logs` emits `AT IDENTIFIER(auth_logs)` without doing
resource lookup.

## Token output format

The format is deterministic and follows source order:

```text
Token(type=STRING, lexeme="hello", literal=hello, line=1)
```

`type` is the uppercase token category; `lexeme` preserves original spelling;
`literal` holds a parsed number/string or prints `null` if absent; `line` is
one-based. Number values use Rust `f64` display, so `3.0` has lexeme `3.0` and
literal `3`. `true`, `false`, and `nil` currently print `literal=null`.

Backslash, LF, CR, and tab are displayed as `\\`, `\n`, `\r`, and `\t` in
lexemes and string literals. Other characters, including quotes, remain unchanged.
This is display escaping only; stored values and source syntax are unchanged.
An actual newline in `a` followed by `b` prints as:

```text
Token(type=STRING, lexeme="a\nb", literal=a\nb, line=1)
```

Literal backslash-plus-`n` instead prints as:

```text
Token(type=STRING, lexeme="a\\nb", literal=a\\nb, line=1)
```

Every successful scan ends with exactly one `EOF`. Files ending with LF place
EOF on the next line; a final newline is therefore significant to token positions.
This output contract is frozen for Lab 1; format changes are recorded below.

## Grammar

Deferred to Lab 2. No integrated parser or complete context-free grammar exists.
The preliminary `src/parser.rs` is not declared as a module by `src/main.rs`.
Lexical patterns above are implemented; expression precedence and statement
syntax validation are not. Recognizing a keyword does not validate its use.

## Parse output format

Deferred to Lab 2. There is no tree-output contract yet; `--parse` currently
opens the scanner REPL as a placeholder.

## Semantics

### Values and types

CSL is intended to be dynamically typed. Rust's static types describe the host
implementation, not a CSL runtime. Scanner numbers are `f64` and string literals
are Rust strings; runtime values and type rules are deferred to Lab 3.

### Value printing

Runtime value printing is deferred to Lab 3. Token display is defined above and
must not be confused with evaluating `print`.

### Truthiness

Deferred to Lab 3; no condition evaluation is implemented.

### Operator semantics

Deferred to Lab 3, including operand compatibility, string addition, mixed types,
equality rules, and division by zero. Operators currently produce tokens only.

### Scope and bindings

Deferred to Lab 4, including declaration, shadowing, uninitialized bindings, and
undefined-name handling. `var x = 1;` does not create a stored variable today.

### Control flow and functions

Deferred to Lab 5, including logical return values, dangling-else behavior,
closures, return defaults, and arity errors.

## Native functions

Deferred to Lab 5. No native functions are callable. `inspect`, `isolate`, and
`execute` are reserved tokens, not operational APIs.

## Errors and diagnostics

| Condition | Diagnostic example | File exit code |
| --- | --- | --- |
| Unexpected character `#` | `line 1: Unexpected character '#'.` | 65 |
| Unclosed string | `line 1: Unterminated string.` | 65 |
| Numeric conversion failure or nonfinite result | `line 1: Numeric literal out of range.` | 65 |

Errors include the actual source line; unclosed multiline strings use their
opening line. After a bad character or number, scanning continues to collect
later errors. An unclosed string consumes through EOF. Diagnostics appear in
source order on stderr. If any lexical error occurs, the entire file's stdout
is empty, even if valid tokens precede the error. Clean scans exit 0.

The REPL prints lexical diagnostics and continues; normal EOF exits 0 despite
previous bad lines. Missing filename/file-read errors exit 1 with Rust error text
on stderr; that text is platform-dependent. Unknown options currently produce
the legacy greeting rather than usage errors. Exit 70 is a future runtime code.

## Testing conventions

| Location | Activity | Extension / mode | Flag |
| --- | --- | --- | --- |
| `tests/lab0` | Legacy greeting | `.src` / sidecar | None |
| `tests/lab1` | Scanner | `.csl` / sidecar | `--tokenize` |
| `tests/cli_tests.rs` | CLI contracts | Rust integration tests | Via Cargo |

Lab 1 contains category folders for comments, coverage, errors, identifiers,
keywords, numbers, operators, strings, and whitespace. Empty-input fixtures and
one shared manifest stay at its root. Discovery is recursive; each `.csl` stays
beside its `.expected` and, for rejection, `.exit` containing `65`. See the
[fixture coverage guide](tests/lab1/README.md) for each case's purpose. Preserve
intentional CRLF bytes in the whitespace and control-character fixtures.

The harness compares stdout and exit codes, not stderr. Rust CLI tests assert
exact diagnostics, REPL recovery, stream separation, and repeated-run output;
unit tests also check numeric limits and unchanged stored values after display.
Expectations must be reviewed against this specification, not blindly regenerated.

From the repository root:

```bash
curl -fSL https://raw.githubusercontent.com/WhiteLicorice/cmsc-124-harness/v1.1/run_tests.py -o run_tests.py
./build.sh
python3 run_tests.py tests/lab0
python3 run_tests.py tests/lab1
cargo test --locked
```

Do not commit the downloaded harness. The existing GitHub Actions workflow runs
both fixture folders on push; local success does not establish remote CI status.
There are no Lab 2–5 suites yet. Additional engineering checks are
`cargo check --locked`, `cargo clippy --locked --all-targets`, and
`cargo fmt --check`. These check the compiled scanner and tests; the unreferenced
parser draft is outside the current Cargo module graph.

## Sample code

These examples show tokenization, not execution. Save the first as `example.csl`
with a final newline and run `./run --tokenize example.csl`:

```text
var greeting = "hello";
```

Output:

```text
Token(type=VAR, lexeme=var, literal=null, line=1)
Token(type=IDENTIFIER, lexeme=greeting, literal=null, line=1)
Token(type=EQUAL, lexeme==, literal=null, line=1)
Token(type=STRING, lexeme="hello", literal=hello, line=1)
Token(type=SEMICOLON, lexeme=;, literal=null, line=1)
Token(type=EOF, lexeme=, literal=null, line=2)
```

The existing `tests/lab1/strings/string_multiline.csl` contains:

```text
"a
b" next
```

Run `./run --tokenize tests/lab1/strings/string_multiline.csl`. Output:

```text
Token(type=STRING, lexeme="a\nb", literal=a\nb, line=1)
Token(type=IDENTIFIER, lexeme=next, literal=null, line=2)
Token(type=EOF, lexeme=, literal=null, line=3)
```

## Design rationale

- Explicit punctuation keeps indentation out of scanning and leaves statement
  boundaries visible; semicolons and braces are intended C-family conventions.
- Whole-name keyword lookup allows `variable` and `iffy` without splitting them
  into a keyword and a suffix. ASCII identifiers keep the naming rule explicit;
  Unicode string contents remain supported.
- One floating-point representation handles integer and decimal spellings without
  separate scanner value types. Rejecting infinity avoids silently replacing an
  oversized source value; this finite-number policy supplies our design-specific
  third rejection case, rather than adding unneeded comment or escape features.
- Raw strings and line comments limit scanner complexity. Multiline file strings
  retain their actual contents; display-only escaping makes diagnostics and token
  comparisons readable while distinguishing a real newline from backslash-plus-n.
- CSL keywords reserve a vocabulary for event sources, capabilities, aggregation,
  and responses. The cost is that users cannot reuse these names as identifiers;
  recognizing them does not promise working monitoring operations.
- Future runtime work should validate targets, capabilities, and permitted
  operations before evaluation, then add deterministic event-window/group/count
  behavior and simulated responses. Optional RFID/CAD integration would expose
  only an enrolled-credential authorization result to the runtime, not raw RFID
  access. Those adapters and their tests remain outside Lab 1.

## Known limitations

- No integrated parsing, evaluation, variable storage, event monitoring, response
  execution, or RFID verification. The `--parse` route is only a prompt placeholder.
- No string escape processing, block/nested comments, Unicode identifiers,
  exponent literals, or exact/arbitrary-precision arithmetic.
- REPL input is processed one line at a time; multiline strings require files.
- Diagnostics report lines, not columns. CLI option validation and OS-error
  formatting are not a stable language interface yet.
- Rust `stable` is not an exact toolchain pin. CI/toolchain hardening remains
  Step 8. The uncompiled parser draft still needs its own implementation and
  validation; clean scanner checks do not establish parser readiness.

## Changelog

| Activity | What changed in the language |
| --- | --- |
| Lab 1 | Established the token vocabulary, `.csl`, line comments, raw multiline strings, finite numeric literals, ordered lexical diagnostics, and scanner REPL. Numeric overflow now rejects with 65. Token display changed from raw text fields to escaped backslash/LF/CR/tab without altering stored values; ordinary token formatting remains compatible. Parsing and runtime semantics remain deferred. |

This specification follows the [CMSC 124 language specification template](https://renscourses.netlify.app/articles/cmsc-124-spec)
and the project's supplied Lab 1 PDF. The PDF remains authoritative for assignment
requirements; language choices above state this project's current contract.
