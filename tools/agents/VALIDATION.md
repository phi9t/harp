# Repository validation and durable qualification

Run `python3 tools/temp_root.py -- sh tools/agents/check.sh` from the repository
root. The default gate retains all registered suites and the existing opt-in
service switches. Use `--list` to find suite names, `--suite NAME` to repeat one
suite, `--timeout SECONDS` to bound each suite, and `--json PATH` to retain timing
and outcomes. A selected subset is evidence for that subset only.

`.github/workflows/check.yml` runs the gate on hosted Linux runners for pull
requests and main-branch pushes. Its setup installs the repository's Rust
toolchain and fetches locked dependencies before the offline checks. It enables
user namespaces on that disposable runner so bubblewrap tests execute. The
workflow preserves the report as an artifact. Hosting and branch protection
must be configured in the destination repository; local workflow files alone
do not establish a required check.

## Qualify both durable backends

Ordinary durable tests allow disposable databases on older SQLite and skip
native cgroup tests when delegation is absent. Their success does not qualify
cgroup execution or the production SQLite library.

On a Linux host with bubblewrap, a supported Python SQLite library and an
explicitly delegated writable cgroup v2 subtree, run:

```sh
python3 tools/temp_root.py -- python3 -m controller.durable.qualify \
  --output .scratch/qualification.json
```

Set `CORENIUS_TEST_CGROUP_ROOT` to that subtree before running. The qualifier
uses production SQLite admission without an override, then runs the process
group, Insula and native cgroup suites. It fails on missing prerequisites,
empty suites, skips, expected failures, or changed source. The JSON records
source hashes, kernel/Python/SQLite versions, executed cases and outcomes.
Exit 0 means all qualification cases passed, 1 means failed tests, and 2 means
unavailable prerequisites. An incomplete report is never a passing result.
Use a fresh output path for each attempt; existing evidence is not overwritten.

If the user systemd manager supports delegation, `--user-scope` creates a fresh
delegated scope for the run, derives the root from verified process membership,
and applies a ten-minute lifetime with a five-second termination grace. This
uses existing user authority and requires no system installation changes:

```sh
python3 tools/temp_root.py -- python3 -m controller.durable.qualify \
  --user-scope --output .scratch/qualification-user-scope.json
```

The Python process still needs a supported SQLite library. A private shared
library selected through `LD_LIBRARY_PATH` can supply it for qualification;
verify its source against the upstream release checksum and retain that
provenance with the report. The system SQLite installation need not change.

`.github/workflows/durable-qualification.yml` exposes this command as a manual
job restricted to the default branch. Before enabling it, an operator must:

- Register an isolated Linux runner with the `corenius-durable` label, a
  supported SQLite library and a user systemd manager that permits delegation.
  The job uses `--user-scope` and stops that exact scope after execution or
  cancellation. It does not modify sibling cgroups.
- Restrict the runner group to this trusted workflow on the default branch.
  A workflow condition alone cannot protect a runner from another workflow.
- Configure the `durable-qualification` environment to allow only the default
  branch, with reviewer approval.

These hosting controls follow GitHub's [self-hosted runner security guidance](https://docs.github.com/en/actions/reference/security/secure-use)
and [job-condition semantics](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts).
The workflow has read-only repository permissions and retains no checkout
credentials. Pull requests use hosted runners; qualification runs trusted code
only. A qualified result applies to the recorded source and host capabilities,
not to untested GPUs, remote fencing, or external exactly-once effects.

## Handoff before committing

Use `python3 tools/agents/effort.py snapshot EFFORT --evidence FILE` to update
STATE.md without staging or committing. Repeat `--evidence`, `--worktree` and
`--run` to record each input. Run it in the effort's record worktree. It preserves
human sections and marks dirty contents as local-only, not backed up. Rehydrate
refuses to recreate a missing dirty worktree as though its edits were restored.

Use `python3 tools/agents/tracker.py handoff EFFORT NN --owner NAME --evidence FILE`
to record implementation readiness. The ticket stays claimed and blocks
dependents until normal accepted-landed resolution. Expiry does not permit
another owner to take or release a handed-off claim. Use owner-only `resume`
with the same effort, ticket and owner arguments to renew the claim for repairs.
Evidence hashes identify files; they do not assert that tests passed.
