# Ticket 17: standalone ticket-source evidence

Scope: ticket 17 at base `ff66dd24df0f90ae85c3f1dab286cd3b0c156c8c`.
Worktree: `/data02/home/philip.yang/workspace/corenius-codex-collab-ops-refresh`.
Branch: `codex/collab-ops-refresh`.
Classification: gated implementation under the approved Collab Ops specification.
Test seam: subprocess calls to the public collab CLI against real disposable Git repositories.

## Result and adapter contract

`init --ticket-source standalone|corenius` persists the selected source.
The default remains `standalone` for existing callers and databases without the new configuration field.
Both sources read committed ticket bytes through `TicketSource.ticket(identity, commit)`.
Both enumerate committed identities through `list_tickets(commit)` and apply admission rules through `require_ready(metadata, commit)`.
Malformed committed tickets remain visible as errors.
Dependencies are checked against the specified trunk commit.
The state store retains Attempt identity; the adapter creates no second ledger.

The standalone source retains `.tickets/ID.md`, the narrow front-matter contract, byte hashes, and ticket closure inside the Integration commit.
The Corenius source lists `effort:number` identities in `status` and `ready`.
`open-attempt effort:number` and the existing `--effort` form use tracker claims.
Corenius ticket creation directs the operator to `tracker.py new`.

In-tree Corenius admission refuses an effort assigned to a different Integration branch or any Record branch before invoking tracker writes.
The error directs legacy effort routing to the optional effort-record module.
Real disposable external worktrees prove that refusal preserves both ticket contents and foreign workspace cleanliness.

## Red and green evidence

The following command first exited 1 because `init` did not accept `--ticket-source`:

```text
python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_refresh_source.RefreshSourceTests.test_standalone_source_is_selected_at_init_and_runs_dependency_lifecycle
```

After configuration support, the same command exited 0: one test passed.
It proves dependency refusal, dependency landing and closure, subsequent readiness, and reservation by an active Attempt.

The following command first exited 1 because `status` exposed standalone tickets instead of the selected Corenius source:

```text
python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_refresh_source.RefreshSourceTests.test_corenius_source_lists_tracker_tickets_and_opens_qualified_identity
```

After adapter enumeration and admission glue, the same command exited 0: one test passed.
The test proves blocked and resolved visibility, readiness, qualified admission, tracker owner/status, and active Attempt visibility.

The following command first exited 1 because Corenius `new` wrote a hidden standalone ticket:

```text
python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_refresh_source.RefreshSourceTests.test_corenius_source_directs_ticket_creation_to_the_tracker
```

After correction, the same command exited 0: one test passed, with no `.tickets` directory created.

Each of the following commands first exited 1 because the tracker modified the disposable foreign record workspace before collab failed:

```text
python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_refresh_source.RefreshSourceTests.test_corenius_claim_refuses_foreign_integration_before_touching_other_worktree
python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_refresh_source.RefreshSourceTests.test_corenius_claim_refuses_record_branch_before_touching_record_worktree
```

After the respective preflight refusals, each command exited 0: one test passed per command.
The tests assert unchanged foreign ticket bytes, a clean foreign worktree, and no allocated Attempt.

The final focused command exited 0: ten tests passed in 6.702 seconds.

```text
python3 tools/temp_root.py -- env PYTHONPATH=/data02/home/philip.yang/workspace/corenius-codex-collab-ops-complete/tools/collab python3 -m unittest tools.collab.tests.test_refresh_source
```

The original standalone collab suite passed unchanged: 70 tests in 41.468 seconds before the routing preflight additions.

```text
python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab
```

After the routing additions, the original suite passed again: 70 tests in 40.524 seconds.

```text
python3 tools/temp_root.py -- env PYTHONPATH=/data02/home/philip.yang/workspace/corenius-codex-collab-ops-complete/tools/collab python3 -m unittest tools.collab.tests.test_collab
```

Compilation and `git diff --check` passed.
The implementation adds no packed statements or toolset lines longer than 120 columns.
The lead owns the repository gate and independent review; this report makes no aggregate acceptance claim.

## Claim routing

The initial ticket 10 and 17 tracker claims selected the existing effort checkout instead of the isolated copies.
The implementing agent stopped tracker writes and preserved those claims at the lead's instruction.
The exact commands and foreign files are recorded in `10-report.md`.
The routing regressions and preflight refusal above address the discovered safety defect for in-tree sources.

## For later tickets

Legacy record-branch routing remains an explicit optional-module operation.
The in-tree source never silently adopts or changes a foreign record checkout.
