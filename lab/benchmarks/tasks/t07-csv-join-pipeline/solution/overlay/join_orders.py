#!/usr/bin/env python3
import csv
import sys


def read_customers(path: str) -> dict[str, str]:
    with open(path, "r", encoding="utf-8", newline="") as handle:
        return {row["customer_id"]: row["name"] for row in csv.DictReader(handle)}


def iter_joined_rows(customers_path: str, orders_path: str):
    customers = read_customers(customers_path)
    with open(orders_path, "r", encoding="utf-8", newline="") as handle:
        for order in csv.DictReader(handle):
            yield [
                order["order_id"],
                order["customer_id"],
                customers[order["customer_id"]],
                order["total_cents"],
                order["notes"],
            ]


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print("usage: join_orders.py CUSTOMERS ORDERS", file=sys.stderr)
        return 2
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(["order_id", "customer_id", "name", "total_cents", "notes"])
    writer.writerows(iter_joined_rows(argv[1], argv[2]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
