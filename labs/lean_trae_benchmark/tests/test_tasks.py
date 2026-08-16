from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


sys.path.insert(0, str(Path(__file__).parents[1]))

from model import ValidationError, sha256_bytes
from tasks import load_task_manifest, materialize_candidate


RUNTIME_ID = "lean-trae-test-v1"


def _ref(path: str, data: bytes) -> dict[str, object]:
    return {
        "path": path,
        "bytes": len(data),
        "digest": {"algorithm": "sha256", "hex": sha256_bytes(data).hex},
    }


def _write(root: Path, relative: str, data: bytes) -> dict[str, object]:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return _ref(relative, data)


def _slot(slot_id: str, source_path: str, prefix: dict[str, object], suffix: dict[str, object]) -> dict[str, object]:
    return {
        "slot_id": slot_id,
        "source_path": source_path,
        "declaration_name": f"task_{slot_id}",
        "declaration_type": "Prop",
        "prefix": prefix,
        "suffix": suffix,
        "mode": "proof_body",
        "forbidden_tokens": ["import", "sorry"],
    }


def _manifest(
    root: Path,
    *,
    strategy: str = "single_slot",
    slots: list[dict[str, object]] | None = None,
    layout: dict[str, object] | None = None,
    runtime_id: str = RUNTIME_ID,
) -> Path:
    prompt = _write(root, "prompt.txt", b"Return only the requested proof body.\n")
    if slots is None:
        prefix = _write(root, "sealed/Task.prefix.lean", b"theorem task_target : True := ")
        suffix = _write(root, "sealed/Task.suffix.lean", b"\n")
        slots = [_slot("target", "Candidate.lean", prefix, suffix)]
    value: dict[str, object] = {
        "schema_version": "harp-lean-task-manifest/v1",
        "seal": {
            "schema_version": "harp-lean-task-seal/v1",
            "task_id": "sealed-task",
            "profile": "core-proposition",
            "runtime_id": runtime_id,
            "generation_budget": 1,
            "repair_budget": 2,
            "candidate_encoding": "utf-8-proof-body",
            "slots": slots,
            "slot_order": [item["slot_id"] for item in slots],
            "project_inventory": [],
            "command_policy": {
                "commands": [{"argv": ["lean", "Candidate.lean"], "cwd": "."}],
                "writable_paths": ["Scratch.lean", "Candidate.lean"],
                "network": False,
            },
            "prompt": prompt,
        },
        "strategy": strategy,
        "layout": layout if layout is not None else {"kind": "slots"},
    }
    manifest = root / "manifest.json"
    manifest.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")
    return manifest


