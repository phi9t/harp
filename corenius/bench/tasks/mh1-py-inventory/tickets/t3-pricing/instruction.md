Implement `inventory/pricing.py` and append the pricing export to
`inventory/__init__.py`.

Required behavior:

- `PriceBook.set_price(sku, unit_price_cents, category="general")` stores an
  integer-cent unit price and category by canonical SKU.
- `add_category_discount(code, category, percent, min_quantity=1)` adds a rule
  that discounts the gross value of matching category lines when their total
  quantity reaches `min_quantity`.
- `add_order_discount(code, threshold_cents, percent)` adds a rule that
  discounts the current order total when it is at or above the threshold.
- `unit_price(sku)` returns an integer-cent price or raises `InventoryError`
  code `UNKNOWN_PRICE`.
- `quote(lines)` accepts `OrderLine` objects, applies rules in canonical
  ascending `code` order regardless of insertion order, floors each discount to
  whole cents, and returns a dictionary with `subtotal_cents`, `discounts`, and
  `total_cents`.

At the end of `inventory/__init__.py`, append a line adding `PriceBook` to
`__all__`. Ticket `t5-reports` appends to the same file; preserve both lines if
there is a conflict.

Follow the top-level money/error/determinism invariants. Do not edit ledger,
reservation, allocator, report, CLI, or test files.
