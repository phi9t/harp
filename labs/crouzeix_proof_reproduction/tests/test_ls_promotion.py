from __future__ import annotations

import hashlib
import inspect
import json
import os
import shutil
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import ls_contract  # noqa: E402
import ls_receipts  # noqa: E402
import ls_validation  # noqa: E402
import protocol  # noqa: E402
from test_ls_receipts import FakeExecutor, make_workspace  # noqa: E402

import ls_promotion  # noqa: E402


FORMAL_TARGET_RELATIVE = Path(
    "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger"
)
GRAPH_NAME = "source-graph.json"
INVENTORY_NAME = "library-inventory.json"
PROMOTION_NAME = "promotion.json"
HISTORICAL_TARGET_ROOT = Path(__file__).resolve().parent / "fixtures/ls_historical"


def canonical_json_bytes(value: object) -> bytes:
    return (
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
        + b"\n"
    )


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def seed_historical_pair(formal_target_root: Path) -> None:
    graph = HISTORICAL_TARGET_ROOT / GRAPH_NAME
    inventory = HISTORICAL_TARGET_ROOT / INVENTORY_NAME
    if sha256_file(graph) != ls_validation.LS_HISTORICAL_GRAPH_SHA256:
        raise AssertionError("historical LS graph fixture digest drifted")
    if sha256_file(inventory) != ls_validation.LS_HISTORICAL_INVENTORY_SHA256:
        raise AssertionError("historical LS inventory fixture digest drifted")
    shutil.copy2(graph, formal_target_root / GRAPH_NAME)
    shutil.copy2(inventory, formal_target_root / INVENTORY_NAME)


def published_candidate_receipts(repository_root: Path) -> dict[str, dict[str, str]]:
    formal_target_root = repository_root / FORMAL_TARGET_RELATIVE
    seed_historical_pair(formal_target_root)
    rows = ls_validation.load_route_graph(formal_target_root / GRAPH_NAME)
    return ls_receipts.publish_ls_receipts(
        rows,
        repository_root,
        formal_target_root,
        executor=FakeExecutor(),
    )


def candidate_roster(repository_root: Path) -> tuple[dict[str, str], ...]:
    published = published_candidate_receipts(repository_root)
    return tuple(
        {
            "node_id": node_id,
            "attempt_path": published[node_id]["attempt_path"],
            "receipt_sha256": published[node_id]["receipt_sha256"],
        }
        for node_id in ls_contract.NODE_ORDER
    )


def transaction_roots(formal_target_root: Path) -> list[Path]:
    return sorted(formal_target_root.glob(f"{ls_promotion.TRANSACTION_PREFIX}*"))


def stage_entries(formal_target_root: Path) -> list[str]:
    return sorted(
        path.name
        for path in formal_target_root.iterdir()
        if path.name.endswith(".stage") or path.name.endswith(".restore")
    )


def run_promotion_with_hook(
    repository_root: Path,
    candidates: tuple[dict[str, str], ...],
    hook,
) -> None:
    with mock.patch.object(ls_promotion, "_HOOK", hook):
        ls_promotion.promote_six_node_route(repository_root, candidates)


def first_candidate_attempt(repository_root: Path, candidates: tuple[dict[str, str], ...]) -> Path:
    return repository_root / candidates[0]["attempt_path"]


