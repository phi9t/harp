#!/usr/bin/env python3
"""Convert managed Harp Markdown links to vault-root-qualified wiki links."""

from __future__ import annotations

import argparse
import json
import re
import stat
import sys
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import unquote


MANAGED_ROOT = Path("knowledge")
LOCAL_INVESTIGATION_ROOT = MANAGED_ROOT / "investigations" / "local"
CAPTURED_ARTIFACT_ROOT = Path("evidence") / "artifacts"
MARKDOWN_SUFFIXES = {".md", ".markdown"}
EXTERNAL_SCHEMES = ("http://", "https://", "mailto:", "tel:")
LINK_PATTERN = re.compile(r"(?<!!)\[([^\]]+)\]\(([^()\s]+(?:\s+\"[^\"]*\")?)\)")
HEADING_PATTERN = re.compile(r"^(#{1,6})[ \t]+(.+?)[ \t]*#*[ \t]*$")
EXPLICIT_HEADING_ID_PATTERN = re.compile(r"\s+\{#([A-Za-z0-9_-]+)\}\s*$")
LINE_LOCATOR_PATTERN = re.compile(r"^L(\d+)(?:-L(\d+))?$")
DUAL_LOCATOR_PREFIX_PATTERN = re.compile(r"\[\[[^\]]+\]\]\s*\($")


class MigrationError(ValueError):
    """Raised when a managed link cannot be migrated safely."""


@dataclass(frozen=True)
class LinkSpan:
    start: int
    end: int
    label: str
    destination: str
    in_code: bool


@dataclass(frozen=True)
class ResolvedTarget:
    path: Path
    repository_path: Path
    fragment: str | None
    is_note: bool


@dataclass(frozen=True)
class MigrationResult:
    text: str
    converted_count: int
    retained_line_locator_count: int
    remaining_managed_markdown_link_count: int


def discover_managed_notes(repo_root: Path) -> list[Path]:
    """Return canonical managed Markdown notes in deterministic repository order."""
    knowledge_root = repo_root / MANAGED_ROOT
    if not knowledge_root.is_dir():
        return []

    return [
        path
        for path in sorted(knowledge_root.rglob("*"))
        if path.is_file()
        and path.suffix.lower() in MARKDOWN_SUFFIXES
        and not is_relative_to(path, repo_root / LOCAL_INVESTIGATION_ROOT)
    ]


def scan_markdown_links(text: str) -> list[LinkSpan]:
    """Find Markdown links outside fenced code, inline code, and wiki links."""
    spans: list[LinkSpan] = []
    fence_marker: str | None = None
    byte_offset = 0
    scan_start = 0
    scan_chunk = ""

    for line in text.splitlines(keepends=True):
        stripped = line.lstrip()
        marker = fence_marker_for(stripped)
        if marker is not None:
            if fence_marker is None:
                spans.extend(scan_link_chunk(scan_chunk, scan_start))
                scan_chunk = ""
                fence_marker = marker
            elif marker[0] == fence_marker[0] and len(marker) >= len(fence_marker):
                fence_marker = None
                scan_start = byte_offset + len(line.encode("utf-8"))
            byte_offset += len(line.encode("utf-8"))
            continue

        if fence_marker is None:
            scan_chunk += line
        byte_offset += len(line.encode("utf-8"))
    spans.extend(scan_link_chunk(scan_chunk, scan_start))
    return spans


def scan_link_chunk(text: str, byte_offset: int) -> list[LinkSpan]:
    spans: list[LinkSpan] = []
    for match in LINK_PATTERN.finditer(text):
        prefix = text[: match.start()]
        if not inside_inline_code(prefix) and not inside_wiki_link(prefix):
            spans.append(
                LinkSpan(
                    start=byte_offset + len(prefix.encode("utf-8")),
                    end=byte_offset + len(text[: match.end()].encode("utf-8")),
                    label=" ".join(match.group(1).split()),
                    destination=match.group(2),
                    in_code=False,
                )
            )
    return spans


def resolve_markdown_target(
    source: Path, destination: str, repo_root: Path
) -> ResolvedTarget | None:
    """Resolve a local Markdown destination without permitting root escapes."""
    raw_destination = destination.split(" ", 1)[0]
    if raw_destination.startswith(EXTERNAL_SCHEMES) or raw_destination.startswith("//"):
        return None
    if raw_destination.startswith("/"):
        raise MigrationError(f"{source}: absolute local link is not allowed: {destination}")

    path_part, separator, fragment = raw_destination.partition("#")
    source_path = source.resolve()
    root_path = repo_root.resolve()
    target_path = source_path if not path_part else (source_path.parent / unquote(path_part))
    target_path = target_path.resolve()

    if not is_relative_to(target_path, root_path):
        raise MigrationError(f"{source}: link escapes repository: {destination}")
    if target_path.is_dir():
        readme = target_path / "README.md"
        provenance = target_path / "PROVENANCE.md"
        if is_regular_non_symlink_file(readme):
            target_path = readme
        elif is_relative_to(target_path, root_path / "evidence") and is_regular_non_symlink_file(
            provenance
        ):
            target_path = provenance
        else:
            raise MigrationError(f"{source}: missing local link target: {destination}")
    if not target_path.is_file():
        raise MigrationError(f"{source}: missing local link target: {destination}")

    repository_path = target_path.relative_to(root_path)
    is_note = target_path.suffix.lower() in MARKDOWN_SUFFIXES
    resolved_fragment = unquote(fragment) if separator else None
    return ResolvedTarget(
        path=target_path,
        repository_path=repository_path,
        fragment=resolved_fragment,
        is_note=is_note,
    )


