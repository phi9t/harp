import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class ConfigTests(unittest.TestCase):
    def run_renderer(self, path):
        return subprocess.run(
            ["python3", str(ROOT / "render_config.py"), str(path)],
            cwd=ROOT,
            text=True,
            capture_output=True,
        )

    def test_generated_config_parses_and_renders_production_profile(self):
        result = self.run_renderer(ROOT / "generated.conf")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            result.stdout.strip(),
            "url=https://127.0.0.1:8443 mode=production audit=json retries=3",
        )

    def test_incompatible_production_profile_is_rejected(self):
        tmp_root = ROOT / ".test-tmp"
        tmp_root.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="config.", dir=tmp_root) as tmp:
            bad = Path(tmp) / "bad.conf"
            bad.write_text(
                textwrap.dedent(
                    """
                    [server]
                    host = 127.0.0.1
                    port = 8443
                    mode = production
                    tls = optional
                    retries = 3

                    [audit]
                    enabled = enabled
                    format = json
                    """
                ).strip()
                + "\n",
                encoding="utf-8",
            )
            result = self.run_renderer(bad)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("production requires tls=required", result.stderr)


if __name__ == "__main__":
    unittest.main()
