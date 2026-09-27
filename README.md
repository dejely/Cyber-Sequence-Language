# Design and Implementation of Programming Languages(CMSC124): Interpreter

This repository currently implements scanning only. Token purposes below describe
intended language semantics; declarations, expressions, and CSL operations are
not parsed or executed yet.

## Lexical Grammar

### Single-character tokens

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

### One- or two-character tokens

| Lexeme | Token Type      | Purpose                          |
| ------ | --------------- | -------------------------------- |
| `!`    | `BANG`          | Logical negation                 |
| `!=`   | `BANG_EQUAL`    | Inequality comparison            |
| `=`    | `EQUAL`         | Assignment                       |
| `==`   | `EQUAL_EQUAL`   | Equality comparison              |
| `>`    | `GREATER`       | Greater-than comparison          |
| `>=`   | `GREATER_EQUAL` | Greater-than-or-equal comparison |
| `<`    | `LESS`          | Less-than comparison             |
| `<=`   | `LESS_EQUAL`    | Less-than-or-equal comparison    |

### Literal tokens

| Lexeme pattern                        | Token Type   | Purpose                                        |
| ------------------------------------- | ------------ | ---------------------------------------------- |
| Name such as `total`                  | `IDENTIFIER` | Names a variable, function, class, or property |
| Text such as `"hello"`                | `STRING`     | Represents text data                           |
| Numeric value such as `123` or `3.14` | `NUMBER`     | Represents a numeric value                     |

### Identifier Grammar

Identifiers are used to name variables, functions, classes, properties, and other user-defined entities.

**Rules:**

* The first character must be an ASCII uppercase letter, ASCII lowercase letter, or underscore (`_`).
* Subsequent characters may be ASCII uppercase letters, ASCII lowercase letters, digits (`0-9`), or underscores.
* Identifiers are case-sensitive.
* Reserved keywords cannot be used as identifiers.

**Pattern:**

```text
identifier → [A-Za-z_][A-Za-z0-9_]*
```

**Examples:**

| Lexeme       | Single identifier? | Token Type                                              |
| ------------ | ------ | ------------------------------------------------------- |
| `total`      | Yes    | `IDENTIFIER`                                            |
| `user_name`  | Yes    | `IDENTIFIER`                                            |
| `_count`     | Yes    | `IDENTIFIER`                                            |
| `event123`   | Yes    | `IDENTIFIER`                                            |
| `MyVariable` | Yes    | `IDENTIFIER`                                            |
| `123event`   | No     | `NUMBER(123)` followed by `IDENTIFIER(event)`; no lexical error              |
| `user-name`  | No     | `IDENTIFIER` followed by `MINUS` and another identifier |

### Keywords

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

## Cyber Sequence Language (CSL)

Cyber Sequence Language (CSL) is the proposed domain language for describing
authorized monitoring and response sequences. A CSL rule can identify an event
source, group or count matching events in a time window, state the capability
it needs, and describe the sequence that should produce an alert or an
inspection result. The interpreter currently tokenizes CSL source; parsing,
validation, and rule execution are still planned work.

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

### CSL keyword tracker

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
| Identifiers | ASCII `[A-Za-z_][A-Za-z0-9_]*`; `_host2` is one identifier; `Host` and `host` are distinct spellings | Implemented |
| Keywords | Match the complete name: `if` is `IF`, but `iffy` and `If` are identifiers | Implemented |
| Numbers | `[0-9]+(\.[0-9]+)?`; `42` and `3.14` produce numeric literals stored as `f64`, with normal floating-point rounding | Implemented |
| Numeric boundaries | `.5` becomes `DOT NUMBER(5)`; `3.` becomes `NUMBER(3) DOT`; `3.toString` becomes `NUMBER(3) DOT IDENTIFIER(toString)` | Implemented |
| Signs and adjacent names | `-2` becomes `MINUS NUMBER(2)`; `123event` becomes `NUMBER(123) IDENTIFIER(event)` | Implemented; these are not scanner errors |
| Numeric range | Require a finite `f64`; a source number consisting of 400 consecutive `9` digits must be rejected | Implemented; rejects with exit 65 |
| Strings | Double quotes delimit text; `"hello"` retains quotes in its lexeme and stores `hello` as its literal; `""` stores an empty string | Implemented |
| Source escapes | No escape processing: `"a\nb"` stores the four characters `a`, backslash, `n`, `b`; a backslash does not protect a following quote | Implemented |
| Multiline strings | Actual newlines inside quotes are allowed; the example below starts on line 1 and places `next` on line 2 | Implemented; string token uses its opening line |
| Comments | `x // note` emits only the identifier before EOF; comments run to newline or EOF; `"//"` is a string | Implemented; block and nested comments are not supported |
| Whitespace | Space, tab, CR, and LF outside strings emit no tokens; `x` followed by LF then `y` puts `y` on line 2 | Implemented; LF increments the line, CR is ignored, so CRLF counts once |
| Operators | Prefer the complete operator: `!=` is `BANG_EQUAL`, while `!` is `BANG` | Implemented |
| Resource references | `@auth_logs` becomes `AT IDENTIFIER(auth_logs)` | Tokenization implemented; resource lookup deferred |
| EOF | Empty input produces only `EOF` on line 1 | Implemented |

