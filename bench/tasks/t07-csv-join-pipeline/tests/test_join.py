import csv
import io
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class JoinOrdersTests(unittest.TestCase):
    def run_join(self, customers_text, orders_text):
        tmp_root = ROOT / ".test-tmp"
        tmp_root.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="join.", dir=tmp_root) as tmp:
            tmp_path = Path(tmp)
            customers = tmp_path / "customers.csv"
            orders = tmp_path / "orders.csv"
            customers.write_text(customers_text, encoding="utf-8")
            orders.write_text(orders_text, encoding="utf-8")
            result = subprocess.run(
                ["python3", str(ROOT / "join_orders.py"), str(customers), str(orders)],
                cwd=ROOT,
                text=True,
                capture_output=True,
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        return list(csv.reader(io.StringIO(result.stdout)))

    def test_keeps_duplicate_order_customer_keys_in_input_order(self):
        rows = self.run_join(
            "customer_id,name\nC1,Atlas\nC2,Boreal\n",
            "order_id,customer_id,total_cents,notes\n"
            "O-1,C1,100,first\n"
            "O-2,C1,250,second\n"
            "O-3,C2,400,third\n",
        )
        self.assertEqual(
            rows,
            [
                ["order_id", "customer_id", "name", "total_cents", "notes"],
                ["O-1", "C1", "Atlas", "100", "first"],
                ["O-2", "C1", "Atlas", "250", "second"],
                ["O-3", "C2", "Boreal", "400", "third"],
            ],
        )

    def test_handles_quoted_commas_in_customer_names_and_notes(self):
        rows = self.run_join(
            'customer_id,name\nC1,"River, Inc."\n',
            'order_id,customer_id,total_cents,notes\nO-9,C1,1200,"rush, fragile"\n',
        )
        self.assertEqual(
            rows,
            [
                ["order_id", "customer_id", "name", "total_cents", "notes"],
                ["O-9", "C1", "River, Inc.", "1200", "rush, fragile"],
            ],
        )


if __name__ == "__main__":
    unittest.main()
