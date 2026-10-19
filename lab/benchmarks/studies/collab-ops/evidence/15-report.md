# Ticket 15: Optional effort record-branch extension

Spec base: `c0a2fc0370f18ff28009dbe9b62a59916de0e582`.
Implementation base: `ff66dd24df0f90ae85c3f1dab286cd3b0c156c8c`.
Status: provisional dependency implementation; root acceptance of ticket 05 remains pending.

Existing moved efforts retain exact committed acceptance and record-branch evidence.
`tools/collab/effort_records.py` contains only optional landing evidence finalization.
Its repair function validates the recorded worktree, branch and common Git directory.
The repair writes only the evidence bound to the recorded Candidate and target.
Repeated repair does not append duplicate landing or override entries.

General effort metadata and discovery live in `tools/collab/effort_catalog.py`.
Tracker, acceptance, effort and worker imports use this catalog through the old land shim.
They do not load the optional landing extension when reading in-tree effort records.

## Behavioral evidence

- The 31 historical landing tests pass. Their assertions remain unchanged.
  The only fixture change configures the legitimate owner in `collab.overrideOwners`.
- The new shim suite verifies a committed record-branch acceptance and evidence export.
  It also breaks record output after code lands and repairs the exact bound output.
- Public tracker invocation on a new in-tree effort does not import `effort_records`.
- All 70 unchanged `test_collab.py` tests pass with `effort_records.py` temporarily absent.
  The file was restored after this disposable module-removal check.
- Existing caller suites pass: effort 17, tracker 28, acceptance 18 and worker 89 tests.

The module-removal result proves that the existing core tests do not depend on this extension.
The integrated new-effort lifecycle still requires root verification after tickets 05, 06 and 07 are combined.

## Scope and authority

Work stayed in `codex/collab-ops-legacy` and disposable real Git repositories.
No observed effort, shared branch, remote, policy or other agent worktree was changed.
The local effort pointer selects this isolated worktree for tracker operations.
Ticket status remains open because dependency acceptance is pending.
The root must reconcile its effort pointer during integration.

The full repository gate and independent review remain root completion gates.
