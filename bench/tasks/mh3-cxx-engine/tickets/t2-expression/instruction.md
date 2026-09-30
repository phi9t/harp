Implement expression parsing and evaluation in `src/expression.cpp`.

Contract from `t1-tokenizer`: `tokenize` returns tokens with kinds and source
offsets for identifiers, numbers, strings, parentheses, arithmetic,
comparisons, and logical `and`/`or`/`not`.

Required behavior:

- Implement `Diagnostic::format`, `Value` constructors, `normalize_number`,
  `render_value`, and `evaluate_expression`.
- Grammar precedence: unary `not`/`-`, `*` and `/`, `+` and `-`, comparisons,
  `and`, then `or`.
- Identifiers read values from `Context`; `true` and `false` are boolean
  literals.
- Arithmetic accepts numbers only. Logical operators accept booleans only.
- Ordered comparisons accept numbers only; equality supports same-kind values.
- Return `E_PARSE` for syntax errors and `E_EVAL` for unknown identifiers,
  type errors, or division by zero.
- Preserve the repo numeric convention: numeric literals and every arithmetic
  operation are rounded to three decimals, and negative zero renders as
  `0.000`.

Do not edit tokenizer, rules, loader, driver, tests, or the facade manifest.
