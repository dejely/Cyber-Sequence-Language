# Cyber Sequence Language (CSL)

This repository implements a scanner, a recursive descent parser, and a
tree-walking evaluator. Lab 4 Weeks 1–3 add print and expression statements,
variable declarations, lookup, assignment, and nested lexical blocks. Lab 5
Week 1 adds `if`/`else`, `while`, and `for` control flow.
CSL monitoring operations remain deferred.

## Running the interpreter

Build with `./build.sh`, then use `./run` or `./run --repl` to start the
program REPL. Enter one expression, `print expression;`, `var name = value;`,
a complete `{ ... }` block, or a complete `if`, `while`, or `for` statement per line. Expressions echo their values, print statements print once, and
declarations are silent. Variables persist across inputs and errors. It continues after scanning,
parsing, or runtime errors. `./run --parse`
starts an AST-printing REPL. Ctrl-D ends the session on
Unix. Each entered line starts at source line 1, and the `> ` prompt is printed
before each read, including the final read that reaches EOF.

Use `./run <file>` to execute a program: statements run in order, and only
`print` statements produce output. Declarations, print statements, and expression
statements require a semicolon; blocks end with `}` and need no semicolon.
The Lab 0 `hello.src` fixture is now a real CSL print program, with the same
expected greeting. File extensions do not change execution behavior. Use
`./run --parse <file>` for expression ASTs, `./run --eval <file>` for expression
values, or `./run --tokenize <file>` for the unchanged Lab 1 token output.

For example, `./run --parse tests/lab2/precedence.csl` prints:

```text
1.0 2.0 3.0 * + 4.0 -
```

Successful files exit 0. Scanning or parsing errors go to stderr and exit 65,
with no program output, ASTs, or tokens printed to stdout. Runtime errors exit 70;
output from earlier statements remains visible. Argument and file-access errors exit 1.
The REPL reports errors on stderr and stays open; reaching EOF exits 0 even if
an earlier entry failed.

For the Week 1 statement example, run `./run examples/week1.csl`:

```csl
print "Hello!";
print 2 + 3;
2 + 3;
```

This prints `Hello!` and `5` on separate lines. The last statement evaluates
without printing. `Stmt::Print` and `Stmt::Expression` both reuse the existing
expression evaluator; the statement kind controls whether its value is printed.

For Week 2, run `./run examples/week2.csl`. It declares a fruit, stock, sales,
and price; updates the stock; and prints `banana`, `9`, and `15`. In the REPL:

```text
> var stock = 12;
> stock = stock - 3
9
> print stock;
9
```

For Week 3, run `./run examples/week3.csl`. A banana stall records two sales,
using a nested discount price while updating stock and revenue in the outer
scope. It prints `4`, `3`, `banana`, `15`, `23`, and `5` on separate lines.
The discount and sale variables disappear when their blocks end.

The [Lab 4 brief](docs/labs/lab4.md) links the course requirements and records
which weekly features are implemented. The [Lab 5 brief](docs/labs/lab5.md)
does the same for control flow and functions.

## Grammar

The implemented expression grammar follows the
[Lab 2 brief](docs/labs/lab2.md). Precedence increases from equality through
primary expressions; assignment has the lowest precedence:

```text
expression → assignment
assignment → IDENTIFIER "=" assignment | equality
equality   → comparison ( ( "!=" | "==" ) comparison )*
comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
term       → factor ( ( "-" | "+" ) factor )*
factor     → unary ( ( "/" | "*" ) unary )*
unary      → ( "!" | "-" ) unary | primary
primary    → NUMBER | STRING | "true" | "false" | "nil" | IDENTIFIER | "(" expression ")"

expressionFile → expression ( ";" expression )* ";"? EOF
expressionRepl → expression ";"? EOF

program     → declaration* EOF
declaration → varDecl | statement
varDecl     → "var" IDENTIFIER ( "=" expression )? ";"
statement   → printStmt | exprStmt | block | ifStmt | whileStmt | forStmt
block       → "{" declaration* "}"
printStmt   → "print" expression ";"
exprStmt    → expression ";"
ifStmt      → "if" "(" expression ")" statement ( "else" statement )?
whileStmt   → "while" "(" expression ")" statement
forStmt     → "for" "(" ( varDecl | exprStmt | ";" )
                  expression? ";"
                  expression? ")" statement
programRepl → ( varDecl | statement | expression ";"? ) EOF
```

