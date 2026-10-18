Implement the CLI-style driver in `src/driver.cpp`, and update the facade
manifest in `include/rules_engine.hpp`.

Contracts from earlier tickets:

- `load_rules(text)` returns a populated `RuleStore`.
- `RuleStore::decide(context)` returns a matched or unmatched `RuleDecision`.
- `evaluate_expression(value_text, Context{})` can parse literal context
  values such as strings, numbers, and booleans.

Required behavior:

- `parse_context(text)` parses `key=value` entries separated by semicolons or
  newlines, with separators inside quotes ignored.
- Context keys must start with a letter or `_` and then contain only letters,
  digits, or `_`.
- Context values are expression literals evaluated with an empty context.
- Context parse failures return `E_DRIVER`; do not throw for user input.
- `run_script(rules_text, context_text)` loads rules, parses context, asks the
  store for a decision, returns the matched action, or returns `NO_MATCH` when
  no rule matches.
- Add `driver` to `feature_manifest()` in canonical order. Another ticket also
  edits this manifest; if you redo this ticket, preserve existing features.

Do not edit tokenizer, expression, rule store, loader, tests, or README.md.
