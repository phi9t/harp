from .errors import fail
from .models import Reservation


class ReservationBook:
    """Reservation state for open orders.

    Ticket t2 implements atomic reservations against the StockLedger contract.
    """

    def __init__(self):
        self._reservations = {}

    def reserve(self, order_id, sku, quantity, warehouse, ledger):
        fail("NOT_IMPLEMENTED", "ReservationBook.reserve is not implemented")

    def reserve_many(self, order_id, requests, ledger):
        made = []
        for req in requests:
            made.append(self.reserve(order_id, req["sku"], req["quantity"], req["warehouse"], ledger))
        return made

    def release(self, reservation_id):
        fail("NOT_IMPLEMENTED", "ReservationBook.release is not implemented")

    def commit(self, reservation_id, ledger):
        fail("NOT_IMPLEMENTED", "ReservationBook.commit is not implemented")

    def reserved_quantity(self, sku, warehouse=None):
        fail("NOT_IMPLEMENTED", "ReservationBook.reserved_quantity is not implemented")

    def active_for_order(self, order_id):
        return [
            r for r in self._reservations.values()
            if isinstance(r, Reservation) and r.order_id == order_id and r.status == "active"
        ]
