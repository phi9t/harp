Implement `inventory/ledger.py`.

Required behavior:

- `StockLedger.receive(sku, warehouse, quantity, unit_cost_cents=0, received_at="")`
  records a positive stock receipt and returns the new balance for that
  SKU/warehouse.
- `apply_adjustment(sku, warehouse, quantity_delta, reason, unit_cost_cents=0,
  received_at="")` records positive or negative adjustments. It must reject
  adjustments that would make stock negative with `InventoryError` code
  `NEGATIVE_STOCK`.
- `on_hand(sku=None, warehouse=None)` returns totals filtered by optional SKU
  and warehouse.
- `unit_cost(sku, warehouse=None)` returns the most recent non-negative unit
  cost for that SKU/warehouse, or the first matching warehouse when no warehouse
  is given.
- `warehouses_for(sku)` returns warehouses with positive stock, sorted
  lexicographically.
- `snapshot()` returns dictionaries sorted by SKU then warehouse, each with
  `sku`, `warehouse`, `on_hand`, and `unit_cost_cents`.
- `history(sku=None)` returns append order entries, optionally filtered by SKU.

Follow the top-level invariants: use integer cents, deterministic ordering, and
`InventoryError` for domain failures. Do not edit reservation, pricing,
allocator, report, CLI, or test files.
