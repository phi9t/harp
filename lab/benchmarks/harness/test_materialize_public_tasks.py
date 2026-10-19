"""Public benchmark materializers produce Corenius task layouts without vendoring upstream trees."""

from __future__ import annotations

import json
import os
import subprocess
import tarfile
import tempfile
import unittest
import gzip
import zipfile
from pathlib import Path

import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import materialize_public_tasks as mat

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "insula" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "insula" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


class MaterializePublicTasksTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=str(mat.ROOT / ".scratch"))
        self.root = Path(self.tmp.name)
        self.source = self.root / "source"
        self.out = self.root / "out"
        self.source.mkdir()

    def tearDown(self):
        self.tmp.cleanup()

    def make_terminal_task(self, name: str = "analyze-access-logs", compose: str | None = "services:\n  client:\n") -> Path:
        task = self.source / name
        (task / "tests").mkdir(parents=True)
        (task / "artifacts").mkdir()
        (task / "tests/test_outputs.py").write_text("def test_ok():\n    assert True\n")
        (task / "solution.sh").write_text("#!/bin/sh\nset -e\n: > /app/report.txt\n")
        (task / "task.yaml").write_text(
            "instruction: |-\n"
            "  Write the report.\n"
            "difficulty: easy\n"
            "tags:\n"
            "  - file-operations\n"
            "parser_name: pytest\n"
            "max_agent_timeout_sec: 900.0\n"
            "max_test_timeout_sec: 180.0\n"
        )
        (task / "artifacts/input.txt").write_text("input\n")
        if compose is not None:
            (task / "docker-compose.yaml").write_text(compose)
        return task

    def git(self, args: list[str], cwd: Path) -> str:
        result = subprocess.run(
            ["git", *args],
            cwd=cwd,
            text=True,
            capture_output=True,
            check=True,
            env={**os.environ, "GIT_AUTHOR_NAME": "test", "GIT_AUTHOR_EMAIL": "test@example.test",
                 "GIT_COMMITTER_NAME": "test", "GIT_COMMITTER_EMAIL": "test@example.test"},
        )
        return result.stdout.strip()

    def make_target_repo(self) -> tuple[Path, str]:
        repo = self.root / "target_repo"
        repo.mkdir()
        self.git(["init", "-q"], repo)
        (repo / "value.txt").write_text("base\n")
        self.git(["add", "value.txt"], repo)
        self.git(["commit", "-q", "-m", "base"], repo)
        return repo, self.git(["rev-parse", "HEAD"], repo)

    def make_deep_swe_task(self, repo: Path, base_commit: str, name: str = "tiny-task") -> Path:
        task = self.source / name
        (task / "tests").mkdir(parents=True)
        (task / "solution").mkdir()
        (task / "environment").mkdir()
        (task / "instruction.md").write_text("Fix the tiny task.\n")
        (task / "task.toml").write_text(
            'schema_version = "1.3"\n'
            "[task]\n"
            'name = "datacurve/tiny-task"\n'
            "[metadata]\n"
            'task_id = "tiny-task"\n'
            'display_title = "Tiny task"\n'
            'display_description = "Make value fixed."\n'
            'category = "test"\n'
            'language = "text"\n'
            f"repository_url = {json.dumps(str(repo))}\n"
            f'base_commit_hash = "{base_commit}"\n'
            "[verifier]\n"
            "timeout_sec = 60.0\n"
            "[agent]\n"
            "timeout_sec = 120.0\n"
            "[environment]\n"
            'docker_image = "example/deep-swe:tiny"\n'
        )
        (task / "tests/test.sh").write_text("#!/bin/bash\nexit 0\n")
        (task / "tests/grader.py").write_text("print('grader')\n")
        (task / "tests/config.json").write_text("{}\n")
        (task / "tests/test.patch").write_text("")
        (task / "solution/solution.patch").write_text(
            "diff --git a/value.txt b/value.txt\n"
            "--- a/value.txt\n"
            "+++ b/value.txt\n"
            "@@ -1 +1 @@\n"
            "-base\n"
            "+fixed\n"
        )
        return task

    def make_swe_bench_row(self, repo: Path, base_commit: str) -> dict:
        return {
            "instance_id": "local__tiny-1",
            "repo": str(repo),
            "base_commit": base_commit,
            "environment_setup_commit": "setup123",
            "image": "example/swe-bench:tiny",
            "version": "1.0",
            "difficulty": "easy",
            "problem_statement": "Fix the tiny SWE-bench task.\n\nDetails follow.",
            "patch": (
                "diff --git a/value.txt b/value.txt\n"
                "--- a/value.txt\n"
                "+++ b/value.txt\n"
                "@@ -1 +1 @@\n"
                "-base\n"
                "+fixed\n"
            ),
            "test_patch": (
                "diff --git a/test_value.py b/test_value.py\n"
                "new file mode 100644\n"
                "--- /dev/null\n"
                "+++ b/test_value.py\n"
                "@@ -0,0 +1,2 @@\n"
                "+def test_value():\n"
                "+    assert open('value.txt').read().strip() == 'fixed'\n"
            ),
            "FAIL_TO_PASS": ["test_value.py::test_value"],
            "PASS_TO_PASS": [],
        }

    def make_program_bench_task(self, instance_id: str = "abishekvashok__cmatrix.5c082c6") -> tuple[Path, Path, Path]:
        task = self.source / instance_id
        task.mkdir(parents=True)
        (task / "task.yaml").write_text(
            "repository: abishekvashok/cmatrix\n"
            "commit: 5c082c64a1296859a11bee60c8c086655953a416\n"
            "language: c\n"
            "difficulty: easy\n"
            "eval_clean_hashes:\n"
            "- 776f1c415ab5cb4e73320eb016f8627ae1fc2364c308dace7794989e9fb4a210\n"
        )
        (task / "tests.json").write_text(json.dumps({
            "branches": {
                "79b69dd3fd98": {
                    "ignored": False,
                    "ignore_reason": "",
                    "tests": ["tests.test_cli.test_help", "tests.test_cli.test_version"],
                    "ignored_tests": [{"name": "tests.test_cli.test_version", "reasons": [{"id": "flaky"}]}],
                }
            }
        }))
        rootfs = self.root / "program_rootfs"
        (rootfs / "workspace").mkdir(parents=True)
        (rootfs / "usr/bin").mkdir(parents=True)
        (rootfs / "usr/local/go/mod-cache").mkdir(parents=True)
        (rootfs / "workspace/README.md").write_text("Run ./executable to inspect behavior.\n")
        executable = rootfs / "workspace/executable"
        executable.write_text("#!/bin/sh\necho cmatrix\n")
        executable.chmod(0o111)
        (rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {
                "env": [
                    "PATH=/usr/local/go/bin:/usr/bin",
                    "GOMODCACHE=/usr/local/go/mod-cache",
                ]
            },
        }))
        blob_cache = self.root / "program_blobs"
        branch_root = self.root / "branch_payload"
        (branch_root / "eval/tests").mkdir(parents=True)
        (branch_root / "eval/run.sh").write_text("#!/bin/sh\n: > eval/results.xml\n")
        (branch_root / "source.c").write_text("int main(void) { return 0; }\n")
        branch_tar = blob_cache / instance_id / "tests/79b69dd3fd98.tar.gz"
        branch_tar.parent.mkdir(parents=True)
        with tarfile.open(branch_tar, "w:gz") as archive:
            for path in sorted(branch_root.rglob("*")):
                archive.add(path, path.relative_to(branch_root))
        return task, rootfs, blob_cache

    def test_materializes_single_client_terminal_bench_task(self):
        task = self.make_terminal_task()
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)
        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_analyze_access_logs"
        self.assertTrue((generated / "task.toml").is_file())
        self.assertTrue((generated / "instruction.md").is_file())
        self.assertTrue((generated / "tests/test_outputs.py").is_file())
        self.assertTrue((generated / "environment/artifacts/input.txt").is_file())
        self.assertIn('capsule = "terminal_bench"', (generated / "task.toml").read_text())
        self.assertIn("/app", (generated / "solution/solve.sh").read_text())

    def test_dockerfile_directory_copy_is_idempotent_when_target_already_exists(self):
        task = self.make_terminal_task(name="assign-seats")
        (task / "Dockerfile").write_text("FROM example\nCOPY deps/ /app/deps/\n")
        (task / "deps/nested").mkdir(parents=True)
        (task / "deps/ask_alice.pkl").write_bytes(b"alice")
        (task / "deps/nested/seats.json").write_text("{}\n")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_assign_seats"
        self.assertEqual((generated / "environment/deps/ask_alice.pkl").read_bytes(), b"alice")
        self.assertEqual((generated / "environment/deps/nested/seats.json").read_text(), "{}\n")

    def test_dockerfile_directory_copy_to_app_slash_places_contents_at_workspace_root(self):
        task = self.make_terminal_task(name="constraints-scheduling")
        (task / "Dockerfile").write_text("FROM example\nWORKDIR /app\nCOPY inputs/ /app/\n")
        (task / "inputs").mkdir()
        (task / "inputs/alice_calendar.ics").write_text("BEGIN:VCALENDAR\nEND:VCALENDAR\n")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_constraints_scheduling"
        self.assertTrue((generated / "environment/inputs/alice_calendar.ics").is_file())
        self.assertEqual(
            (generated / "environment/alice_calendar.ics").read_text(),
            "BEGIN:VCALENDAR\nEND:VCALENDAR\n",
        )

    def test_dockerfile_directory_copy_without_trailing_slash_copies_contents(self):
        task = self.make_terminal_task(name="fix-git")
        (task / "Dockerfile").write_text("FROM example\nCOPY resources /app/resources\n")
        (task / "resources/patch_files").mkdir(parents=True)
        (task / "resources/patch_files/default.html").write_text("<h1>Hello</h1>\n")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_fix_git"
        self.assertEqual(
            (generated / "environment/resources/patch_files/default.html").read_text(),
            "<h1>Hello</h1>\n",
        )
        self.assertFalse((generated / "environment/resources/resources").exists())

    def test_materializes_classifier_debug_with_verifier_outside_workspace(self):
        task = self.make_terminal_task(name="classifier-debug")
        (task / "code.py").write_text("import numpy as np\n")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        task_toml = (self.out / "terminal_bench_classifier_debug/task.toml").read_text()
        self.assertIn('command = "cd /run/insula-tmp && python3 -m pytest /workspace/tests -rA"', task_toml)
        self.assertIn('protected = ["tests"]', task_toml)

    def test_materializes_cancel_async_tasks_with_verifier_helper(self):
        task = self.make_terminal_task(name="cancel-async-tasks")
        (task / "tests/test.py").write_text("print('helper')\n")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_cancel_async_tasks"
        self.assertEqual((generated / "environment/test.py").read_text(), "print('helper')\n")
        self.assertIn('protected = ["tests", "test.py"]', (generated / "task.toml").read_text())

    def test_materializes_tmux_advanced_workflow_setup_outputs(self):
        task = self.make_terminal_task(name="tmux-advanced-workflow")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_tmux_advanced_workflow"
        process_data = generated / "environment/project/src/process_data.py"
        monitor = generated / "environment/project/src/monitor.py"
        self.assertTrue(process_data.is_file())
        self.assertTrue(monitor.is_file())
        self.assertTrue(os.access(process_data, os.X_OK))
        self.assertTrue(os.access(monitor, os.X_OK))
        self.assertIn("parts = line.split", process_data.read_text())
        self.assertTrue((generated / "environment/project/data/input.csv").is_file())
        self.assertTrue((generated / "environment/project/logs/process.log").is_file())

    def test_materializes_jsonl_aggregator_build_time_records(self):
        task = self.make_terminal_task(name="jsonl-aggregator")
        (task / "Dockerfile").write_text(
            "FROM example\n"
            "COPY task-deps/generate_records.py /app/generate_records.py\n"
            "RUN python3 generate_records.py\n"
            "RUN rm /app/generate_records.py\n"
        )
        (task / "task-deps").mkdir()
        (task / "task-deps/generate_records.py").write_text(
            "from pathlib import Path\n"
            "Path('records_1.jsonl').write_text('{\"user\":\"ada\"}\\n')\n"
        )

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_jsonl_aggregator"
        self.assertEqual((generated / "environment/records_1.jsonl").read_text(), '{"user":"ada"}\n')
        self.assertFalse((generated / "environment/generate_records.py").exists())
        self.assertFalse((generated / "environment/task-deps").exists())

    def test_materializes_flood_monitoring_basic_with_data_alias(self):
        task = self.make_terminal_task(name="flood-monitoring-basic")
        (task / "Dockerfile").write_text(
            'FROM example\n'
            'RUN mkdir -p /data && \\\n'
            '    echo "timestamp,water_level_m" > /data/station_A.csv && \\\n'
            '    echo "2024-01-01 00:00,2.3" >> /data/station_A.csv\n'
        )

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        self.assertTrue(result["custom_policy"])
        generated = self.out / "terminal_bench_flood_monitoring_basic"
        self.assertEqual(
            (generated / "environment/data/station_A.csv").read_text(),
            "timestamp,water_level_m\n2024-01-01 00:00,2.3\n",
        )
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["policy"]["workspace_path_aliases"]["/data"], "data")

    def test_materializes_processing_pipeline_broken_initial_state_with_data_alias(self):
        task = self.make_terminal_task(name="processing-pipeline")
        for name in ("run_pipeline.sh", "collect_data.sh", "process_data.sh", "generate_report.sh"):
            (task / name).write_text("#!/bin/bash\necho ok\n")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_processing_pipeline"
        self.assertEqual((generated / "environment/run_pipeline.sh").stat().st_mode & 0o777, 0o644)
        self.assertEqual((generated / "environment/process_data.sh").stat().st_mode & 0o777, 0o644)
        self.assertEqual((generated / "environment/collect_data.sh").stat().st_mode & 0o777, 0o311)
        self.assertEqual((generated / "environment/generate_report.sh").stat().st_mode & 0o777, 0o755)
        self.assertIn(b"\r\n", (generated / "environment/process_data.sh").read_bytes())
        self.assertTrue((generated / "environment/generate_report.sh").read_text().startswith("#!/bin/nonexistent"))
        self.assertEqual((generated / "environment/data/output").stat().st_mode & 0o777, 0)
        (generated / "environment/data/output").chmod(0o700)
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["policy"]["workspace_path_aliases"]["/data"], "data")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual((generated / "environment/data/output").stat().st_mode & 0o777, 0)
        (generated / "environment/data/output").chmod(0o700)

    def test_materializes_count_call_stack_unzipped_log(self):
        task = self.make_terminal_task(name="count-call-stack")
        with zipfile.ZipFile(task / "log.stack.zip", "w") as archive:
            archive.writestr("log.stack", "root\n  child\n")
        (task / "Dockerfile").write_text(
            "FROM example\n"
            "WORKDIR /app\n"
            "COPY log.stack.zip /app/log.stack.zip\n"
            "RUN unzip log.stack.zip && rm log.stack.zip\n"
        )

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_count_call_stack"
        self.assertEqual((generated / "environment/log.stack").read_text(), "root\n  child\n")
        self.assertFalse((generated / "environment/log.stack.zip").exists())

    def test_materializes_gcode_to_text_decompressed_input_and_offline_reference(self):
        task = self.make_terminal_task(name="gcode-to-text")
        with gzip.open(task / "text.gcode.gz", "wb") as fh:
            fh.write(b"G1 X1 Y2 E3\n")
        (task / "Dockerfile").write_text(
            "FROM example\n"
            "COPY text.gcode.gz /app\n"
            "RUN gzip -d /app/text.gcode.gz\n"
        )
        (task / "solution.sh").write_text("#!/bin/bash\napt-get update -y\npip install pytesseract\n")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_gcode_to_text"
        self.assertEqual((generated / "environment/text.gcode").read_text(), "G1 X1 Y2 E3\n")
        self.assertFalse((generated / "environment/text.gcode.gz").exists())
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("test -s /app/text.gcode", solution)
        self.assertIn("flag{gc0d3_iz_ch4LLenGiNg}", solution)
        self.assertNotIn("apt-get", solution)
        self.assertNotIn("pip install", solution)

    def test_materializes_large_scale_text_editing_build_time_csvs(self):
        task = self.make_terminal_task(name="large-scale-text-editing")
        (task / "gen_large_csv.py").write_text(
            "import sys\n"
            "from pathlib import Path\n"
            "assert sys.argv[1:] == ['both']\n"
            "Path('input.csv').write_text('messy\\n')\n"
            "Path('expected.csv').write_text('clean\\n')\n"
        )

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_large_scale_text_editing"
        self.assertEqual((generated / "environment/input.csv").read_text(), "messy\n")
        self.assertEqual((generated / "environment/expected.csv").read_text(), "clean\n")
        self.assertFalse((generated / "environment/gen_large_csv.py").exists())
        self.assertIn(
            "large_scale_text_editing_input.csv",
            (generated / "task.toml").read_text(),
        )

    def test_materializes_log_summary_date_ranges_build_time_logs(self):
        task = self.make_terminal_task(name="log-summary-date-ranges")
        (task / "log_generator_deterministic.py").write_text(
            "import os\n"
            "output_dir = \"/app/logs\"\n"
            "os.makedirs(output_dir, exist_ok=True)\n"
            "open(os.path.join(output_dir, '2025-08-12_app.log'), 'w').write('log\\n')\n"
        )

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_log_summary_date_ranges"
        self.assertEqual((generated / "environment/logs/2025-08-12_app.log").read_text(), "log\n")
        self.assertFalse((generated / "environment/log_generator_deterministic.py").exists())

    def test_materializes_recover_accuracy_log_workspace(self):
        task = self.make_terminal_task(name="recover-accuracy-log")
        (task / "task_deps").mkdir()
        (task / "task_deps/generator.log").write_text("{}\n")
        (task / "task_deps/judge.log").write_text("{}\n")
        (task / "Dockerfile").write_text(
            "FROM example\n"
            "WORKDIR /app\n"
            "COPY task_deps/ /app/raw_logs/\n"
            "RUN mkdir -p /app/recovered_logs/\n"
        )

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_recover_accuracy_log"
        self.assertEqual((generated / "environment/raw_logs/generator.log").read_text(), "{}\n")
        self.assertTrue((generated / "environment/recovered_logs").is_dir())
        self.assertFalse((generated / "environment/task_deps").exists())

    def test_materializes_deterministic_tarball_source_tree(self):
        task = self.make_terminal_task(name="deterministic-tarball")
        (task / "setup_source_tree.sh").write_text(
            "#!/bin/bash\n"
            "set -euo pipefail\n"
            "mkdir -p /app/src/deep\n"
            "printf 'hello\\r\\n' > /app/src/deep/readme.txt\n"
            "ln -s deep/readme.txt /app/src/readme-link\n"
        )

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_deterministic_tarball"
        self.assertEqual((generated / "environment/src/deep/readme.txt").read_bytes(), b"hello\r\n")
        self.assertTrue((generated / "environment/src/readme-link").is_symlink())
        self.assertFalse((generated / "environment/setup_source_tree.sh").exists())

    def test_materializes_jq_data_processing_inputs_and_yq_shim(self):
        task = self.make_terminal_task(name="jq-data-processing")
        (task / "task-deps").mkdir()
        (task / "task-deps/generate_data.py").write_text("raise SystemExit('not used')\n")

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["extra_layers"], ["jq_data_processing_tools"])
        generated = self.out / "terminal_bench_jq_data_processing"
        self.assertTrue((generated / "environment/users.json").is_file())
        self.assertTrue((generated / "environment/.auto_gen_key.dat").is_file())
        self.assertFalse((generated / "environment/generate_data.py").exists())
        self.assertFalse((generated / "environment/task-deps").exists())
        shim = generated / "tool_layers/jq_data_processing/yq"
        self.assertTrue(os.access(shim, os.X_OK))
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][-1]["path"], ["."])

    def test_materializes_tree_directory_parser_generated_tree_map(self):
        task = self.make_terminal_task(name="tree-directory-parser")
        (task / "task-deps").mkdir()
        (task / "task-deps/generate_data.sh").write_text(
            "#!/bin/bash\n"
            "set -euo pipefail\n"
            "mkdir -p /app/files_dir/nested\n"
            "touch /app/files_dir/nested/item\n"
            "tree -F /app/files_dir > /app/tree_map.txt\n"
            "rm -rf /app/files_dir\n"
        )

        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)

        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_tree_directory_parser"
        self.assertIn("/app/files_dir/", (generated / "environment/tree_map.txt").read_text())
        self.assertTrue((generated / "environment/.auto_gen_key.dat").is_file())
        self.assertFalse((generated / "environment/generate_data.sh").exists())
        self.assertFalse((generated / "environment/task-deps").exists())
        self.assertFalse((generated / "environment/files_dir").exists())

    def test_materializes_sha_and_sparql_dependency_layers(self):
        for name, packages in {
            "sha-puzzle": ["word2number==1.1"],
            "sparql-university": ["rdflib==7.1.4"],
            "sparql-professors-universities": ["rdflib==7.1.4"],
        }.items():
            with self.subTest(name=name):
                task = self.make_terminal_task(name=name)
                result = mat.materialize_terminal_bench_task(
                    task,
                    self.out,
                    replace=True,
                    prepare_python_deps=False,
                )
                self.assertEqual(result["status"], "materialized")
                self.assertEqual(result["python_deps"], packages)

    def test_materializes_accelerate_maximal_square_with_python_deps(self):
        task = self.make_terminal_task(name="accelerate-maximal-square")
        (task / "solution.sh").write_text("pip3 install numba==0.53.1 \n\n: > /app/maximal_square.py\n")
        result = mat.materialize_terminal_bench_task(
            task,
            self.out,
            replace=True,
            prepare_python_deps=False,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["python_deps"], ["numpy==1.26.4"])
        solution = (self.out / "terminal_bench_accelerate_maximal_square/solution/solve.sh").read_text()
        self.assertIn("gcc -O3 -march=native -fPIC -shared", solution)
        self.assertIn("ctypes.CDLL", solution)
        self.assertNotIn("pip3 install numba==0.53.1", solution)

    def test_materializes_audio_synth_stft_peaks_with_offline_python_deps(self):
        task = self.make_terminal_task(name="audio-synth-stft-peaks")
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "set -euo pipefail\n"
            "gen_peaks() { awk 'BEGIN { print 1 }'; }\n"
            "if [ \"${PEAKS_ONLY:-0}\" = \"1\" ]; then\n"
            "  gen_peaks\n"
            "  exit 0\n"
            "fi\n"
            "# Update package manager and install pip (normal full pipeline)\n"
            "apt-get update -y\n"
            "apt-get install -y python3-pip\n"
            "\n"
            "# Pin versions for reproducibility\n"
            "pip3 install numpy==1.26.4 scipy==1.14.0 --break-system-packages\n"
            "python3 audioproc.py\n"
            "gen_peaks\n"
        )
        result = mat.materialize_terminal_bench_task(
            task,
            self.out,
            replace=True,
            prepare_python_deps=False,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["python_deps"], ["numpy==1.26.4", "scipy==1.14.0"])
        solution = (self.out / "terminal_bench_audio_synth_stft_peaks/solution/solve.sh").read_text()
        self.assertIn("#!/bin/bash", solution)
        self.assertIn("set -euo pipefail", solution)
        self.assertIn("gen_peaks() { awk", solution)
        self.assertIn("import scipy", solution)
        self.assertNotIn("apt-get", solution)
        self.assertNotIn("pip3 install", solution)

    def test_materializes_heterogeneous_dates_with_offline_pandas(self):
        task = self.make_terminal_task(name="heterogeneous-dates")
        (task / "Dockerfile").write_text("FROM example\nCOPY task-deps/ ./\n")
        (task / "task-deps").mkdir()
        (task / "task-deps/daily_temp_sf_high.csv").write_text("date,temperature\n2025-04-19,64\n")
        (task / "task-deps/daily_temp_sf_low.csv").write_text("date,temperature\n04/19/2025 06:00:00,48\n")
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "cat /app/daily_temp_sf_high.csv\n"
            "# Step 2: Install required dependencies\n"
            "echo \"Installing required dependencies...\"\n"
            "pip install pandas\n"
            "cat << 'EOF' > /app/process_data.py\n"
            "import pandas as pd\n"
            "EOF\n"
            "python /app/process_data.py\n"
        )
        result = mat.materialize_terminal_bench_task(
            task,
            self.out,
            replace=True,
            prepare_python_deps=False,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["python_deps"], ["numpy==2.3.1", "pandas==2.3.0"])
        generated = self.out / "terminal_bench_heterogeneous_dates"
        self.assertTrue((generated / "environment/daily_temp_sf_high.csv").is_file())
        self.assertTrue((generated / "environment/daily_temp_sf_low.csv").is_file())
        self.assertFalse((generated / "environment/task-deps").exists())
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("import pandas", solution)
        self.assertIn("python3 /app/process_data.py", solution)
        self.assertNotIn("pip install pandas", solution)
        self.assertNotIn("python /app/process_data.py", solution)

    def test_materializes_pandas_etl_with_offline_pandas(self):
        task = self.make_terminal_task(name="pandas-etl")
        (task / "Dockerfile").write_text("FROM example\nCOPY task-deps/data.csv ./\n")
        (task / "task-deps").mkdir()
        (task / "task-deps/data.csv").write_text("participant_id,first_name,last_name\n1,Ada,Lovelace\n")
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "curl -LsSf https://astral.sh/uv/install.sh | sh\n"
            "source $HOME/.local/bin/env\n"
            "\n"
            "uv init\n"
            "uv add \"pandas==2.2.3\"\n"
            "cat << 'EOF' > run_etl.py\n"
            "import pandas as pd\n"
            "EOF\n"
            "uv run run_etl.py\n"
        )
        packages = ["pandas==2.2.3"]
        digest = mat.hashlib.sha256("\n".join(packages).encode("utf-8")).hexdigest()[:12]
        cache_root = self.root / "python_layers"
        existing = cache_root / f"terminal_bench_pandas_etl_py{mat.sys.version_info.major}{mat.sys.version_info.minor}_{digest}"
        (existing / "site-packages").mkdir(parents=True)
        (existing / "manifest.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "python": f"{mat.sys.version_info.major}.{mat.sys.version_info.minor}",
                    "packages": packages,
                }
            )
        )
        result = mat.materialize_terminal_bench_task(
            task,
            self.out,
            replace=True,
            python_layer_cache=cache_root,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["python_deps"], packages)
        self.assertTrue(result["python_deps_ready"])
        generated = self.out / "terminal_bench_pandas_etl"
        self.assertTrue((generated / "environment/data.csv").is_file())
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("import pandas", solution)
        self.assertIn("python3 run_etl.py", solution)
        self.assertNotIn("curl -LsSf", solution)
        self.assertNotIn("uv init", solution)
        self.assertNotIn("uv add", solution)
        self.assertNotIn("uv run", solution)
        request = json.loads((generated / "capsule_request.json").read_text())
        tree_layers = [layer for layer in request["layers"] if layer["kind"] == "tree"]
        self.assertEqual(tree_layers[0]["name"], "python_deps")
        self.assertEqual(tree_layers[0]["link_mode"], "hardlink")

    def test_materializes_pandas_sql_query_with_generated_data(self):
        task = self.make_terminal_task(name="pandas-sql-query")
        (task / "Dockerfile").write_text(
            "FROM example\n"
            "WORKDIR /app\n"
            "COPY tests/generate_data.py /app/generate_data.py\n"
            "RUN /root/.local/bin/uv run python /app/generate_data.py\n"
        )
        (task / "tests/generate_data.py").write_text("print('generate')\n")
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "source $HOME/.local/bin/env\n"
            "\n"
            "cat <<'SQL' > /app/answer.sql\n"
            "SELECT 1;\n"
            "SQL\n"
        )
        packages = ["pandas==2.1.4", "duckdb==1.3.2"]
        digest = mat.hashlib.sha256("\n".join(packages).encode("utf-8")).hexdigest()[:12]
        cache_root = self.root / "python_layers"
        existing = cache_root / f"terminal_bench_pandas_sql_query_py{mat.sys.version_info.major}{mat.sys.version_info.minor}_{digest}"
        (existing / "site-packages").mkdir(parents=True)
        (existing / "manifest.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "python": f"{mat.sys.version_info.major}.{mat.sys.version_info.minor}",
                    "packages": packages,
                }
            )
        )
        result = mat.materialize_terminal_bench_task(
            task,
            self.out,
            replace=True,
            python_layer_cache=cache_root,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["python_deps"], packages)
        self.assertTrue(result["python_deps_ready"])
        generated = self.out / "terminal_bench_pandas_sql_query"
        self.assertTrue((generated / "environment/generate_data.py").is_file())
        products = (generated / "environment/products.csv").read_text().splitlines()
        orders = (generated / "environment/orders.csv").read_text().splitlines()
        self.assertEqual(products[0], "product_id,category,product_name")
        self.assertEqual(orders[0], "order_id,product_id,customer_id,order_date,quantity,unit_price")
        self.assertEqual(len(products), 101)
        self.assertEqual(len(orders), 10001)
        solution = (generated / "solution/solve.sh").read_text()
        self.assertNotIn("source $HOME/.local/bin/env", solution)
        request = json.loads((generated / "capsule_request.json").read_text())
        tree_layers = [layer for layer in request["layers"] if layer["kind"] == "tree"]
        self.assertEqual(tree_layers[0]["name"], "python_deps")
        self.assertEqual(tree_layers[0]["link_mode"], "hardlink")

    def test_materializes_numpy_scipy_tasks_without_runtime_pip(self):
        for name in ("optimal-transport", "raman-fitting"):
            with self.subTest(name=name):
                task = self.make_terminal_task(name=name)
                if name == "optimal-transport":
                    (task / "solution.sh").write_text(
                        "#!/bin/bash\n"
                        "# Install required packages\n"
                        "pip install numpy==2.3.1 scipy==1.16.0\n"
                    )
                else:
                    (task / "solution.sh").write_text(
                        "#!/bin/bash\n"
                        "# install python dependencies\n"
                        "pip install scipy==1.16.0 numpy==2.3.1\n"
                        "python --version\n"
                        "python convert.py\n"
                        "python fit.py\n"
                    )
                result = mat.materialize_terminal_bench_task(
                    task,
                    self.out,
                    replace=True,
                    prepare_python_deps=False,
                )
                self.assertEqual(result["status"], "materialized")
                self.assertEqual(result["python_deps"], ["numpy==2.3.1", "scipy==1.16.0"])
                generated = self.out / f"terminal_bench_{name.replace('-', '_')}"
                solution = (generated / "solution/solve.sh").read_text()
                self.assertIn("import numpy", solution)
                self.assertIn("import scipy", solution)
                self.assertNotIn("pip install numpy==2.3.1 scipy==1.16.0", solution)
                self.assertNotIn("pip install scipy==1.16.0 numpy==2.3.1", solution)

    def test_reuses_python_dependency_layers_with_matching_manifest(self):
        packages = ["numpy==2.3.1", "scipy==1.16.0"]
        digest = mat.hashlib.sha256("\n".join(packages).encode("utf-8")).hexdigest()[:12]
        cache_root = self.root / "python_layers"
        existing = cache_root / f"terminal_bench_raman_fitting_py{mat.sys.version_info.major}{mat.sys.version_info.minor}_{digest}"
        (existing / "site-packages").mkdir(parents=True)
        (existing / "manifest.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "python": f"{mat.sys.version_info.major}.{mat.sys.version_info.minor}",
                    "packages": packages,
                }
            )
        )

        selected = mat.prepare_python_deps_layer(
            "terminal_bench_optimal_transport",
            packages,
            cache_root=cache_root,
        )

        self.assertEqual(selected, existing)
        self.assertFalse((cache_root / f"terminal_bench_optimal_transport_py{mat.sys.version_info.major}{mat.sys.version_info.minor}_{digest}").exists())

    def test_materializes_organization_json_generator_with_dockerfile_copies(self):
        task = self.make_terminal_task(name="organization-json-generator")
        (task / "Dockerfile").write_text(
            "FROM example\n"
            "COPY departments.csv /app/data/\n"
            "COPY employees.csv /app/data/\n"
            "COPY projects.csv /app/data/\n"
            "COPY schema.json /app/\n"
        )
        for name in ("departments.csv", "employees.csv", "projects.csv", "schema.json"):
            (task / name).write_text("{}\n" if name.endswith(".json") else "id,name\n")
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "pip install jsonschema\n"
            "cat > /app/generate_organization.py << 'EOF'\n"
            "import jsonschema\n"
            "EOF\n"
        )
        result = mat.materialize_terminal_bench_task(
            task,
            self.out,
            replace=True,
            prepare_python_deps=False,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["python_deps"], ["jsonschema==4.25.1"])
        generated = self.out / "terminal_bench_organization_json_generator"
        self.assertTrue((generated / "environment/data/departments.csv").is_file())
        self.assertTrue((generated / "environment/data/employees.csv").is_file())
        self.assertTrue((generated / "environment/data/projects.csv").is_file())
        self.assertTrue((generated / "environment/schema.json").is_file())
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("import jsonschema", solution)
        self.assertNotIn("pip install jsonschema", solution)

    def test_materializes_math_dependency_tasks_without_runtime_installs(self):
        mixed = self.make_terminal_task(name="mixed-integer-programming")
        (mixed / "Dockerfile").write_text("FROM example\nCOPY question.mps /app\n")
        (mixed / "question.mps").write_text("NAME question\nENDATA\n")
        (mixed / "solution.sh").write_text(
            "#!/bin/bash\n"
            "pip install pulp==3.2.2\n"
            "cat > test_mip.py << EOF\n"
            "import pulp as pl\n"
            "solver = pl.PULP_CBC_CMD(msg=True, threads=16)\n"
            "EOF\n"
            "python test_mip.py\n"
        )
        result = mat.materialize_terminal_bench_task(
            mixed,
            self.out,
            replace=True,
            prepare_python_deps=False,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["python_deps"], ["pulp==3.2.2"])
        generated = self.out / "terminal_bench_mixed_integer_programming"
        self.assertTrue((generated / "environment/question.mps").is_file())
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("import pulp", solution)
        self.assertIn("timeLimit=60, gapRel=0.02", solution)
        self.assertIn("python3 test_mip.py", solution)
        self.assertNotIn("pip install pulp", solution)
        self.assertNotIn("python test_mip.py", solution)

        integral = self.make_terminal_task(name="multistep-definite-integral")
        (integral / "solution.sh").write_text(
            "#! /bin/bash\n"
            "set -euo pipefail\n"
            "# Create and activate virtual environment\n"
            "python3 -m venv venv\n"
            "source venv/bin/activate\n"
            "\n"
            "# install sympy library\n"
            "pip install --no-cache-dir sympy\n"
            "cat > integrate.py << 'EOF'\n"
            "import sympy\n"
            "EOF\n"
            "python integrate.py\n"
            "# leave the virtual environment\n"
            "deactivate\n"
        )
        result = mat.materialize_terminal_bench_task(
            integral,
            self.out,
            replace=True,
            prepare_python_deps=False,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["python_deps"], ["sympy==1.14.0"])
        solution = (self.out / "terminal_bench_multistep_definite_integral/solution/solve.sh").read_text()
        self.assertIn("import sympy", solution)
        self.assertIn("python3 integrate.py", solution)
        self.assertNotIn("python3 -m venv", solution)
        self.assertNotIn("pip install", solution)
        self.assertNotIn("deactivate", solution)

    def test_materializes_kv_store_grpc_with_verifier_service(self):
        task = self.make_terminal_task(name="kv-store-grpc")
        (task / "task.yaml").write_text(
            "instruction: |-\n"
            "  You need to build a simple KV store server using grpc.\n"
            "  1. Install grpcio (1.73.0) and grpcio-tools (1.73.0) python packages system-wide.\n"
            "  2. Run the service on port 5328.\n"
            "  5. Run the server.py file and keep it running in the background.\n"
            "difficulty: medium\n"
            "tags:\n"
            "  - coding\n"
            "parser_name: pytest\n"
            "max_agent_timeout_sec: 900.0\n"
            "max_test_timeout_sec: 180.0\n"
        )
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "# Install the grpcio and grpcio-tools packages with pinned versions for reproducibility\n"
            "pip install grpcio==1.73.0 grpcio-tools==1.73.0\n"
            "cat > kv-store.proto << EOF\n"
            "syntax = \"proto3\";\n"
            "EOF\n"
            "python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. kv-store.proto\n"
            "cat > server.py << EOF\n"
            "print('server')\n"
            "EOF\n"
            "# Run the server in the background so the script can continue\n"
            "python server.py &"
        )
        packages = ["grpcio==1.73.0", "grpcio-tools==1.73.0"]
        digest = mat.hashlib.sha256("\n".join(packages).encode("utf-8")).hexdigest()[:12]
        cache_root = self.root / "python_layers"
        existing = cache_root / f"terminal_bench_kv_store_grpc_py{mat.sys.version_info.major}{mat.sys.version_info.minor}_{digest}"
        (existing / "site-packages").mkdir(parents=True)
        (existing / "manifest.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "python": f"{mat.sys.version_info.major}.{mat.sys.version_info.minor}",
                    "packages": packages,
                }
            )
        )
        result = mat.materialize_terminal_bench_task(
            task,
            self.out,
            replace=True,
            python_layer_cache=cache_root,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["grpc_server"])
        self.assertEqual(result["python_deps"], packages)
        self.assertTrue(result["python_deps_ready"])
        generated = self.out / "terminal_bench_kv_store_grpc"
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.grpc_server]", task_toml)
        self.assertIn('phases = ["verifier"]', task_toml)
        self.assertIn("python3 /workspace/server.py", task_toml)
        self.assertIn("socket.create_connection(('127.0.0.1', 5328)", task_toml)
        self.assertIn('capsule_request = "capsule_request.json"', task_toml)
        instruction = (generated / "instruction.md").read_text()
        self.assertIn("offline grpcio", instruction)
        self.assertIn("the Corenius verifier will start it during grading", instruction)
        self.assertNotIn("keep it running in the background", instruction)
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("import grpc", solution)
        self.assertIn("python3 -m grpc_tools.protoc", solution)
        self.assertIn("python3 -m py_compile server.py", solution)
        self.assertNotIn("pip install grpcio", solution)
        self.assertNotIn("python server.py &", solution)
        request = json.loads((generated / "capsule_request.json").read_text())
        tree_layers = [layer for layer in request["layers"] if layer["kind"] == "tree"]
        self.assertEqual(tree_layers[0]["name"], "python_deps")
        self.assertEqual(tree_layers[0]["link_mode"], "hardlink")

    def test_materializes_form_filling_with_offline_rag_deps(self):
        task = self.make_terminal_task(name="form-filling")
        (task / "task.yaml").write_text(
            "instruction: |-\n"
            "  Create a simple RAG based tool for filling PDF forms.\n"
            "  - A fillable PDF form that can be downloaded from https://sample-files.com/downloads/documents/pdf/fillable-form.pdf and should be saved to `/app/fillable_form.pdf`\n"
            "  - Use the pypdf library for PDF analysis\n"
            "  - Use scikit-learn for implementing TF-IDF and similarity calculations\n"
            "difficulty: medium\n"
            "tags:\n"
            "  - information-retrieval\n"
            "parser_name: pytest\n"
            "max_agent_timeout_sec: 900.0\n"
            "max_test_timeout_sec: 180.0\n"
        )
        (task / "Dockerfile").write_text(
            "FROM example\n"
            "WORKDIR /app\n"
            "COPY resources/sample_resume.txt /app/documents/\n"
            "COPY resources/sample_application.txt /app/documents/\n"
            "COPY resources/sample_personal_info.txt /app/documents/\n"
        )
        (task / "resources").mkdir()
        for name in ("sample_resume.txt", "sample_application.txt", "sample_personal_info.txt"):
            (task / "resources" / name).write_text("Name: Ada Lovelace\nEmail: ada@example.test\n")
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "# Install pypdf for PDF form filling and testing\n"
            "pip install pypdf==5.6.1\n"
            "# Download the fillable form from the URL mentioned in the task\n"
            "echo \"Downloading fillable PDF form...\"\n"
            "wget https://sample-files.com/downloads/documents/pdf/fillable-form.pdf -O /app/fillable_form.pdf || {\n"
            "    echo \"Failed to download fillable form, creating a simple one instead\"\n"
            "    echo '%PDF-1.7' > /app/fillable_form.pdf\n"
            "}\n"
            "cat > /app/form_filler.py << 'EOF'\n"
            "import numpy\n"
            "import pypdf\n"
            "import sklearn\n"
            "EOF\n"
            "python3 /app/form_filler.py\n"
        )
        packages = ["pypdf==5.6.1", "scikit-learn==1.7.0", "numpy==2.3.1"]
        digest = mat.hashlib.sha256("\n".join(packages).encode("utf-8")).hexdigest()[:12]
        cache_root = self.root / "python_layers"
        existing = cache_root / f"terminal_bench_form_filling_py{mat.sys.version_info.major}{mat.sys.version_info.minor}_{digest}"
        (existing / "site-packages").mkdir(parents=True)
        (existing / "manifest.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "python": f"{mat.sys.version_info.major}.{mat.sys.version_info.minor}",
                    "packages": packages,
                }
            )
        )
        result = mat.materialize_terminal_bench_task(
            task,
            self.out,
            replace=True,
            python_layer_cache=cache_root,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["python_deps"], packages)
        self.assertTrue(result["python_deps_ready"])
        generated = self.out / "terminal_bench_form_filling"
        for name in ("sample_resume.txt", "sample_application.txt", "sample_personal_info.txt"):
            self.assertTrue((generated / "environment/documents" / name).is_file())
        instruction = (generated / "instruction.md").read_text()
        self.assertIn("benchmark runs are offline", instruction)
        self.assertIn("provided offline scikit-learn and NumPy", instruction)
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("import pypdf", solution)
        self.assertIn("import sklearn", solution)
        self.assertIn("Corenius offline placeholder form", solution)
        self.assertNotIn("pip install pypdf", solution)
        self.assertNotIn("wget https://sample-files.com", solution)
        request = json.loads((generated / "capsule_request.json").read_text())
        tree_layers = [layer for layer in request["layers"] if layer["kind"] == "tree"]
        self.assertEqual(tree_layers[0]["name"], "python_deps")
        self.assertEqual(tree_layers[0]["link_mode"], "hardlink")

    def test_skips_missing_solution(self):
        task = self.make_terminal_task()
        (task / "solution.sh").unlink()
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)
        self.assertEqual(result["status"], "skipped")
        self.assertIn("solution.sh", result["reason"])

    def test_skips_multi_service_by_default(self):
        task = self.make_terminal_task(compose="services:\n  client:\n  db:\n")
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)
        self.assertEqual(result["status"], "skipped")
        self.assertIn("multi-service", result["reason"])
        self.assertEqual(result["service_gap"]["kind"], "unsupported_compose")

    def test_materializes_home_server_https_with_rootfs_overlays(self):
        task = self.make_terminal_task(
            name="home-server-https",
            compose="services:\n  webserver:\n  google-api-mock:\n",
        )
        (task / "mock-api-server.py").write_text("from flask import Flask\napp = Flask(__name__)\n")
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "openssl x509 -req -in /root/.acme.sh/homeserver.local_ecc/homeserver.local.csr \\\n"
            "    -signkey /root/.acme.sh/homeserver.local_ecc/homeserver.local.key \\\n"
            "    -out /root/.acme.sh/homeserver.local_ecc/homeserver.local.cer \\\n"
            "    -days 365 \\\n"
            "    -extensions v3_req \\\n"
            "    -extfile <(cat <<EOF\n"
            "[v3_req]\n"
            "basicConstraints = CA:FALSE\n"
            "keyUsage = nonRepudiation, digitalSignature, keyEncipherment\n"
            "subjectAltName = @alt_names\n"
            "\n"
            "[alt_names]\n"
            "DNS.1 = homeserver.local\n"
            "EOF\n"
            ")\n"
        )
        rootfs = self.root / "home_rootfs"
        for rel in (
            "usr/sbin/nginx",
            "usr/bin/openssl",
            "usr/bin/curl",
            "usr/bin/netstat",
            "usr/bin/nslookup",
            "usr/bin/pgrep",
            "usr/bin/python3",
            "usr/lib/python3/dist-packages/flask/__init__.py",
            "usr/lib/python3/dist-packages/pytest/__init__.py",
            "usr/lib/python3/dist-packages/requests/__init__.py",
            "usr/lib/python3/dist-packages/urllib3/__init__.py",
        ):
            path = rootfs / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("")
        (rootfs / "usr/sbin").mkdir(exist_ok=True)
        (rootfs / "etc/nginx/sites-available").mkdir(parents=True)
        (rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {"env": ["PATH=/usr/bin:/usr/sbin"]},
        }))
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, rootfs=rootfs)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["google_api_mock", "nginx"])
        self.assertTrue(result["rootfs_ready"])
        self.assertEqual(result["python_deps"], [])
        generated = self.out / "terminal_bench_home_server_https"
        self.assertTrue((generated / "environment/.insula_overlays/var/www/html").is_dir())
        self.assertTrue((generated / "environment/.insula_overlays/etc/ddclient.conf").is_file())
        self.assertTrue((generated / "environment/.insula_overlays/etc/nginx/nginx.conf").is_file())
        self.assertTrue((generated / "environment/.insula_overlays/etc/nginx/sites-available/default").is_file())
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.google_api_mock]", task_toml)
        self.assertIn("[bench.services.nginx]", task_toml)
        self.assertIn('phases = ["agent", "verifier"]', task_toml)
        self.assertIn('phases = ["verifier"]', task_toml)
        self.assertIn('aliases = ["homeserver.local"]', task_toml)
        self.assertIn("nginx -g 'daemon off;'", task_toml)
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("/run/insula-tmp/homeserver-openssl-ext.cnf", solution)
        self.assertNotIn("<(cat <<EOF", solution)
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][0]["kind"], "rootfs")
        self.assertIn("/usr/sbin", request["layers"][0]["path"])
        self.assertEqual(request["policy"]["user"], "root")
        self.assertEqual(request["policy"]["cap_add"], ["CAP_NET_BIND_SERVICE"])
        overlays = request["policy"]["workspace_writable_overlays"]
        self.assertIn({"dest": "/var/www", "path": ".insula_overlays/var/www", "kind": "dir"}, overlays)
        self.assertIn({"dest": "/root/.acme.sh", "path": ".insula_overlays/root/.acme.sh", "kind": "dir"}, overlays)
        self.assertIn(
            {
                "dest": "/etc/nginx/nginx.conf",
                "path": ".insula_overlays/etc/nginx/nginx.conf",
                "kind": "file",
            },
            overlays,
        )
        self.assertIn(
            {
                "dest": "/etc/nginx/sites-available/default",
                "path": ".insula_overlays/etc/nginx/sites-available/default",
                "kind": "file",
            },
            overlays,
        )

    def test_materializes_acl_permissions_with_rootfs_identity_and_srv_overlay(self):
        task = self.make_terminal_task(name="acl-permissions-inheritance")
        rootfs = self.root / "acl_rootfs"
        for rel in (
            "usr/bin/bash",
            "usr/bin/chgrp",
            "usr/bin/chmod",
            "usr/bin/getfacl",
            "usr/bin/gpasswd",
            "usr/bin/python3",
            "usr/bin/setfacl",
            "usr/bin/stat",
            "usr/sbin/runuser",
            "usr/sbin/useradd",
            "usr/lib/python3/dist-packages/pytest/__init__.py",
        ):
            path = rootfs / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("")
        (rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {"env": ["PATH=/usr/bin:/usr/sbin"]},
        }))
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, rootfs=rootfs)
        self.assertEqual(result["status"], "materialized")
        self.assertTrue(result["rootfs_ready"])
        self.assertTrue(result["custom_policy"])
        generated = self.out / "terminal_bench_acl_permissions_inheritance"
        self.assertIn('capsule_request = "capsule_request.json"', (generated / "task.toml").read_text())
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][0]["kind"], "rootfs")
        self.assertEqual(request["policy"]["user"], "root")
        self.assertIn(
            {"dest": "/srv", "path": ".insula_overlays/srv", "kind": "dir"},
            request["policy"]["workspace_writable_overlays"],
        )
        identity = request["policy"]["identity"]
        self.assertIn({"name": "research", "gid": 2000, "members": ["alice", "bob"]}, identity["groups"])
        self.assertIn({"name": "mallory", "uid": 2003, "gid": 2003, "home": "/home/mallory", "shell": "/bin/bash"}, identity["users"])

    def test_skips_terminal_bench_task_with_unavailable_public_source(self):
        task = self.make_terminal_task(name="fix-git")
        (task / "setup.sh").write_text(
            "git clone https://github.com/TheMikeMerrill/personal-site.git\n"
            "cd personal-site\n"
            "git reset --hard d7d3e4b\n"
        )
        generated = self.out / "terminal_bench_fix_git"
        generated.mkdir(parents=True)
        (generated / "stale.txt").write_text("stale\n")
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)
        self.assertEqual(result["status"], "skipped")
        self.assertEqual(result["materialization_blocker"]["kind"], "source_unavailable")
        self.assertIn("personal-site.git", result["reason"])
        self.assertFalse(generated.exists())

    def test_materializes_filter_js_from_html_with_browser_rootfs_and_offline_vectors(self):
        task = self.make_terminal_task(name="filter-js-from-html")
        (task / "tests/test_outputs.py").write_text(
            "from pathlib import Path\n"
            "import urllib.request\n"
            "import zipfile\n"
            "from selenium import webdriver\n"
            "from bs4 import BeautifulSoup\n"
            "\n"
            "def download_attack_vectors():\n"
            "    \"\"\"Download XSS attack vectors from HTML sanitizer testbed.\"\"\"\n"
            f"    testcases_dir = Path(\"{mat.UPSTREAM_SYSTEM_TMP}/xss_testcases\")\n"
            "    testcases_dir.mkdir(exist_ok=True)\n"
            "\n"
            "    # Download the repository as a zip file\n"
            "    url = \"https://github.com/davidwagner/html-sanitizer-testbed/archive/refs/heads/master.zip\"\n"
            "    zip_path = testcases_dir / \"testbed.zip\"\n"
            "\n"
            "    urllib.request.urlretrieve(url, zip_path)\n"
            "\n"
            "    # Extract the zip file\n"
            "    with zipfile.ZipFile(zip_path, \"r\") as zip_ref:\n"
            "        zip_ref.extractall(testcases_dir)\n"
            "\n"
            "    # Return path to testcases\n"
            "    return testcases_dir / \"html-sanitizer-testbed-master\" / \"testcases\"\n"
        )
        fixture_archive = self.root / "xss.zip"
        fixture_root = self.root / "archive/html-sanitizer-testbed-master/testcases"
        fixture_root.mkdir(parents=True)
        (fixture_root / "t0.html").write_text("<script>alert(1)</script>\n")
        with zipfile.ZipFile(fixture_archive, "w") as archive:
            archive.write(fixture_root / "t0.html", "html-sanitizer-testbed-master/testcases/t0.html")
        old_download = mat.download_public_archive
        mat.download_public_archive = lambda _url: fixture_archive
        try:
            rootfs = self.root / "browser_rootfs"
            for rel in (
                "usr/bin/chromedriver",
                "usr/bin/chromium",
                "usr/bin/python",
                "usr/bin/python3",
                "usr/lib/python3/dist-packages/bs4/__init__.py",
                "usr/lib/python3/dist-packages/pytest/__init__.py",
                "usr/lib/python3/dist-packages/selenium/__init__.py",
            ):
                path = rootfs / rel
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("")
            result = mat.materialize_terminal_bench_task(task, self.out, replace=True, rootfs=rootfs)
        finally:
            mat.download_public_archive = old_download
        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_filter_js_from_html"
        self.assertEqual(
            (generated / "environment/test_deps/xss_testcases/t0.html").read_text(),
            "<script>alert(1)</script>\n",
        )
        tests = (generated / "tests/test_outputs.py").read_text()
        self.assertIn('return Path("/app/test_deps/xss_testcases")', tests)
        self.assertNotIn("urllib.request.urlretrieve", tests)
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][0]["kind"], "rootfs")

    def test_materializes_break_filter_js_from_html_with_browser_rootfs(self):
        task = self.make_terminal_task(name="break-filter-js-from-html")
        rootfs = self.root / "browser_rootfs"
        for rel in (
            "usr/bin/chromedriver",
            "usr/bin/chromium",
            "usr/bin/python",
            "usr/bin/python3",
            "usr/lib/python3/dist-packages/bs4/__init__.py",
            "usr/lib/python3/dist-packages/pytest/__init__.py",
            "usr/lib/python3/dist-packages/selenium/__init__.py",
        ):
            path = rootfs / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("")
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, rootfs=rootfs)
        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_break_filter_js_from_html"
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][0]["kind"], "rootfs")

    def test_materializes_supported_terminal_bench_sidecar(self):
        task = self.make_terminal_task(name="simple-web-scraper", compose="services:\n  client:\n  server:\n")
        (task / "server").mkdir()
        (task / "server/mock_website.py").write_text("from http.server import HTTPServer, SimpleHTTPRequestHandler\nHTTPServer(('', 8000), SimpleHTTPRequestHandler).serve_forever()\n")
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, prepare_python_deps=False)
        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_simple_web_scraper"
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.server]", task_toml)
        self.assertIn('aliases = ["server"]', task_toml)
        self.assertTrue((generated / "environment/server/mock_website.py").is_file())
        self.assertFalse((generated / "capsule_request.json").exists())

    def test_materializes_supported_terminal_bench_sqlite_api_sidecar(self):
        task = self.make_terminal_task(name="simple-sheets-put", compose="services:\n  client:\n  api:\n  db:\n")
        (task / "api/app").mkdir(parents=True)
        (task / "api/app/main.py").write_text("from fastapi import FastAPI\napp = FastAPI()\n")
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, prepare_python_deps=False)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["api"])
        self.assertIn("fastapi==0.104.1", result["python_deps"])
        generated = self.out / "terminal_bench_simple_sheets_put"
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.api]", task_toml)
        self.assertIn("sqlite:////workspace/simple_sheets.sqlite", task_toml)
        self.assertIn('aliases = ["api"]', task_toml)
        self.assertTrue((generated / "environment/api/app/main.py").is_file())

    def test_materializes_supported_terminal_bench_debug_program_sidecar(self):
        task = self.make_terminal_task(name="debug-long-program", compose="services:\n  client:\n  program:\n")
        (task / "debug_server.py").write_text(
            "from flask import Flask, jsonify, request\n"
            "from flask_cors import CORS\n"
            "\n"
            "NUM_LINES = 4325\n"
            "PROB_LINES = [102, 2345]\n"
            "app = Flask(__name__)\n"
            "CORS(app)\n"
            "\n"
            "\n"
            "class ProgramState:\n"
            "    def __init__(self):\n"
            "        self.num_lines = NUM_LINES\n"
            "        self.buggy_lines = PROB_LINES\n"
            "        self.fix_attempts = []\n"
            "\n"
            "        self.checkpoint_cnt = 0\n"
            "        self.fix_cnt = 0\n"
            "\n"
            "    def status(self):\n"
            "        return {\"fix_attempts\": self.fix_attempts, \"checkpoint_cnt\": self.checkpoint_cnt, \"fix_cnt\": self.fix_cnt}\n"
            "\n"
            "    def checkpoint(self, line):\n"
            "        self.checkpoint_cnt += 1\n"
            "\n"
            "        if line > self.num_lines:\n"
            "            return (False, \"Error\")\n"
            "        return (True, \"Ok\")\n"
            "\n"
            "    def fix(self, line):\n"
            "        self.fix_cnt += 1\n"
            "        self.fix_attempts.append(line)\n"
            "        return (True, \"Ok\")\n"
        )
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, prepare_python_deps=False)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["program"])
        self.assertIn("flask==3.0.3", result["python_deps"])
        self.assertIn("flask-cors==4.0.1", result["python_deps"])
        generated = self.out / "terminal_bench_debug_long_program"
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.program]", task_toml)
        self.assertIn("python3 debug_server.py", task_toml)
        self.assertIn('aliases = ["program"]', task_toml)
        self.assertFalse((generated / "capsule_request.json").exists())
        server = (generated / "environment/debug_server.py").read_text()
        self.assertIn('STATE_PATH = Path("/workspace/.debug_long_program_state.json")', server)
        self.assertIn("self.load()", server)
        self.assertIn("self.save()", server)

    def test_materializes_supported_terminal_bench_ancient_puzzle_sidecar(self):
        task = self.make_terminal_task(name="ancient-puzzle", compose="services:\n  client:\n  decryptor:\n")
        (task / "decryptor_server.py").write_text("SECRET_INCANTATION = 'hidden'\n")
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "PASSCODE=12345\n"
            "# Unzip using the computed passcode\n"
            "7z x artifacts/final_clue.zip -p\"$PASSCODE\" -aoa > /dev/null\n"
            "INCANTATION=$(<sealed_incantation.txt)\n"
        )
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, prepare_python_deps=False)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["decryptor"])
        self.assertEqual(result["extra_layers"], ["ancient_puzzle_decryptor"])
        self.assertIn("pyzipper==0.3.6", result["python_deps"])
        generated = self.out / "terminal_bench_ancient_puzzle"
        self.assertFalse((generated / "environment/decryptor_server.py").exists())
        self.assertTrue((generated / "service_sources/ancient_puzzle/decryptor_server.py").is_file())
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.decryptor]", task_toml)
        self.assertIn("/opt/ancient_puzzle_decryptor/decryptor_server.py", task_toml)
        self.assertIn('aliases = ["decryptor"]', task_toml)
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("import pyzipper", solution)
        self.assertIn("INCANTATION=$(cat sealed_incantation.txt)", solution)
        self.assertNotIn("7z x", solution)
        self.assertNotIn("$(<sealed_incantation.txt)", solution)
        request = json.loads((generated / "capsule_request.json").read_text())
        tree_layers = [layer for layer in request["layers"] if layer["kind"] == "tree"]
        self.assertEqual(tree_layers[0]["name"], "ancient_puzzle_decryptor")
        self.assertEqual(tree_layers[0]["source"], "service_sources/ancient_puzzle")
        self.assertEqual(tree_layers[0]["file_binds"][0]["dest"], "/opt/ancient_puzzle_decryptor/decryptor_server.py")

    def test_materializes_supported_terminal_bench_maze_sidecar(self):
        task = self.make_terminal_task(name="interactive-maze-game", compose="services:\n  client:\n  maze:\n")
        (task / "maze_connectivity.json").write_text("{}\n")
        (task / "maze_server.py").write_text(
            "import json\n"
            "import os\n"
            "\n"
            "from flask import Flask, jsonify, request\n"
            "from flask_cors import CORS\n"
            "\n"
            "# Get maze size from environment variable\n"
            "MAZE_SIZE_N = int(os.environ.get(\"MAZE_SIZE_N\", \"9\"))\n"
            "\n"
            "\n"
            "class GameState:\n"
            "    def __init__(self):\n"
            "        self.current_position = [1, 1]\n"
            "        self.visited_positions = {}\n"
            "        self.position_history = []\n"
            "        self.game_over = False\n"
            "        self.game_won = False\n"
            "        self.failure_reason = \"\"\n"
            "        self.steps_taken = 0\n"
            "        self.failure = 0\n"
            "        self.game_success = 0  # Track if game was completed successfully\n"
            "\n"
            "    def load_maze(self, maze_file=\"/app/maze_connectivity.json\"):\n"
            "        return True\n"
            "\n"
            "    def reset_game(self):\n"
            "        \"\"\"Reset the game state for a new session\"\"\"\n"
            "        self.current_position = [1, 1]\n"
            "        self.visited_positions = {(1, 1): 1}\n"
            "        self.position_history = [[1, 1]]\n"
            "        self.game_over = False\n"
            "        self.game_won = False\n"
            "        self.failure_reason = \"\"\n"
            "        self.steps_taken = 0\n"
            "        self.failure = 0  # Reset failure tracking\n"
            "        self.game_success = 0  # Reset success tracking\n"
            "\n"
            "    def get_current_connectivity(self):\n"
            "        return [1, 0, 0, 0]\n"
            "\n"
            "    def make_move(self, direction):\n"
            "        if direction < 0:\n"
            "            self.failure = 1  # Record failure\n"
            "            return {\"success\": False, \"reason\": self.failure_reason}\n"
            "        if self.current_position == [self.n, self.n]:\n"
            "            # Just return success with game_won indication, but don't set game_over\n"
            "            return {\n"
            "                \"success\": True,\n"
            "            }\n"
            "        return {\"success\": True, \"position\": self.current_position}\n"
            "\n"
            "    def get_game_status(self):\n"
            "        return {\n"
            "            \"current_position\": self.current_position,\n"
            "            \"visited_positions\": {f\"{k[0]},{k[1]}\": v for k, v in self.visited_positions.items()},\n"
            "            \"position_history\": self.position_history,\n"
            "            \"game_over\": self.game_over,\n"
            "            \"game_won\": self.game_won,\n"
            "            \"failure_reason\": self.failure_reason,\n"
            "            \"steps_taken\": self.steps_taken,\n"
            "            \"failure\": self.failure,\n"
            "            \"game_success\": self.game_success,\n"
            "        }\n"
            "\n"
            "game_state = GameState()\n"
            "\n"
            "def finish_notification():\n"
            "    if game_state.current_position == [game_state.n, game_state.n]:\n"
            "        game_state.failure_reason = \"Successfully reached destination\"\n"
            "\n"
            "        return jsonify({\"status\": \"Congratulations, destination reached\"})\n"
            "    else:\n"
            "        game_state.failure = 1\n"
            "        game_state.failure_reason = \"Not at destination\"\n"
            "        return jsonify(\n"
        )
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, prepare_python_deps=False)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["maze"])
        self.assertIn("flask==3.0.3", result["python_deps"])
        generated = self.out / "terminal_bench_interactive_maze_game"
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.maze]", task_toml)
        self.assertIn("python3 maze_server.py", task_toml)
        self.assertIn("MAZE_SIZE_N=9", task_toml)
        self.assertIn('aliases = ["maze"]', task_toml)
        server = (generated / "environment/maze_server.py").read_text()
        self.assertIn('STATE_PATH = "/workspace/.interactive_maze_game_state.json"', server)
        self.assertIn("self.restored_state = True", server)
        self.assertIn("def save_state(self):", server)
        self.assertIn("if self.restored_state:", server)

    def test_materializes_supported_terminal_bench_find_restaurant_sidecar(self):
        task = self.make_terminal_task(name="find-restaurant", compose="services:\n  client:\n  services:\n")
        (task / "solution.sh").write_text(
            "#!/bin/bash\n"
            "echo -e \"taylor\\nfood\\nhang up\" | ./call_august\n"
            "echo -e \"$PHRASE\\nfood\\nhang up\" | ./call_taylor\n"
            "echo -e \"call $RESTAURANT\\nno\\nquit\" | ./call_restaurant\n"
            "echo -e \"call $RESTAURANT\\nyes\\n$COMPANY\\nquit\" | ./call_restaurant\n"
        )
        (task / "client").mkdir()
        (task / "client/call_august").write_text("#!/bin/sh\necho august\n")
        (task / "client/call_august").chmod(0o755)
        (task / "services").mkdir()
        for name in ("august_service.py", "taylor_service.py", "restaurant_service.py", "schedule_service.py"):
            (task / "services" / name).write_text("print('service')\n")
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, prepare_python_deps=False)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["services"])
        self.assertEqual(result["extra_layers"], ["find_restaurant_services"])
        self.assertIn("flask==3.1.0", result["python_deps"])
        generated = self.out / "terminal_bench_find_restaurant"
        self.assertTrue((generated / "environment/call_august").is_file())
        self.assertFalse((generated / "environment/client").exists())
        self.assertFalse((generated / "environment/services").exists())
        self.assertTrue((generated / "service_sources/find_restaurant/services/august_service.py").is_file())
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.services]", task_toml)
        self.assertIn("/opt/find_restaurant_services/august_service.py", task_toml)
        self.assertIn('aliases = ["services"]', task_toml)
        self.assertIn('capsule_request = "capsule_request.json"', task_toml)
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn('printf "%s\\n" "taylor" "food" "hang up" | ./call_august', solution)
        self.assertNotIn("echo -e", solution)
        request = json.loads((generated / "capsule_request.json").read_text())
        tree_layers = [layer for layer in request["layers"] if layer["kind"] == "tree"]
        self.assertEqual(tree_layers[0]["name"], "find_restaurant_services")
        self.assertEqual(tree_layers[0]["source"], "service_sources/find_restaurant")
        self.assertEqual(tree_layers[0]["file_binds"][0]["dest"], "/opt/find_restaurant_services")

    def test_materializes_supported_terminal_bench_celery_redis_rce_sidecar(self):
        task = self.make_terminal_task(
            name="security-celery-redis-rce",
            compose="services:\n  client:\n  redis:\n  celery:\n",
        )
        (task / "solution.sh").write_text(
            "#!/bin/sh\n"
            f"python3 exploit.py redis 'touch {mat.UPSTREAM_SYSTEM_TMP}/security_mark3.txt'\n"
            "echo done\n"
        )
        original_monitor = f'MONITOR_DIR = "{mat.UPSTREAM_SYSTEM_TMP}"\n'
        original_verifier = f'MARKER = "{mat.UPSTREAM_SYSTEM_TMP}/security_mark3.txt"\n'
        (task / "file_monitor.py").write_text(original_monitor)
        (task / "tests/test_outputs.py").write_text(original_verifier)
        (task / "task.yaml").write_text(
            (task / "task.yaml").read_text().replace("Write the report.", f"Create {mat.UPSTREAM_SYSTEM_TMP}/security_mark3.txt via Redis.")
        )
        (task / "tasks.py").write_text("print('tasks')\n")
        rootfs = self.root / "celery_rootfs"
        for rel in (
            "usr/bin/redis-server",
            "usr/bin/python3",
            "usr/lib/python3/dist-packages/pytest/__init__.py",
            "usr/lib/python3/dist-packages/redis/__init__.py",
            "usr/lib/python3/dist-packages/requests/__init__.py",
        ):
            path = rootfs / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("")
        (rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {"env": ["PATH=/usr/bin"]},
        }))
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, rootfs=rootfs)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["celery_redis"])
        self.assertEqual(result["extra_layers"], ["security_celery_redis_rce_services"])
        self.assertEqual(result["python_deps"], [])
        generated = self.out / "terminal_bench_security_celery_redis_rce"
        self.assertFalse((generated / "environment/file_monitor.py").exists())
        self.assertFalse((generated / "environment/tasks.py").exists())
        self.assertTrue((generated / "environment/.insula_overlays/security_celery_redis_rce").is_dir())
        self.assertTrue((generated / "service_sources/security_celery_redis_rce/file_monitor.py").is_file())
        self.assertTrue((generated / "service_sources/security_celery_redis_rce/vulnerable_redis_worker.py").is_file())
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.celery_redis]", task_toml)
        self.assertIn("redis-server --bind 127.0.0.1 --port 6379", task_toml)
        self.assertIn('aliases = ["redis", "celery"]', task_toml)
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn(f"python3 exploit.py redis 'touch {mat.SECURITY_RCE_STATE}/security_mark3.txt'\nsleep 1\n", solution)
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][0]["kind"], "rootfs")
        tree_layers = [layer for layer in request["layers"] if layer["kind"] == "tree"]
        self.assertEqual(tree_layers[0]["name"], "security_celery_redis_rce_services")
        self.assertEqual(tree_layers[0]["file_binds"][0]["dest"], "/opt/security_celery_redis_rce")
        self.assertIn({"dest": mat.SECURITY_RCE_STATE, "path": ".insula_overlays/security_celery_redis_rce", "kind": "dir"}, request["policy"]["workspace_writable_overlays"])
        # Monitor, exploit and verifier agree on the same persisted marker,
        # including both copies of verifier code used by the harness.
        monitor = {}
        exec((generated / "service_sources/security_celery_redis_rce/file_monitor.py").read_text(), monitor)
        marker = str(Path(monitor["MONITOR_DIR"]) / "security_mark3.txt")
        for relative in ("tests/test_outputs.py", "environment/tests/test_outputs.py"):
            verifier = {}
            exec((generated / relative).read_text(), verifier)
            self.assertEqual(verifier["MARKER"], marker)
        self.assertIn(f"touch {marker}", solution)
        self.assertIn(marker, (generated / "instruction.md").read_text())
        self.assertIn("Monitored exploit state is relocated", task_toml)
        self.assertEqual((task / "file_monitor.py").read_text(), original_monitor)
        self.assertEqual((task / "tests/test_outputs.py").read_text(), original_verifier)


    def test_materializes_supported_terminal_bench_minio_rootfs_service(self):
        task = self.make_terminal_task(
            name="security-vulhub-minio",
            compose="services:\n  client:\n  node1:\n  node2:\n  node3:\n",
        )
        (task / "solution.sh").write_text(
            "#!/bin/sh\n"
            "apt-get update\n"
            "apt-get install -y curl\n"
            "\n"
            "curl -X POST http://node1:9000/minio/bootstrap/v1/verify --data ''\n"
            "echo 'Password: minioadmin-vulhub' > /app/result.txt\n"
            "echo 'Username: minioadmin' >> /app/result.txt\n"
        )
        rootfs = self.root / "minio_rootfs"
        for rel in (
            "opt/bin/minio",
            "usr/bin/curl",
            "usr/bin/python3",
            "usr/lib/python3/dist-packages/_pytest/__init__.py",
        ):
            path = rootfs / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("")
        (rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {"env": ["PATH=/opt/bin:/usr/bin"]},
        }))
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, rootfs=rootfs)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["minio"])
        self.assertTrue(result["rootfs_ready"])
        generated = self.out / "terminal_bench_security_vulhub_minio"
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.minio]", task_toml)
        self.assertIn("MINIO_ROOT_USER=minioadmin", task_toml)
        self.assertIn('aliases = ["node1", "node2", "node3"]', task_toml)
        self.assertIn('phases = ["agent"]', task_toml)
        solution = (generated / "solution/solve.sh").read_text()
        self.assertIn("command -v curl >/dev/null", solution)
        self.assertNotIn("apt-get", solution)
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][0]["kind"], "rootfs")
        self.assertEqual(request["layers"][0]["source"], str(rootfs.resolve()))
        self.assertIn("/opt/bin", request["layers"][0]["path"])

    def test_materializes_supported_terminal_bench_spring_isolated_service(self):
        task = self.make_terminal_task(
            name="spring-messaging-vul",
            compose="services:\n  client:\n  spring:\n",
        )
        (task / "task-deps").mkdir()
        (task / "task-deps/server-secret1.txt").write_text("secret1\n")
        (task / "task-deps/server-secret2.txt").write_text("secret2\n")
        rootfs = self.root / "spring_main_rootfs"
        for rel in (
            "usr/bin/curl",
            "usr/bin/python3",
            "usr/lib/python3/dist-packages/pytest/__init__.py",
            "usr/lib/python3/dist-packages/requests/__init__.py",
        ):
            path = rootfs / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            if "." in path.name:
                path.write_text("")
            else:
                path.mkdir(exist_ok=True)
        (rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {"env": ["PATH=/usr/bin"]},
        }))
        service_image = mat.terminal_bench_service_rootfs_image("spring-messaging-vul")
        service_rootfs = self.root / (mat.rootfs_prefix(service_image) + "_test")
        for rel in (
            "websocket-0.0.1-SNAPSHOT.jar",
            "server",
            "usr/bin/java",
        ):
            path = service_rootfs / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            if "." in path.name:
                path.write_text("")
            else:
                path.mkdir(exist_ok=True)
        (service_rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {"env": ["PATH=/usr/bin"]},
        }))
        result = mat.materialize_terminal_bench_task(
            task,
            self.out,
            replace=True,
            rootfs=rootfs,
            rootfs_root=self.root,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["spring"])
        self.assertTrue(result["rootfs_ready"])
        generated = self.out / "terminal_bench_spring_messaging_vul"
        self.assertFalse((generated / "environment/task-deps").exists())
        self.assertTrue((generated / "service_sources/spring_messaging_vul/server/secret1.txt").is_file())
        self.assertFalse((generated / "service_sources/spring_messaging_vul/server/server-secret1.txt").exists())
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.spring]", task_toml)
        self.assertIn('capsule_request = "spring_service_capsule_request.json"', task_toml)
        self.assertIn('net = "host"', task_toml)
        self.assertIn('ro_workspace = true', task_toml)
        self.assertIn("[bench.services.spring.env]", task_toml)
        self.assertIn('JAVA_TOOL_OPTIONS = "-Djava.io.tmpdir=/run/insula-tmp"', task_toml)
        self.assertIn('[agent]\ntimeout_sec = 900\nnet = "host"', task_toml)
        self.assertIn(f"service_docker_image = {json.dumps(service_image)}", task_toml)
        self.assertIn(f"service_rootfs = {json.dumps(str(service_rootfs.resolve()))}", task_toml)
        agent_request = json.loads((generated / "capsule_request.json").read_text())
        self.assertFalse(any(
            layer.get("name") == "spring_messaging_vul_secrets"
            for layer in agent_request["layers"]
        ))
        service_request = json.loads((generated / "spring_service_capsule_request.json").read_text())
        self.assertEqual(service_request["layers"][0]["kind"], "rootfs")
        self.assertEqual(service_request["layers"][0]["source"], str(service_rootfs.resolve()))
        tree_layers = [layer for layer in service_request["layers"] if layer["kind"] == "tree"]
        self.assertEqual(tree_layers[0]["name"], "spring_messaging_vul_secrets")
        self.assertEqual(tree_layers[0]["file_binds"][0]["dest"], "/server")

    def test_materializes_supported_terminal_bench_sql_injection_sidecar(self):
        task = self.make_terminal_task(
            name="sql-injection-attack",
            compose="services:\n  client:\n  vulnerable_service:\n  master_service:\n  db_init:\n",
        )
        (task / "vulnerable_login.py").write_text("print('vulnerable')\n")
        (task / "services").mkdir()
        (task / "services/init_db.py").write_text("print('init')\n")
        (task / "services/master_service.py").write_text("print('master')\n")
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, prepare_python_deps=False)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["sql_injection_services"])
        self.assertEqual(result["extra_layers"], ["sql_injection_services"])
        self.assertIn("flask==3.0.3", result["python_deps"])
        generated = self.out / "terminal_bench_sql_injection_attack"
        self.assertTrue((generated / "environment/vulnerable_login.py").is_file())
        self.assertFalse((generated / "environment/services").exists())
        self.assertTrue((generated / "service_sources/sql_injection/services/init_db.py").is_file())
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.sql_injection_services]", task_toml)
        self.assertIn("/opt/sql_injection_services/init_db.py", task_toml)
        self.assertIn('aliases = ["master_service", "vulnerable_service"]', task_toml)
        request = json.loads((generated / "capsule_request.json").read_text())
        tree_layers = [layer for layer in request["layers"] if layer["kind"] == "tree"]
        self.assertEqual(tree_layers[0]["name"], "sql_injection_services")
        self.assertEqual(tree_layers[0]["source"], "service_sources/sql_injection")
        self.assertEqual(tree_layers[0]["file_binds"][0]["dest"], "/opt/sql_injection_services")

    def test_materializes_supported_terminal_bench_postgres_rootfs_sidecar(self):
        task = self.make_terminal_task(name="postgres-csv-clean", compose="services:\n  client:\n  db:\n")
        rootfs = self.root / "postgres_rootfs"
        for rel in (
            "usr/lib/postgresql/16/bin/postgres",
            "usr/lib/postgresql/16/bin/initdb",
            "usr/lib/postgresql/16/bin/createdb",
            "usr/lib/postgresql/16/bin/psql",
            "usr/bin/python3",
            "usr/lib/python3/dist-packages/psycopg2/__init__.py",
            "usr/lib/python3/dist-packages/pytest/__init__.py",
        ):
            path = rootfs / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("")
        (rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {"env": ["PATH=/usr/bin:/usr/lib/postgresql/16/bin"]},
        }))
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True, rootfs=rootfs)
        self.assertEqual(result["status"], "materialized")
        self.assertEqual(result["services"], ["db"])
        self.assertTrue(result["rootfs_ready"])
        generated = self.out / "terminal_bench_postgres_csv_clean"
        task_toml = (generated / "task.toml").read_text()
        self.assertIn("[bench.services.db]", task_toml)
        self.assertIn("--encoding=UTF8", task_toml)
        self.assertIn("CREATE EXTENSION IF NOT EXISTS pg_stat_statements", task_toml)
        self.assertIn("pg_ctl -D /workspace/pgdata -m fast -w stop", task_toml)
        self.assertIn('PGCLIENTENCODING = "UTF8"', task_toml)
        self.assertIn('PGHOST = "db"', task_toml)
        self.assertIn("[agent.env]", task_toml)
        self.assertIn('DATABASE_URL = "postgresql://postgres:postgres@db:5432/customers_db"', task_toml)
        self.assertIn(f"rootfs = {json.dumps(str(rootfs.resolve()))}", task_toml)
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][0]["kind"], "rootfs")
        self.assertEqual(request["layers"][0]["source"], str(rootfs.resolve()))
        self.assertIn("/usr/lib/postgresql/16/bin", request["layers"][0]["path"])
        self.assertEqual(request["policy"]["workspace_aliases"], ["/app"])
        self.assertTrue((generated / "environment/data").is_symlink())

    def test_replace_removes_stale_directory_when_task_is_now_skipped(self):
        task = self.make_terminal_task()
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)
        self.assertEqual(result["status"], "materialized")
        generated = self.out / "terminal_bench_analyze_access_logs"
        self.assertTrue(generated.exists())
        (task / "solution.sh").unlink()
        result = mat.materialize_terminal_bench_task(task, self.out, replace=True)
        self.assertEqual(result["status"], "skipped")
        self.assertFalse(generated.exists())

    def test_cli_writes_manifest(self):
        self.make_terminal_task()
        code = mat.main(["--source-root", str(self.source), "--out-root", str(self.out), "--all", "--replace"])
        self.assertEqual(code, 0)
        manifest = json.loads((self.out / "materialization_manifest.json").read_text())
        self.assertEqual(manifest["counts"]["materialized"], 1)

    def test_materializes_deep_swe_task_with_task_local_rootfs_request(self):
        repo, base_commit = self.make_target_repo()
        task = self.make_deep_swe_task(repo, base_commit)
        rootfs = self.root / "rootfs"
        (rootfs / "usr/bin").mkdir(parents=True)
        (rootfs / "root/go/bin").mkdir(parents=True)
        (rootfs / "usr/local/go/bin").mkdir(parents=True)
        (rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {"env": ["PATH=/root/go/bin:/usr/local/go/bin:/usr/bin", "GOPATH=/root/go"]},
        }))
        result = mat.materialize_deep_swe_task(
            task,
            self.out,
            replace=True,
            repo_cache_root=self.root / "repo_cache",
            rootfs=rootfs,
        )
        self.assertEqual(result["status"], "materialized")
        self.assertTrue(result["rootfs_ready"])
        generated = self.out / "deep_swe_tiny_task"
        self.assertEqual((generated / "environment/value.txt").read_text(), "base\n")
        self.assertTrue((generated / "environment/.git").is_dir())
        self.assertTrue((generated / "verifier_tests/corenius_verify.py").is_file())
        self.assertIn("/logs/artifacts/model.patch", (generated / "verifier_tests/corenius_verify.py").read_text())
        self.assertIn("/run/insula-tmp/solution/solution.patch", (generated / "solution/solve.sh").read_text())
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][0]["kind"], "rootfs")
        self.assertIn("/root/go/bin", request["layers"][0]["path"])
        self.assertEqual(request["layers"][0]["env"]["GOPATH"], "/root/go")
        self.assertEqual(request["policy"]["workspace_path_aliases"]["/tests"], "verifier_tests")
        self.assertEqual(request["policy"]["workspace_path_aliases"]["/logs"], "verifier_logs")
        self.assertIn('capsule_request = "capsule_request.json"', (generated / "task.toml").read_text())

    def test_deep_swe_cli_manifest_records_missing_rootfs(self):
        repo, base_commit = self.make_target_repo()
        self.make_deep_swe_task(repo, base_commit)
        code = mat.main([
            "--suite", "deep_swe",
            "--source-root", str(self.source),
            "--out-root", str(self.out),
            "--repo-cache-root", str(self.root / "repo_cache"),
            "--all",
            "--replace",
        ])
        self.assertEqual(code, 0)
        manifest = json.loads((self.out / "materialization_manifest.json").read_text())
        self.assertEqual(manifest["counts"]["materialized"], 1)
        self.assertFalse(manifest["tasks"][0]["rootfs_ready"])
        generated = self.out / "deep_swe_tiny_task"
        self.assertFalse((generated / "capsule_request.json").exists())
        self.assertIn('capsule = "deep_swe"', (generated / "task.toml").read_text())

    def test_materializes_swe_bench_task_from_full_row(self):
        repo, base_commit = self.make_target_repo()
        row = self.make_swe_bench_row(repo, base_commit)
        rootfs = self.root / "swe_rootfs"
        (rootfs / "opt/miniconda3/bin").mkdir(parents=True)
        (rootfs / "insula-rootfs-manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "config": {"env": ["PATH=/opt/miniconda3/bin:/usr/bin"]},
        }))
        result = mat.materialize_swe_bench_task(
            row,
            self.out,
            dataset="SWE-bench/SWE-bench_Lite",
            split="dev",
            replace=True,
            repo_cache_root=self.root / "repo_cache",
            rootfs=rootfs,
        )
        self.assertEqual(result["status"], "materialized")
        generated = self.out / "swe_bench_local_tiny_1"
        self.assertEqual((generated / "environment/value.txt").read_text(), "base\n")
        self.assertIn("Fix the tiny SWE-bench task.", (generated / "instruction.md").read_text())
        self.assertTrue((generated / "verifier_tests/corenius_verify.py").is_file())
        metadata = json.loads((generated / "verifier_tests/metadata.json").read_text())
        self.assertEqual(metadata["fail_to_pass"], ["test_value.py::test_value"])
        self.assertIn("assert open('value.txt')", (generated / "verifier_tests/test.patch").read_text())
        self.assertIn("/run/insula-tmp/solution/solution.patch", (generated / "solution/solve.sh").read_text())
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["policy"]["workspace_aliases"], ["/testbed"])
        self.assertEqual(request["policy"]["workspace_path_aliases"]["/logs"], "verifier_logs")
        self.assertIn("/opt/miniconda3/bin", request["layers"][0]["path"])
        task_toml = (generated / "task.toml").read_text()
        self.assertIn('capsule = "swe_bench"', task_toml)
        self.assertIn('capsule_request = "capsule_request.json"', task_toml)

    def test_materializes_program_bench_task_with_branch_blob_and_rootfs(self):
        task, rootfs, blob_cache = self.make_program_bench_task()
        result = mat.materialize_program_bench_task(
            task,
            self.out,
            replace=True,
            rootfs=rootfs,
            blob_cache_root=blob_cache,
            branches=["79b69dd3fd98"],
        )
        self.assertEqual(result["status"], "materialized")
        generated = self.out / "program_bench_abishekvashok_cmatrix_5c082c6"
        self.assertEqual((generated / "environment/README.md").read_text(), "Run ./executable to inspect behavior.\n")
        self.assertEqual((rootfs / "workspace/executable").stat().st_mode & 0o777, 0o111)
        self.assertEqual((generated / "environment/executable").stat().st_mode & 0o777, 0o111)
        self.assertTrue((generated / "verifier_tests/branches/79b69dd3fd98.tar.gz").is_file())
        self.assertTrue((generated / "solution/source.tar.gz").is_file())
        metadata = json.loads((generated / "verifier_tests/metadata.json").read_text())
        self.assertEqual(metadata["branches"], ["79b69dd3fd98"])
        self.assertEqual(metadata["ignored_tests"], ["79b69dd3fd98/tests.test_cli.test_version"])
        request = json.loads((generated / "capsule_request.json").read_text())
        self.assertEqual(request["layers"][0]["kind"], "rootfs")
        self.assertEqual(request["layers"][0]["env"]["GOMODCACHE"], "/usr/local/go/mod-cache")
        self.assertEqual(request["policy"]["workspace_path_aliases"]["/logs"], "verifier_logs")
        task_toml = (generated / "task.toml").read_text()
        self.assertIn('capsule = "program_bench"', task_toml)
        self.assertIn('capsule_request = "capsule_request.json"', task_toml)
        self.assertIn("oracle_branch_smoke", task_toml)


if __name__ == "__main__":
    unittest.main()
