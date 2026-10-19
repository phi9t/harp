import unittest

from invoice import total_cents


class InvoiceTests(unittest.TestCase):
    def test_decimal_cents_are_exact(self):
        self.assertEqual(total_cents([("0.10", 3), ("0.20", 1)]), 50)

    def test_rounds_half_up_to_nearest_cent(self):
        self.assertEqual(total_cents([("1.005", 1)]), 101)


if __name__ == "__main__":
    unittest.main()
