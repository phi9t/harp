# DeepSeek Harness study receipt

Captured: 2026-08-15.

This receipt records local, keyless checks used by the
`knowledge/deepseek_harness/` design-decision study. It is a Harp-authored
receipt, not an upstream artifact.

## Source identity

```text
$ git -C /tmp/harp-deepseek-harness rev-parse HEAD
47f943859bef60e4160492346772ded9b24f765a

$ git -C /tmp/harp-deepseek-harness status --short --branch
## HEAD (no branch)
```

The checked temp source matched the vendored DSH revision recorded under
`evidence/implementations/deepseek_harness/REVISION`.

## Runtime and package manager

```text
$ node --version
v24.13.0

$ corepack pnpm@11.7.0 --version
11.7.0
```

The repository root `package.json` declares `packageManager: pnpm@11.7.0` and
the `dsh` script. A package-manager probe printed:

```text
@deepseek-ai/dsh-root pnpm@11.7.0 has-dsh-script
```

## Dependency installation

Command:

```sh
corepack pnpm@11.7.0 install --frozen-lockfile
```

Result: exit 0.

Important output:

```text
Scope: all 238 workspace projects
✓ Lockfile passes supply-chain policies (1203 entries in 28.3s)
Done in 36s using pnpm v11.7.0
```

Observed warnings:

```text
native/landlock-run/packages/linux-arm64 | [WARN] Unsupported platform ...
native/landlock-run/packages/linux-x64   | [WARN] Unsupported platform ...
[WARN] Failed to create bin ... examples/node_modules/.bin/dsh-acp-demo ...
[WARN] Failed to create bin ... python/sdk-runtime/node_modules/.bin/dsh-jsonrpc-agent ...
```

Interpretation: installation completed on macOS. Linux-only native Landlock
packages were unsupported on this host, and two demo bin links pointed at
unbuilt demo outputs. This receipt does not use those demo bins.

## Targeted tool-runtime smoke

Command:

```sh
corepack pnpm@11.7.0 exec vitest run packages/core/tools/tests/tools.spec.ts
```

Result: exit 0.

Key output:

```text
RUN  v4.1.8 /private/tmp/harp-deepseek-harness

Test Files  1 passed (1)
     Tests  136 passed (136)
  Duration  1.26s
```

The tested file includes tool registration, schema projection, host-callback
exclusion from model-visible schemas, execution, `tools/result` observation,
canonical value handling, presentation metadata, argument errors, output
errors, policy/event hooks, scoped tools, and related tool-runtime behavior.

## What this proves

- The local DSH checkout used for study was at the pinned commit.
- The declared package-manager version was available through Corepack.
- The workspace dependency install completed under the lockfile.
- The core tool-runtime test suite passed locally, giving a concrete no-key
  exercise of the tool registry and execution pipeline discussed in
  `knowledge/deepseek_harness/06_tool_registry_and_policy_pipeline.md`.

## What this does not prove

- No DeepSeek API key or other model-provider credential was used.
- No `dsh web`, `dsh headless`, ACP server, Python SDK, or browser UI run is
  claimed here.
- No benchmark score, DeepSWE run, or leaderboard result is reproduced.
- No platform sandbox guarantee is established. The receipt was produced on
  macOS, and Linux-only Landlock packages were not exercised.
- No HMR, subagent continuation, persistence crash-recovery, or API-gateway
  end-to-end round trip was executed.
