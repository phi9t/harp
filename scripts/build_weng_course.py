from __future__ import annotations

import argparse
import csv
import html
import io
import json
import os
import re
import stat
import tempfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "content/weng-source-cards.tsv"
COMPARISON_MATRIX = ROOT / "content/weng-comparison-matrix.tsv"
CLAIM_LADDER = ROOT / "content/weng-claim-ladder.json"
STATUS = ROOT / "content/weng-course-status.json"
CARD_ROOT = ROOT / "knowledge/rsi/weng-sources"
REFERENCE_ROOT = ROOT / "reference"
EVIDENCE_GRAPH = ROOT / "content/sources/evidence_graph.tsv"
SOURCE_REGISTRY = ROOT / "content/sources/source_registry.tsv"

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
        "../knowledge/rsi/weng/01-system-being-improved.md",
        "../lessons/0001-system-being-improved.html",
    ),
    (
        "harness-design-patterns",
        "Harness design patterns",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#harness-design-patterns",
        "../knowledge/rsi/weng/02-harness-design-patterns.md",
        "../lessons/0002-harness-design-patterns.html",
    ),
    (
        "harness-layer-vs-core-intelligence",
        "Harness layer versus core intelligence",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#harness-layer-vs-core-intelligence",
        "../knowledge/rsi/weng/03-harness-layer-vs-core-intelligence.md",
        "../lessons/0003-harness-vs-core-intelligence.html",
    ),
    (
        "context-engineering",
        "Context engineering",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#context-engineering",
        "../knowledge/rsi/weng/04-context-engineering.md",
        "../lessons/0004-context-engineering.html",
    ),
    (
        "workflow-design-and-search",
        "Workflow design and search",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#workflow-design",
        "../knowledge/rsi/weng/05-workflow-design-and-search.md",
        "../lessons/0005-workflow-design-and-auto-research.html",
    ),
    (
        "self-improving-harnesses",
        "Self-improving harnesses",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#self-improving-harness",
        "../knowledge/rsi/weng/06-self-improving-harnesses.md",
        "../lessons/0006-self-improving-harnesses.html",
    ),
    (
        "evolutionary-search",
        "Evolutionary search",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#evolutionary-search",
        "../knowledge/rsi/weng/07-evolutionary-search.md",
        "../lessons/0007-evolutionary-search.html",
    ),
    (
        "joint-harness-weight-optimization",
        "Joint harness and weight optimization",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#joint-optimization-with-model-weights",
        "../knowledge/rsi/weng/08-joint-harness-weight-optimization.md",
        "../lessons/0008-joint-harness-weight-optimization.html",
    ),
    (
        "future-challenges",
        "Future challenges",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#future-challenges",
        "../knowledge/rsi/weng/09-future-challenges.md",
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
SOURCE_REGISTRY_FIELDS = (
    "label",
    "source_id",
    "depth",
    "cohort",
    "access_status",
    "artifact",
    "relationship",
    "primary_locator",
    "version_or_digest",
    "venue_status",
    "inspected_on",
    "claim_ceiling",
)
MAX_CARD_PROSE_WORDS = 240
ALLOWED_STATES = {"building", "cards-complete", "complete"}
BODY_LINK_LOCATORS = {
    "KARPATHY-AUTORESEARCH": "body-link-workflow-automation",
    "WENG-REWARD": "body-link-reward-hacking",
    "ANTHROPIC-RSI": "body-link-ai-progress",
}
COMPARISON_FIELDS = (
    "source_id",
    "edited_object",
    "persistence",
    "evaluator",
    "update_operator",
    "model_weight_change",
    "evidence_class",
    "comparison_claim_ceiling",
)
COMPARISON_COLUMNS = (
    "edited object",
    "persistence",
    "evaluator",
    "update operator",
    "model-weight change",
    "evidence class",
    "claim ceiling",
)
CLAIM_LEVEL_FIELDS = (
    "name",
    "required_evidence",
    "positive_example",
    "common_overclaim",
    "example_route",
    "canonical_concept_links",
)
CONCEPT_LINK_FIELDS = ("name", "route")
CLAIM_LEVEL_NAMES = (
    "task iteration",
    "persistent adaptation",
    "harness improvement",
    "successor improvement",
    "demonstrated recursive improvement",
)
@dataclass(frozen=True)
class Card:
    metadata: dict[str, str]
    sections: dict[str, str]


@dataclass(frozen=True)
class ConceptLink:
    name: str
    route: str


@dataclass(frozen=True)
class ClaimLevel:
    name: str
    required_evidence: str
    positive_example: str
    common_overclaim: str
    example_route: str
    canonical_concept_links: tuple[ConceptLink, ...]


def _nonempty_one_line(value: object, field: str) -> str:
    if not isinstance(value, str) or not value.strip() or value != value.strip():
        raise ValueError(f"{field} must be a nonempty string without outer whitespace")
    if "\n" in value or "\r" in value or "\t" in value:
        raise ValueError(f"{field} must be a one-line string")
    return value


def _repository_route(value: object, field: str) -> str:
    route = _nonempty_one_line(value, field)
    path_text, separator, fragment = route.partition("#")
    route_path = PurePosixPath(path_text)
    if (
        not path_text
        or path_text.startswith("/")
        or "\\" in path_text
        or route_path.is_absolute()
        or ".." in route_path.parts
        or "." in route_path.parts
        or not separator and "#" in fragment
        or (separator and (not fragment or "#" in fragment))
        or any(character.isspace() for character in route)
        or "?" in route
        or ":" in route_path.parts[0]
    ):
        raise ValueError(f"{field} must be a safe repository-relative route")
    return route


def parse_comparison_matrix_text(
    text: str, expected_source_ids: tuple[str, ...]
) -> dict[str, tuple[str, ...]]:
    reader = csv.DictReader(io.StringIO(text), delimiter="\t")
    if tuple(reader.fieldnames or ()) != COMPARISON_FIELDS:
        raise ValueError("unexpected Weng comparison matrix header")
    rows = list(reader)
    if any(None in row or None in row.values() for row in rows):
        raise ValueError("malformed Weng comparison matrix row")
    if len(rows) != len(expected_source_ids):
        raise ValueError("Weng comparison matrix must contain the exact source roster")
    source_ids = tuple(
        _nonempty_one_line(row["source_id"], "comparison source_id") for row in rows
    )
    if source_ids != expected_source_ids or len(set(source_ids)) != len(source_ids):
        raise ValueError("Weng comparison matrix source order does not match the roster")
    comparisons: dict[str, tuple[str, ...]] = {}
    for row in rows:
        source_id = row["source_id"]
        comparisons[source_id] = tuple(
            _nonempty_one_line(row[field], f"{source_id} {field}")
            for field in COMPARISON_FIELDS[1:]
        )
    return comparisons


def parse_claim_ladder_text(text: str) -> tuple[ClaimLevel, ...]:
    try:
        payload = json.loads(text)
    except json.JSONDecodeError as error:
        raise ValueError("invalid Weng claim-ladder JSON") from error
    if not isinstance(payload, dict) or tuple(payload) != ("schema_version", "levels"):
        raise ValueError("unexpected Weng claim-ladder schema")
    if payload["schema_version"] != 1 or not isinstance(payload["levels"], list):
        raise ValueError("unexpected Weng claim-ladder contract")
    if len(payload["levels"]) != len(CLAIM_LEVEL_NAMES):
        raise ValueError("Weng claim ladder must contain exactly five levels")

    levels: list[ClaimLevel] = []
    for index, raw_level in enumerate(payload["levels"]):
        if not isinstance(raw_level, dict) or tuple(raw_level) != CLAIM_LEVEL_FIELDS:
            raise ValueError("unexpected Weng claim-level schema")
        name = _nonempty_one_line(raw_level["name"], "claim level name")
        if name != CLAIM_LEVEL_NAMES[index]:
            raise ValueError("Weng claim-level order does not match the contract")
        raw_links = raw_level["canonical_concept_links"]
        if not isinstance(raw_links, list) or not raw_links:
            raise ValueError(f"{name} canonical_concept_links must be a nonempty list")
        links: list[ConceptLink] = []
        seen_names: set[str] = set()
        seen_routes: set[str] = set()
        for raw_link in raw_links:
            if not isinstance(raw_link, dict) or tuple(raw_link) != CONCEPT_LINK_FIELDS:
                raise ValueError(f"unexpected {name} concept-link schema")
            link = ConceptLink(
                _nonempty_one_line(raw_link["name"], f"{name} concept-link name"),
                _repository_route(raw_link["route"], f"{name} concept-link route"),
            )
            if link.name in seen_names or link.route in seen_routes:
                raise ValueError(f"{name} contains a duplicate canonical concept link")
            seen_names.add(link.name)
            seen_routes.add(link.route)
            links.append(link)
        levels.append(
            ClaimLevel(
                name,
                _nonempty_one_line(
                    raw_level["required_evidence"], f"{name} required_evidence"
                ),
                _nonempty_one_line(
                    raw_level["positive_example"], f"{name} positive_example"
                ),
                _nonempty_one_line(
                    raw_level["common_overclaim"], f"{name} common_overclaim"
                ),
                _repository_route(raw_level["example_route"], f"{name} example_route"),
                tuple(links),
            )
        )
    return tuple(levels)


def _decode_frontmatter_scalar(raw: str) -> str:
    if raw.startswith('"') or raw.endswith('"'):
        if not (raw.startswith('"') and raw.endswith('"')):
            raise ValueError("frontmatter scalar has mismatched quotes")
        try:
            value = json.loads(raw)
        except json.JSONDecodeError as error:
            raise ValueError("frontmatter scalar must use JSON string quoting") from error
        if not isinstance(value, str):
            raise ValueError("frontmatter quoted scalar must decode to a string")
        if not value or "\n" in value or "\r" in value:
            raise ValueError("frontmatter scalar must be a nonempty one-line string")
        return value
    if _plain_scalar_requires_quotes(raw):
        raise ValueError("frontmatter plain scalar uses YAML-significant syntax")
    return raw


def _plain_scalar_requires_quotes(raw: str) -> bool:
    yaml_indicators = "-?:,[]{}#&*!|>'\"%@`"
    yaml_keywords = {
        "null",
        "true",
        "false",
        "yes",
        "no",
        "on",
        "off",
        "~",
        ".nan",
        ".inf",
        "+.inf",
        "-.inf",
    }
    return (
        not raw
        or raw[0] in yaml_indicators
        or raw.casefold() in yaml_keywords
        or _numeric_or_date_scalar(raw)
        or " #" in raw
        or ": " in raw
        or raw.endswith(":")
        or any(
            character in "[]{}\t" or ord(character) < 0x20 or ord(character) == 0x7F
            for character in raw
        )
    )


def _numeric_or_date_scalar(raw: str) -> bool:
    if re.fullmatch(r"\d{4}-\d{1,2}-\d{1,2}(?:[Tt ].+)?", raw):
        return True
    if re.fullmatch(
        r"[+-]?(?:0[xX][0-9A-Fa-f_]+|0[oO][0-7_]+|0[bB][01_]+)",
        raw,
    ):
        return True
    try:
        float(raw.replace("_", ""))
    except ValueError:
        return False
    return True


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
        raw_value = value.strip()
        if not key or not raw_value:
            raise ValueError("frontmatter keys and values must be nonempty")
        if key in metadata:
            raise ValueError(f"duplicate frontmatter key: {key}")
        metadata[key] = _decode_frontmatter_scalar(raw_value)
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
    word_count = sum(len(section.split()) for section in sections.values())
    if word_count > MAX_CARD_PROSE_WORDS:
        raise ValueError(
            f"card prose exceeds {MAX_CARD_PROSE_WORDS} words for "
            f"{expected_title}: {word_count}"
        )
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


def load_source_registry() -> dict[str, dict[str, str]]:
    with SOURCE_REGISTRY.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != SOURCE_REGISTRY_FIELDS:
            raise ValueError("unexpected source registry header")
        rows = list(reader)
    if any(None in row or None in row.values() for row in rows):
        raise ValueError("malformed source registry row")
    if any(not all(row[field].strip() for field in SOURCE_REGISTRY_FIELDS) for row in rows):
        raise ValueError("source registry row contains an empty field")
    registry: dict[str, dict[str, str]] = {}
    for row in rows:
        source_id = row["source_id"]
        if source_id in registry:
            raise ValueError(f"duplicate source registry ID: {source_id}")
        registry[source_id] = row
    return registry


def validate_registry_parity(
    row: dict[str, str],
    metadata: dict[str, str],
    registry_row: dict[str, str],
) -> None:
    source_id = row["source_id"]
    comparisons = (
        ("title", row["title"], registry_row["artifact"]),
        ("primary_url", row["primary_url"], registry_row["primary_locator"]),
        (
            "publication_state",
            metadata["publication_state"],
            registry_row["venue_status"],
        ),
        ("claim_ceiling", row["claim_ceiling"], registry_row["claim_ceiling"]),
    )
    for field, actual, expected in comparisons:
        if actual != expected:
            raise ValueError(f"{source_id} {field} disagrees with source registry")
    if row["evidence_state"] == "card-complete":
        if registry_row["label"] != "EVIDENCE":
            raise ValueError(f"{source_id} card-complete registry label must be EVIDENCE")
        access_status = registry_row["access_status"]
        if access_status != "inspected" and not access_status.endswith("-inspected"):
            raise ValueError(
                f"{source_id} card-complete registry access_status must end in inspected"
            )


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
    registry = load_source_registry()

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
            metadata["canonical_route"], ROOT / "knowledge/rsi", "canonical_route"
        )
        registry_row = registry.get(row["source_id"])
        if registry_row is None:
            raise ValueError(f"{row['source_id']} is absent from the source registry")
        validate_registry_parity(row, metadata, registry_row)
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


