# Durable intent

## Intent pipeline

Materialize human intent in the Matt Pocock style: develop a detailed specification,
then split it into bounded tickets that each deliver an independently useful result.
Use the existing [Git-managed Markdown tracker](../../docs/agents/issue-tracker.md):
project specifications live in `.scratch/<feature>/spec.md` and tickets in
`.scratch/<feature>/issues/`. Keep acceptance concrete enough to check the requested
behavior without granting workers broader authority.

This package supplies framework primitives for reading committed records, freezing
inputs and enforcing change policy. It does not own the project specifications or
issues. SQLite journals execution, attempts and decisions; it does not replace the
Git-managed intent records.

## Committed readiness

[records.py](records.py) follows `Parent:` links to freeze context. A Parent may
remain open while its child task proceeds: its status is not a completion gate.
Issue readiness instead depends on records linked under `## Blocked by`. Each
dependency must contain `Status: done`, and traversal checks dependencies
transitively. Cycles, unfinished dependencies and paths escaping the repository
are refused. Dependency links must resolve to local Markdown tracker records
under `.scratch/`.

Only tracker issues under `.scratch/` with `/issues/` in their path infer
dependencies from that section. Non-issue Markdown does not infer dependency
edges from incidental prose links. Keep context links and completion dependencies
distinct when preparing a task for admission.

## Frozen assignment

The [admission implementation](../jobs/coding/admission.py) and
[CLI](../cli.py) expose two independent revision choices. `admit
--task-revision` selects the committed task, parent, dependency, acceptance and
owner-check bytes. `--base` selects the implementation checkout. An updated parent
at the task revision can legitimately be newer than the implementation base.

The worker receives the frozen task, parent and dependency context with the
admitted scope. Editing ambient source files cannot redefine that assignment.
[inputs.py](inputs.py) publishes immutable, digest-identified input bundles and
verifies their manifests. Context freeze is the admission boundary.

Inspect the actual admission options without launching work:

```sh
python3 -m durable --state /example/runtime/state admit --help
```

## Oracle boundary

Acceptance declares explicit `allowed_paths` for candidate changes and
owner-frozen `oracle_files` for independent checks. For example:

```json
{
  "allowed_paths": ["example/module.py"],
  "oracle_files": [".scratch/example/check.py"],
  "checks": [["/usr/bin/python3", "-I", "/oracle/.scratch/example/check.py"]]
}
```

[policy.py](policy.py) validates the policy and checks changed paths against it.
The declared script is frozen from the task revision separately from the candidate
and exposed under `/oracle/`. Python's `-I` excludes candidate import paths.
Candidate edits cannot replace the owner checker; candidate-authored passing tests
do not grant acceptance or landing authority.

## Execution authority

Providers propose candidates. The durable framework captures the exact candidate
and independently verifies it against the admitted checks and scope. The CLI
separates `capture` and `verify` from `approve` and `land`; human approval remains
necessary for landing the exact verified candidate. Execution success alone is
not approval.

Codex is the first qualified provider. Provider conversations, execution logs and
SQLite history supply operational evidence while authority remains with the
frozen assignment and explicit human decisions. See the
[framework entry point](../README.md) for the broader execution lifecycle.
