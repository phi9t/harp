import unittest
import pricing

class T(unittest.TestCase):
    def test_discount(self):
        self.assertEqual(pricing.discount(200.0, 15), 170.0)

    def test_tax(self):
        self.assertEqual(pricing.with_tax(100.0, 0.08), 108.0)

    def test_total_still_works(self):
        self.assertEqual(pricing.total([(2.0, 3), (1.5, 2)]), 9.0)

if __name__ == "__main__":
    unittest.main()
