# Ticket 02 gate-scope report

## Built
- `tools/agents/checks.d/65-collab.sh` now determines the change set against
  `main` by default, with `CHECK_COLLAB_BASE_REF` available for an explicit
  base reference.
- The collab suite runs when a changed path is under `tools/collab/`.
- The collab suite runs fail-closed when the change set cannot be determined.
- The collab suite skips with an explicit message when no changed path is under
  `tools/collab/`.
- `tools/agents/test_check.py` covers the skip, run and undetermined paths
  through the real `tools/agents/check.sh` dispatcher.

## Acceptance
- [x] A change outside the toolset skips the suite and says so.
  - `python3 tools/temp_root.py -- env PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_check.py`
    first failed before the shell change because the suite still ran for
    `README.md`.
  - Final result: 8 tests passed, including the skip assertion and message.
- [x] A change inside the toolset runs it.
  - Final result: 8 dispatcher tests passed, including the
    `tools/collab/collab.py` changed-file case.
- [x] An undeterminable change set runs it.
  - Final result: 8 dispatcher tests passed, including the failing-Git case and
    the `RUN: collab tests (change set undetermined)` message.
- [x] Shellcheck is clean for the check script.
  - `python3 tools/temp_root.py -- shellcheck tools/agents/checks.d/65-collab.sh`
    passed.

## Claims
- Focused dispatcher proof passed:
  - `python3 tools/temp_root.py -- env PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_check.py`
  - Result: 8 tests passed.
- Real collab suite passed:
  - `python3 tools/temp_root.py -- env PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/collab/tests -v`
  - Result: 48 tests passed in 72.013s.
- Direct current-candidate collab gate behavior passed:
  - `python3 tools/temp_root.py -- sh tools/agents/checks.d/65-collab.sh`
  - Result: exit 0 with `SKIP: collab tests (no tools/collab/ changes)`.
- Diff hygiene passed:
  - `python3 tools/temp_root.py -- git diff --check -- tools/agents/checks.d/65-collab.sh tools/agents/test_check.py`
  - Result: clean.

## Outside declared paths
- `docs/efforts/collab-ops/evidence/` parent directory was created only so the
  declared report path `docs/efforts/collab-ops/evidence/02-report.md` could be
  written.

## For later tickets
none

## Not checked
- A full `tools/agents/check.sh` completion was not achieved inside the
  120-second command limit. The attempted bounded command was:
  ```sh
  python3 tools/temp_root.py -- sh -c 'target=$(mktemp -d "$TMPDIR/cargo-target.XXXXXX"); \
  PYTHONDONTWRITEBYTECODE=1 CARGO_TARGET_DIR="$target" CHECK_CARGO=0 timeout 115s tools/agents/check.sh'
  ```
- That attempt exited 124 while running the existing `tools/agents/test_worker.py`
  suite, after earlier check phases including `tools/agents/test_check.py` had
  passed.
- Optional `CHECK_TEAM_CONTROL_PLANE=1`, `CHECK_BENCH=1`, `CHECK_SYNC=1` and the
  root `CHECK_CARGO=1` profile were not run.
