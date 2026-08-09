from __future__ import annotations

import argparse
import csv
import html
import json
import os
import re
import stat
import tempfile
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "content/weng-source-cards.tsv"
STATUS = ROOT / "content/weng-course-status.json"
CARD_ROOT = ROOT / "content/weng-sources"
REFERENCE_ROOT = ROOT / "reference"
EVIDENCE_GRAPH = ROOT / "content/sources/evidence_graph.tsv"

FIELDS = (
    "source_id",
    "weng_locator",
    "title",
    "section_id",
    "card_path",
    "primary_url",
    "captured_path",
    "evidence_state",
    "claim_ceiling",
    "lesson_ids",
    "retrieval_state",
)
HEADINGS = (
    "Problem",
    "Core mechanism",
    "Reported evidence",
    "Key limitation",
    "Why Weng cites it",
)
SECTION_ORDER = (
    "system-being-improved",
    "harness-design-patterns",
    "harness-layer-vs-core-intelligence",
    "context-engineering",
    "workflow-design-and-search",
    "self-improving-harnesses",
    "evolutionary-search",
    "joint-harness-weight-optimization",
    "future-challenges",
)
SECTION_META = (
    (
        "system-being-improved",
        "What system is being improved?",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#case-study-coding-agent-harness",
        "../content/weng/01-system-being-improved.md",
        "../lessons/0001-system-being-improved.html",
    ),
    (
        "harness-design-patterns",
        "Harness design patterns",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#harness-design-patterns",
        "../content/weng/02-harness-design-patterns.md",
        "../lessons/0002-harness-design-patterns.html",
    ),
    (
        "harness-layer-vs-core-intelligence",
        "Harness layer versus core intelligence",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#harness-layer-vs-core-intelligence",
        "../content/weng/03-harness-layer-vs-core-intelligence.md",
        "../lessons/0003-harness-vs-core-intelligence.html",
    ),
    (
        "context-engineering",
        "Context engineering",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#context-engineering",
        "../content/weng/04-context-engineering.md",
        "../lessons/0004-context-engineering.html",
    ),
    (
        "workflow-design-and-search",
        "Workflow design and search",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#workflow-design",
        "../content/weng/05-workflow-design-and-search.md",
        "../lessons/0005-workflow-design-and-auto-research.html",
    ),
    (
        "self-improving-harnesses",
        "Self-improving harnesses",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#self-improving-harness",
        "../content/weng/06-self-improving-harnesses.md",
        "../lessons/0006-self-improving-harnesses.html",
    ),
    (
        "evolutionary-search",
        "Evolutionary search",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#evolutionary-search",
        "../content/weng/07-evolutionary-search.md",
        "../lessons/0007-evolutionary-search.html",
    ),
    (
        "joint-harness-weight-optimization",
        "Joint harness and weight optimization",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#joint-optimization-with-model-weights",
        "../content/weng/08-joint-harness-weight-optimization.md",
        "../lessons/0008-joint-harness-weight-optimization.html",
    ),
    (
        "future-challenges",
        "Future challenges",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#future-challenges",
        "../content/weng/09-future-challenges.md",
        "../lessons/0009-future-challenges-and-evaluation.html",
    ),
)
CARD_METADATA = (
    "source_id",
    "title",
    "weng_locator",
    "section_id",
    "primary_url",
    "captured_path",
    "publication_state",
    "evidence_state",
    "edited_object_family",
    "claim_ceiling",
    "lesson_ids",
    "card_path",
    "canonical_route",
)
ALLOWED_STATES = {"building", "cards-complete", "complete"}
BODY_LINK_LOCATORS = {
    "KARPATHY-AUTORESEARCH": "body-link-workflow-automation",
    "WENG-REWARD": "body-link-reward-hacking",
    "ANTHROPIC-RSI": "body-link-ai-progress",
}


@dataclass(frozen=True)
class Card:
    metadata: dict[str, str]
    sections: dict[str, str]


