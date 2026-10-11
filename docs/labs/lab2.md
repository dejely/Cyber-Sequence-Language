# Lab 2 — Expression parser

This is a repository summary of the
[CMSC 124 Lab 2 brief](https://renscourses.netlify.app/articles/cmsc-124-lab2),
checked against the course's `Content/Materials/cmsc-124-lab2.md` on
2026-10-11. The course material is the source of the laboratory requirements;
this page records their implementation scope.

The lab asks for an unambiguous context-free grammar in the repository README
and a recursive descent parser that consumes scanner tokens and builds an
expression AST. The grammar includes equality, comparisons, addition and
subtraction, multiplication and division, unary negation and logical negation,
number and string literals, booleans, nil, and parenthesized expressions.
Binary operators associate to the left.

The AST printer uses parenthesized prefix notation. For example, `1 + 2 * 3`
prints `(+ 1.0 (* 2.0 3.0))`, and `(1 + 2)` prints
`(group (+ 1.0 2.0))`. Ordinary integral numbers print with `.0`; strings
print without quotes, and booleans and nil print as `true`, `false`, or `nil`.

The scanner, parser, and printer must work together in the REPL. Malformed
expressions should produce useful diagnostics, including missing parentheses
and missing operands, and the session should continue after errors. Students
provide tests demonstrating precedence, left associativity, literals,
grouping, and malformed input.

The project's Week 2 implementation covers AST definitions, parser helpers,
all precedence levels, literals and grouping, AST printing, the file command
and REPL, and fixtures. Week 3 adds recovery, stderr diagnostics with static
error exit 65, coverage for Labs 0 through 2, and documentation of grammar
choices and limitations.

Semicolon-separated expression files and recovery at semicolons are project
choices added for file testing. They do not introduce statement parsing or
evaluation. The [README grammar](../../README.md#grammar) records these choices,
including optional final semicolons, rejection of trailing tokens, and
empty-input behavior.
