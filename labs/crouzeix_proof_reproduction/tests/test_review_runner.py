from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
TESTS = Path(__file__).resolve().parent
sys.path.insert(0, str(LAB))
sys.path.insert(0, str(TESTS))

import protocol
import review_contracts
import review_runner
import tickets
from test_tickets import runtime_ticket


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_text(value: str) -> str:
    return sha256_bytes(value.encode("utf-8"))


def sha256_json(value: object) -> str:
    return sha256_bytes(
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
    )


CANDIDATE = "Candidate proof bytes.\n"
REPAIRED = "Candidate proof bytes with repaired gap.\n"


def ledger_sha256() -> str:
    ledger = finding_ledger(include_digest=False)
    return sha256_json(ledger)


def write_fake_cli(path: Path, mode: str) -> None:
    script = f"""\
#!/usr/bin/env python3
import json
import pathlib
import sys

MODE = {mode!r}
if "--version" in sys.argv:
    print("traecli review-fake-1.0")
    raise SystemExit(0)

def option(name):
    return sys.argv[sys.argv.index(name) + 1]

def options(name):
    return [
        sys.argv[index + 1]
        for index, item in enumerate(sys.argv)
        if item == name
    ]

workspace = pathlib.Path(option("--cd"))
final_path = pathlib.Path(option("--output-last-message"))
prompt = sys.stdin.read()

assert "--ignore-user-config" in sys.argv
assert "--ignore-rules" in sys.argv
assert "--ephemeral" in sys.argv
assert "--json" in sys.argv
assert options("--allowed-tool") == ["Write"], options("--allowed-tool")
assert sorted(item.name for item in workspace.iterdir()) == []
for forbidden in ["expert_frontier", "hidden-model", "cost_usd", "source_node_artifact_sha256"]:
    assert forbidden not in prompt, forbidden

if MODE == "failed":
    print(json.dumps({{"type": "thread.started", "thread_id": "review-thread"}}))
    print(json.dumps({{"type": "turn.completed"}}))
    raise SystemExit(9)

if MODE == "review_complete":
    payload = {{
        "schema_version": "crouzeix-correctness-review-payload/v1",
        "anonymous_candidate_id": "anon-candidate-alpha",
        "candidate_sha256": {sha256_text(CANDIDATE)!r},
        "reviewer_index": 1,
        "outcome": "complete",
        "findings": [],
        "theorem_strength_obligations": []
    }}
elif MODE == "repair":
    payload = {{
        "schema_version": "crouzeix-repair-payload/v1",
        "repair_id": "repair-candidate-alpha",
        "parent_candidate_sha256": {sha256_text(CANDIDATE)!r},
        "finding_ledger_sha256": {ledger_sha256()!r},
        "candidate_text": {REPAIRED!r},
        "dispositions": [
            {{
                "namespaced_finding_id": "r1:finding-gap-main",
                "disposition": "fixed",
                "rationale": "The gap is explicitly patched."
            }}
        ],
        "unproved_obligations": []
    }}
elif MODE == "classification_mutates":
    payload = {{
        "schema_version": "crouzeix-mechanism-classification-payload/v1",
        "classification_id": "classify-candidate-alpha",
        "candidate_sha256": {sha256_text(CANDIDATE)!r},
        "frozen_correctness_sha256": {"c" * 64!r},
        "similarities": [],
        "candidate_rewrite": "mutated",
        "correctness_outcome": "complete"
    }}
else:
    payload = {{
        "schema_version": "crouzeix-correctness-review-payload/v1",
        "anonymous_candidate_id": "anon-candidate-alpha",
        "candidate_sha256": {sha256_text(CANDIDATE)!r},
        "reviewer_index": 1,
        "outcome": "incomplete",
        "findings": [
            {{
                "finding_id": "finding-gap-main",
                "severity": "critical",
                "locator": "candidate.tex#L12",
                "statement": "The theorem-strength gap is open.",
                "falsifying_test_or_gap": "Provide the independent bound."
            }}
        ],
        "theorem_strength_obligations": [
            {{
                "obligation_id": "obl-main-gap",
                "locator": "candidate.tex#L12",
                "statement": "Close the theorem-strength gap."
            }}
        ]
    }}

final_path.write_text(json.dumps(payload))
print(json.dumps({{"type": "thread.started", "thread_id": "review-thread"}}))
print(json.dumps({{"type": "turn.started"}}))
print(json.dumps({{"type": "item.completed", "item": {{"type": "agent_message", "text": final_path.read_text()}}}}))
print(json.dumps({{"type": "turn.completed", "usage": {{"input_tokens": 13, "output_tokens": 8}}}}))
"""
    path.write_text(textwrap.dedent(script))
    path.chmod(0o755)