def parse_frontmatter(text: str) -> tuple[dict[str, str], str]:
    if not text.startswith("---\n"):
        raise ValueError("card must start with frontmatter")
    try:
        frontmatter, body = text[4:].split("\n---\n", 1)
    except ValueError as error:
        raise ValueError("card must close frontmatter") from error
    metadata: dict[str, str] = {}
    for line in frontmatter.splitlines():
        if not line or line.startswith((" ", "\t")) or ":" not in line:
            raise ValueError("frontmatter must contain top-level key/value scalars")
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if not key or not value:
            raise ValueError("frontmatter keys and values must be nonempty")
        if key in metadata:
            raise ValueError(f"duplicate frontmatter key: {key}")
        metadata[key] = value
    if tuple(metadata) != CARD_METADATA:
        raise ValueError(f"unexpected card metadata: {tuple(metadata)}")
    return metadata, body


def parse_sections(body: str, expected_title: str) -> dict[str, str]:
    if not body.startswith("\n") or body.startswith("\n\n"):
        raise ValueError("card frontmatter must be followed by exactly one blank separator")
    body = body[1:]
    lines_iter = iter(body.splitlines())
    try:
        title_line = next(lines_iter)
    except StopIteration as error:
        raise ValueError("card body must start with its H1 title") from error
    if not title_line.startswith("# ") or not title_line[2:].strip():
        raise ValueError(f"card must have a nonempty H1 for: {expected_title}")

    remaining = list(lines_iter)
    blank_count = 0
    while blank_count < len(remaining) and not remaining[blank_count].strip():
        blank_count += 1
    if blank_count == 0:
        raise ValueError("card H1 must be followed by a blank line")
    lines = remaining[blank_count:]
    if not lines or lines[0] != "## Problem":
        raise ValueError("card H1 must be followed by ## Problem")

    sections: dict[str, str] = {}
    current: str | None = None
    section_lines: list[str] = []
    for line in lines:
        if line.lstrip().startswith("# "):
            raise ValueError("card body must contain exactly one H1 title")
        if line.startswith("## "):
            if current is not None:
                if current in sections:
                    raise ValueError(f"duplicate card section: {current}")
                sections[current] = _plain_section(current, section_lines)
            current = line[3:].strip()
            section_lines = []
        elif current is not None:
            section_lines.append(line)
        elif line.strip():
            raise ValueError("card body must start with the first required heading")
    if current is not None:
        if current in sections:
            raise ValueError(f"duplicate card section: {current}")
        sections[current] = _plain_section(current, section_lines)
    if tuple(sections) != HEADINGS:
        raise ValueError(f"unexpected card sections: {tuple(sections)}")
    return sections


def _plain_section(heading: str, lines: list[str]) -> str:
    text = "\n".join(lines).strip()
    if not text:
        raise ValueError(f"empty card section: {heading}")
    if (
        "`" in text
        or "](" in text
        or "![" in text
        or re.search(r"</?[A-Za-z][^>\n]*>|<![^>\n]*>|<\?[^>\n]*\?>", text)
        or re.search(r"(?m)^[ \t]*(?:[-*+>] |\d+\. )", text)
    ):
        raise ValueError(f"card section must contain plain paragraphs: {heading}")
    return text


def _reference_locator(evidence_locator: str) -> str:
    match = re.fullmatch(r"Reference ([1-9]\d?)(?:\b.*)?", evidence_locator)
    if match is None:
        raise ValueError(f"unknown Weng reference locator: {evidence_locator}")
    number = int(match.group(1))
    if number not in range(1, 40):
        raise ValueError(f"Weng reference number is out of range: {number}")
    return f"reference-{number}"


def _expected_locators_from_rows(rows: list[dict[str, str]]) -> dict[str, str]:
    expected: dict[str, str] = {}
    locator_sources: dict[str, str] = {}
    body_sources: set[str] = set()
    for row in rows:
        if row["source_id"] != "WENG-HARNESS":
            continue
        relationship = row["relationship"]
        if relationship == "cites":
            locator = _reference_locator(row["evidence_locator"])
        elif relationship == "body-links":
            source_id = row["target_id"]
            try:
                locator = BODY_LINK_LOCATORS[source_id]
            except KeyError as error:
                raise ValueError(f"unknown Weng body-link source: {source_id}") from error
            body_sources.add(source_id)
        else:
            raise ValueError(f"unknown Weng evidence relationship: {relationship}")

        source_id = row["target_id"]
        if source_id in expected:
            raise ValueError(f"duplicate Weng source edge: {source_id}")
        if locator in locator_sources:
            raise ValueError(
                f"duplicate Weng locator {locator}: "
                f"{locator_sources[locator]} and {source_id}"
            )
        expected[source_id] = locator
        locator_sources[locator] = source_id

    numbered = {
        locator
        for locator in expected.values()
        if locator.startswith("reference-")
    }
    required_numbered = {f"reference-{number}" for number in range(1, 40)}
    if numbered != required_numbered:
        raise ValueError("Weng numbered-reference locator set is incomplete")
    if body_sources != set(BODY_LINK_LOCATORS):
        raise ValueError("Weng body-link locator set is incomplete")
    if len(expected) != 42:
        raise ValueError(f"expected 42 Weng source locators, found {len(expected)}")
    return expected


