"""Static, fail-closed provider-independence checks for Lean source routes."""

from __future__ import annotations

import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


LEAN_MODULE_NAME = re.compile(
    r"[A-Za-z_][A-Za-z0-9_']*(?:\.[A-Za-z_][A-Za-z0-9_']*)*\Z"
)
LEAN_MODULE_NAME_BODY = r"[A-Za-z_][A-Za-z0-9_']*(?:\.[A-Za-z_][A-Za-z0-9_']*)*"
LEAN_SECTION_NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_']*\Z")
IMPORT_COMMAND = re.compile(
    r"(?:(?P<public>public)\s+)?"
    r"(?:(?P<meta>meta)\s+)?"
    r"import"
    r"(?:\s+(?P<all>all))?"
    rf"\s+(?P<module>{LEAN_MODULE_NAME_BODY})\Z"
)
SECTION_COMMAND = re.compile(
    r"(?:(?:public)\s+)?(?:meta\s+)?"
    r"section(?:\s+(?P<name>[A-Za-z_][A-Za-z0-9_']*))?\Z"
)
HEADER_INITIAL_KEYWORDS = frozenset(
    {"all", "import", "meta", "module", "prelude", "public"}
)
MODIFIED_BODY_COMMANDS = frozenset(
    {
        "abbrev",
        "axiom",
        "class",
        "def",
        "elab",
        "example",
        "inductive",
        "instance",
        "lemma",
        "macro",
        "notation",
        "opaque",
        "prefix",
        "postfix",
        "scoped",
        "structure",
        "syntax",
        "theorem",
    }
)
TOKEN = re.compile(r"(?<![\w'])[_A-Za-z][A-Za-z0-9_']*(?![\w'!?])")
DANGER_TOKENS = frozenset(
    {"sorry", "admit", "unsafe", "native_decide", "implemented_by"}
)
MANAGED_LOCAL_NAMESPACE_ROOTS = ("Crouzeix", "CrouzeixConjecture")
MAX_LOCAL_MODULES = 4096
MAX_SOURCE_BYTES = 4 * 1024 * 1024


class ProviderIndependenceError(ValueError):
    """A source route cannot be certified as provider independent."""


@dataclass(frozen=True, order=True)
class SourceFinding:
    module: str
    line: int
    token: str
    source_line: str


@dataclass(frozen=True)
class SourceScan:
    dangers: tuple[SourceFinding, ...]
    set_options: tuple[SourceFinding, ...]


@dataclass(frozen=True)
class ProviderIndependenceReport:
    roots: tuple[str, ...]
    modules: tuple[str, ...]
    set_options: tuple[SourceFinding, ...]


def parse_active_imports(source: str, module: str) -> tuple[str, ...]:
    """Return active imports from a Lean module header.

    Lean imports precede the first ordinary command, with optional ``module``
    and ``prelude`` commands before them, in that order. Comments and blank
    lines do not end the header.
    """

    imports: list[str] = []
    saw_module = False
    saw_prelude = False
    for line_number, command, is_header_candidate in _iter_active_header_commands(
        source, module
    ):
        if not is_header_candidate:
            break
        if command == "module":
            if saw_module or saw_prelude or imports:
                raise ProviderIndependenceError(
                    f"malformed module command in {module}:{line_number}"
                )
            saw_module = True
            continue
        if _starts_keyword(command, "module"):
            raise ProviderIndependenceError(
                f"malformed module command in {module}:{line_number}"
            )
        if command == "prelude":
            if saw_prelude or imports:
                raise ProviderIndependenceError(
                    f"malformed prelude command in {module}:{line_number}"
                )
            saw_prelude = True
            continue
        if _starts_keyword(command, "prelude"):
            raise ProviderIndependenceError(
                f"malformed prelude command in {module}:{line_number}"
            )
        import_match = IMPORT_COMMAND.fullmatch(command)
        if import_match is not None:
            if (
                import_match.group("public") is not None
                or import_match.group("meta") is not None
                or import_match.group("all") is not None
            ) and not saw_module:
                raise ProviderIndependenceError(
                    f"modified import requires module in {module}:{line_number}"
                )
            imports.append(import_match.group("module"))
            continue
        if SECTION_COMMAND.fullmatch(command) is not None:
            break
        if _is_valid_modified_body_command(command):
            break
        if (
            _starts_keyword(command, "import")
            or _starts_keyword(command, "meta")
            or _starts_keyword(command, "public")
            or _starts_keyword(command, "all")
        ):
            raise ProviderIndependenceError(
                f"malformed import-like command in {module}:{line_number}"
            )
        break
    return tuple(imports)


