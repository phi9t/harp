from __future__ import annotations

import io
import hashlib
import json
import os
import stat
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).parents[1]))

import meta_harness_trae as harness


MEMORY_SYSTEM = """
from text_classification.memory_system import MemorySystem

class CandidateMemory(MemorySystem):
    def __init__(self, llm):
        super().__init__(llm)
        self.items = []

    def predict(self, input):
        return self.call_llm(input), {"items": len(self.items)}

    def learn_from_batch(self, batch_results):
        self.items.extend(item["ground_truth"] for item in batch_results)

    def get_state(self):
        import json
        return json.dumps(self.items)

    def set_state(self, state):
        import json
        self.items = json.loads(state)
""".lstrip()


def candidate(name: str) -> dict[str, str]:
    return {
        "name": name,
        "path": f"agents/{name}.py",
        "axis": "retrieval",
        "hypothesis": f"{name} tests a bounded retrieval policy.",
    }


def final_response(*names: str) -> dict[str, object]:
    return {
        "schema_version": "harp-meta-harness-trae-final/v1",
        "candidates": [candidate(name) for name in names],
    }


def write_reference_workspace(root: Path, names: tuple[str, ...]) -> None:
    (root / "agents").mkdir(parents=True)
    (root / "logs").mkdir()
    package = root / "reference/text_classification"
    package.mkdir(parents=True)
    (package / "__init__.py").write_text("")
    (package / "memory_system.py").write_text(
        Path(__file__)
        .parents[3]
        .joinpath(
            "evidence/implementations/meta_harness/snapshot/"
            "reference_examples/text_classification/memory_system.py"
        )
        .read_text()
    )
    (package / "llm.py").write_text(
        "from typing import Protocol\n"
        "class LLMCallable(Protocol):\n"
        "    def __call__(self, prompt: str) -> str: ...\n"
    )
    for name in names:
        (root / f"agents/{name}.py").write_text(MEMORY_SYSTEM)
    (root / "logs/pending_eval.json").write_text(
        json.dumps({"candidates": [candidate(name) for name in names]}) + "\n"
    )


class CommandTests(unittest.TestCase):
    def test_build_command_pins_model_tools_and_sandbox(self) -> None:
        command = harness.build_command(
            workspace=Path("/tmp/workspace"),
            schema=Path("/tmp/final.schema.json"),
            final_message=Path("/tmp/final.json"),
        )

        self.assertEqual(command[:2], ["traecli", "exec"])
        for required in [
            "--ignore-user-config",
            "--ignore-rules",
            "--ephemeral",
            "--sandbox",
            "workspace-write",
            "--config",
            'approval_policy="never"',
            "--model",
            "gpt-5.4",
            "--json",
        ]:
            self.assertIn(required, command)
        self.assertNotIn("--permission-mode", command)
        allowed = [
            command[index + 1]
            for index, value in enumerate(command)
            if value == "--allowed-tool"
        ]
        self.assertEqual(allowed, ["Read", "Glob", "Grep", "Bash", "Write", "Edit"])
        self.assertNotIn("WebSearch", command)
        self.assertNotIn("spawn_agent", command)


class SchemaTests(unittest.TestCase):
    def test_validate_response_accepts_exactly_three_matching_candidates(self) -> None:
        response = final_response("alpha_memory", "beta_memory", "gamma_memory")
        pending = {"candidates": response["candidates"]}

        result = harness.validate_proposal(response, pending)

        self.assertEqual([item["name"] for item in result], [
            "alpha_memory",
            "beta_memory",
            "gamma_memory",
        ])

    def test_validate_response_rejects_malformed_or_duplicate_candidates(self) -> None:
        with self.assertRaisesRegex(harness.ValidationError, "schema_version"):
            harness.validate_proposal(
                {"candidates": []},
                {"candidates": []},
            )
        with self.assertRaisesRegex(harness.ValidationError, "exactly three"):
            harness.validate_proposal(
                final_response("alpha_memory", "beta_memory"),
                {"candidates": [candidate("alpha_memory"), candidate("beta_memory")]},
            )
        duplicate = final_response("alpha_memory", "alpha_memory", "gamma_memory")
        with self.assertRaisesRegex(harness.ValidationError, "unique"):
            harness.validate_proposal(
                duplicate,
                {"candidates": duplicate["candidates"]},
            )

    def test_validate_response_rejects_path_mismatch_and_extra_fields(self) -> None:
        response = final_response("alpha_memory", "beta_memory", "gamma_memory")
        response["candidates"][0]["path"] = "agents/wrong.py"
        with self.assertRaisesRegex(harness.ValidationError, "matching path"):
            harness.validate_proposal(
                response,
                {"candidates": response["candidates"]},
            )

        response = final_response("alpha_memory", "beta_memory", "gamma_memory")
        response["candidates"][0]["score"] = 1
        with self.assertRaisesRegex(harness.ValidationError, "allowed fields"):
            harness.validate_proposal(
                response,
                {"candidates": response["candidates"]},
            )