def load_expected_locators() -> dict[str, str]:
    with EVIDENCE_GRAPH.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        required = {"source_id", "relationship", "target_id", "evidence_locator"}
        if not required.issubset(reader.fieldnames or ()):
            raise ValueError("unexpected evidence graph header")
        rows = list(reader)
    return _expected_locators_from_rows(rows)


def validate_matrix_locators(
    rows: list[dict[str, str]], expected: dict[str, str]
) -> None:
    seen_locators: set[str] = set()
    present_sources: set[str] = set()
    for row in rows:
        source_id = row["source_id"]
        if source_id not in expected:
            raise ValueError(f"unexpected Weng source ID: {source_id}")
        locator = row["weng_locator"]
        if locator != expected[source_id]:
            raise ValueError(
                f"Weng locator mismatch for {source_id}: "
                f"expected {expected[source_id]}, found {locator}"
            )
        if locator in seen_locators:
            raise ValueError(f"duplicate matrix Weng locator: {locator}")
        seen_locators.add(locator)
        present_sources.add(source_id)
    if len(rows) == len(expected) and present_sources != set(expected):
        raise ValueError("complete matrix does not match the Weng source roster")


def _existing_file_beneath(relative: str, base: Path, field: str) -> Path:
    relative_path = Path(relative)
    if relative_path.is_absolute() or ".." in relative_path.parts:
        raise ValueError(f"{field} must be a repository-relative path")
    candidate = ROOT.joinpath(relative_path)
    try:
        candidate.relative_to(base)
    except ValueError as error:
        raise ValueError(f"{field} must stay beneath {base.relative_to(ROOT)}") from error

    current = ROOT
    for part in relative_path.parts:
        current /= part
        if current.is_symlink():
            raise ValueError(f"{field} must not traverse a symlink: {relative}")
    if not candidate.is_file():
        raise ValueError(f"{field} does not resolve to a file: {relative}")

    resolved_root = ROOT.resolve(strict=True)
    resolved_base = base.resolve(strict=True)
    resolved = candidate.resolve(strict=True)
    if (
        not resolved_base.is_relative_to(resolved_root)
        or not resolved.is_relative_to(resolved_base)
    ):
        raise ValueError(f"{field} escapes its approved root: {relative}")
    return resolved