def write_review_dir(root: Path, cli: Path) -> Path:
    review_dir = root / "review-001"
    for relative in ("calls", "prompts", "schemas", "tickets"):
        (review_dir / relative).mkdir(parents=True)
    for name in (
        "correctness_review.md",
        "review_repair.md",
        "mechanism_classification.md",
    ):
        (review_dir / "prompts" / name).write_text((LAB / "prompts" / name).read_text())
    for name in (
        "correctness_review.schema.json",
        "repair_result.schema.json",
        "mechanism_classification.schema.json",
    ):
        (review_dir / "schemas" / name).write_bytes((LAB / "schemas" / name).read_bytes())
    spec = {
        "schema_version": "crouzeix-run-spec/v1",
        "run_id": review_dir.name,
        "arm": "guided",
        "leakage": "L2",
        "model": "gpt-5.6-sol",
        "cli": {
            "path": str(cli.resolve()),
            "version": "traecli review-fake-1.0",
            "sha256": sha256_bytes(cli.read_bytes()),
        },
        "historical_prompt": {
            "source_bytes": 4106,
            "source_sha256": protocol.HISTORICAL_PROMPT_SHA256,
            "execution_bytes": 4060,
            "execution_sha256": "e" * 64,
            "normalization": "replace path",
        },
        "sandbox": "workspace-write",
        "approval_policy": "never",
        "allowed_tools": ["Read", "Write"],
        "network_access": False,
        "timeout_seconds": 30,
        "max_calls": 12,
        "token_accounting": {"boundary": "review calls"},
        "generation_visible_files": ["inputs/theorem.txt"],
        "generation_excluded_classes": ["public proof manuscripts"],
        "created_at_utc": "2026-08-15T04:00:00Z",
    }
    protocol.validate_run_spec(spec)
    (review_dir / "run_spec.json").write_text(json.dumps(spec, indent=2, sort_keys=True) + "\n")
    return review_dir


def ticket_event_digest(review_dir: Path, ticket_id: str) -> str:
    lines = (review_dir / "tickets" / ticket_id / "ticket_events.jsonl").read_text().splitlines()
    self = json.loads(lines[-1])
    return sha256_json(self)


def candidate_projection(candidate: str = CANDIDATE) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-candidate-projection/v1",
        "projection_id": "candidate-alpha",
        "source_node_id": "node-g2-d1",
        "source_node_artifact_sha256": "a" * 64,
        "source_reconciliation_sha256": "b" * 64,
        "candidate_sha256": sha256_text(candidate),
        "candidate_byte_count": len(candidate.encode("utf-8")),
        "candidate_projection_sha256": "c" * 64,
        "arm": "expert_frontier",
        "model": "hidden-model",
        "cost_usd": "9.99",
        "lineage": ["node-g0"],
        "mechanism_label": "jin-like",
        "prior_findings": [{"finding_id": "leak"}],
        "reviewer_output": {"outcome": "complete"},
    }


def finding_ledger(*, include_digest: bool = True) -> dict[str, object]:
    ledger: dict[str, object] = {
        "schema_version": "crouzeix-correctness-reconciliation/v1",
        "reconciliation_id": "reconcile-candidate-alpha",
        "candidate_sha256": sha256_text(CANDIDATE),
        "outcome": "incomplete",
        "finding_dispositions": [
            {
                "namespaced_finding_id": "r1:finding-gap-main",
                "disposition": "unresolved",
                "severity": "critical",
                "locator": "candidate.tex#L12",
                "statement": "Gap.",
                "falsifying_test_or_gap": "Test.",
                "rationale": "Still open.",
            }
        ],
    }
    if include_digest:
        ledger["finding_ledger_sha256"] = sha256_json(ledger)
    return ledger


