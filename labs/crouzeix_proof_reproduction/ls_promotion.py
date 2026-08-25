from __future__ import annotations

import ctypes
import errno
import json
import os
import platform
import stat
import uuid
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Callable, Mapping, Sequence

import ls_contract
import ls_validation
import protocol


FORMAL_TARGET_RELATIVE = Path(
    "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger"
)
GRAPH_NAME = "source-graph.json"
INVENTORY_NAME = "library-inventory.json"
PROMOTION_NAME = "promotion.json"
LOCK_NAME = ".ls-promotion.lock"
TRANSACTION_PREFIX = ".ls-promotion-tx-"
LOCK_FIELDS = frozenset({"schema_version", "pid"})
TRANSACTION_FIELDS = frozenset(
    {
        "schema_version",
        "transaction_id",
        "graph_stage",
        "inventory_stage",
        "marker_stage",
        "original_graph_sha256",
        "original_inventory_sha256",
        "new_graph_sha256",
        "new_inventory_sha256",
    }
)
DARWIN_RENAME_EXCL = 0x00000004
LINUX_RENAME_NOREPLACE = 0x00000001
_HOOK: Callable[[str], None] | None = None


class PromotionCollisionError(protocol.ValidationError):
    """Another publisher currently owns the LS promotion boundary."""


class CommittedPromotionError(protocol.ValidationError):
    """The marker committed, but post-commit validation or durability failed."""

    def __init__(
        self, message: str, graph_sha256: str, inventory_sha256: str
    ) -> None:
        self.graph_sha256 = graph_sha256
        self.inventory_sha256 = inventory_sha256
        super().__init__(message)


@dataclass(frozen=True)
class _CandidateReceipt:
    node_id: str
    attempt_path: str
    receipt_sha256: str


@dataclass(frozen=True)
class _OwnedLock:
    descriptor: int
    device: int
    inode: int


@dataclass(frozen=True)
class _Transaction:
    pinned: ls_validation._PinnedDirectory
    record: dict[str, str]


def promote_six_node_route(repository_root, candidate_receipts) -> None:
    root = Path(os.path.abspath(os.fspath(repository_root)))
    candidates = _candidate_receipts(candidate_receipts)
    repository_chain = ls_validation._pin_absolute_chain(root, "LS repository root")
    formal_target_chain: tuple[ls_validation._PinnedDirectory, ...] = ()
    lock: _OwnedLock | None = None
    try:
        repository_directory = repository_chain[-1]
        formal_target_root = root / FORMAL_TARGET_RELATIVE
        formal_target_chain = ls_validation._pin_absolute_chain(
            formal_target_root, "LS formal target root"
        )
        target_directory = formal_target_chain[-1]
        lock = _acquire_lock(target_directory)
        _recover_pending_transaction(target_directory)
        current = ls_validation.load_route_state(formal_target_root)
        receipts = _validate_candidate_receipts(
            candidates, target_directory, repository_directory
        )
        graph_bytes = _canonical_graph_bytes(candidates, receipts)
        inventory_bytes = _canonical_inventory_bytes()
        if current["promotion"] is not None:
            _require_current_generation(current, graph_bytes, inventory_bytes)
            return
        _publish_generation(formal_target_root, target_directory, graph_bytes, inventory_bytes)
    finally:
        if lock is not None and formal_target_chain:
            _release_lock(formal_target_chain[-1], lock)
        if formal_target_chain:
            ls_validation._close_pinned_directories(formal_target_chain)
        ls_validation._close_pinned_directories(repository_chain)


