# Durable coding work

Harp's integrated coding path is `harp durable` on Linux. Its Python + SQLite
runtime owns admission, capacity, attempts, cancellation, recovery and accepted
results. Insula and per-attempt delegated cgroups contain each worker. Use
Codex for the consolidated path; provider sessions are not resumed automatically.

The [integration tickets](../.scratch/harp-runtime-integration/tickets.md) retain
qualification evidence for one worker, a mixed pair and four mixed-provider
workers on Linux.
A successful coding activity produces a proposal. Independent verification,
a separate human approval and checked-out local landing are subsequent steps.

## Prepare a project

Use Python 3.11 or newer on the Linux host and for `/usr/bin/python3` in the
native rootfs. Use private, owned local storage, a supported SQLite version
(at least 3.51.3), and a writable delegated cgroup v2 subtree. Preparation pins
the runtime code, native rootfs contents and provider executable bytes. A changed profile requires
a new execution record; it never silently upgrades an existing admission.

The native rootfs must contain executable `/usr/bin/python3`, `/usr/bin/git`,
`/bin/sh`, their libraries, and `/usr/bin/bwrap` for nested independent checks.
Networked providers also need their normal CA/DNS closure. Preparation refuses
an empty, missing or symlinked resolver snapshot; populate a separate native
closure with valid resolver and TLS trust paths before admitting networked work. The provider
binary must be named `codex` in the selected directory. A retained Trae adapter is optional.
Codex also requires its matching packaged `codex-code-mode-host` alongside it.
Supply only each provider's private authentication JSON file. Whole host homes,
configuration directories and owner capabilities are not worker inputs.

An operator can explicitly enter a temporary delegated session, for example:

```sh
systemd-run --user --scope -p Delegate=yes bash
```

Inside that session, select absolute project-owned paths and its delegated root:

```sh
state=/absolute/project-runtime/state
execution=/absolute/project-runtime/execution.json
rootfs=/absolute/native-rootfs
providers=/absolute/provider-binaries
cgroup=/sys/fs/cgroup$(awk -F: '$1 == "0" {print $3}' /proc/self/cgroup)
harp durable --state "$state" --execution "$execution" prepare \
  --rootfs "$rootfs" --cgroup-root "$cgroup" --network host \
  --providers-bin "$providers" \
  --codex-auth /absolute/private/codex-auth.json
```

Codex uses its packaged code-mode host for coding tools, with apps and plugins
disabled. Trae also disables plugins and automatic daemon startup.
Neither adapter resumes host sessions or loads user configuration/rules.

Host networking is explicit for provider calls. Deterministic coding and
verification narrow it to no network. A coding worker sees only its selected
credential, immutable inputs and private writable attempt directory. Candidate
check programs run in another namespace with their candidate workspace writable;
the parent verifier's proofs, logs, inputs and credentials are hidden.

The admitted OS delegation must remain available across controller restarts.
Ending that delegation prevents new dispatch under its execution record. Retained
status and evidence remain inspectable. This contract does not promise host reboot
recovery or automatic replacement of an execution capability.

For new work after a delegation ends, prepare a new execution record against a
fresh explicit delegation. The same private project state may retain completed
history, but first confirm no active attempts, unresolved cleanup blocks or held
controller lock. Running or blocked work keeps its original contract and must be
reconciled under it; a new profile never rebinds old attempts. Inspect or approve
old results with their old execution record, and admit fresh work with the new one.

## Admit and execute

Commit a task Markdown file and an acceptance JSON file in the source repository.
Acceptance binds an explicit `allowed_paths` list and one through 32 checks.
Checks use isolated Python (`/usr/bin/python3 -I -c ...`) or a committed oracle
script declared in `oracle_files` and named as `/oracle/<path>` in the check
arguments. The verifier resolves these logical names into its private readonly
oracle mount; inline checks can receive the resolved path as an argument. The owner
freezes oracle bytes separately from worker inputs. Candidate files cannot
replace the verifier. For example:

```json
{"checks":[["/usr/bin/python3","-m","unittest","discover","-s","tests"]]}
```

Admission reads the selected committed revisions, not working-file edits. It
freezes the task, acceptance, source base, workflow, execution contract and
provider. The run ID is an idempotency key; conflicting reuse is refused.