def load_cards() -> list[Card]:
    with MATRIX.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != FIELDS:
            raise ValueError("unexpected matrix header")
        rows = list(reader)
    if any(None in row or None in row.values() for row in rows):
        raise ValueError("malformed matrix row")
    if len({row["source_id"] for row in rows}) != len(rows):
        raise ValueError("duplicate source ID")
    if any(not all(row[field].strip() for field in FIELDS) for row in rows):
        raise ValueError("matrix row contains an empty field")
    if any(
        not re.fullmatch(r"\d{4}(,\d{4})*", row["lesson_ids"])
        for row in rows
    ):
        raise ValueError("invalid lesson_ids")
    expected_locators = load_expected_locators()
    validate_matrix_locators(rows, expected_locators)

    cards = []
    for row in rows:
        card_path = _existing_file_beneath(row["card_path"], CARD_ROOT, "card_path")
        metadata, body = parse_frontmatter(card_path.read_text(encoding="utf-8"))
        if metadata["source_id"] != row["source_id"]:
            raise ValueError(f"source ID mismatch for {card_path}")
        if metadata["card_path"] != row["card_path"]:
            raise ValueError(f"card path mismatch for {card_path}")
        for field in (
            "title",
            "weng_locator",
            "section_id",
            "primary_url",
            "captured_path",
            "evidence_state",
            "claim_ceiling",
            "lesson_ids",
        ):
            if metadata[field] != row[field]:
                raise ValueError(f"{field} mismatch for {card_path}")
        if metadata["section_id"] not in SECTION_ORDER:
            raise ValueError(f"unknown Weng section for {card_path}")
        if not metadata["primary_url"].startswith(("https://", "http://")):
            raise ValueError(f"invalid primary URL for {card_path}")
        _existing_file_beneath(
            metadata["captured_path"], ROOT / "evidence", "captured_path"
        )
        if metadata["canonical_route"] == metadata["card_path"]:
            raise ValueError(f"canonical route must differ from card path for {card_path}")
        _existing_file_beneath(
            metadata["canonical_route"], ROOT / "content", "canonical_route"
        )
        cards.append(Card(metadata, parse_sections(body, metadata["title"])))

    locator_order = {
        **{f"reference-{index}": index for index in range(1, 40)},
        "body-link-workflow-automation": 40,
        "body-link-reward-hacking": 41,
        "body-link-ai-progress": 42,
    }
    try:
        return sorted(
            cards,
            key=lambda card: (
                locator_order[card.metadata["weng_locator"]],
                card.metadata["source_id"],
            ),
        )
    except KeyError as error:
        raise ValueError(f"unknown Weng locator: {error.args[0]}") from error


def load_status() -> str:
    data = json.loads(STATUS.read_text(encoding="utf-8"))
    if data != {
        "schema_version": 1,
        "state": data.get("state"),
        "expected_source_cards": 42,
        "expected_lessons": 10,
        "expected_references": 4,
    }:
        raise ValueError("unexpected Weng course status contract")
    state = data["state"]
    if state not in ALLOWED_STATES:
        raise ValueError(f"unknown Weng course state: {state}")
    return state


def paragraphs(text: str) -> str:
    blocks = [block.strip() for block in text.split("\n\n") if block.strip()]
    return "".join(f"<p>{html.escape(block)}</p>" for block in blocks)


def render_card(card: Card) -> str:
    metadata = card.metadata
    source_id = metadata["source_id"]
    fields = "".join(
        (
            f'<section class="card-field" aria-labelledby="{source_id}-{heading.lower().replace(" ", "-")}">'
            f'<h3 id="{source_id}-{heading.lower().replace(" ", "-")}">{html.escape(heading)}</h3>'
            f"{paragraphs(card.sections[heading])}</section>"
        )
        for heading in HEADINGS
    )
    return (
        f'<article id="source-{html.escape(source_id)}" data-source-card '
        f'data-source-id="{html.escape(source_id)}" '
        f'data-section="{html.escape(metadata["section_id"])}" '
        f'data-family="{html.escape(metadata["edited_object_family"])}">'
        f'<p class="source-id">{html.escape(source_id)}</p>'
        f'<h2>{html.escape(metadata["title"])}</h2>'
        f"{fields}"
        f'<p><a href="{html.escape(metadata["primary_url"])}">Primary source</a></p>'
        f'<p><a href="../{html.escape(metadata["canonical_route"])}">Canonical Harp route</a></p>'
        "</article>"
    )


def render_source_cards(cards: list[Card]) -> str:
    card_html = "".join(render_card(card) for card in cards)
    return (
        '<!doctype html><html lang="en"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        "<title>Weng Source Cards</title>"
        '<link rel="stylesheet" href="../assets/course.css"></head>'
        "<body><main><h1>Weng Source Cards</h1>"
        '<form aria-label="Filter source cards"><label for="source-query">Filter</label>'
        '<input id="source-query" data-source-query type="search"></form>'
        f'<p data-result-count aria-live="polite">{len(cards)} sources</p>'
        f"<div data-card-list>{card_html}</div>"
        '</main><script src="../assets/coverage.js"></script></body></html>\n'
    )


