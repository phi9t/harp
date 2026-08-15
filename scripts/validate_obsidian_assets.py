#!/usr/bin/env python3
"""Validate portable Obsidian navigation assets without Obsidian installed."""

from __future__ import annotations

import argparse
import json
import re
import stat
import sys
from pathlib import Path
from typing import Any


REQUIRED_PROFILE_FILES = {
    "app.json",
    "core-plugins.json",
    "bookmarks.json",
    "workspace.json",
    "snippets/harp-knowledge.css",
}
REQUIRED_CORE_PLUGINS = {
    "file-explorer",
    "global-search",
    "graph",
    "backlink",
    "outgoing-link",
    "properties",
    "page-preview",
    "tag-pane",
    "outline",
    "bookmarks",
    "bases",
    "canvas",
}
BASE_ALLOWED_KEYS = {"filters", "formulas", "properties", "summaries", "views"}
BASE_VIEW_TYPES = {"table", "cards", "list", "map"}
CANVAS_ID_PATTERN = re.compile(r"^[0-9a-f]{16}$")


class ValidationError(ValueError):
    """Raised when a portable Obsidian asset violates its checked-in contract."""


def is_regular_non_symlink(path: Path) -> bool:
    try:
        return stat.S_ISREG(path.lstat().st_mode)
    except FileNotFoundError:
        return False


def require_regular_file(path: Path, description: str) -> None:
    if path.is_symlink():
        raise ValidationError(f"{description} must not be a symlink: {path}")
    if not is_regular_non_symlink(path):
        raise ValidationError(f"missing {description}: {path}")


def ensure_relative_to(path: Path, root: Path, description: str) -> None:
    try:
        path.relative_to(root)
    except ValueError as error:
        raise ValidationError(f"{description} escapes repository root: {path}") from error


def validate_profile(profile_root: Path) -> None:
    if profile_root.is_symlink() or not profile_root.is_dir():
        raise ValidationError(f"missing portable profile directory: {profile_root}")

    found_files = {
        path.relative_to(profile_root).as_posix()
        for path in profile_root.rglob("*")
        if path.is_file() or path.is_symlink()
    }
    missing = sorted(REQUIRED_PROFILE_FILES - found_files)
    if missing:
        raise ValidationError(f"missing portable profile file: {missing[0]}")
    unexpected = sorted(found_files - REQUIRED_PROFILE_FILES)
    if unexpected:
        raise ValidationError(f"profile file is not allowlisted: {unexpected[0]}")

    for relative_path in sorted(REQUIRED_PROFILE_FILES):
        require_regular_file(profile_root / relative_path, "portable profile file")

    try:
        plugins = json.loads(
            (profile_root / "core-plugins.json").read_text(encoding="utf-8")
        )
    except json.JSONDecodeError as error:
        raise ValidationError(f"invalid core plugin JSON: {error}") from error
    if not isinstance(plugins, list) or not all(isinstance(plugin, str) for plugin in plugins):
        raise ValidationError("core-plugins.json must contain a JSON string list")
    missing_plugins = sorted(REQUIRED_CORE_PLUGINS - set(plugins))
    if missing_plugins:
        raise ValidationError(f"missing required core plugin: {missing_plugins[0]}")
    unexpected_plugins = sorted(set(plugins) - REQUIRED_CORE_PLUGINS)
    if unexpected_plugins:
        raise ValidationError(f"unexpected core plugin: {unexpected_plugins[0]}")


def parse_scalar(value: str) -> Any:
    value = value.strip()
    if value.startswith(("'", '"')) and value.endswith(("'", '"')) and len(value) >= 2:
        return value[1:-1]
    if value.startswith("[") and value.endswith("]"):
        contents = value[1:-1].strip()
        return [] if not contents else [item.strip() for item in contents.split(",")]
    if value in {"true", "false"}:
        return value == "true"
    return value


def parse_base_yaml(text: str) -> dict[str, Any]:
    """Parse the intentionally small Base YAML subset used by Harp."""
    result: dict[str, Any] = {}
    current_section: str | None = None
    current_view: dict[str, Any] | None = None

    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        if not raw_line.strip() or raw_line.lstrip().startswith("#"):
            continue
        indent = len(raw_line) - len(raw_line.lstrip(" "))
        stripped = raw_line.strip()
        if indent == 0:
            if ":" not in stripped:
                raise ValidationError(f"invalid Base YAML at line {line_number}")
            key, value = stripped.split(":", 1)
            if key in result:
                raise ValidationError(f"duplicate Base key: {key}")
            current_section = key
            current_view = None
            result[key] = parse_scalar(value) if value.strip() else (
                [] if key == "views" else {}
            )
            continue
        if current_section is None:
            raise ValidationError(f"indented Base content before a key at line {line_number}")
        if current_section == "views":
            if stripped.startswith("- "):
                if not isinstance(result["views"], list):
                    raise ValidationError("Base views must be a list")
                current_view = {}
                result["views"].append(current_view)
                stripped = stripped[2:].strip()
                if stripped:
                    if ":" not in stripped:
                        raise ValidationError(f"invalid Base view at line {line_number}")
                    key, value = stripped.split(":", 1)
                    current_view[key] = parse_scalar(value)
            else:
                if current_view is None or ":" not in stripped:
                    raise ValidationError(f"invalid Base view field at line {line_number}")
                key, value = stripped.split(":", 1)
                if key in current_view:
                    raise ValidationError(f"duplicate Base view field: {key}")
                current_view[key] = parse_scalar(value)
        elif current_section == "properties":
            if indent == 2 and stripped.endswith(":"):
                result["properties"][stripped[:-1]] = {}
            elif indent >= 4 and ":" in stripped:
                property_name = next(reversed(result["properties"]), None)
                if property_name is None:
                    raise ValidationError(f"Base property missing name at line {line_number}")
                key, value = stripped.split(":", 1)
                result["properties"][property_name][key] = parse_scalar(value)
            else:
                raise ValidationError(f"invalid Base property at line {line_number}")
        elif current_section in {"formulas", "summaries"}:
            if ":" not in stripped:
                raise ValidationError(f"invalid Base mapping at line {line_number}")
            key, value = stripped.split(":", 1)
            result[current_section][key] = parse_scalar(value)
        else:
            raise ValidationError(f"unexpected nested Base content at line {line_number}")
    return result


