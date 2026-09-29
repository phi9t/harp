def total_cents(lines):
    total = 0.0
    for amount, quantity in lines:
        total += float(amount) * quantity
    return int(total * 100)
