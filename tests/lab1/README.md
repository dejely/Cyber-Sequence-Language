# Scanner regression fixtures

The official v1.1 harness uses `manifest.json` to discover `.csl` sources and
compare stdout with `.expected` sidecars. Rejections have `.exit` containing 65.
The harness does not assert stderr; `tests/cli_tests.rs` verifies diagnostics,
stream separation, exit codes, REPL behavior, and repeated-run output.

| Fixtures | Evidence |
| --- | --- |
| `categories`, `keyword` | All token categories, punctuation, keywords, numbers and empty/ordinary strings |
| `identifier_boundaries` | Complete keyword matching, prefixes, case and ASCII names |
| `operator_boundaries` | Adjacent longest-match operators and a single slash at EOF |
| `number_boundaries` | `3.toString`, `.5`, `3.`, `123event`, separate minus and decimal at EOF |
| `comments`, `comment_eof` | Comments discarded, including EOF without a newline |
| `comment_text_in_strings` | Comment-like text inside strings is retained |
| `multiline`, `crlf_lines` | Line counts across LF and CRLF, including comments |
| `empty`, `whitespace_only` | Only EOF emitted; whitespace advances line numbers |
| `string_multiline`, `string_raw_backslash`, `string_control_characters` | Actual multiline strings, display escapes, empty/Unicode strings and following-token positions |
| `invalid`, `unterminated` | Invalid-character and unclosed-string errors |
| `numeric_overflow`, `numeric_overflow_recovery` | Third rejection case and continued scanning |
| `mixed_errors` | Error order and opening-line reporting for an unterminated multiline string |

New expectations are derived from the documented lexical rules, not copied from
scanner output. Boundary fixtures deliberately omit final newlines.
`crlf_lines.csl` contains actual CRLF bytes; `string_control_characters.csl`
contains actual tab/CRLF bytes inside quotes. Preserve those bytes when editing
or introducing line-ending normalization.

After building, run from the repository root:

```bash
cargo test --locked
python3 run_tests.py tests/lab0
python3 run_tests.py tests/lab1
```

Fetch the pinned v1.1 harness separately if absent; do not commit the runner.
