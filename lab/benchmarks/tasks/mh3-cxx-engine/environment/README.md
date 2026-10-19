# Rules Engine Conventions

This repo models a small rule engine: expressions read values from a context,
rules use expressions as conditions, the loader reads rule rows, and the driver
ties those pieces together.

Cross-cutting invariants:

- User input failures never throw. Return `Result<T>` with a `Diagnostic`.
- Diagnostic codes are owned by the layer that detects the issue:
  `E_LEX`, `E_PARSE`, `E_EVAL`, `E_RULE`, `E_LOAD`, or `E_DRIVER`.
- Diagnostic formatting is exactly `CODE@OFFSET: message`, with zero-based byte
  offsets in the input owned by the detecting layer.
- Numbers use fixed three-decimal semantics. Normalize every numeric literal and
  every arithmetic result to the nearest thousandth after each operation.
- Negative zero renders as `0.000`.
- Rule precedence is canonical: higher priority first, ties by rule name in
  ascending byte order.
- The facade manifest in `include/rules_engine.hpp` is canonical and must remain
  in this order: `tokenizer`, `expression`, `rule-store`, `driver`.
