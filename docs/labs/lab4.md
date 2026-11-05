# Lab 4 — Statements and State

Course reference: [CMSC 124 Lab 4](https://renscourses.netlify.app/articles/cmsc-124-lab4),
published October 22, 2026; checked November 6, 2026.

The repository follows the imperative route with mutable bindings.

- Week 1: execute print and expression statements from files, retain earlier
  flags, and add annotated script tests.
- Week 2: create and read variables, assign values, retain REPL bindings across
  inputs and errors, and test the binding rules.
- Week 3: add nested scopes, shadowing, and scope-aware error recovery, then
  verify all five suites in CI.

Weeks 1 and 2 are implemented. Week 3 remains pending. Week 2 uses a single
global environment; declarations without initializers store `nil`, redeclarations
replace the previous value, and assignment requires an existing name. The README
records the grammar and runtime decisions. `examples/week2.csl` demonstrates
variables and assignment; nested-scope examples belong to Week 3.

Verify locally with `./build.sh`, `cargo test`, and
`python3 run_tests.py tests/lab4`. Run Labs 0–3 as regression checks.
