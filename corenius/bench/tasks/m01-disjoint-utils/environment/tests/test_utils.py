import unittest
from strings_util import title_case
from numbers_util import clamp

class T(unittest.TestCase):
    def test_title_keeps_small_words_lower_and_collapses_spaces(self):
        self.assertEqual(title_case("the  lord of the rings"), "The Lord of the Rings")

    def test_clamp_both_ends(self):
        self.assertEqual(clamp(5, 0, 3), 3)
        self.assertEqual(clamp(-1, 0, 3), 0)
        self.assertEqual(clamp(2, 0, 3), 2)

if __name__ == "__main__":
    unittest.main()