Branches and loop bodies are `statement`, not `declaration`, so
`if (x) var y = 1;` is a static error; wrap a declaration in braces.
Because a branch can itself be an `if`, `else if` needs no extra rule. A
dangling `else` binds to the nearest unmatched `if`: the innermost
`if_statement` call is the one still parsing when `else` arrives.
`for` never reaches the evaluator. The parser desugars it into blocks and
`while`, as described under [Control flow](#control-flow-and-functions).

Binary operators associate to the left: `10 - 3 - 2` prints
`10.0 3.0 - 2.0 -`. Unary operators nest to the right: `--10` prints
`10.0 - -`. Parentheses produce an explicit `group` node in the AST, but the printer omits
the group marker because postfix order already preserves grouping. Thus
`(1 + 2) * 3` prints `1.0 2.0 + 3.0 *`. Ordinary integral numbers
print with `.0`, strings print their contents without quotes, booleans print `true` or
`false`, and `nil` prints `nil`. These are tree representations, not results:
even `1 / 0` is a valid expression to parse.

In `--parse` and `--eval` modes, files may contain several expressions separated
by semicolons, with an optional
final semicolon. Newlines remain whitespace, so a file expression may span
lines. The REPL accepts one complete expression per entry, also with an optional
final semicolon; it does not collect multiline expressions. Print statements and
variable declarations require a semicolon even in the default REPL. A REPL block
must fit on one input line; its internal statements still require semicolons.
Neither interface
silently accepts trailing tokens. Empty or comment-only input in expression modes is an error, as
in the laboratory examples, and repeated semicolons create empty expressions
that are also rejected.

Programs may be empty or contain only comments; both execute without output.
In script mode, expression-statement values are discarded but still evaluated,
so `1 / 0;` reports a runtime error. Only `print` writes program output.
All script statements are parsed before any execute; syntax errors prevent
the entire program from running.

In expression modes, after a file parsing error, the parser skips through the next semicolon and
tries the following expression. It collects diagnostics in source order and
rejects the whole file if any expression failed. Without a semicolon, recovery
reaches EOF. For example, `1 + ;` on line 1 followed by `2 * ;` on line 2
reports both errors:

```text
[line 1] Error at ';': Expect expression.
[line 2] Error at ';': Expect expression.
```

Program recovery also stops before a `var`, `print`, `{`, `if`, `while`, or
`for` statement starter
and preserves a closing `}` for the block that owns it. A stray `}` at top level
is diagnosed and consumed. Errors inside nested blocks are collected in source
order; EOF before a closing brace reports `Expect '}' after block.`. Syntax errors
anywhere reject the entire program, including earlier valid statements.

Assignment associates to the right: `a = b = 7` stores `7` in both existing
bindings and evaluates to `7`. Its postfix AST is `a b 7.0 = =`. Only a bare
identifier is an assignment target; `(a) = 7` and `1 = 7` are static errors.
Declarations are accepted in programs and the default REPL; `--parse` and
`--eval` retain their expression-only interfaces. Unary `+`, `and`/`or`, calls,
functions, `return`, `break`, arrays, and CSL rules remain outside the grammar. Chained comparisons are parsed left to right; the evaluator
checks operand types at runtime. The scanner's existing numeric and raw-string
rules still apply: exponent syntax is unsupported, oversized numbers are
rejected, and large or small values may print in scientific notation. AST string
output preserves embedded newlines and backslashes without display escaping;
token output escapes control characters and backslashes. A lexical error rejects scanning before
the parser runs.

## Tests

Run the six laboratory suites after building:

```sh
./build.sh
python3 run_tests.py tests/lab0
python3 run_tests.py tests/lab1
python3 run_tests.py tests/lab2
python3 run_tests.py tests/lab3
python3 run_tests.py tests/lab4
python3 run_tests.py tests/lab5
cargo test
```

The course harness is downloaded by CI from
[`cmsc-124-harness/v1.1`](https://github.com/WhiteLicorice/cmsc-124-harness/blob/v1.1/run_tests.py)
and is ignored locally. Lab 0 and Lab 1 retain their sidecar expectations.
Lab 2 uses inline `// expect:` and `// expect error:` annotations. Lab 3 adds
`// expect runtime error:` checks for runtime failures. Rust tests also
check exact diagnostics, suppression of partial output, CLI arguments, and
REPL continuation. Lab 4 tests plain file execution, explicit output, discarded
values, variables, assignment order, redeclarations, nested lookup, shadowing,
outward assignment, scope exit, and static/runtime errors. The manifest discovers
all `.csl` fixtures, with inline expectations and no execution flag. REPL
integration tests check scope restoration after success and errors. Lab 5
tests every `if` branch, truthy and falsey conditions, a missing `else`,
`else if` chains, the dangling `else`, nested conditionals, `while` and `for`
loops (including zero iterations, empty clauses, and loop-variable scope), and
static and runtime errors. Untaken branches and skipped loop bodies contain
`1 / 0`, so a test fails if they run. CI runs all six laboratory suites and
`cargo test`.

- Jemarco Briz ([Jmbriz123](https://github.com/Jmbriz123))
- Dejel Cyrus De Asis ([dejely](https://github.com/dejely))

## Overview

CSL is a proposed dynamically typed language for people describing authorized
monitoring and response sequences: watch an event source, group or count events
within a time window, require a capability, then describe an alert or inspection.
Its intended syntax combines explicit punctuation with domain words such as
`watch`, `source`, and `sequence`. This CMSC 124 prototype currently implements
lexical scanning, expression parsing, expression evaluation, and basic statements. Lab 1 covers the scanner, diagnostics,
tests, and lexical specification; Lab 2 adds expression ASTs and printing;
Lab 3 adds the tree-walking evaluator and runtime errors; Lab 4 Week 1 adds
print and expression statements in programs; Week 2 adds global variables and
assignment; Week 3 adds blocks and nested lexical lookup. Event
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
| `./run` | Starts the program REPL |
| `./run --repl` | Explicit alias for the program REPL |
| `./run --tokenize <file>` | Scans a UTF-8 source file and prints tokens |
| `./run tests/lab0/hello.src` | Executes the CSL program that prints the Lab 0 greeting |
| `./run <file>` | Executes statements; only explicit print statements produce output |
| `./run --parse <file>` | Scans the file, parses expressions, and prints ASTs |
| `./run --eval <file>` | Evaluates expressions and prints one value per line |

Example file invocation:

```bash
./run --tokenize tests/lab1/coverage/categories.csl
```

The default REPL prints `> `, scans and parses one input line, and prints a value or diagnostics.
Each line starts at source line 1 and accepts one expression, print statement,
variable declaration, or complete block. The interpreter retains its environment for the session. A bad line such as `#` reports an error and returns to the
prompt. End input with EOF (Ctrl-D on Unix-like terminals). Multiline strings
must be scanned from files because the REPL does not accumulate input lines.

Clean file scans or parses and normal REPL exit use code 0. Lexically or
syntactically rejected files use 65, with diagnostics on stderr and no stdout.
Missing/unreadable input files or invalid arguments use code 1 through Rust's error return.
Evaluation file runtime errors use exit 70; the evaluation REPL continues after errors.

## File extension

CSL source files use `.csl`, matching `tests/lab1/manifest.json`. The Lab 0
fixture retains its `.src` filename, but contains ordinary CSL source.

## Lexical structure

Scanning is handwritten, character by character, without regular expressions.
Patterns below describe the grammar; they are not regex-based implementation.
All token categories are defined in [src/token.rs](src/token.rs); scanning is in
[src/scanner.rs](src/scanner.rs).

### Keywords

The words below are recognized now. Their purposes describe intended semantics;
`print` and `var` execute statements; `true`, `false`, and `nil` are literals.
The remaining keywords are reserved for future forms.

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

The statement terminator is `;`, blocks use `{}`, grouping/calls use
`()`, and `[]` is reserved for arrays/indexing. `{ print x; }` parses as a block;
`items[0]` only tokenizes today. Newlines do not terminate statements
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
the AST nodes in `src/expr.rs`. Script parsing builds print and expression
statements in `src/stmt.rs`; CSL rule grammars remain deferred.

## Parse output format

The AST printer uses postfix notation (operands before operators), as documented in the
[expression grammar](#grammar). `--parse` files print one AST per expression
after the whole file parses successfully; the `--parse` REPL prints one AST per
successful entry.

## Semantics

These decisions are frozen for Lab 3. The evaluator in `src/evaluator.rs`,
the value type in `src/value.rs`, and the fixtures in `tests/lab3/` must agree
with every rule below.

### Value representation

CSL is dynamically typed, but every Rust variable has one static type. Runtime
values therefore use a tagged union, the Rust enum `Value` in `src/value.rs`:

```rust
pub enum Value { Number(f64), String(String), Boolean(bool), Nil }
```

- **Uniform storage:** every evaluator function returns `Value`, whatever the
  CSL type turns out to be.
- **Runtime interrogation:** `match` reads the tag, and the compiler rejects a
  match that forgets a variant.
- **Conversion:** a host `f64` goes in with `Value::Number(x)` and comes out by
  matching `Value::Number(x)`; arithmetic happens on the host `f64`.

We rejected `Box<dyn Any>` with downcasting. It rebuilds Java's `Object`
approach and loses exhaustiveness checking. Strings are owned `String`s compared
by content, so `"ab" == "ab"` is `true`; there is no reference identity to get
wrong.

### Value printing

`./run --eval <file>` evaluates semicolon-separated expressions and prints one
value per line. The REPL prints with the same `Display` implementation, so the
two modes cannot format differently. `--parse` still prints ASTs.

| Value | Prints as | Example |
| --- | --- | --- |
| Integral number | No trailing `.0` | `2 + 3` → `5` |
| Fractional number | Shortest round-trip form | `7 / 2` → `3.5`, `0.1 + 0.2` → `0.30000000000000004` |
| Negative zero | `-0` (IEEE 754 sign kept) | `-0` → `-0` |
| String | Contents, no quotes | `"hi" + " there"` → `hi there` |
| Boolean | `true` / `false` | `1 < 2` → `true` |
| Nil | `nil` | `nil` → `nil` |

Large integral numbers print every digit (`1000000 * 1000000` →
`1000000000000`); there is no exponent notation. Script `print` statements
use this same value formatting.

### Truthiness

`false` and `nil` are falsey. **Every other value is truthy**, including `0`,
`-1`, and the empty string `""`. This is the Lox rule. It needs one check and
no per-type exceptions, so `!0` is `false` and `!!""` is `true`. The rule
lives in one function, `Value::is_truthy`, which Lab 5 `if`, `while`, and
`for` conditions reuse. `tests/lab3/truthiness.csl` is the executable form of this rule.

### Operators

| Operator | Operands | Result | Otherwise |
| --- | --- | --- | --- |
| `-x` | number | negated number | `Operand must be a number.` |
| `!x` | any value | `!is_truthy(x)` | never fails |
| `+` | two numbers / two strings | sum / concatenation | `Operands must be two numbers or two strings.` |
| `-` `*` `/` | two numbers | number | `Operands must be numbers.` |
| `<` `<=` `>` `>=` | two numbers | boolean | `Operands must be numbers.` |
| `==` `!=` | any pair | boolean | never fails |

- **`+` with strings:** two strings concatenate (`"a" + "b"` → `ab`). Mixing
  a string with anything else is a runtime error (`"a" + 1`). There is no
  implicit conversion, so a typo does not silently produce `"a1"`.
- **Equality:** values of different variants are unequal, never an error.
  `1 == "1"`, `0 == false`, and `nil == false` are all `false`; `1 == 1.0`
  and `0 == -0` are `true`.
- **Comparison** only orders numbers. Strings and booleans cannot be compared
  with `<`.
- **Evaluation order:** operands evaluate **left to right**, and a node
  evaluates its children before itself (post-order). This is invisible while
  expressions have no side effects, but it is fixed now so that
  `print f() + g();` in Lab 5 always runs `f` first.

### Division by zero and overflow

Dividing by zero (`1 / 0`, `0 / 0`) is a **runtime error**:
`Division by zero.`, exit 70. IEEE 754 would give `Infinity` or `NaN`. We
reject that because a script that divides by zero almost always has a bug, and
CSL rule counts and rates should fail loudly rather than carry `Infinity`
forward. For the same reason, an arithmetic result that overflows to infinity
(`1e308 * 10` written out in full) reports `Numeric result is not finite.`
Every number CSL prints is finite, and `NaN` can never be produced.

### Runtime errors and exit codes

Runtime errors are reported on stderr with the operator or name's line and lexeme,
using the token stored on each AST node:

```text
[line 1] Runtime error at '/': Division by zero.
```

The evaluator returns `Result<Value, RuntimeError>`, and `?` unwinds to one
place. File mode prints the error and exits **70**. The REPL prints it and
shows the next prompt. Values printed before the failing expression stay
printed.

| Outcome | Exit code |
| --- | --- |
| File evaluated successfully | 0 |
| Scanner or parser rejected the file (nothing evaluated, stdout empty) | 65 |
| File parsed, then hit a runtime error | 70 |
| Missing file or bad arguments | 1 |

Interpreter bugs aside, no CSL input should make the host panic or print a
Rust stack trace.

### Scope and bindings

CSL is imperative with mutable, dynamically typed bindings and lexical block
scope. Each `{ ... }` creates an environment linked to its enclosing scope.
Declarations always define in the current scope; reads and assignments search
from the current scope outward and select the nearest matching name. Assignment
updates that binding and never creates a new name. A local declaration shadows
an outer binding without replacing it. Exiting the block exposes the outer
binding again. Same-scope redeclaration replaces the previous value, at both
global and block scope.

`var x = 1;` evaluates the initializer before installing the binding; `var x;`
stores `nil`. An initializer may read a previous binding of the same name,
including an outer one: `var x = x + 1;` uses the nearest existing `x`. Without
one, `var x = x;` fails. A failed initializer does not install or replace its
binding; completed assignment effects inside the initializer remain.

Lookup and assignment to an undefined name produce `Undefined variable 'x'.`
at runtime (file exit 70). The right side is evaluated before assignment, and
assignment returns its value. Values and their types may change. Binary operands
still evaluate left before right. Errors stop the current input or file without
rolling back earlier assignments or printed output.

A name becomes visible and binds to storage and a value when its declaration
executes, after its initializer succeeds. Type and value bindings occur at
runtime. Global storage lasts for the file execution or REPL session; local
storage lasts for that block's execution. Local storage is stack-dynamic in the
guest language: a declaration creates storage, and block exit releases it, even
though the Rust maps and enclosing links use heap allocation. There are no explicit
allocation operations, functions, or captured bindings yet.

Every block restores its enclosing environment on success or a runtime error,
including errors inside nested blocks. The REPL can then read its session
bindings and declare new globals. Assignments to enclosing bindings completed
before the error remain visible. File expression statements discard their
values; only `print` writes output. REPL bare expressions echo values, while
expressions inside a block stay silent unless printed.

### Control flow and functions

CSL takes the imperative route: statements, mutable variables, and loops.

- **Conditionals:** `if (cond) a else b` evaluates `cond` once, applies the
  truthiness rule above, and executes only the selected branch. Without an
  `else`, a falsey condition does nothing and execution continues. `if` is a
  statement, so it produces no value.
- **`while`:** the condition is evaluated before every pass, so a falsey
  start runs the body zero times. A braced body is a block, so each pass
  starts with fresh locals that disappear at the end of the pass.
- **`for`:** desugared in the parser, so the evaluator has one loop. The loop
  `for (init; cond; incr) body` becomes
  `{ init; while (cond) { body; incr; } }`. All three clauses are optional. A
  missing condition is `true`, so `for (;;)` runs until a runtime error stops
  it. The outer block keeps a `var` loop variable from leaking into the
  enclosing scope. Every iteration shares that single loop variable, like
  JavaScript's `var` and unlike its `let`. This becomes visible only when
  closures arrive.
- **Errors:** a runtime error in a condition or body stops the program (file
  exit 70). Every block still restores its enclosing scope, so the REPL keeps
  working after a failing loop.

Known limitations: there is no `break` or `continue`, and an infinite loop
runs until the process is killed. Logical operators, functions, `return`,
closures, native functions, and arity checking arrive in later Lab 5 weeks.

## Native functions

Deferred to Lab 5. No native functions are callable. `inspect`, `isolate`, and
`execute` are reserved tokens, not operational APIs.

## Errors and diagnostics

| Condition | Diagnostic example | File exit code |
| --- | --- | --- |
| `./run --tokenize tests/lab1/coverage/categories.csl` | Prints tokens | Preserve |
| `./run --repl` | Accepts one expression, print statement, declaration, or complete block per line; errors return the prompt | Implemented |
| `./run` | Starts the program REPL | Implemented |
| `./run tests/lab0/hello.src` | Executes a CSL print statement | Same Lab 0 output |
| Rust toolchain | `stable` in `rust-toolchain.toml` | **Step 8:** pin audited version `1.98.0` |

CSL source uses `.csl`; the `.src` Lab 0 fixture is also interpreted as CSL.
The default REPL satisfies the no-argument interface while keeping the
earlier Lab 0 harness invocation working.

### Intended syntax and deferred semantics

CSL is intended to be dynamically typed. Rust's static types describe the scanner
and parser implementation, not an implemented CSL type system. For example,
`var x = 1;` creates a global variable in script mode and the program REPL.
Expression modes accept variable reads and assignment, but no declarations.

* `{}` delimits lexical blocks, as in `{ print x; }`.
* `[]` is reserved for arrays/indexing, as in `items[0]`.
* `;` terminates statements, as in `print 42;`.
* `()` groups expressions such as `(1 + 2)`; calls such as `inspect(host)`
  remain planned.
* `if`, `else`, `while`, and `for` execute as described above. Reserved words
  such as `fun`, `return`, `and`, and `or` are recognized; their execution
  semantics remain deferred.

Explicit punctuation avoids indentation-sensitive scanning. Whole-name keyword
lookup keeps identifiers such as `variable` usable. Raw strings and line comments
keep the scanner small enough to explain and test. Finite numeric values
and escaped token records are now implemented.

The Lab 1 scope is the scanner, diagnostics, tests, and documentation. Lab 2 adds
expression parsing, operator precedence, AST printing, and parser diagnostics.
Expression runtime values, evaluation, print statements, and expression
statements, variables, assignment, and nested blocks are implemented. CSL rule evaluation and
RFID/CAD integration remain future work. Recognizing `watch` or `inspect` does not implement monitoring or
inspection.

## File Structure

| File name        | Purpose                                                                   |
| ---------------- | ------------------------------------------------------------------------- |
| `src/token.rs`   | Defines `TokenType`, `Literal`, and `Token`                               |
| `src/scanner.rs` | Reads the raw source code character by character and turns it into tokens |
| `src/environment.rs` | Stores local bindings and enclosing scopes; resolves lookup and assignment |
| `src/value.rs` | Defines runtime values, truthiness, and value printing |
| `src/evaluator.rs` | Evaluates expression trees, executes statements, and reports runtime errors |
| `src/expr.rs` | Defines expression AST nodes and their printed representation |
| `src/parser.rs` | Parses expressions and programs; recovers at statement and block boundaries |
| `src/stmt.rs` | Defines blocks, declarations, print, expression, `if`, and `while` statements |
| `src/repl.rs` | Keeps session bindings across input lines and errors |
| `src/main.rs` | Dispatches script execution, earlier flags, and the REPL |
| `tests/lab2/` | Parser fixtures with inline AST and diagnostic expectations |
| `tests/lab3/` | Evaluator fixtures with inline value and error expectations |
| `tests/lab4/` | Script fixtures with inline output, silence, and error expectations |
| `tests/lab5/` | Control-flow fixtures for conditionals, loops, scope, and errors |
| `tests/cli.rs` | Checks CLI output, diagnostics, exit status, and REPL recovery |
| `tests/cli_tests.rs` | Checks scanner regressions and default REPL compatibility |