class WorkspaceTests(unittest.TestCase):
    def test_validate_workspace_accepts_declared_candidate_writes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            names = ("alpha_memory", "beta_memory", "gamma_memory")
            write_reference_workspace(root, names)

            result = harness.validate_workspace(root, list(map(candidate, names)))

        self.assertEqual(len(result["candidates"]), 3)
        self.assertTrue(result["interface_checks_passed"])
        self.assertFalse(any("__pycache__" in item for item in result["workspace_files"]))

    def test_validate_workspace_rejects_traversal_symlinks_and_extra_writes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            names = ("alpha_memory", "beta_memory", "gamma_memory")
            write_reference_workspace(root, names)
            (root / "agents/extra.py").write_text(MEMORY_SYSTEM)
            with self.assertRaisesRegex(harness.ValidationError, "undeclared"):
                harness.validate_workspace(root, list(map(candidate, names)))

            (root / "agents/extra.py").unlink()
            (root / "surprise.txt").write_text("undeclared")
            with self.assertRaisesRegex(harness.ValidationError, "undeclared"):
                harness.validate_workspace(root, list(map(candidate, names)))

            (root / "surprise.txt").unlink()
            (root / "agents/beta_memory.py").unlink()
            (root / "agents/beta_memory.py").symlink_to(root / "agents/alpha_memory.py")
            with self.assertRaisesRegex(harness.ValidationError, "symlink"):
                harness.validate_workspace(root, list(map(candidate, names)))

        with self.assertRaisesRegex(harness.ValidationError, "normal relative"):
            harness.validate_proposal(
                {
                    "schema_version": "harp-meta-harness-trae-final/v1",
                    "candidates": [
                        {**candidate("alpha_memory"), "path": "../alpha_memory.py"},
                        candidate("beta_memory"),
                        candidate("gamma_memory"),
                    ],
                },
                {
                    "candidates": [
                        {**candidate("alpha_memory"), "path": "../alpha_memory.py"},
                        candidate("beta_memory"),
                        candidate("gamma_memory"),
                    ]
                },
            )

    def test_validate_workspace_rejects_mutation_syntax_import_interface_and_timeout(self) -> None:
        names = ("alpha_memory", "beta_memory", "gamma_memory")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_reference_workspace(root, names)
            baseline = root / "reference/text_classification/memory_system.py"
            expected = {
                path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                for path in (root / "reference").rglob("*")
                if path.is_file()
            }
            (root / "reference/extra.txt").write_text("new reference")
            with self.assertRaisesRegex(harness.ValidationError, "reference file set"):
                harness.validate_workspace(root, list(map(candidate, names)), expected)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_reference_workspace(root, names)
            baseline = root / "reference/text_classification/memory_system.py"
            expected = {
                path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                for path in (root / "reference").rglob("*")
                if path.is_file()
            }
            expected["reference/text_classification/memory_system.py"] = "0" * 64
            baseline.write_text(baseline.read_text() + "\n# mutation\n")
            with self.assertRaisesRegex(harness.ValidationError, "reference digest"):
                harness.validate_workspace(root, list(map(candidate, names)), expected)

        for invalid, message, timeout in [
            ("def broken(:\n", "syntax", 2.0),
            (MEMORY_SYSTEM + "\nraise RuntimeError('import failed')\n", "import", 2.0),
            ("class NotMemory:\n    pass\n", "MemorySystem subclass", 2.0),
            (
                MEMORY_SYSTEM.replace(
                    "return self.call_llm(input),",
                    "while True: pass\n        return self.call_llm(input),",
                ),
                "timed out",
                0.1,
            ),
        ]:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                write_reference_workspace(root, names)
                (root / "agents/alpha_memory.py").write_text(invalid)
                with self.assertRaisesRegex(harness.ValidationError, message):
                    harness.validate_workspace(
                        root,
                        list(map(candidate, names)),
                        interface_timeout=timeout,
                    )

    def test_assessment_retains_invalid_candidates_without_aborting(self) -> None:
        names = ("alpha_memory", "beta_memory", "gamma_memory")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_reference_workspace(root, names)
            invalid = MEMORY_SYSTEM.replace(
                "from text_classification.memory_system",
                "from reference.text_classification.memory_system",
            )
            (root / "agents/alpha_memory.py").write_text(invalid)

            report = harness.assess_workspace(root, list(map(candidate, names)))

        self.assertEqual(report["candidate_count"], 3)
        self.assertEqual(report["valid_candidate_count"], 2)
        self.assertFalse(report["interface_checks_passed"])
        self.assertFalse(report["candidates"][0]["valid"])
        self.assertIn("import or interface failure", report["candidates"][0]["error"])


