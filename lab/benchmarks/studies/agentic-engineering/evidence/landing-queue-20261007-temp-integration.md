# Combined readiness/controller temporary contract

Recorded: 2026-10-07T08:31:10.428001+00:00. Readiness input: `ca97624`.
Controller overlay SHA-256: `95a4ed7cfbd2911f09f4cd48656ca4fd29d289ab6521cbd8e8dae95fd27a1546`.
Integration adopted the existing stronger controller contract under the user’s review-and-integration authorization on 2026-10-07.

## Decision and conflict reconciliation

Host temporary roots remain absolute, owned and outside the system temporary
mount. Capsule `TMPDIR` remains fixed to private `/run/corenius-tmp` tmpfs;
`TMUX_TMPDIR` uses that same private root and `TMPPREFIX` its `zsh` prefix.
Retained worker output/session files are mounted separately at `/home/bench/run`.
Worker launches and resumes preserve readiness runtime selection, isolated Git
workspaces, access profiles, command arguments, acceptance and exact-tip landing.
No host temporary directory is projected to satisfy capsule temporary allocation.

Five textual conflict files: results.py retains both readiness and controller
schemas; land.py retains readiness guarded landing/statuses and checked owned
temporary worktrees; sqlite_baseline.py retains readiness private run creation;
test_sqlite_baseline.py retains test-owned fixture directories; worker.py retains
all readiness runtime/interactive/access behavior with adapted capsule paths.
Controller import-time temporary policy blocks remain where applied cleanly.

The older readiness scanner now delegates to the stronger shared source scanner.
No former private sandbox path allowlist survives. One exact historical terminal
display line in test_lead_launch.py is exempt; operational path literals, even in
that same file, remain subject to the ban. Scanner imports remain argument-safe.

Two imported whitespace errors are corrected: empty ticket06 Blocked by header,
and controller_mode.py final blank line. EFFORT status and authority describe the
completed deterministic subset, active ticket03 and historical interview boundary.
Ticket03 remains claimed and its acceptance remains unchecked.

## Causal proof and qualification boundary

Actual fake-runtime capsule-launch regression failed before path adaptation:
worker exited64, with no candidate/output. After adaptation it passed, proving
TMPDIR/TMUX_TMPDIR/TMPPREFIX, actual tempfile allocation and shell heredoc output
remain in private tmpfs, while the host run temporary tree has no heredoc file.
No model was called. Commands/logs and final checks follow after completion.

No full Rust gate, live-provider trial or model spend is claimed here. Primary
verification runs the frozen Rust sources separately; crate bytes imported from
the controller snapshot are unchanged. Protected Python test restoration is
qualified for deterministic-reference grading, not hostile candidate code that
can modify Python interpreter state. Current readiness and ticket03 limits stand.

## Execution correction

The first overlay apply accidentally used the original checkout working directory.
Only that exact patch was immediately reversed. Verification found original HEAD
`6b67eafe573f763af65cc78bcfb13352194c4601`, empty tracked/index diffs, and only its
previous .claude/worktrees and transcript untracked paths. No reset/clean/ref move
was used. All subsequent integration edits are in this isolated worktree.

## Fresh focused verification

- Actual capsule red/green regression: red exit1 before adaptation; green exit0.
- `TMPDIR=/data02/home/philip.yang/.cbt python3 -m unittest tools.agents.test_worker tools.agents.test_land`: exit0, 120 tests in83.818s.
- `TMPDIR=/data02/home/philip.yang/.cbt python3 -m unittest tools.agents.test_no_host_tmp tools.test_temp_root tools.agents.test_sqlite_baseline tools.insula.test_capsule`: exit0, 36 tests in4.139s.
- `python3 tools/test_sync_agentic_workflow.py`: exit0, 24 tests.
- Controller tracker lint and both shared/delegated source scanners: exit0.
- Python compile checks on reconciled worker/lander/baseline/results/scanner: exit0.
- All23 modified controller crate sources match the frozen controller tree exactly.
- Readiness durable status identifies both remaining tickets14 and15; historical checklists remain unchanged.

First grouped attempt failed on scanner import-time argument handling, a changed
diagnostic expectation and an incorrect nonexistent test-module selection. Scanner
entrypoint and expectation were repaired; final focused checks above passed.
An additional readiness-scenario suite is running independently; no pass is
claimed until it finishes. Primary performs complete gates on the frozen commit.

Retained diagnostic logs (SHA-256):
- `/data02/home/philip.yang/devx/tmp/combined-temp-red.log`: `cb156a79b8667f1f6e20954207bcda6217325cccb350edbbee956803eae63e75`.
- `/data02/home/philip.yang/devx/tmp/combined-worker-land.log`: `8a00956c6ceb8cee4bb4277185c8e7780897caea01156e2a3a1f19dfd8bb2a79`.
- `/data02/home/philip.yang/devx/tmp/combined-temp-fast-green.log`: `d473566b4a4e5384e8d105668b64dfb535437ac6280c1308da2f594c61e6d3b4`.

## Final verification

The completed component gates, original failed native invocation, repair coverage,
and source-provenance limits are recorded in landing-queue-20261007-final.md and
landing-queue-20261007-final-gates.json. Earlier pending statements above describe
the state when this integration record was first written.
