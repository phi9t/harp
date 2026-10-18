Implement rule storage and precedence in `src/rules.cpp`, and update the
facade manifest in `include/rules_engine.hpp`.

Contract from `t2-expression`: `evaluate_expression(condition, context)`
returns a `Value`; rule conditions must evaluate to booleans.

Required behavior:

- `RuleStore::add_rule` rejects invalid names, duplicate names, and empty
  conditions using `E_RULE`.
- Valid names start with a letter or `_` and then contain only letters, digits,
  `_`, or `-`.
- `ordered_rules()` returns a copy ordered by priority descending, then rule
  name ascending.
- `decide(context)` evaluates rules in canonical order and returns the first
  true rule as a `RuleDecision`; no match is a successful unmatched decision.
- A non-boolean condition is an `E_RULE` diagnostic. Expression diagnostics may
  pass through unchanged.
- Add `rule-store` to `feature_manifest()` in canonical order. Another ticket
  also edits this manifest; if you redo this ticket, preserve existing features.

Do not edit tokenizer, expression, loader, driver, tests, or README.md.
