# Lab 4 — Statements and State

Course reference: [CMSC 124 Lab 4](https://renscourses.netlify.app/articles/cmsc-124-lab4),
published October 22, 2026; checked November 7, 2026.

The repository follows the imperative route with mutable bindings.

- Week 1: execute print and expression statements from files, retain earlier
  flags, and add annotated script tests.
- Week 2: create and read variables, assign values, retain REPL bindings across
  inputs and errors, and test the binding rules.
- Week 3: add nested scopes, shadowing, and scope-aware error recovery, then
  verify all five suites in CI.

Weeks 1–3 are implemented. Declarations without initializers store `nil`,
same-scope redeclarations replace the previous value, and assignment requires an
existing name. Blocks create nested lexical scopes; reads and assignments select
the nearest enclosing binding. Leaving a block, including through a runtime
error, restores its parent environment. The README records grammar, recovery,
binding time, storage lifetime, mutability, output, and undefined-name decisions.

`examples/week2.csl` demonstrates variables and assignment;
`examples/week3.csl` tracks stock and revenue with a nested discount scope.
The Lab 4 harness manifest discovers all `.csl` fixtures recursively, including
the Week 3 scope and error cases. REPL integration tests cover scope restoration
after runtime errors and syntax recovery across block boundaries.

Week 3 progress reports are scheduled for November 9–10, 2026. Use the nested
example to trace lookup, shadowing, outward assignment, and local storage lifetime.
The final commit's CI result is recorded in `docs/TRACKER.md`. Individual
reflection and peer assessment follow the Week 4 defense.

Verify locally with `./build.sh`, `cargo test`, and
`python3 run_tests.py tests/lab4`. Run Labs 0–3 as regression checks.
