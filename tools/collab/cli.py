"""Argument parsing and command dispatch for the collab CLI."""
from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
import os
from pathlib import Path
import sys
from typing import Callable

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from temp_root import temporary_root

from common import Refused
from core import Collab
from state_store import StoreError


@dataclass(frozen=True)
class Command:
    configure: Callable[[argparse.ArgumentParser], None]
    run: Callable[[Collab, argparse.Namespace], object]


def _add_init(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--trunk", required=True)
    parser.add_argument("--check-json", required=True)
    parser.add_argument("--protected-paths-json")
    parser.add_argument("--check-timeout-seconds", type=float)
    parser.add_argument("--workspace-root")
    parser.add_argument("--override-owners-json")


def _run_init(collab: Collab, args: argparse.Namespace) -> object:
    protected_paths = json.loads(args.protected_paths_json) if args.protected_paths_json is not None else None
    return collab.init(
        args.trunk,
        json.loads(args.check_json),
        args.workspace_root,
        protected_paths,
        args.check_timeout_seconds,
        json.loads(args.override_owners_json) if args.override_owners_json is not None else None,
    )


def _add_new(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("ticket")
    parser.add_argument("title")
    parser.add_argument("--deps", nargs="*", default=[])


def _run_new(collab: Collab, args: argparse.Namespace) -> object:
    return collab.new(args.ticket, args.title, args.deps)


def _add_open_attempt(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("ticket")
    parser.add_argument("--owner", required=True)
    parser.add_argument("--effort")
    parser.add_argument("--ttl-hours", type=float, default=24.0)


def _run_open_attempt(collab: Collab, args: argparse.Namespace) -> object:
    return collab.open_attempt(args.ticket, args.owner, args.effort, args.ttl_hours)


def _add_attempt(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("attempt")


def _add_confirmed_attempt(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("attempt")
    parser.add_argument("--confirm-stopped", action="store_true")


def _run_show(collab: Collab, args: argparse.Namespace) -> object:
    return collab.store.row(args.attempt)


def _run_submit(collab: Collab, args: argparse.Namespace) -> object:
    return collab.submit(args.attempt)


def _run_check(collab: Collab, args: argparse.Namespace) -> object:
    return collab.check(args.attempt)


def _run_land(collab: Collab, args: argparse.Namespace) -> object:
    return collab.land(args.attempt, args.owner_mainline_override)


def _add_land(parser: argparse.ArgumentParser) -> None:
    _add_attempt(parser)
    parser.add_argument("--owner-mainline-override")


def _run_recover(collab: Collab, args: argparse.Namespace) -> object:
    return collab.recover(args.attempt, args.confirm_stopped)


def _run_abandon(collab: Collab, args: argparse.Namespace) -> object:
    return collab.abandon(args.attempt, args.confirm_stopped)


def _run_gc(collab: Collab, args: argparse.Namespace) -> object:
    return collab.gc(args.attempt, args.confirm_stopped)


def _add_sweep(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--apply", action="store_true")


def _run_sweep(collab: Collab, args: argparse.Namespace) -> object:
    return collab.sweep(args.apply)


def _add_no_args(parser: argparse.ArgumentParser) -> None:
    del parser


def _run_status(collab: Collab, args: argparse.Namespace) -> object:
    del args
    return collab.status()


def _run_ready(collab: Collab, args: argparse.Namespace) -> object:
    del args
    return [ticket for ticket in collab.status()["tickets"] if ticket["ready"]]


def _add_backup(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("destination")


def _run_backup(collab: Collab, args: argparse.Namespace) -> object:
    return collab.backup(args.destination)


COMMANDS = {
    "init": Command(_add_init, _run_init),
    "new": Command(_add_new, _run_new),
    "open-attempt": Command(_add_open_attempt, _run_open_attempt),
    "show": Command(_add_attempt, _run_show),
    "submit": Command(_add_attempt, _run_submit),
    "check": Command(_add_attempt, _run_check),
    "land": Command(_add_land, _run_land),
    "recover": Command(_add_confirmed_attempt, _run_recover),
    "abandon": Command(_add_confirmed_attempt, _run_abandon),
    "gc": Command(_add_confirmed_attempt, _run_gc),
    "sweep": Command(_add_sweep, _run_sweep),
    "status": Command(_add_no_args, _run_status),
    "ready": Command(_add_no_args, _run_ready),
    "backup": Command(_add_backup, _run_backup),
}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Local cooperative Git-ticket coordination. Python standard library + Git.\n\n"
            "Not an agent runtime or sandbox. Commands mutate local state only; never push.\n"
            "See README.md for the trust, crash-recovery and ticket-format limitations."
        )
    )
    parser.add_argument("--repo", default=".")
    subcommands = parser.add_subparsers(dest="command", required=True)
    for name, command in COMMANDS.items():
        subparser = subcommands.add_parser(name)
        command.configure(subparser)
    return parser


def main(argv=None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        temporary = temporary_root()
        os.environ.update(TMPDIR=str(temporary), TMP=str(temporary), TEMP=str(temporary))
        collab = Collab(args.repo)
        out = COMMANDS[args.command].run(collab, args)
        print(json.dumps(out, indent=2, sort_keys=True))
        return 0
    except (Refused, StoreError, OSError, ValueError) as exc:
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        return 2
