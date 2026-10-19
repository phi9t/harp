import hashlib
import tempfile
import unittest
from pathlib import Path

from cachekey import build_cache_key


class CacheKeyTests(unittest.TestCase):
    def write_file(self, root: Path, relative: str, content: bytes) -> Path:
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def test_same_relative_file_and_content_match_across_temp_roots(self):
        with tempfile.TemporaryDirectory() as left_tmp, tempfile.TemporaryDirectory() as right_tmp:
            left_root = Path(left_tmp) / "checkout"
            right_root = Path(right_tmp) / "checkout"
            left = self.write_file(left_root, "inputs/config.txt", b"mode=fast\n")
            right = self.write_file(right_root, "inputs/config.txt", b"mode=fast\n")

            left_key = build_cache_key(left, left_root)
            right_key = build_cache_key(right, right_root)

            self.assertEqual(left_key, right_key)
            self.assertNotIn(left_tmp, left_key)
            self.assertNotIn(right_tmp, right_key)

    def test_relative_path_participates_in_the_key(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            first = self.write_file(root, "inputs/a.txt", b"same bytes\n")
            second = self.write_file(root, "generated/a.txt", b"same bytes\n")

            self.assertNotEqual(build_cache_key(first, root), build_cache_key(second, root))

    def test_content_digest_participates_in_the_key(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = self.write_file(root, "inputs/config.txt", b"mode=fast\n")
            expected = "inputs/config.txt:" + hashlib.sha256(b"mode=fast\n").hexdigest()

            self.assertEqual(build_cache_key(source, root), expected)
            source.write_bytes(b"mode=safe\n")
            self.assertNotEqual(build_cache_key(source, root), expected)

    def test_source_outside_workspace_is_rejected(self):
        with tempfile.TemporaryDirectory() as workspace_tmp, tempfile.TemporaryDirectory() as outside_tmp:
            outside = self.write_file(Path(outside_tmp), "config.txt", b"mode=fast\n")

            with self.assertRaises(ValueError):
                build_cache_key(outside, Path(workspace_tmp))


if __name__ == "__main__":
    unittest.main()
