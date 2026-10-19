#!/bin/sh
set -eu

cat > inventory/pricing.py <<'PY'
from .errors import fail
from .models import OrderLine, canonical_sku, parse_quantity


class PriceBook:
    def __init__(self):
        self._prices = {}
        self._rules = []

    def set_price(self, sku, unit_price_cents, category="general"):
        sku = canonical_sku(sku)
        try:
            unit_price_cents = int(unit_price_cents)
        except ValueError:
            fail("INVALID_MONEY", f"unit price cents must be an integer: {unit_price_cents!r}")
        if unit_price_cents < 0:
            fail("INVALID_MONEY", "unit price cents must be non-negative")
        category = str(category or "general").strip().lower()
        self._prices[sku] = {"unit_price_cents": unit_price_cents, "category": category}

    def add_category_discount(self, code, category, percent, min_quantity=1):
        self._rules.append(
            {
                "code": self._code(code),
                "kind": "category",
                "category": str(category).strip().lower(),
                "percent": self._percent(percent),
                "min_quantity": parse_quantity(min_quantity),
            }
        )

    def add_order_discount(self, code, threshold_cents, percent):
        try:
            threshold_cents = int(threshold_cents)
        except ValueError:
            fail("INVALID_MONEY", f"threshold cents must be an integer: {threshold_cents!r}")
        if threshold_cents < 0:
            fail("INVALID_MONEY", "threshold cents must be non-negative")
        self._rules.append(
            {
                "code": self._code(code),
                "kind": "order",
                "threshold_cents": threshold_cents,
                "percent": self._percent(percent),
            }
        )

    def unit_price(self, sku):
        sku = canonical_sku(sku)
        if sku not in self._prices:
            fail("UNKNOWN_PRICE", f"no price for {sku}")
        return self._prices[sku]["unit_price_cents"]

    def quote(self, lines):
        normalized = [line if isinstance(line, OrderLine) else OrderLine(line["sku"], line["quantity"]) for line in lines]
        line_rows = []
        category_totals = {}
        subtotal = 0
        for line in sorted(normalized, key=lambda item: item.sku):
            if line.sku not in self._prices:
                fail("UNKNOWN_PRICE", f"no price for {line.sku}")
            price = self._prices[line.sku]["unit_price_cents"]
            category = self._prices[line.sku]["category"]
            extended = price * line.quantity
            subtotal += extended
            bucket = category_totals.setdefault(category, {"quantity": 0, "gross_cents": 0})
            bucket["quantity"] += line.quantity
            bucket["gross_cents"] += extended
            line_rows.append(
                {
                    "sku": line.sku,
                    "quantity": line.quantity,
                    "unit_price_cents": price,
                    "extended_cents": extended,
                    "category": category,
                }
            )

        current_total = subtotal
        discounts = []
        for rule in sorted(self._rules, key=lambda item: item["code"]):
            if rule["kind"] == "order":
                if current_total < rule["threshold_cents"]:
                    continue
                amount = current_total * rule["percent"] // 100
            else:
                bucket = category_totals.get(rule["category"], {"quantity": 0, "gross_cents": 0})
                if bucket["quantity"] < rule["min_quantity"]:
                    continue
                amount = bucket["gross_cents"] * rule["percent"] // 100
            if amount <= 0:
                continue
            discounts.append({"code": rule["code"], "amount_cents": amount})
            current_total -= amount

        return {
            "subtotal_cents": subtotal,
            "discounts": discounts,
            "total_cents": current_total,
            "lines": line_rows,
        }

    def _code(self, code):
        code = str(code).strip().upper()
        if not code:
            fail("INVALID_RULE", "discount code must not be empty")
        return code

    def _percent(self, percent):
        try:
            percent = int(percent)
        except ValueError:
            fail("INVALID_RULE", f"percent must be an integer: {percent!r}")
        if percent < 0 or percent > 100:
            fail("INVALID_RULE", "percent must be between 0 and 100")
        return percent
PY

cat >> inventory/__init__.py <<'PY'

__all__.append("PriceBook")
PY