def second_finding_ledger(*, include_digest: bool = True) -> dict[str, object]:
    ledger: dict[str, object] = {
        "schema_version": "crouzeix-correctness-reconciliation/v1",
        "reconciliation_id": "reconcile-candidate-beta",
        "candidate_sha256": "1" * 64,
        "outcome": "incomplete",
        "finding_dispositions": [
            {
                "namespaced_finding_id": "r1:finding-critical",
                "disposition": "unresolved",
                "severity": "critical",
                "locator": "candidate.tex#L7",
                "statement": "Critical gap.",
                "falsifying_test_or_gap": "Close critical gap.",
                "rationale": "Still open.",
            },
            {
                "namespaced_finding_id": "r2:finding-major",
                "disposition": "unresolved",
                "severity": "major",
                "locator": "candidate.tex#L3",
                "statement": "Major gap.",
                "falsifying_test_or_gap": "Close major gap.",
                "rationale": "Still open.",
            },
        ],
    }
    if include_digest:
        ledger["finding_ledger_sha256"] = sha256_json(ledger)
    return ledger


def publish_review_ticket(
    review_dir: Path,
    *,
    ticket_id: str,
    task_kind: str,
    context_sha256: str,
    prompt_sha256: str,
    schema_sha256: str,
    parent_artifact_sha256: str | None,
) -> dict[str, object]:
    ticket = runtime_ticket(
        ticket_id=ticket_id,
        run_id=review_dir.name,
        task_kind=task_kind,
        node_id="candidate-alpha",
        parent_node_id=None,
        generation=0,
        direction_id=None,
        role=task_kind,
        objective=f"Run {task_kind}",
        expected_deliverable="Strict review artifact or typed failure.",
        dependency_ticket_ids=[],
        context_sha256=context_sha256,
        schema_sha256=schema_sha256,
        prompt_sha256=prompt_sha256,
        parent_artifact_sha256=parent_artifact_sha256,
        allowed_tools=["Write"],
        forbidden_sources=[
            "treatment",
            "model",
            "cost",
            "lineage",
            "prior findings",
            "network",
            "delegation",
        ],
        timeout_seconds=30,
        max_output_bytes=1048576,
        owner_type="orchestrator"
        if task_kind == "finding_reconciliation"
        else "reviewer",
    )
    tickets.publish_ticket(review_dir / "tickets", ticket)
    return ticket