class ArchiveTests(unittest.TestCase):
    def test_deterministic_archive_round_trips_with_member_digests(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "raw"
            source.mkdir()
            (source / "events.jsonl").write_text('{"type":"done"}\n')
            (source / "stderr.txt").write_text("")
            archive = root / "run.tar.gz"
            inventory = root / "members.tsv"

            harness.create_deterministic_archive(source, archive, inventory)
            output = root / "unpacked"
            harness.safe_extract_archive(archive, inventory, output)

            first = archive.read_bytes()
            extracted = (output / "events.jsonl").read_text()
            harness.create_deterministic_archive(source, archive, inventory)
            second = archive.read_bytes()

        self.assertEqual(second, first)
        self.assertEqual(extracted, '{"type":"done"}\n')

    def test_archive_rejects_traversal_symlink_digest_and_size(self) -> None:
        def archive_with(info: tarfile.TarInfo, body: bytes = b"") -> bytes:
            stream = io.BytesIO()
            with tarfile.open(fileobj=stream, mode="w:gz") as archive:
                archive.addfile(info, io.BytesIO(body))
            return stream.getvalue()

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            inventory = root / "members.tsv"
            inventory.write_text("path\tbytes\tsha256\n")
            output = root / "out"

            traversal = tarfile.TarInfo("../escape")
            traversal.size = 1
            (root / "bad.tar.gz").write_bytes(archive_with(traversal, b"x"))
            with self.assertRaisesRegex(harness.ValidationError, "normal relative"):
                harness.safe_extract_archive(root / "bad.tar.gz", inventory, output)

            link = tarfile.TarInfo("link")
            link.type = tarfile.SYMTYPE
            link.linkname = "target"
            (root / "bad.tar.gz").write_bytes(archive_with(link))
            with self.assertRaisesRegex(harness.ValidationError, "regular files"):
                harness.safe_extract_archive(root / "bad.tar.gz", inventory, output)

            member = tarfile.TarInfo("file.txt")
            member.size = 4
            (root / "bad.tar.gz").write_bytes(archive_with(member, b"data"))
            inventory.write_text("path\tbytes\tsha256\nfile.txt\t4\t" + "0" * 64 + "\n")
            with self.assertRaisesRegex(harness.ValidationError, "digest"):
                harness.safe_extract_archive(root / "bad.tar.gz", inventory, output)
            with self.assertRaisesRegex(harness.ValidationError, "size"):
                harness.safe_extract_archive(
                    root / "bad.tar.gz",
                    inventory,
                    output,
                    max_decompressed_bytes=3,
                )

    def test_secret_scan_aborts_without_redaction(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "events.jsonl"
            path.write_text('{"authorization":"Bearer secret-token"}\n')

            with self.assertRaisesRegex(harness.ValidationError, "sensitive"):
                harness.scan_for_secrets([path])

            self.assertIn("secret-token", path.read_text())


class RunnerTests(unittest.TestCase):
    def test_fake_run_preserves_output_and_never_launches_benchmark(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            names = ("alpha_memory", "beta_memory", "gamma_memory")
            write_reference_workspace(root, names)
            final = root / "final.json"
            final.write_text(json.dumps(final_response(*names)))

            completed = harness.run_proposer(
                root,
                Path("schema.json"),
                final,
                executor=lambda command, **kwargs: mock.Mock(
                    returncode=0,
                    stdout='{"type":"done"}\n',
                    stderr="",
                ),
            )

        self.assertEqual(completed.returncode, 0)
        self.assertFalse(any("benchmark" in part or "--test" == part for part in completed.args))

    def test_event_audit_rejects_benchmark_and_network_shell_commands(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            events = Path(directory) / "events.jsonl"
            events.write_text(
                json.dumps(
                    {
                        "type": "item.completed",
                        "item": {
                            "type": "command_execution",
                            "command": "python benchmark.py --test",
                            "exit_code": 0,
                        },
                    }
                )
                + "\n"
            )
            with self.assertRaisesRegex(harness.ValidationError, "benchmark"):
                harness.audit_event_log(events)

            events.write_text(
                json.dumps(
                    {
                        "type": "tool_call",
                        "tool_name": "Bash",
                        "arguments": {"command": "curl https://example.com"},
                    }
                )
                + "\n"
            )
            with self.assertRaisesRegex(harness.ValidationError, "network"):
                harness.audit_event_log(events)

            events.write_text(
                json.dumps(
                    {
                        "type": "tool_call",
                        "tool_name": "Bash",
                        "arguments": {"command": "python -m py_compile agents/a.py"},
                    }
                )
                + "\n"
            )
            harness.audit_event_log(events)


if __name__ == "__main__":
    unittest.main()
