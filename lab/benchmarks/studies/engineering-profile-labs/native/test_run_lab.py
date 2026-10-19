"""Model-free fixture tests and re-reduction of actual retained native evidence."""

import contextlib
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import run_lab


class CliCompletionContractTest(unittest.TestCase):
    """Test the acceptance seam against loss, failure, and identity ambiguity."""

    def setUp(self):
        self.events = [
            {"type": "thread.started", "thread_id": "fixture-thread"},
            {"type": "turn.started"},
            {
                "type": "item.completed",
                "item_type": "agent_message",
                "arithmetic_answer_matches": True,
            },
            {"type": "turn.completed", "usage": {"output_tokens": 1}},
        ]

    def test_exit_zero_does_not_replace_structured_completion(self):
        self.assertFalse(run_lab.cli_accepted(self.events[:-1], 0, False))

    def test_partial_answer_does_not_replace_completed_item(self):
        self.events[2]["type"] = "item.started"
        self.assertFalse(run_lab.cli_accepted(self.events, 0, False))

    def test_missing_thread_identity_rejects(self):
        self.assertFalse(run_lab.cli_accepted(self.events[1:], 0, False))

    def test_mixed_thread_identity_rejects(self):
        self.events.append({"type": "thread.started", "thread_id": "other-thread"})
        self.assertFalse(run_lab.cli_accepted(self.events, 0, False))

    def test_failed_exit_timeout_or_native_failure_rejects(self):
        self.assertFalse(run_lab.cli_accepted(self.events, 1, False))
        self.assertFalse(run_lab.cli_accepted(self.events, 0, True))
        self.events.append({"type": "turn.failed"})
        self.assertFalse(run_lab.cli_accepted(self.events, 0, False))

    def test_completed_arithmetic_passes(self):
        self.assertTrue(run_lab.cli_accepted(self.events, 0, False))

    def test_answer_requires_one_ordered_turn_interval(self):
        thread, started, answer, completed = self.events
        for label, events in (
            ("missing_start", [thread, answer, completed]),
            ("answer_before_start", [thread, answer, started, completed]),
            ("answer_after_end", [thread, started, completed, answer]),
            ("end_before_start", [thread, completed, started, answer]),
            ("multiple_starts", [thread, started, started, answer, completed]),
            ("thread_after_start", [started, thread, answer, completed]),
        ):
            with self.subTest(label=label):
                self.assertFalse(run_lab.cli_accepted(events, 0, False))

    def test_answer_from_another_thread_rejects(self):
        self.events[2]["thread_id"] = "other-thread"
        self.assertFalse(run_lab.cli_accepted(self.events, 0, False))

    def test_native_projection_preserves_mismatched_item_thread_identity(self):
        raw = json.dumps(
            {
                "type": "item.completed",
                "thread_id": "other-thread",
                "item": {"type": "agent_message", "text": "42"},
            }
        )
        self.events[2] = run_lab.cli_projection(raw)[0]
        self.assertFalse(run_lab.cli_accepted(self.events, 0, False))

    def test_unexpected_tool_rejects(self):
        self.events.append({"type": "item.completed", "item_type": "command_execution"})
        self.assertFalse(run_lab.cli_accepted(self.events, 0, False))

    def test_unknown_and_unclassified_items_reject(self):
        for item_type in (
            "web_search",
            "mcp_tool_call",
            "collab_agent_tool_call",
            "future_item",
            None,
        ):
            with self.subTest(item_type=item_type):
                events = self.events + [
                    {"type": "item.completed", "item_type": item_type}
                ]
                self.assertFalse(run_lab.cli_accepted(events, 0, False))

    def test_native_diagnostic_item_does_not_prevent_arithmetic_completion(self):
        self.events.insert(1, {"type": "item.completed", "item_type": "error"})
        self.assertTrue(run_lab.cli_accepted(self.events, 0, False))

    def test_cli_and_rpc_tool_flags_fail_closed_for_unknown_items(self):
        for item_type in (
            "web_search",
            "mcp_tool_call",
            "dynamicToolCall",
            "future_item",
            None,
        ):
            cli = run_lab.cli_projection(
                json.dumps(
                    {
                        "type": "item.completed",
                        "item": {"type": item_type},
                    }
                )
            )
            rpc = run_lab.rpc_projection(
                [
                    {
                        "direction": "runtime_to_client",
                        "message": {
                            "method": "item/completed",
                            "params": {"item": {"type": item_type}},
                        },
                    }
                ]
            )
            with self.subTest(item_type=item_type):
                self.assertTrue(run_lab.unexpected_tool_observed(cli))
                self.assertTrue(run_lab.unexpected_tool_observed(rpc))

    def test_cli_projection_omits_arbitrary_runtime_messages(self):
        source = json.dumps(
            {
                "type": "item.completed",
                "item": {"type": "agent_message", "text": "private fixture text"},
                "extra": "private fixture field",
            }
        )
        projected = run_lab.cli_projection(source)
        self.assertNotIn("private fixture", json.dumps(projected))
        self.assertFalse(projected[0]["arithmetic_answer_matches"])

    def test_host_rpc_projection_keeps_identity_without_broad_content(self):
        source = [
            {
                "direction": "runtime_to_client",
                "message": {
                    "method": "turn/completed",
                    "params": {
                        "threadId": "fixture-thread",
                        "turn": {
                            "id": "fixture-turn",
                            "status": "completed",
                            "secretFixture": "omit",
                        },
                        "arbitraryFixture": "omit",
                    },
                },
            }
        ]
        projected = run_lab.rpc_projection(source)
        self.assertEqual(projected[0]["threadId"], "fixture-thread")
        self.assertEqual(projected[0]["turn"]["id"], "fixture-turn")
        self.assertNotIn("omit", json.dumps(projected))


