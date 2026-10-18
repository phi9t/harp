# 01: Remove PostgreSQL/Temporal from active builds and gates

Status: blocked
Owner: Codex (service retirement slice)
Worktree: `harp-remove-services` (base `ea50a11f`)
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Build and check the selected local product without PostgreSQL or Temporal services, SDK dependencies or hidden fixture requirements. Preserve recoverable source history and license/evidence records.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [x] Normal build, bootstrap dependency resolution and selected local checks no longer select the abandoned PostgreSQL/Temporal stack.
- [x] User-facing entry points and contributor instructions identify the lightweight Python + SQLite runtime as the target; removed experiment commands do not silently start services.
- [x] Any removal from the imported payload has an explicit provenance disposition. Captured evidence bytes and upstream license records are preserved.
- [ ] Real clean-build/dependency-graph and gate-selection evidence demonstrates the absence of PostgreSQL/Temporal requirements; changing only documentation is insufficient.

## Blocked by

Implementation and the selected local gate are complete. Full clean-build/dependency
resolution evidence awaits the integration candidate on devx: the local offline
Cargo cache lacks pinned `hyper-rustls 0.27.10` (only `0.27.9` is cached).
Do not downgrade the unrelated pin or fetch dependencies as part of this ticket.

## Scope

This independently useful cleanup removes a parked dependency path. It does not require provider execution, a new database service or a rewrite of retained historical evidence.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.


### Implemented slice

- Removed the PostgreSQL/Temporal experiment from the Corenius Cargo workspace,
  removed its source, service gate and historical lab runner from the active
  payload, and pruned only unreachable lock packages. No remaining package record
  (version, checksum or dependency list) changed.
- Declared 49 exact-path retirements in `corenius/HARP_IMPORT.json`, each with the
  retained source blob, mode, SHA256 and disposition. The verifier checks original
  Git bytes, branch/worktree ancestry, undeclared removals and reintroduced retired
  payloads. Historical captures and license records are unchanged.
- The old gate environment flag cannot enable service execution; selecting the
  removed suite reports an unknown suite. Removed only the obsolete Temporal
  launcher temp-root test, retaining the generic temporary-root boundary tests.

### Fresh verification

- `mise run verify -- --plan --base ea50a11f`: selects diff, Corenius preservation,
  import-verifier contract tests and document checks; no Lean or services.
- `mise run verify -- --base ea50a11f`: passed, including 12 import-verifier tests.
- `python3 corenius/tools/agents/test_check.py`: 11 passed, exercising actual gate
  dispatch with harmless command shims, including stale service flag and explicit
  retired-suite selection.
- `cargo +1.95.0 metadata --offline --locked --no-deps --format-version 1
  --manifest-path corenius/Cargo.toml`: passed; members are `corenius` and
  `corenius-sqlite-controller`.
- Exact TOML lock-graph audit against `ea50a11f`: all 256 retained package records
  equal their originals, every dependency resolves within the lock, all packages
  are reachable from those two members, and no PostgreSQL/Temporal package remains.
  Removed 153 unreachable package records.
- Full offline locked Cargo metadata remains blocked by the cache prerequisite
  above. No dependency installation, service startup or provider/model call ran.
- Supplementary `python3.11 corenius/tools/test_temp_root.py`: 8 passed, 4 failed
  from existing macOS socket-length and `/tmp` symlink assumptions. Running the
  exact four cases from base `ea50a11f` reproduced the same two errors and two
  failures against the unchanged temp-root implementation. Linux/devx rerun remains
  explicit; these failures were not removed or weakened for this slice.
