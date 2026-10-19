from decimal import Decimal, ROUND_HALF_UP


def total_cents(lines):
    total = Decimal("0")
    for amount, quantity in lines:
        total += Decimal(amount) * quantity
    return int((total * 100).quantize(Decimal("1"), rounding=ROUND_HALF_UP))
