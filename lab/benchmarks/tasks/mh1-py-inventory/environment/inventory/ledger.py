from .errors import fail
from .models import canonical_sku, canonical_warehouse, parse_quantity


class StockLedger:
    """Append-only stock ledger.

    The starter keeps the public shape but not the behavior. Ticket t1 replaces
    this with a real ledger that tracks per-SKU, per-warehouse balances and
    deterministic snapshots.
    """

    def __init__(self):
        self._entries = []

    def receive(self, sku, warehouse, quantity, unit_cost_cents=0, received_at=""):
        parse_quantity(quantity)
        fail("NOT_IMPLEMENTED", "StockLedger.receive is not implemented")

    def apply_adjustment(self, sku, warehouse, quantity_delta, reason, unit_cost_cents=0, received_at=""):
        canonical_sku(sku)
        canonical_warehouse(warehouse)
        if not reason:
            fail("INVALID_REASON", "reason is required")
        fail("NOT_IMPLEMENTED", "StockLedger.apply_adjustment is not implemented")

    def on_hand(self, sku=None, warehouse=None):
        fail("NOT_IMPLEMENTED", "StockLedger.on_hand is not implemented")

    def unit_cost(self, sku, warehouse=None):
        fail("NOT_IMPLEMENTED", "StockLedger.unit_cost is not implemented")

    def warehouses_for(self, sku):
        fail("NOT_IMPLEMENTED", "StockLedger.warehouses_for is not implemented")

    def snapshot(self):
        fail("NOT_IMPLEMENTED", "StockLedger.snapshot is not implemented")

    def history(self, sku=None):
        fail("NOT_IMPLEMENTED", "StockLedger.history is not implemented")
