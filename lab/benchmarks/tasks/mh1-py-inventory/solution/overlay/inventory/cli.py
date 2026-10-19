import argparse
import csv
import json
import sys

from .allocator import allocate_order
from .errors import InventoryError
from .ledger import StockLedger
from .models import OrderLine, parse_cents, parse_quantity
from .pricing import PriceBook
from .reservations import ReservationBook


def read_order(path):
    with open(path, newline="") as f:
        return [OrderLine(row["sku"], row["quantity"]) for row in csv.DictReader(f)]


def read_prices(path):
    prices = PriceBook()
    with open(path, newline="") as f:
        for row in csv.DictReader(f):
            prices.set_price(row["sku"], parse_cents(row["unit_price"]), row.get("category") or "general")
    return prices


def read_stock(path):
    ledger = StockLedger()
    with open(path, newline="") as f:
        for row in csv.DictReader(f):
            ledger.receive(
                row["sku"],
                row["warehouse"],
                parse_quantity(row["quantity"]),
                unit_cost_cents=parse_cents(row.get("unit_cost", "0")),
            )
    return ledger


def read_priority(path):
    with open(path, newline="") as f:
        return {row["warehouse"].strip().upper(): int(row["priority"]) for row in csv.DictReader(f)}


def add_category_spec(price_book, spec):
    code, category, percent, min_quantity = spec.split(":", 3)
    price_book.add_category_discount(code, category, int(percent), int(min_quantity))


def add_order_spec(price_book, spec):
    code, threshold_cents, percent = spec.split(":", 2)
    price_book.add_order_discount(code, int(threshold_cents), int(percent))


def cmd_quote(args):
    prices = read_prices(args.prices)
    for spec in args.category_discount:
        add_category_spec(prices, spec)
    for spec in args.order_discount:
        add_order_spec(prices, spec)
    print(json.dumps(prices.quote(read_order(args.order)), sort_keys=True))
    return 0


def cmd_allocate(args):
    ledger = read_stock(args.stock)
    reservations = ReservationBook()
    allocations = allocate_order(
        args.order_id,
        read_order(args.order),
        ledger,
        reservations,
        read_priority(args.warehouse_priority),
    )
    print(json.dumps({"allocations": [allocation.as_dict() for allocation in allocations]}, sort_keys=True))
    return 0


def build_parser():
    parser = argparse.ArgumentParser(prog="inventory.cli")
    sub = parser.add_subparsers(dest="command", required=True)

    quote = sub.add_parser("quote")
    quote.add_argument("--prices", required=True)
    quote.add_argument("--order", required=True)
    quote.add_argument("--category-discount", action="append", default=[])
    quote.add_argument("--order-discount", action="append", default=[])
    quote.set_defaults(func=cmd_quote)

    allocate = sub.add_parser("allocate")
    allocate.add_argument("--stock", required=True)
    allocate.add_argument("--order", required=True)
    allocate.add_argument("--warehouse-priority", required=True)
    allocate.add_argument("--order-id", required=True)
    allocate.set_defaults(func=cmd_allocate)
    return parser


def main(argv=None):
    parser = build_parser()
    try:
        args = parser.parse_args(argv)
        return args.func(args)
    except InventoryError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    except (KeyError, ValueError) as exc:
        print(f"INVALID_INPUT: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
