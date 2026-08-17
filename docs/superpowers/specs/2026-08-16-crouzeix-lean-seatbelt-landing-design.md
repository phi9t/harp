# Crouzeix Lean Seatbelt Landing Design

## Objective

Land the reviewed Lean proof-suite infrastructure without overwriting newer
local `master` formalization work, then add a macOS Seatbelt execution gate
before any real Lean proof-suite run is called hermetic.

The immediate target is not to prove Crouzeix, not to resume CPFR-032, and not
to create training data. The target is a clean landing path plus a precise
CPFR-L007 contract for real Lean execution on
`formalization/mathematical_foundations`.

## Current State

`feat/crouzeix-proof-reproduction` contains reviewed Lean proof-suite work:

- `CPFR-L001`: runtime-lock and suite-manifest validators.
- `CPFR-L002`: Harp-owned fake Lean/Lake fixtures and source inventory checks.
- `CPFR-L003`: declared-write-surface runner and typed receipts.

`CPFR-L003` is intentionally not an operating-system sandbox. It materializes
declared files into a fresh execution root, blocks declared absolute write
channels in `argv` and `env`, validates pre/post inventories, and records
`local_process_no_os_sandbox` for that limitation.

Local `master` and `feat/crouzeix-proof-reproduction` have diverged. Current
`master` already contains Lean/formalization material, including
`formalization/mathematical_foundations`, that must not be deleted or replaced
by landing the feature branch wholesale.

The reusable macOS sandbox guidance is now installed at:

```text
~/.agents/skills/macos-seatbelt-sandbox/SKILL.md
```

## Non-Goals

- No direct merge of the full divergent feature branch into `master`.
- No deletion of the feature worktree or branch until a clean integration
  branch is verified and local `master` is advanced.
- No real Lean/Lake execution before CPFR-L007.
- No claim that CPFR-L003 is hermetic macOS execution.
- No CPFR-032, CPFR-033, live provider, proof-agent, or DGM execution.
- No generated proof-training examples or datasets.
- No repository-wide license decision.

## Design Decision

Use a clean integration branch from current `master`. Cherry-pick only the
reviewed Lean proof-suite commits from `feat/crouzeix-proof-reproduction`, then
verify and fast-forward `master` from that integration branch.

Do not try to rebase or merge the full feature branch. The full branch carries
older proof-reproduction state that conflicts with newer `master`
formalization work.

## Phase 1: Clean Landing Branch

Create a temporary integration branch from local `master`, then cherry-pick the
reviewed Lean-suite commits in order:

```text
750891d docs(crouzeix): design hermetic lean proof suite
8fb1139 docs(crouzeix): plan hermetic lean proof suite
5750bc5 feat(crouzeix): validate lean proof suite contracts
d8c7fe7 fix(crouzeix): require locked lean command first
95556ac fix(crouzeix): harden lean suite lock validation
b09e782 fix(crouzeix): reject noncanonical lean suite paths
d363fdb feat(crouzeix): add lean proof suite fixtures
6c6e849 feat(crouzeix): run lean proof suite fixtures
e4a1718 fix(crouzeix): bind lean suite execution roots
5733c58 fix(crouzeix): block drifted lean suite runtimes
795c9dd docs(crouzeix): narrow lean suite runner boundary
34978ca fix(crouzeix): keep lean suite write roots closed
bd5e9f6 fix(crouzeix): block embedded lean suite write paths
2f791b7 fix(crouzeix): bind lean suite receipt modules
```

Conflict resolution must preserve current `master` material unless the
conflict is specifically in the reviewed Lean-suite surface. In particular,
preserve:

- `formalization/mathematical_foundations`;
- existing Lean evidence and scripts;
- current `master` Crouzeix formal-validation records;
- current source/evidence manifests unless a Lean-suite commit intentionally
  updates the tracked payload receipt.

The untracked primary-checkout file
`docs/superpowers/plans/2026-08-16-crouzeix-end-to-end-continuation-prompt.md`
must not be lost. It should either remain untracked or be handled by a
separate owner decision.

## Phase 2: Verification And Landing

