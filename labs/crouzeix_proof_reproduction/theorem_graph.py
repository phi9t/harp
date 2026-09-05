"""Build a route-independent Crouzeix theorem graph from route manifests."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping

try:
    from . import route_validation
except ImportError:  # pragma: no cover - direct script import path
    import route_validation


GRAPH_SCHEMA_VERSION = "crouzeix-theorem-graph/v1"
GRAPH_ROUTE_IDS = ("jin", "lorist-schwenninger", "harp")
ROUTE_ORDER = {route_id: index for index, route_id in enumerate(GRAPH_ROUTE_IDS)}
PROOF_STATUSES = ("blocked", "deprecated", "open", "passed-external", "passed-local")
READBACK_STATUSES = ("current", "flagged", "missing", "stale")
CLAIM_CEILINGS = (
    "external-candidate",
    "harp-derived-local-certification",
    "reused-route-local-certification",
    "source-faithful-local-certification",
)


class TheoremGraphError(ValueError):
    """The theorem graph cannot be built from the available route artifacts."""


@dataclass(frozen=True)
class GraphRoute:
    route_id: str
    claim_kind: str
    manifest_path: str
    manifest_sha256: str
    terminal_declaration: str
    terminal_type_sha256: str
    receipt_path: str
    receipt_sha256: str | None
    review_path: str
    review_sha256: str | None
    allowed_axioms: tuple[str, ...]

    def to_json(self) -> dict[str, object]:
        return {
            "route_id": self.route_id,
            "claim_kind": self.claim_kind,
            "manifest_path": self.manifest_path,
            "manifest_sha256": self.manifest_sha256,
            "terminal_declaration": self.terminal_declaration,
            "terminal_type_sha256": self.terminal_type_sha256,
            "receipt_path": self.receipt_path,
            "receipt_sha256": self.receipt_sha256,
            "review_path": self.review_path,
            "review_sha256": self.review_sha256,
            "allowed_axioms": list(self.allowed_axioms),
        }


@dataclass(frozen=True)
class GraphNode:
    node_id: str
    route_ids: tuple[str, ...]
    route_node_ids: tuple[str, ...]
    roles: tuple[str, ...]
    lean_name: str
    statement_sha256: str
    statement_text_paths: tuple[str, ...]
    natural_language_statement: str | None
    source_locators: tuple[str, ...]
    dependency_ids: tuple[str, ...]
    proof_status: str
    proof_receipt_paths: tuple[str, ...]
    allowed_axioms: tuple[str, ...]
    readback_status: str
    claim_ceiling: str

    def to_json(self) -> dict[str, object]:
        return {
            "node_id": self.node_id,
            "route_ids": list(self.route_ids),
            "route_node_ids": list(self.route_node_ids),
            "roles": list(self.roles),
            "lean_name": self.lean_name,
            "statement_sha256": self.statement_sha256,
            "statement_text_paths": list(self.statement_text_paths),
            "natural_language_statement": self.natural_language_statement,
            "source_locators": list(self.source_locators),
            "dependency_ids": list(self.dependency_ids),
            "proof_status": self.proof_status,
            "proof_receipt_paths": list(self.proof_receipt_paths),
            "allowed_axioms": list(self.allowed_axioms),
            "readback_status": self.readback_status,
            "claim_ceiling": self.claim_ceiling,
        }


@dataclass(frozen=True)
class TheoremGraph:
    schema_version: str
    routes: tuple[GraphRoute, ...]
    nodes: tuple[GraphNode, ...]
    frontier: dict[str, tuple[str, ...]]

    def to_json(self) -> dict[str, object]:
        return {
            "schema_version": self.schema_version,
            "routes": [route.to_json() for route in self.routes],
            "nodes": [node.to_json() for node in self.nodes],
            "frontier": {
                key: list(value)
                for key, value in sorted(self.frontier.items())
            },
        }


@dataclass(frozen=True)
class _NodePart:
    route_id: str
    route_node_id: str
    role: str
    declaration: str
    statement_sha256: str
    declaration_type_path: str
    source_locator: str | None
    dependency_route_node_ids: tuple[str, ...]
    proof_status: str
    proof_receipt_path: str | None
    allowed_axioms: tuple[str, ...]
    provenance_kind: str

    @property
    def graph_key(self) -> tuple[str, str]:
        return (self.declaration, self.statement_sha256)


def build_theorem_graph(repository_root: Path) -> TheoremGraph:
    root = Path(repository_root)
    routes: list[GraphRoute] = []
    parts: list[_NodePart] = []
    key_by_route_node: dict[tuple[str, str], tuple[str, str]] = {}

    for route_id in GRAPH_ROUTE_IDS:
        manifest_path = route_validation.ROUTE_MANIFEST_PATHS[route_id]
        manifest = route_validation.load_route_manifest(root, manifest_path)
        if manifest.route_id != route_id:
            raise TheoremGraphError(f"route manifest path mismatch: {route_id}")
        manifest_bytes = _read_manifest_bytes(root, manifest_path)
        manifest_sha256 = _sha256(manifest_bytes)
        routes.append(
            GraphRoute(
                route_id=manifest.route_id,
                claim_kind=manifest.claim_kind,
                manifest_path=manifest_path.as_posix(),
                manifest_sha256=manifest_sha256,
                terminal_declaration=manifest.terminal_declaration,
                terminal_type_sha256=manifest.terminal_type_sha256,
                receipt_path=manifest.receipt_path,
                receipt_sha256=manifest.receipt_sha256,
                review_path=manifest.review_path,
                review_sha256=manifest.review_sha256,
                allowed_axioms=manifest.allowed_axioms,
            )
        )
        for node in manifest.nodes:
            proof_status = _proof_status(manifest)
            part = _NodePart(
                route_id=manifest.route_id,
                route_node_id=node.node_id,
                role=node.role,
                declaration=node.declaration,
                statement_sha256=node.statement_sha256,
                declaration_type_path=node.declaration_type_path,
                source_locator=node.source_locator,
                dependency_route_node_ids=node.dependency_ids,
                proof_status=proof_status,
                proof_receipt_path=manifest.receipt_path if proof_status == "passed-local" else None,
                allowed_axioms=manifest.allowed_axioms if proof_status == "passed-local" else (),
                provenance_kind=node.provenance_kind,
            )
            key = part.graph_key
            existing_key = key_by_route_node.setdefault((part.route_id, part.route_node_id), key)
            if existing_key != key:
                raise TheoremGraphError(
                    f"route node changed graph identity: {part.route_id}:{part.route_node_id}"
                )
            parts.append(part)

    graph_id_by_key = _assign_graph_ids(parts)
    dependency_keys = _dependency_keys(parts, key_by_route_node)
    nodes = tuple(
        _collapse_node(key, grouped_parts, dependency_keys[key], graph_id_by_key)
        for key, grouped_parts in _group_parts(parts).items()
    )
    nodes = _topological_order(nodes)
    _validate_graph(nodes)
    return TheoremGraph(
        schema_version=GRAPH_SCHEMA_VERSION,
        routes=tuple(routes),
        nodes=nodes,
        frontier=_frontier(nodes),
    )


def validate_theorem_graph(value: Mapping[str, object]) -> TheoremGraph:
    _exact_fields(value, {"schema_version", "routes", "nodes", "frontier"}, "theorem graph")
    if value["schema_version"] != GRAPH_SCHEMA_VERSION:
        raise TheoremGraphError("invalid theorem graph schema version")
    raw_routes = value["routes"]
    if not isinstance(raw_routes, list) or len(raw_routes) != len(GRAPH_ROUTE_IDS):
        raise TheoremGraphError("routes must contain all route summaries")
    routes = tuple(_parse_route(route) for route in raw_routes)
    if tuple(route.route_id for route in routes) != GRAPH_ROUTE_IDS:
        raise TheoremGraphError("routes are not in canonical order")
    raw_nodes = value["nodes"]
    if not isinstance(raw_nodes, list) or not raw_nodes:
        raise TheoremGraphError("nodes must be a nonempty array")
    nodes = tuple(_parse_node(node) for node in raw_nodes)
    _validate_graph(nodes)
    observed = _frontier(nodes)
    raw_frontier = value["frontier"]
    if not isinstance(raw_frontier, dict):
        raise TheoremGraphError("frontier must be an object")
    _exact_fields(raw_frontier, set(observed), "frontier")
    normalized_frontier = {
        str(key): _string_tuple(items, "frontier node id", allow_empty=True)
        for key, items in raw_frontier.items()
    }
    if normalized_frontier != observed:
        raise TheoremGraphError("frontier does not match graph nodes")
    return TheoremGraph(
        schema_version=GRAPH_SCHEMA_VERSION,
        routes=routes,
        nodes=nodes,
        frontier=observed,
    )


def build_theorem_graph_json(repository_root: Path) -> dict[str, object]:
    return build_theorem_graph(repository_root).to_json()


def _proof_status(manifest: route_validation.RouteManifest) -> str:
    if manifest.receipt_sha256 is None:
        return "open"
    return "passed-local"


def _assign_graph_ids(parts: Iterable[_NodePart]) -> dict[tuple[str, str], str]:
    grouped = _group_parts(parts)
    used: set[str] = set()
    ids: dict[tuple[str, str], str] = {}
    for key, group in grouped.items():
        candidate = _preferred_node_id(group)
        if candidate in used:
            candidate = _slug(key[0])
        if candidate in used:
            candidate = f"{candidate}-{key[1][:12]}"
        if candidate in used:
            raise TheoremGraphError(f"cannot assign unique graph node id for {key[0]}")
        used.add(candidate)
        ids[key] = candidate
    return ids


def _dependency_keys(
    parts: Iterable[_NodePart],
    key_by_route_node: Mapping[tuple[str, str], tuple[str, str]],
) -> dict[tuple[str, str], set[tuple[str, str]]]:
    dependencies: dict[tuple[str, str], set[tuple[str, str]]] = {}
    for part in parts:
        deps = dependencies.setdefault(part.graph_key, set())
        for dependency in part.dependency_route_node_ids:
            key = key_by_route_node.get((part.route_id, dependency))
            if key is None:
                raise TheoremGraphError(
                    f"unknown dependency {dependency!r} in route {part.route_id}"
                )
            deps.add(key)
    return dependencies


def _collapse_node(
    key: tuple[str, str],
    parts: tuple[_NodePart, ...],
    dependency_keys: set[tuple[str, str]],
    graph_id_by_key: Mapping[tuple[str, str], str],
) -> GraphNode:
    declaration, statement_sha256 = key
    route_ids = _unique_sorted(part.route_id for part in parts)
    route_node_ids = tuple(
        f"{part.route_id}:{part.route_node_id}"
        for part in sorted(parts, key=lambda item: (item.route_id, item.route_node_id))
    )
    roles = _unique_sorted(part.role for part in parts)
    statement_paths = _unique_sorted(part.declaration_type_path for part in parts)
    source_locators = _unique_sorted(
        part.source_locator for part in parts if part.source_locator is not None
    )
    proof_receipts = _unique_sorted(
        part.proof_receipt_path for part in parts if part.proof_receipt_path is not None
    )
    allowed_axioms = _unique_sorted(
        axiom for part in parts for axiom in part.allowed_axioms
    )
    statuses = {part.proof_status for part in parts}
    proof_status = _combined_proof_status(statuses)
    provenance = {part.provenance_kind for part in parts}
    dependency_ids = _unique_sorted(graph_id_by_key[dependency] for dependency in dependency_keys)
    return GraphNode(
        node_id=graph_id_by_key[key],
        route_ids=route_ids,
        route_node_ids=route_node_ids,
        roles=roles,
        lean_name=declaration,
        statement_sha256=statement_sha256,
        statement_text_paths=statement_paths,
        natural_language_statement=None,
        source_locators=source_locators,
        dependency_ids=dependency_ids,
        proof_status=proof_status,
        proof_receipt_paths=proof_receipts,
        allowed_axioms=allowed_axioms,
        readback_status="missing",
        claim_ceiling=_claim_ceiling(route_ids, provenance),
    )


def _combined_proof_status(statuses: set[str]) -> str:
    for status in ("passed-local", "passed-external", "blocked", "open", "deprecated"):
        if status in statuses:
            return status
    raise TheoremGraphError("empty proof status set")


def _claim_ceiling(route_ids: tuple[str, ...], provenance: set[str]) -> str:
    if provenance & {"source"}:
        return "source-faithful-local-certification"
    if provenance == {"reused-route"}:
        return "reused-route-local-certification"
    if "harp" in route_ids:
        return "harp-derived-local-certification"
    return "external-candidate"


def _frontier(nodes: tuple[GraphNode, ...]) -> dict[str, tuple[str, ...]]:
    missing_readbacks = [
        node.node_id
        for node in nodes
        if node.readback_status != "current"
        and (
            {"terminal", "consequence"} & set(node.roles)
            or node.source_locators
            or node.claim_ceiling == "reused-route-local-certification"
        )
    ]
    external_candidates = [
        node.node_id
        for node in nodes
        if node.proof_status == "passed-local"
        and node.claim_ceiling
        in {
            "source-faithful-local-certification",
            "harp-derived-local-certification",
        }
        and "consequence" not in node.roles
    ]
    return {
        "blocked_nodes": tuple(node.node_id for node in nodes if node.proof_status == "blocked"),
        "external_candidates": tuple(external_candidates),
        "missing_readbacks": tuple(missing_readbacks),
        "open_nodes": tuple(node.node_id for node in nodes if node.proof_status == "open"),
    }


def _validate_graph(nodes: tuple[GraphNode, ...]) -> None:
    ids = [node.node_id for node in nodes]
    if len(ids) != len(set(ids)):
        raise TheoremGraphError("duplicate graph node id")
    known = set(ids)
    for node in nodes:
        _validate_node(node)
        for dependency in node.dependency_ids:
            if dependency not in known:
                raise TheoremGraphError(f"dangling dependency: {dependency}")
            if dependency == node.node_id:
                raise TheoremGraphError(f"self dependency: {node.node_id}")
    _assert_acyclic(nodes)


def _validate_node(node: GraphNode) -> None:
    if node.proof_status not in PROOF_STATUSES:
        raise TheoremGraphError(f"invalid proof status: {node.proof_status}")
    if node.readback_status not in READBACK_STATUSES:
        raise TheoremGraphError(f"invalid readback status: {node.readback_status}")
    if node.claim_ceiling not in CLAIM_CEILINGS:
        raise TheoremGraphError(f"invalid claim ceiling: {node.claim_ceiling}")
    if node.proof_status in {"passed-local", "passed-external"} and not node.proof_receipt_paths:
        raise TheoremGraphError(f"proof-bearing node lacks receipt: {node.node_id}")
    if node.proof_status == "passed-local" and not node.allowed_axioms:
        raise TheoremGraphError(f"local proof-bearing node lacks axiom policy: {node.node_id}")
    if node.claim_ceiling == "source-faithful-local-certification" and not node.source_locators:
        raise TheoremGraphError(f"source-backed node lacks source locator: {node.node_id}")
    if node.claim_ceiling == "reused-route-local-certification" and not node.proof_receipt_paths:
        raise TheoremGraphError(f"reused-route node lacks upstream receipt: {node.node_id}")
    if route_validation.SHA256_RE.fullmatch(node.statement_sha256) is None:
        raise TheoremGraphError(f"invalid statement hash: {node.node_id}")


def _assert_acyclic(nodes: tuple[GraphNode, ...]) -> None:
    by_id = {node.node_id: node for node in nodes}
    temporary: set[str] = set()
    permanent: set[str] = set()

    def visit(node_id: str) -> None:
        if node_id in permanent:
            return
        if node_id in temporary:
            raise TheoremGraphError(f"cycle detected at {node_id}")
        temporary.add(node_id)
        for dependency in by_id[node_id].dependency_ids:
            visit(dependency)
        temporary.remove(node_id)
        permanent.add(node_id)

    for node in nodes:
        visit(node.node_id)


def _topological_order(nodes: tuple[GraphNode, ...]) -> tuple[GraphNode, ...]:
    by_id = {node.node_id: node for node in nodes}
    ordered: list[GraphNode] = []
    temporary: set[str] = set()
    permanent: set[str] = set()

    def visit(node_id: str) -> None:
        if node_id in permanent:
            return
        if node_id in temporary:
            raise TheoremGraphError(f"cycle detected at {node_id}")
        temporary.add(node_id)
        for dependency in sorted(by_id[node_id].dependency_ids):
            visit(dependency)
        temporary.remove(node_id)
        permanent.add(node_id)
        ordered.append(by_id[node_id])

    for node_id in sorted(by_id):
        visit(node_id)
    return tuple(ordered)


def _group_parts(parts: Iterable[_NodePart]) -> dict[tuple[str, str], tuple[_NodePart, ...]]:
    grouped: dict[tuple[str, str], list[_NodePart]] = {}
    for part in parts:
        grouped.setdefault(part.graph_key, []).append(part)
    return {key: tuple(value) for key, value in grouped.items()}


def _parse_route(value: object) -> GraphRoute:
    if not isinstance(value, dict):
        raise TheoremGraphError("route summary must be an object")
    _exact_fields(
        value,
        {
            "route_id",
            "claim_kind",
            "manifest_path",
            "manifest_sha256",
            "terminal_declaration",
            "terminal_type_sha256",
            "receipt_path",
            "receipt_sha256",
            "review_path",
            "review_sha256",
            "allowed_axioms",
        },
        "route summary",
    )
    return GraphRoute(
        route_id=_text(value["route_id"], "route id"),
        claim_kind=_text(value["claim_kind"], "claim kind"),
        manifest_path=_text(value["manifest_path"], "manifest path"),
        manifest_sha256=_digest(value["manifest_sha256"], "manifest sha256"),
        terminal_declaration=_text(value["terminal_declaration"], "terminal declaration"),
        terminal_type_sha256=_digest(value["terminal_type_sha256"], "terminal type sha256"),
        receipt_path=_text(value["receipt_path"], "receipt path"),
        receipt_sha256=_optional_digest(value["receipt_sha256"], "receipt sha256"),
        review_path=_text(value["review_path"], "review path"),
        review_sha256=_optional_digest(value["review_sha256"], "review sha256"),
        allowed_axioms=_string_tuple(value["allowed_axioms"], "allowed axiom"),
    )


def _parse_node(value: object) -> GraphNode:
    if not isinstance(value, dict):
        raise TheoremGraphError("graph node must be an object")
    _exact_fields(
        value,
        {
            "node_id",
            "route_ids",
            "route_node_ids",
            "roles",
            "lean_name",
            "statement_sha256",
            "statement_text_paths",
            "natural_language_statement",
            "source_locators",
            "dependency_ids",
            "proof_status",
            "proof_receipt_paths",
            "allowed_axioms",
            "readback_status",
            "claim_ceiling",
        },
        "graph node",
    )
    natural = value["natural_language_statement"]
    if natural is not None:
        natural = _text(natural, "natural language statement")
    return GraphNode(
        node_id=_text(value["node_id"], "node id"),
        route_ids=_string_tuple(value["route_ids"], "route id"),
        route_node_ids=_string_tuple(value["route_node_ids"], "route node id"),
        roles=_string_tuple(value["roles"], "node role"),
        lean_name=_text(value["lean_name"], "lean name"),
        statement_sha256=_digest(value["statement_sha256"], "statement sha256"),
        statement_text_paths=_string_tuple(value["statement_text_paths"], "statement text path"),
        natural_language_statement=natural,
        source_locators=_string_tuple(value["source_locators"], "source locator", allow_empty=True),
        dependency_ids=_string_tuple(value["dependency_ids"], "dependency id", allow_empty=True),
        proof_status=_text(value["proof_status"], "proof status"),
        proof_receipt_paths=_string_tuple(value["proof_receipt_paths"], "proof receipt path", allow_empty=True),
        allowed_axioms=_string_tuple(value["allowed_axioms"], "allowed axiom", allow_empty=True),
        readback_status=_text(value["readback_status"], "readback status"),
        claim_ceiling=_text(value["claim_ceiling"], "claim ceiling"),
    )


def _read_manifest_bytes(repository_root: Path, manifest_path: Path) -> bytes:
    path = Path(repository_root) / manifest_path
    try:
        return path.read_bytes()
    except OSError as error:
        raise TheoremGraphError(f"cannot read manifest {manifest_path}: {error}") from error


def _sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def _unique_sorted(values: Iterable[str]) -> tuple[str, ...]:
    return tuple(sorted(set(values)))


def _preferred_node_id(parts: tuple[_NodePart, ...]) -> str:
    ordered = sorted(
        parts,
        key=lambda part: (
            0 if part.provenance_kind == "source" else 1,
            ROUTE_ORDER.get(part.route_id, len(ROUTE_ORDER)),
            part.route_node_id,
        ),
    )
    return ordered[0].route_node_id


def _slug(value: str) -> str:
    output: list[str] = []
    previous_dash = False
    for character in value:
        if character.isalnum():
            output.append(character.lower())
            previous_dash = False
        elif not previous_dash:
            output.append("-")
            previous_dash = True
    return "".join(output).strip("-")[:128] or "node"


def _exact_fields(value: Mapping[str, object], fields: set[str], label: str) -> None:
    unknown = sorted(set(value) - fields)
    missing = sorted(fields - set(value))
    if unknown:
        raise TheoremGraphError(f"{label} has unknown field: {unknown[0]}")
    if missing:
        raise TheoremGraphError(f"{label} is missing field: {missing[0]}")


def _text(value: object, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise TheoremGraphError(f"{label} must be a nonempty string")
    return value


def _digest(value: object, label: str) -> str:
    text = _text(value, label)
    if route_validation.SHA256_RE.fullmatch(text) is None:
        raise TheoremGraphError(f"{label} must be a sha256 digest")
    return text


def _optional_digest(value: object, label: str) -> str | None:
    if value is None:
        return None
    return _digest(value, label)


def _string_tuple(
    value: object,
    label: str,
    *,
    allow_empty: bool = False,
) -> tuple[str, ...]:
    if not isinstance(value, list) or (not allow_empty and not value):
        raise TheoremGraphError(f"{label} must be a nonempty array")
    result = tuple(_text(item, label) for item in value)
    if len(result) != len(set(result)):
        raise TheoremGraphError(f"{label} contains duplicates")
    return result
