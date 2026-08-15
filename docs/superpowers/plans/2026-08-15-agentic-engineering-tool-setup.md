# Agentic Engineering Tool Setup Plan

## Goal

Install and smoke-test the local-only foundation tools from the agentic
engineering reference architecture:

- `kata` for durable intent/task tracking;
- `roborev` for independent code-review verification; and
- `agentsview` for session, token, and activity observability.

## Scope

This setup is intentionally local-only. It must not enable production or remote
authority by default.

## Initial install policy

- Prefer documented Homebrew or package-manager installs over shell-piped
  installer scripts when available.
- Do not write Git hooks during the initial install.
- Do not configure GitHub App, GitHub sync, federation, remote daemons, hosted
  mode, PostgreSQL sync, provider tokens, or launchd autostart.
- Do not let any external tool rewrite Harp's `AGENTS.md`; Harp owns its own
  operating guidance.
- Keep tool state outside the repository unless a repo binding is explicitly
  approved.

## Steps

1. Inspect whether `kata`, `roborev`, and `agentsview` are already installed.
2. Install missing foundation tools with the least-authority documented method.
3. Run version/help smoke tests.
4. Confirm no hooks were added and no persistent daemons were started.
5. Report the exact commands, installed versions, and any deferred setup.

## Setup result

- Installed `kata` with Homebrew: `v0.14.3`.
- Installed `agentsview` with Homebrew cask: `v0.40.1`.
- Installed `roborev` from the reviewed GitHub release installer:
  `v0.64.0` at `~/.local/bin/roborev`.
- `kata version`, `agentsview --version`, and `roborev version` are the safe
  no-daemon smoke checks.
- `kata list` can start a local Kata daemon for the bound project. It was used
  once to verify the `harp` project binding and then stopped.
- `agentsview projects` can start the AgentsView server and initial session
  sync. It was attempted once, then the startup process was stopped manually.
- `roborev status` can start the roborev daemon. It was attempted once, then
  the daemon was stopped.

## Deferred setup

- `kata init --with-agents`.
- Kata federation, GitHub sync, remote daemon, hosted mode, and PostgreSQL.
- roborev `init`, Git hooks, GitHub App, PR comments, CI polling, and daemon.
- AgentsView daemon, remote access, PostgreSQL sync, and background autostart.
- Forge and Ghosthub installation/configuration.

## Verification

After setup:

```sh
command -v kata roborev agentsview
kata --version
roborev --version
agentsview --version
git status --short --branch
```

If any tool lacks a `--version` command, use its documented `--help` command
and record that behavior.