class LSPromotionTests(unittest.TestCase):
    def test_public_signature_is_closed_two_argument_api(self) -> None:
        self.assertEqual(
            tuple(inspect.signature(ls_promotion.promote_six_node_route).parameters),
            ("repository_root", "candidate_receipts"),
        )

    def test_promotion_candidates_must_be_canonical_six_entry_roster(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, _ = make_workspace(Path(directory).resolve())
            candidates = candidate_roster(repository_root)

            with self.assertRaisesRegex(protocol.ValidationError, "exactly six"):
                ls_promotion.promote_six_node_route(repository_root, candidates[:-1])

            with self.assertRaisesRegex(protocol.ValidationError, "canonical LS node order"):
                ls_promotion.promote_six_node_route(
                    repository_root,
                    (
                        candidates[1],
                        candidates[0],
                        *candidates[2:],
                    ),
                )

            with self.assertRaisesRegex(protocol.ValidationError, "candidate receipt fields"):
                ls_promotion.promote_six_node_route(
                    repository_root,
                    tuple(
                        {
                            "node_id": item["node_id"],
                            "attempt_path": item["attempt_path"],
                            "receipt_sha256": item["receipt_sha256"],
                            "status": "passed",
                        }
                        for item in candidates
                    ),
                )

    def test_load_route_state_accepts_current_historical_pair_without_marker(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)

            state = ls_validation.load_route_state(formal_target_root)

            self.assertEqual(
                tuple(row.status for row in state["graph"]),
                ("passed", "blocked", "passed", "blocked", "blocked", "blocked"),
            )
            self.assertIsNone(state["promotion"])
            self.assertEqual(
                sha256_file(formal_target_root / GRAPH_NAME),
                state["graph_sha256"],
            )
            self.assertEqual(
                sha256_file(formal_target_root / INVENTORY_NAME),
                state["inventory_sha256"],
            )

    def test_load_route_state_rejects_new_graph_without_marker(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            del repository_root
            graph = {
                "schema_version": ls_contract.SCHEMA_VERSION,
                "source_id": ls_contract.SOURCE_ID,
                "source_identity": ls_contract.SOURCE_IDENTITY,
                "nodes": [
                    {
                        "node_id": node.node_id,
                        "source_locator": node.source_locator,
                        "statement_sha256": node.statement_sha256,
                        "lean_name": node.declaration,
                        "dependencies": list(node.dependencies),
                        "role": node.role,
                        "status": "passed",
                        "receipt_sha256": "0" * 64,
                    }
                    for node in ls_contract.NODES
                ],
            }
            inventory = {
                "schema_version": "crouzeix-ls-library-inventory/v1",
                "source_identity": ls_contract.SOURCE_IDENTITY,
                "facts": [
                    {
                        "fact_id": node.node_id,
                        "statement_sha256": node.statement_sha256,
                        "source_locator": node.source_locator,
                        "resolution": "local_compiled",
                    }
                    for node in ls_contract.NODES
                ],
            }
            (formal_target_root / GRAPH_NAME).write_bytes(canonical_json_bytes(graph))
            (formal_target_root / INVENTORY_NAME).write_bytes(canonical_json_bytes(inventory))

            with self.assertRaisesRegex(protocol.ValidationError, "promotion marker"):
                ls_validation.load_route_state(formal_target_root)

    def test_promote_six_node_route_writes_marker_bound_graph_and_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            original_graph = (formal_target_root / GRAPH_NAME).read_bytes()
            original_inventory = (formal_target_root / INVENTORY_NAME).read_bytes()
            candidates = candidate_roster(repository_root)

            ls_promotion.promote_six_node_route(repository_root, candidates)

            state = ls_validation.load_route_state(formal_target_root)
            marker = read_json(formal_target_root / PROMOTION_NAME)
            self.assertEqual(
                marker,
                {
                    "schema_version": "crouzeix-ls-promotion/v1",
                    "graph_sha256": sha256_file(formal_target_root / GRAPH_NAME),
                    "inventory_sha256": sha256_file(formal_target_root / INVENTORY_NAME),
                },
            )
            self.assertIsNotNone(state["promotion"])
            self.assertEqual(tuple(row.status for row in state["graph"]), ("passed",) * 6)
            self.assertEqual(
                [fact["fact_id"] for fact in state["inventory"]["facts"]],
                list(ls_contract.NODE_ORDER),
            )
            self.assertTrue(all(fact["resolution"] == "local_compiled" for fact in state["inventory"]["facts"]))
            self.assertNotEqual((formal_target_root / GRAPH_NAME).read_bytes(), original_graph)
            self.assertNotEqual((formal_target_root / INVENTORY_NAME).read_bytes(), original_inventory)
            self.assertEqual(ls_validation.load_route_graph(formal_target_root / GRAPH_NAME), state["graph"])

    def test_promote_rejects_duplicate_candidate_receipt_digest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, _ = make_workspace(Path(directory).resolve())
            candidates = list(candidate_roster(repository_root))
            candidates[1] = {
                "node_id": candidates[1]["node_id"],
                "attempt_path": candidates[1]["attempt_path"],
                "receipt_sha256": candidates[0]["receipt_sha256"],
            }
            with self.assertRaisesRegex(protocol.ValidationError, "duplicate.*receipt_sha256"):
                ls_promotion.promote_six_node_route(repository_root, tuple(candidates))

    def test_missing_current_module_fails_promotion(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)
            module_path = repository_root / ls_contract.NODES[0].build_target
            module_path.unlink()

            with self.assertRaisesRegex(protocol.ValidationError, "local module does not exist"):
                ls_promotion.promote_six_node_route(repository_root, candidates)

    def test_stale_wrapper_candidate_is_rejected_before_graph_inventory_or_marker_mutation(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            original_graph = (formal_target_root / GRAPH_NAME).read_bytes()
            original_inventory = (formal_target_root / INVENTORY_NAME).read_bytes()
            candidates = candidate_roster(repository_root)
            wrapper = repository_root / "scripts/check_lean_library.sh"
            wrapper.write_text("#!/bin/sh\n# stale wrapper\n", encoding="utf-8")

            with self.assertRaisesRegex(protocol.ValidationError, "wrapper_sha256"):
                ls_promotion.promote_six_node_route(repository_root, candidates)

            self.assertEqual((formal_target_root / GRAPH_NAME).read_bytes(), original_graph)
            self.assertEqual(
                (formal_target_root / INVENTORY_NAME).read_bytes(), original_inventory
            )
            self.assertFalse((formal_target_root / PROMOTION_NAME).exists())
            self.assertEqual(transaction_roots(formal_target_root), [])
            self.assertEqual(stage_entries(formal_target_root), [])

    def test_pre_marker_failures_restore_exact_original_snapshots(self) -> None:
        failpoints = (
            "before_graph_stage_write",
            "after_graph_stage_write",
            "before_graph_rename",
            "after_graph_rename",
            "after_graph_fsync",
            "before_inventory_stage_write",
            "after_inventory_stage_write",
            "before_inventory_rename",
            "after_inventory_rename",
            "after_inventory_fsync",
            "before_marker_stage_write",
            "after_marker_stage_write",
            "before_marker_rename",
        )
        for failpoint in failpoints:
            with self.subTest(failpoint=failpoint):
                with tempfile.TemporaryDirectory() as directory:
                    repository_root, formal_target_root = make_workspace(Path(directory).resolve())
                    seed_historical_pair(formal_target_root)
                    original_graph = (formal_target_root / GRAPH_NAME).read_bytes()
                    original_inventory = (formal_target_root / INVENTORY_NAME).read_bytes()
                    candidates = candidate_roster(repository_root)

                    def hook(name: str) -> None:
                        if name == failpoint:
                            raise RuntimeError(failpoint)

                    with self.assertRaisesRegex(RuntimeError, failpoint):
                        run_promotion_with_hook(repository_root, candidates, hook)

                    self.assertEqual((formal_target_root / GRAPH_NAME).read_bytes(), original_graph)
                    self.assertEqual(
                        (formal_target_root / INVENTORY_NAME).read_bytes(), original_inventory
                    )
                    self.assertFalse((formal_target_root / PROMOTION_NAME).exists())
                    self.assertEqual(transaction_roots(formal_target_root), [])
                    self.assertEqual(stage_entries(formal_target_root), [])
                    state = ls_validation.load_route_state(formal_target_root)
                    self.assertIsNone(state["promotion"])
                    self.assertEqual(
                        tuple(row.status for row in state["graph"]),
                        ("passed", "blocked", "passed", "blocked", "blocked", "blocked"),
                    )

    def test_retry_recovers_crash_residue_before_marker_commit(self) -> None:
        class SimulatedCrash(BaseException):
            pass

        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            original_graph = (formal_target_root / GRAPH_NAME).read_bytes()
            original_inventory = (formal_target_root / INVENTORY_NAME).read_bytes()
            candidates = candidate_roster(repository_root)

            def hook(name: str) -> None:
                if name == "after_inventory_rename":
                    raise SimulatedCrash("crash")

            with self.assertRaises(SimulatedCrash):
                run_promotion_with_hook(repository_root, candidates, hook)

            self.assertNotEqual((formal_target_root / GRAPH_NAME).read_bytes(), original_graph)
            self.assertNotEqual(
                (formal_target_root / INVENTORY_NAME).read_bytes(), original_inventory
            )
            self.assertFalse((formal_target_root / PROMOTION_NAME).exists())
            self.assertEqual(len(transaction_roots(formal_target_root)), 1)

            ls_promotion.promote_six_node_route(repository_root, candidates)

            state = ls_validation.load_route_state(formal_target_root)
            self.assertIsNotNone(state["promotion"])
            self.assertEqual(transaction_roots(formal_target_root), [])
            self.assertEqual(stage_entries(formal_target_root), [])

    def test_post_marker_failure_reports_committed_error_and_preserves_generation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)

            def hook(name: str) -> None:
                if name == "after_marker_fsync":
                    raise RuntimeError("post-marker")

            with self.assertRaises(ls_promotion.CommittedPromotionError) as caught:
                run_promotion_with_hook(repository_root, candidates, hook)

            self.assertIn("post-marker", str(caught.exception))
            state = ls_validation.load_route_state(formal_target_root)
            self.assertIsNotNone(state["promotion"])
            self.assertEqual(tuple(row.status for row in state["graph"]), ("passed",) * 6)

    def test_late_selected_receipt_mutation_before_marker_restores_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            original_graph = (formal_target_root / GRAPH_NAME).read_bytes()
            original_inventory = (formal_target_root / INVENTORY_NAME).read_bytes()
            candidates = candidate_roster(repository_root)
            attempt = first_candidate_attempt(repository_root, candidates)

            def hook(name: str) -> None:
                if name == "after_inventory_fsync":
                    (attempt / "receipt.json").write_text("{\"broken\":true}\n", encoding="utf-8")

            with self.assertRaisesRegex(protocol.ValidationError, "candidate receipt"):
                run_promotion_with_hook(repository_root, candidates, hook)

            self.assertEqual((formal_target_root / GRAPH_NAME).read_bytes(), original_graph)
            self.assertEqual((formal_target_root / INVENTORY_NAME).read_bytes(), original_inventory)
            self.assertFalse((formal_target_root / PROMOTION_NAME).exists())
            self.assertEqual(transaction_roots(formal_target_root), [])
            self.assertEqual(stage_entries(formal_target_root), [])

    def test_late_reachable_source_mutation_before_marker_restores_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            original_graph = (formal_target_root / GRAPH_NAME).read_bytes()
            original_inventory = (formal_target_root / INVENTORY_NAME).read_bytes()
            candidates = candidate_roster(repository_root)
            module_path = repository_root / ls_contract.NODES[0].build_target

            def hook(name: str) -> None:
                if name == "after_inventory_fsync":
                    module_path.write_text(
                        "import Mathlib.Algebra.Order.Ring.Defs\n-- mutated late source\n",
                        encoding="utf-8",
                    )

            with self.assertRaisesRegex(protocol.ValidationError, "candidate source closure"):
                run_promotion_with_hook(repository_root, candidates, hook)

            self.assertEqual((formal_target_root / GRAPH_NAME).read_bytes(), original_graph)
            self.assertEqual((formal_target_root / INVENTORY_NAME).read_bytes(), original_inventory)
            self.assertFalse((formal_target_root / PROMOTION_NAME).exists())
            self.assertEqual(transaction_roots(formal_target_root), [])
            self.assertEqual(stage_entries(formal_target_root), [])

    def test_pre_marker_rollback_failure_surfaces_original_and_rollback_errors(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            original_graph = (formal_target_root / GRAPH_NAME).read_bytes()
            original_inventory = (formal_target_root / INVENTORY_NAME).read_bytes()
            candidates = candidate_roster(repository_root)

            original_replace = ls_promotion._replace_file_from_transaction

            def hook(name: str) -> None:
                if name == "after_graph_fsync":
                    raise RuntimeError("original pre-marker failure")

            def failing_replace(target_directory, transaction, source_name: str, destination_name: str) -> None:
                if (
                    source_name == "original-graph.json"
                    and destination_name == GRAPH_NAME
                ):
                    raise OSError("rollback restore failure")
                original_replace(target_directory, transaction, source_name, destination_name)

            with mock.patch.object(
                ls_promotion,
                "_replace_file_from_transaction",
                side_effect=failing_replace,
            ):
                with self.assertRaises(ls_promotion.RollbackPromotionError) as caught:
                    run_promotion_with_hook(repository_root, candidates, hook)

            self.assertIsInstance(caught.exception.original_error, RuntimeError)
            self.assertEqual(str(caught.exception.original_error), "original pre-marker failure")
            self.assertIsInstance(caught.exception.rollback_error, OSError)
            self.assertEqual(str(caught.exception.rollback_error), "rollback restore failure")
            self.assertEqual((formal_target_root / INVENTORY_NAME).read_bytes(), original_inventory)
            self.assertFalse((formal_target_root / PROMOTION_NAME).exists())
            self.assertNotEqual((formal_target_root / GRAPH_NAME).read_bytes(), original_graph)

    def test_rename_no_replace_rejects_unsupported_platform_without_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            source = root / "marker.stage"
            destination = root / PROMOTION_NAME
            source.write_text("staged-marker\n", encoding="utf-8")
            destination.write_text("existing-marker\n", encoding="utf-8")

            parent_fd = os.open(root, os.O_RDONLY)
            try:
                with mock.patch.object(ls_promotion.platform, "system", return_value="Plan9"):
                    with mock.patch.object(ls_promotion.os, "replace") as replace_mock:
                        with self.assertRaisesRegex(
                            protocol.ValidationError, "unsupported platform"
                        ):
                            ls_promotion._rename_file_no_replace_at(
                                parent_fd, source.name, destination.name
                            )
                replace_mock.assert_not_called()
            finally:
                os.close(parent_fd)

            self.assertEqual(source.read_text(encoding="utf-8"), "staged-marker\n")
            self.assertEqual(destination.read_text(encoding="utf-8"), "existing-marker\n")

    def test_transaction_creation_failure_cleans_partial_owned_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            original_graph = (formal_target_root / GRAPH_NAME).read_bytes()
            original_inventory = (formal_target_root / INVENTORY_NAME).read_bytes()
            candidates = candidate_roster(repository_root)

            original_write = ls_promotion._write_file_create_only_at
            calls = {"count": 0}

            def failing_write(parent_fd: int, name: str, data: bytes) -> None:
                calls["count"] += 1
                if calls["count"] == 3:
                    raise OSError("tx create failure")
                original_write(parent_fd, name, data)

            with mock.patch.object(ls_promotion, "_write_file_create_only_at", side_effect=failing_write):
                with self.assertRaisesRegex(OSError, "tx create failure"):
                    ls_promotion.promote_six_node_route(repository_root, candidates)

            self.assertEqual((formal_target_root / GRAPH_NAME).read_bytes(), original_graph)
            self.assertEqual((formal_target_root / INVENTORY_NAME).read_bytes(), original_inventory)
            self.assertFalse((formal_target_root / PROMOTION_NAME).exists())
            self.assertEqual(transaction_roots(formal_target_root), [])
            self.assertEqual(stage_entries(formal_target_root), [])

    def test_transaction_creation_base_exception_recovery_on_retry(self) -> None:
        class SimulatedCrash(BaseException):
            pass

        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)

            original_write = ls_promotion._write_file_create_only_at
            calls = {"count": 0}

            def crashing_write(parent_fd: int, name: str, data: bytes) -> None:
                calls["count"] += 1
                if calls["count"] == 3:
                    raise SimulatedCrash("tx create crash")
                original_write(parent_fd, name, data)

            with mock.patch.object(ls_promotion, "_write_file_create_only_at", side_effect=crashing_write):
                with self.assertRaises(SimulatedCrash):
                    ls_promotion.promote_six_node_route(repository_root, candidates)

            self.assertEqual(len(transaction_roots(formal_target_root)), 1)
            self.assertFalse((formal_target_root / PROMOTION_NAME).exists())

            ls_promotion.promote_six_node_route(repository_root, candidates)

            state = ls_validation.load_route_state(formal_target_root)
            self.assertIsNotNone(state["promotion"])
            self.assertEqual(transaction_roots(formal_target_root), [])
            self.assertEqual(stage_entries(formal_target_root), [])

    def test_post_marker_partial_transaction_cleanup_surfaces_committed_error_and_retry_cleans(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)

            original_remove = ls_promotion._remove_transaction
            fail_once = {"used": False}

            def partial_remove(transaction) -> None:
                if not fail_once["used"]:
                    fail_once["used"] = True
                    ls_promotion._cleanup_stage(transaction.pinned.descriptor, "marker.json")
                    raise OSError("partial cleanup failure")
                original_remove(transaction)

            with mock.patch.object(ls_promotion, "_remove_transaction", side_effect=partial_remove):
                with self.assertRaises(ls_promotion.CommittedPromotionError) as caught:
                    ls_promotion.promote_six_node_route(repository_root, candidates)

            self.assertIn("partial cleanup failure", str(caught.exception))
            state = ls_validation.load_route_state(formal_target_root)
            self.assertIsNotNone(state["promotion"])
            self.assertEqual(len(transaction_roots(formal_target_root)), 1)

            ls_promotion.promote_six_node_route(repository_root, candidates)

            self.assertEqual(transaction_roots(formal_target_root), [])
            self.assertEqual(stage_entries(formal_target_root), [])
            state = ls_validation.load_route_state(formal_target_root)
            self.assertIsNotNone(state["promotion"])

    def test_post_marker_lock_release_failure_surfaces_committed_error_and_retry_recovers(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)

            original_release = ls_promotion._release_lock
            fail_once = {"used": False}

            def failing_release(target_directory, lock) -> None:
                if not fail_once["used"]:
                    fail_once["used"] = True
                    raise OSError("release failure")
                original_release(target_directory, lock)

            with mock.patch.object(ls_promotion, "_release_lock", side_effect=failing_release):
                with self.assertRaises(ls_promotion.CommittedPromotionError) as caught:
                    ls_promotion.promote_six_node_route(repository_root, candidates)

            self.assertIn("release failure", str(caught.exception))
            state = ls_validation.load_route_state(formal_target_root)
            self.assertIsNotNone(state["promotion"])
            self.assertTrue((formal_target_root / ls_promotion.LOCK_NAME).exists())

            ls_promotion.promote_six_node_route(repository_root, candidates)

            self.assertFalse((formal_target_root / ls_promotion.LOCK_NAME).exists())
            self.assertEqual(transaction_roots(formal_target_root), [])

    def test_marker_unknown_field_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)
            ls_promotion.promote_six_node_route(repository_root, candidates)

            marker = read_json(formal_target_root / PROMOTION_NAME)
            marker["unexpected"] = True
            (formal_target_root / PROMOTION_NAME).write_bytes(canonical_json_bytes(marker))

            with self.assertRaisesRegex(protocol.ValidationError, "fields are invalid"):
                ls_validation.load_route_state(formal_target_root)

    def test_marker_digest_mismatch_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)
            ls_promotion.promote_six_node_route(repository_root, candidates)

            marker = read_json(formal_target_root / PROMOTION_NAME)
            marker["graph_sha256"] = "0" * 64
            (formal_target_root / PROMOTION_NAME).write_bytes(canonical_json_bytes(marker))

            with self.assertRaisesRegex(protocol.ValidationError, "graph_sha256 mismatch"):
                ls_validation.load_route_state(formal_target_root)

    def test_duplicate_marker_json_key_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)
            ls_promotion.promote_six_node_route(repository_root, candidates)
            graph_sha = sha256_file(formal_target_root / GRAPH_NAME)
            inventory_sha = sha256_file(formal_target_root / INVENTORY_NAME)
            payload = (
                "{"
                "\"schema_version\":\"crouzeix-ls-promotion/v1\","
                f"\"graph_sha256\":\"{graph_sha}\","
                f"\"graph_sha256\":\"{graph_sha}\","
                f"\"inventory_sha256\":\"{inventory_sha}\""
                "}\n"
            )
            (formal_target_root / PROMOTION_NAME).write_text(payload, encoding="utf-8")

            with self.assertRaisesRegex(protocol.ValidationError, "duplicate JSON key"):
                ls_validation.load_route_state(formal_target_root)

    def test_second_successful_call_is_idempotent_without_marker_rewrite(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)
            ls_promotion.promote_six_node_route(repository_root, candidates)
            marker_stat = (formal_target_root / PROMOTION_NAME).stat()
            graph_stat = (formal_target_root / GRAPH_NAME).stat()
            inventory_stat = (formal_target_root / INVENTORY_NAME).stat()

            time.sleep(0.02)
            ls_promotion.promote_six_node_route(repository_root, candidates)

            self.assertEqual((formal_target_root / PROMOTION_NAME).stat().st_mtime_ns, marker_stat.st_mtime_ns)
            self.assertEqual((formal_target_root / GRAPH_NAME).stat().st_mtime_ns, graph_stat.st_mtime_ns)
            self.assertEqual(
                (formal_target_root / INVENTORY_NAME).stat().st_mtime_ns,
                inventory_stat.st_mtime_ns,
            )

    def test_existing_live_lock_from_active_publisher_causes_collision(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)
            start = threading.Event()
            release = threading.Event()
            worker_done = threading.Event()
            worker_error: list[BaseException] = []

            def hook(name: str) -> None:
                if name == "before_graph_stage_write":
                    start.set()
                    release.wait(timeout=5)

            def worker() -> None:
                try:
                    run_promotion_with_hook(repository_root, candidates, hook)
                except BaseException as error:  # pragma: no cover - captured for assertion
                    worker_error.append(error)
                finally:
                    worker_done.set()

            thread = threading.Thread(target=worker)
            thread.start()
            self.assertTrue(start.wait(timeout=5))
            try:
                with self.assertRaises(ls_promotion.PromotionCollisionError):
                    ls_promotion.promote_six_node_route(repository_root, candidates)
            finally:
                release.set()
                self.assertTrue(worker_done.wait(timeout=5))
                thread.join(timeout=5)
            self.assertEqual(worker_error, [])

    def test_candidate_receipt_symlink_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)
            attempt = repository_root / candidates[0]["attempt_path"]
            receipt_path = attempt / "receipt.json"
            backup = attempt / "receipt-backup.json"
            receipt_path.rename(backup)
            receipt_path.symlink_to(backup.name)

            with self.assertRaisesRegex(protocol.ValidationError, "cannot be a symlink"):
                ls_promotion.promote_six_node_route(repository_root, candidates)

    def test_candidate_receipt_hardlink_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)
            attempt = repository_root / candidates[0]["attempt_path"]
            receipt_path = attempt / "receipt.json"
            backup = attempt / "receipt-backup.json"
            receipt_path.rename(backup)
            os.link(backup, receipt_path)

            with self.assertRaisesRegex(protocol.ValidationError, "exactly one hard link"):
                ls_promotion.promote_six_node_route(repository_root, candidates)

    def test_candidate_receipt_oversize_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(Path(directory).resolve())
            seed_historical_pair(formal_target_root)
            candidates = candidate_roster(repository_root)
            attempt = repository_root / candidates[0]["attempt_path"]
            (attempt / "receipt.json").write_bytes(b"x" * (ls_validation.MAX_JSON_BYTES + 1))

            with self.assertRaisesRegex(protocol.ValidationError, "exceeds byte cap"):
                ls_promotion.promote_six_node_route(repository_root, candidates)


if __name__ == "__main__":
    unittest.main()
