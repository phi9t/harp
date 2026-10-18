# 02: Operate the durable runtime through Harp’s CLI

Status: done
Owner: Codex
Branch: codex/harp-durable-runtime
Worktree: harp-corenius-consolidation/harp
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Explicitly start, inspect and stop a project-scoped instance of the Corenius-adapted Python durable runtime through Harp. Run a deterministic managed activity and retain its state for a later invocation.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] The public CLI executes a complete deterministic workflow using the existing adapted runtime, records its result, and reports its durable identity and outcome.
- [ ] Stopping and reopening the same state recovers accepted results without repeating completed activities. A concurrent competing runtime for that state is refused.
- [ ] Status distinguishes pending, running, completed and blocked work and exposes useful logs without exposing private runtime credentials.
- [ ] The runtime validates the supported local SQLite/filesystem and execution contract before dispatch. Missing prerequisites are reported before launching an activity.
- [ ] Explicit runtime lifetime requires no installed always-on service and no dependency on another checkout, the supplied ZIP or a private historical environment.

## Blocked by

None (can start immediately).

## Scope

Reuse the existing replay and guardian mechanisms. This slice demonstrates a managed deterministic activity, not a qualified coding provider or an added scheduler.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

### Implementation and fresh evidence

- Public `harp durable` replaces its process with the same-checkout Python
  interface, preserving argv, streams, signals and exit status. It controls the
  import path rather than depending on another checkout.
- Foreground `start`, token-addressed `stop`, `status`, `events`, `check`,
  deterministic `submit` and `cancel` use the selected runtime and guardian.
  Capacity is bounded to one through four. Trusted demonstration activities use
  the existing process-group backend; this does not qualify coding isolation.
- State admission requires private owned local Linux storage and rejects linked
  database, sidecar, lock, attempt and instance targets before SQLite opens them.
- Independent review found and repaired a replacement-token overwrite after lock
  release, and linked database targets. Review of the repairs found no remaining
  required defect in this slice.
- Five backend lifecycle tests passed on devx with existing fixed SQLite 3.51.3:
  retained completion without another attempt, competing controller refusal,
  explicit stop/reopen, interrupted activity cleanup/reopen, and private-storage
  and linked-target refusal. Tests use real subprocesses and durable state.
- `mise run verify -- --base ea50a11f` passed locally: repository verification,
  verification dispatch tests, 467 library tests, 16 CLI tests, five new bridge
  process tests and document checks. Linux backend tests explicitly skip on
  macOS; the separate devx run above supplies execution evidence.
- Remaining acceptance: run those lifecycle tests through the compiled Linux
  Harp executable, rather than Python entry alone. The offline devx build refuses
  missing locked `pulldown-cmark-escape`. Cache authorization has been requested
  under the prerequisite rule; no dependency hydration was performed. The test
  runner accepts `HARP_DURABLE_BIN` for this composed public-interface check.

Example trusted demonstration (Linux with the fixed SQLite library available):

```sh
harp durable --state /absolute/private/project-state submit example --value 7
harp durable --state /absolute/private/project-state start --until-idle
harp durable --state /absolute/private/project-state status example
```

The long-lived foreground form omits `--until-idle`; another invocation requests
`stop` for that exact runtime instance. State persists after shutdown. No service
is installed. Coding admission, providers, verification and landing remain later
tickets, and this ticket remains incomplete until composed Linux acceptance.

### Resolved prerequisite and composed acceptance

Revalidated using the pinned Rust 1.92.0 on devx: the locked offline Harp build
still refuses missing `pulldown-cmark-escape`. Existing default registry source,
archive and index caches do not contain it; there is no live build to wait on.
The owner-facing Cargo-cache authorization request is pending. No cache was
hydrated and no package pins changed. Next action on authorization: populate a
task-owned cache with the locked dependencies, build the Linux Harp executable,
then run all five lifecycle tests with `HARP_DURABLE_BIN` selecting that binary.
The owner subsequently authorized all necessary devx setup. A task-owned Cargo
home was populated from existing cached bytes first, then missing locked crates
were fetched. No dependency pins changed or service was installed.

The pinned Rust 1.92.0 offline locked Harp build passed on devx. All five lifecycle
tests then passed through the compiled public Harp executable, using
`HARP_DURABLE_BIN=/data03/harp-runtime-integration-philip-yang/build/debug/harp`
and the fixed SQLite 3.51.3 library. This supersedes the blocked precondition above
and completes the composed CLI acceptance. Ticket 03 is now unblocked.
