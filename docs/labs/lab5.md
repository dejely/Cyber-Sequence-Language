# Lab 5 — Functions

Course reference: CMSC 124 Lab 5 PDF ("Functions"), published November 6, 2026;
deadline December 9, 2026. It covers Chapters 9 and 10 of *Crafting
Interpreters*, "Control Flow" and "Functions".

The repository continues the imperative route from Lab 4: mutable bindings,
lexical block scope, `while` and `for` loops, and (planned) explicit `return`.

- Week 1 (Nov 9–10): conditionals and the repetition route in the grammar,
  with conditional harness tests and a loop test.
- Week 2 (Nov 16–17): working repetition, logical operators, function creation
  and calls, and an attempt at capture.
- Week 3 (Nov 23–24): function results, higher-order use, capture tests,
  native functions, arity checking, green CI across all six folders, and the
  example program under `examples/`.
- Week 4 (Dec 1): laboratory defense. Nov 30 is Bonifacio Day.

Week 1 is implemented. `if`/`else` executes only the selected branch, using the
Lab 3 truthiness rule. `while` re-checks its condition before every pass.
`for` is desugared by the parser into `{ init; while (cond) { body; incr; } }`,
so the evaluator has no `for` case. The README records the grammar, the
dangling-else rule, loop scope, and the current limitations (no `break`).

Trace for the progress report: `for (var i = 0; i < 2; i = i + 1) print i;`
parses to a block holding `var i = 0;` and a `while (i < 2)` whose body is a
block of `print i;` and `i = i + 1;`. The loop variable lives in the outer
block and is gone after the loop (`tests/lab5/for_variable_does_not_leak.csl`).

Week 2 is implemented. `and` and `or` short-circuit and return the operand
that decided the result. `fun` declarations create function values that hold
the scope they were declared in. Calls are a postfix operator on any
expression; the call expression evaluates the callee and arguments, checks
callability and arity once for every `Callable`, and runs the body in a fresh
scope whose parent is the captured scope. Scopes are now shared through
`Rc<RefCell<Environment>>`, so a captured scope outlives its block or call.
Without `return`, functions escape by assignment, and the closure fixtures use
that to show retained and independent state.

Trace for the progress report, `tests/lab5/fun_lexical_scope.csl`: `greet` is
declared in the global scope, so its closure is the global scope. `rude()`
creates scope R (parent: global) and defines `greeting = "Go away"` in R.
Calling `greet()` from R creates scope G whose parent is `greet`'s closure,
the global scope, not R. Looking up `greeting` from G finds nothing in G and
then reaches the global `"Hello"`; R is never searched.

Verify locally with `./build.sh`, `cargo test`, and
`python3 run_tests.py tests/lab5`. Run Labs 0–4 as regression checks.