def _iter_active_header_commands(
    source: str, module: str
) -> Iterable[tuple[int, str, bool]]:
    """Yield comment-masked header candidate lines until the first body command."""

    index = 0
    line_number = 1
    block_depth = 0
    while index < len(source):
        start_line = line_number
        command: list[str] = []
        saw_header_initial_keyword = False

        while index < len(source):
            pair = source[index : index + 2]
            character = source[index]
            if block_depth:
                if pair == "/-":
                    block_depth += 1
                    index += 2
                    continue
                if pair == "-/":
                    block_depth -= 1
                    index += 2
                    if command:
                        command.append(" ")
                    continue
                if character in "\r\n":
                    index, line_number = _consume_newline(
                        source, index, line_number
                    )
                    break
                index += 1
                continue

            if pair == "--":
                index = _skip_to_newline(source, index + 2)
                if index < len(source):
                    index, line_number = _consume_newline(
                        source, index, line_number
                    )
                break
            if pair == "/-":
                if command:
                    command.append(" ")
                block_depth = 1
                index += 2
                continue
            if pair == "-/":
                raise ProviderIndependenceError(
                    f"unmatched block comment close in {module}"
                )
            if character in "\r\n":
                index, line_number = _consume_newline(source, index, line_number)
                break
            if not command and character.isspace():
                index += 1
                continue
            if _is_identifier_start(character):
                token_start = index
                index += 1
                while index < len(source) and _is_identifier_part(source[index]):
                    index += 1
                token = source[token_start:index]
                if not command and token not in HEADER_INITIAL_KEYWORDS:
                    yield start_line, token, False
                    return
                if not command:
                    saw_header_initial_keyword = True
                command.append(token)
                continue
            if not saw_header_initial_keyword:
                yield start_line, character, False
                return
            command.append(character)
            index += 1
        else:
            if block_depth:
                raise ProviderIndependenceError(
                    f"unterminated block comment in {module}"
                )

        stripped = "".join(command).strip()
        if stripped:
            yield start_line, stripped, True

    if block_depth:
        raise ProviderIndependenceError(f"unterminated block comment in {module}")


def _is_valid_modified_body_command(command: str) -> bool:
    parts = command.split()
    if len(parts) < 2:
        return False
    if parts[0] == "meta":
        return parts[1] in MODIFIED_BODY_COMMANDS
    if parts[0] != "public":
        return False
    if parts[1] in MODIFIED_BODY_COMMANDS:
        return True
    if parts[1] == "noncomputable":
        return (
            len(parts) in {3, 4}
            and parts[2] == "section"
            and (len(parts) == 3 or LEAN_SECTION_NAME.fullmatch(parts[3]) is not None)
        )
    if len(parts) < 3 or parts[1] != "meta":
        return False
    return parts[2] in MODIFIED_BODY_COMMANDS


def _consume_newline(
    source: str, index: int, line_number: int
) -> tuple[int, int]:
    if source.startswith("\r\n", index):
        return index + 2, line_number + 1
    return index + 1, line_number + 1


def _skip_to_newline(source: str, index: int) -> int:
    while index < len(source) and source[index] not in "\r\n":
        index += 1
    return index


def _is_identifier_start(character: str) -> bool:
    return character == "_" or character.isalpha()


def _is_identifier_part(character: str) -> bool:
    return character == "_" or character == "'" or character.isalnum()


def scan_active_source(source: str, module: str) -> SourceScan:
    """Find active proof-bypass tokens and report-only ``set_option`` uses."""

    active = _mask_inactive_source(source, module)
    original_lines = source.splitlines()
    dangers: list[SourceFinding] = []
    set_options: list[SourceFinding] = []
    for line_number, line in enumerate(active.splitlines(), start=1):
        source_line = original_lines[line_number - 1].strip()
        for match in TOKEN.finditer(line):
            token = match.group(0)
            if token == "set_option":
                set_options.append(
                    SourceFinding(module, line_number, token, source_line)
                )
            elif token in DANGER_TOKENS:
                dangers.append(SourceFinding(module, line_number, token, source_line))
        axiom = re.match(r"^[ \t]*axiom(?![\w'!?])", line)
        if axiom is not None:
            dangers.append(SourceFinding(module, line_number, "axiom", source_line))
    return SourceScan(tuple(sorted(dangers)), tuple(sorted(set_options)))