def heading_for_fragment(note: Path, fragment: str) -> str:
    """Return the sole heading title matching a Markdown fragment."""
    expected_slug = slugify_heading(fragment)
    matches = [
        title
        for title, explicit_id in headings_in(note)
        if slugify_heading(title) == expected_slug
        or (explicit_id is not None and explicit_id == fragment)
    ]
    if not matches:
        raise MigrationError(f"{note}: missing heading for fragment #{fragment}")
    if len(matches) > 1:
        raise MigrationError(f"{note}: ambiguous heading for fragment #{fragment}")
    return matches[0]


def wiki_destination(
    target: ResolvedTarget, label: str, source: Path | None = None
) -> str:
    """Render a root-qualified wiki link for an already-resolved target."""
    path = target.repository_path.as_posix()
    if target.is_note:
        if target.fragment:
            heading = heading_for_fragment(target.path, target.fragment)
            if source is not None and target.path == source.resolve():
                path = f"#{heading}"
            else:
                path = f"{strip_markdown_suffix(path)}#{heading}"
        else:
            path = strip_markdown_suffix(path)
    elif target.fragment:
        path = f"{path}#{target.fragment}"
    return f"[[{path}|{label}]]"


def migrate_text(source: Path, text: str, repo_root: Path) -> str:
    """Migrate every eligible internal Markdown link in one source document."""
    return migrate_text_with_result(source, text, repo_root).text


