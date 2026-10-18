# Inventory fulfilment task

This repository is a small Python standard-library inventory and order
fulfilment package. Complete the tickets so the package supports stock ledger
updates, reservations, pricing rules, allocation, reports, and a CLI.

Cross-cutting invariants every ticket must preserve:

- Money is represented as integer cents at module boundaries. Parse decimal
  input once, keep totals as `int`, and format for humans only at report/CLI
  boundaries.
- Public domain failures raise `inventory.errors.InventoryError`. Use stable
  uppercase codes such as `INVALID_QUANTITY`, `OUT_OF_STOCK`,
  `UNKNOWN_PRICE`, and `DUPLICATE_RESERVATION`; `str(error)` must be
  `<CODE>: <message>`.
- All externally visible collections are deterministic. Sort by SKU, then
  warehouse, then reservation/order identifiers unless a ticket specifies a
  stronger ordering rule.
- Do not add third-party dependencies or network access. The verifier runs
  `python3 -m unittest discover -s tests`.

Tickets:

- `t1-ledger`: implement the stock ledger.
- `t2-reservations`: implement atomic reservations on top of the ledger
  contract.
- `t3-pricing`: implement cents-based pricing rules and append pricing exports
  to `inventory/__init__.py`.
- `t4-allocator`: implement deterministic allocation using the ledger and
  reservation contracts.
- `t5-reports`: implement reports and the standard-library CLI, and append
  report exports to `inventory/__init__.py`.

The pricing and report tickets intentionally edit the same export file so the
second one to land has a textual conflict. Resolve by preserving both export
sets.
