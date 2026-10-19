# Ticket 03 open-attempt report

## Built
- Renamed the collab public Attempt-opening command from `claim` to `open-attempt`.
- Added Corenius tracker-ticket support for `open-attempt <NN> --effort <effort> --owner <owner>`.
- For Corenius tickets, `open-attempt` now preflights readiness through
  `tools/agents/tracker.py list`, claims through `tools/agents/tracker.py claim`,
  commits that tracker claim on trunk, records the Attempt in SQLite and creates
  the locked worker worktree from the committed claim.
- Attempt branches now follow the agent/topic convention as
  `<owner>/collab-<ticket>-<attempt>`.
- Allocation recovery now completes a Corenius Attempt that crashed after the
  allocation record, after the tracker claim, or after worktree creation.
- Updated the collab README to describe `open-attempt`, Corenius tracker tickets
  and allocation recovery.

## Acceptance
- [x] Opening an Attempt for a ready ticket claims it in the tracker, creates the
  worktree and records the Attempt together.
  - `test_open_attempt_claims_corenius_ticket_and_creates_worktree` covers the
    tracker claim text, committed clean trunk, Attempt row, branch and worktree.
- [x] A blocked, claimed or resolved ticket is refused.
  - `test_open_attempt_refuses_unready_corenius_tracker_tickets` covers blocked,
    claimed and resolved tracker tickets with no Attempt rows created.
- [x] A crash after the claim or after the worktree exists leaves a state that
  recover resolves.
  - `test_open_attempt_recover_claims_corenius_ticket_after_state_record_crash`
    covers a crash after the SQLite allocation record and before the tracker
    claim.
  - `test_open_attempt_recover_completes_corenius_allocation_after_tracker_claim`
    covers a crash after the tracker claim and before worktree creation.
  - `test_open_attempt_recover_adopts_corenius_worktree_after_creation_crash`
    covers a crash after worktree creation.
- [x] The tracker remains the only writer of ticket files.
  - Corenius ticket mutation is routed through `tools/agents/tracker.py claim`;
    collab only stages and commits the path reported by that command.
- [x] Behavior is proven through the CLI in disposable Git repositories.
  - The new tests use `tools/collab/collab.py` subprocesses against real
    temporary Git repositories.

## Claims
- Focused red evidence before implementation:
  ```sh
  python3 tools/temp_root.py -- python3 tools/collab/tests/test_collab.py \
    CollabTests.test_open_attempt_claims_corenius_ticket_and_creates_worktree \
    CollabTests.test_open_attempt_refuses_unready_corenius_tracker_tickets \
    CollabTests.test_open_attempt_recover_completes_corenius_allocation_after_tracker_claim \
    CollabTests.test_open_attempt_recover_adopts_corenius_worktree_after_creation_crash
  ```
  Result: failed because `open-attempt` was not a valid subcommand.
- Focused final proof passed:
  ```sh
  python3 tools/temp_root.py -- python3 tools/collab/tests/test_collab.py \
    CollabTests.test_old_claim_command_is_renamed_to_open_attempt \
    CollabTests.test_open_attempt_claims_corenius_ticket_and_creates_worktree \
    CollabTests.test_open_attempt_refuses_unready_corenius_tracker_tickets \
    CollabTests.test_open_attempt_recover_claims_corenius_ticket_after_state_record_crash \
    CollabTests.test_open_attempt_recover_completes_corenius_allocation_after_tracker_claim \
    CollabTests.test_open_attempt_recover_adopts_corenius_worktree_after_creation_crash
  ```
  Result: ran 6 tests in 5.963s, OK.
- Full collab suite passed:
  `python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -v`
  ran 56 tests in 74.442s, OK.
- Collab gate slice passed:
  `python3 tools/temp_root.py -- env PYTHONDONTWRITEBYTECODE=1 sh tools/agents/checks.d/65-collab.sh`
  ran 56 tests in 74.465s, OK.
- Compile check passed:
  `python3 tools/temp_root.py -- python3 -m py_compile tools/collab/*.py tools/collab/tests/test_collab.py`.
- Hygiene checks passed:
  `python3 tools/temp_root.py -- git diff --check -- tools/collab` and a
  line-length scan over `tools/collab` reported no lines over 120.

## Outside declared paths
- None changed outside `tools/collab/` and this report path.

## For later tickets
none

## Not checked
- I did not run the full repository `tools/agents/check.sh`; prior effort
  evidence records that it exceeds this worker run's 120-second per-command cap.
- Corenius check/landing ticket closure for tracker-format tickets remains for
  later tickets; this ticket covers the claim/open Attempt side only.
