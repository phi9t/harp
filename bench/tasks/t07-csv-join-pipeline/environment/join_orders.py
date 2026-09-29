#!/usr/bin/env python3
import sys


def read_customers(path: str) -> dict[str, str]:
    customers = {}
    with open(path, "r", encoding="utf-8") as handle:
        next(handle)
        for line in handle:
            customer_id, name = line.rstrip("\n").split(",")
            customers[customer_id] = name
    return customers


def read_orders(path: str) -> dict[str, tuple[str, str, str]]:
    orders = {}
    with open(path, "r", encoding="utf-8") as handle:
        next(handle)
        for line in handle:
            order_id, customer_id, total_cents, notes = line.rstrip("\n").split(",")
            orders[customer_id] = (order_id, total_cents, notes)
    return orders


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print("usage: join_orders.py CUSTOMERS ORDERS", file=sys.stderr)
        return 2
    customers = read_customers(argv[1])
    orders = read_orders(argv[2])
    print("order_id,customer_id,name,total_cents,notes")
    for customer_id in sorted(orders):
        order_id, total_cents, notes = orders[customer_id]
        print(f"{order_id},{customer_id},{customers[customer_id]},{total_cents},{notes}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
