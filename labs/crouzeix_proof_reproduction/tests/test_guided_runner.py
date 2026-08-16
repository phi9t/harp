from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import guided_runner
import protocol
import tickets


THEOREM_TEXT = "For every square matrix A, prove the Crouzeix bound."


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


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def mechanism_card(*, include_digest: bool = True) -> dict[str, object]:
    card: dict[str, object] = {
        "schema_version": "crouzeix-guided-mechanism-card/v1",
        "card_id": "jin-mechanism-card",
        "source_family": "public_jin_mechanism_summary",
        "attribution": [
            {
                "source_id": "public-jin-proof-note",
                "locator": "source.md#L10-L24",
                "claim": "Uses an attributed mechanism summary, not manuscript bytes.",
            }
        ],
        "mechanism_summary": "Reduce the numerical-range estimate to a positive-kernel construction.",
        "allowed_guidance": [
            "Use the named positive-kernel mechanism as a diagnostic hint.",
            "Reconstruct all proof steps from the theorem statement and card.",
        ],
        "excluded_materials": [
            "manuscript bytes",
            "formalization bytes",
            "source proof text",
        ],
    }
    if include_digest:
        card["mechanism_card_sha256"] = sha256_json(card)
    return card


def make_source_inputs(root: Path) -> tuple[Path, Path]:
    theorem = root / "theorem.txt"
    card = root / "card.json"
    theorem.write_text(THEOREM_TEXT, encoding="utf-8")
    write_json(card, mechanism_card())
    return theorem, card


def guided_payload(*, endpoint_kind: str = "candidate_proof") -> dict[str, object]:
    endpoint_text = (
        "Guided candidate proof with reconstructed positive-kernel steps."
        if endpoint_kind == "candidate_proof"
        else "The supplied mechanism card is insufficient to close the dilation gap."
    )
    return {
        "schema_version": "crouzeix-guided-result/v1",
        "run_id": "guided-001",
        "ticket_id": "guided-reconstruction",
        "mechanism_card_sha256": mechanism_card()["mechanism_card_sha256"],
        "theorem_sha256": sha256_text(THEOREM_TEXT),
        "endpoint": {"kind": endpoint_kind, "text": endpoint_text},
        "used_guidance": ["positive-kernel construction"],
        "unproved_obligations": [],
        "confidence_basis": "fake provider fixture",
    }