def _candidate_receipts(value: Any) -> tuple[_CandidateReceipt, ...]:
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise protocol.ValidationError("candidate receipts must be a bounded sequence")
    materialized = tuple(value)
    if len(materialized) != len(ls_contract.NODES):
        raise protocol.ValidationError(
            "candidate receipts must contain exactly six entries"
        )
    result: list[_CandidateReceipt] = []
    seen_nodes: set[str] = set()
    seen_receipts: set[str] = set()
    for expected_node_id, item in zip(ls_contract.NODE_ORDER, materialized):
        if not isinstance(item, Mapping):
            raise protocol.ValidationError("candidate receipt must be an object")
        if set(item) != {"node_id", "attempt_path", "receipt_sha256"}:
            raise protocol.ValidationError("candidate receipt fields are invalid")
        node_id = ls_validation.formal_target._runtime_id(item["node_id"], "node_id")
        if node_id != expected_node_id:
            raise protocol.ValidationError(
                "candidate receipts must use canonical LS node order"
            )
        if node_id in seen_nodes:
            raise protocol.ValidationError("duplicate candidate node_id")
        receipt_sha256 = ls_validation.formal_target._digest(
            item["receipt_sha256"], "receipt_sha256"
        )
        if receipt_sha256 in seen_receipts:
            raise protocol.ValidationError("duplicate candidate receipt_sha256")
        attempt_path = _safe_attempt_path(item["attempt_path"], node_id)
        result.append(
            _CandidateReceipt(
                node_id=node_id,
                attempt_path=attempt_path,
                receipt_sha256=receipt_sha256,
            )
        )
        seen_nodes.add(node_id)
        seen_receipts.add(receipt_sha256)
    return tuple(result)


def _safe_attempt_path(value: Any, node_id: str) -> str:
    text = ls_validation._bounded_string(value, "attempt_path", 1, 4096)
    path = PurePosixPath(text)
    expected_prefix = FORMAL_TARGET_RELATIVE / "proof-slices" / node_id
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        raise protocol.ValidationError("candidate attempt_path is unsafe")
    if len(path.parts) != len(expected_prefix.parts) + 1:
        raise protocol.ValidationError("candidate attempt_path is invalid")
    if Path(*path.parts[:-1]) != expected_prefix:
        raise protocol.ValidationError(
            "candidate attempt_path is outside LS proof-slices"
        )
    if ls_validation.ATTEMPT_NAME.fullmatch(path.name) is None:
        raise protocol.ValidationError(
            "candidate attempt_path must name one attempt directory"
        )
    return path.as_posix()


def _validate_candidate_receipts(
    candidates: tuple[_CandidateReceipt, ...],
    target_directory: ls_validation._PinnedDirectory,
    repository_directory: ls_validation._PinnedDirectory,
) -> dict[str, dict[str, object]]:
    rows = tuple(
        ls_validation.LSGraphRow(
            node_id=contract.node_id,
            source_locator=contract.source_locator,
            statement_sha256=contract.statement_sha256,
            lean_name=contract.declaration,
            dependencies=contract.dependencies,
            role=contract.role,
            status="passed",
            receipt_sha256=candidate.receipt_sha256,
        )
        for candidate in candidates
        for contract in (ls_contract.BY_ID[candidate.node_id],)
    )
    proof_slices = ls_validation._pin_directory_at(
        target_directory,
        "proof-slices",
        target_directory.path / "proof-slices",
        "LS proof-slices root",
    )
    try:
        _require_candidate_attempt_paths(candidates, proof_slices)
        return ls_validation.validate_committed_receipts(
            rows, target_directory.path, repository_directory.path
        )
    finally:
        ls_validation._close_pinned_directories((proof_slices,))


def _require_candidate_attempt_paths(
    candidates: tuple[_CandidateReceipt, ...],
    proof_slices: ls_validation._PinnedDirectory,
) -> None:
    for candidate in candidates:
        relative = PurePosixPath(candidate.attempt_path)
        node_root = ls_validation._pin_directory_at(
            proof_slices,
            candidate.node_id,
            proof_slices.path / candidate.node_id,
            f"LS proof-slice directory for node {candidate.node_id}",
        )
        selected: ls_validation._PinnedDirectory | None = None
        try:
            selected = ls_validation._pin_directory_at(
                node_root,
                relative.name,
                node_root.path / relative.name,
                f"LS candidate attempt for node {candidate.node_id}",
            )
            receipt_data = ls_validation._read_file_at(
                selected,
                "receipt.json",
                f"LS candidate receipt for node {candidate.node_id}",
                ls_validation.MAX_JSON_BYTES,
            )
            if protocol.sha256_bytes(receipt_data) != candidate.receipt_sha256:
                raise protocol.ValidationError(
                    f"candidate receipt_sha256 mismatch: {candidate.node_id}"
                )
        finally:
            if selected is not None:
                ls_validation._close_pinned_directories((selected,))
            ls_validation._close_pinned_directories((node_root,))