class TaskManifestTests(unittest.TestCase):
    def test_canonical_core_task_is_a_sealed_single_slot_manifest(self) -> None:
        task_root = Path(__file__).parents[1] / "tasks" / "core-and-left"
        task = load_task_manifest(task_root / "manifest.json", "lean-trae-darwin-arm64-v1")
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            materialize_candidate(task, "by\n  intro h\n  exact h.1", output)
            self.assertEqual(
                (output / "Candidate.lean").read_text(encoding="utf-8"),
                "theorem core_and_left (a b : Prop) : a ∧ b → a :=\nby\n  intro h\n  exact h.1\n",
            )

    def test_single_slot_materializes_only_a_raw_by_proof_body(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            task = load_task_manifest(_manifest(root), RUNTIME_ID)
            output = Path(directory) / "output"
            output.mkdir()

            result = materialize_candidate(task, "by\n  trivial", output)

            self.assertEqual(result.slot_ids, ("target",))
            self.assertEqual(
                (output / "Candidate.lean").read_text(encoding="utf-8"),
                "theorem task_target : True := by\n  trivial\n",
            )

            second = Path(directory) / "second"
            second.mkdir()
            materialize_candidate(task, "by\n  by_cases h : True\n  · trivial\n  · contradiction", second)

    def test_multi_slot_uses_declared_fixed_order_and_canonical_json(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            first_prefix = _write(root, "sealed/First.prefix", b"theorem task_first : True := ")
            first_suffix = _write(root, "sealed/First.suffix", b"\n")
            second_prefix = _write(root, "sealed/Second.prefix", b"theorem task_second : True := ")
            second_suffix = _write(root, "sealed/Second.suffix", b"\n")
            manifest = _manifest(
                root,
                strategy="multi_slot",
                slots=[
                    _slot("first", "First.lean", first_prefix, first_suffix),
                    _slot("second", "Second.lean", second_prefix, second_suffix),
                ],
            )
            task = load_task_manifest(manifest, RUNTIME_ID)
            output = Path(directory) / "output"
            output.mkdir()
            raw = '{"first":"by\\n  trivial","second":"by\\n  trivial"}'

            result = materialize_candidate(task, raw, output)

            self.assertEqual(result.slot_ids, ("first", "second"))
            self.assertEqual((output / "First.lean").read_text(), "theorem task_first : True := by\n  trivial\n")
            self.assertEqual((output / "Second.lean").read_text(), "theorem task_second : True := by\n  trivial\n")
            other = Path(directory) / "other"
            other.mkdir()
            with self.assertRaisesRegex(ValidationError, "canonical JSON"):
                materialize_candidate(task, '{ "first":"by\\n  trivial", "second":"by\\n  trivial" }', other)

    def test_module_slice_composes_only_sealed_regions_and_declared_slots(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            empty = _write(root, "sealed/empty", b"")
            slots = [
                _slot("helper", "Module.lean", empty, empty),
                _slot("target", "Module.lean", empty, empty),
            ]
            before = _write(root, "sealed/before", b"theorem task_helper : True := ")
            middle = _write(root, "sealed/middle", b"\ntheorem task_target : True := ")
            after = _write(root, "sealed/after", b"\n")
            manifest = _manifest(
                root,
                strategy="module_slice",
                slots=slots,
                layout={
                    "kind": "module_slices",
                    "source_path": "Module.lean",
                    "pieces": [
                        {"artifact": before},
                        {"slot_id": "helper"},
                        {"artifact": middle},
                        {"slot_id": "target"},
                        {"artifact": after},
                    ],
                },
            )
            task = load_task_manifest(manifest, RUNTIME_ID)
            output = Path(directory) / "output"
            output.mkdir()

            materialize_candidate(task, '{"helper":"by trivial","target":"by trivial"}', output)

            self.assertEqual(
                (output / "Module.lean").read_text(),
                "theorem task_helper : True := by trivial\ntheorem task_target : True := by trivial\n",
            )

    def test_loader_verifies_runtime_and_all_sealed_sources(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            manifest = _manifest(root)
            with self.assertRaisesRegex(ValidationError, "runtime ID"):
                load_task_manifest(manifest, "other-runtime")

            task = load_task_manifest(manifest, RUNTIME_ID)
            (root / "sealed/Task.prefix.lean").write_text("tampered", encoding="utf-8")
            output = Path(directory) / "output"
            output.mkdir()
            with self.assertRaisesRegex(ValidationError, "byte size|digest"):
                materialize_candidate(task, "by trivial", output)

    def test_object_contract_rejects_unknown_missing_duplicate_and_fenced_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            first_prefix = _write(root, "sealed/First.prefix", b"theorem task_first : True := ")
            suffix = _write(root, "sealed/suffix", b"\n")
            task = load_task_manifest(
                _manifest(root, strategy="multi_slot", slots=[_slot("first", "Candidate.lean", first_prefix, suffix)]),
                RUNTIME_ID,
            )
            for index, (raw, message) in enumerate((
                ('{"unknown":"by trivial"}', "fields are not exact"),
                ('{}', "fields are not exact"),
                ('{"first":"by trivial","first":"by trivial"}', "duplicate key"),
                ('```lean\\nby trivial\\n```', "strict JSON"),
            )):
                output = Path(directory) / f"output-{index}"
                output.mkdir()
                with self.assertRaisesRegex(ValidationError, message):
                    materialize_candidate(task, raw, output)

            single_root = Path(directory) / "single"
            single_root.mkdir()
            single = load_task_manifest(_manifest(single_root), RUNTIME_ID)
            for index, raw in enumerate(("", "```lean\nby trivial\n```", "Here is the proof:\nby trivial", "by trivial\n\nby trivial", "by " + "x" * (64 * 1024))):
                output = Path(directory) / f"single-{index}"
                output.mkdir()
                with self.assertRaises(ValidationError):
                    materialize_candidate(single, raw, output)

    def test_guard_rejects_declaration_and_environment_escapes_with_leading_whitespace(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            task = load_task_manifest(_manifest(root), RUNTIME_ID)
            forbidden = (
                "import Foo",
                "open Foo",
                "namespace Foo",
                "section Foo",
                "end Foo",
                "private theorem injected : True := by trivial",
                "protected def injected := True",
                "noncomputable def injected := True",
                "unsafe def injected := True",
                "extern \"injected\" opaque injected : True",
                "theorem injected : True := by trivial",
                "def injected := True",
                "opaque injected : True",
                "axiom injected : True",
                "example : True := by trivial",
                "inductive Injected where",
                "structure Injected where",
                "class Injected where",
                "abbrev Injected := True",
                "instance : Inhabited True := ⟨True.intro⟩",
                "#eval 1",
                "run_tac Lean.Elab.Command.elabCommand",
                "set_option pp.universes true",
            )
            for index, token in enumerate(forbidden):
                output = Path(directory) / f"output-{index}"
                output.mkdir()
                with self.assertRaisesRegex(ValidationError, "forbidden"):
                    materialize_candidate(task, f"by\n  trivial\n  \n  {token}", output)
                output = Path(directory) / f"leading-{index}"
                output.mkdir()
                with self.assertRaisesRegex(ValidationError, "forbidden"):
                    materialize_candidate(task, f"  {token}", output)

    def test_guard_rejects_a_block_comment_prefixed_top_level_declaration(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            task = load_task_manifest(_manifest(root), RUNTIME_ID)
            output = Path(directory) / "output"
            output.mkdir()
            payload = "by\n  trivial\n/- -/ theorem injected : True := by trivial"

            with self.assertRaisesRegex(ValidationError, "comment"):
                materialize_candidate(task, payload, output)

    def test_guard_rejects_an_attribute_prefixed_top_level_declaration(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            task = load_task_manifest(_manifest(root), RUNTIME_ID)
            output = Path(directory) / "output"
            output.mkdir()
            payload = "by\n  intro h\n  exact h.1\n@[simp] theorem injected : True := by trivial"

            with self.assertRaisesRegex(ValidationError, "attribute"):
                materialize_candidate(task, payload, output)

    def test_guard_rejects_a_partial_definition_after_a_valid_proof(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            task = load_task_manifest(_manifest(root), RUNTIME_ID)
            output = Path(directory) / "output"
            output.mkdir()
            payload = "by\n  trivial\npartial def injected : Nat := 0"

            with self.assertRaisesRegex(ValidationError, "forbidden"):
                materialize_candidate(task, payload, output)

    def test_candidate_path_and_symlink_defenses(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "task"
            root.mkdir()
            manifest = _manifest(root)
            value = json.loads(manifest.read_text())
            value["seal"]["slots"][0]["source_path"] = "../escape.lean"
            manifest.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaisesRegex(ValidationError, "relative"):
                load_task_manifest(manifest, RUNTIME_ID)

            manifest = _manifest(root)
            task = load_task_manifest(manifest, RUNTIME_ID)
            output = Path(directory) / "output"
            output.mkdir()
            (output / "outside").write_text("not a candidate", encoding="utf-8")
            (output / "Candidate.lean").symlink_to(output / "outside")
            with self.assertRaisesRegex(ValidationError, "symlink"):
                materialize_candidate(task, "by trivial", output)

            link = output / "Candidate.lean"
            link.unlink()
            link.write_text("operator-owned", encoding="utf-8")
            with self.assertRaisesRegex(ValidationError, "already exists"):
                materialize_candidate(task, "by trivial", output)
            self.assertEqual(link.read_text(encoding="utf-8"), "operator-owned")