def load_comparison_matrix(cards: list[Card]) -> dict[str, tuple[str, ...]]:
    source_ids = tuple(card.metadata["source_id"] for card in cards)
    return parse_comparison_matrix_text(
        COMPARISON_MATRIX.read_text(encoding="utf-8"), source_ids
    )


def load_claim_ladder() -> tuple[ClaimLevel, ...]:
    return parse_claim_ladder_text(CLAIM_LADDER.read_text(encoding="utf-8"))


def reference_href(route: str) -> str:
    return f"../{route}"


def paragraphs(text: str) -> str:
    blocks = [block.strip() for block in text.split("\n\n") if block.strip()]
    return "".join(
        f"<p>{html.escape(block).replace('/', '&#47;')}</p>" for block in blocks
    )


def heading_slug(heading: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", heading.casefold()).strip("-")


def render_card(card: Card) -> str:
    metadata = card.metadata
    source_id = metadata["source_id"]
    fields = "".join(
        (
            f'<section class="card-field" aria-labelledby="source-{html.escape(source_id)}-{heading_slug(heading)}">'
            f'<h3 id="source-{html.escape(source_id)}-{heading_slug(heading)}">'
            f"{html.escape(heading)}</h3>"
            f"{paragraphs(card.sections[heading])}</section>"
        )
        for heading in HEADINGS
    )
    return (
        f'<article data-source-card id="source-{html.escape(source_id)}" '
        f'data-source-id="{html.escape(source_id)}" '
        f'data-locator="{html.escape(metadata["weng_locator"])}" '
        f'data-section="{html.escape(metadata["section_id"])}" '
        f'data-family="{html.escape(metadata["edited_object_family"])}" '
        f'data-evidence-state="{html.escape(metadata["evidence_state"])}">'
        f'<p class="source-id">{html.escape(source_id)}</p>'
        f'<h2>{html.escape(metadata["title"])}</h2>'
        f"{fields}"
        f'<p><a href="{html.escape(metadata["primary_url"], quote=True)}">'
        "Primary source</a></p>"
        f'<p><a href="../{html.escape(metadata["canonical_route"], quote=True)}">'
        "Canonical Harp route</a></p>"
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
        '<p class="reference-panel">Generated from the canonical inputs '
        '<a href="../content/weng-source-cards.tsv">content/weng-source-cards.tsv</a>, '
        '<code>knowledge/rsi/weng-sources/</code>, and the '
        '<a href="../content/sources/source_registry.tsv">source registry</a>. '
        "This HTML is a derived, non-authoritative teaching projection.</p>"
        '<form aria-label="Filter source cards">'
        '<label for="source-query">Search source cards</label>'
        '<input id="source-query" data-source-query type="search" '
        'placeholder="Source, section, family, or text"></form>'
        f'<p data-result-count aria-live="polite">{len(cards)} of {len(cards)} sources</p>'
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
        f'<a href="{html.escape(original, quote=True)}">Original public anchor</a> · '
        f'<a href="{html.escape(companion, quote=True)}">Canonical companion</a> · '
        f'<a href="{html.escape(lesson, quote=True)}">Matching lesson</a></li>'
        for section, title, original, companion, lesson in SECTION_META
        for count in (section_counts.get(section, 0),)
    )
    return (
        '<!doctype html><html lang="en"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        "<title>Weng Harness Map</title>"
        '<link rel="stylesheet" href="../assets/course.css"></head>'
        "<body><main><h1>Weng Harness Map</h1>"
        '<p class="reference-panel">Generated from '
        '<a href="../content/weng-source-cards.tsv">content/weng-source-cards.tsv</a>, '
        '<code>knowledge/rsi/weng-sources/</code>, and the '
        '<a href="../content/sources/source_registry.tsv">source registry</a>. '
        "This map is a derived, non-authoritative teaching projection.</p>"
        "<p><strong>Optimization ladder:</strong> prompts -&gt; structured context "
        "-&gt; workflow -&gt; harness code -&gt; optimizer code</p>"
        f"<ol>{sections}</ol>"
        f"<p><strong>Total:</strong> {sum(section_counts.values())} sources.</p>"
        "</main></body></html>\n"
    )


def comparison_rows(
    cards: list[Card],
    comparisons: dict[str, tuple[str, ...]],
) -> list[tuple[Card, tuple[str, ...]]]:
    card_ids = {card.metadata["source_id"] for card in cards}
    comparison_ids = set(comparisons)
    missing = card_ids - comparison_ids
    extra = comparison_ids - card_ids
    if missing or extra:
        raise ValueError(
            "comparison registry must exactly match source cards: "
            f"missing={sorted(missing)}, extra={sorted(extra)}"
        )
    for source_id, comparison in comparisons.items():
        if len(comparison) != len(COMPARISON_COLUMNS):
            raise ValueError(
                f"comparison registry row for {source_id} must contain "
                f"{len(COMPARISON_COLUMNS)} columns"
            )
        if not all(value.strip() for value in comparison):
            raise ValueError(f"comparison registry row for {source_id} has an empty value")
    return [(card, comparisons[card.metadata["source_id"]]) for card in cards]


def render_claim_ladder(claim_levels: tuple[ClaimLevel, ...]) -> str:
    levels = "".join(
        (
            f'<li id="claim-level-{index}" class="reference-panel">'
            f"<h2>{index}. {html.escape(level.name)}</h2>"
            f"<p><strong>Required evidence:</strong> {html.escape(level.required_evidence)}</p>"
            f'<p><strong>Positive example:</strong> <a href="{html.escape(reference_href(level.example_route), quote=True)}">'
            f"{html.escape(level.positive_example)}</a></p>"
            f"<p><strong>Common overclaim:</strong> {html.escape(level.common_overclaim)}</p>"
            "<p><strong>Canonical Harp concepts:</strong> "
            + " · ".join(
                f'<a href="{html.escape(reference_href(link.route), quote=True)}">{html.escape(link.name)}</a>'
                for link in level.canonical_concept_links
            )
            + "</p></li>"
        )
        for index, level in enumerate(claim_levels, start=1)
    )
    return (
        '<!doctype html><html lang="en"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        "<title>RSI Claim Ladder</title>"
        '<link rel="stylesheet" href="../assets/course.css"></head>'
        "<body><main><h1>RSI Claim Ladder</h1>"
        '<p class="reference-panel">Generated from '
        '<a href="../content/weng-claim-ladder.json">'
        "content/weng-claim-ladder.json</a> and its canonical Harp concept links. "
        "This HTML is derived and non-authoritative. Harp does not claim to have "
        "demonstrated recursive self-improvement.</p>"
        "<ol>"
        f"{levels}"
        "</ol></main></body></html>\n"
    )


def render_comparison_matrix(
    cards: list[Card], comparisons: dict[str, tuple[str, ...]]
) -> str:
    headers = "".join(f"<th scope=\"col\">{html.escape(column)}</th>" for column in COMPARISON_COLUMNS)
    rows = "".join(
        (
            "<tr data-comparison-row>"
            f'<th scope="row"><a href="weng-source-cards.html#source-{html.escape(card.metadata["source_id"], quote=True)}">'
            f"{html.escape(card.metadata['title'])}</a>"
            f'<span class="source-id">{html.escape(card.metadata["source_id"])}</span></th>'
            + "".join(f"<td>{html.escape(value)}</td>" for value in comparison)
            + "</tr>"
        )
        for card, comparison in comparison_rows(cards, comparisons)
    )
    return (
        '<!doctype html><html lang="en"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        "<title>Harness Comparison Matrix</title>"
        '<link rel="stylesheet" href="../assets/course.css"></head>'
        "<body><main><h1>Harness Comparison Matrix</h1>"
        '<p class="reference-panel">Generated from '
        '<a href="../content/weng-comparison-matrix.tsv">'
        "content/weng-comparison-matrix.tsv</a>, "
        '<a href="../content/weng-source-cards.tsv">content/weng-source-cards.tsv</a>, '
        '<code>knowledge/rsi/weng-sources/</code>, and the '
        '<a href="../content/sources/source_registry.tsv">source registry</a>. '
        "This comparison is a derived, non-authoritative teaching projection; "
        "its evidence classes and claim ceilings bound rather than extend the "
        "canonical cards.</p>"
        '<table><thead><tr><th scope="col">source</th>'
        f"{headers}</tr></thead><tbody>{rows}</tbody></table>"
        "</main></body></html>\n"
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
    comparison_header = "\t".join(
        (
            "source_id",
            "edited_object",
            "persistence",
            "evaluator",
            "update_operator",
            "model_weight_change",
            "evidence_class",
            "comparison_claim_ceiling",
        )
    )
    comparison_rows_text = "\n".join(
        (
            comparison_header,
            "SOURCE-A\tobject a\tpersistence a\tevaluator a\tupdate a\tno\tclass a\tceiling a",
            "SOURCE-B\tobject b\tpersistence b\tevaluator b\tupdate b\tyes\tclass b\tceiling b",
            "",
        )
    )
    parsed_comparisons = parse_comparison_matrix_text(
        comparison_rows_text, ("SOURCE-A", "SOURCE-B")
    )
    if tuple(parsed_comparisons) != ("SOURCE-A", "SOURCE-B"):
        raise AssertionError("comparison parser did not preserve semantic order")
    invalid_comparison_inputs = (
        comparison_rows_text.replace("edited_object", "unknown", 1),
        comparison_rows_text.replace("SOURCE-A", "SOURCE-B", 1),
        comparison_rows_text.replace(
            "SOURCE-A\tobject a\tpersistence a\tevaluator a\tupdate a\tno\tclass a\tceiling a\n",
            "",
            1,
        ),
        comparison_rows_text.replace(
            "SOURCE-B\tobject b\tpersistence b\tevaluator b\tupdate b\tyes\tclass b\tceiling b\n",
            "SOURCE-C\tobject c\tpersistence c\tevaluator c\tupdate c\tno\tclass c\tceiling c\n",
            1,
        ),
        comparison_rows_text.replace("\tobject a\t", "\t\t", 1),
        comparison_rows_text.replace("ceiling a", "ceiling a\ncontinued", 1),
    )
    for invalid in invalid_comparison_inputs:
        try:
            parse_comparison_matrix_text(invalid, ("SOURCE-A", "SOURCE-B"))
        except ValueError:
            pass
        else:
            raise AssertionError("invalid canonical comparison TSV unexpectedly passed")

    claim_level_names = (
        "task iteration",
        "persistent adaptation",
        "harness improvement",
        "successor improvement",
        "demonstrated recursive improvement",
    )
    claim_levels_payload = {
        "schema_version": 1,
        "levels": [
            {
                "name": name,
                "required_evidence": f"Required evidence for {name}.",
                "positive_example": f"Positive example for {name}.",
                "common_overclaim": f"Common overclaim for {name}.",
                "example_route": "knowledge/rsi/concepts/improvement-types.md",
                "canonical_concept_links": [
                    {
                        "name": "Improvement types",
                        "route": "knowledge/rsi/concepts/improvement-types.md",
                    },
                    {
                        "name": "System state",
                        "route": "knowledge/rsi/concepts/system-state-and-notation.md#notation",
                    },
                ],
            }
            for name in claim_level_names
        ],
    }
    claim_levels_text = json.dumps(claim_levels_payload)
    parsed_levels = parse_claim_ladder_text(claim_levels_text)
    if tuple(level.name for level in parsed_levels) != claim_level_names:
        raise AssertionError("claim-ladder parser did not preserve level order")
    invalid_claim_payloads = []
    for mutate in (
        lambda value: value.update({"unknown": True}),
        lambda value: value.pop("schema_version"),
        lambda value: value["levels"][0].update({"unknown": "field"}),
        lambda value: value["levels"][0].pop("required_evidence"),
        lambda value: value["levels"].reverse(),
        lambda value: value["levels"].pop(),
        lambda value: value["levels"].append(dict(value["levels"][-1])),
        lambda value: value["levels"][0].update({"required_evidence": ""}),
        lambda value: value["levels"][0].update(
            {"positive_example": "first line\nsecond line"}
        ),
        lambda value: value["levels"][0].update(
            {"example_route": "../outside.md"}
        ),
        lambda value: value["levels"][0].update(
            {"example_route": "knowledge/rsi/concepts/improvement-types.md?view=unsafe"}
        ),
        lambda value: value["levels"][0]["canonical_concept_links"].append(
            dict(value["levels"][0]["canonical_concept_links"][0])
        ),
        lambda value: value["levels"][0]["canonical_concept_links"].append(
            {
                "name": "Duplicate route",
                "route": value["levels"][0]["canonical_concept_links"][0]["route"],
            }
        ),
        lambda value: value["levels"][0]["canonical_concept_links"][0].pop("route"),
        lambda value: value["levels"][0]["canonical_concept_links"][0].update(
            {"route": "https://example.test/unsafe"}
        ),
    ):
        invalid = json.loads(claim_levels_text)
        mutate(invalid)
        invalid_claim_payloads.append(json.dumps(invalid))
    for invalid in invalid_claim_payloads:
        try:
            parse_claim_ladder_text(invalid)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid canonical claim-ladder JSON unexpectedly passed")

    cards = load_cards()
    comparisons = load_comparison_matrix(cards)
    claim_levels = load_claim_ladder()

    source_cards = render_source_cards(cards)
    if source_cards.count("<article data-source-card") != 42:
        raise AssertionError("source-card reference must contain 42 cards")
    for attribute in (
        "data-source-id=",
        "data-locator=",
        "data-section=",
        "data-family=",
        "data-evidence-state=",
    ):
        if source_cards.count(attribute) != 42:
            raise AssertionError(f"source-card reference must contain 42 {attribute} attributes")
    if "42 of 42 sources" not in source_cards:
        raise AssertionError("source-card reference must expose its initial live count")
    comparison_matrix = render_comparison_matrix(cards, comparisons)
    if comparison_matrix.count("<tr data-comparison-row>") != 42:
        raise AssertionError("comparison matrix must contain 42 source rows")
    if comparison_matrix.count('href="weng-source-cards.html#source-') != 42:
        raise AssertionError("comparison matrix must link all 42 source cards")
    claim_ladder = render_claim_ladder(claim_levels)
    if claim_ladder.count('<li id="claim-level-') != 5:
        raise AssertionError("claim ladder must contain five ordered levels")
    harness_map = render_harness_map(cards)
    if harness_map.count("<li data-section=") != 9:
        raise AssertionError("harness map must contain nine sections")

    first_source_id = cards[0].metadata["source_id"]
    incomplete = dict(comparisons)
    incomplete.pop(first_source_id)
    try:
        comparison_rows(cards, incomplete)
    except ValueError:
        pass
    else:
        raise AssertionError("incomplete comparison registry unexpectedly passed")

    quoted_frontmatter = "\n".join(
        (
            "---",
            "source_id: TEST",
            'title: "Absolute Zero: Reinforced Self-play Reasoning with Zero Data"',
            "weng_locator: reference-1",
            "section_id: system-being-improved",
            "primary_url: https://example.test",
            "captured_path: evidence/test.txt",
            "publication_state: preprint",
            "evidence_state: card-complete",
            "edited_object_family: harness",
            "claim_ceiling: Test claim",
            "lesson_ids: 0001,0010",
            "card_path: knowledge/rsi/weng-sources/test.md",
            "canonical_route: knowledge/rsi/test.md",
            "---",
            "",
        )
    )
    metadata, _ = parse_frontmatter(quoted_frontmatter)
    if metadata["title"] != "Absolute Zero: Reinforced Self-play Reasoning with Zero Data":
        raise AssertionError("quoted colon title did not decode")
    for safe_plain in (
        "Author-reported mechanism; no independent reproduction",
        "ICML 2026 official poster",
    ):
        if _decode_frontmatter_scalar(safe_plain) != safe_plain:
            raise AssertionError(f"safe plain scalar did not round-trip: {safe_plain}")
    for unsafe_plain in (
        "plain # comment",
        "[one, two]",
        "true",
        "*alias",
        "@reserved",
        "Absolute Zero: Reinforced Self-play Reasoning with Zero Data",
        "foo:",
        "0xFF",
        "0b101",
        "0o77",
        "2026-08-08",
        "42",
    ):
        try:
            _decode_frontmatter_scalar(unsafe_plain)
        except ValueError:
            pass
        else:
            raise AssertionError(f"unsafe plain scalar unexpectedly passed: {unsafe_plain}")
    for quoted, expected in (
        ('"plain # comment"', "plain # comment"),
        ('"[one, two]"', "[one, two]"),
        ('"true"', "true"),
        ('"*alias"', "*alias"),
        ('"@reserved"', "@reserved"),
        ('"Absolute Zero: Reinforced Self-play Reasoning with Zero Data"', "Absolute Zero: Reinforced Self-play Reasoning with Zero Data"),
        ('"foo:"', "foo:"),
        ('"0xFF"', "0xFF"),
        ('"0b101"', "0b101"),
        ('"0o77"', "0o77"),
    ):
        if _decode_frontmatter_scalar(quoted) != expected:
            raise AssertionError(f"quoted scalar did not decode: {quoted}")
    for invalid_title in (
        "Absolute Zero: Reinforced Self-play Reasoning with Zero Data",
        '"Absolute Zero: Reinforced Self-play Reasoning with Zero Data',
        'Absolute Zero"',
        '"Absolute Zero\\q"',
    ):
        invalid = quoted_frontmatter.replace(
            'title: "Absolute Zero: Reinforced Self-play Reasoning with Zero Data"',
            f"title: {invalid_title}",
        )
        try:
            parse_frontmatter(invalid)
        except ValueError:
            pass
        else:
            raise AssertionError(f"unsafe frontmatter scalar unexpectedly passed: {invalid_title}")

    def word_boundary_body(word_count: int) -> str:
        counts = (word_count - 4, 1, 1, 1, 1)
        if len(HEADINGS) != len(counts):
            raise AssertionError("self-test heading and word-count shapes diverged")
        sections = "\n".join(
            f"## {heading}\n{' '.join(['word'] * count)}"
            for heading, count in zip(HEADINGS, counts)
        )
        return f"\n# Test Card\n\n{sections}"

    parse_sections(word_boundary_body(240), "Test Card")
    try:
        parse_sections(word_boundary_body(241), "Test Card")
    except ValueError:
        pass
    else:
        raise AssertionError("241-word source card unexpectedly passed")

    matrix_row = {
        "source_id": "TEST",
        "title": "Test title",
        "primary_url": "https://example.test",
        "evidence_state": "card-complete",
        "claim_ceiling": "Test ceiling",
    }
    card_metadata = {
        "source_id": "TEST",
        "title": "Test title",
        "primary_url": "https://example.test",
        "publication_state": "preprint",
        "evidence_state": "card-complete",
        "claim_ceiling": "Test ceiling",
    }
    registry_row = {
        "label": "EVIDENCE",
        "artifact": "Test title",
        "primary_locator": "https://example.test",
        "venue_status": "preprint",
        "access_status": "vendored-inspected",
        "claim_ceiling": "Test ceiling",
    }
    validate_registry_parity(matrix_row, card_metadata, registry_row)
    validate_registry_parity(
        matrix_row,
        card_metadata,
        {**registry_row, "access_status": "fetched-local-inspected"},
    )
    for field, value in (
        ("artifact", "Wrong title"),
        ("primary_locator", "https://wrong.example"),
        ("venue_status", "accepted"),
        ("claim_ceiling", "Wrong ceiling"),
        ("label", "MISSING"),
        ("access_status", "vendored-uninspected"),
    ):
        invalid_registry = {**registry_row, field: value}
        try:
            validate_registry_parity(matrix_row, card_metadata, invalid_registry)
        except ValueError:
            pass
        else:
            raise AssertionError(f"registry divergence unexpectedly passed: {field}")

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
    comparisons = load_comparison_matrix(cards)
    claim_levels = load_claim_ladder()
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
            (
                REFERENCE_ROOT / "rsi-claim-ladder.html",
                render_claim_ladder(claim_levels),
            ),
            (
                REFERENCE_ROOT / "harness-comparison-matrix.html",
                render_comparison_matrix(cards, comparisons),
            ),
        ],
        args.check,
    )


if __name__ == "__main__":
    main()