def audit_provider_independence(
    lean_root: Path,
    roots: Iterable[str],
    forbidden_modules: Iterable[str],
    *,
    forbidden_prefixes: Iterable[str] = (),
) -> ProviderIndependenceReport:
    """Audit the transitive local import closure of explicit Lean roots."""

    root_path = Path(lean_root)
    if root_path.is_symlink():
        raise ProviderIndependenceError("Lean source root cannot be a symlink")
    if not root_path.is_dir():
        raise ProviderIndependenceError(f"Lean source root does not exist: {root_path}")
    root_modules = _validated_module_names(roots, "root")
    if not root_modules:
        raise ProviderIndependenceError("at least one root module is required")
    forbidden = frozenset(_validated_module_names(forbidden_modules, "forbidden"))
    prefixes = _validated_module_names(forbidden_prefixes, "forbidden prefix")

    for root_module in root_modules:
        _ensure_local_module_file(root_path, root_module, is_root=True)
        if _is_forbidden(root_module, forbidden, prefixes):
            raise ProviderIndependenceError(f"forbidden root module: {root_module}")

    visited: set[str] = set()
    pending = list(reversed(root_modules))
    set_options: list[SourceFinding] = []
    while pending:
        module = pending.pop()
        if module in visited:
            continue
        if len(visited) >= MAX_LOCAL_MODULES:
            raise ProviderIndependenceError(
                f"local module closure exceeds cap of {MAX_LOCAL_MODULES}"
            )
        module_path = _ensure_local_module_file(root_path, module, is_root=False)
        if module_path is None:
            continue
        source = _read_source(module_path, module)
        visited.add(module)

        scan = scan_active_source(source, module)
        if scan.dangers:
            finding = scan.dangers[0]
            raise ProviderIndependenceError(
                f"danger token {finding.token} in {finding.module}:{finding.line}"
            )
        set_options.extend(scan.set_options)

        imports = parse_active_imports(source, module)
        for imported in imports:
            if _is_forbidden(imported, forbidden, prefixes):
                raise ProviderIndependenceError(
                    f"forbidden module {imported} imported by {module}"
                )
        for imported in reversed(imports):
            if imported not in visited:
                imported_path = _ensure_local_module_file(
                    root_path, imported, is_root=False
                )
                if imported_path is not None:
                    pending.append(imported)

    return ProviderIndependenceReport(
        roots=tuple(sorted(root_modules)),
        modules=tuple(sorted(visited)),
        set_options=tuple(sorted(set_options)),
    )


def _ensure_local_module_file(
    lean_root: Path, module: str, *, is_root: bool
) -> Path | None:
    path = _module_path(lean_root, module)
    current = lean_root
    for part in (*module.split(".")[:-1], module.split(".")[-1] + ".lean"):
        current = current / part
        if current.is_symlink():
            raise ProviderIndependenceError(f"symlink in Lean module path: {module}")
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        if _is_managed_local_module(module):
            raise ProviderIndependenceError(f"local module does not exist: {module}")
        if is_root:
            raise ProviderIndependenceError(f"root module does not exist: {module}")
        return None
    except OSError as error:
        raise ProviderIndependenceError(
            f"cannot inspect Lean module {module}: {error}"
        ) from error
    if path.is_symlink() or not path.is_file():
        raise ProviderIndependenceError(f"Lean module is not a regular file: {module}")
    if metadata.st_size > MAX_SOURCE_BYTES:
        raise ProviderIndependenceError(f"Lean module exceeds size cap: {module}")
    return path


