# Cyber Sequence Language (CSL)

This repository implements a scanner and a recursive descent expression parser.
The parser builds and prints an abstract syntax tree (AST); it does not evaluate
expressions. Token purposes below describe intended language semantics.
Declarations, statements, and CSL operations are not parsed or executed yet.

## Running the interpreter

Build with `./build.sh`, then use `./run`, `./run --repl`, or `./run --parse`
to start the expression REPL. Enter one expression per line; the REPL prints its
AST and continues after scanning or parsing errors. Ctrl-D ends the session on
Unix. Each entered line starts at source line 1, and the `> ` prompt is printed
before each read, including the final read that reaches EOF.

Use `./run --parse <file>` to parse a file, or pass a `.csl` filename directly.
The original `./run tests/lab0/hello.src` invocation still prints the Lab 0
greeting. Positional `.src` files use that compatibility behavior; an explicit
`--parse` parses files regardless of their extension. Use
`./run --tokenize <file>` for the unchanged Lab 1 token output.

For example, `./run --parse tests/lab2/precedence.csl` prints:

```text
(- (+ 1.0 (* 2.0 3.0)) 4.0)
```

Successful files exit 0. Scanning or parsing errors go to stderr and exit 65,
with no ASTs or tokens printed to stdout. Argument and file-access errors exit 1.
The REPL reports errors on stderr and stays open; reaching EOF exits 0 even if
an earlier entry failed.

## Grammar

The implemented expression grammar follows the
[Lab 2 brief](docs/labs/lab2.md). Precedence increases from equality through
primary expressions:

```text
expression → equality
equality   → comparison ( ( "!=" | "==" ) comparison )*
comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
term       → factor ( ( "-" | "+" ) factor )*
factor     → unary ( ( "/" | "*" ) unary )*
unary      → ( "!" | "-" ) unary | primary
primary    → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")"

file       → expression ( ";" expression )* ";"? EOF
repl       → expression ";"? EOF
```

Binary operators associate to the left: `10 - 3 - 2` prints
`(- (- 10.0 3.0) 2.0)`. Unary operators nest to the right: `--10` prints
`(- (- 10.0))`. Parentheses produce an explicit `group` node, so
`(1 + 2) * 3` prints `(* (group (+ 1.0 2.0)) 3.0)`. Ordinary integral numbers
print with `.0`, strings print their contents without quotes, booleans print `true` or
`false`, and `nil` prints `nil`. These are tree representations, not results:
even `1 / 0` is a valid expression to parse.

Files may contain several expressions separated by semicolons, with an optional
final semicolon. Newlines remain whitespace, so a file expression may span
lines. The REPL accepts one complete expression per entry, also with an optional
final semicolon; it does not collect multiline expressions. Neither interface
silently accepts trailing tokens. Empty or comment-only input is an error, as
in the laboratory examples, and repeated semicolons create empty expressions
that are also rejected.

After a file parsing error, the parser skips through the next semicolon and
tries the following expression. It collects diagnostics in source order and
rejects the whole file if any expression failed. Without a semicolon, recovery
reaches EOF. For example, `1 + ;` on line 1 followed by `2 * ;` on line 2
reports both errors:

```text
[line 1] Error at ';': Expect expression.
[line 2] Error at ';': Expect expression.
```

Identifiers, assignment, unary `+`, `and`/`or`, calls, arrays, declarations,
and CSL rules remain outside this expression grammar even when their tokens
are recognized. Chained comparisons are parsed left to right; no evaluation or
type checking is performed. The scanner's existing numeric and raw-string
rules still apply: exponent syntax is unsupported, oversized numbers are
rejected, and large or small values may print in scientific notation. AST string
output preserves embedded newlines and backslashes without display escaping;
token output escapes control characters and backslashes. A lexical error rejects scanning before
the parser runs.

## Tests

Run the three laboratory suites after building:

```sh
./build.sh
python3 run_tests.py tests/lab0
python3 run_tests.py tests/lab1
python3 run_tests.py tests/lab2
cargo test
```

