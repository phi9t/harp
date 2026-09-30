Implement expression tokenization in `src/tokenizer.cpp`.

Required contract:

- `tokenize(std::string_view)` returns `Result<std::vector<Token>>`.
- Skip whitespace and stop at `#` comments.
- Recognize identifiers, decimal numbers, quoted strings, parentheses,
  arithmetic operators, comparison operators, and logical keywords
  `and`, `or`, `not`.
- Quoted string token text excludes the quotes and preserves commas.
- Every token offset is the zero-based byte offset where that token starts.
- For invalid input, return `E_LEX` diagnostics with a useful offset; do not
  throw for user input.

Do not edit parser, rule store, loader, driver, tests, or the facade manifest.
