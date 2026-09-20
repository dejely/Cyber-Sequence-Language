# Design and Implementation of Programming Languages(CMSC124): Interpreter

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

* The first character must be an uppercase letter, lowercase letter, or underscore (`_`).
* Subsequent characters may be uppercase letters, lowercase letters, digits (`0-9`), or underscores.
* Identifiers are case-sensitive.
* Reserved keywords cannot be used as identifiers.

**Pattern:**

```text
identifier → [A-Za-z_][A-Za-z0-9_]*
```

**Examples:**

| Lexeme       | Valid? | Token Type                                              |
| ------------ | ------ | ------------------------------------------------------- |
| `total`      | Yes    | `IDENTIFIER`                                            |
| `user_name`  | Yes    | `IDENTIFIER`                                            |
| `_count`     | Yes    | `IDENTIFIER`                                            |
| `event123`   | Yes    | `IDENTIFIER`                                            |
| `MyVariable` | Yes    | `IDENTIFIER`                                            |
| `123event`   | No     | `NUMBER` followed by an invalid identifier              |
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

### CSL features

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

* `{}` creates blocks.
* `[]` is reserved for arrays/indexing.
* `;` ends statements.
* `()` groups expressions and calls functions.
* Identifiers start with a letter or underscore and may contain letters, digits, and underscores.
* Identifiers are case-sensitive.
* Reserved keywords cannot be used as identifiers.
* Planned keywords include `if`, `else`, `for`, `fun`, `and`, and `var`.

## File Structure

| File name        | Purpose                                                                   |
| ---------------- | ------------------------------------------------------------------------- |
| `src/token.rs`   | Defines `TokenType`, `Literal`, and `Token`                               |
| `src/scanner.rs` | Reads the raw source code character by character and turns it into tokens |
