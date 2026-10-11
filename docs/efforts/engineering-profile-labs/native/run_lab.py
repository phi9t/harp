#!/usr/bin/env python3
"""Bounded native-runtime learning probes; never synthesize native evidence."""

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import runpy
import selectors
import shutil
import signal
import subprocess
import tempfile
import time


PROMPT = "Without tools, files, commands, or explanation, answer only: what is 19 + 23?"
FEATURES = (
    "memories",
    "goals",
    "multi_agent",
    "multi_agent_v2",
    "code_mode_host",
    "plugins",
    "plugin_hooks",
    "hooks",
)
TOOLS = (
    "exec",
    "exec_command",
    "write_stdin",
    "shell",
    "shell_command",
    "run_shell_command",
    "web_search",
    "apply_patch",
    "view_image",
    "update_plan",
    "LSP",
    "wait",
    "get_goal",
    "create_goal",
    "update_goal",
    "spawn_agent",
    "send_input",
    "wait_agent",
    "close_agent",
    "resume_agent",
)
NON_TOOL_ARITHMETIC_ITEM_TYPES = frozenset(
    {
        "agent_message",
        "reasoning",
        "error",  # CLI diagnostics are not terminal failures.
        "userMessage",
        "agentMessage",
        "contextCompaction",  # App-server item names.
    }
)
REPOSITORY_ROOT = Path(__file__).resolve().parents[4]
temporary_root = runpy.run_path(str(REPOSITORY_ROOT / "tools" / "temp_root.py"))[
    "temporary_root"
]


