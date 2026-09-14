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

### CSL keywords

Planned keywords for Cyber Sequence Language (CSL).

| Lexeme       | Token Type   | Purpose                                       |
| ------------ | ------------ | --------------------------------------------- |
| `watch`      | `WATCH`      | Declares an event-monitoring rule             |
| `ability`    | `ABILITY`    | Declares a named ability                      |
| `target`     | `TARGET`     | Specifies the target of an ability            |
| `source`     | `SOURCE`     | Specifies an event or log source              |
| `sequence`   | `SEQUENCE`   | Groups operation steps                        |
| `require`    | `REQUIRE`    | Declares a required capability                |
| `capability` | `CAPABILITY` | Introduces a capability reference             |
| `count`      | `COUNT`      | Counts matching events                        |
| `by`         | `BY`         | Groups events by a field                      |
| `within`     | `WITHIN`     | Sets a query's time window                    |
| `alert`      | `ALERT`      | Raises an alert                               |
| `inspect`    | `INSPECT`    | Inspects a simulated target                   |
| `isolate`    | `ISOLATE`    | Isolates a simulated target                   |
| `execute`    | `EXECUTE`    | Requests execution of the declared operations |

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
