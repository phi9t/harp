"""Sealed Lean task manifests and proof-body-only candidate materialization.

The materializer is deliberately narrower than Lean syntax.  A candidate is a
non-empty, bounded tactic proof beginning with ``by``; it is inserted only into
fixed task-owned holes.  The lexical guard is defence in depth, not a Lean
parser: the fresh compiler and declaration audit remain responsible for the
semantic proof boundary.
"""

from __future__ import annotations

import re
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator

from model import (
    ArtifactRef,
    TaskSeal,
    ValidationError,
    canonical_json_bytes,
    parse_json_object,
    require_exact_fields,
    validate_relative_artifact_path,
)


_MANIFEST_SCHEMA = "harp-lean-task-manifest/v1"
_MAX_PROOF_BODY_BYTES = 64 * 1024
_STRATEGIES = frozenset({"single_slot", "multi_slot", "module_slice"})
_ATTRIBUTE_PREFIX = re.compile(r"@\s*\[")
_DECLARATION_WORDS = frozenset(
    {
        "import",
        "open",
        "namespace",
        "section",
        "end",
        "private",
        "protected",
        "noncomputable",
        "partial",
        "unsafe",
        "extern",
        "builtin",
        "abstract",
        "theorem",
        "def",
        "opaque",
        "axiom",
        "example",
        "inductive",
        "structure",
        "class",
        "abbrev",
        "instance",
        "run_tac",
        "elab",
        "macro",
        "syntax",
        "scoped",
        "attribute",
        "set_option",
        "initialize",
        "builtin_initialize",
        "unsafe",
        "mutual",
        "universe",
        "variable",
        "include",
        "omit",
        "export",
        "local",
        "macro_rules",
        "declare_syntax_cat",
        "infix",
        "infixl",
        "infixr",
        "prefix",
        "postfix",
        "notation",
        "reserved",
        "simproc",
        "builtin_simproc",
    }
)


@dataclass(frozen=True)
class ModulePiece:
    """One task-owned byte region or one declared proof-body hole."""

    artifact: ArtifactRef | None
    slot_id: str | None


@dataclass(frozen=True)
class ModuleLayout:
    source_path: str
    pieces: tuple[ModulePiece, ...]


@dataclass(frozen=True)
class SealedTask:
    """An immutable manifest whose task-owned assets have been byte-verified."""

    root: Path
    seal: TaskSeal
    strategy: str
    module_layout: ModuleLayout | None


@dataclass(frozen=True)
class MaterializedCandidate:
    """The only candidate source files written beneath an output directory."""

    slot_ids: tuple[str, ...]
    sources: tuple[ArtifactRef, ...]


def load_task_manifest(path: Path, runtime_id: str) -> SealedTask:
    """Load a closed manifest and verify every task-owned artifact before use."""
    _require_regular_file(path, "task manifest")
    root = path.parent
    _require_real_directory(root, "task root")
    try:
        raw = parse_json_object(path.read_bytes(), "task manifest")
    except OSError as error:
        raise ValidationError(f"cannot read task manifest: {error}") from error
    require_exact_fields(raw, ("schema_version", "seal", "strategy", "layout"), "task manifest")
    if raw["schema_version"] != _MANIFEST_SCHEMA:
        raise ValidationError("task manifest schema version is not recognized")
    if not isinstance(raw["seal"], dict):
        raise ValidationError("task manifest seal must be an object")
    seal = TaskSeal.from_value(raw["seal"], "task manifest seal")
    if not isinstance(runtime_id, str) or not runtime_id:
        raise ValidationError("expected runtime ID must be non-empty text")
    if seal.runtime_id != runtime_id:
        raise ValidationError("task manifest runtime ID does not match the resolved runtime")
    strategy = raw["strategy"]
    if not isinstance(strategy, str) or strategy not in _STRATEGIES:
        raise ValidationError("task manifest strategy is not recognized")
    layout = _parse_layout(raw["layout"], strategy, seal)
    task = SealedTask(root=root, seal=seal, strategy=strategy, module_layout=layout)
    _verify_task_assets(task)
    return task


def materialize_candidate(task: SealedTask, model_output: str, output_root: Path) -> MaterializedCandidate:
    """Create candidates from fixed source assets and declared proof-body slots only.

    ``single_slot`` consumes exactly one raw tactic proof body.  ``multi_slot``
    and ``module_slice`` consume exact canonical JSON with one string per
    declared slot.  Neither interface accepts a free-form Lean module.
    """
    if not isinstance(task, SealedTask):
        raise TypeError("task must be a SealedTask")
    _require_real_directory(output_root, "candidate output root")
    _verify_task_assets(task)
    bodies = _parse_model_output(task, model_output)
    if task.strategy == "module_slice":
        assert task.module_layout is not None
        source_bytes = bytearray()
        for piece in task.module_layout.pieces:
            if piece.artifact is not None:
                source_bytes.extend(piece.artifact.verify(task.root).read_bytes())
            else:
                assert piece.slot_id is not None
                source_bytes.extend(bodies[piece.slot_id].encode("utf-8", "strict"))
        planned = ((task.module_layout.source_path, bytes(source_bytes)),)
    else:
        planned = tuple(
            (
                slot.source_path,
                slot.prefix.verify(task.root).read_bytes()
                + bodies[slot.slot_id].encode("utf-8", "strict")
                + slot.suffix.verify(task.root).read_bytes(),
            )
            for slot in task.seal.slots
        )
    return _write_candidates(output_root, task.seal, planned)


