import unittest
from dataclasses import fields, is_dataclass
from typing import Any, get_type_hints

import analytics
import caller


class AnalyticsContractTests(unittest.TestCase):
    def sample_rows(self) -> list[dict[str, Any]]:
        return [
            {"name": "build", "duration_ms": "10"},
            {"name": "test", "duration_ms": 30},
            {"name": "package", "duration_ms": 20},
        ]

    def test_typed_caller_consumes_summary_attributes(self):
        self.assertEqual(caller.headline(self.sample_rows()), "3 runs, average 20.0 ms")

    def test_summary_type_shape_is_explicit(self):
        summary_cls = getattr(analytics, "RunSummary", None)

        self.assertTrue(is_dataclass(summary_cls))
        self.assertEqual([field.name for field in fields(summary_cls)], ["count", "total_ms", "average_ms"])
        self.assertEqual(get_type_hints(summary_cls), {"count": int, "total_ms": int, "average_ms": float})
        self.assertIs(get_type_hints(analytics.summarize).get("return"), summary_cls)

    def test_empty_input_keeps_numeric_shape(self):
        summary = analytics.summarize([])

        self.assertFalse(isinstance(summary, dict))
        self.assertEqual(summary.count, 0)
        self.assertEqual(summary.total_ms, 0)
        self.assertEqual(summary.average_ms, 0.0)


if __name__ == "__main__":
    unittest.main()