def _mask_inactive_source(source: str, module: str) -> str:
    """Replace Lean comments and literals with spaces, preserving newlines."""

    output: list[str] = []
    index = 0
    block_depth = 0
    state = "code"
    raw_closer = ""
    interpolation_depths: list[int] = []

    def mask(character: str) -> str:
        return character if character in "\r\n" else " "

    while index < len(source):
        pair = source[index : index + 2]
        character = source[index]
        if state == "line_comment":
            output.append(mask(character))
            index += 1
            if character in "\r\n":
                state = "code"
            continue
        if state == "block_comment":
            if pair == "/-":
                block_depth += 1
                output.extend((" ", " "))
                index += 2
            elif pair == "-/":
                block_depth -= 1
                output.extend((" ", " "))
                index += 2
                if block_depth == 0:
                    state = "code"
            else:
                output.append(mask(character))
                index += 1
            continue
        if state in {"string", "character"}:
            output.append(mask(character))
            index += 1
            if character == "\\":
                if index >= len(source):
                    break
                output.append(mask(source[index]))
                index += 1
            elif character == ('"' if state == "string" else "'"):
                state = "code"
            continue
        if state == "interpolated_text":
            if pair in {"{{", "}}"}:
                output.extend((" ", " "))
                index += 2
                continue
            output.append(mask(character))
            index += 1
            if character == "\\":
                if index >= len(source):
                    break
                output.append(mask(source[index]))
                index += 1
            elif character == '"':
                state = "code"
            elif character == "{":
                interpolation_depths.append(1)
                state = "code"
            continue
        if state == "raw_string":
            if source.startswith(raw_closer, index):
                output.extend(" " for _ in raw_closer)
                index += len(raw_closer)
                state = "code"
            else:
                output.append(mask(character))
                index += 1
            continue
        if state == "escaped_identifier":
            output.append(mask(character))
            index += 1
            if character == "»":
                state = "code"
            continue

        if interpolation_depths and character == "{":
            interpolation_depths[-1] += 1
            output.append(character)
            index += 1
            continue
        if interpolation_depths and character == "}":
            interpolation_depths[-1] -= 1
            output.append(character)
            index += 1
            if interpolation_depths[-1] == 0:
                interpolation_depths.pop()
                state = "interpolated_text"
            continue
        if source.startswith('s!"', index) and _at_token_start(source, index):
            output.extend((" ", " ", " "))
            index += 3
            state = "interpolated_text"
            continue
        if character == "«":
            output.append(" ")
            index += 1
            state = "escaped_identifier"
            continue
        if pair == "--":
            output.extend((" ", " "))
            index += 2
            state = "line_comment"
            continue
        elif pair == "/-":
            output.extend((" ", " "))
            index += 2
            block_depth = 1
            state = "block_comment"
            continue
        elif pair == "-/":
            raise ProviderIndependenceError(
                f"unmatched block comment close in {module}"
            )
        else:
            raw_start = _raw_string_start(source, index)
            if raw_start is not None:
                raw_length, raw_closer = raw_start
                output.extend(" " for _ in range(raw_length))
                index += raw_length
                state = "raw_string"
                continue
        if character == '"':
            output.append(" ")
            index += 1
            state = "string"
        elif character == "'" and _starts_character_literal(source, index):
            output.append(" ")
            index += 1
            state = "character"
        else:
            output.append(character)
            index += 1

    if state == "block_comment":
        raise ProviderIndependenceError(f"unterminated block comment in {module}")
    if state == "string":
        raise ProviderIndependenceError(f"unterminated string literal in {module}")
    if state == "character":
        raise ProviderIndependenceError(f"unterminated character literal in {module}")
    if state == "raw_string":
        raise ProviderIndependenceError(f"unterminated raw string literal in {module}")
    if state == "interpolated_text" or interpolation_depths:
        raise ProviderIndependenceError(
            f"unterminated interpolated string literal in {module}"
        )
    if state == "escaped_identifier":
        raise ProviderIndependenceError(f"unterminated escaped identifier in {module}")
    return "".join(output)


def _raw_string_start(source: str, index: int) -> tuple[int, str] | None:
    if source[index] != "r" or not _at_token_start(source, index):
        return None
    cursor = index + 1
    while cursor < len(source) and source[cursor] == "#":
        cursor += 1
    if cursor >= len(source) or source[cursor] != '"':
        return None
    hash_count = cursor - index - 1
    return cursor - index + 1, '"' + "#" * hash_count


def _at_token_start(source: str, index: int) -> bool:
    return index == 0 or not (
        source[index - 1].isalnum() or source[index - 1] in "_'!?»"
    )


def _starts_character_literal(source: str, index: int) -> bool:
    if index > 0 and _is_identifier_part(source[index - 1]):
        return False
    if index + 2 >= len(source):
        return True
    if source[index + 1] == "\\":
        return True
    return source[index + 2] == "'"


def _validated_module_names(values: Iterable[str], label: str) -> tuple[str, ...]:
    result = tuple(values)
    for module in result:
        if not isinstance(module, str) or LEAN_MODULE_NAME.fullmatch(module) is None:
            raise ProviderIndependenceError(f"invalid {label} module: {module!r}")
    if len(set(result)) != len(result):
        raise ProviderIndependenceError(f"duplicate {label} module")
    return result


def _starts_keyword(command: str, keyword: str) -> bool:
    return command == keyword or (
        command.startswith(keyword)
        and len(command) > len(keyword)
        and command[len(keyword)].isspace()
    )


def _module_path(lean_root: Path, module: str) -> Path:
    return lean_root.joinpath(*module.split(".")).with_suffix(".lean")


def _read_source(path: Path, module: str) -> str:
    try:
        descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        with os.fdopen(descriptor, "rb") as handle:
            data = handle.read(MAX_SOURCE_BYTES + 1)
        if len(data) > MAX_SOURCE_BYTES:
            raise ProviderIndependenceError(f"Lean module exceeds size cap: {module}")
        return data.decode("utf-8")
    except (OSError, UnicodeError) as error:
        raise ProviderIndependenceError(
            f"cannot read Lean module {module}: {error}"
        ) from error


def _has_forbidden_prefix(module: str, prefixes: tuple[str, ...]) -> bool:
    return any(
        module == prefix or module.startswith(prefix + ".") for prefix in prefixes
    )


def _is_forbidden(
    module: str, forbidden: frozenset[str], prefixes: tuple[str, ...]
) -> bool:
    return module in forbidden or _has_forbidden_prefix(module, prefixes)


def _is_managed_local_module(module: str) -> bool:
    return any(
        module == root or module.startswith(root + ".")
        for root in MANAGED_LOCAL_NAMESPACE_ROOTS
    )
