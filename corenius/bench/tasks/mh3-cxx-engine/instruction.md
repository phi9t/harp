Build the small C++17 rules-engine library to pass the verifier. The library is
split into five tickets:

- `t1-tokenizer`: tokenizes rule expressions and reports lexical diagnostics.
- `t2-expression`: parses and evaluates expressions using the tokenizer.
- `t3-rule-store`: stores rules, orders them by precedence, and chooses a match.
- `t4-loader`: loads rules from the repo's CSV-ish rule format.
- `t5-driver`: provides a CLI-style entry point that loads rules, parses context
  text, and returns the selected action.

Read `README.md` before changing code. Its numeric and diagnostic conventions
are cross-cutting invariants: every module must round numeric results to
thousandths after every arithmetic operation, normalize negative zero, and
return `Result<T>` diagnostics rather than throwing for user input. The
integration tests intentionally combine quoted CSV fields, expression
rounding, rule precedence, and driver error handling.

Two tickets intentionally edit `include/rules_engine.hpp`: `t3-rule-store` adds
the `rule-store` feature to the facade manifest, and `t5-driver` adds the
`driver` feature. Both edits start from the same line, so the second ticket to
land conflicts; rerunning that ticket must preserve both features in canonical
manifest order.
