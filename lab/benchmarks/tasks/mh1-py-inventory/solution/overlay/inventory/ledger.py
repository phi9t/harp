from .errors import fail
from .models import canonical_sku, canonical_warehouse, parse_quantity


class StockLedger:
    def __init__(self):
        self._balances = {}
        self._entries = []
        self._sequence = 0

    def receive(self, sku, warehouse, quantity, unit_cost_cents=0, received_at=""):
        quantity = parse_quantity(quantity)
        return self.apply_adjustment(
            sku,
            warehouse,
            quantity,
            "receive",
            unit_cost_cents=unit_cost_cents,
            received_at=received_at,
        )

    def apply_adjustment(self, sku, warehouse, quantity_delta, reason, unit_cost_cents=0, received_at=""):
        sku = canonical_sku(sku)
        warehouse = canonical_warehouse(warehouse)
        try:
            delta = int(quantity_delta)
        except ValueError:
            fail("INVALID_QUANTITY", f"quantity delta must be an integer: {quantity_delta!r}")
        if delta == 0:
            fail("INVALID_QUANTITY", "quantity delta must not be zero")
        if not str(reason).strip():
            fail("INVALID_REASON", "reason is required")
        try:
            unit_cost_cents = int(unit_cost_cents)
        except ValueError:
            fail("INVALID_MONEY", f"unit cost cents must be an integer: {unit_cost_cents!r}")
        if unit_cost_cents < 0:
            fail("INVALID_MONEY", "unit cost cents must be non-negative")

        key = (sku, warehouse)
        current = self._balances.get(key, {"quantity": 0, "unit_cost_cents": 0})
        new_quantity = current["quantity"] + delta
        if new_quantity < 0:
            fail("NEGATIVE_STOCK", f"{sku} in {warehouse} would become negative")

        unit_cost = unit_cost_cents if unit_cost_cents or current["unit_cost_cents"] == 0 else current["unit_cost_cents"]
        self._balances[key] = {"quantity": new_quantity, "unit_cost_cents": unit_cost}
        self._sequence += 1
        self._entries.append(
            {
                "seq": self._sequence,
                "sku": sku,
                "warehouse": warehouse,
                "quantity_delta": delta,
                "balance": new_quantity,
                "reason": str(reason),
                "received_at": str(received_at or ""),
                "unit_cost_cents": unit_cost,
            }
        )
        return new_quantity

    def on_hand(self, sku=None, warehouse=None):
        sku = canonical_sku(sku) if sku is not None else None
        warehouse = canonical_warehouse(warehouse) if warehouse is not None else None
        total = 0
        for (entry_sku, entry_warehouse), row in self._balances.items():
            if sku is not None and entry_sku != sku:
                continue
            if warehouse is not None and entry_warehouse != warehouse:
                continue
            total += row["quantity"]
        return total

    def unit_cost(self, sku, warehouse=None):
        sku = canonical_sku(sku)
        if warehouse is not None:
            warehouse = canonical_warehouse(warehouse)
            return self._balances.get((sku, warehouse), {"unit_cost_cents": 0})["unit_cost_cents"]
        matches = [
            (entry_warehouse, row["unit_cost_cents"])
            for (entry_sku, entry_warehouse), row in self._balances.items()
            if entry_sku == sku and row["quantity"] > 0
        ]
        return sorted(matches)[0][1] if matches else 0

    def warehouses_for(self, sku):
        sku = canonical_sku(sku)
        return [
            warehouse
            for (entry_sku, warehouse), row in sorted(self._balances.items())
            if entry_sku == sku and row["quantity"] > 0
        ]

    def snapshot(self):
        rows = []
        for (sku, warehouse), row in sorted(self._balances.items()):
            if row["quantity"] <= 0:
                continue
            rows.append(
                {
                    "sku": sku,
                    "warehouse": warehouse,
                    "on_hand": row["quantity"],
                    "unit_cost_cents": row["unit_cost_cents"],
                }
            )
        return rows

    def history(self, sku=None):
        sku = canonical_sku(sku) if sku is not None else None
        return [
            dict(entry)
            for entry in self._entries
            if sku is None or entry["sku"] == sku
        ]
