"""A portable pass, skipped native check or old source receipt cannot qualify."""
import copy
import unittest
from verify_native_receipt import verify


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        self.report = {'status':'passed','integrated':True,'tests_run':2,'tests_expected':2,
                       'source':{'files':{'durable/runtime/engine.py':'current'}},
                       'capabilities':{'production_sqlite_admission':True}}

    def test_complete_exact_source_passes(self):
        verify(self.report, self.report['source']['files'], 'integrated')

    def test_incomplete_skipped_wrong_suite_or_stale_sources_fail(self):
        for change in ({'status':'incomplete'}, {'skipped':[['case','unavailable']]},
                       {'tests_run':0}, {'integrated':False}, {'capabilities':{}},
                       {'source':{'files':{'durable/runtime/engine.py':'old'}}}):
            report=copy.deepcopy(self.report);report.update(change)
            with self.subTest(change=change), self.assertRaises(ValueError):
                verify(report, self.report['source']['files'], 'integrated')
