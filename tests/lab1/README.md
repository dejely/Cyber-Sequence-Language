# Scanner regression fixtures

The official v1.1 harness uses `manifest.json` to discover `.csl` sources and
compare stdout with `.expected` sidecars. Rejections have `.exit` containing 65.
The harness does not assert stderr; `tests/cli_tests.rs` verifies diagnostics,
stream separation, exit codes, REPL behavior, and repeated-run output.

## Category layout

```text
tests/lab1/
├── comments/
├── coverage/       # Combined token-vocabulary fixture
├── errors/         # Invalid characters and mixed diagnostics
├── identifiers/
├── keywords/
├── numbers/        # Includes numeric overflow rejections
├── operators/
├── strings/        # Includes unterminated-string rejection
├── whitespace/     # LF, CRLF, and whitespace-only inputs
├── empty.csl
├── empty.expected
├── manifest.json
└── README.md
```

Each source stays beside its `.expected` and optional `.exit` sidecars. The
single root manifest applies to all subfolders: the official harness discovers
sources recursively, as does the CLI repeatability test. Commands remain the same.

| Fixtures (paths relative to this folder) | Evidence |
| --- | --- |
| `coverage/categories`, `keywords/keyword` | Token vocabulary and ordinary values |
| `identifiers/identifier_boundaries` | Whole-name keywords, prefixes, case and ASCII names |
| `operators/operator_boundaries` | Longest-match adjacent operators and slash at EOF |
| `numbers/number_boundaries` | Dot, name adjacency, sign and decimal boundaries |
| `numbers/numeric_overflow`, `numbers/numeric_overflow_recovery` | Third rejection case and continued scanning |
| `comments/comments`, `comments/comment_eof` | Comments discarded, including EOF without newline |
| `strings/comment_text_in_strings` | Comment-like text remains string content |
| `strings/string_multiline`, `strings/string_raw_backslash`, `strings/string_control_characters` | Multiline values, display escapes, empty/Unicode strings and positions |
| `strings/unterminated` | Unclosed-string rejection |
| `whitespace/multiline`, `whitespace/crlf_lines` | LF and CRLF line counts |
| `empty`, `whitespace/whitespace_only` | Only EOF emitted; whitespace advances lines |
| `errors/invalid`, `errors/mixed_errors` | Invalid characters, error order and unclosed-string opening line |

New expectations are derived from the documented lexical rules, not copied from
scanner output. Boundary fixtures deliberately omit final newlines.
`whitespace/crlf_lines.csl` contains actual CRLF bytes; `strings/string_control_characters.csl`
contains actual tab/CRLF bytes inside quotes. Preserve those bytes when editing
or introducing line-ending normalization.

After building, run from the repository root:

```bash
cargo test --locked
python3 run_tests.py tests/lab0
python3 run_tests.py tests/lab1
```

Fetch the pinned v1.1 harness separately if absent; do not commit the runner.
