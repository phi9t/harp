from .errors import InventoryError, fail
from .models import Allocation, OrderLine


def allocate_order(order_id, lines, ledger, reservations, warehouse_priority=None):
    order_id = str(order_id).strip()
    if not order_id:
        fail("INVALID_ORDER", "order id must not be empty")
    priority = {str(k).strip().upper(): int(v) for k, v in (warehouse_priority or {}).items()}
    normalized = [line if isinstance(line, OrderLine) else OrderLine(line["sku"], line["quantity"]) for line in lines]
    created = []
    allocations = []
    try:
        for line in sorted(normalized, key=lambda item: item.sku):
            remaining = line.quantity
            candidates = []
            for warehouse in ledger.warehouses_for(line.sku):
                available = ledger.on_hand(line.sku, warehouse) - reservations.reserved_quantity(line.sku, warehouse)
                if available > 0:
                    candidates.append((priority.get(warehouse, 1_000_000), warehouse, available))
            for _, warehouse, available in sorted(candidates):
                if remaining <= 0:
                    break
                quantity = min(remaining, available)
                reservation = reservations.reserve(order_id, line.sku, quantity, warehouse, ledger)
                created.append(reservation.reservation_id)
                allocations.append(Allocation(order_id, line.sku, warehouse, quantity, reservation.reservation_id))
                remaining -= quantity
            if remaining:
                fail("OUT_OF_STOCK", f"{line.sku} is short by {remaining}")
    except InventoryError:
        for reservation_id in reversed(created):
            reservations.release(reservation_id)
        raise
    return sorted(allocations, key=lambda item: (item.sku, item.warehouse, item.reservation_id))