def _parse_layout(value: object, strategy: str, seal: TaskSeal) -> ModuleLayout | None:
    if not isinstance(value, dict):
        raise ValidationError("task manifest layout must be an object")
    if strategy in {"single_slot", "multi_slot"}:
        require_exact_fields(value, ("kind",), "task manifest layout")
        if value["kind"] != "slots":
            raise ValidationError("task manifest slot layout kind is not recognized")
        if strategy == "single_slot" and len(seal.slots) != 1:
            raise ValidationError("single_slot tasks require exactly one slot")
        if strategy == "multi_slot" and len(seal.slots) < 1:
            raise ValidationError("multi_slot tasks require one or more slots")
        if len({slot.source_path for slot in seal.slots}) != len(seal.slots):
            raise ValidationError("slot source paths must be unique outside module_slice tasks")
        return None

    require_exact_fields(value, ("kind", "source_path", "pieces"), "task manifest module layout")
    if value["kind"] != "module_slices":
        raise ValidationError("task manifest module layout kind is not recognized")
    source_path = validate_relative_artifact_path(value["source_path"], "task manifest module source_path")
    if not isinstance(value["pieces"], list) or not value["pieces"]:
        raise ValidationError("task manifest module pieces must be a non-empty array")
    pieces: list[ModulePiece] = []
    declared_slots = tuple(slot.slot_id for slot in seal.slots)
    observed_slots: list[str] = []
    for index, raw_piece in enumerate(value["pieces"]):
        if not isinstance(raw_piece, dict):
            raise ValidationError(f"task manifest module pieces[{index}] must be an object")
        if set(raw_piece) == {"artifact"}:
            pieces.append(ModulePiece(ArtifactRef.from_value(raw_piece["artifact"], f"task manifest module pieces[{index}] artifact"), None))
        elif set(raw_piece) == {"slot_id"}:
            slot_id = raw_piece["slot_id"]
            if not isinstance(slot_id, str) or slot_id not in declared_slots:
                raise ValidationError(f"task manifest module pieces[{index}] names an undeclared slot")
            pieces.append(ModulePiece(None, slot_id))
            observed_slots.append(slot_id)
        else:
            raise ValidationError(f"task manifest module pieces[{index}] must name exactly one artifact or slot_id")
    if tuple(observed_slots) != declared_slots:
        raise ValidationError("task manifest module slot pieces must appear once in declared fixed order")
    if any(slot.source_path != source_path for slot in seal.slots):
        raise ValidationError("module_slice slots must share the declared module source_path")
    return ModuleLayout(source_path=source_path, pieces=tuple(pieces))


def _parse_model_output(task: SealedTask, raw: str) -> dict[str, str]:
    if not isinstance(raw, str):
        raise ValidationError("model output must be text")
    slot_ids = tuple(slot.slot_id for slot in task.seal.slots)
    if task.strategy == "single_slot":
        _validate_proof_body(raw, task.seal.slots[0].forbidden_tokens)
        return {slot_ids[0]: raw}
    parsed = parse_json_object(raw, "multi-slot model output")
    if raw.encode("utf-8", "strict") != canonical_json_bytes(parsed):
        raise ValidationError("multi-slot model output must be canonical JSON")
    require_exact_fields(parsed, slot_ids, "multi-slot model output")
    bodies: dict[str, str] = {}
    for slot in task.seal.slots:
        body = parsed[slot.slot_id]
        if not isinstance(body, str):
            raise ValidationError(f"multi-slot model output {slot.slot_id!r} must be text")
        _validate_proof_body(body, slot.forbidden_tokens)
        bodies[slot.slot_id] = body
    return bodies


