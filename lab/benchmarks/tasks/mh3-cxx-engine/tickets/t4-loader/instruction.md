Implement the CSV-ish rule loader in `src/loader.cpp`.

Contracts from earlier tickets:

- `RuleStore::add_rule` validates rules and returns `Result<bool>`.
- Rule rows are `name,priority,condition,action`.
- Conditions are expression strings evaluated later by the rule store.

Required behavior:

- `load_rules(std::string_view)` returns `Result<RuleStore>`.
- Ignore blank lines and lines whose first non-space character is `#`.
- Accept and skip an optional header exactly naming `name,priority,condition,action`
  case-insensitively.
- Parse quoted CSV fields with doubled quotes (`""`) as escaped quotes, so
  conditions and actions may contain commas.
- Trim unquoted field whitespace.
- Reject rows with the wrong field count or invalid integer priority using
  `E_LOAD` with the input line's byte offset.
- If `RuleStore::add_rule` rejects a row, surface it as an `E_LOAD` diagnostic
  at that row.

Do not edit tokenizer, expression, rules, driver, tests, or the facade manifest.
