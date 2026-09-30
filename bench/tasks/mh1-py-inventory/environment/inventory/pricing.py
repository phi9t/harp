from .errors import fail


class PriceBook:
    """Cents-based SKU price book and discount engine.

    Ticket t3 implements price storage, category discounts, order discounts,
    deterministic rule ordering, and quote generation.
    """

    def __init__(self):
        self._prices = {}
        self._rules = []

    def set_price(self, sku, unit_price_cents, category="general"):
        fail("NOT_IMPLEMENTED", "PriceBook.set_price is not implemented")

    def add_category_discount(self, code, category, percent, min_quantity=1):
        fail("NOT_IMPLEMENTED", "PriceBook.add_category_discount is not implemented")

    def add_order_discount(self, code, threshold_cents, percent):
        fail("NOT_IMPLEMENTED", "PriceBook.add_order_discount is not implemented")

    def unit_price(self, sku):
        fail("NOT_IMPLEMENTED", "PriceBook.unit_price is not implemented")

    def quote(self, lines):
        fail("NOT_IMPLEMENTED", "PriceBook.quote is not implemented")