class ReviewRunnerTests(unittest.TestCase):
    def test_finding_reconciliation_is_ticketed_and_selects_repair_deterministically(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            cli = root / "fake_review_cli.py"
            write_fake_cli(cli, "review_complete")
            review_dir = write_review_dir(root, cli)
            ledgers = [
                finding_ledger(),
                second_finding_ledger(),
            ]
            context = review_runner.build_reconciliation_context(
                reconciliation_id="reconcile-review-alpha",
                finding_ledgers=ledgers,
            )
            publish_review_ticket(
                review_dir,
                ticket_id="reconcile-review-alpha",
                task_kind="finding_reconciliation",
                context_sha256=sha256_json(context),
                prompt_sha256=review_runner.RECONCILIATION_PROMPT_SHA256,
                schema_sha256=review_runner.RECONCILIATION_SCHEMA_SHA256,
                parent_artifact_sha256=None,
            )

            decision = review_runner.run_finding_reconciliation(
                review_dir,
                ticket_id="reconcile-review-alpha",
                reconciliation_id="reconcile-review-alpha",
                finding_ledgers=ledgers,
            )

            self.assertEqual(decision["decision"], "repair_required")
            self.assertEqual(decision["selected_candidate_sha256"], sha256_text(CANDIDATE))
            self.assertEqual(
                [item["candidate_sha256"] for item in decision["candidate_decisions"]],
                [sha256_text(CANDIDATE), "1" * 64],
            )
            self.assertEqual(
                decision,
                json.loads(
                    (review_dir / "reconciliation_decisions/reconcile-review-alpha.json").read_text()
                ),
            )
            events = [
                json.loads(line)
                for line in (
                    review_dir
                    / "tickets/reconcile-review-alpha/ticket_events.jsonl"
                ).read_text().splitlines()
            ]
            self.assertEqual([event["to_state"] for event in events], ["admitted", "running", "completed", "accepted"])

    def test_correctness_review_requires_precreated_ticket_and_preserves_failure(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            cli = root / "fake_review_cli.py"
            write_fake_cli(cli, "failed")
            review_dir = write_review_dir(root, cli)
            prompt = (review_dir / "prompts/correctness_review.md").read_text()
            schema = review_dir / "schemas/correctness_review.schema.json"
            context = review_contracts.build_correctness_context(
                theorem_text="Theorem.",
                candidate_text=CANDIDATE,
                candidate_projection=candidate_projection(),
                reviewer_index=1,
                ticket_id="review-candidate-alpha-r1",
            )
            ticket = publish_review_ticket(
                review_dir,
                ticket_id="review-candidate-alpha-r1",
                task_kind="correctness_review",
                context_sha256=sha256_json(context),
                prompt_sha256=sha256_bytes(prompt.encode("utf-8")),
                schema_sha256=sha256_bytes(schema.read_bytes()),
                parent_artifact_sha256=sha256_text(CANDIDATE),
            )

            missing_dir = root / "missing-ticket-review"
            missing_dir.mkdir()
            (missing_dir / "calls").mkdir()
            (missing_dir / "tickets").mkdir()
            (missing_dir / "run_spec.json").write_text((review_dir / "run_spec.json").read_text())
            with self.assertRaisesRegex(protocol.ValidationError, "ticket"):
                review_runner.run_correctness_review(
                    missing_dir,
                    ticket_id="review-candidate-alpha-r1",
                    reviewer_index=1,
                    theorem_text="Theorem.",
                    candidate_text=CANDIDATE,
                    candidate_projection=candidate_projection(),
                )
            self.assertFalse((missing_dir / "calls/review-candidate-alpha-r1").exists())

            result = review_runner.run_correctness_review(
                review_dir,
                ticket_id="review-candidate-alpha-r1",
                reviewer_index=1,
                theorem_text="Theorem.",
                candidate_text=CANDIDATE,
                candidate_projection=candidate_projection(),
            )

            self.assertEqual(result["receipt"]["status"], "failed")
            self.assertIsNone(result["review"])
            self.assertEqual(result["receipt"]["ticket_id"], ticket["ticket_id"])
            self.assertTrue((review_dir / "calls/review-candidate-alpha-r1/receipt.json").is_file())
            events = [
                json.loads(line)
                for line in (
                    review_dir / "tickets/review-candidate-alpha-r1/ticket_events.jsonl"
                ).read_text().splitlines()
            ]
            self.assertEqual([event["to_state"] for event in events], ["admitted", "running", "failed"])

    def test_completed_correctness_review_envelope_uses_terminal_ticket_event_digest(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            cli = root / "fake_review_cli.py"
            write_fake_cli(cli, "review_complete")
            review_dir = write_review_dir(root, cli)
            prompt = (review_dir / "prompts/correctness_review.md").read_text()
            schema = review_dir / "schemas/correctness_review.schema.json"
            context = review_contracts.build_correctness_context(
                theorem_text="Theorem.",
                candidate_text=CANDIDATE,
                candidate_projection=candidate_projection(),
                reviewer_index=1,
                ticket_id="review-candidate-alpha-r1",
            )
            publish_review_ticket(
                review_dir,
                ticket_id="review-candidate-alpha-r1",
                task_kind="correctness_review",
                context_sha256=sha256_json(context),
                prompt_sha256=sha256_bytes(prompt.encode("utf-8")),
                schema_sha256=sha256_bytes(schema.read_bytes()),
                parent_artifact_sha256=sha256_text(CANDIDATE),
            )

            result = review_runner.run_correctness_review(
                review_dir,
                ticket_id="review-candidate-alpha-r1",
                reviewer_index=1,
                theorem_text="Theorem.",
                candidate_text=CANDIDATE,
                candidate_projection=candidate_projection(),
            )

            self.assertEqual(result["receipt"]["status"], "completed")
            self.assertIsNotNone(result["review"])
            self.assertEqual(
                result["review"]["terminal_ticket_event_sha256"],
                ticket_event_digest(review_dir, "review-candidate-alpha-r1"),
            )
            self.assertNotEqual(
                result["review"]["terminal_ticket_event_sha256"],
                review_contracts.canonical_sha256(result["receipt"]),
            )

    def test_repair_changed_bytes_require_two_fresh_re_review_tickets(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            cli = root / "fake_review_cli.py"
            write_fake_cli(cli, "repair")
            review_dir = write_review_dir(root, cli)
            ledger = finding_ledger()
            prompt = (review_dir / "prompts/review_repair.md").read_text()
            schema = review_dir / "schemas/repair_result.schema.json"
            request = review_contracts.build_repair_request(
                repair_id="repair-candidate-alpha",
                parent_candidate_text=CANDIDATE,
                finding_ledger=ledger,
                ticket_id="repair-candidate-alpha",
            )
            publish_review_ticket(
                review_dir,
                ticket_id="repair-candidate-alpha",
                task_kind="correctness_repair",
                context_sha256=sha256_json(request),
                prompt_sha256=sha256_bytes(prompt.encode("utf-8")),
                schema_sha256=sha256_bytes(schema.read_bytes()),
                parent_artifact_sha256=sha256_text(CANDIDATE),
            )

            with self.assertRaisesRegex(protocol.ValidationError, "re-review tickets"):
                review_runner.run_repair(
                    review_dir,
                    ticket_id="repair-candidate-alpha",
                    repair_id="repair-candidate-alpha",
                    parent_candidate_text=CANDIDATE,
                    finding_ledger=ledger,
                )
            self.assertTrue((review_dir / "calls/repair-candidate-alpha/receipt.json").is_file())

        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            cli = root / "fake_review_cli.py"
            write_fake_cli(cli, "repair")
            review_dir = write_review_dir(root, cli)
            ledger = finding_ledger()
            prompt = (review_dir / "prompts/review_repair.md").read_text()
            schema = review_dir / "schemas/repair_result.schema.json"
            request = review_contracts.build_repair_request(
                repair_id="repair-candidate-alpha",
                parent_candidate_text=CANDIDATE,
                finding_ledger=ledger,
                ticket_id="repair-candidate-alpha",
            )
            publish_review_ticket(
                review_dir,
                ticket_id="repair-candidate-alpha",
                task_kind="correctness_repair",
                context_sha256=sha256_json(request),
                prompt_sha256=sha256_bytes(prompt.encode("utf-8")),
                schema_sha256=sha256_bytes(schema.read_bytes()),
                parent_artifact_sha256=sha256_text(CANDIDATE),
            )

            for reviewer in (1, 2):
                context = review_contracts.build_correctness_context(
                    theorem_text="Theorem.",
                    candidate_text=REPAIRED,
                    candidate_projection=candidate_projection(REPAIRED),
                    reviewer_index=reviewer,
                    ticket_id=f"review-repair-candidate-alpha-r{reviewer}",
                )
                publish_review_ticket(
                    review_dir,
                    ticket_id=f"review-repair-candidate-alpha-r{reviewer}",
                    task_kind="correctness_review",
                    context_sha256=sha256_json(context),
                    prompt_sha256=sha256_bytes((review_dir / "prompts/correctness_review.md").read_bytes()),
                    schema_sha256=sha256_bytes((review_dir / "schemas/correctness_review.schema.json").read_bytes()),
                    parent_artifact_sha256=sha256_text(REPAIRED),
                )

            result = review_runner.run_repair(
                review_dir,
                ticket_id="repair-candidate-alpha",
                repair_id="repair-candidate-alpha",
                parent_candidate_text=CANDIDATE,
                finding_ledger=ledger,
            )

            self.assertTrue(result["repair_result"]["changed_candidate_bytes"])
            self.assertEqual(
                result["required_re_review_ticket_ids"],
                ["review-repair-candidate-alpha-r1", "review-repair-candidate-alpha-r2"],
            )
            self.assertEqual(
                [event["to_state"] for event in (
                    json.loads(line)
                    for line in (
                        review_dir / "tickets/repair-candidate-alpha/ticket_events.jsonl"
                    ).read_text().splitlines()
                )],
                ["admitted", "running", "completed", "accepted"],
            )

    def test_malformed_review_action_appends_failed_ticket_event(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            cli = root / "fake_review_cli.py"
            write_fake_cli(cli, "classification_mutates")
            review_dir = write_review_dir(root, cli)
            frozen = {
                "correctness_reconciliation_sha256": "c" * 64,
                "candidate_sha256": sha256_text(CANDIDATE),
                "correctness_bytes_sha256": "e" * 64,
            }
            reference_cards = [
                {
                    "reference_id": "jin",
                    "mechanism_summary": "Known public mechanism summary.",
                    "reference_sha256": "f" * 64,
                }
            ]
            context = review_contracts.build_mechanism_classification_context(
                classification_id="classify-candidate-alpha",
                theorem_text="Theorem.",
                candidate_text=CANDIDATE,
                frozen_correctness=frozen,
                reference_cards=reference_cards,
                ticket_id="classify-candidate-alpha",
            )
            publish_review_ticket(
                review_dir,
                ticket_id="classify-candidate-alpha",
                task_kind="mechanism_classification",
                context_sha256=sha256_json(context),
                prompt_sha256=sha256_bytes((review_dir / "prompts/mechanism_classification.md").read_bytes()),
                schema_sha256=sha256_bytes((review_dir / "schemas/mechanism_classification.schema.json").read_bytes()),
                parent_artifact_sha256=sha256_text(CANDIDATE),
            )

            result = review_runner.run_mechanism_classification(
                review_dir,
                ticket_id="classify-candidate-alpha",
                classification_id="classify-candidate-alpha",
                theorem_text="Theorem.",
                candidate_text=CANDIDATE,
                frozen_correctness=frozen,
                reference_cards=reference_cards,
            )

            self.assertEqual(result["receipt"]["status"], "malformed")
            events = [
                json.loads(line)
                for line in (
                    review_dir / "tickets/classify-candidate-alpha/ticket_events.jsonl"
                ).read_text().splitlines()
            ]
            self.assertEqual([event["to_state"] for event in events], ["admitted", "running", "failed"])

    def test_mechanism_classification_runs_after_frozen_correctness_and_cannot_mutate_it(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            cli = root / "fake_review_cli.py"
            write_fake_cli(cli, "classification_mutates")
            review_dir = write_review_dir(root, cli)
            frozen = {
                "correctness_reconciliation_sha256": "c" * 64,
                "candidate_sha256": sha256_text(CANDIDATE),
                "correctness_bytes_sha256": "e" * 64,
            }
            reference_cards = [
                {
                    "reference_id": "jin",
                    "mechanism_summary": "Known public mechanism summary.",
                    "reference_sha256": "f" * 64,
                }
            ]
            context = review_contracts.build_mechanism_classification_context(
                classification_id="classify-candidate-alpha",
                theorem_text="Theorem.",
                candidate_text=CANDIDATE,
                frozen_correctness=frozen,
                reference_cards=reference_cards,
                ticket_id="classify-candidate-alpha",
            )
            publish_review_ticket(
                review_dir,
                ticket_id="classify-candidate-alpha",
                task_kind="mechanism_classification",
                context_sha256=sha256_json(context),
                prompt_sha256=sha256_bytes((review_dir / "prompts/mechanism_classification.md").read_bytes()),
                schema_sha256=sha256_bytes((review_dir / "schemas/mechanism_classification.schema.json").read_bytes()),
                parent_artifact_sha256=sha256_text(CANDIDATE),
            )

            result = review_runner.run_mechanism_classification(
                review_dir,
                ticket_id="classify-candidate-alpha",
                classification_id="classify-candidate-alpha",
                theorem_text="Theorem.",
                candidate_text=CANDIDATE,
                frozen_correctness=frozen,
                reference_cards=reference_cards,
            )

            self.assertEqual(result["receipt"]["status"], "malformed")
            self.assertIsNone(result["classification"])


if __name__ == "__main__":
    unittest.main()
