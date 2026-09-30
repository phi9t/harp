Implement `inventory/reservations.py`.

This ticket depends on the ledger contract from `t1-ledger`:

- `ledger.on_hand(sku, warehouse)` returns integer stock on hand.
- `ledger.apply_adjustment(sku, warehouse, delta, reason)` records shipment
  adjustments and raises `InventoryError` if stock would go negative.

Required behavior:

- `ReservationBook.reserve(order_id, sku, quantity, warehouse, ledger)` creates
  a reservation only if on-hand minus active reservations can satisfy it.
- Reservation ids are deterministic per order: `ORDER:001`, `ORDER:002`, ...
- `reserve_many(order_id, requests, ledger)` is atomic. If any request fails,
  all reservations created by that call are released before re-raising the
  original `InventoryError`.
- `release(reservation_id)` marks an active reservation released and returns it.
- `commit(reservation_id, ledger)` ships the active reservation by applying a
  negative ledger adjustment, then marks the reservation committed.
- `reserved_quantity(sku, warehouse=None)` counts only active reservations.
- `active_for_order(order_id)` returns active reservations sorted by
  reservation id.

Use `InventoryError` codes `OUT_OF_STOCK`, `UNKNOWN_RESERVATION`, and
`DUPLICATE_RESERVATION` where appropriate. Do not edit other modules.
