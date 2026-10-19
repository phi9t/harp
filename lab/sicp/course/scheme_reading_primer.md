# Scheme reading primer for systems readers

This is a notation bridge for readers who already understand programming-language semantics. Read each form as a prompt about evaluation, binding, identity, or control—not as syntax to memorize.

## Prefix combinations

```scheme
(* (+ 2 3) (- 10 4))
```

Semantic question: which subexpressions are evaluated, in what environment and order, before the operator is applied?

## `define`

```scheme
(define (square x) (* x x))
```

Semantic question: what binding does this declaration install, and when does a later reference resolve to it?

## `lambda`

```scheme
(define double (lambda (x) (+ x x)))
```

Semantic question: which environment is captured when the procedure object is created?

## Lexical binding

```scheme
(let ((x 3))
  ((lambda (y) (+ x y)) 4))
```

Semantic question: does `x` resolve at the procedure's definition site or at its call site?

## `if` and `cond`

```scheme
(cond ((< x 0) (- x))
      ((= x 0) 0)
      (else x))
```

Semantic question: which predicates and result branches are evaluated, and which remain unevaluated?

## Quotation

```scheme
(car '(a b c))
```

Semantic question: when is a list-shaped expression data rather than a request to apply a procedure?

## Pairs and lists

```scheme
(cons (list 1 2) (list 3 4))
```

Semantic question: which tree structure and sharing relationships are created by nested pairs?

## `set!`

```scheme
(define balance 100)
(set! balance (- balance 25))
```

Semantic question: which existing location changes, and which observations can distinguish the before and after states?

## `set-car!` and `set-cdr!`

```scheme
(define link (cons 'a 'b))
(set-car! link 'x)
(set-cdr! link link)
```

Semantic question: which aliases observe the mutation, and has the pair graph become cyclic?

## Delayed streams

```scheme
(define ones (cons-stream 1 ones))
(stream-ref ones 100)
```

Semantic question: what work is demanded now, what result is memoized, and what structure remains reachable afterward?

## Read versus eval

```scheme
(eval (read input-port) the-global-environment)
```

Semantic question: where does textual input become structured data, and where does that data acquire executable meaning and authority?

## Scheme-to-Rust concept bridge

The following are reading analogies only; **the semantics are not identical**. Rust's ownership, evaluation, mutation, and type rules must be reasoned about on their own terms.

| Scheme concept | Rust concept to compare | Important mismatch |
|---|---|---|
| Lexical closure | Closure capturing locals | Rust distinguishes borrow, mutable borrow, and move capture. |
| Pair/list recursion | `enum` plus `Box`/`Rc` | Allocation, sharing, and representation are explicit in Rust types. |
| Higher-order procedure | Generic or trait-object callable | Static dispatch and `dyn Fn` have different costs and constraints. |
| `set!` binding mutation | `let mut` assignment | Rust mutation is constrained by ownership and borrowing. |
| Shared mutable pair | `Rc<RefCell<T>>` or synchronization type | Runtime borrow checks or synchronization replace unrestricted mutation. |
| Delayed stream | Iterator or memoizing lazy structure | Standard iterators are usually pull-based and not automatically memoized. |
| Symbolic expression | Recursive enum/AST | Rust variants are statically closed unless an extension boundary is designed. |
| `eval`/`apply` | Interpreter dispatch over an AST | Rust has no direct ambient equivalent of evaluating arbitrary source in a lexical environment. |