def render_harness_map(cards: list[Card]) -> str:
    section_counts: dict[str, int] = {}
    for card in cards:
        section = card.metadata["section_id"]
        section_counts[section] = section_counts.get(section, 0) + 1
    sections = "".join(
        f'<li data-section="{html.escape(section)}">'
        f"<strong>{html.escape(title)}</strong>: {count} sources · "
        f'<a href="{html.escape(original)}">Original</a> · '
        f'<a href="{html.escape(companion)}">Companion</a> · '
        f'<a href="{html.escape(lesson)}">Lesson</a></li>'
        for section, title, original, companion, lesson in SECTION_META
        for count in (section_counts.get(section, 0),)
    )
    return (
        '<!doctype html><html lang="en"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        "<title>Weng Harness Map</title>"
        '<link rel="stylesheet" href="../assets/course.css"></head>'
        "<body><main><h1>Weng Harness Map</h1>"
        "<p>prompts → structured context → workflow → harness code → optimizer code</p>"
        f"<ol>{sections}</ol></main></body></html>\n"
    )


def _safe_output_parent(
    directory: Path,
    *,
    create: bool,
    root: Path,
    reference_root: Path,
) -> Path | None:
    try:
        reference_relative = reference_root.relative_to(root)
        directory_relative = directory.relative_to(root)
        directory.relative_to(reference_root)
    except ValueError as error:
        raise ValueError("generated teaching target escapes reference/") from error
    if (
        ".." in reference_relative.parts
        or ".." in directory_relative.parts
        or directory == root
    ):
        raise ValueError("generated teaching target has an invalid parent")

    root_stat = os.lstat(root)
    if stat.S_ISLNK(root_stat.st_mode) or not stat.S_ISDIR(root_stat.st_mode):
        raise ValueError("repository root must be a real directory")
    current = root
    for part in directory_relative.parts:
        current /= part
        try:
            metadata = os.lstat(current)
        except FileNotFoundError:
            if not create:
                return None
            current.mkdir()
            metadata = os.lstat(current)
        if stat.S_ISLNK(metadata.st_mode):
            raise ValueError(f"generated output parent is a symlink: {current}")
        if not stat.S_ISDIR(metadata.st_mode):
            raise ValueError(f"generated output parent is not a directory: {current}")

    resolved_root = root.resolve(strict=True)
    resolved_reference = reference_root.resolve(strict=True)
    resolved_directory = directory.resolve(strict=True)
    if (
        not resolved_reference.is_relative_to(resolved_root)
        or not resolved_directory.is_relative_to(resolved_reference)
    ):
        raise ValueError("generated teaching target escapes the repository")
    return resolved_directory


def _preflight_output_target(
    path: Path,
    *,
    create_parent: bool,
    root: Path = ROOT,
    reference_root: Path = REFERENCE_ROOT,
) -> bool:
    if path == reference_root or ".." in path.parts:
        raise ValueError("generated teaching target has an invalid path")
    parent = _safe_output_parent(
        path.parent,
        create=create_parent,
        root=root,
        reference_root=reference_root,
    )
    if parent is None:
        return False
    try:
        metadata = os.lstat(path)
    except FileNotFoundError:
        return False
    if stat.S_ISLNK(metadata.st_mode):
        raise ValueError(f"generated teaching target is a symlink: {path}")
    if not stat.S_ISREG(metadata.st_mode):
        raise ValueError(f"generated teaching target is not a regular file: {path}")
    return True


def _output_bytes(content: str) -> bytes:
    return (content.rstrip("\n") + "\n").encode("utf-8")