def migrate_text_with_result(source: Path, text: str, repo_root: Path) -> MigrationResult:
    """Migrate text and return audit counters without modifying the source."""
    source = source.resolve()
    repo_root = repo_root.resolve()
    ensure_managed_source(source, repo_root)
    replacements: list[tuple[int, int, str, bool]] = []
    retained_line_locator_count = 0

    for span in scan_markdown_links(text):
        target = resolve_markdown_target(source, span.destination, repo_root)
        if target is None:
            continue

        if target.fragment and LINE_LOCATOR_PATTERN.fullmatch(target.fragment):
            lines = LINE_LOCATOR_PATTERN.fullmatch(target.fragment)
            assert lines is not None
            start_line, end_line = lines.groups()
            end_line = end_line or start_line
            if follows_native_artifact_link(text, span):
                retained_line_locator_count += 1
                continue
            wiki_target = ResolvedTarget(
                path=target.path,
                repository_path=target.repository_path,
                fragment=None,
                is_note=target.is_note,
            )
            replacement = (
                f"{wiki_destination(wiki_target, span.label, source)} "
                f"([exact lines {start_line}–{end_line}]({span.destination}))"
            )
            replacements.append((span.start, span.end, replacement, True))
            retained_line_locator_count += 1
        else:
            replacements.append(
                (span.start, span.end, wiki_destination(target, span.label, source), False)
            )

    encoded_text = text.encode("utf-8")
    for start, end, replacement, _ in reversed(replacements):
        encoded_text = encoded_text[:start] + replacement.encode("utf-8") + encoded_text[end:]
    migrated = encoded_text.decode("utf-8")
    remaining = remaining_managed_markdown_links(source, migrated, repo_root)
    return MigrationResult(
        text=migrated,
        converted_count=len(replacements),
        retained_line_locator_count=retained_line_locator_count,
        remaining_managed_markdown_link_count=remaining,
    )


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Migrate Harp managed Markdown links to Obsidian wiki links."
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="audit without writing")
    mode.add_argument("--write", action="store_true", help="write migrated notes")
    parser.add_argument("--report", type=Path, help="write deterministic JSON report")
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path.cwd(),
        help="repository root (defaults to current directory)",
    )
    parser.add_argument("paths", nargs="*", type=Path, help="managed notes to audit")
    options = parser.parse_args(arguments)
    repo_root = options.repo_root.resolve()

    try:
        sources = (
            [validate_input_path(path, repo_root) for path in options.paths]
            if options.paths
            else discover_managed_notes(repo_root)
        )
        plans = [
            (source, migrate_text_with_result(source, source.read_text(encoding="utf-8"), repo_root))
            for source in sources
        ]
    except (MigrationError, OSError, UnicodeError) as error:
        print(f"migrate_obsidian_links: {error}", file=sys.stderr)
        return 1

    if options.write:
        for source, result in plans:
            if result.text != source.read_text(encoding="utf-8"):
                source.write_text(result.text, encoding="utf-8")

    report = {
        "converted_count": sum(result.converted_count for _, result in plans),
        "input_count": len(sources),
        "remaining_managed_markdown_link_count": sum(
            result.remaining_managed_markdown_link_count for _, result in plans
        ),
        "retained_line_locator_count": sum(
            result.retained_line_locator_count for _, result in plans
        ),
        "skipped_captured_count": count_captured_markdown(repo_root),
    }
    if options.report:
        options.report.parent.mkdir(parents=True, exist_ok=True)
        options.report.write_text(
            json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
    print(json.dumps(report, sort_keys=True))

    if options.check and (
        report["converted_count"] or report["remaining_managed_markdown_link_count"]
    ):
        return 1
    return 0


def validate_input_path(path: Path, repo_root: Path) -> Path:
    candidate = path.resolve()
    ensure_managed_source(candidate, repo_root)
    if not candidate.is_file():
        raise MigrationError(f"{candidate}: input is not a regular file")
    if candidate.suffix.lower() not in MARKDOWN_SUFFIXES:
        raise MigrationError(f"{candidate}: input is not Markdown")
    return candidate


def ensure_managed_source(source: Path, repo_root: Path) -> None:
    managed_root = repo_root / MANAGED_ROOT
    if not is_relative_to(source, managed_root) or is_relative_to(
        source, repo_root / LOCAL_INVESTIGATION_ROOT
    ):
        raise MigrationError(f"{source}: input is outside managed knowledge roots")


def headings_in(note: Path) -> list[tuple[str, str | None]]:
    headings: list[tuple[str, str | None]] = []
    fenced = False
    for line in note.read_text(encoding="utf-8").splitlines():
        marker = fence_marker_for(line.lstrip())
        if marker is not None:
            fenced = not fenced
            continue
        if fenced:
            continue
        match = HEADING_PATTERN.match(line)
        if match:
            raw_title = match.group(2).strip()
            explicit_id = EXPLICIT_HEADING_ID_PATTERN.search(raw_title)
            title = (
                raw_title[: explicit_id.start()].rstrip()
                if explicit_id is not None
                else raw_title
            )
            headings.append(
                (title, explicit_id.group(1) if explicit_id is not None else None)
            )
    return headings


def remaining_managed_markdown_links(source: Path, text: str, repo_root: Path) -> int:
    remaining = 0
    for span in scan_markdown_links(text):
        target = resolve_markdown_target(source, span.destination, repo_root)
        if target is not None and not (
            target.fragment and LINE_LOCATOR_PATTERN.fullmatch(target.fragment)
        ):
            remaining += 1
    return remaining


def count_captured_markdown(repo_root: Path) -> int:
    root = repo_root / CAPTURED_ARTIFACT_ROOT
    if not root.is_dir():
        return 0
    return sum(
        1
        for path in root.rglob("*")
        if path.is_file() and path.suffix.lower() in MARKDOWN_SUFFIXES
    )


def follows_native_artifact_link(text: str, span: LinkSpan) -> bool:
    prefix = text.encode("utf-8")[: span.start].decode("utf-8")
    return DUAL_LOCATOR_PREFIX_PATTERN.search(prefix.rstrip()) is not None


def strip_markdown_suffix(path: str) -> str:
    suffix = Path(path).suffix.lower()
    return path[: -len(suffix)] if suffix in MARKDOWN_SUFFIXES else path


def slugify_heading(value: str) -> str:
    value = unquote(value).strip().lower()
    value = re.sub(r"[`*_~]", "", value)
    value = re.sub(r"[^\w\s-]", "", value)
    return re.sub(r"[\s-]+", "-", value).strip("-")


def fence_marker_for(line: str) -> str | None:
    match = re.match(r"(`{3,}|~{3,})", line)
    return match.group(1) if match else None


def inside_inline_code(prefix: str) -> bool:
    return len(re.findall(r"(?<!\\\\)`", prefix)) % 2 == 1


def inside_wiki_link(prefix: str) -> bool:
    return prefix.rfind("[[") > prefix.rfind("]]")


def is_relative_to(path: Path, parent: Path) -> bool:
    try:
        path.resolve().relative_to(parent.resolve())
    except ValueError:
        return False
    return True


def is_regular_non_symlink_file(path: Path) -> bool:
    try:
        return stat.S_ISREG(path.lstat().st_mode)
    except FileNotFoundError:
        return False


if __name__ == "__main__":
    raise SystemExit(main())
