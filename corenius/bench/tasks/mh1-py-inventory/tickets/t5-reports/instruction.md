Implement `inventory/reports.py`, implement the standard-library CLI in
`inventory/cli.py`, and append the report export to `inventory/__init__.py`.

This ticket depends on these contracts:

- The ledger `snapshot()` returns `sku`, `warehouse`, `on_hand`, and
  `unit_cost_cents` dictionaries in deterministic order.
- `PriceBook.unit_price(sku)` returns integer cents.
- Allocations expose `as_dict()` with order id, SKU, warehouse, quantity, and
  reservation id.

Required behavior:

- `inventory_report(ledger, price_book=None)` returns CSV text with header
  `sku,warehouse,on_hand,unit_price,total_value`.
- Sort rows by SKU then warehouse even if the ledger implementation already
  sorts.
- If `price_book` is provided, value stock at the selling unit price; otherwise
  use the ledger unit cost.
- Format money only at this report boundary as `12.34`; keep calculations in
  integer cents.
- Add a CLI with these commands:
  - `quote --prices PRICES.csv --order ORDER.csv [--category-discount SPEC ...]
    [--order-discount SPEC ...]`
  - `allocate --stock STOCK.csv --order ORDER.csv --warehouse-priority
    PRIORITY.csv --order-id ORDER_ID`
- CLI CSV contracts:
  - Prices: `sku,unit_price,category`; parse `unit_price` to integer cents with
    `parse_cents`.
  - Stock: `sku,warehouse,quantity,unit_cost`; load into `StockLedger.receive`.
  - Order: `sku,quantity`; load into `OrderLine`.
  - Priority: `warehouse,priority`; lower numbers are preferred.
- Discount specs are `CODE:CATEGORY:PERCENT:MIN_QUANTITY` for category rules and
  `CODE:THRESHOLD_CENTS:PERCENT` for order rules.
- Output JSON to stdout with deterministic key ordering. On `InventoryError`,
  print `<CODE>: <message>` to stderr and exit with status 1. Return status 2
  for argument parsing failures.

At the end of `inventory/__init__.py`, append a line adding `inventory_report`
to `__all__`. Ticket `t3-pricing` appends to the same file; preserve both lines
if there is a conflict.

Do not edit ledger, reservation, pricing, allocator, or test files.
