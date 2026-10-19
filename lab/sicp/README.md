# SICP evaluator lab

This dependency-free Rust crate is an executable companion to the SICP
metacircular-evaluator deep dive in `content/sicp/`. It models the eval/apply
cycle, lexical closures, special forms, quoted data, primitive procedures, and
SICP-style linear environment lookup. It is intentionally a small semantic
model, not a complete Scheme implementation and not a metacircular evaluator:
the implementation language is Rust rather than the interpreted language.

Run its behavior tests:

```sh
cargo test -p sicp-evaluator-lab
```

Run the built-in lexical-scope example:

```sh
cargo run --quiet -p sicp-evaluator-lab
```

Evaluate one or more expressions in a shared global environment:

```sh
cargo run --quiet -p sicp-evaluator-lab -- \
  '(define x 4)' '((lambda (y) (+ x y)) 3)'
```