class FakeGuidedProvider:
    def __init__(self, payload: dict[str, object] | None = None, status: str = "completed") -> None:
        self.payload = payload or guided_payload()
        self.status = status
        self.calls: list[dict[str, object]] = []

    def run_guided(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> guided_runner.ProviderAttempt:
        self.calls.append({"run_dir": run_dir, "ticket": ticket, "context": context})
        if self.status != "completed":
            return guided_runner.ProviderAttempt(
                terminal_status=self.status,
                provider_call={"kind": "fake-guided"},
                receipt={"status": self.status, "ticket_id": ticket["ticket_id"]},
                validated_output=None,
                reason=f"fake {self.status}",
            )
        return guided_runner.ProviderAttempt(
            terminal_status="completed",
            provider_call={"kind": "fake-guided"},
            receipt={"status": "completed", "ticket_id": ticket["ticket_id"]},
            validated_output=self.payload,
            reason=None,
        )


class GuidedRunnerTests(unittest.TestCase):
    def test_trigger_truth_table_is_pure_and_preflight_gated(self) -> None:
        self.assertTrue(
            guided_runner.should_run_guided(
                h_correctness_outcome="incomplete",
                e_correctness_outcome="invalid",
                resource_preflight_passed=True,
            )
        )
        for h_outcome, e_outcome, preflight in (
            ("complete", "incomplete", True),
            ("incomplete", "complete", True),
            ("invalid", "indeterminate", False),
        ):
            self.assertFalse(
                guided_runner.should_run_guided(
                    h_correctness_outcome=h_outcome,
                    e_correctness_outcome=e_outcome,
                    resource_preflight_passed=preflight,
                )
            )
        with self.assertRaisesRegex(protocol.ValidationError, "correctness outcome"):
            guided_runner.should_run_guided(
                h_correctness_outcome="self_judged",
                e_correctness_outcome="incomplete",
                resource_preflight_passed=True,
            )

    def test_false_trigger_preparation_writes_cancellation_receipt_and_no_call_state(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            theorem, card = make_source_inputs(root)
            run_dir = root / "guided-001"

            receipt = guided_runner.prepare_guided_run(
                run_dir,
                theorem_path=theorem,
                mechanism_card_path=card,
                cli_identity={"path": "/bin/echo", "version": "fake", "sha256": "1" * 64},
                model="fake-model",
                h_correctness_outcome="complete",
                e_correctness_outcome="incomplete",
                resource_preflight_passed=True,
            )

            self.assertEqual(receipt["status"], "not_applicable")
            self.assertFalse((run_dir / "calls").exists())
            self.assertFalse((run_dir / "tickets").exists())
            self.assertTrue((run_dir / "guided_trigger.json").is_file())

    def test_prepare_guided_run_validates_l3_card_and_binds_one_call_budget(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            theorem, card = make_source_inputs(root)
            run_dir = root / "guided-001"

            receipt = guided_runner.prepare_guided_run(
                run_dir,
                theorem_path=theorem,
                mechanism_card_path=card,
                cli_identity={"path": "/bin/echo", "version": "fake", "sha256": "1" * 64},
                model="fake-model",
                h_correctness_outcome="incomplete",
                e_correctness_outcome="invalid",
                resource_preflight_passed=True,
            )

            self.assertEqual(receipt["status"], "prepared")
            spec = protocol.read_run_spec(run_dir / "run_spec.json")
            self.assertEqual(spec["arm"], "guided")
            self.assertEqual(spec["leakage"], "L3")
            self.assertEqual(spec["max_calls"], 1)
            self.assertEqual(spec["guided"]["mechanism_card_sha256"], mechanism_card()["mechanism_card_sha256"])
            self.assertEqual(spec["guided"]["theorem_sha256"], sha256_text(THEOREM_TEXT))
            self.assertIn("public proof manuscripts", spec["generation_excluded_classes"])
            self.assertIn("inputs/mechanism_card.json", spec["generation_visible_files"])
            self.assertTrue((run_dir / "prompts/guided_reconstruction.md").is_file())
            self.assertTrue((run_dir / "schemas/guided_result.schema.json").is_file())
            ticket = json.loads((run_dir / "tickets/guided-reconstruction/ticket.json").read_text())
            self.assertEqual(ticket["task_kind"], "expert")
            self.assertEqual(ticket["owner_type"], "expert")
            self.assertEqual(ticket["allowed_tools"], ["Write"])
            self.assertEqual(ticket["context_sha256"], sha256_json(json.loads((run_dir / "contexts/guided-reconstruction.json").read_text())))

            with self.assertRaisesRegex(protocol.ValidationError, "existing"):
                guided_runner.prepare_guided_run(
                    run_dir,
                    theorem_path=theorem,
                    mechanism_card_path=card,
                    cli_identity={"path": "/bin/echo", "version": "fake", "sha256": "1" * 64},
                    model="fake-model",
                    h_correctness_outcome="incomplete",
                    e_correctness_outcome="invalid",
                    resource_preflight_passed=True,
                )

    def test_prepare_rejects_mechanism_card_with_disallowed_bytes(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            theorem = root / "theorem.txt"
            theorem.write_text(THEOREM_TEXT, encoding="utf-8")
            card = mechanism_card(include_digest=False)
            card["manuscript_excerpt"] = "proof bytes from a public manuscript"
            card["mechanism_card_sha256"] = sha256_json(card)
            card_path = root / "bad-card.json"
            write_json(card_path, card)

            with self.assertRaisesRegex(protocol.ValidationError, "mechanism card fields"):
                guided_runner.prepare_guided_run(
                    root / "guided-001",
                    theorem_path=theorem,
                    mechanism_card_path=card_path,
                    cli_identity={"path": "/bin/echo", "version": "fake", "sha256": "1" * 64},
                    model="fake-model",
                    h_correctness_outcome="incomplete",
                    e_correctness_outcome="invalid",
                    resource_preflight_passed=True,
                )

    def test_run_guided_reconstruction_writes_typed_candidate_receipt(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = self._prepared_run(Path(directory))
            provider = FakeGuidedProvider()

            receipt = guided_runner.run_guided_reconstruction(run_dir, provider=provider)

            self.assertEqual(receipt["terminal_status"], "candidate")
            self.assertEqual(len(provider.calls), 1)
            context = provider.calls[0]["context"]
            self.assertEqual(context["delegation_allowed"], False)
            self.assertEqual(context["allowed_tools"], ["Write"])
            self.assertEqual(context["leakage"], "L3")
            self.assertNotIn("manuscript_excerpt", json.dumps(context))
            self.assertTrue((run_dir / "guided_candidate/candidate.tex").is_file())
            candidate_receipt = json.loads((run_dir / "guided_candidate/receipt.json").read_text())
            self.assertEqual(candidate_receipt["endpoint"]["kind"], "candidate_proof")
            self.assertEqual(
                candidate_receipt["candidate_sha256"],
                sha256_text(provider.payload["endpoint"]["text"]),
            )
            events = (run_dir / "tickets/guided-reconstruction/ticket_events.jsonl").read_text().splitlines()
            self.assertEqual([json.loads(line)["to_state"] for line in events], ["admitted", "running", "completed", "accepted"])

    def test_run_guided_reconstruction_writes_typed_blocker_receipt(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = self._prepared_run(Path(directory))

            receipt = guided_runner.run_guided_reconstruction(
                run_dir,
                provider=FakeGuidedProvider(guided_payload(endpoint_kind="blocker")),
            )

            self.assertEqual(receipt["terminal_status"], "blocker")
            blocker_receipt = json.loads((run_dir / "guided_blocker/receipt.json").read_text())
            self.assertEqual(blocker_receipt["endpoint"]["kind"], "blocker")
            self.assertIsNone(blocker_receipt["candidate_sha256"])
            self.assertEqual(blocker_receipt["blocker_sha256"], sha256_text(blocker_receipt["endpoint"]["text"]))

    def test_run_guided_reconstruction_records_provider_failure_without_candidate(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = self._prepared_run(Path(directory))

            receipt = guided_runner.run_guided_reconstruction(
                run_dir,
                provider=FakeGuidedProvider(status="timed_out"),
            )

            self.assertEqual(receipt["terminal_status"], "timed_out")
            self.assertFalse((run_dir / "guided_candidate").exists())
            events = (run_dir / "tickets/guided-reconstruction/ticket_events.jsonl").read_text().splitlines()
            self.assertEqual([json.loads(line)["to_state"] for line in events], ["admitted", "running", "timed_out"])

    def test_run_guided_reconstruction_rejects_malformed_provider_output(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = self._prepared_run(Path(directory))
            payload = guided_payload()
            payload["mechanism_card_sha256"] = "2" * 64

            receipt = guided_runner.run_guided_reconstruction(
                run_dir,
                provider=FakeGuidedProvider(payload),
            )

            self.assertEqual(receipt["terminal_status"], "failed")
            self.assertFalse((run_dir / "guided_candidate").exists())
            events = (run_dir / "tickets/guided-reconstruction/ticket_events.jsonl").read_text().splitlines()
            self.assertEqual([json.loads(line)["to_state"] for line in events], ["admitted", "running", "failed"])

    def test_check_reconciles_prepared_run_and_terminal_receipt(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = self._prepared_run(Path(directory))
            guided_runner.run_guided_reconstruction(run_dir, provider=FakeGuidedProvider())

            check = guided_runner.check_guided_run(run_dir)

            self.assertEqual(check["status"], "ok")
            self.assertEqual(check["call_count"], 1)
            self.assertEqual(check["terminal_status"], "candidate")

    def _prepared_run(self, root: Path) -> Path:
        theorem, card = make_source_inputs(root)
        run_dir = root / "guided-001"
        guided_runner.prepare_guided_run(
            run_dir,
            theorem_path=theorem,
            mechanism_card_path=card,
            cli_identity={"path": "/bin/echo", "version": "fake", "sha256": "1" * 64},
            model="fake-model",
            h_correctness_outcome="incomplete",
            e_correctness_outcome="invalid",
            resource_preflight_passed=True,
        )
        return run_dir


if __name__ == "__main__":
    unittest.main()