def _backup_output(path: Path) -> Path:
    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    source_descriptor = os.open(path, flags)
    backup: Path | None = None
    try:
        metadata = os.fstat(source_descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            raise ValueError(f"generated teaching target is not a regular file: {path}")
        with tempfile.NamedTemporaryFile(
            mode="wb",
            delete=False,
            dir=path.parent,
            prefix=f".{path.name}.",
            suffix=".bak",
        ) as handle:
            backup = Path(handle.name)
            while chunk := os.read(source_descriptor, 1024 * 1024):
                handle.write(chunk)
            handle.flush()
            os.fsync(handle.fileno())
        return backup
    except BaseException:
        if backup is not None:
            try:
                backup.unlink()
            except FileNotFoundError:
                pass
        raise
    finally:
        os.close(source_descriptor)


def _fsync_directory(directory: Path) -> None:
    flags = os.O_RDONLY
    if hasattr(os, "O_DIRECTORY"):
        flags |= os.O_DIRECTORY
    try:
        descriptor = os.open(directory, flags)
    except OSError:
        return
    try:
        try:
            os.fsync(descriptor)
        except OSError:
            pass
    finally:
        os.close(descriptor)


def publish_outputs(
    outputs: list[tuple[Path, str]],
    check: bool,
    *,
    root: Path = ROOT,
    reference_root: Path = REFERENCE_ROOT,
) -> None:
    """Publish a generation with in-process rollback.

    Every new output and existing-target backup is staged and synced before the
    first replacement. In-process replacement failures roll back to the prior
    generation. OS or process death between renames cannot be made cross-file
    atomic on portable POSIX; a later ``--check`` detects any mixed generation.
    """

    prepared = [(path, _output_bytes(content)) for path, content in outputs]
    if len({path for path, _ in prepared}) != len(prepared):
        raise ValueError("duplicate generated teaching target")
    if check:
        for path, encoded in prepared:
            exists = _preflight_output_target(
                path,
                create_parent=False,
                root=root,
                reference_root=reference_root,
            )
            if not exists or path.read_bytes() != encoded:
                raise SystemExit(
                    f"generated teaching reference is stale: {path.relative_to(root)}"
                )
        return

    existed: dict[Path, bool] = {}
    for path, _ in prepared:
        existed[path] = _preflight_output_target(
            path,
            create_parent=True,
            root=root,
            reference_root=reference_root,
        )

    staged: list[tuple[Path, Path]] = []
    backups: dict[Path, Path | None] = {}
    replaced: list[Path] = []
    try:
        for path, encoded in prepared:
            with tempfile.NamedTemporaryFile(
                mode="wb",
                delete=False,
                dir=path.parent,
                prefix=f".{path.name}.",
                suffix=".tmp",
            ) as handle:
                staged.append((Path(handle.name), path))
                handle.write(encoded)
                handle.flush()
                os.fsync(handle.fileno())
        for path, _ in prepared:
            current_exists = _preflight_output_target(
                path,
                create_parent=False,
                root=root,
                reference_root=reference_root,
            )
            if current_exists != existed[path]:
                raise ValueError(f"generated teaching target changed during publication: {path}")
            backups[path] = _backup_output(path) if current_exists else None
        for path, _ in prepared:
            current_exists = _preflight_output_target(
                path,
                create_parent=False,
                root=root,
                reference_root=reference_root,
            )
            if current_exists != existed[path]:
                raise ValueError(f"generated teaching target changed during publication: {path}")
        for temporary, path in staged:
            try:
                os.replace(temporary, path)
            except BaseException as publication_error:
                rollback_error: BaseException | None = None
                for replaced_path in reversed(replaced):
                    try:
                        backup = backups[replaced_path]
                        if backup is None:
                            replaced_path.unlink()
                        else:
                            os.replace(backup, replaced_path)
                    except BaseException as error:
                        rollback_error = error
                for directory in {path.parent for path in replaced}:
                    _fsync_directory(directory)
                if rollback_error is not None:
                    raise RuntimeError(
                        "generated teaching rollback failed"
                    ) from publication_error
                raise
            replaced.append(path)
        for backup in backups.values():
            if backup is not None:
                backup.unlink()
        for directory in {path.parent for _, path in staged}:
            _fsync_directory(directory)
    finally:
        for temporary, _ in staged:
            try:
                temporary.unlink()
            except FileNotFoundError:
                pass
        for backup in backups.values():
            if backup is not None:
                try:
                    backup.unlink()
                except FileNotFoundError:
                    pass


def write_or_check(path: Path, content: str, check: bool) -> None:
    publish_outputs([(path, content)], check)


def run_self_tests() -> None:
    if (
        _reference_locator("Reference 38 and Joint Optimization with Model Weights")
        != "reference-38"
    ):
        raise AssertionError("compound reference locator parsing failed")
    expected_locators = {
        "SOURCE-A": "reference-1",
        "SOURCE-B": "reference-2",
    }
    for invalid_rows in (
        [
            {"source_id": "SOURCE-A", "weng_locator": "reference-2"},
            {"source_id": "SOURCE-B", "weng_locator": "reference-1"},
        ],
        [
            {"source_id": "SOURCE-A", "weng_locator": "reference-1"},
            {"source_id": "SOURCE-B", "weng_locator": "reference-1"},
        ],
    ):
        try:
            validate_matrix_locators(invalid_rows, expected_locators)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid matrix source-locator mapping unexpectedly passed")

    plain_sections = "\n".join(
        f"## {heading}\nx < y > z. See https://example.test." for heading in HEADINGS
    )
    ace_title = (
        "Agentic Context Engineering: Evolving Contexts for "
        "Self-Improving Language Models"
    )
    plain_body = f"\n# Agentic Context Engineering\n\n{plain_sections}"
    parse_sections(plain_body, ace_title)
    for invalid_body in (
        f"\n{plain_sections}",
        f"\n# \n\n{plain_sections}",
        f"\n# Agentic Context Engineering\n\n# Duplicate\n\n{plain_sections}",
        f"\n# Agentic Context Engineering\n\nExtra prose.\n\n{plain_sections}",
        f"\n# Agentic Context Engineering\n\n### Extra heading\n\n{plain_sections}",
        f"\n# Agentic Context Engineering\n\n{plain_sections}\n\n# Later title",
        f"# Agentic Context Engineering\n\n{plain_sections}",
        f"\n\n# Agentic Context Engineering\n\n{plain_sections}",
    ):
        try:
            parse_sections(invalid_body, ace_title)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid card H1 contract unexpectedly passed")
    try:
        _expected_locators_from_rows(
            [
                {
                    "source_id": "WENG-HARNESS",
                    "relationship": "unknown",
                    "target_id": "SOURCE",
                    "evidence_locator": "Unknown",
                }
            ]
        )
    except ValueError as error:
        if "unknown Weng evidence relationship" not in str(error):
            raise
    else:
        raise AssertionError("unknown Weng evidence relationship unexpectedly passed")

    with tempfile.TemporaryDirectory() as raw:
        root = Path(raw)
        reference_root = root / "reference"
        try:
            publish_outputs(
                [(reference_root / "missing.html", "missing")],
                True,
                root=root,
                reference_root=reference_root,
            )
        except SystemExit:
            pass
        else:
            raise AssertionError("stale output check unexpectedly passed")
        if reference_root.exists():
            raise AssertionError("read-only output check created reference/")

    with tempfile.TemporaryDirectory() as raw:
        root = Path(raw)
        reference_root = root / "reference"
        nested_target = reference_root / "nested/first.html"
        publish_outputs(
            [(nested_target, "first build")],
            False,
            root=root,
            reference_root=reference_root,
        )
        if nested_target.read_bytes() != b"first build\n":
            raise AssertionError("first-build publication failed")

    if hasattr(os, "symlink"):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            real = root / "real"
            real.mkdir()
            reference_root = root / "reference"
            reference_root.symlink_to(real, target_is_directory=True)
            try:
                publish_outputs(
                    [(reference_root / "unsafe.html", "unsafe")],
                    False,
                    root=root,
                    reference_root=reference_root,
                )
            except ValueError:
                pass
            else:
                raise AssertionError("symlinked output parent unexpectedly passed")

    if hasattr(os, "symlink"):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            reference_root = root / "reference"
            reference_root.mkdir()
            target = reference_root / "target.html"
            real = root / "real.html"
            real.write_text("real", encoding="utf-8")
            target.symlink_to(real)
            try:
                publish_outputs(
                    [(target, "unsafe")],
                    False,
                    root=root,
                    reference_root=reference_root,
                )
            except ValueError:
                pass
            else:
                raise AssertionError("symlinked output target unexpectedly passed")

    with tempfile.TemporaryDirectory() as raw:
        root = Path(raw)
        reference_root = root / "reference"
        target = reference_root / "directory.html"
        target.mkdir(parents=True)
        try:
            publish_outputs(
                [(target, "unsafe")],
                False,
                root=root,
                reference_root=reference_root,
            )
        except ValueError:
            pass
        else:
            raise AssertionError("non-regular output target unexpectedly passed")

    with tempfile.TemporaryDirectory() as raw:
        root = Path(raw)
        reference_root = root / "reference"
        targets = [
            (reference_root / "one.html", "one"),
            (reference_root / "two.html", "two"),
        ]
        original_fsync = os.fsync
        fsync_calls = 0

        def fail_second_sync(file_descriptor: int) -> None:
            nonlocal fsync_calls
            fsync_calls += 1
            if fsync_calls == 2:
                raise OSError("simulated fsync failure")
            original_fsync(file_descriptor)

        os.fsync = fail_second_sync
        try:
            try:
                publish_outputs(
                    targets,
                    False,
                    root=root,
                    reference_root=reference_root,
                )
            except OSError:
                pass
            else:
                raise AssertionError("simulated fsync failure was not raised")
        finally:
            os.fsync = original_fsync
        if list(reference_root.glob(".*.tmp")):
            raise AssertionError("failed staged outputs were not cleaned up")
        if any(path.exists() for path, _ in targets):
            raise AssertionError("a destination changed after failed staging")

    with tempfile.TemporaryDirectory() as raw:
        root = Path(raw)
        reference_root = root / "reference"
        reference_root.mkdir()
        targets = [
            (reference_root / "one.html", "one"),
            (reference_root / "two.html", "two"),
        ]
        targets[0][0].write_bytes(b"old one\n")
        targets[1][0].write_bytes(b"old two\n")
        original_replace = os.replace
        replace_calls = 0

        def fail_second_replace(source: str | Path, destination: str | Path) -> None:
            nonlocal replace_calls
            replace_calls += 1
            if replace_calls == 2:
                raise OSError(f"simulated replacement failure: {source} -> {destination}")
            original_replace(source, destination)

        os.replace = fail_second_replace
        try:
            try:
                publish_outputs(
                    targets,
                    False,
                    root=root,
                    reference_root=reference_root,
                )
            except OSError:
                pass
            else:
                raise AssertionError("simulated replacement failure was not raised")
        finally:
            os.replace = original_replace
        if replace_calls < 2:
            raise AssertionError("unexpected replacement count during failure probe")
        if targets[0][0].read_bytes() != b"old one\n":
            raise AssertionError("first existing output was not rolled back")
        if targets[1][0].read_bytes() != b"old two\n":
            raise AssertionError("second existing output changed after failed replacement")
        if list(reference_root.glob(".*.tmp")) or list(reference_root.glob(".*.bak")):
            raise AssertionError("publication rollback left temporary residue")

    with tempfile.TemporaryDirectory() as raw:
        root = Path(raw)
        reference_root = root / "reference"
        reference_root.mkdir()
        targets = [
            (reference_root / "one.html", "one"),
            (reference_root / "two.html", "two"),
        ]
        targets[1][0].write_bytes(b"old two\n")
        original_replace = os.replace
        replace_calls = 0

        def fail_second_replace(source: str | Path, destination: str | Path) -> None:
            nonlocal replace_calls
            replace_calls += 1
            if replace_calls == 2:
                raise OSError(f"simulated replacement failure: {source} -> {destination}")
            original_replace(source, destination)

        os.replace = fail_second_replace
        try:
            try:
                publish_outputs(
                    targets,
                    False,
                    root=root,
                    reference_root=reference_root,
                )
            except OSError:
                pass
            else:
                raise AssertionError("simulated replacement failure was not raised")
        finally:
            os.replace = original_replace
        if targets[0][0].exists():
            raise AssertionError("new output was not removed during rollback")
        if targets[1][0].read_bytes() != b"old two\n":
            raise AssertionError("existing output changed after failed replacement")
        if list(reference_root.glob(".*.tmp")) or list(reference_root.glob(".*.bak")):
            raise AssertionError("publication rollback left temporary residue")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_tests()
        return
    state = load_status()
    cards = load_cards()
    if state not in {"cards-complete", "complete"}:
        raise SystemExit("teaching references require cards-complete state")
    if len(cards) != 42:
        raise SystemExit(f"expected 42 source cards, found {len(cards)}")
    publish_outputs(
        [
            (
                REFERENCE_ROOT / "weng-source-cards.html",
                render_source_cards(cards),
            ),
            (
                REFERENCE_ROOT / "weng-harness-map.html",
                render_harness_map(cards),
            ),
        ],
        args.check,
    )


if __name__ == "__main__":
    main()
