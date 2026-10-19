from .models import format_cents


def inventory_report(ledger, price_book=None):
    rows = ["sku,warehouse,on_hand,unit_price,total_value"]
    for item in sorted(ledger.snapshot(), key=lambda row: (row["sku"], row["warehouse"])):
        unit_price = price_book.unit_price(item["sku"]) if price_book is not None else item["unit_cost_cents"]
        total = unit_price * item["on_hand"]
        rows.append(
            ",".join(
                [
                    item["sku"],
                    item["warehouse"],
                    str(item["on_hand"]),
                    format_cents(unit_price),
                    format_cents(total),
                ]
            )
        )
    return "\n".join(rows)
