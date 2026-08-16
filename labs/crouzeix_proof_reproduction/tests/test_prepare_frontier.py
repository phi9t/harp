from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import protocol
import tickets
import prepare_frontier


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fixture_prompt() -> bytes:
    return (
        b"Crouzeix theorem fixture\nCurrent task statement\nwrite to "
        + protocol.HISTORICAL_OUTPUT_PATH.encode()
        + b"\n"
    )


def write_cli(path: Path, *, version: str = "traecli test-version") -> None:
    path.write_text(f"#!/bin/sh\nprintf '%s\\n' '{version}'\n")
    path.chmod(0o755)


class FrontierPreparationTests(unittest.TestCase):
    def prepare(self, root: Path, **overrides: object) -> Path:
        prompt = root / "historical.txt"
        source = fixture_prompt()
        prompt.write_bytes(source)
        cli = root / "traecli"
        write_cli(cli, version=str(overrides.pop("cli_version", "traecli test-version")))
        run_dir = root / "expert-frontier-001"
        with mock.patch.object(
            prepare_frontier,
            "verify_historical_prompt",
            side_effect=lambda data: protocol.verify_historical_prompt(
                data,
                expected_bytes=len(source),
                expected_sha256=digest(source),
            ),
        ):
            return prepare_frontier.prepare_frontier(
                run_dir=Path(overrides.pop("run_dir", run_dir)),
                historical_prompt_path=Path(overrides.pop("historical_prompt_path", prompt)),
                cli_path=Path(overrides.pop("cli_path", cli)),
                model=str(overrides.pop("model", "gpt-5.6-sol")),
                timeout_seconds=int(overrides.pop("timeout_seconds", 3600)),
                created_at_utc=str(overrides.pop("created_at_utc", "2026-08-15T12:00:00Z")),
            )

    def test_frontier_constants_are_pinned(self) -> None:
        self.assertEqual(protocol.HISTORICAL_PROMPT_BYTES, 4106)
        self.assertEqual(
            protocol.HISTORICAL_PROMPT_SHA256,
            "0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc",
        )
        self.assertEqual(prepare_frontier.FRONTIER_MODEL, "gpt-5.6-sol")
        self.assertEqual(prepare_frontier.SELECTION_SEED, 20260814)
        self.assertEqual(
            prepare_frontier.EXPERT_ROLES,
            (
                "function_theory",
                "operator_dilation",
                "matrix_extremal",
                "completion_positivity",
                "approximation_audit",
            ),
        )
        self.assertEqual(len(prepare_frontier.PROOF_PROGRESS_PROBE_IDS), 10)

    def test_prepare_frontier_writes_run_spec_tree_and_root_tickets(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.prepare(root)
            run_dir = root / "expert-frontier-001"

            self.assertEqual(result, run_dir.resolve())
            spec = protocol.read_run_spec(run_dir / "run_spec.json")
            self.assertEqual(spec["arm"], "expert_frontier")
            self.assertEqual(spec["leakage"], "L1")
            self.assertEqual(spec["model"], "gpt-5.6-sol")
            self.assertEqual(spec["max_calls"], 33)
            self.assertEqual(spec["timeout_seconds"], 3600)
            self.assertEqual(spec["allowed_tools"], ["Write"])
            self.assertFalse(spec["network_access"])
            self.assertEqual(spec["frontier"]["selection_seed"], 20260814)
            self.assertEqual(spec["frontier"]["root_expert_count"], 5)
            self.assertEqual(spec["frontier"]["child_generations"], 3)
            self.assertEqual(spec["frontier"]["draws_per_generation"], 2)
            self.assertEqual(spec["frontier"]["admitted_mathematical_node_budget"], 11)
            self.assertEqual(spec["frontier"]["total_call_budget"], 33)
            self.assertEqual(spec["frontier"]["per_call_preflight"]["memory_mib"], 4096)
            self.assertEqual(
                spec["frontier"]["expert_roles"],
                list(prepare_frontier.EXPERT_ROLES),
            )
            self.assertEqual(
                spec["frontier"]["proof_progress_probe_ids"],
                list(prepare_frontier.PROOF_PROGRESS_PROBE_IDS),
            )
            self.assertIn("public proof manuscripts", spec["generation_excluded_classes"])
            self.assertIn("prompt/expert.md", spec["digests"])
            self.assertIn("schema/expert_result.schema.json", spec["digests"])
            self.assertIn("config/frontier", spec["digests"])
            self.assertEqual(spec["digests"]["cli"], spec["cli"]["sha256"])

            self.assertFalse((run_dir / "inputs/historical_prompt.txt").exists())
            self.assertNotIn(
                protocol.HISTORICAL_OUTPUT_PATH.encode(),
                (run_dir / "inputs/execution_prompt.txt").read_bytes(),
            )
            self.assertTrue((run_dir / "inputs/theorem.txt").is_file())

            for relative in (
                "tickets",
                "attempts",
                "admissions",
                "mathematical_nodes",
                "node_evaluations",
                "archive_entries",
                "candidate_projections",
                "review",
                "calls",
            ):
                self.assertTrue((run_dir / relative).is_dir(), relative)

            ticket_root = run_dir / "tickets"
            ticket_ids = sorted(path.name for path in ticket_root.iterdir())
            self.assertEqual(
                ticket_ids,
                [
                    "expert-g0-approximation-audit",
                    "expert-g0-completion-positivity",
                    "expert-g0-function-theory",
                    "expert-g0-matrix-extremal",
                    "expert-g0-operator-dilation",
                ],
            )
            for ticket_id in ticket_ids:
                ticket_path = ticket_root / ticket_id / "ticket.json"
                event_path = ticket_root / ticket_id / "ticket_events.jsonl"
                ticket = tickets.validate_runtime_ticket(
                    json.loads(ticket_path.read_text())
                )
                self.assertEqual(ticket["run_id"], "expert-frontier-001")
                self.assertEqual(ticket["generation"], 0)
                self.assertEqual(ticket["task_kind"], "expert")
                self.assertEqual(ticket["owner_type"], "expert")
                self.assertEqual(ticket["allowed_tools"], ["Write"])
                self.assertEqual(ticket["timeout_seconds"], 3600)
                self.assertEqual(ticket["max_output_bytes"], 1048576)
                self.assertIsNone(ticket["parent_artifact_sha256"])
                self.assertIn("public proof manuscripts", ticket["forbidden_sources"])
                self.assertFalse(event_path.exists())

            check = prepare_frontier.check_frontier_preparation(run_dir)
            self.assertEqual(check["root_ticket_count"], 5)
            self.assertEqual(check["run_spec_sha256"], digest((run_dir / "run_spec.json").read_bytes()))

    def test_prepare_frontier_rejects_existing_symlink_wrong_cli_and_bad_config(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            run_dir = root / "existing"
            run_dir.mkdir()
            with self.assertRaisesRegex(protocol.ValidationError, "existing"):
                self.prepare(root, run_dir=run_dir)

            prompt = root / "historical.txt"
            prompt.write_bytes(fixture_prompt())
            prompt_link = root / "prompt-link"
            prompt_link.symlink_to(prompt)
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                self.prepare(root, historical_prompt_path=prompt_link)

            bad_cli = root / "not-trae"
            write_cli(bad_cli, version="othercli 1.0")
            with self.assertRaisesRegex(protocol.ValidationError, "TRAE CLI identity"):
                self.prepare(root, run_dir=root / "bad-cli", cli_path=bad_cli)

            with self.assertRaisesRegex(protocol.ValidationError, "model"):
                self.prepare(root, run_dir=root / "bad-model", model="gpt-5")
            with self.assertRaisesRegex(protocol.ValidationError, "timeout"):
                self.prepare(root, run_dir=root / "bad-timeout", timeout_seconds=3599)


if __name__ == "__main__":
    unittest.main()
