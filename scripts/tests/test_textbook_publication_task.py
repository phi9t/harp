"""Run the publication task with distinct built and XDG-cache executables."""

import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


def publication_command():
    source = (ROOT / "mise.toml").read_text()
    task = re.search(
        r"(?ms)^\[tasks\.crouzeix-textbook-publication\]\n(.*?)(?=^\[|\Z)",
        source,
    )
    if task is None:
        raise AssertionError("Missing textbook publication task")
    command = re.search(r'(?ms)^run = """\n(.*?)\n"""', task.group(1))
    if command is None:
        raise AssertionError("Expected the publication task's multiline shell command")
    return command.group(1)


class TextbookPublicationTaskTests(unittest.TestCase):
    def test_publish_and_check_use_the_binary_built_before_xdg_setup(self):
        with tempfile.TemporaryDirectory(prefix="harp publication task ") as directory:
            root = Path(directory).resolve()
            scripts = root / "scripts"
            scripts.mkdir()
            temporary = root / "receipts"
            temporary.mkdir()
            log = root / "calls"
            built = root / "built target"
            cached = root / "cached target"
            for target, label in [(built, "built"), (cached, "cached")]:
                binary = target / "size/harp"
                binary.parent.mkdir(parents=True)
                binary.write_text(
                    "#!/bin/sh\nset -eu\n"
                    'test "$1" = crouzeix-textbook\n'
                    'test "$3" = --receipt\n'
                    'test -s "$4"\n'
                    f'printf "{label} %s\\n" "$2" >> "$TASK_CALLS"\n'
                )
                binary.chmod(0o755)
            (scripts / "harp_xdg_env.sh").write_text(
                'export HARP_TARGET_DIR="$TASK_CACHED_TARGET"\n'
                'export HARP_ELAN_HOME="$TASK_ELAN_HOME"\n'
                'export HARP_LEAN_TOOLCHAIN_BIN="$TASK_TOOLCHAIN_BIN"\n'
            )
            wrapper = scripts / "check_lean_library.sh"
            wrapper.write_text(
                "#!/bin/sh\nset -eu\n"
                'test "$1" = CrouzeixTextbook\n'
                'test "$2" = --receipt-output\n'
                'test "$ELAN_HOME" = "$TASK_ELAN_HOME"\n'
                'test "$HARP_TARGET_DIR" = "$TASK_CACHED_TARGET"\n'
                'printf "%s\\n" receipt > "$3"\n'
            )
            wrapper.chmod(0o755)
            environment = os.environ.copy()
            environment.update(
                HARP_TARGET_DIR=str(built),
                TASK_CACHED_TARGET=str(cached),
                TASK_ELAN_HOME=str(root / "elan"),
                TASK_TOOLCHAIN_BIN=str(root / "toolchain"),
                TASK_CALLS=str(log),
                TMPDIR=str(temporary),
            )
            result = subprocess.run(
                ["/bin/bash", "-c", publication_command()],
                cwd=root,
                env=environment,
                capture_output=True,
                text=True,
                timeout=15,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(log.read_text().splitlines(), ["built publish", "built check"])
            self.assertEqual(list(temporary.iterdir()), [], "Receipt cleanup did not run")


if __name__ == "__main__":
    unittest.main()