def _canonical_graph_bytes(
    candidates: tuple[_CandidateReceipt, ...],
    receipts: Mapping[str, Mapping[str, object]],
) -> bytes:
    nodes = []
    for candidate in candidates:
        contract = ls_contract.BY_ID[candidate.node_id]
        receipt = receipts[candidate.node_id]
        if receipt["build_target"] != contract.build_target:
            raise protocol.ValidationError(
                f"candidate receipt build_target mismatch: {candidate.node_id}"
            )
        nodes.append(
            {
                "node_id": contract.node_id,
                "source_locator": contract.source_locator,
                "statement_sha256": contract.statement_sha256,
                "lean_name": contract.declaration,
                "dependencies": list(contract.dependencies),
                "role": contract.role,
                "status": "passed",
                "receipt_sha256": candidate.receipt_sha256,
            }
        )
    return _canonical_json_bytes(
        {
            "schema_version": ls_contract.SCHEMA_VERSION,
            "source_id": ls_contract.SOURCE_ID,
            "source_identity": ls_contract.SOURCE_IDENTITY,
            "nodes": nodes,
        }
    )


def _canonical_inventory_bytes() -> bytes:
    return _canonical_json_bytes(
        {
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
    )


def _require_current_generation(
    current: Mapping[str, object], graph_bytes: bytes, inventory_bytes: bytes
) -> None:
    expected_graph = protocol.sha256_bytes(graph_bytes)
    expected_inventory = protocol.sha256_bytes(inventory_bytes)
    if current["graph_sha256"] != expected_graph:
        raise protocol.ValidationError("promoted LS graph does not match candidate receipts")
    if current["inventory_sha256"] != expected_inventory:
        raise protocol.ValidationError(
            "promoted LS inventory does not match candidate receipts"
        )


def _publish_generation(
    formal_target_root: Path,
    target_directory: ls_validation._PinnedDirectory,
    graph_bytes: bytes,
    inventory_bytes: bytes,
) -> None:
    original_graph = ls_validation._read_file_at(
        target_directory, GRAPH_NAME, "LS source graph", ls_validation.MAX_JSON_BYTES
    )
    original_inventory = ls_validation._read_file_at(
        target_directory,
        INVENTORY_NAME,
        "LS library inventory",
        ls_validation.MAX_JSON_BYTES,
    )
    transaction = _create_transaction(
        target_directory, original_graph, original_inventory, graph_bytes, inventory_bytes
    )
    graph_replaced = False
    inventory_replaced = False
    marker_committed = False
    try:
        _checkpoint("before_graph_stage_write")
        _stage_file_from_transaction(target_directory, transaction, "new-graph.json", transaction.record["graph_stage"])
        _checkpoint("after_graph_stage_write")
        _checkpoint("before_graph_rename")
        os.replace(
            transaction.record["graph_stage"],
            GRAPH_NAME,
            src_dir_fd=target_directory.descriptor,
            dst_dir_fd=target_directory.descriptor,
        )
        graph_replaced = True
        _checkpoint("after_graph_rename")
        _fsync_descriptor(target_directory.descriptor)
        _checkpoint("after_graph_fsync")

        _checkpoint("before_inventory_stage_write")
        _stage_file_from_transaction(
            target_directory,
            transaction,
            "new-inventory.json",
            transaction.record["inventory_stage"],
        )
        _checkpoint("after_inventory_stage_write")
        _checkpoint("before_inventory_rename")
        os.replace(
            transaction.record["inventory_stage"],
            INVENTORY_NAME,
            src_dir_fd=target_directory.descriptor,
            dst_dir_fd=target_directory.descriptor,
        )
        inventory_replaced = True
        _checkpoint("after_inventory_rename")
        _fsync_descriptor(target_directory.descriptor)
        _checkpoint("after_inventory_fsync")

        _checkpoint("before_marker_stage_write")
        _stage_file_from_transaction(
            target_directory,
            transaction,
            "marker.json",
            transaction.record["marker_stage"],
        )
        _checkpoint("after_marker_stage_write")
        _checkpoint("before_marker_rename")
        _rename_file_no_replace_at(
            target_directory.descriptor, transaction.record["marker_stage"], PROMOTION_NAME
        )
        marker_committed = True
        _checkpoint("after_marker_rename")
        _fsync_descriptor(target_directory.descriptor)
        _checkpoint("after_marker_fsync")
        state = ls_validation.load_route_state(formal_target_root)
        _require_current_generation(state, graph_bytes, inventory_bytes)
        _checkpoint("after_commit_validation")
        _remove_transaction(transaction)
    except Exception as error:
        if marker_committed:
            raise CommittedPromotionError(
                "LS promotion committed new graph/inventory, but post-marker validation "
                f"or durability failed: {error}",
                protocol.sha256_bytes(graph_bytes),
                protocol.sha256_bytes(inventory_bytes),
            ) from error
        _rollback_pre_marker(
            target_directory, transaction, graph_replaced=graph_replaced, inventory_replaced=inventory_replaced
        )
        raise


def _create_transaction(
    target_directory: ls_validation._PinnedDirectory,
    original_graph: bytes,
    original_inventory: bytes,
    new_graph: bytes,
    new_inventory: bytes,
) -> _Transaction:
    transaction_id = uuid.uuid4().hex
    name = f"{TRANSACTION_PREFIX}{transaction_id}"
    os.mkdir(name, mode=0o700, dir_fd=target_directory.descriptor)
    pinned = ls_validation._pin_directory_at(
        target_directory,
        name,
        target_directory.path / name,
        "LS promotion transaction",
    )
    marker = {
        "schema_version": "crouzeix-ls-promotion/v1",
        "graph_sha256": protocol.sha256_bytes(new_graph),
        "inventory_sha256": protocol.sha256_bytes(new_inventory),
    }
    record = {
        "schema_version": "crouzeix-ls-promotion-transaction/v1",
        "transaction_id": transaction_id,
        "graph_stage": f".{transaction_id}.graph.stage",
        "inventory_stage": f".{transaction_id}.inventory.stage",
        "marker_stage": f".{transaction_id}.marker.stage",
        "original_graph_sha256": protocol.sha256_bytes(original_graph),
        "original_inventory_sha256": protocol.sha256_bytes(original_inventory),
        "new_graph_sha256": marker["graph_sha256"],
        "new_inventory_sha256": marker["inventory_sha256"],
    }
    _write_file_create_only_at(pinned.descriptor, "original-graph.json", original_graph)
    _write_file_create_only_at(
        pinned.descriptor, "original-inventory.json", original_inventory
    )
    _write_file_create_only_at(pinned.descriptor, "new-graph.json", new_graph)
    _write_file_create_only_at(pinned.descriptor, "new-inventory.json", new_inventory)
    _write_file_create_only_at(
        pinned.descriptor, "marker.json", _canonical_json_bytes(marker)
    )
    _write_file_create_only_at(
        pinned.descriptor, "journal.json", _canonical_json_bytes(record)
    )
    _fsync_descriptor(pinned.descriptor)
    _fsync_descriptor(target_directory.descriptor)
    return _Transaction(pinned=pinned, record=record)


def _stage_file_from_transaction(
    target_directory: ls_validation._PinnedDirectory,
    transaction: _Transaction,
    source_name: str,
    stage_name: str,
) -> None:
    data = ls_validation._read_file_at(
        transaction.pinned,
        source_name,
        f"LS promotion transaction member {source_name}",
        ls_validation.MAX_JSON_BYTES,
    )
    _write_file_create_only_at(target_directory.descriptor, stage_name, data)
    _fsync_descriptor(target_directory.descriptor)


def _rollback_pre_marker(
    target_directory: ls_validation._PinnedDirectory,
    transaction: _Transaction,
    *,
    graph_replaced: bool,
    inventory_replaced: bool,
) -> None:
    try:
        if inventory_replaced:
            _replace_file_from_transaction(
                target_directory, transaction, "original-inventory.json", INVENTORY_NAME
            )
        if graph_replaced:
            _replace_file_from_transaction(
                target_directory, transaction, "original-graph.json", GRAPH_NAME
            )
        for stage_name in (
            transaction.record["graph_stage"],
            transaction.record["inventory_stage"],
            transaction.record["marker_stage"],
        ):
            _cleanup_stage(target_directory.descriptor, stage_name)
        _remove_transaction(transaction)
        _fsync_descriptor(target_directory.descriptor)
    except Exception as rollback_error:
        raise protocol.ValidationError(
            f"LS pre-marker rollback failed: {rollback_error}"
        ) from rollback_error


def _replace_file_from_transaction(
    target_directory: ls_validation._PinnedDirectory,
    transaction: _Transaction,
    source_name: str,
    destination_name: str,
) -> None:
    stage_name = f".{transaction.record['transaction_id']}.{destination_name}.restore"
    _stage_file_from_transaction(target_directory, transaction, source_name, stage_name)
    os.replace(
        stage_name,
        destination_name,
        src_dir_fd=target_directory.descriptor,
        dst_dir_fd=target_directory.descriptor,
    )


def _recover_pending_transaction(target_directory: ls_validation._PinnedDirectory) -> None:
    names = _transaction_names(target_directory)
    if not names:
        return
    if len(names) != 1:
        raise protocol.ValidationError("multiple LS promotion transactions are present")
    pinned = ls_validation._pin_directory_at(
        target_directory,
        names[0],
        target_directory.path / names[0],
        "LS promotion transaction",
    )
    transaction = _load_transaction(pinned)
    try:
        marker = ls_validation._optional_read_file_at(
            target_directory, PROMOTION_NAME, "LS promotion marker", ls_validation.MAX_JSON_BYTES
        )
        if marker is not None:
            try:
                ls_validation.load_route_state(target_directory.path)
            except protocol.ValidationError as error:
                raise protocol.ValidationError(
                    f"LS promotion marker is invalid during recovery: {error}"
                ) from error
            _remove_transaction(transaction)
            return
        current_graph = ls_validation._read_file_at(
            target_directory, GRAPH_NAME, "LS source graph", ls_validation.MAX_JSON_BYTES
        )
        current_inventory = ls_validation._read_file_at(
            target_directory,
            INVENTORY_NAME,
            "LS library inventory",
            ls_validation.MAX_JSON_BYTES,
        )
        _require_known_generation(transaction, current_graph, current_inventory)
        _replace_file_from_transaction(
            target_directory, transaction, "original-graph.json", GRAPH_NAME
        )
        _replace_file_from_transaction(
            target_directory, transaction, "original-inventory.json", INVENTORY_NAME
        )
        for stage_name in (
            transaction.record["graph_stage"],
            transaction.record["inventory_stage"],
            transaction.record["marker_stage"],
        ):
            _cleanup_stage(target_directory.descriptor, stage_name)
        _remove_transaction(transaction)
        _fsync_descriptor(target_directory.descriptor)
    finally:
        if transaction.pinned.descriptor >= 0:
            ls_validation._close_pinned_directories((transaction.pinned,))


def _transaction_names(target_directory: ls_validation._PinnedDirectory) -> tuple[str, ...]:
    names = []
    for name in os.listdir(target_directory.descriptor):
        if not name.startswith(TRANSACTION_PREFIX):
            continue
        metadata = os.stat(name, dir_fd=target_directory.descriptor, follow_symlinks=False)
        if stat.S_ISLNK(metadata.st_mode):
            raise protocol.ValidationError("LS promotion transaction cannot be a symlink")
        if stat.S_ISDIR(metadata.st_mode):
            names.append(name)
    return tuple(sorted(names))


def _load_transaction(pinned: ls_validation._PinnedDirectory) -> _Transaction:
    record = ls_validation._json_object_from_bytes(
        ls_validation._read_file_at(
            pinned, "journal.json", "LS promotion journal", ls_validation.MAX_JSON_BYTES
        ),
        "LS promotion journal",
    )
    ls_validation._require_fields(record, TRANSACTION_FIELDS, "LS promotion journal")
    ls_validation._require_equal(
        record["schema_version"], "crouzeix-ls-promotion-transaction/v1", "schema_version"
    )
    normalized = {
        "schema_version": "crouzeix-ls-promotion-transaction/v1",
        "transaction_id": ls_validation._bounded_string(
            record["transaction_id"], "transaction_id", 1, 128
        ),
        "graph_stage": ls_validation._bounded_string(record["graph_stage"], "graph_stage", 1, 256),
        "inventory_stage": ls_validation._bounded_string(record["inventory_stage"], "inventory_stage", 1, 256),
        "marker_stage": ls_validation._bounded_string(record["marker_stage"], "marker_stage", 1, 256),
        "original_graph_sha256": ls_validation.formal_target._digest(
            record["original_graph_sha256"], "original_graph_sha256"
        ),
        "original_inventory_sha256": ls_validation.formal_target._digest(
            record["original_inventory_sha256"], "original_inventory_sha256"
        ),
        "new_graph_sha256": ls_validation.formal_target._digest(
            record["new_graph_sha256"], "new_graph_sha256"
        ),
        "new_inventory_sha256": ls_validation.formal_target._digest(
            record["new_inventory_sha256"], "new_inventory_sha256"
        ),
    }
    for name in (
        "original-graph.json",
        "original-inventory.json",
        "new-graph.json",
        "new-inventory.json",
        "marker.json",
    ):
        data = ls_validation._read_file_at(
            pinned, name, f"LS promotion transaction member {name}", ls_validation.MAX_JSON_BYTES
        )
        if name == "original-graph.json":
            expected = normalized["original_graph_sha256"]
        elif name == "original-inventory.json":
            expected = normalized["original_inventory_sha256"]
        elif name == "new-graph.json":
            expected = normalized["new_graph_sha256"]
        elif name == "new-inventory.json":
            expected = normalized["new_inventory_sha256"]
        else:
            marker = ls_validation._json_object_from_bytes(data, "LS promotion marker")
            expected = ls_validation.formal_target._digest(
                marker["graph_sha256"], "graph_sha256"
            )
            if expected != normalized["new_graph_sha256"]:
                raise protocol.ValidationError("LS promotion marker graph_sha256 mismatch")
            expected_inventory = ls_validation.formal_target._digest(
                marker["inventory_sha256"], "inventory_sha256"
            )
            if expected_inventory != normalized["new_inventory_sha256"]:
                raise protocol.ValidationError(
                    "LS promotion marker inventory_sha256 mismatch"
                )
            continue
        if protocol.sha256_bytes(data) != expected:
            raise protocol.ValidationError(
                f"LS promotion transaction digest mismatch: {name}"
            )
    return _Transaction(pinned=pinned, record=normalized)


def _require_known_generation(
    transaction: _Transaction, current_graph: bytes, current_inventory: bytes
) -> None:
    current_graph_sha256 = protocol.sha256_bytes(current_graph)
    current_inventory_sha256 = protocol.sha256_bytes(current_inventory)
    if current_graph_sha256 not in {
        transaction.record["original_graph_sha256"],
        transaction.record["new_graph_sha256"],
    }:
        raise protocol.ValidationError("LS source graph changed outside promotion recovery")
    if current_inventory_sha256 not in {
        transaction.record["original_inventory_sha256"],
        transaction.record["new_inventory_sha256"],
    }:
        raise protocol.ValidationError(
            "LS library inventory changed outside promotion recovery"
        )


def _remove_transaction(transaction: _Transaction) -> None:
    for name in (
        "journal.json",
        "marker.json",
        "new-inventory.json",
        "new-graph.json",
        "original-inventory.json",
        "original-graph.json",
    ):
        _cleanup_stage(transaction.pinned.descriptor, name)
    parent = transaction.pinned.parent
    if parent is None or transaction.pinned.entry_name is None:
        raise protocol.ValidationError("LS promotion transaction parent is missing")
    os.rmdir(transaction.pinned.entry_name, dir_fd=parent.descriptor)
    ls_validation._close_pinned_directories((transaction.pinned,))
    object.__setattr__(transaction.pinned, "descriptor", -1)


def _acquire_lock(target_directory: ls_validation._PinnedDirectory) -> _OwnedLock:
    while True:
        payload = _canonical_json_bytes(
            {"schema_version": "crouzeix-ls-promotion-lock/v1", "pid": os.getpid()}
        )
        flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
        if hasattr(os, "O_CLOEXEC"):
            flags |= os.O_CLOEXEC
        try:
            descriptor = os.open(LOCK_NAME, flags, 0o600, dir_fd=target_directory.descriptor)
        except FileExistsError:
            lock_data = ls_validation._read_file_at(
                target_directory, LOCK_NAME, "LS promotion lock", ls_validation.MAX_JSON_BYTES
            )
            lock = ls_validation._json_object_from_bytes(lock_data, "LS promotion lock")
            ls_validation._require_fields(lock, LOCK_FIELDS, "LS promotion lock")
            ls_validation._require_equal(
                lock["schema_version"], "crouzeix-ls-promotion-lock/v1", "schema_version"
            )
            pid = ls_validation._integer(lock["pid"], "pid", 1, 2**31 - 1)
            if _pid_alive(pid):
                raise PromotionCollisionError(
                    f"another LS promotion publisher is active: pid {pid}"
                )
            _cleanup_stage(target_directory.descriptor, LOCK_NAME)
            _fsync_descriptor(target_directory.descriptor)
            continue
        try:
            _write_all(descriptor, payload)
            _fsync_descriptor(descriptor)
            metadata = os.fstat(descriptor)
            _fsync_descriptor(target_directory.descriptor)
            return _OwnedLock(
                descriptor=descriptor, device=metadata.st_dev, inode=metadata.st_ino
            )
        except Exception:
            os.close(descriptor)
            raise


def _release_lock(
    target_directory: ls_validation._PinnedDirectory, lock: _OwnedLock
) -> None:
    try:
        metadata = os.stat(LOCK_NAME, dir_fd=target_directory.descriptor, follow_symlinks=False)
        if not stat.S_ISREG(metadata.st_mode) or (metadata.st_dev, metadata.st_ino) != (
            lock.device,
            lock.inode,
        ):
            raise protocol.ValidationError("LS promotion lock identity changed")
        os.unlink(LOCK_NAME, dir_fd=target_directory.descriptor)
        _fsync_descriptor(target_directory.descriptor)
    except FileNotFoundError:
        pass
    finally:
        os.close(lock.descriptor)


def _pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def _write_all(descriptor: int, data: bytes) -> None:
    written = 0
    while written < len(data):
        written += os.write(descriptor, data[written:])


def _write_file_create_only_at(parent_fd: int, name: str, data: bytes) -> None:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    descriptor = os.open(name, flags, 0o600, dir_fd=parent_fd)
    try:
        _write_all(descriptor, data)
        _fsync_descriptor(descriptor)
    finally:
        os.close(descriptor)


def _cleanup_stage(parent_fd: int, name: str) -> None:
    try:
        os.unlink(name, dir_fd=parent_fd)
    except FileNotFoundError:
        pass


def _fsync_descriptor(descriptor: int) -> None:
    os.fsync(descriptor)


def _canonical_json_bytes(value: object) -> bytes:
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


def _checkpoint(name: str) -> None:
    if _HOOK is not None:
        _HOOK(name)


def _rename_file_no_replace_at(
    parent_fd: int, source_name: str, destination_name: str
) -> None:
    system = platform.system()
    if system == "Darwin":
        _renameatx_np(parent_fd, source_name, parent_fd, destination_name)
        return
    if system == "Linux":
        _renameat2(parent_fd, source_name, parent_fd, destination_name)
        return
    if os.path.exists(destination_name):
        raise protocol.ValidationError("destination already exists")
    os.replace(
        source_name, destination_name, src_dir_fd=parent_fd, dst_dir_fd=parent_fd
    )


def _renameatx_np(
    source_fd: int, source_name: str, destination_fd: int, destination_name: str
) -> None:
    library = ctypes.CDLL(None, use_errno=True)
    rename = getattr(library, "renameatx_np", None)
    if rename is None:
        raise protocol.ValidationError("renameatx_np unavailable")
    rename.argtypes = [
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_uint,
    ]
    rename.restype = ctypes.c_int
    result = rename(
        source_fd,
        os.fsencode(source_name),
        destination_fd,
        os.fsencode(destination_name),
        DARWIN_RENAME_EXCL,
    )
    if result != 0:
        error = ctypes.get_errno()
        if error == errno.EEXIST:
            raise PromotionCollisionError("LS promotion marker already exists")
        raise OSError(error, os.strerror(error))


def _renameat2(
    source_fd: int, source_name: str, destination_fd: int, destination_name: str
) -> None:
    library = ctypes.CDLL(None, use_errno=True)
    rename = getattr(library, "renameat2", None)
    if rename is None:
        raise protocol.ValidationError("renameat2 unavailable")
    rename.argtypes = [
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_uint,
    ]
    rename.restype = ctypes.c_int
    result = rename(
        source_fd,
        os.fsencode(source_name),
        destination_fd,
        os.fsencode(destination_name),
        LINUX_RENAME_NOREPLACE,
    )
    if result != 0:
        error = ctypes.get_errno()
        if error == errno.EEXIST:
            raise PromotionCollisionError("LS promotion marker already exists")
        raise OSError(error, os.strerror(error))
