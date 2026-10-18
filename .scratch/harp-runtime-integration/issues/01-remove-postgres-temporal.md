# 01: Remove PostgreSQL/Temporal from active builds and gates

Status: done
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
- [x] Real clean-build/dependency-graph and gate-selection evidence demonstrates the absence of PostgreSQL/Temporal requirements; changing only documentation is insufficient.

## Blocked by

None for the selected Linux operation. The exact Cargo-emitted lock passes the
Linux offline locked workspace check. Full cross-platform metadata still needs
uncached Windows-only `anstyle-wincon 3.0.11`; the macOS cache also lacks pinned
`hyper-rustls 0.27.10`. Those cache limits do not invalidate the completed Linux
check and are not permission to downgrade pins or fetch dependencies.

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
  payload. The final reduced lock is emitted by Cargo, preserving every retained
  version and checksum while removing feature edges enabled only by the retired
  workspace member.
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
- The initial manual TOML closure audit retained 256 package records unchanged.
  This was insufficient: real Cargo resolution on devx rejected the lock as
  stale because feature-dependent edges still reflected the retired workspace
  member. That audit is superseded by the Cargo-emitted lock evidence below.
- Full offline locked Cargo metadata remains blocked by the cache prerequisite
  above. No dependency installation, service startup or provider/model call ran.
- Supplementary `python3.11 corenius/tools/test_temp_root.py`: 8 passed, 4 failed
  from existing macOS socket-length and `/tmp` symlink assumptions. Running the
  exact four cases from base `ea50a11f` reproduced the same two errors and two
  failures against the unchanged temp-root implementation. The subsequent exact
  Linux/devx run passed all 12 cases; these tests were not weakened for this slice.


### Required lock repair

The integration owner ran Cargo 1.95.0 `metadata --offline` diagnostically in the
owned devx `source-01` snapshot. Cargo rewrote the stale lock to its reduced
feature graph before reporting missing Windows-only `anstyle-wincon 3.0.11`.
No dependency download was performed. This follow-up copies those exact emitted
bytes, rather than attempting another manual reconstruction.

- Source: `devx_b200_dbg:/data03/harp-runtime-integration-philip-yang/source-01/corenius/Cargo.lock`.
- The remote `qualification/corenius-before.lock` exactly matches the lock from
  commit `83af4bba` (SHA256
  `47031cad404afad1f811fc8d78f44814900542a409f4751ff5ff4f79405de161`).
- Cargo-emitted lock SHA256:
  `5f256da84fdc5d84ba3dd3086079164d697dfcb2a4e7239dfba179861bbd1539`.
- Parsed comparison against the original import verifies all 249 retained
  package identities, sources, versions and checksums are unchanged. Cargo
  removed seven additional packages and reduced/normalized dependency edges;
  no PostgreSQL/Temporal package remains. The manifest adaptation records the
  new exact lock digest.
- The integration owner verified this exact emitted lock on devx with
  `CARGO_TARGET_DIR=/data03/harp-runtime-integration-philip-yang/build-01 cargo
  +1.95.0 check --offline --locked --manifest-path corenius/Cargo.toml --workspace`:
  passed in 10.20 seconds, checking both retained Linux workspace members and
  pinned `hyper-rustls 0.27.10`. No download or service startup ran.
- The integration owner's exact post-ticket Linux
  `PYTHONDONTWRITEBYTECODE=1 python3 corenius/tools/test_temp_root.py` passed
  12/12 after including the retained benchmark harness in the standalone copy.
- Local follow-up `mise run verify -- --base ea50a11f` passed, including all
  12 preservation boundary tests; offline locked `--no-deps` metadata also passed.
  The full cross-platform metadata cache limit remains explicit above.
