import csv
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import inventory
from inventory import (
    Allocation,
    PriceBook,
    ReservationBook,
    StockLedger,
    allocate_order,
    inventory_report,
)
from inventory.errors import InventoryError
from inventory.models import OrderLine, parse_cents


def rows(path, header, data):
    with path.open("w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(header)
        writer.writerows(data)


class InventoryIntegrationTests(unittest.TestCase):
    def assert_inventory_error(self, code, func, *args, **kwargs):
        with self.assertRaises(InventoryError) as raised:
            func(*args, **kwargs)
        self.assertEqual(raised.exception.code, code)
        self.assertTrue(str(raised.exception).startswith(f"{code}: "))

    def test_ledger_and_reservations_are_atomic(self):
        ledger = StockLedger()
        ledger.receive("SKU-A", "W2", 3, unit_cost_cents=500, received_at="2026-01-02")
        ledger.receive("SKU-A", "W1", 5, unit_cost_cents=500, received_at="2026-01-01")
        ledger.receive("SKU-B", "W1", 2, unit_cost_cents=125, received_at="2026-01-03")

        self.assertEqual(
            ledger.snapshot(),
            [
                {"sku": "SKU-A", "warehouse": "W1", "on_hand": 5, "unit_cost_cents": 500},
                {"sku": "SKU-A", "warehouse": "W2", "on_hand": 3, "unit_cost_cents": 500},
                {"sku": "SKU-B", "warehouse": "W1", "on_hand": 2, "unit_cost_cents": 125},
            ],
        )
        self.assert_inventory_error(
            "NEGATIVE_STOCK",
            ledger.apply_adjustment,
            "SKU-B",
            "W1",
            -3,
            "bad correction",
        )

        book = ReservationBook()
        self.assert_inventory_error(
            "OUT_OF_STOCK",
            book.reserve_many,
            "ORDER-1",
            [
                {"sku": "SKU-A", "warehouse": "W1", "quantity": 4},
                {"sku": "SKU-A", "warehouse": "W2", "quantity": 99},
            ],
            ledger,
        )
        self.assertEqual(book.active_for_order("ORDER-1"), [])
        self.assertEqual(book.reserved_quantity("SKU-A"), 0)

        made = book.reserve_many(
            "ORDER-2",
            [
                {"sku": "SKU-A", "warehouse": "W1", "quantity": 4},
                {"sku": "SKU-A", "warehouse": "W2", "quantity": 1},
            ],
            ledger,
        )
        self.assertEqual([r.reservation_id for r in made], ["ORDER-2:001", "ORDER-2:002"])
        self.assertEqual(book.reserved_quantity("SKU-A"), 5)
        self.assertEqual(ledger.on_hand("SKU-A"), 8)
        book.commit("ORDER-2:001", ledger)
        self.assertEqual(ledger.on_hand("SKU-A", "W1"), 1)
        self.assertEqual(book.reserved_quantity("SKU-A", "W1"), 0)

    def test_allocator_tie_breaks_and_rolls_back_failed_orders(self):
        ledger = StockLedger()
        ledger.receive("MUG", "W2", 3, received_at="2026-01-02")
        ledger.receive("MUG", "W1", 3, received_at="2026-01-01")
        ledger.receive("MUG", "W3", 5, received_at="2026-01-03")
        ledger.receive("PAD", "W3", 1, received_at="2026-01-04")
        reservations = ReservationBook()

        plan = allocate_order(
            "ORDER-ALLOC",
            [OrderLine("MUG", 5)],
            ledger,
            reservations,
            warehouse_priority={"W2": 1, "W1": 1, "W3": 5},
        )
        self.assertEqual(
            [(a.sku, a.warehouse, a.quantity, a.reservation_id) for a in plan],
            [("MUG", "W1", 3, "ORDER-ALLOC:001"), ("MUG", "W2", 2, "ORDER-ALLOC:002")],
        )
        self.assertEqual(reservations.reserved_quantity("MUG"), 5)

        self.assert_inventory_error(
            "OUT_OF_STOCK",
            allocate_order,
            "ORDER-BAD",
            [OrderLine("MUG", 4), OrderLine("PAD", 2)],
            ledger,
            reservations,
            {"W1": 0, "W2": 0, "W3": 0},
        )
        self.assertEqual(
            [(r.order_id, r.sku, r.quantity) for r in reservations.active_for_order("ORDER-BAD")],
            [],
        )
        self.assertEqual(reservations.reserved_quantity("MUG"), 5)

    def test_pricing_uses_cents_and_canonical_rule_order(self):
        self.assertIn("PriceBook", inventory.__all__)
        prices = PriceBook()
        prices.set_price("KIT-A", parse_cents("19.99"), category="kit")
        prices.set_price("PART-B", parse_cents("3.33"), category="part")
        prices.add_order_discount("BULK5", threshold_cents=4000, percent=5)
        prices.add_category_discount("KIT10", category="kit", percent=10, min_quantity=2)

        quote = prices.quote([OrderLine("PART-B", 1), OrderLine("KIT-A", 2)])

        self.assertEqual(quote["subtotal_cents"], 4331)
        self.assertEqual(
            quote["discounts"],
            [
                {"code": "BULK5", "amount_cents": 216},
                {"code": "KIT10", "amount_cents": 399},
            ],
        )
        self.assertEqual(quote["total_cents"], 3716)
        self.assert_inventory_error(
            "UNKNOWN_PRICE",
            prices.quote,
            [OrderLine("MISSING", 1)],
        )

    def test_reports_are_sorted_and_format_money_at_boundaries(self):
        self.assertIn("inventory_report", inventory.__all__)
        ledger = StockLedger()
        ledger.receive("BB", "W2", 3, unit_cost_cents=parse_cents("2.00"), received_at="2026-01-02")
        ledger.receive("AA", "W1", 2, unit_cost_cents=parse_cents("1.50"), received_at="2026-01-01")
        ledger.receive("AA", "W3", 1, unit_cost_cents=parse_cents("1.25"), received_at="2026-01-03")

        prices = PriceBook()
        prices.set_price("BB", parse_cents("2.00"), category="part")
        prices.set_price("AA", parse_cents("1.50"), category="kit")

        self.assertEqual(
            inventory_report(ledger, prices).splitlines(),
            [
                "sku,warehouse,on_hand,unit_price,total_value",
                "AA,W1,2,1.50,3.00",
                "AA,W3,1,1.50,1.50",
                "BB,W2,3,2.00,6.00",
            ],
        )

    def test_cli_quote_and_allocate_end_to_end(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            stock = root / "stock.csv"
            prices = root / "prices.csv"
            order = root / "order.csv"
            priority = root / "priority.csv"
            rows(
                stock,
                ["sku", "warehouse", "quantity", "unit_cost"],
                [
                    ["KIT-A", "W2", "2", "8.00"],
                    ["KIT-A", "W1", "2", "8.00"],
                    ["PART-B", "W1", "4", "1.25"],
                ],
            )
            rows(
                prices,
                ["sku", "unit_price", "category"],
                [["KIT-A", "19.99", "kit"], ["PART-B", "3.33", "part"]],
            )
            rows(order, ["sku", "quantity"], [["KIT-A", "2"], ["PART-B", "1"]])
            rows(priority, ["warehouse", "priority"], [["W2", "1"], ["W1", "1"]])

            quote = subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "inventory.cli",
                    "quote",
                    "--prices",
                    str(prices),
                    "--order",
                    str(order),
                    "--category-discount",
                    "KIT10:kit:10:2",
                    "--order-discount",
                    "BULK5:4000:5",
                ],
                check=True,
                text=True,
                stdout=subprocess.PIPE,
            )
            self.assertEqual(json.loads(quote.stdout)["total_cents"], 3716)

            allocated = subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "inventory.cli",
                    "allocate",
                    "--stock",
                    str(stock),
                    "--order",
                    str(order),
                    "--warehouse-priority",
                    str(priority),
                    "--order-id",
                    "CLI-1",
                ],
                check=True,
                text=True,
                stdout=subprocess.PIPE,
            )
            self.assertEqual(
                json.loads(allocated.stdout),
                {
                    "allocations": [
                        {"order_id": "CLI-1", "quantity": 2, "reservation_id": "CLI-1:001", "sku": "KIT-A", "warehouse": "W1"},
                        {"order_id": "CLI-1", "quantity": 1, "reservation_id": "CLI-1:002", "sku": "PART-B", "warehouse": "W1"},
                    ]
                },
            )


if __name__ == "__main__":
    unittest.main()
