import unittest

from slugger import slugify


class SluggerTests(unittest.TestCase):
    def test_collapses_punctuation_and_dashes(self):
        self.assertEqual(slugify(" Q3: Core  API -- Audit! "), "q3-core-api-audit")

    def test_empty_after_cleanup_is_untitled(self):
        self.assertEqual(slugify("!!!"), "untitled")


if __name__ == "__main__":
    unittest.main()