The course harness is downloaded by CI from
[`cmsc-124-harness/v1.1`](https://github.com/WhiteLicorice/cmsc-124-harness/blob/v1.1/run_tests.py)
and is ignored locally. Lab 0 and Lab 1 retain their sidecar expectations.
Lab 2 uses inline `// expect:` and `// expect error:` annotations so the
harness checks AST output, stderr diagnostics, and exit status. Rust tests also
check exact diagnostics, suppression of partial output, CLI arguments, and
REPL continuation. CI runs all three laboratory suites and `cargo test`.

- Jemarco Briz ([Jmbriz123](https://github.com/Jmbriz123))
- Dejel Cyrus De Asis ([dejely](https://github.com/dejely))

## Overview

CSL is a proposed dynamically typed language for people describing authorized
monitoring and response sequences: watch an event source, group or count events
within a time window, require a capability, then describe an alert or inspection.
Its intended syntax combines explicit punctuation with domain words such as
`watch`, `source`, and `sequence`. This CMSC 124 prototype currently implements
lexical scanning and expression parsing. Lab 1 covers the scanner, diagnostics,
tests, and lexical specification; Lab 2 adds expression ASTs and printing.
Evaluation remains a later milestone. Event
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
use Git Bash. Node/Prettier configuration is development tooling, not required
to build or run the interpreter.

## Running it

| Command | Current behavior |
| --- | --- |
| `./run` | Starts the expression REPL |
| `./run --repl` | Explicit alias for the expression REPL |
| `./run --tokenize <file>` | Scans a UTF-8 source file and prints tokens |
| `./run tests/lab0/hello.src` | Prints `Hello, JM & Dejel!` for Lab 0 compatibility |
| `./run <file>` | Parses expressions; positional `.src` files retain the Lab 0 greeting |
| `./run --parse <file>` | Scans the file, parses expressions, and prints ASTs |
| `./run --eval <file>` | Not implemented; reports a usage error with exit 1 |

Example file invocation:

```bash
./run --tokenize tests/lab1/coverage/categories.csl
```

The REPL prints `> `, scans and parses one input line, and prints an AST or diagnostics.
Each line is independent, with line numbering starting at 1; `var x = 1;` is
rejected by the expression grammar. A bad line such as `#` reports an error and returns to the
prompt. End input with EOF (Ctrl-D on Unix-like terminals). Multiline strings
must be scanned from files because the REPL does not accumulate input lines.

Clean file scans or parses and normal REPL exit use code 0. Lexically or
syntactically rejected files use 65, with diagnostics on stderr and no stdout.
Missing/unreadable input files or invalid arguments use code 1 through Rust's error return.
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

The scanner emits operator tokens; the parser applies the precedence and
associativity below. Levels increase from equality (1) to unary (5); literals
and grouping are primary expressions. Assignment and logical conjunction or
disjunction are recognized only by the scanner.

| Operator | Token type | Category | Operands | Associativity | Precedence |
| --- | --- | --- | --- | --- | --- |
| `+` | `PLUS` | Arithmetic addition | Binary | Left | 3 |
| `-` | `MINUS` | Subtraction / negation | Binary / unary | Left / right | 3 / 5 |
| `*` | `STAR` | Arithmetic multiplication | Binary | Left | 4 |
| `/` | `SLASH` | Arithmetic division | Binary | Left | 4 |
| `=` | `EQUAL` | Assignment | Binary | Deferred | Deferred |
| `==` | `EQUAL_EQUAL` | Equality | Binary | Left | 1 |
| `!=` | `BANG_EQUAL` | Inequality | Binary | Left | 1 |
| `<` | `LESS` | Comparison | Binary | Left | 2 |
| `<=` | `LESS_EQUAL` | Comparison | Binary | Left | 2 |
| `>` | `GREATER` | Comparison | Binary | Left | 2 |
| `>=` | `GREATER_EQUAL` | Comparison | Binary | Left | 2 |
| `!` | `BANG` | Logical negation | Unary | Right | 5 |
| `and` | `AND` | Logical conjunction | Binary | Deferred | Deferred |
| `or` | `OR` | Logical disjunction | Binary | Deferred | Deferred |

Longest valid matches win: `!=` emits one `BANG_EQUAL`, and `===` emits
`EQUAL_EQUAL` followed by `EQUAL`. A single `/` is an operator unless followed
by another slash beginning a comment.

Punctuation and single-character vocabulary (meanings beyond tokenization are
reserved for later stages):

## Cyber Sequence Language (CSL)

Cyber Sequence Language (CSL) is the proposed domain language for describing
authorized monitoring and response sequences. A CSL rule can identify an event
source, group or count matching events in a time window, state the capability
it needs, and describe the sequence that should produce an alert or an
inspection result. The interpreter currently tokenizes CSL source and parses
the expression grammar above; CSL rule parsing, validation, and execution
are still planned work.

CSL also has an optional hardware-verification component. An RFID reader can
act as a credential access device (CAD): before a protected rule or response is
accepted, it reads an enrolled RFID credential and passes its verification
result to CSL. This is intended only for authorized users, enrolled credentials,
and approved systems. The language should receive a simple verification result,
such as an authorized credential identity or a rejection, rather than expose
raw RFID access to every rule.

### Planned CSL features

* Event and log-source monitoring with `watch` and `source`.
* Rule inputs that identify a `target`, a `require`d capability, and an
  `ability` or `sequence` to run.
* Event aggregation with `count`, `by`, and `within`.
* Alerting and controlled response intents with `alert`, `inspect`, `isolate`,
  and `execute`.
* Optional RFID/CAD authorization verification before protected operations.

### CSL keyword trackerz  

The checked items are recognized by the current scanner. They still need parser
rules and runtime behavior before they become usable CSL statements.

- [x] `watch` (`WATCH`) — declares an event-monitoring rule.
- [x] `ability` (`ABILITY`) — declares a named ability.
- [x] `target` (`TARGET`) — specifies the target of an ability.
- [x] `source` (`SOURCE`) — specifies an event or log source.
- [x] `sequence` (`SEQUENCE`) — groups operation steps.
- [x] `require` (`REQUIRE`) — declares a required capability.
- [x] `capability` (`CAPABILITY`) — introduces a capability reference.
- [x] `count` (`COUNT`) — counts matching events.
- [x] `by` (`BY`) — groups events by a field.
- [x] `within` (`WITHIN`) — sets a query time window.
- [x] `alert` (`ALERT`) — raises an alert.
- [x] `inspect` (`INSPECT`) — requests an inspection of an approved target.
- [x] `isolate` (`ISOLATE`) — requests isolation of an approved target.
- [x] `execute` (`EXECUTE`) — requests execution of declared, authorized operations.

### CSL implementation checklist

- [x] Add CSL keyword token types and scanner recognition.
- [x] Add the `@` token for resource references such as `@auth_logs`.
- [ ] Define the CSL grammar and abstract syntax tree for rules, sources,
  capabilities, and sequences.
- [ ] Parse CSL statements and report source locations for invalid syntax.
- [ ] Validate targets, required capabilities, and allowed operations before
  evaluation.
- [ ] Implement event-source adapters and deterministic `count`, `by`, and
  `within` evaluation.
- [ ] Implement alerts and simulated `inspect`, `isolate`, and `execute`
  results for development and testing.
- [ ] Define an RFID/CAD adapter that provides only an authorization result to
  the runtime, with enrolled-credential and audit requirements.
- [ ] Add unit and integration tests for CSL tokenization, parsing,
  authorization, and rejected operations.

### End-of-input token

| Lexeme        | Token Type | Purpose                           |
| ------------- | ---------- | --------------------------------- |
| End of source | `EOF`      | Marks the end of the token stream |

## Language Design Decisions

### Agreed lexical rules

These rules describe the scanner unless a row is explicitly marked pending.
Token sequences below omit the final `EOF` for brevity. The grammar patterns
are documentation, not regular expressions used by the implementation.

| Area | Rule and concrete example | Implementation status |
| --- | --- | --- |
| Number | `42`, `3.14`; `[0-9]+(\.[0-9]+)?` | `NUMBER` with finite `f64` value |
| String | `"hello"`, `""`, or quoted multiline text | `STRING` with contents excluding quotes |
| Boolean | `true`, `false` | Keyword tokens; the parser creates Boolean AST literals |
| Nil | `nil` | Keyword token; the parser creates a null AST literal |

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

## Parser implementation

The [expression grammar](#grammar) is implemented in `src/parser.rs` and
integrated through `src/main.rs`. It validates expression syntax and builds
the AST nodes in `src/expr.rs`. Statement and CSL rule grammars remain deferred.

## Parse output format

The AST printer uses parenthesized prefix notation, as documented in the
[expression grammar](#grammar). Files print one AST per expression after the
whole file parses successfully; the REPL prints one AST per successful entry.

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
| `./run --tokenize tests/lab1/coverage/categories.csl` | Prints tokens | Preserve |
| `./run --repl` | Parses one expression per line; a bad line does not end the session | Implemented |
| `./run` | Starts the expression REPL | Implemented |
| `./run tests/lab0/hello.src` | Prints `Hello, JM & Dejel!` | Preserve legacy Lab 0 behavior |
| Rust toolchain | `stable` in `rust-toolchain.toml` | **Step 8:** pin audited version `1.98.0` |

CSL source uses `.csl`; the `.src` Lab 0 fixture remains a compatibility exception.
The default REPL satisfies the no-argument interface while keeping the
earlier Lab 0 harness invocation working.

### Intended syntax and deferred semantics

CSL is intended to be dynamically typed. Rust's static types describe the scanner
and parser implementation, not an implemented CSL type system. For example,
`var x = 1;` produces tokens with `--tokenize`, but the expression parser
rejects it; it does not create a variable.

* `{}` will delimit blocks, as in `{ print x; }`.
* `[]` is reserved for arrays/indexing, as in `items[0]`.
* `;` will terminate statements, as in `print x;`.
* `()` groups expressions such as `(1 + 2)`; calls such as `inspect(host)`
  remain planned.
* Reserved words such as `if`, `else`, `for`, `fun`, `and`, and `var` are already
  recognized; their execution semantics remain deferred.

Explicit punctuation avoids indentation-sensitive scanning. Whole-name keyword
lookup keeps identifiers such as `variable` usable. Raw strings and line comments
keep the scanner small enough to explain and test. Finite numeric values
and escaped token records are now implemented.

The Lab 1 scope is the scanner, diagnostics, tests, and documentation. Lab 2 adds
expression parsing, operator precedence, AST printing, and parser diagnostics.
Runtime typing, statements, rule evaluation, and RFID/CAD integration remain
future work. Recognizing `watch` or `inspect` does not implement monitoring or
inspection.

## File Structure

| File name        | Purpose                                                                   |
| ---------------- | ------------------------------------------------------------------------- |
| `src/token.rs`   | Defines `TokenType`, `Literal`, and `Token`                               |
| `src/scanner.rs` | Reads the raw source code character by character and turns it into tokens |
| `src/expr.rs` | Defines expression AST nodes and their printed representation |
| `src/parser.rs` | Parses expressions and recovers at file-expression boundaries |
| `src/repl.rs` | Reads and parses one expression per input line |
| `src/main.rs` | Dispatches the file parser, tokenizer, REPL, and Lab 0 command |
| `tests/lab2/` | Parser fixtures with inline AST and diagnostic expectations |
| `tests/cli.rs` | Checks CLI output, diagnostics, exit status, and REPL recovery |
| `tests/cli_tests.rs` | Checks scanner regressions and default REPL compatibility |
