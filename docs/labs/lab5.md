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

Verify locally with `./build.sh`, `cargo test`, and
`python3 run_tests.py tests/lab5`. Run Labs 0–4 as regression checks.
