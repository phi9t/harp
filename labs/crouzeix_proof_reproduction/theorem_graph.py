"""Build a route-independent Crouzeix theorem graph from route manifests."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping

try:
    from . import graph_kernel, route_validation
except ImportError:  # pragma: no cover - direct script import path
    import graph_kernel
    import route_validation


GRAPH_SCHEMA_VERSION = "crouzeix-theorem-graph/v1"
OBLIGATION_GRAPH_SCHEMA_VERSION = "crouzeix-proof-obligation-graph/v2"
OBLIGATION_LEDGER_SCHEMA_VERSION = "crouzeix-proof-obligation-ledger/v2"
PROVE2ME_EXPORT_SCHEMA_VERSION = "crouzeix-prove2me-export/v2"
READBACK_SCHEMA_VERSION = "crouzeix-theorem-readbacks/v1"
READBACK_PATH = Path(
    "labs/crouzeix_proof_reproduction/formal_targets/theorem-readbacks.json"
)
OBLIGATION_LEDGER_PATH = Path(
    "labs/crouzeix_proof_reproduction/formal_targets/proof-obligations.json"
)
PLANNED_OBLIGATION_STATEMENTS_PATH = Path(
    "labs/crouzeix_proof_reproduction/formal_targets/harp/planned-obligation-statements"
)
GRAPH_ROUTE_IDS = ("jin", "lorist-schwenninger", "harp")
ROUTE_ORDER = {route_id: index for index, route_id in enumerate(GRAPH_ROUTE_IDS)}
PROOF_STATUSES = ("blocked", "deprecated", "open", "passed-external", "passed-local")
READBACK_STATUSES = ("current", "flagged", "missing", "stale")
OBLIGATION_KINDS = (
    "derived-obligation",
    "planned-obligation",
    "reused-obligation",
    "shared-obligation",
)
OBLIGATION_ROLES = ("intermediate", "parent-completion", "terminal")
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
class ExtractionSource:
    module_path: str
    declaration: str

    def to_json(self) -> dict[str, str]:
        return {
            "module_path": self.module_path,
            "declaration": self.declaration,
        }


@dataclass(frozen=True)
class ProofObligationNode:
    node_id: str
    parent_node_id: str
    route_ids: tuple[str, ...]
    kind: str
    role: str
    route_node_id: str | None
    lean_name: str | None
    statement_sha256: str
    statement_text_paths: tuple[str, ...]
    dependency_ids: tuple[str, ...]
    proof_status: str
    proof_receipt_paths: tuple[str, ...]
    allowed_axioms: tuple[str, ...]
    readback_status: str
    natural_language_statement: str | None
    prove2me_candidate: bool
    covers_parent: bool
    proposed_lean_name: str | None
    extraction_source: ExtractionSource | None
    claim_ceiling: str
    source_locators: tuple[str, ...]

    def to_json(self) -> dict[str, object]:
        return {
            "node_id": self.node_id,
            "parent_node_id": self.parent_node_id,
            "route_ids": list(self.route_ids),
            "kind": self.kind,
            "role": self.role,
            "route_node_id": self.route_node_id,
            "lean_name": self.lean_name,
            "statement_sha256": self.statement_sha256,
            "statement_text_paths": list(self.statement_text_paths),
            "dependency_ids": list(self.dependency_ids),
            "proof_status": self.proof_status,
            "proof_receipt_paths": list(self.proof_receipt_paths),
            "allowed_axioms": list(self.allowed_axioms),
            "readback_status": self.readback_status,
            "natural_language_statement": self.natural_language_statement,
            "prove2me_candidate": self.prove2me_candidate,
            "covers_parent": self.covers_parent,
            "proposed_lean_name": self.proposed_lean_name,
            "extraction_source": (
                self.extraction_source.to_json()
                if self.extraction_source is not None
                else None
            ),
            "claim_ceiling": self.claim_ceiling,
            "source_locators": list(self.source_locators),
        }


@dataclass(frozen=True)
class ProofObligationParent:
    parent_node_id: str
    child_ids: tuple[str, ...]
    obligation_status: str

    def to_json(self) -> dict[str, object]:
        return {
            "parent_node_id": self.parent_node_id,
            "child_ids": list(self.child_ids),
            "obligation_status": self.obligation_status,
        }


@dataclass(frozen=True)
class ProofObligationGraph:
    schema_version: str
    route_node_ids: tuple[str, ...]
    parents: tuple[ProofObligationParent, ...]
    nodes: tuple[ProofObligationNode, ...]
    frontier: dict[str, tuple[str, ...]]

    def to_json(self) -> dict[str, object]:
        return {
            "schema_version": self.schema_version,
            "route_node_ids": list(self.route_node_ids),
            "parents": [parent.to_json() for parent in self.parents],
            "nodes": [node.to_json() for node in self.nodes],
            "frontier": {key: list(value) for key, value in sorted(self.frontier.items())},
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


@dataclass(frozen=True)
class _Readback:
    node_id: str
    lean_name: str
    statement_sha256: str
    statement_text_paths: tuple[str, ...]
    readback_status: str
    natural_language_statement: str | None


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
    nodes = _apply_readbacks(nodes, _load_readbacks(root))
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


def build_proof_obligation_graph(repository_root: Path) -> ProofObligationGraph:
    root = Path(repository_root)
    route_graph = build_theorem_graph(root)
    route_nodes = {node.node_id: node for node in route_graph.nodes}
    payload = _read_json(root, OBLIGATION_LEDGER_PATH, "proof-obligation ledger")
    _exact_fields(payload, {"schema_version", "obligations"}, "proof-obligation ledger")
    if payload["schema_version"] != OBLIGATION_LEDGER_SCHEMA_VERSION:
        raise TheoremGraphError("invalid proof-obligation ledger schema version")
    raw_nodes = payload["obligations"]
    if not isinstance(raw_nodes, list) or not raw_nodes:
        raise TheoremGraphError("proof obligations must be a nonempty array")
    nodes = _bind_obligation_metadata(
        tuple(_parse_obligation_node(value) for value in raw_nodes), route_nodes
    )
    _validate_obligation_graph(root, route_nodes, nodes)
    nodes = _topological_order_obligations(nodes, set(route_nodes))
    parents = _obligation_parents(nodes)
    frontier = _obligation_frontier(route_graph.nodes, parents, nodes)
    return ProofObligationGraph(
        schema_version=OBLIGATION_GRAPH_SCHEMA_VERSION,
        route_node_ids=tuple(node.node_id for node in route_graph.nodes),
        parents=parents,
        nodes=nodes,
        frontier=frontier,
    )


def build_proof_obligation_graph_json(repository_root: Path) -> dict[str, object]:
    return build_proof_obligation_graph(repository_root).to_json()


def validate_proof_obligation_graph(
    value: Mapping[str, object], repository_root: Path
) -> ProofObligationGraph:
    _exact_fields(
        value,
        {"schema_version", "route_node_ids", "parents", "nodes", "frontier"},
        "proof-obligation graph",
    )
    if value["schema_version"] != OBLIGATION_GRAPH_SCHEMA_VERSION:
        raise TheoremGraphError("invalid proof-obligation graph schema version")
    route_graph = build_theorem_graph(repository_root)
    route_nodes = {node.node_id: node for node in route_graph.nodes}
    route_node_ids = _string_tuple(value["route_node_ids"], "route graph node id")
    expected_route_ids = tuple(route_nodes)
    if route_node_ids != expected_route_ids:
        raise TheoremGraphError("proof-obligation route node roster is stale")
    raw_nodes = value["nodes"]
    if not isinstance(raw_nodes, list) or not raw_nodes:
        raise TheoremGraphError("proof-obligation nodes must be a nonempty array")
    nodes = tuple(
        _parse_serialized_obligation_node(item, route_nodes) for item in raw_nodes
    )
    _validate_obligation_graph(Path(repository_root), route_nodes, nodes)
    ordered = _topological_order_obligations(nodes, set(route_nodes))
    if nodes != ordered:
        raise TheoremGraphError("proof-obligation nodes are not in canonical order")
    parents = _parse_obligation_parents(value["parents"])
    observed_parents = _obligation_parents(nodes)
    if parents != observed_parents:
        raise TheoremGraphError("proof-obligation parent summaries do not match nodes")
    frontier = _parse_obligation_frontier(value["frontier"])
    observed_frontier = _obligation_frontier(route_graph.nodes, parents, nodes)
    if frontier != observed_frontier:
        raise TheoremGraphError("proof-obligation frontier does not match graph nodes")
    return ProofObligationGraph(
        schema_version=OBLIGATION_GRAPH_SCHEMA_VERSION,
        route_node_ids=route_node_ids,
        parents=parents,
        nodes=nodes,
        frontier=frontier,
    )


def build_prove2me_export(repository_root: Path) -> dict[str, object]:
    graph = build_proof_obligation_graph(repository_root)
    route_graph = build_theorem_graph(repository_root)
    candidates = set(graph.frontier["prove2me_candidates"])
    obligation_by_id = {node.node_id: node for node in graph.nodes}
    route_by_id = {node.node_id: node for node in route_graph.nodes}
    cards = []
    for node in graph.nodes:
        if node.node_id not in candidates:
            continue
        cards.append(
            {
                "theorem_id": node.node_id,
                "parent_theorem_id": node.parent_node_id,
                "lean_name": node.lean_name,
                "statement_sha256": node.statement_sha256,
                "statement_text_paths": list(node.statement_text_paths),
                "dependencies": list(node.dependency_ids),
                "natural_language_readback": node.natural_language_statement,
                "claim_ceiling": node.claim_ceiling,
                "local_proof_status": node.proof_status,
                "allowed_axioms": list(node.allowed_axioms),
                "source_locators": list(node.source_locators),
                "authorship_status": (
                    "reused-route"
                    if node.kind == "reused-obligation"
                    else "harp-authored"
                ),
            }
        )
    prerequisite_ids = _dependency_closure(
        tuple(card["theorem_id"] for card in cards),
        candidates,
        obligation_by_id,
        route_by_id,
    )
    prerequisites = [
        _prerequisite_record(
            theorem_id, obligation_by_id.get(theorem_id), route_by_id.get(theorem_id)
        )
        for theorem_id in prerequisite_ids
    ]
    exported_ids = candidates | set(prerequisite_ids)
    for item in (*cards, *prerequisites):
        unresolved = set(item["dependencies"]) - exported_ids
        if unresolved:
            raise TheoremGraphError(
                f"unresolved Prove2Me export dependency: {sorted(unresolved)[0]}"
            )
    return {
        "schema_version": PROVE2ME_EXPORT_SCHEMA_VERSION,
        "dry_run": True,
        "network_access": False,
        "theorem_cards": cards,
        "prerequisites": prerequisites,
    }


def _dependency_closure(
    roots: tuple[str, ...],
    excluded: set[str],
    obligation_by_id: Mapping[str, ProofObligationNode],
    route_by_id: Mapping[str, GraphNode],
) -> tuple[str, ...]:
    dependencies = {
        node_id: node.dependency_ids
        for node_id, node in {**route_by_id, **obligation_by_id}.items()
    }
    prerequisite_roots = tuple(
        dependency
        for root in roots
        for dependency in sorted(obligation_by_id[root].dependency_ids)
    )
    try:
        return graph_kernel.dependency_closure(
            prerequisite_roots, dependencies, excluded=excluded
        )
    except graph_kernel.GraphKernelError as error:
        detail = str(error)
        if detail.startswith(("unknown dependency: ", "unknown root: ")):
            theorem_id = detail.partition(": ")[2]
            detail = f"unresolved Prove2Me export dependency: {theorem_id}"
        raise TheoremGraphError(detail) from error


def _prerequisite_record(
    theorem_id: str,
    obligation: ProofObligationNode | None,
    route: GraphNode | None,
) -> dict[str, object]:
    node = obligation or route
    if node is None:
        raise TheoremGraphError(
            f"unresolved Prove2Me export dependency: {theorem_id}"
        )
    record: dict[str, object] = {
        "theorem_id": theorem_id,
        "node_kind": "obligation" if obligation is not None else "route",
        "lean_name": node.lean_name,
        "statement_sha256": node.statement_sha256,
        "statement_text_paths": list(node.statement_text_paths),
        "dependencies": list(node.dependency_ids),
        "natural_language_readback": node.natural_language_statement,
        "claim_ceiling": node.claim_ceiling,
        "local_proof_status": node.proof_status,
        "allowed_axioms": list(node.allowed_axioms),
        "source_locators": list(node.source_locators),
    }
    if obligation is not None:
        record["parent_theorem_id"] = obligation.parent_node_id
        record["proposed_lean_name"] = obligation.proposed_lean_name
        record["extraction_source"] = (
            obligation.extraction_source.to_json()
            if obligation.extraction_source is not None
            else None
        )
    return record


def _parse_obligation_node(value: object) -> ProofObligationNode:
    if not isinstance(value, dict):
        raise TheoremGraphError("proof obligation must be an object")
    _exact_fields(
        value,
        {
            "node_id",
            "parent_node_id",
            "route_ids",
            "kind",
            "role",
            "route_node_id",
            "lean_name",
            "statement_sha256",
            "statement_text_paths",
            "dependency_ids",
            "proof_status",
            "proof_receipt_paths",
            "allowed_axioms",
            "readback_status",
            "natural_language_statement",
            "prove2me_candidate",
            "covers_parent",
            "proposed_lean_name",
            "extraction_source",
        },
        "proof obligation",
    )
    route_node_id = value["route_node_id"]
    if route_node_id is not None:
        route_node_id = _text(route_node_id, "obligation route node id")
    lean_name = value["lean_name"]
    if lean_name is not None:
        lean_name = _text(lean_name, "obligation Lean name")
    natural = value["natural_language_statement"]
    if natural is not None:
        natural = _text(natural, "obligation natural language statement")
    candidate = value["prove2me_candidate"]
    if not isinstance(candidate, bool):
        raise TheoremGraphError("obligation Prove2Me candidate must be a boolean")
    covers_parent = value["covers_parent"]
    if not isinstance(covers_parent, bool):
        raise TheoremGraphError("obligation covers_parent must be a boolean")
    proposed_lean_name = value["proposed_lean_name"]
    if proposed_lean_name is not None:
        proposed_lean_name = _text(
            proposed_lean_name, "obligation proposed Lean name"
        )
    extraction_source = _parse_extraction_source(value["extraction_source"])
    kind = _text(value["kind"], "obligation kind")
    if kind not in OBLIGATION_KINDS:
        raise TheoremGraphError(f"invalid obligation kind: {kind}")
    role = _text(value["role"], "obligation role")
    if role not in OBLIGATION_ROLES:
        raise TheoremGraphError(f"invalid obligation role: {role}")
    route_ids = _string_tuple(value["route_ids"], "obligation route id")
    unknown_routes = set(route_ids) - set(GRAPH_ROUTE_IDS)
    if unknown_routes:
        raise TheoremGraphError(f"unknown obligation route: {sorted(unknown_routes)[0]}")
    if tuple(sorted(route_ids, key=ROUTE_ORDER.__getitem__)) != route_ids:
        raise TheoremGraphError("obligation routes are not in canonical order")
    return ProofObligationNode(
        node_id=_text(value["node_id"], "obligation node id"),
        parent_node_id=_text(value["parent_node_id"], "obligation parent node id"),
        route_ids=route_ids,
        kind=kind,
        role=role,
        route_node_id=route_node_id,
        lean_name=lean_name,
        statement_sha256=_digest(value["statement_sha256"], "obligation statement sha256"),
        statement_text_paths=_string_tuple(
            value["statement_text_paths"], "obligation statement text path"
        ),
        dependency_ids=_string_tuple(
            value["dependency_ids"], "obligation dependency id", allow_empty=True
        ),
        proof_status=_text(value["proof_status"], "obligation proof status"),
        proof_receipt_paths=_string_tuple(
            value["proof_receipt_paths"], "obligation proof receipt path", allow_empty=True
        ),
        allowed_axioms=_string_tuple(
            value["allowed_axioms"], "obligation allowed axiom", allow_empty=True
        ),
        readback_status=_text(value["readback_status"], "obligation readback status"),
        natural_language_statement=natural,
        prove2me_candidate=candidate,
        covers_parent=covers_parent,
        proposed_lean_name=proposed_lean_name,
        extraction_source=extraction_source,
        claim_ceiling=(
            "reused-route-local-certification"
            if kind == "reused-obligation"
            else "harp-derived-local-certification"
        ),
        source_locators=(),
    )


def _parse_extraction_source(value: object) -> ExtractionSource | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        raise TheoremGraphError("obligation extraction source must be an object")
    _exact_fields(
        value, {"module_path", "declaration"}, "obligation extraction source"
    )
    return ExtractionSource(
        module_path=_text(value["module_path"], "extraction source module path"),
        declaration=_text(value["declaration"], "extraction source declaration"),
    )


def _parse_serialized_obligation_node(
    value: object, route_nodes: Mapping[str, GraphNode]
) -> ProofObligationNode:
    if not isinstance(value, dict):
        raise TheoremGraphError("proof-obligation graph node must be an object")
    derived_fields = {"claim_ceiling", "source_locators"}
    raw = dict(value)
    if not derived_fields.issubset(raw):
        raise TheoremGraphError("proof-obligation graph node lacks derived fields")
    claim_ceiling = raw.pop("claim_ceiling")
    source_locators = raw.pop("source_locators")
    node = _bind_obligation_metadata((_parse_obligation_node(raw),), route_nodes)[0]
    if claim_ceiling != node.claim_ceiling:
        raise TheoremGraphError(f"obligation claim ceiling mismatch: {node.node_id}")
    if _string_tuple(source_locators, "obligation source locator", allow_empty=True) != node.source_locators:
        raise TheoremGraphError(f"obligation source locator mismatch: {node.node_id}")
    return node


def _bind_obligation_metadata(
    nodes: tuple[ProofObligationNode, ...],
    route_nodes: Mapping[str, GraphNode],
) -> tuple[ProofObligationNode, ...]:
    bound = []
    for node in nodes:
        route_node = route_nodes.get(node.route_node_id) if node.route_node_id else None
        bound.append(
            ProofObligationNode(
                node_id=node.node_id,
                parent_node_id=node.parent_node_id,
                route_ids=node.route_ids,
                kind=node.kind,
                role=node.role,
                route_node_id=node.route_node_id,
                lean_name=node.lean_name,
                statement_sha256=node.statement_sha256,
                statement_text_paths=node.statement_text_paths,
                dependency_ids=node.dependency_ids,
                proof_status=node.proof_status,
                proof_receipt_paths=node.proof_receipt_paths,
                allowed_axioms=node.allowed_axioms,
                readback_status=node.readback_status,
                natural_language_statement=node.natural_language_statement,
                prove2me_candidate=node.prove2me_candidate,
                covers_parent=node.covers_parent,
                proposed_lean_name=node.proposed_lean_name,
                extraction_source=node.extraction_source,
                claim_ceiling=(
                    route_node.claim_ceiling if route_node is not None else node.claim_ceiling
                ),
                source_locators=(
                    route_node.source_locators if route_node is not None else ()
                ),
            )
        )
    return tuple(bound)


def _parse_obligation_parents(value: object) -> tuple[ProofObligationParent, ...]:
    if not isinstance(value, list) or not value:
        raise TheoremGraphError("proof-obligation parents must be a nonempty array")
    parents: list[ProofObligationParent] = []
    for raw in value:
        if not isinstance(raw, dict):
            raise TheoremGraphError("proof-obligation parent must be an object")
        _exact_fields(raw, {"parent_node_id", "child_ids", "obligation_status"}, "proof-obligation parent")
        parents.append(
            ProofObligationParent(
                parent_node_id=_text(raw["parent_node_id"], "parent node id"),
                child_ids=_string_tuple(raw["child_ids"], "parent child id"),
                obligation_status=_text(raw["obligation_status"], "parent obligation status"),
            )
        )
    return tuple(parents)


def _parse_obligation_frontier(value: object) -> dict[str, tuple[str, ...]]:
    if not isinstance(value, dict):
        raise TheoremGraphError("proof-obligation frontier must be an object")
    fields = {
        "blocked_obligations",
        "missing_obligation_readbacks",
        "open_obligations",
        "oversized_parent_nodes",
        "prove2me_candidates",
    }
    _exact_fields(value, fields, "proof-obligation frontier")
    return {
        key: _string_tuple(value[key], "proof-obligation frontier node id", allow_empty=True)
        for key in sorted(fields)
    }


def _validate_obligation_graph(
    repository_root: Path,
    route_nodes: Mapping[str, GraphNode],
    nodes: tuple[ProofObligationNode, ...],
) -> None:
    ids = [node.node_id for node in nodes]
    if len(ids) != len(set(ids)):
        raise TheoremGraphError("duplicate proof-obligation node id")
    obligation_ids = set(ids)
    obligation_by_id = {node.node_id: node for node in nodes}
    route_ids = set(route_nodes)
    for node in nodes:
        if route_validation.NODE_ID_RE.fullmatch(node.node_id) is None:
            raise TheoremGraphError(f"invalid proof-obligation node id: {node.node_id}")
        if node.parent_node_id not in route_ids:
            raise TheoremGraphError(f"unknown obligation parent: {node.parent_node_id}")
        if node.kind not in OBLIGATION_KINDS:
            raise TheoremGraphError(f"invalid obligation kind: {node.kind}")
        if node.role not in OBLIGATION_ROLES:
            raise TheoremGraphError(f"invalid obligation role: {node.role}")
        unknown_routes = set(node.route_ids) - set(GRAPH_ROUTE_IDS)
        if unknown_routes:
            raise TheoremGraphError(f"unknown obligation route: {sorted(unknown_routes)[0]}")
        if tuple(sorted(node.route_ids, key=ROUTE_ORDER.__getitem__)) != node.route_ids:
            raise TheoremGraphError(f"obligation routes are not canonical: {node.node_id}")
        if not set(node.route_ids).issubset(route_nodes[node.parent_node_id].route_ids):
            raise TheoremGraphError(f"obligation route does not match parent: {node.node_id}")
        if node.proof_status not in PROOF_STATUSES:
            raise TheoremGraphError(f"invalid obligation proof status: {node.proof_status}")
        if node.readback_status not in READBACK_STATUSES:
            raise TheoremGraphError(f"invalid obligation readback status: {node.readback_status}")
        if node.readback_status == "stale":
            raise TheoremGraphError(f"stale obligation readback: {node.node_id}")
        if node.readback_status == "current" and node.natural_language_statement is None:
            raise TheoremGraphError(f"current obligation lacks readback statement: {node.node_id}")
        if node.proof_status in {"passed-local", "passed-external"}:
            if node.lean_name is None:
                raise TheoremGraphError(f"passed obligation lacks Lean statement: {node.node_id}")
            if not node.proof_receipt_paths:
                raise TheoremGraphError(f"passed obligation lacks receipt: {node.node_id}")
        if node.readback_status == "current" and node.lean_name is None:
            raise TheoremGraphError(f"unbound current readback: {node.node_id}")
        if node.proof_status == "passed-local" and not node.allowed_axioms:
            raise TheoremGraphError(f"passed local obligation lacks axiom policy: {node.node_id}")
        if node.lean_name is None and node.proof_status != "open":
            raise TheoremGraphError(f"unbound obligation must remain open: {node.node_id}")
        if node.kind == "planned-obligation":
            expected_path = (PLANNED_OBLIGATION_STATEMENTS_PATH / f"{node.node_id}.txt").as_posix()
            if node.statement_text_paths != (expected_path,):
                raise TheoremGraphError(f"invalid planned statement path: {node.node_id}")
            if (
                node.route_node_id is not None
                or node.lean_name is not None
                or node.proof_status != "open"
                or node.proof_receipt_paths
                or node.allowed_axioms
                or node.prove2me_candidate
                or node.covers_parent
            ):
                raise TheoremGraphError(f"invalid planned obligation state: {node.node_id}")
            _validate_extraction_source(repository_root, node)
        elif node.route_node_id is None:
            raise TheoremGraphError(f"proof-bearing obligation lacks route node: {node.node_id}")
        elif node.proposed_lean_name is not None or node.extraction_source is not None:
            raise TheoremGraphError(
                f"proof-bearing obligation has planned extraction metadata: {node.node_id}"
            )
        if node.prove2me_candidate and (
            node.lean_name is None
            or node.proof_status != "passed-local"
            or node.readback_status != "current"
            or (node.kind == "reused-obligation" and not node.source_locators)
        ):
            raise TheoremGraphError(f"invalid Prove2Me candidate: {node.node_id}")
        statement_parts = [
            _read_obligation_text(repository_root, path, node.node_id)
            for path in node.statement_text_paths
        ]
        statement_text = "\n".join(statement_parts)
        if route_validation.normalized_type_sha256(statement_text) != node.statement_sha256:
            raise TheoremGraphError(f"obligation statement hash mismatch: {node.node_id}")
        if node.route_node_id is not None:
            route_node = route_nodes.get(node.route_node_id)
            if route_node is None:
                raise TheoremGraphError(f"unknown bound route node: {node.route_node_id}")
            _validate_route_node_binding(node, route_node)
        if node.proof_status == "passed-local":
            _validate_receipt_binding(repository_root, node)
        for dependency in node.dependency_ids:
            if dependency not in route_ids and dependency not in obligation_ids:
                raise TheoremGraphError(f"dangling obligation dependency: {dependency}")
            if dependency == node.node_id:
                raise TheoremGraphError(f"self obligation dependency: {node.node_id}")
            dependent = obligation_by_id.get(dependency)
            route_dependency = route_nodes.get(dependency)
            if (
                node.proof_status in {"passed-local", "passed-external"}
                and (
                    dependent is not None
                    and not node.covers_parent
                    and (
                        dependent.proof_status not in {"passed-local", "passed-external"}
                        or dependent.readback_status != "current"
                    )
                    or route_dependency is not None
                    and route_dependency.proof_status
                    not in {"passed-local", "passed-external"}
                )
            ):
                raise TheoremGraphError(
                    f"unproved obligation dependency: {dependency}"
                )
            if dependent is not None and dependent.parent_node_id != node.parent_node_id:
                if dependent.lean_name is None:
                    raise TheoremGraphError(
                        f"cross-parent dependency lacks Lean statement: {dependency}"
                    )
            if not node.covers_parent and node.route_node_id is not None:
                bound_dependencies = set(route_nodes[node.route_node_id].dependency_ids)
                represented_route_id = (
                    dependent.route_node_id if dependent is not None else dependency
                )
                if represented_route_id not in bound_dependencies:
                    raise TheoremGraphError(
                        f"unjustified obligation dependency: {node.node_id} -> {dependency}"
                    )
    _validate_parent_coverages(route_nodes, nodes)
    _topological_order_obligations(nodes, route_ids)


def _validate_extraction_source(
    repository_root: Path, node: ProofObligationNode
) -> None:
    if node.proposed_lean_name is None or node.extraction_source is None:
        raise TheoremGraphError(
            f"planned obligation lacks extraction source: {node.node_id}"
        )
    source = node.extraction_source
    module_path = Path(source.module_path)
    lean_root = Path("formalization/lean")
    if (
        module_path.suffix != ".lean"
        or not module_path.is_relative_to(lean_root)
    ):
        raise TheoremGraphError(
            f"invalid extraction source module: {node.node_id}"
        )
    text = _read_obligation_text(repository_root, source.module_path, node.node_id)
    declaration_name = source.declaration.rsplit(".", 1)[-1]
    if not any(
        marker in text
        for marker in (
            f"theorem {declaration_name}",
            f"def {declaration_name}",
            f"lemma {declaration_name}",
        )
    ):
        raise TheoremGraphError(
            f"extraction source declaration not found: {node.node_id}"
        )


def _validate_parent_coverages(
    route_nodes: Mapping[str, GraphNode],
    nodes: tuple[ProofObligationNode, ...],
) -> None:
    grouped: dict[str, list[ProofObligationNode]] = {}
    for node in nodes:
        grouped.setdefault(node.parent_node_id, []).append(node)
    for parent_id, children in grouped.items():
        completions = [child for child in children if child.covers_parent]
        if len(completions) != 1:
            raise TheoremGraphError(
                f"parent requires exactly one covering completion: {parent_id}"
            )
        completion = completions[0]
        parent = route_nodes[parent_id]
        if (
            completion.role not in {"parent-completion", "terminal"}
            or completion.route_node_id != parent_id
            or completion.lean_name != parent.lean_name
            or completion.statement_sha256 != parent.statement_sha256
        ):
            raise TheoremGraphError(
                f"invalid covering completion binding: {completion.node_id}"
            )
        sibling_ids = {child.node_id for child in children if child is not completion}
        dependency_ids = set(completion.dependency_ids)
        if not sibling_ids.issubset(dependency_ids):
            raise TheoremGraphError(
                f"covering completion does not cover children: {completion.node_id}"
            )
        by_id = {node.node_id: node for node in nodes}
        for dependency in dependency_ids - sibling_ids:
            child = by_id.get(dependency)
            if (
                child is None
                or child.parent_node_id == parent_id
                or child.route_node_id not in parent.dependency_ids
            ):
                raise TheoremGraphError(
                    f"invalid covering completion dependency: {completion.node_id} -> {dependency}"
                )


def _validate_route_node_binding(
    obligation: ProofObligationNode, route_node: GraphNode
) -> None:
    if obligation.lean_name != route_node.lean_name:
        raise TheoremGraphError(f"obligation Lean name mismatch: {obligation.node_id}")
    if obligation.statement_sha256 != route_node.statement_sha256:
        raise TheoremGraphError(f"obligation statement hash mismatch: {obligation.node_id}")
    if not set(obligation.statement_text_paths).issubset(route_node.statement_text_paths):
        raise TheoremGraphError(f"obligation statement path mismatch: {obligation.node_id}")
    if not set(obligation.proof_receipt_paths).issubset(route_node.proof_receipt_paths):
        raise TheoremGraphError(f"obligation receipt mismatch: {obligation.node_id}")
    if not set(obligation.allowed_axioms).issubset(route_node.allowed_axioms):
        raise TheoremGraphError(f"obligation axiom policy mismatch: {obligation.node_id}")
    if obligation.claim_ceiling != route_node.claim_ceiling:
        raise TheoremGraphError(f"obligation claim ceiling mismatch: {obligation.node_id}")
    if obligation.source_locators != route_node.source_locators:
        raise TheoremGraphError(f"obligation source locator mismatch: {obligation.node_id}")


def _validate_receipt_binding(
    repository_root: Path, obligation: ProofObligationNode
) -> None:
    for receipt_path in obligation.proof_receipt_paths:
        receipt = _read_json(
            repository_root,
            Path(receipt_path),
            f"proof receipt for {obligation.node_id}",
        )
        raw_results = receipt.get("axiom_results")
        if not isinstance(raw_results, list):
            raise TheoremGraphError(
                f"proof receipt lacks axiom results: {obligation.node_id}"
            )
        matching = [
            result
            for result in raw_results
            if isinstance(result, dict)
            and result.get("declaration") == obligation.lean_name
        ]
        if not matching:
            raise TheoremGraphError(
                f"proof receipt lacks declaration: {obligation.node_id}"
            )
        if any(result.get("axioms") != list(obligation.allowed_axioms) for result in matching):
            raise TheoremGraphError(
                f"proof receipt axiom mismatch: {obligation.node_id}"
            )


def _read_obligation_text(repository_root: Path, value: str, node_id: str) -> str:
    try:
        data = route_validation._read_bytes(
            repository_root, value, f"obligation artifact for {node_id}"
        )
        return data.decode("utf-8")
    except (route_validation.RouteValidationError, UnicodeDecodeError) as error:
        raise TheoremGraphError(f"cannot read obligation artifact: {node_id}: {error}") from error


def _topological_order_obligations(
    nodes: tuple[ProofObligationNode, ...], route_node_ids: set[str]
) -> tuple[ProofObligationNode, ...]:
    by_id = {node.node_id: node for node in nodes}
    dependencies = {node_id: node.dependency_ids for node_id, node in by_id.items()}
    try:
        ordered = graph_kernel.topological_order(
            dependencies, external_ids=route_node_ids, cycle_label="obligation"
        )
    except graph_kernel.GraphKernelError as error:
        raise TheoremGraphError(str(error)) from error
    return tuple(by_id[node_id] for node_id in ordered)


def _obligation_parents(
    nodes: tuple[ProofObligationNode, ...],
) -> tuple[ProofObligationParent, ...]:
    grouped: dict[str, list[ProofObligationNode]] = {}
    for node in nodes:
        grouped.setdefault(node.parent_node_id, []).append(node)
    parents = []
    for parent_id, children in sorted(grouped.items()):
        current = all(
            child.proof_status in {"passed-local", "passed-external"}
            and child.readback_status == "current"
            for child in children
        )
        parents.append(
            ProofObligationParent(
                parent_node_id=parent_id,
                child_ids=tuple(sorted(child.node_id for child in children)),
                obligation_status="expanded-current" if current else "expanded-incomplete",
            )
        )
    return tuple(parents)


def _obligation_frontier(
    route_nodes: tuple[GraphNode, ...],
    parents: tuple[ProofObligationParent, ...],
    nodes: tuple[ProofObligationNode, ...],
) -> dict[str, tuple[str, ...]]:
    expanded = {parent.parent_node_id for parent in parents}
    route_by_id = {node.node_id: node for node in route_nodes}
    oversized = tuple(
        node.node_id
        for node in route_nodes
        if node.node_id not in expanded
        and len(node.dependency_ids) >= 4
    )
    missing_readbacks = tuple(
        node.node_id
        for node in nodes
        if node.readback_status != "current"
        and (
            node.proof_status in {"passed-local", "passed-external"}
            or node.prove2me_candidate
            or node.kind == "reused-obligation"
            or node.role == "terminal"
            or node.source_locators
            or "terminal" in route_by_id[node.parent_node_id].roles
        )
    )
    return {
        "blocked_obligations": tuple(node.node_id for node in nodes if node.proof_status == "blocked"),
        "missing_obligation_readbacks": missing_readbacks,
        "open_obligations": tuple(node.node_id for node in nodes if node.proof_status == "open"),
        "oversized_parent_nodes": oversized,
        "prove2me_candidates": tuple(node.node_id for node in nodes if node.prove2me_candidate),
    }


def _read_json(
    repository_root: Path, relative_path: Path, label: str
) -> dict[str, object]:
    try:
        value = route_validation._read_json(repository_root, relative_path, label)
    except route_validation.RouteValidationError as error:
        raise TheoremGraphError(f"cannot read {label}: {error}") from error
    return value


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


def _load_readbacks(repository_root: Path) -> dict[str, _Readback]:
    path = repository_root / READBACK_PATH
    if not path.exists() and not path.is_symlink():
        return {}
    payload = _read_json(repository_root, READBACK_PATH, "theorem readbacks")
    _exact_fields(payload, {"schema_version", "readbacks"}, "theorem readbacks")
    if payload["schema_version"] != READBACK_SCHEMA_VERSION:
        raise TheoremGraphError("invalid theorem readback schema version")
    raw_readbacks = payload["readbacks"]
    if not isinstance(raw_readbacks, list):
        raise TheoremGraphError("theorem readbacks must be an array")
    readbacks: dict[str, _Readback] = {}
    for raw in raw_readbacks:
        readback = _parse_readback(raw)
        if readback.node_id in readbacks:
            raise TheoremGraphError(f"duplicate theorem readback: {readback.node_id}")
        readbacks[readback.node_id] = readback
    return readbacks


def _parse_readback(value: object) -> _Readback:
    if not isinstance(value, dict):
        raise TheoremGraphError("theorem readback must be an object")
    _exact_fields(
        value,
        {
            "node_id",
            "lean_name",
            "statement_sha256",
            "statement_text_paths",
            "readback_status",
            "natural_language_statement",
        },
        "theorem readback",
    )
    natural = value["natural_language_statement"]
    if natural is not None:
        natural = _text(natural, "natural language statement")
    readback = _Readback(
        node_id=_text(value["node_id"], "readback node id"),
        lean_name=_text(value["lean_name"], "readback Lean name"),
        statement_sha256=_digest(value["statement_sha256"], "readback statement sha256"),
        statement_text_paths=_string_tuple(
            value["statement_text_paths"], "readback statement text path"
        ),
        readback_status=_text(value["readback_status"], "readback status"),
        natural_language_statement=natural,
    )
    if route_validation.NODE_ID_RE.fullmatch(readback.node_id) is None:
        raise TheoremGraphError(f"invalid readback node id: {readback.node_id}")
    if readback.readback_status not in READBACK_STATUSES:
        raise TheoremGraphError(f"invalid readback status: {readback.readback_status}")
    if readback.readback_status == "current" and readback.natural_language_statement is None:
        raise TheoremGraphError(f"current readback lacks natural statement: {readback.node_id}")
    return readback


def _apply_readbacks(
    nodes: tuple[GraphNode, ...],
    readbacks: Mapping[str, _Readback],
) -> tuple[GraphNode, ...]:
    if not readbacks:
        return nodes
    by_id = {node.node_id: node for node in nodes}
    unknown = sorted(set(readbacks) - set(by_id))
    if unknown:
        raise TheoremGraphError(f"readback references unknown node: {unknown[0]}")
    updated: list[GraphNode] = []
    for node in nodes:
        readback = readbacks.get(node.node_id)
        if readback is None:
            updated.append(node)
            continue
        _validate_readback_binding(node, readback)
        updated.append(
            GraphNode(
                node_id=node.node_id,
                route_ids=node.route_ids,
                route_node_ids=node.route_node_ids,
                roles=node.roles,
                lean_name=node.lean_name,
                statement_sha256=node.statement_sha256,
                statement_text_paths=node.statement_text_paths,
                natural_language_statement=readback.natural_language_statement,
                source_locators=node.source_locators,
                dependency_ids=node.dependency_ids,
                proof_status=node.proof_status,
                proof_receipt_paths=node.proof_receipt_paths,
                allowed_axioms=node.allowed_axioms,
                readback_status=readback.readback_status,
                claim_ceiling=node.claim_ceiling,
            )
        )
    return tuple(updated)


def _validate_readback_binding(node: GraphNode, readback: _Readback) -> None:
    if readback.lean_name != node.lean_name:
        raise TheoremGraphError(f"readback Lean name mismatch: {node.node_id}")
    if readback.statement_sha256 != node.statement_sha256:
        raise TheoremGraphError(f"readback statement hash mismatch: {node.node_id}")
    if readback.statement_text_paths != node.statement_text_paths:
        raise TheoremGraphError(f"readback statement path mismatch: {node.node_id}")


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
    unknown_routes = set(node.route_ids) - set(GRAPH_ROUTE_IDS)
    if unknown_routes:
        raise TheoremGraphError(f"unknown graph node route: {sorted(unknown_routes)[0]}")
    if tuple(sorted(node.route_ids)) != node.route_ids:
        raise TheoremGraphError(f"graph node routes are not canonical: {node.node_id}")
    if node.proof_status not in PROOF_STATUSES:
        raise TheoremGraphError(f"invalid proof status: {node.proof_status}")
    if node.readback_status not in READBACK_STATUSES:
        raise TheoremGraphError(f"invalid readback status: {node.readback_status}")
    if node.readback_status == "current" and node.natural_language_statement is None:
        raise TheoremGraphError(f"current graph node lacks readback statement: {node.node_id}")
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
    dependencies = {node.node_id: node.dependency_ids for node in nodes}
    try:
        graph_kernel.topological_order(dependencies)
    except graph_kernel.GraphKernelError as error:
        raise TheoremGraphError(str(error)) from error


def _topological_order(nodes: tuple[GraphNode, ...]) -> tuple[GraphNode, ...]:
    by_id = {node.node_id: node for node in nodes}
    dependencies = {node_id: node.dependency_ids for node_id, node in by_id.items()}
    try:
        ordered = graph_kernel.topological_order(dependencies)
    except graph_kernel.GraphKernelError as error:
        raise TheoremGraphError(str(error)) from error
    return tuple(by_id[node_id] for node_id in ordered)


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
