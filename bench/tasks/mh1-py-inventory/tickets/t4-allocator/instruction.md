Implement `inventory/allocator.py`.

This ticket depends on contracts from earlier tickets:

- The ledger exposes `warehouses_for(sku)`, `on_hand(sku, warehouse)`, and
  deterministic SKU/warehouse normalization.
- The reservation book exposes `reserve(order_id, sku, quantity, warehouse,
  ledger)`, `release(reservation_id)`, and
  `reserved_quantity(sku, warehouse=None)`.

Required behavior:

- `allocate_order(order_id, lines, ledger, reservations, warehouse_priority=None)`
  returns `Allocation` objects and creates reservations for every allocation.
- Process order lines in sorted SKU order for deterministic output.
- For each SKU, consider warehouses with available stock, sorted by
  `(priority, warehouse)`, where missing priorities sort after provided ones.
- Allocate as much as possible from each warehouse before moving to the next.
- If any line cannot be fully allocated, release every reservation created by
  this call and raise `InventoryError` code `OUT_OF_STOCK`.
- Return allocations sorted by SKU, warehouse, and reservation id.

Follow the top-level invariants. Do not edit ledger, reservation, pricing,
report, CLI, or test files.
