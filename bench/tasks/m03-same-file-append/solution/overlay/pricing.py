def total(items):
    return sum(price * qty for price, qty in items)


def discount(amount, percent):
    return round(amount * (100 - percent) / 100, 2)


def with_tax(amount, rate):
    return round(amount * (1 + rate), 2)
