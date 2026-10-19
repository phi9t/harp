import unittest
import sizes
import report

class T(unittest.TestCase):
    def test_format_size_units(self):
        self.assertEqual(sizes.format_size(512), "512 B")
        self.assertEqual(sizes.format_size(1536), "1.5 KiB")
        self.assertEqual(sizes.format_size(5 * 1024**2), "5.0 MiB")

    def test_report_uses_human_sizes(self):
        out = report.render([("a", "1.5 KiB"), ("b", "2 GiB")])
        self.assertEqual(out, "a: 1.5 KiB\nb: 2.0 GiB")

if __name__ == "__main__":
    unittest.main()
