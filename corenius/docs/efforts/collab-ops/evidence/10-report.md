# Ticket 10: refresh implementation evidence

Scope: ticket 10 at base `ff66dd24df0f90ae85c3f1dab286cd3b0c156c8c`.
Worktree: `/data02/home/philip.yang/workspace/corenius-codex-collab-ops-refresh`.
Branch: `codex/collab-ops-refresh`.
Classification: gated implementation under the approved Collab Ops specification.
Test seam: subprocess calls to the public collab CLI against real disposable Git repositories.

## Result

`refresh ATTEMPT --confirm-stopped` validates the stopped worker, worktree identity, clean state, linear history, and unchanged ticket contract.
It preserves the old worktree, branch, Candidate refs, and Check receipt.
It abandons the old Attempt and creates a new Attempt on current trunk.
It replays commits outside the current trunk's history, in their original order.
The new Attempt has no Candidate, Integration commit, or Check receipt.

A failed cherry-pick returns exit code 2 and names the new Attempt, worktree, and failed commit.
The worktree retains conflict markers, index entries, `CHERRY_PICK_HEAD`, and the successfully applied prefix.
The later commits remain unapplied and recoverable from the original branch.
The command does not abort, skip, resolve, or rebase a contribution.

Corenius refresh transfers an unchanged tracker claim to a new Attempt with the same owner.
The tracker ticket remains unchanged.
Check, lane, and metadata locks cover validation, replacement allocation, and replay.
Database transactions finish before each cherry-pick.
Durable events record the original request and the replacement's origin and commit list.

## Red and green evidence

The following literal command first exited 1 because the CLI had no `refresh` command:

```text
python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_refresh_source.RefreshSourceTests.test_refresh_replays_only_contribution_onto_directly_moved_trunk_and_drops_old_receipt
```

After the first implementation, the same command exited 0: one test passed.
It proves direct trunk movement, two contribution commits, preservation of old work and receipts, and refusal to land the unchecked replacement.

The following command first exited 1 with two failures:

```text
python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_refresh_source.RefreshSourceTests.test_refresh_conflict_reports_failure_and_preserves_first_conflict_and_unapplied_suffix tools.collab.tests.test_refresh_source.RefreshSourceTests.test_corenius_refresh_transfers_owned_tracker_claim_to_new_attempt
```

The conflict incorrectly returned exit code 0, and Corenius admission rejected the existing owned claim.
After correction, the same command exited 0: two tests passed.

The following command first exited 1 because refresh tried to replay a direct trunk commit already present in the worker history:

```text
python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_refresh_source.RefreshSourceTests.test_refresh_excludes_trunk_commits_already_present_in_worker_history
```

After excluding commits reachable from current trunk, the same command exited 0: one test passed.

The final focused command exited 0: ten tests passed in 6.702 seconds.
Its liveness dependency was read from the lead's integrated ticket-09 implementation; no foreign file was copied or edited.

```text
python3 tools/temp_root.py -- env PYTHONPATH=/data02/home/philip.yang/workspace/corenius-codex-collab-ops-complete/tools/collab python3 -m unittest tools.collab.tests.test_refresh_source
python3 tools/temp_root.py -- python3 -m py_compile tools/collab/refresh.py tools/collab/ticket_source.py tools/collab/core.py tools/collab/cli.py tools/collab/tests/test_refresh_source.py
python3 tools/temp_root.py -- git diff --check
```

Compilation and diff checks exited 0.
The additional refusal test preserves missing-stop-confirmation, dirty work, and changed-contract cases before abandonment.

## Integration requirements and limits

`core.py` and `cli.py` changes are isolated in a separate glue commit.
Refresh requires the mandatory `liveness.require_worker_stopped` API from ticket 09.
The lead must verify refresh with a real bound live process after integrating the worker-binding schema and CLI.
The lead must adapt allocation recovery for an exact transferred Corenius claim after an allocation crash.
The replacement allocation event records `previous` and `claim_transferred`; `refresh_started` records the replay commits.
The original work survives an interrupted allocation or replay.
This report does not claim the repository gate, independent review, landing, or aggregate acceptance.

## Claim-routing incident

These initial tracker commands succeeded but selected the existing `claude/collab-ops-spec` effort checkout through its `Integration branch` metadata:

```text
python3 tools/temp_root.py -- python3 tools/agents/tracker.py claim collab-ops 10 --owner refresh-source --ttl-hours 24
python3 tools/temp_root.py -- python3 tools/agents/tracker.py claim collab-ops 17 --owner refresh-source --ttl-hours 24
```

They changed these foreign ticket files:

- `/data02/home/philip.yang/workspace/corenius-claude-collab-ops-spec/docs/efforts/collab-ops/tickets/10-refresh.md`
- `/data02/home/philip.yang/workspace/corenius-claude-collab-ops-spec/docs/efforts/collab-ops/tickets/17-standalone-source.md`

The implementing agent stopped tracker writes when it discovered the routing.
The lead instructed it to preserve the claims and report the incident.
The local ticket copies remain unchanged; the agent made no subsequent foreign mutation or cleanup.
Ticket 17 adds public-CLI regression tests and a preflight refusal for this routing behavior.

## Integration notes

No later-ticket findings beyond the integration requirements above.

## For later tickets
none
