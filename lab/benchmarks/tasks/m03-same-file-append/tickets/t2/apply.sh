cat >> pricing.py <<'PY'


def with_tax(amount, rate):
    return round(amount * (1 + rate), 2)
PY