The integration branch must pass:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
python3 -m json.tool labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json
python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
cargo run -q -p harp -- repository verify
mise run verify
```

If `repository verify` reports a stale `docs/import-receipt.md` digest, refresh
only the digest after all other tracked changes settle, then rerun the
verifier.

Only after the integration branch is verified may local `master` be
fast-forwarded. After `master` is advanced, remove the old
`crouzeix-proof-reproduction` worktree and delete the old local feature branch.
Do not push.

## Phase 3: CPFR-L007 Seatbelt Execution Gate

CPFR-L007 implements real macOS sandboxing for Lean and other code execution.
It must use the global `macos-seatbelt-sandbox` skill and the Codex reference
model:

- `codex-rs/sandboxing/src/seatbelt.rs` in the sibling Codex checkout.
- `codex-rs/sandboxing/src/seatbelt_base_policy.sbpl`.
- `codex-rs/sandboxing/src/seatbelt_network_policy.sbpl`.
- `codex-rs/sandboxing/src/manager.rs`.
- `codex-rs/core/src/exec.rs`.

The implementation model is a policy compiler:

1. Pin `/usr/bin/sandbox-exec`; never resolve it from `PATH`.
2. Generate SBPL from a typed Harp policy model.
3. Start with `(deny default)`.
4. Add only vetted platform allowances needed for Lean/Lake.
5. Pass read and write roots through parameters, not raw path interpolation.
6. Deny network by default.
7. Protect metadata names inside otherwise writable roots: `.git`, `.agents`,
   `.codex`, `.trae`, and Harp control directories.
8. Record policy text digest, parameter map, sandbox executable identity,
   argv, cwd, env, stdout/stderr digests, exit status, and denial evidence.

CPFR-L007 must prove the sandbox with fake tools before any real Lean run:

- hardcoded absolute outside write is denied;
- `..` traversal write is denied;
- metadata directory creation/write is denied;
- network socket creation is denied;
- read-only source mutation is denied;
- extra output outside declared write roots is denied.

If `/usr/bin/sandbox-exec` is missing, unavailable, or cannot apply the policy,
the result is `blocked`; there is no silent fallback to CPFR-L003.

## Phase 4: Real Lean Validation

Only after CPFR-L007 passes may Harp run real Lean against:

```text
formalization/mathematical_foundations
```

The first real target is a repeatable local validation, not a Crouzeix proof
claim. The run must inspect and receipt:

- `lean-toolchain`;
- `lakefile.toml`;
- `lake-manifest.json`;
- Lean/Lake executable identities;
- package/cache roots;
- source inventory;
- Seatbelt policy digest and parameters;
- stdout/stderr;
- exit status.

Outcomes are:

- `passed`: Seatbelted Lean/Lake command succeeds and all receipts validate.
- `failed`: Lean/Lake executes under Seatbelt and the proof/build fails.
- `blocked`: toolchain, package cache, Seatbelt, permissions, or dependency
  setup is missing before proof correctness can be judged.

Do not call the result hermetic if Seatbelt is bypassed.

## Review And Acceptance

The integration landing is accepted when:

- the reviewed Lean-suite commits are present on the clean integration branch;
- current `master` formalization material is preserved;
- full verification passes;
- local `master` is fast-forwarded without push;
- the old feature worktree and branch are removed only after landing.

The real Lean validation lane is accepted only after:

- CPFR-L007 is implemented and reviewed;
- fake malicious tools prove Seatbelt containment;
- `formalization/mathematical_foundations` is run under Seatbelt;
- the result is recorded as `passed`, `failed`, or `blocked` with receipts.

## Risks

- Cherry-picking can accidentally reintroduce old proof-reproduction state over
  newer `master` formalization work. Mitigation: keep the integration branch
  small and preserve `master` on conflicts.
- Seatbelt is path-mediation, not a container or mount namespace. Mitigation:
  state that CPFR-L007 provides macOS Seatbelt containment, not a private
  filesystem graph.
- Real Lean may be blocked by missing toolchains or package caches. Mitigation:
  record a typed `blocked` result and do not install or fetch implicitly.
