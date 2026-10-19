from .errors import InventoryError, fail
from .models import Reservation, canonical_sku, canonical_warehouse, parse_quantity


class ReservationBook:
    def __init__(self):
        self._reservations = {}
        self._order_counts = {}

    def _next_id(self, order_id):
        order_id = str(order_id).strip()
        if not order_id:
            fail("INVALID_ORDER", "order id must not be empty")
        self._order_counts[order_id] = self._order_counts.get(order_id, 0) + 1
        return f"{order_id}:{self._order_counts[order_id]:03d}"

    def reserve(self, order_id, sku, quantity, warehouse, ledger):
        sku = canonical_sku(sku)
        warehouse = canonical_warehouse(warehouse)
        quantity = parse_quantity(quantity)
        available = ledger.on_hand(sku, warehouse) - self.reserved_quantity(sku, warehouse)
        if available < quantity:
            fail("OUT_OF_STOCK", f"{sku} in {warehouse} has {available} available, need {quantity}")
        reservation_id = self._next_id(order_id)
        if reservation_id in self._reservations and self._reservations[reservation_id].status == "active":
            fail("DUPLICATE_RESERVATION", f"reservation already exists: {reservation_id}")
        reservation = Reservation(reservation_id, str(order_id).strip(), sku, warehouse, quantity, "active")
        self._reservations[reservation_id] = reservation
        return reservation

    def reserve_many(self, order_id, requests, ledger):
        made = []
        try:
            for req in requests:
                made.append(self.reserve(order_id, req["sku"], req["quantity"], req["warehouse"], ledger))
        except InventoryError:
            for reservation in reversed(made):
                self.release(reservation.reservation_id)
            raise
        return made

    def release(self, reservation_id):
        reservation = self._reservations.get(reservation_id)
        if reservation is None:
            fail("UNKNOWN_RESERVATION", f"unknown reservation: {reservation_id}")
        if reservation.status != "active":
            return reservation
        released = Reservation(
            reservation.reservation_id,
            reservation.order_id,
            reservation.sku,
            reservation.warehouse,
            reservation.quantity,
            "released",
        )
        self._reservations[reservation_id] = released
        return released

    def commit(self, reservation_id, ledger):
        reservation = self._reservations.get(reservation_id)
        if reservation is None:
            fail("UNKNOWN_RESERVATION", f"unknown reservation: {reservation_id}")
        if reservation.status != "active":
            fail("UNKNOWN_RESERVATION", f"reservation is not active: {reservation_id}")
        ledger.apply_adjustment(
            reservation.sku,
            reservation.warehouse,
            -reservation.quantity,
            f"ship {reservation.order_id}",
        )
        committed = Reservation(
            reservation.reservation_id,
            reservation.order_id,
            reservation.sku,
            reservation.warehouse,
            reservation.quantity,
            "committed",
        )
        self._reservations[reservation_id] = committed
        return committed

    def reserved_quantity(self, sku, warehouse=None):
        sku = canonical_sku(sku)
        warehouse = canonical_warehouse(warehouse) if warehouse is not None else None
        total = 0
        for reservation in self._reservations.values():
            if reservation.status != "active" or reservation.sku != sku:
                continue
            if warehouse is not None and reservation.warehouse != warehouse:
                continue
            total += reservation.quantity
        return total

    def active_for_order(self, order_id):
        return sorted(
            [
                reservation
                for reservation in self._reservations.values()
                if reservation.order_id == str(order_id).strip() and reservation.status == "active"
            ],
            key=lambda reservation: reservation.reservation_id,
        )