def validate_base(base_path: Path) -> None:
    require_regular_file(base_path, "Harp knowledge Base")
    base = parse_base_yaml(base_path.read_text(encoding="utf-8"))
    unknown = sorted(set(base) - BASE_ALLOWED_KEYS)
    if unknown:
        raise ValidationError(f"unsupported Base top-level key: {unknown[0]}")
    if not isinstance(base.get("views"), list) or not base["views"]:
        raise ValidationError("Base must contain at least one view")
    for view in base["views"]:
        if not isinstance(view, dict):
            raise ValidationError("Base view must be a mapping")
        view_type = view.get("type")
        if view_type not in BASE_VIEW_TYPES:
            raise ValidationError(f"unsupported Base view type: {view_type!r}")


def validate_canvas(canvas_path: Path, repository_root: Path) -> None:
    require_regular_file(canvas_path, "Harp knowledge Canvas")
    try:
        canvas = json.loads(canvas_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise ValidationError(f"invalid Canvas JSON: {error}") from error
    if not isinstance(canvas, dict):
        raise ValidationError("Canvas must be a JSON object")
    nodes = canvas.get("nodes")
    edges = canvas.get("edges")
    if not isinstance(nodes, list) or not isinstance(edges, list):
        raise ValidationError("Canvas must contain nodes and edges arrays")

    identifiers: set[str] = set()
    node_identifiers: set[str] = set()
    for node in nodes:
        if not isinstance(node, dict):
            raise ValidationError("Canvas node must be an object")
        identifier = node.get("id")
        if not isinstance(identifier, str) or not CANVAS_ID_PATTERN.fullmatch(identifier):
            raise ValidationError(f"invalid Canvas node ID: {identifier!r}")
        if identifier in identifiers:
            raise ValidationError(f"duplicate Canvas ID: {identifier}")
        identifiers.add(identifier)
        node_identifiers.add(identifier)

    for edge in edges:
        if not isinstance(edge, dict):
            raise ValidationError("Canvas edge must be an object")
        identifier = edge.get("id")
        if not isinstance(identifier, str) or not CANVAS_ID_PATTERN.fullmatch(identifier):
            raise ValidationError(f"invalid Canvas edge ID: {identifier!r}")
        if identifier in identifiers:
            raise ValidationError(f"duplicate Canvas ID: {identifier}")
        identifiers.add(identifier)

    for node in nodes:
        if node.get("type") == "file":
            identifier = node["id"]
            file_name = node.get("file")
            if not isinstance(file_name, str) or not file_name:
                raise ValidationError(f"Canvas file node {identifier} has no file path")
            target = (repository_root / file_name).resolve(strict=False)
            ensure_relative_to(target, repository_root, "Canvas file node")
            if target.is_symlink() or not is_regular_non_symlink(target):
                raise ValidationError(f"Canvas file node target is missing or unsafe: {file_name}")

    for edge in edges:
        identifier = edge.get("id")
        for endpoint in ("fromNode", "toNode"):
            node_id = edge.get(endpoint)
            if node_id not in node_identifiers:
                raise ValidationError(
                    f"Canvas edge {identifier} has unknown {endpoint}: {node_id!r}"
                )
        label = edge.get("label")
        if label is not None and label not in {"read next", "audit through"}:
            raise ValidationError(f"unsupported Canvas edge label: {label!r}")


def validate_repository_assets(repository_root: Path) -> None:
    root = repository_root.resolve()
    if repository_root.is_symlink() or not root.is_dir():
        raise ValidationError(f"repository root must be a real directory: {repository_root}")
    validate_profile(root / "tools" / "obsidian" / "profile")
    navigation_root = root / "knowledge" / "obsidian"
    validate_base(navigation_root / "harp_knowledge.base")
    validate_canvas(navigation_root / "harp_knowledge_map.canvas", root)


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="repository root (defaults to this script's repository)",
    )
    options = parser.parse_args(arguments)
    try:
        validate_repository_assets(options.repo_root)
    except (OSError, UnicodeError, ValidationError) as error:
        print(f"Obsidian asset validation failed: {error}", file=sys.stderr)
        return 1
    print("Obsidian portable assets are valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