def digest(path):
    """Hash a declared public binary or schema, never a credential file."""
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def stop(process):
    """Dispose the owned process group even if the leader has already exited."""
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=3)
    # A child may outlive a leader that responded to SIGTERM.
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def bounded(argv, timeout, cwd=None):
    """Run a trusted command with a finite deadline and retained output in memory."""
    process = subprocess.Popen(
        argv,
        cwd=cwd,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    timed_out = False
    try:
        stdout, stderr = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True
        stop(process)
        stdout, stderr = process.communicate(timeout=3)
    finally:
        stop(process)
    return {
        "argv": argv,
        "exit_code": process.returncode,
        "timed_out": timed_out,
        "stdout": stdout.decode(errors="replace"),
        "stderr_bytes": len(stderr),
        "stderr_sha256": hashlib.sha256(stderr).hexdigest(),
        "owned_process_group_disposed": True,
    }


def inventory():
    """Inspect public installed-tool help and binary identity without reading config."""
    result = {}
    commands = {
        "traecli": [
            ("--version",),
            ("--help",),
            ("exec", "--help"),
            ("exec", "resume", "--help"),
            ("acp", "serve", "--help"),
        ],
        "codex": [
            ("--version",),
            ("exec", "--help"),
            ("exec", "resume", "--help"),
            ("app-server", "--help"),
            ("app-server", "generate-json-schema", "--help"),
        ],
        "temporal": [("--version",), ("--help",)],
    }
    for name, probes in commands.items():
        executable = shutil.which(name)
        if executable is None:
            result[name] = {"available": False}
            continue
        result[name] = {
            "available": True,
            "executable": str(Path(executable).resolve()),
            "sha256": digest(Path(executable).resolve()),
            "probes": [bounded([executable, *probe], 10) for probe in probes],
        }
    return {"classification": "INSTALLED_BINARY_READ_ONLY", "tools": result}


def insula(executable, root, args):
    """Adapt native_init.py's proven private model-free bwrap boundary."""
    (root / "home").mkdir(exist_ok=True)
    (root / "workspace").mkdir(exist_ok=True)
    return [
        "bwrap",
        "--unshare-all",
        "--new-session",
        "--die-with-parent",
        "--as-pid-1",
        "--ro-bind",
        "/usr",
        "/usr",
        "--symlink",
        "usr/bin",
        "/bin",
        "--symlink",
        "usr/lib",
        "/lib",
        "--symlink",
        "usr/lib64",
        "/lib64",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/run/corenius-tmp",
        "--bind",
        str(root / "home"),
        "/home/worker",
        "--bind",
        str(root / "workspace"),
        "/workspace",
        "--ro-bind",
        str(Path(executable).resolve()),
        "/native-runtime",
        "--clearenv",
        "--setenv",
        "TMPDIR",
        "/run/corenius-tmp",
        "--setenv",
        "HOME",
        "/home/worker",
        "--setenv",
        "PATH",
        "/usr/bin:/bin",
        "--chdir",
        "/workspace",
        "/native-runtime",
        *args,
    ]


class Rpc:
    """Small bounded stdio JSON-RPC client; it is a lab, not a product adapter."""

    def __init__(self, argv, timeout):
        self.process = subprocess.Popen(
            argv,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.buffer = b""
        self.deadline = time.monotonic() + timeout
        self.events = []
        self.sequence = 0

    def send(self, method, params=None, notification=False):
        self.sequence += 1
        value = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            value["params"] = params
        if not notification:
            value["id"] = self.sequence
        self.events.append({"direction": "client_to_runtime", "message": value})
        self.process.stdin.write((json.dumps(value) + "\n").encode())
        self.process.stdin.flush()
        return value.get("id")

    def receive(self):
        while b"\n" not in self.buffer:
            remaining = self.deadline - time.monotonic()
            if remaining <= 0 or not self.selector.select(remaining):
                raise TimeoutError("overall protocol deadline elapsed")
            chunk = os.read(self.process.stdout.fileno(), 65536)
            if not chunk:
                raise RuntimeError("native stdout closed before matching response")
            self.buffer += chunk
            if len(self.buffer) > 2_000_000:
                raise RuntimeError("protocol buffer limit exceeded")
        line, self.buffer = self.buffer.split(b"\n", 1)
        value = json.loads(line)
        self.events.append({"direction": "runtime_to_client", "message": value})
        return value

    def request(self, method, params):
        request_id = self.send(method, params)
        while True:
            value = self.receive()
            if value.get("id") == request_id:
                return value
            if "id" in value and "method" in value:
                # Read-only probes never grant runtime-requested capabilities.
                refusal = {
                    "jsonrpc": "2.0",
                    "id": value["id"],
                    "error": {"code": -32601, "message": "lab denies tools"},
                }
                self.process.stdin.write((json.dumps(refusal) + "\n").encode())
                self.process.stdin.flush()

    def close(self):
        self.selector.close()
        stop(self.process)
        self.process.stdin.close()
        self.process.stdout.close()


def protocol(kind, timeout):
    """Exercise actual lifecycle RPCs offline; never issue a provider prompt."""
    name = "codex" if kind == "app-server" else "traecli"
    executable = shutil.which(name)
    if executable is None or shutil.which("bwrap") is None:
        raise RuntimeError(f"{name} and bwrap are required; no host fallback")
    with tempfile.TemporaryDirectory(prefix="native-lab-") as directory:
        root = Path(directory)
        args = ["app-server", "--stdio"] if kind == "app-server" else ["acp", "serve"]
        for flag in FEATURES:
            args += ["-c", f"features.{flag}=false"]
        args += ["-c", "project_doc_max_bytes=0"]
        if kind == "acp":
            for tool in TOOLS:
                args += ["--disallowed-tool", tool]
        client = Rpc(insula(executable, root, args), timeout)
        result = {
            "classification": "MODEL_FREE_ACTUAL_PROTOCOL",
            "kind": kind,
            "binary_sha256": digest(Path(executable).resolve()),
            "network_isolated": True,
            "credentials_mounted": False,
            "provider_turns": 0,
            "boundary_source": "crates/sqlite-controller/tests/support/native_init.py",
        }
        try:
            if kind == "app-server":
                result["initialize"] = client.request(
                    "initialize",
                    {
                        "clientInfo": {
                            "name": "corenius_learning_lab",
                            "version": "0.1",
                        },
                        "capabilities": {"experimentalApi": True},
                    },
                )
                client.send("initialized", notification=True)
                started = client.request(
                    "thread/start",
                    {
                        "cwd": "/workspace",
                        "approvalPolicy": "never",
                        "sandbox": "read-only",
                        "baseInstructions": "Arithmetic lab. Do not use tools.",
                        "developerInstructions": "Do not use tools.",
                    },
                )
                result["thread_start"] = started
                thread_id = started.get("result", {}).get("thread", {}).get("id")
                if thread_id:
                    result["thread_read"] = client.request(
                        "thread/read",
                        {
                            "threadId": thread_id,
                            "includeTurns": True,
                        },
                    )
                    result["thread_read_without_turns"] = client.request(
                        "thread/read",
                        {
                            "threadId": thread_id,
                            "includeTurns": False,
                        },
                    )
                    result["thread_resume"] = client.request(
                        "thread/resume",
                        {
                            "threadId": thread_id,
                        },
                    )
                    result["thread_resume_exclude_turns"] = client.request(
                        "thread/resume",
                        {
                            "threadId": thread_id,
                            "excludeTurns": True,
                            "approvalPolicy": "never",
                            "sandbox": "read-only",
                        },
                    )
                    result["thread_turns_list"] = client.request(
                        "thread/turns/list",
                        {
                            "threadId": thread_id,
                            "limit": 5,
                        },
                    )
                    result["interrupt_idle_thread"] = client.request(
                        "turn/interrupt",
                        {
                            "threadId": thread_id,
                            "turnId": "lab-no-active-turn",
                        },
                    )
                result["alive_after_rpcs"] = client.process.poll() is None
                result["first_incarnation_exchanges"] = client.events
                client.close()
                result["first_exit_code"] = client.process.returncode
                # An owned new incarnation uses the same private store, never a new thread/start.
                client = Rpc(insula(executable, root, args), timeout)
                result["restart_initialize"] = client.request(
                    "initialize",
                    {
                        "clientInfo": {
                            "name": "corenius_learning_lab",
                            "version": "0.1",
                        },
                        "capabilities": {"experimentalApi": True},
                    },
                )
                client.send("initialized", notification=True)
                if thread_id:
                    result["restart_resume"] = client.request(
                        "thread/resume",
                        {
                            "threadId": thread_id,
                            "excludeTurns": True,
                            "approvalPolicy": "never",
                            "sandbox": "read-only",
                        },
                    )
            else:
                result["initialize"] = client.request(
                    "initialize",
                    {
                        "protocolVersion": 1,
                        "clientCapabilities": {},
                        "clientInfo": {
                            "name": "corenius_learning_lab",
                            "version": "0.1",
                        },
                    },
                )
                result["session_new"] = client.request(
                    "session/new",
                    {
                        "cwd": "/workspace",
                        "mcpServers": [],
                        "_meta": {"trae": {"skills": [], "mcpServers": []}},
                    },
                )
                result["alive_after_rpcs"] = client.process.poll() is None
            result["outcome"] = "completed_protocol_probe"
        except (
            TimeoutError,
            RuntimeError,
            BrokenPipeError,
            json.JSONDecodeError,
        ) as error:
            result["outcome"] = "blocked"
            result["reason"] = str(error)
        finally:
            result["exchanges"] = client.events
            client.close()
            result["exit_code_after_disposal"] = client.process.returncode
            result["owned_process_group_disposed"] = True
        return result


def cli_projection(raw):
    """Allowlist evidence before disk persistence; do not retain arbitrary runtime text."""
    events = []
    for line in raw.splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(value, dict):
            continue
        kind = value.get("type")
        row = {"type": kind}
        if isinstance(value.get("thread_id"), str):
            row["thread_id"] = value["thread_id"]
        if kind == "thread.started":
            row["thread_id"] = value.get("thread_id", value.get("id"))
        if kind == "turn.completed":
            usage = value.get("usage", {})
            row["usage"] = {
                key: count
                for key, count in usage.items()
                if key in ("input_tokens", "output_tokens", "cached_input_tokens")
                and isinstance(count, int)
            }
        item = value.get("item", {})
        if isinstance(item, dict):
            row["item_type"] = item.get("type")
            if item.get("type") == "agent_message":
                answer = item.get("text", "").strip()
                row["arithmetic_answer_matches"] = answer == "42"
                if answer == "42":
                    row["answer"] = "42"
        if kind in ("error", "turn.failed") or (
            isinstance(item, dict) and item.get("type") == "error"
        ):
            # Retain failure categories, never arbitrary provider messages/URLs.
            row["error_categories"] = error_categories(json.dumps(value))
        events.append(row)
    return events


def error_categories(text):
    """Project error diagnostics into fixed, secret-free categories."""
    text = text.lower()
    categories = []
    if "model" in text and any(
        term in text
        for term in ("not found", "unknown", "not supported", "unsupported", "invalid")
    ):
        categories.append("model_selection_error")
    if any(
        term in text for term in ("authentication", "unauthorized", "sign in", "login")
    ):
        categories.append("authentication_error")
    if any(term in text for term in ("timeout", "timed out", "deadline")):
        categories.append("timeout")
    if any(term in text for term in ("network", "connection", "transport")):
        categories.append("transport_error")
    return categories or ["unclassified_runtime_error"]


def unexpected_tool_observed(events):
    """Reject unknown item categories instead of assuming they cannot execute tools."""
    for row in events:
        is_item = row.get("type") in ("item.started", "item.completed") or row.get(
            "method"
        ) in ("item/started", "item/completed")
        item_type = row.get("item_type")
        if (
            is_item or item_type is not None
        ) and item_type not in NON_TOOL_ARITHMETIC_ITEM_TYPES:
            return True
    return False


def cli_accepted(events, exit_code, timed_out):
    """Keep transport exit, structured completion, answer, and tool behavior separate."""
    threads = [
        index for index, row in enumerate(events) if row["type"] == "thread.started"
    ]
    starts = [
        index for index, row in enumerate(events) if row["type"] == "turn.started"
    ]
    ends = [
        index for index, row in enumerate(events) if row["type"] == "turn.completed"
    ]
    answers = [
        index
        for index, row in enumerate(events)
        if row["type"] == "item.completed"
        and row.get("item_type") == "agent_message"
        and row.get("arithmetic_answer_matches")
    ]
    if not all(len(indices) == 1 for indices in (threads, starts, ends, answers)):
        return False
    identity = events[threads[0]].get("thread_id")
    return (
        exit_code == 0
        and not timed_out
        and isinstance(identity, str)
        and bool(identity)
        and threads[0] < starts[0] < answers[0] < ends[0]
        and all(row.get("thread_id", identity) == identity for row in events)
        and not any(row["type"] in ("turn.failed", "error") for row in events)
        and not unexpected_tool_observed(events)
    )


def live_cli(name, model, timeout):
    """Run one expressly admitted arithmetic turn in a disposable host directory."""
    executable = shutil.which(name)
    if executable is None:
        raise RuntimeError(f"{name} not found; no fixture fallback")
    with tempfile.TemporaryDirectory(prefix="native-arithmetic-") as directory:
        argv = [executable]
        if name == "traecli":
            argv += ["--no-daemon"]
        argv += [
            "exec",
            "--json",
            "--ephemeral",
            "--ignore-user-config",
            "--ignore-rules",
            "--skip-git-repo-check",
            "--sandbox",
            "read-only",
            "--color",
            "never",
            "-m",
            model,
            "-c",
            "project_doc_max_bytes=0",
            "-c",
            "model_reasoning_effort=low",
            "-c",
            'approval_policy="never"',
        ]
        for flag in FEATURES + ("shell_tool", "unified_exec"):
            argv += ["-c", f"features.{flag}=false"]
        if name == "traecli":
            for tool in TOOLS:
                argv += ["--disallowed-tool", tool]
        argv += [PROMPT]
        run = bounded(argv, timeout, cwd=directory)
        events = cli_projection(run.pop("stdout"))
        completed = any(row["type"] == "turn.completed" for row in events)
        answer = any(row.get("arithmetic_answer_matches") for row in events)
        unexpected_tool = unexpected_tool_observed(events)
        return {
            "classification": "REAL_PROVIDER_ATTEMPT",
            "kind": name,
            "model": model,
            "binary_sha256": digest(Path(executable).resolve()),
            "prompt": PROMPT,
            "boundary": "trusted_installed_cli_on_host_read_only_tools_disabled",
            "insula": False,
            "credentials_read_by_lab": False,
            "deadline_seconds": timeout,
            "hard_output_token_limit": None,
            "hard_billing_limit": None,
            "max_provider_turns_requested": 1,
            "projected_events": events,
            "structured_turn_completed": completed,
            "expected_answer_observed": answer,
            "unexpected_tool_observed": unexpected_tool,
            "accepted": cli_accepted(events, run["exit_code"], run["timed_out"]),
            **run,
        }


def rpc_projection(events):
    """Retain host RPC identity and arithmetic results, never broad responses."""
    rows = []
    for event in events:
        message = event["message"]
        row = {"direction": event["direction"]}
        for key in ("id", "method"):
            if key in message:
                row[key] = message[key]
        params = message.get("params", {})
        for key in ("threadId", "turnId", "itemId"):
            if key in params:
                row[key] = params[key]
        if "turn" in params:
            row["turn"] = {key: params["turn"].get(key) for key in ("id", "status")}
        item = params.get("item", {})
        if item:
            row["item_type"] = item.get("type")
        if item.get("type") == "agentMessage":
            row["arithmetic_answer_matches"] = item.get("text", "").strip() == "42"
        if message.get("method") == "item/agentMessage/delta":
            row["delta_matches_42"] = params.get("delta", "").strip() == "42"
        result = message.get("result", {})
        if isinstance(result, dict):
            if "thread" in result:
                row["thread_id"] = result["thread"].get("id")
            if "turn" in result:
                row["turn"] = {key: result["turn"].get(key) for key in ("id", "status")}
            if "usage" in result:
                row["usage"] = result["usage"]
        if "error" in message:
            row["error_code"] = message["error"].get("code")
            row["error_categories"] = error_categories(json.dumps(message["error"]))
        rows.append(row)
    return rows


def live_app_server(model, timeout):
    """One trusted host app-server turn; the lab never opens the authentication store."""
    executable = shutil.which("codex")
    if executable is None:
        raise RuntimeError("codex required; no fixture fallback")
    args = [executable, "app-server", "--stdio", "-c", "project_doc_max_bytes=0"]
    for flag in FEATURES + ("shell_tool", "unified_exec"):
        args += ["-c", f"features.{flag}=false"]
    with tempfile.TemporaryDirectory(prefix="native-app-arithmetic-") as directory:
        client = Rpc(args, timeout)
        result = {
            "classification": "REAL_PROVIDER_ATTEMPT",
            "kind": "codex-app-server",
            "model": model,
            "binary_sha256": digest(Path(executable).resolve()),
            "boundary": "trusted_installed_host_app_server_read_only_tools_denied",
            "insula": False,
            "credentials_read_by_lab": False,
            "global_config_changed_by_lab": False,
            "ambient_runtime_config_inherited": True,
            "deadline_seconds": timeout,
            "hard_output_token_limit": None,
            "hard_billing_limit": None,
            "max_provider_turns_requested": 1,
            "prompt": PROMPT,
            "accepted": False,
        }
        try:
            initialize = client.request(
                "initialize",
                {
                    "clientInfo": {"name": "corenius_learning_lab", "version": "0.1"},
                    "capabilities": {"experimentalApi": True},
                },
            )
            if "error" in initialize:
                raise RuntimeError("initialize refused")
            client.send("initialized", notification=True)
            started = client.request(
                "thread/start",
                {
                    "model": model,
                    "cwd": directory,
                    "approvalPolicy": "never",
                    "sandbox": "read-only",
                    "ephemeral": True,
                    "dynamicTools": [],
                    "baseInstructions": "You are an arithmetic assistant. Never use tools.",
                    "developerInstructions": "Answer the arithmetic question with digits only.",
                },
            )
            if "error" in started:
                raise RuntimeError("thread/start refused")
            thread_id = started["result"]["thread"]["id"]
            result["thread_id"] = thread_id
            result["resolved_model"] = started["result"].get("model")
            result["resolved_model_provider"] = started["result"].get("modelProvider")
            # The request response proves admission; notifications may arrive before it.
            turn = client.request(
                "turn/start",
                {
                    "threadId": thread_id,
                    "model": model,
                    "effort": "low",
                    "input": [{"type": "text", "text": PROMPT, "text_elements": []}],
                },
            )
            if "error" in turn:
                raise RuntimeError("turn/start refused")
            turn_id = turn["result"]["turn"]["id"]
            result["turn_id"] = turn_id
            terminal = None
            while terminal is None:
                for event in client.events:
                    value = event["message"]
                    params = value.get("params", {})
                    if (
                        value.get("method") == "turn/completed"
                        and params.get("threadId") == thread_id
                        and params.get("turn", {}).get("id") == turn_id
                    ):
                        terminal = params["turn"]
                        break
                if terminal is None:
                    value = client.receive()
                    if "id" in value and "method" in value:
                        refusal = {
                            "jsonrpc": "2.0",
                            "id": value["id"],
                            "error": {"code": -32601, "message": "lab denies tools"},
                        }
                        client.process.stdin.write(
                            (json.dumps(refusal) + "\n").encode()
                        )
                        client.process.stdin.flush()
            result["terminal_status"] = terminal.get("status")
            result["alive_after_turn_completed"] = client.process.poll() is None
            projected = rpc_projection(client.events)
            result["expected_answer_observed"] = any(
                event.get("arithmetic_answer_matches")
                and event.get("method") == "item/completed"
                and event.get("threadId") == thread_id
                and event.get("turnId") == turn_id
                for event in projected
            )
            result["unexpected_tool_observed"] = unexpected_tool_observed(projected)
            result["accepted"] = (
                terminal.get("status") == "completed"
                and result["expected_answer_observed"]
                and result["alive_after_turn_completed"]
                and not result["unexpected_tool_observed"]
            )
            result["outcome"] = "completed_provider_attempt"
        except (
            TimeoutError,
            RuntimeError,
            BrokenPipeError,
            json.JSONDecodeError,
            KeyError,
        ) as error:
            result["outcome"] = "blocked"
            result["reason"] = str(error)
        finally:
            result["projected_exchanges"] = rpc_projection(client.events)
            client.close()
            result["exit_code_after_disposal"] = client.process.returncode
            result["owned_process_group_disposed"] = True
        return result


def schema(timeout):
    """Generate actual installed app-server schemas within the offline boundary."""
    executable = shutil.which("codex")
    if executable is None or shutil.which("bwrap") is None:
        raise RuntimeError("codex and bwrap required; no host fallback")
    with tempfile.TemporaryDirectory(prefix="native-schema-") as directory:
        root = Path(directory)
        run = bounded(
            insula(
                executable,
                root,
                [
                    "app-server",
                    "generate-json-schema",
                    "--experimental",
                    "--out",
                    "/workspace/schema",
                ],
            ),
            timeout,
        )
        files = root / "workspace/schema"
        selected = {}
        for path in sorted(files.rglob("*.json")):
            if path.stem in (
                "ThreadStartParams",
                "ThreadResumeParams",
                "ThreadReadParams",
                "TurnStartParams",
                "TurnInterruptParams",
                "TurnCompletedNotification",
            ):
                value = json.loads(path.read_text())
                selected[path.name] = {
                    "sha256": digest(path),
                    "required": value.get("required", []),
                    "properties": sorted(value.get("properties", {})),
                }
        return {
            "classification": "MODEL_FREE_ACTUAL_SCHEMA",
            "run": run,
            "schema_files": len(list(files.rglob("*.json"))),
            "selected": selected,
        }


def catalog(timeout):
    """Read the native Trae model catalog; a catalog is not an entitlement proof."""
    executable = shutil.which("traecli")
    if executable is None:
        raise RuntimeError("traecli required; no fixture fallback")
    run = bounded([executable, "models", "--json"], timeout)
    raw = run.pop("stdout")
    try:
        values = json.loads(raw) if run["exit_code"] == 0 else []
    except json.JSONDecodeError:
        values = []
    if isinstance(values, dict):
        values = values.get("models", [])
    names = [
        value["name"]
        for value in values
        if isinstance(value, dict) and isinstance(value.get("name"), str)
    ]
    return {
        "classification": "MODEL_FREE_NATIVE_CATALOG",
        "model_names": names,
        "entitlement_verified": False,
        "run": run,
        "outcome": "completed_catalog_probe" if names else "blocked",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "mode",
        choices=(
            "inventory",
            "catalog",
            "app-server",
            "acp",
            "schema",
            "live-cli",
            "live-app-server",
        ),
    )
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=20)
    parser.add_argument("--runtime", choices=("traecli", "codex"))
    parser.add_argument("--model")
    parser.add_argument("--authorize-model-spend", action="store_true")
    args = parser.parse_args()
    if not 1 <= args.timeout <= 60:
        parser.error("timeout must be between 1 and 60 seconds")
    if args.out.exists():
        parser.error("output exists; choose a new file to preserve evidence")
    if args.mode == "live-cli" and not (
        args.authorize_model_spend and args.model and args.runtime
    ):
        parser.error(
            "live-cli requires --runtime, --model, and --authorize-model-spend"
        )
    if args.mode == "live-app-server" and not (
        args.authorize_model_spend and args.model
    ):
        parser.error("live-app-server requires --model and --authorize-model-spend")
    temp = os.environ.get("TMPDIR")
    if not temp:
        parser.error("invoke through python3 tools/temp_root.py; owned TMPDIR required")
    try:
        root = temporary_root(temp)
    except (ValueError, OSError) as error:
        parser.error(f"temporary root: {error}")
    os.environ.update(TMPDIR=str(root), TMP=str(root), TEMP=str(root))
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    if args.mode == "inventory":
        result = inventory()
    elif args.mode == "schema":
        result = schema(args.timeout)
    elif args.mode == "catalog":
        result = catalog(args.timeout)
    elif args.mode == "live-cli":
        result = live_cli(args.runtime, args.model, args.timeout)
    elif args.mode == "live-app-server":
        result = live_app_server(args.model, args.timeout)
    else:
        result = protocol(args.mode, args.timeout)
    result["observed_at_utc"] = started
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("x") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(
        json.dumps(
            {
                "output": str(args.out.resolve()),
                "classification": result["classification"],
                "outcome": result.get("outcome"),
                "accepted": result.get("accepted"),
            }
        )
    )
    if result.get("outcome") == "blocked" or result.get("accepted") is False:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
