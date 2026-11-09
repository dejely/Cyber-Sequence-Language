# CSL declaration annotations

Issue #53 adds declaration metadata to the existing scanner/parser/evaluator
pipeline. The scanner already recognized `@` as `AT`; annotation names use the
existing identifier tokens and argument strings use existing raw string literals.
No new reserved words or dependencies are needed.

## Grammar and validation

```text
declaration → annotation* varDecl | statement
annotation  → "@" IDENTIFIER ( "(" STRING? ")" )?
varDecl     → "var" IDENTIFIER ( "=" expression )? ";"
```

The annotation grammar describes structure; the parser also validates names and
argument counts:

| Name | Arguments | Meaning |
| --- | --- | --- |
| `deprecated` | Zero or one string | Discouraged declaration, with an optional explanatory message. |
| `doc` | Exactly one string | Documentation describing the declaration. |

`@deprecated`, `@deprecated()`, and `@deprecated("Use stock")` are valid.
`@doc("Stock available for sale")` is valid. Empty strings are accepted.
Names are case-sensitive. Each name may occur once per declaration, in either
order. Comments and whitespace, including newlines, may separate annotations
and the declaration; comments containing `@` are still ordinary comments.

Only `var` declarations support metadata, whether global, block-local, or a
`for` initializer. Branches and loop bodies still require a block around a
variable declaration. Annotations cannot follow `var` or attach to a statement,
assignment, expression, block, parameter, or function. In the REPL, the complete
annotated declaration must fit on one input line, like other declarations.

## Representation and execution

`Annotation` in `src/stmt.rs` stores the name token (including its source line)
and literal arguments. Every `Stmt::Var` owns its annotation list in source
order; an unannotated declaration has an empty list. Parsing keeps metadata
attached to that declaration even inside blocks and desugared `for` loops.
There is no pending metadata shared between declarations or REPL inputs.

The parser recognizes `@` as a declaration/recovery starter. Validation errors
use the existing parser diagnostics and prevent execution of the entire file
(exit 65, stderr diagnostics, empty stdout). REPL errors leave previous bindings
intact and the next input starts a fresh parse. Dangling annotations are errors.

The evaluator deliberately ignores validated metadata. Initializers execute
once as usual; deprecated declarations can still be read and assigned without
warnings. Metadata is not evaluated, stored in the environment, propagated to
values, or exposed through a runtime API. Runtime failures retain exit 70.

## Examples and unsupported forms

Run `./run examples/annotations.csl` to see global and local declarations,
multiple annotations, and messages. Output is `banana`, `mango`, and `5`.

These forms are rejected:

```csl
@entry var main;                    // unknown name
@doc var fruit;                     // missing argument
@doc(42) var fruit;                 // string required
@doc("a" + "b") var fruit;         // expressions are unsupported
@doc("a", "b") var fruit;          // at most one argument
@deprecated @deprecated var fruit;  // duplicate name
@deprecated print "old";           // unsupported target
@deprecated                         // missing declaration
```

Runnable rejection fixtures are in `tests/annotations/`; they include harness
expectation comments and are expected to fail with exit 65. The suite also
contains successful execution and runtime-error fixtures. `// expect:` and
`// expect error:` remain harness comments, not language metadata.

## Extension boundaries

To add a built-in annotation, define its name, arguments, allowed targets, and
semantics, extend parser validation, and add attachment and execution tests.
The literal-list representation allows future argument forms without changing
how declarations own metadata. New runtime effects require an explicit design.

Function declarations/calls, function annotations, `@entry` execution, custom
annotation definitions, named or expression arguments, reflection, and warnings
are not implemented. The examples in the issue using `fn` do not define current
CSL syntax: this checkout reserves `fun` for future functions and treats `fn`
as an identifier. Variable lifetime and GLOBAL/LOCAL explanatory labels remain
separate documentation concerns and do not become annotations automatically.

## Verification

```sh
./build.sh
cargo test
python3 run_tests.py tests/annotations
```

Run each existing lab suite (`tests/lab0` through `tests/lab5`) separately as
regression checks. CI runs all seven harness directories and the Rust tests.
