from .errors import fail


def allocate_order(order_id, lines, ledger, reservations, warehouse_priority=None):
    """Reserve stock for an order and return deterministic Allocation objects.

    Ticket t4 implements the allocator. The final algorithm sorts order lines by
    SKU, then candidate warehouses by priority and warehouse id, and rolls back
    any reservations if a later line cannot be fulfilled.
    """

    fail("NOT_IMPLEMENTED", "allocate_order is not implemented")