def _validate_proof_body(body: str, forbidden_tokens: tuple[str, ...]) -> None:
    """Apply a conservative lexical guard to a tactic proof body, not a parser.

    The accepted surface starts with ``by`` and never contains a line whose
    first token is a Lean command, declaration, or declaration modifier
    (including ``partial``, ``unsafe``, and ``extern``). This intentionally
    rejects some legal proof terms; the boundary is designed for a sealed
    benchmark, not as a complete Lean grammar.
    """
    try:
        encoded = body.encode("utf-8", "strict")
    except UnicodeEncodeError as error:
        raise ValidationError("proof body must be UTF-8 text") from error
    stripped = body.strip()
    if not stripped:
        raise ValidationError("proof body must not be blank")
    if len(encoded) > _MAX_PROOF_BODY_BYTES:
        raise ValidationError("proof body exceeds the sealed size limit")
    if "\x00" in body or "```" in body:
        raise ValidationError("proof body must not contain fenced or binary output")
    # Parsing nested Lean comments correctly would require a full lexical state
    # machine with string and character literal handling.  This boundary has no
    # need to accept comments, so reject both delimiter spellings everywhere,
    # including inside would-be strings.  That conservative choice prevents a
    # comment prefix from hiding a fresh top-level command from the line guard.
    if "/-" in body or "--" in body:
        raise ValidationError("proof body must not contain Lean comment syntax")
    # Attributes are another non-keyword command prefix (``@[simp] theorem``).
    # Reject the complete spelling, including whitespace variants, everywhere
    # rather than attempting to distinguish an attribute from quoted syntax.
    # Together with comment/fence rejection, ``#`` command rejection, and the
    # keyword guard below, this covers the leading lexical escape forms this
    # proof-body contract deliberately does not parse.
    if _ATTRIBUTE_PREFIX.search(body) is not None:
        raise ValidationError("proof body must not contain Lean attribute syntax")
    for line in body.splitlines():
        token = line.lstrip()
        if not token:
            continue
        if token.startswith("#"):
            raise ValidationError("proof body contains a forbidden Lean command")
        first = token.split(None, 1)[0].rstrip("(")
        if first in _DECLARATION_WORDS:
            raise ValidationError("proof body contains a forbidden Lean command or declaration")
    for token in forbidden_tokens:
        if token in body:
            raise ValidationError("proof body contains a forbidden task token")
    if not (stripped == "by" or stripped.startswith("by ") or stripped.startswith("by\n")):
        raise ValidationError("proof body must be one raw tactic proof beginning with by")
    by_lines = [
        line
        for line in body.splitlines()
        if line.lstrip() == "by" or line.lstrip().startswith("by ")
    ]
    if len(by_lines) != 1:
        raise ValidationError("proof body must contain exactly one outer by block")


def _verify_task_assets(task: SealedTask) -> None:
    _require_real_directory(task.root, "task root")
    for artifact in _task_artifacts(task):
        artifact.verify(task.root)


def _task_artifacts(task: SealedTask) -> Iterator[ArtifactRef]:
    yield task.seal.prompt
    yield from task.seal.project_inventory
    for slot in task.seal.slots:
        yield slot.prefix
        yield slot.suffix
    if task.module_layout is not None:
        for piece in task.module_layout.pieces:
            if piece.artifact is not None:
                yield piece.artifact


def _write_candidates(output_root: Path, seal: TaskSeal, planned: tuple[tuple[str, bytes], ...]) -> MaterializedCandidate:
    paths = tuple(path for path, _ in planned)
    if len(paths) != len(set(paths)):
        raise ValidationError("candidate output paths must be unique")
    for path, _ in planned:
        _candidate_target(output_root, path)
    refs: list[ArtifactRef] = []
    created: list[Path] = []
    try:
        for path, data in planned:
            target = _candidate_target(output_root, path)
            with target.open("xb") as handle:
                # Record only files this invocation has actually created; a
                # competing file must never be removed during cleanup.
                created.append(target)
                handle.write(data)
            _require_regular_file(target, "candidate source")
            refs.append(ArtifactRef.from_file(output_root, path))
    except Exception:
        # A partial candidate is never a valid result.  Keep cleanup tightly
        # scoped to newly-created targets and never follow symlinks.
        for target in created:
            try:
                if target.exists() and not target.is_symlink() and stat.S_ISREG(target.lstat().st_mode):
                    target.unlink()
            except OSError:
                pass
        raise
    return MaterializedCandidate(tuple(slot.slot_id for slot in seal.slots), tuple(refs))


def _candidate_target(root: Path, relative_path: str) -> Path:
    relative_path = validate_relative_artifact_path(relative_path, "candidate source path")
    _require_real_directory(root, "candidate output root")
    target = root
    parts = relative_path.split("/")
    for index, part in enumerate(parts):
        target = target / part
        if index == len(parts) - 1:
            if target.exists() or target.is_symlink():
                if target.is_symlink():
                    raise ValidationError("candidate source path must not be a symlink")
                raise ValidationError("candidate source already exists")
            continue
        try:
            metadata = target.lstat()
        except OSError as error:
            raise ValidationError("candidate source parent must be a pre-created real directory") from error
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISDIR(metadata.st_mode):
            raise ValidationError("candidate source path must not traverse a symlink or non-directory")
    return target


def _require_real_directory(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISDIR(metadata.st_mode):
        raise ValidationError(f"{label} must be a real non-symlink directory")


def _require_regular_file(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        raise ValidationError(f"{label} must be a regular non-symlink file")