Multiline source example (the newline inside the string is an actual newline):

```text
"a
b" next
```

The agreed numeric-range restriction supplies the design-specific third rejection
case requested by the assignment. Choosing finite numbers is our language policy,
not a PDF requirement. It avoids silently converting an oversized source value
into infinity. Tiny values may round to zero under `f64` conversion; exact decimal
arithmetic, exponent syntax, and arbitrary-precision numbers are not supported.

### Token values and printed representation

Current output uses this layout:

```text
Token(type=STRING, lexeme="hello", literal=hello, line=1)
```

The lexeme preserves source spelling, while the literal stores the parsed value.
Nonliteral tokens print `literal=null`; `true`, `false`, and `nil` currently have
keyword token types and no parsed literal. Line numbers are one-based.

**Implemented:** escape backslash as `\\`, LF as `\n`, CR as `\r`, and tab as
`\t` in displayed lexemes and string literal values. Other characters, including
quotes, retain their existing presentation. For example, the multiline string
above prints as a single record:

```text
Token(type=STRING, lexeme="a\nb", literal=a\nb, line=1)
```

A source string containing a literal backslash followed by `n` instead prints:

```text
Token(type=STRING, lexeme="a\\nb", literal=a\\nb, line=1)
```

Text fields escape these four characters at display time, keeping token records
on one physical line for LF/CR-containing strings. Stored lexemes and literal
values are unchanged, and source-language escape processing is still unsupported.
Escaping backslashes distinguishes literal backslash-plus-`n` from an actual LF.
Other characters, including Unicode text and punctuation, pass through unchanged.

### Errors and pending interface changes

An unexpected character such as `#` reports
`line 1: Unexpected character '#'.`; an unclosed `"hello` reports
`line 1: Unterminated string.`. The scanner collects errors in source order,
continues where input remains, and rejects the file with exit 65, diagnostics on
stderr, and no stdout. Clean file scans exit 0. Numeric conversion failure or overflow to a nonfinite `f64` reports
`line 1: Numeric literal out of range.` (using the actual source line). The whole
number is consumed before rejection, so later errors are still reported. A file
with an oversized number followed by `#` reports both errors, emits no tokens,
and exits 65. Finite rounding and underflow to zero remain accepted.

| Interface or environment | Current behavior | Agreed target |
| --- | --- | --- |
| `./run --tokenize tests/lab1/categories.csl` | Prints tokens | Preserve |
| `./run --repl` | Scans each input line; a bad line does not end the session | Preserve as an alias |
| `./run` | Starts the REPL | Implemented in Step 2 |
| `./run tests/lab0/hello.src` | Prints `Hello, JM & Dejel!` | Preserve legacy Lab 0 behavior |
| Rust toolchain | `stable` in `rust-toolchain.toml` | **Step 8:** pin audited version `1.98.0` |

CSL source uses `.csl`; the `.src` Lab 0 fixture remains a compatibility exception.
The default REPL satisfies the PDF's no-argument contract while keeping the
earlier harness invocation working.

### Intended syntax and deferred semantics

CSL is intended to be dynamically typed. Rust's static types describe the scanner
implementation, not an implemented CSL type system. For example, `var x = 1;`
currently produces tokens only; it does not create a variable.

* `{}` will delimit blocks, as in `{ print x; }`.
* `[]` is reserved for arrays/indexing, as in `items[0]`.
* `;` will terminate statements, as in `print x;`.
* `()` will group expressions and calls, as in `(1 + 2)` and `inspect(host)`.
* Reserved words such as `if`, `else`, `for`, `fun`, `and`, and `var` are already
  recognized; their execution semantics remain deferred.

Explicit punctuation avoids indentation-sensitive scanning. Whole-name keyword
lookup keeps identifiers such as `variable` usable. Raw strings and line comments
keep the initial scanner small enough to explain and test; finite numeric values
and readable token records are the next agreed refinements.

The Lab 1 scope is the scanner, diagnostics, tests, and documentation. Parsing,
operator precedence, runtime typing, rule evaluation, and RFID/CAD integration
remain future work. Recognizing `watch` or `inspect` does not implement monitoring
or inspection. The full course-template README reorganization is Step 6.

## File Structure

| File name        | Purpose                                                                   |
| ---------------- | ------------------------------------------------------------------------- |
| `src/token.rs`   | Defines `TokenType`, `Literal`, and `Token`                               |
| `src/scanner.rs` | Reads the raw source code character by character and turns it into tokens |

## Running the scanner REPL

Build with `./build.sh`, then run `./run` (or the equivalent `./run --repl`).
Each `> ` prompt accepts one line and prints its tokens. For example, enter
`var x = 1;` to see a declaration token stream; variables are not evaluated.
Each line is scanned independently, with line numbering starting at 1. A lexical
error is printed on stderr and the prompt returns, so entering `#` followed by
`print x;` still scans the second line. End input with EOF (Ctrl-D on Unix-like
terminals) to exit cleanly. The REPL cannot accumulate multiline strings across
prompts; use `./run --tokenize <file>` for multiline source files.

