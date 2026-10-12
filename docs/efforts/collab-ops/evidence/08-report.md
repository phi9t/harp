# Ticket 08 Report

## Built

- Added initialization-time protected path configuration for collab.
- Stored `protected_paths` in the collab config, using Corenius defaults when no repository-specific list is supplied.
- Added `--protected-paths-json` to `collab init` for repositories that choose their own protected path list.
- Enforced the stored protected path list at `submit` against the Candidate's changed paths.
- Documented the Corenius default list in `tools/collab/README.md`.

## Acceptance

- [x] A Candidate touching a gate script, the gate entry point or the constitution is refused at submit.
  - Covered by `test_default_protected_paths_include_gate_and_constitution`.
- [x] A repository can choose its own list at init.
  - Covered by `test_repository_specific_protected_paths_are_set_at_init`.
- [x] The default list matches the documented one.
  - Covered by the init output assertion in `test_default_protected_paths_include_gate_and_constitution`.
  - Documented in `tools/collab/README.md`.

## Claims

- Red evidence: before implementation,
  `python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_default_protected_paths_include_gate_and_constitution -v`
  failed with `KeyError: 'protected_paths'`.
- Focused protected-path tests passed:
  `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_default_protected_paths_include_gate_and_constitution tools.collab.tests.test_collab.CollabTests.test_repository_specific_protected_paths_are_set_at_init tools.collab.tests.test_collab.CollabTests.test_init_rejects_invalid_protected_path_json tools.collab.tests.test_collab.CollabTests.test_protected_coordination_and_policy_paths_cannot_submit -v`
  passed 4 tests.
- Collab gate shard passed:
  `python3 tools/temp_root.py -- sh tools/agents/checks.d/65-collab.sh`
  passed 53 tests in 73.196s.
- `git diff --check` passed before the implementation commit.

## Outside declared paths

- No implementation changes outside `tools/collab/`.
- This report was written at `docs/efforts/collab-ops/evidence/08-report.md`, the ticket report path allowed by the worker brief.

## For later tickets

- None.

## Not checked

- The default repository gate did not complete within this worker's 120-second command limit:
  `timeout 120s python3 tools/temp_root.py -- sh tools/agents/check.sh`.
- A cargo-disabled full gate attempt also did not complete within the same limit:
  `CHECK_CARGO=0 timeout 120s python3 tools/temp_root.py -- sh tools/agents/check.sh`.
