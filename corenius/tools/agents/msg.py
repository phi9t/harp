#!/usr/bin/env python3
"""Append-only messages and handoffs for agentic-engineering efforts."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from worker import WorkerError, main_worktree, now  # noqa: E402

MESSAGE_TYPES = ("info", "instruction", "question", "handoff")
HEADER_NAMES = ("From", "To", "Type", "Re", "Needs-ack")
HEADER_RE = re.compile(r"^([^:]+):(?: ?(.*))?$")
STAMP_RE = re.compile(r"^(.+)\.(\d{6})-.+\.md$")


def clean_component(value: str, label: str) -> str:
    if not value:
        raise WorkerError(f"{label} is required")
    if "/" in value or "\\" in value or ".." in value or value in (".", ".."):
        raise WorkerError(f"{label} must be a single path component without '..'")
    if "\x00" in value:
        raise WorkerError(f"{label} contains a NUL byte")
    return value


def clean_message_name(value: str, label: str = "message file") -> str:
    clean_component(value, label)
    if value.endswith(".ack"):
        raise WorkerError(f"{label} must name a message, not an ack sidecar")
    if not value.endswith(".md"):
        raise WorkerError(f"{label} must end with .md")
    return value


def resolve_root(root: str | None) -> Path:
    if root:
        return Path(root).resolve()
    return main_worktree(".")


def messages_root(root: Path, effort: str) -> Path:
    effort = clean_component(effort, "effort")
    return root / ".scratch" / effort / "messages"


def inbox_dir(root: Path, effort: str, recipient: str) -> Path:
    return messages_root(root, effort) / clean_component(recipient, "to")


def header_line(name: str, value: str) -> str:
    return f"{name}: {value}" if value else f"{name}:"


def build_message(sender: str, recipient: str, kind: str, re_file: str, needs_ack: bool, body: str) -> str:
    lines = [
        header_line("From", sender),
        header_line("To", recipient),
        header_line("Type", kind),
        header_line("Re", re_file),
        header_line("Needs-ack", "yes" if needs_ack else "no"),
    ]
    return "\n".join(lines) + "\n\n" + body


def timestamp_for_filename(timestamp: str) -> str:
    clean_component(timestamp, "timestamp")
    return timestamp


def next_message_path(directory: Path, timestamp: str, sender: str) -> Path:
    prefix = f"{timestamp}."
    used: set[int] = set()
    if directory.exists():
        for path in directory.glob("*.md"):
            name = path.name
            if not name.startswith(prefix):
                continue
            match = STAMP_RE.match(name)
            if match:
                used.add(int(match.group(2)))
    for sequence in range(1_000_000):
        if sequence in used:
            continue
        return directory / f"{timestamp}.{sequence:06d}-{sender}.md"
    raise WorkerError(f"too many messages for timestamp {timestamp}")


def send_message(args: argparse.Namespace) -> int:
    root = resolve_root(args.root)
    effort = clean_component(args.effort, "effort")
    sender = clean_component(args.sender, "from")
    recipient = clean_component(args.recipient, "to")
    if args.re:
        re_file = clean_message_name(args.re, "re")
    else:
        re_file = ""
    body = args.body_file.read_text() if args.body_file else (args.body or "")
    timestamp = timestamp_for_filename(args.timestamp or now())
    directory = inbox_dir(root, effort, recipient)
    directory.mkdir(parents=True, exist_ok=True)
    text = build_message(sender, recipient, args.kind, re_file, args.needs_ack, body)
    while True:
        path = next_message_path(directory, timestamp, sender)
        try:
            with path.open("x", encoding="utf-8") as handle:
                handle.write(text)
            print(str(path))
            return 0
        except FileExistsError:
            continue


def parse_message(path: Path) -> tuple[dict[str, str], list[str]]:
    text = path.read_text()
    problems: list[str] = []
    if "\n\n" in text:
        header_text, _ = text.split("\n\n", 1)
    else:
        header_text = text
        problems.append("missing blank line after headers")
    lines = header_text.splitlines()
    headers: dict[str, str] = {}
    names: list[str] = []
    for index, line in enumerate(lines, start=1):
        match = HEADER_RE.match(line)
        if not match:
            problems.append(f"bad header line {index}")
            continue
        name, value = match.group(1), match.group(2) or ""
        names.append(name)
        if name in headers:
            problems.append(f"duplicate header {name}")
        headers[name] = value
        if name not in HEADER_NAMES:
            problems.append(f"unknown extra header {name}")

    for name in HEADER_NAMES:
        if name not in headers:
            problems.append(f"missing header {name}")

    known = [name for name in names if name in HEADER_NAMES]
    if all(known.count(name) == 1 for name in HEADER_NAMES) and known != list(HEADER_NAMES):
        problems.append("wrong header order: expected From, To, Type, Re, Needs-ack")
    if len(lines) != len(HEADER_NAMES):
        problems.append(f"expected {len(HEADER_NAMES)} headers, found {len(lines)}")
    if "Type" in headers and headers["Type"] not in MESSAGE_TYPES:
        problems.append(f"bad Type {headers['Type']!r}")
    if "Needs-ack" in headers and headers["Needs-ack"] not in ("yes", "no"):
        problems.append(f"bad Needs-ack {headers['Needs-ack']!r}")
    if headers.get("Re"):
        try:
            clean_message_name(headers["Re"], "Re")
        except WorkerError as error:
            problems.append(str(error))
    return headers, problems


def message_sort_key(path: Path) -> tuple[str, int, str]:
    match = STAMP_RE.match(path.name)
    if match:
        return match.group(1), int(match.group(2)), path.name
    return path.name, -1, path.name


def inbox_messages(directory: Path) -> list[Path]:
    if not directory.is_dir():
        return []
    messages = [p for p in directory.glob("*.md") if not (directory / (p.name + ".ack")).exists()]
    return sorted(messages, key=message_sort_key)


def cmd_inbox(args: argparse.Namespace) -> int:
    directory = inbox_dir(resolve_root(args.root), args.effort, args.recipient)
    items = []
    for path in inbox_messages(directory):
        headers, problems = parse_message(path)
        if problems:
            raise WorkerError(f"{path.name} failed lint: {problems[0]}")
        needs_ack = headers["Needs-ack"] == "yes"
        items.append(
            {
                "file": path.name,
                "path": str(path),
                "from": headers["From"],
                "to": headers["To"],
                "type": headers["Type"],
                "re": headers["Re"],
                "needs_ack": needs_ack,
                "flags": ["NEEDS-ACK"] if needs_ack else [],
            }
        )
    if args.json:
        print(json.dumps(items, indent=2, sort_keys=True))
    else:
        for item in items:
            flag = "NEEDS-ACK" if item["needs_ack"] else "-"
            print(f"{item['file']}\t{flag}\t{item['from']}\t{item['type']}\t{item['re']}")
    return 0


def cmd_ack(args: argparse.Namespace) -> int:
    directory = inbox_dir(resolve_root(args.root), args.effort, args.recipient)
    name = clean_message_name(args.file)
    path = directory / name
    if not path.is_file():
        raise WorkerError(f"message {name} not found for {args.recipient}")
    ack_path = directory / f"{name}.ack"
    text = "\n".join([header_line("Acked-by", args.by), header_line("Acked-at", args.timestamp or now()), ""])
    try:
        with ack_path.open("x", encoding="utf-8") as handle:
            handle.write(text)
    except FileExistsError:
        pass
    print(str(ack_path))
    return 0


def display_path(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def cmd_lint(args: argparse.Namespace) -> int:
    root = resolve_root(args.root)
    base = messages_root(root, args.effort)
    problems: list[tuple[Path, str]] = []
    if base.is_dir():
        for path in sorted(base.glob("*/*.md"), key=lambda p: str(p)):
            _, file_problems = parse_message(path)
            problems.extend((path, problem) for problem in file_problems)
    for path, problem in problems:
        print(f"{display_path(path, root)}: {problem}")
    return 1 if problems else 0


def cmd_show(args: argparse.Namespace) -> int:
    print(Path(args.file).read_text(), end="")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--root", help="main worktree root (default: git main worktree)")

    p = sub.add_parser("send", parents=[common])
    p.add_argument("--effort", required=True)
    p.add_argument("--from", dest="sender", required=True)
    p.add_argument("--to", dest="recipient", required=True)
    p.add_argument("--type", dest="kind", choices=MESSAGE_TYPES, required=True)
    p.add_argument("--re")
    p.add_argument("--needs-ack", action="store_true")
    body = p.add_mutually_exclusive_group()
    body.add_argument("--body")
    body.add_argument("--body-file", type=Path)
    p.add_argument("--timestamp", help=argparse.SUPPRESS)
    p.set_defaults(fn=send_message)

    p = sub.add_parser("inbox", parents=[common])
    p.add_argument("--effort", required=True)
    p.add_argument("--to", dest="recipient", required=True)
    p.add_argument("--json", action="store_true")
    p.set_defaults(fn=cmd_inbox)

    p = sub.add_parser("ack", parents=[common])
    p.add_argument("--effort", required=True)
    p.add_argument("--to", dest="recipient", required=True)
    p.add_argument("file")
    p.add_argument("--by", required=True)
    p.add_argument("--timestamp", help=argparse.SUPPRESS)
    p.set_defaults(fn=cmd_ack)

    p = sub.add_parser("lint", parents=[common])
    p.add_argument("--effort", required=True)
    p.set_defaults(fn=cmd_lint)

    p = sub.add_parser("show")
    p.add_argument("file")
    p.set_defaults(fn=cmd_show)

    args = parser.parse_args(argv)
    try:
        return args.fn(args)
    except WorkerError as error:
        print(f"msg: {error}", file=sys.stderr)
        return 64


if __name__ == "__main__":
    raise SystemExit(main())
