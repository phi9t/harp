# Validation and handoff

Base read: `e045d4a`. Worktree: `/data02/home/philip.yang/workspace/corenius-durable`.
Branch: `codex/insula-durable`. All changes remain uncommitted and unmerged.

## Fresh feature evidence

```text
python3 tools/temp_root.py -- python3 -m unittest \
  controller.durable.runtime_test controller.durable.insula_test \
  controller.durable.cgroup_test -v

Ran 73 tests in 20.489s
OK (skipped=4)
```

This means 69 tests passed. Four tests require explicit writable cgroup
delegation and were skipped. Python is 3.11.2; SQLite is 3.40.1. Every executable
test database was disposable and used the explicit old-SQLite override.
Normal CLI submission was also tested: it refused SQLite 3.40.1 before creating
the database. These results do not qualify that library for persistent WAL state.

The 35 supplied reference tests passed before adaptation. The final suite also
covers manual effects interrupted by cancellation or sibling failure, multiple
uncertain effects, invalid receipts, temporary environment policy, custom
registry versions, immutable code admission and bounded regular JSON files.

Real bubblewrap tests exercise the arithmetic pipeline and all six controller
crash points. Receipt-before-SQL recovery retains one attempt. Writable payload
data cannot overwrite host guardian receipts. Private temporary storage and
masked cgroup controls were checked inside real namespaces, including a readonly
sysfs projection.

The recorded admitted launcher smoke completed with `sum: 181` and four attempts.
See [smoke.json](evidence/smoke.json). The cgroup admission attempt refused the
current session root because the current UID has no writable delegation.

`compileall`, tracker lint and whitespace checks passed. The submitted archive's
member SHA-256 checks all matched its `SHA256SUMS` manifest.

## Independent review

The repository's separate-context design critique, Standards review, Spec review
and branch defect review completed. Two repair rounds resolved the findings:

- Preserve uncertain manual effects across cancellation and sibling failure.
- Inject approved temporary storage and forbid activity overrides.
- Bind full capsule, code, state and backend through the admitted launcher.
- Reject every file excluded by the existing code-tree digest, including bytecode
  caches and `.DS_Store`.
- Mask cgroup controls when sysfs is projected.
- Retain the fork child's ownership descriptor until cgroup membership is durable;
  require that membership record before guardian-loss cleanup and retry.
- Reject FIFO control files without waiting for a writer.

Final re-reviews reported no unresolved Standards, Spec or defect findings.
Reviewers did not claim positive native cgroup qualification.

`roborev review --branch --base main --local` refused with “no commits on branch
since main.” No commit was authorized, so the required branch defect gate used a
separate clean-context reviewer against `e045d4a` and the WIP specification.

## Repository gate

The full `python3 tools/temp_root.py -- sh tools/agents/check.sh` completed with
exit 1. The feature suite passed within it. Existing suites failed at Unix-socket
pathname limits under the default worktree temporary root:

| Existing suite | Observed failure |
| --- | --- |
| `15-worker.sh` | One tmux test: `File name too long` |
| `45-lead-driven-harness.sh` | One interactive scenario: tmux `File name too long` |
| `70-sqlite-controller.sh` | Native-controller fixtures refused transport paths exceeding the Unix-socket limit |

No existing implementation or test source in those suites changed. The worker
failure passed when repeated with the shorter owned temporary root
`/data02/home/philip.yang/.cdt`. The native-controller retry used that same root. It passed 39 test
cases, including 10 native cases, before the extra retry was stopped. Active
fixture helpers completed cleanup before the test harness was interrupted. This
is partial evidence, not a passing full native suite; see [retry record](evidence/native-retry.json).

Raw command logs remain under `.scratch/insula-durable/`. The failure limits are
part of this handoff; the repository gate is not reported as green.

## Evidence ceiling and remaining operator work

The process-group backend and admitted Insula launcher ran successfully on this
host. Process groups remain a trusted-worker contract. Cgroup refusal and recovery
preconditions were tested, but its positive kernel/runtime tests need
`CORENIUS_TEST_CGROUP_ROOT` on a delegated host. The stronger backend is therefore
implemented but unqualified here. No real GPU or training workload was tested.

The SQLite release check, local code binding and guardian protocol do not prove
physical power-loss durability, remote fencing, native-closure attestation or
exactly-once effects at external destinations. State relocation, backup
coordination, schema migrations, artifact quotas and garbage collection remain
outside this implementation.

No commit, push, PR or merge was authorized. Tracker closure requires an accepted,
landed Candidate and was refused accordingly; implementation evidence is present,
but tickets remain open for that human-controlled stage. `effort.py dehydrate`
was not invoked because it commits automatically. `STATE.md` records this
uncommitted handoff directly.