```sh
harp durable --state "$state" --execution "$execution" admit task-001 \
  --repository /absolute/source-repository --task task.md \
  --acceptance acceptance.json --base HEAD --task-revision HEAD --provider codex
harp durable --state "$state" --execution "$execution" start \
  --capacity 1 --until-idle --timeout 1800
harp durable --state "$state" --execution "$execution" status task-001
```

Use `--provider trae` for Trae. Capacity is configurable from one through four;
it belongs to this project state and is released only after proven cleanup.
Status includes attempt paths and durable stdout/stderr locations. A second
controller for the same state is refused. `stop` requests the current runtime
instance to drain; it cannot signal a replacement through an old PID.

## Verify, approve and land

```sh
harp durable --state "$state" --execution "$execution" capture task-001
harp durable --state "$state" --execution "$execution" verify task-001
harp durable --state "$state" --execution "$execution" start \
  --capacity 1 --until-idle --timeout 1800
```

`verify` returns a separate verification run ID. Inspect that run's status,
result and retained logs. Capture retains exact Git objects independently of the
worker directory. Passing verification refers to the captured base, commit,
tree, required checks and execution identity. Worker-written files do not grant
approval.

The human operator registers a private owner capability outside the state,
source checkout, native rootfs and every projected input. Keep it outside model
processes. Then, after reviewing the concrete candidate and check evidence:

```sh
harp durable --state "$state" --execution "$execution" owner-init \
  --capability /absolute/private-owner/landing-capability
harp durable --state "$state" --execution "$execution" approve task-001 \
  --verification VERIFICATION_RUN_ID --destination /absolute/source-repository \
  --capability /absolute/private-owner/landing-capability
harp durable --state "$state" --execution "$execution" land APPROVAL_ID \
  --capability /absolute/private-owner/landing-capability
```

Approval binds one verified candidate and destination base. Landing serializes
Git mutation and synchronizes its branch, index and tracked worktree. A dirty,
substituted or advanced destination is refused without forcing it clean. An
advanced base requires a new admitted integration candidate, fresh checks and a
new human approval. An interrupted landing is reconciled against its durable
intent and exact Git identities before another mutation.

Landing requires canonical Git blob bytes. Unqualified checkout conversions,
local attribute overrides and sparse checkouts are refused before mutation;
filtered/LFS materializations are outside this qualification. Use a normal clean
checkout with compatible Git settings rather than forcing the destination.

## Cancel and reconcile

`cancel RUN_ID` requests cancellation; inspect status for actual cleanup and the
remaining effect decisions. Runtime death after a worker starts can leave an
uncertain effect even when descendants are gone. Cleanup and effect semantics
are distinct decisions:

```sh
harp durable --state "$state" --execution "$execution" ack-cleanup ATTEMPT_ID \
  --evidence 'Observed the exact owned cgroup empty; recorded kernel evidence'
harp durable --state "$state" --execution "$execution" resolve RUN_ID coding retry \
  --evidence 'Inspected retained attempt and determined a new attempt is safe'
```

Neither command starts work. Resolve each uncertain effect explicitly, and use
`start` when reconciliation is complete. A valid durable receipt replays without
rerunning the worker; a started ambiguous effect is not automatically retried.
Missing cleanup evidence blocks capacity reuse. Keep the actual evidence behind
an operator decision; the example text alone is not cleanup proof.

Use `events` for operational history and `check` for SQLite integrity and admitted
version. Task intent remains Git-managed Markdown. No Temporal, PostgreSQL,
tracker daemon or installed always-on Harp service is required.

## Source migration and retained history

The current package is `durable`; old package paths and competing controllers
are retired. Prepare a fresh execution for these source bytes. Completed records
retain their original execution contract. Never replace their sealed code to make
old state run under the new package. Inspect historical receipts as evidence and
reconcile ambiguous effects explicitly before starting another attempt.

Git-managed `.scratch/<feature>/spec.md` and `issues/*.md` remain the authoring
interface. Admission reads exact committed task and dependency bytes, rejects
unfinished dependencies and freezes their hashes with the allowed change scope.
The SQLite journal records execution; it is not an editable intent backlog.

Archiving an existing execution must preserve the full identity of every sealed
layer, including nanosecond timestamps where its fingerprint binds them. A
byte-for-byte copy with rounded timestamps may fail admission. Keep the original
layer records and verify the restored fingerprint before inspecting or landing
an old candidate; never regenerate its profile against a new source or layer.
