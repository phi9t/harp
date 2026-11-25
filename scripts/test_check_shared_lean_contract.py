"""The optional shared interface must preserve Harp's recorded source identity."""

from pathlib import Path
import shutil
import tempfile
import unittest

from check_shared_lean_contract import ROOT, SOURCE_FILES, check


class SharedLeanContractChecks(unittest.TestCase):
    def snapshot(self, root: Path) -> None:
        for relative in [*SOURCE_FILES, "lab/math/shared/linear.json", "lab/math/shared/provenance.json"]:
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, target)

    def test_original_source_and_contract_are_consistent_without_claiming_a_proof(self):
        result = check()
        self.assertEqual(result["source_files"], 2)
        self.assertFalse(result["proof_verified"])

    def test_changed_source_and_contract_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.snapshot(root)
            source = root / "lab/math/lean/MathematicalFoundations/Linear.lean"
            original = source.read_bytes()
            source.write_bytes(original + b"\n-- changed source\n")
            with self.assertRaisesRegex(ValueError, "source digest mismatch"):
                check(root)
            source.write_bytes(original)
            contract = root / "lab/math/shared/linear.json"
            contract.write_bytes(contract.read_bytes().replace(b"Fin n", b"Fin 1"))
            with self.assertRaisesRegex(ValueError, "contract digest mismatch"):
                check(root)

    def test_a_symlink_does_not_count_as_preserved_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.snapshot(root)
            relative = "lab/math/lean/MathematicalFoundations/Linear.lean"
            (root / relative).unlink()
            (root / relative).symlink_to(ROOT / relative)
            with self.assertRaisesRegex(ValueError, "symlinked"):
                check(root)


if __name__ == "__main__":
    unittest.main()