class TemporaryRootContractTest(unittest.TestCase):
    """Exercise the CLI boundary; no native binary runs in these fixtures."""

    def invoke(self, root, output):
        argv = ["run_lab.py", "inventory", "--out", str(output)]
        with patch.object(sys, "argv", argv), patch.dict(os.environ, TMPDIR=str(root)):
            with patch.object(
                run_lab,
                "inventory",
                return_value={"classification": "FAKE_CONFORMANCE_FIXTURE"},
            ):
                with (
                    contextlib.redirect_stdout(io.StringIO()),
                    contextlib.redirect_stderr(io.StringIO()),
                ):
                    return run_lab.main()

    def test_system_temporary_descendant_rejected_before_native_execution(self):
        with tempfile.TemporaryDirectory(prefix="native-policy-fixture-") as directory:
            output = Path(directory) / "result.json"
            forbidden = Path("/") / "tmp" / "native-unallocated-fixture"
            with self.assertRaises(SystemExit) as failure:
                self.invoke(forbidden, output)
            self.assertEqual(failure.exception.code, 2)
            self.assertFalse(output.exists())

    def test_symlink_to_system_temporary_directory_rejected(self):
        with tempfile.TemporaryDirectory(prefix="native-policy-fixture-") as directory:
            root = Path(directory)
            alias = root / "alias"
            alias.symlink_to(Path("/") / "tmp", target_is_directory=True)
            with self.assertRaises(SystemExit) as failure:
                self.invoke(alias, root / "result.json")
            self.assertEqual(failure.exception.code, 2)

    def test_shared_writable_root_rejected(self):
        with tempfile.TemporaryDirectory(prefix="native-policy-fixture-") as directory:
            root = Path(directory)
            unsafe = root / "unsafe"
            unsafe.mkdir(mode=0o777)
            unsafe.chmod(0o777)
            with self.assertRaises(SystemExit) as failure:
                self.invoke(unsafe, root / "result.json")
            self.assertEqual(failure.exception.code, 2)

    def test_root_owned_by_different_uid_rejected(self):
        with tempfile.TemporaryDirectory(prefix="native-policy-fixture-") as directory:
            root = Path(directory)
            uid = os.getuid()
            with patch.object(os, "getuid", return_value=uid + 1):
                with self.assertRaises(SystemExit) as failure:
                    self.invoke(root, root / "result.json")
            self.assertEqual(failure.exception.code, 2)

    def test_owned_private_root_accepted(self):
        with tempfile.TemporaryDirectory(prefix="native-policy-fixture-") as directory:
            root = Path(directory)
            self.assertEqual(self.invoke(root, root / "result.json"), 0)
            self.assertEqual(
                json.loads((root / "result.json").read_text())["classification"],
                "FAKE_CONFORMANCE_FIXTURE",
            )


class HistoricalCliEvidenceTest(unittest.TestCase):
    """Re-reduce actual retained native rows without executing a runtime."""

    def test_recorded_codex_and_trae_outcomes_survive_stricter_guards(self):
        evidence = Path(__file__).parent / "evidence"
        for name, accepted in (
            ("20261011-codex-cli.json", True),
            ("20261011-traecli.json", False),
            ("20261011-traecli-catalog-model.json", False),
        ):
            with self.subTest(name=name):
                run = json.loads((evidence / name).read_text())
                self.assertEqual(
                    run_lab.cli_accepted(
                        run["projected_events"], run["exit_code"], run["timed_out"]
                    ),
                    accepted,
                )


if __name__ == "__main__":
    unittest.main()
