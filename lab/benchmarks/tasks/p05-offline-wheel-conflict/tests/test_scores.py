import unittest

from alpha_calc import alpha_score
from beta_calc import beta_score
from shared_math import scale


class ScoreTests(unittest.TestCase):
    def test_alpha_and_beta_share_the_same_scale(self):
        self.assertEqual(scale(4), 40)
        self.assertEqual(alpha_score(4), 41)
        self.assertEqual(beta_score(4), 42)

    def test_zero_keeps_package_offsets(self):
        self.assertEqual(alpha_score(0), 1)
        self.assertEqual(beta_score(0), 2)


if __name__ == "__main__":
    unittest.main()
