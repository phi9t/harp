from dataclasses import dataclass

from .errors import fail


def canonical_sku(value):
    text = str(value).strip().upper()
    if not text:
        fail("INVALID_SKU", "SKU must not be empty")
    return text


def canonical_warehouse(value):
    text = str(value).strip().upper()
    if not text:
        fail("INVALID_WAREHOUSE", "warehouse must not be empty")
    return text


def parse_quantity(value):
    try:
        quantity = int(str(value).strip())
    except ValueError:
        fail("INVALID_QUANTITY", f"quantity must be an integer: {value!r}")
    if quantity <= 0:
        fail("INVALID_QUANTITY", "quantity must be positive")
    return quantity


def parse_cents(value):
    text = str(value).strip()
    if not text:
        fail("INVALID_MONEY", "money value must not be empty")
    if text.startswith("$"):
        text = text[1:]
    sign = -1 if text.startswith("-") else 1
    if text[:1] in "+-":
        text = text[1:]
    whole, dot, frac = text.partition(".")
    if not whole.isdigit() or (dot and (not frac.isdigit() or len(frac) > 2)) or (not dot and frac):
        fail("INVALID_MONEY", f"money must have at most two decimals: {value!r}")
    cents = int(whole) * 100
    if dot:
        cents += int(frac.ljust(2, "0"))
    return sign * cents


def format_cents(cents):
    cents = int(cents)
    sign = "-" if cents < 0 else ""
    cents = abs(cents)
    return f"{sign}{cents // 100}.{cents % 100:02d}"


@dataclass(frozen=True)
class OrderLine:
    sku: str
    quantity: int

    def __post_init__(self):
        object.__setattr__(self, "sku", canonical_sku(self.sku))
        object.__setattr__(self, "quantity", parse_quantity(self.quantity))


@dataclass(frozen=True)
class Reservation:
    reservation_id: str
    order_id: str
    sku: str
    warehouse: str
    quantity: int
    status: str = "active"


@dataclass(frozen=True)
class Allocation:
    order_id: str
    sku: str
    warehouse: str
    quantity: int
    reservation_id: str

    def as_dict(self):
        return {
            "order_id": self.order_id,
            "quantity": self.quantity,
            "reservation_id": self.reservation_id,
            "sku": self.sku,
            "warehouse": self.warehouse,
        }
