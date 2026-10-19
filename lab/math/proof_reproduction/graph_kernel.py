"""Deterministic algorithms for dependency graphs."""

from __future__ import annotations

from typing import Iterable, Mapping


class GraphKernelError(ValueError):
    """The dependency graph is not closed or acyclic."""


def topological_order(
    dependencies: Mapping[str, Iterable[str]],
    *,
    external_ids: Iterable[str] = (),
    cycle_label: str = "graph",
) -> tuple[str, ...]:
    """Return internal node ids in deterministic dependency-first order."""

    normalized = {node_id: tuple(sorted(items)) for node_id, items in dependencies.items()}
    external = set(external_ids)
    internal = set(normalized)
    ordered: list[str] = []
    temporary: set[str] = set()
    permanent: set[str] = set()

    def visit(node_id: str) -> None:
        if node_id in external:
            return
        if node_id not in internal:
            raise GraphKernelError(f"unknown dependency: {node_id}")
        if node_id in permanent:
            return
        if node_id in temporary:
            raise GraphKernelError(f"{cycle_label} cycle detected at {node_id}")
        temporary.add(node_id)
        for dependency in normalized[node_id]:
            visit(dependency)
        temporary.remove(node_id)
        permanent.add(node_id)
        ordered.append(node_id)

    for node_id in sorted(internal):
        visit(node_id)
    return tuple(ordered)


def dependency_closure(
    roots: Iterable[str],
    dependencies: Mapping[str, Iterable[str]],
    *,
    excluded: Iterable[str] = (),
    external_ids: Iterable[str] = (),
) -> tuple[str, ...]:
    """Return the dependency-first subgraph reachable from ``roots``."""

    normalized = {node_id: tuple(items) for node_id, items in dependencies.items()}
    external = set(external_ids)
    root_ids = tuple(roots)
    for root in root_ids:
        if root not in normalized and root not in external:
            raise GraphKernelError(f"unknown root: {root}")

    topological_order(normalized, external_ids=external)
    omitted = set(excluded)
    ordered: list[str] = []
    visited: set[str] = set()

    def collect(node_id: str) -> None:
        if node_id in visited or node_id in external:
            return
        visited.add(node_id)
        for dependency in sorted(normalized[node_id]):
            collect(dependency)
        if node_id not in omitted:
            ordered.append(node_id)

    for root in root_ids:
        collect(root)

    return tuple(ordered)
