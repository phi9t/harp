cat >> pricing.py <<'PY'


def discount(amount, percent):
    return round(amount * (100 - percent) / 100, 2)
PY
