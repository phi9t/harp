# Lab 2: Temporal wait, restart, rollover and replay

Source read: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`.
The executable assertions are in
`crates/team-control-plane/tests/temporal.rs`.
The bridge and workflows are in `src/temporal.rs` under that component.
The exact Rust SDK/client/workflow/macros dependencies are version 1.0.0;
the lockfile resolves SDK Core 0.9.0.

## Learning model

PostgreSQL remains authoritative for Task state, command dispositions and
workflow cursors. Temporal histories retain orchestration milestones.
A signal is a hint to inspect durable state. Losing that signal must not lose
the command or the committed answer.

Workflow code calls Activities for database and external work.
The workflow itself retains deterministic control flow: activity scheduling,
signals, timers, and Continue-As-New. The native workflow performs four drain
iterations with a 500 ms timer or hint wait before Continue-As-New.
Do not infer a production cadence or sizing policy from this small fixture.

There are three distinct identities:

| Identity | Meaning |
| --- | --- |
| Workflow ID | Stable aggregate/generation/epoch orchestration identity |
| Temporal Run ID | One server execution in a Continue-As-New chain |
| Database generation | Current authority version for that aggregate workflow |

Changing a Run ID through rollover does not authorize a new Task admission.
The bridge records the observed server execution and its previous-run chain.
A matching caller-supplied string alone cannot establish this observation.

## Execute the real fixtures

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/run.py --lab temporal --timeout-seconds 1200
```

The shell fixture starts its own headless Temporal dev server on loopback,
with an ephemeral port and scratch SQLite file. It also starts a disposable
PostgreSQL container. No existing server or namespace is required.
Rust SDK workers are real child processes of the native test.

The suite's ignored `temporal_worker_process` is an intentional child entrypoint.
The parent invokes it explicitly with `--exact --ignored` and owns its lifetime.
Do not treat its default ignored status as a skipped recovery proof.
Do not use global `--include-ignored`; that entrypoint requires parent inputs.

## Exercise A: registry, cursor and generation

Read `public_migration_installs_transactional_workflow_registry`.

1. Observe an idempotent workflow registration and one initial wakeup.
2. Enqueue sequence 1 with a rejected admission and sequence 2 with a valid one.
3. Try sequence 2 first; expect `StaleRevision` and cursor 0.
4. Apply sequence 1; its rejection is durable.
5. Apply sequence 2; expect cursor 2 and an identical result on repeated apply.
6. Close the workflow, enqueue a late command, and observe its rejection.
7. Register an authorized successor generation.
8. Apply through the old reference; expect `StaleGeneration`.

A rejected command still has a disposition and can advance the contiguous
cursor. A missing command cannot be skipped by a later sequence.
Locate `Bridge::apply_with_capability` and its `sequence != cursor + 1` check.

The suite also tests operation-family identity and a verifier's inability to
suppress the initial wakeup. These are real PostgreSQL bridge checks.

## Exercise B: lose hints, retry an Activity, kill the worker

Read `real_temporal_recovers_lost_hints_rollover_and_retained_replay`.

The parent delivers the initial wakeup twice and sends duplicate signals.
The stable Workflow ID uses `UseExisting` and `RejectDuplicate`.
Then it enqueues a valid command without delivering its hint.
A durable timer must discover it and move the database cursor to 1.

Next, it enqueues an assignment without a hint and waits for cursor 2.
Its Activity interceptor records an actual successful canonical drain,
then deliberately returns an Activity failure once.
The SDK must retry; `activity-invocations.jsonl` must include an attempt
number of at least 2.

The parent SIGKILLs the real worker and waits for its confirmed signal exit.
It starts a replacement worker on the same task queue.
A later rejected command advances the cursor to 3.
The database still contains exactly one Attempt, operation and reservation.

This proves recovery across worker process death and a lost Activity reply.
The deliberately injected reply loss is a fault hook on a real Activity path.
It is not a substitute implementation of Temporal or PostgreSQL.
No actual provider operation is dispatched by this lab.

## Exercise C: Continue-As-New preserves progress

Continue reading the same native case.

The test waits until the server reports a Run ID different from the first run.
It checks registry incarnation against the actual previous-run chain.
It rejects observation from a stale run and applies another command after rollover.
The cursor reaches 4; closing the bridge completes the workflow.

Required artifacts and assertions:

| Artifact or state | Required property |
| --- | --- |
| `task-first.json` | Actual Continue-As-New history |
| Successor workflow input | Cursor 3 and the same retained waits/reference |
| Terminal result | Cursor 4, closed, one retained wait |
| `registry-observation.json` | First run, observed successor and terminal identity |
| Late wakeup after close | `StaleGeneration` |
| Command after terminal | Durable rejected disposition |

The result has `workflow_closed` as its operation limitation.
This is orchestration evidence; it does not claim an external runtime operation
completed. The queued old-generation dispatch test uses a fixture runtime adapter
to assert zero calls after replacement. It is ancillary seam coverage, not a
native runtime qualification claim.

## Exercise D: retain a mailbox wait across process death

Read `real_temporal_mailbox_wait_survives_worker_death_and_lost_answer_hint`.

The fixture admits a planning Episode, sends a question and yields a checkpoint.
Before killing the worker, it waits for actual Activity history to contain an
unsatisfied wait with that Episode, question and checkpoint.
Reading only the database registry would not establish that Temporal recorded it.

While the worker is dead, the fixture commits an answer twice.
Both replies identify one durable message.
It starts a replacement worker without delivering the answer's wakeup.

The test allows 30 seconds for the observed incarnation to advance.
An in-flight Workflow Task can require its normal 10-second timeout.
The one-second sticky schedule-to-start timeout does not replace that timeout.

After observing the successor, the test waits for actual Activity completion
history to contain the satisfied wait. Registry observation occurs inside an
Activity before the server records completion; those are different barriers.

Finally, duplicate signals are delivered, the bridge is closed, and the terminal
result must retain the exact checkpoint with a satisfied question.
Mailbox references must not become gateway admission waits.

## Exercise E: replay the retained histories

Both native recovery tests construct a real
`WorkflowReplayer`, register MissionWorkflow and TaskWorkflow, and call
`replay_workflow` on SDK-exported histories.

The first case replays its three fresh task/mission histories plus the three
retained v1 histories already tracked in
`docs/efforts/team-control-plane/evidence/ticket13-temporal-bridge/`.
The mailbox case replays its two fresh histories plus the same retained inputs.

A replacement worker's progress is not the replay assertion.
The explicit SDK replayer is the compatibility check.
The tests scan history strings and decoded base64 payloads for fixture credentials.

The native replay fixtures serialize through
`fetch_history(...).to_json()`. Do not replace these inputs with CLI shorthand
JSON without a verified conversion. The earlier feasibility spike documented
that serializer mismatch; this fresh lab uses the working SDK path.

## Inspect fresh artifact structure

Use the evidence directory printed by the runner. This read-only command reports
history sizes, event types, and the highest observed Activity retry:

```bash
python3 tools/temp_root.py -- python3 - EVIDENCE_DIRECTORY/temporal <<'PY'
from collections import Counter
import json
from pathlib import Path
import sys

root = Path(sys.argv[1])
for name in ["task-first.json", "task-terminal.json", "mission.json",
             "mailbox-first.json", "mailbox-terminal.json"]:
    data = json.loads((root / name).read_text())
    types = Counter(event.get("event_type") for event in data["events"])
    print(name, len(data["events"]), dict(types))
rows = [json.loads(line) for line in
        (root / "activity-invocations.jsonl").read_text().splitlines()]
print("maximum Activity attempt:", max(row["attempt"] for row in rows))
print("cleanup:", (root / "cleanup.json").read_text())
PY
```

Replace `EVIDENCE_DIRECTORY` with the printed run root. This inspection does
not itself execute replay; the native test already did so.

## Rerun only the mailbox case

Create an owned evidence directory and keep it outside `/tmp`:

```bash
python3 tools/temp_root.py -- mkdir -p "$PWD/.scratch/engineering-profile-labs/mailbox-focused"
python3 tools/temp_root.py -- env CARGO_TARGET_DIR="$PWD/.scratch/engineering-profile-labs/cargo-target" TEMPORAL_FIXTURE_EVIDENCE="$PWD/.scratch/engineering-profile-labs/mailbox-focused" bash crates/team-control-plane/tests/run-temporal.sh --offline --test temporal real_temporal_mailbox_wait_survives_worker_death_and_lost_answer_hint -- --exact --test-threads=1
```

Prefer the full runner for indexed, deadline-bound acceptance evidence.
If a case fails, preserve its histories and diagnose the first failed assertion.
Do not replace the failure with a synthetic progress report.

## Completion questions and limits

Explain why signals are hints, how command dispositions close cursor gaps,
and why an Activity completion history is stronger than a registry observation.
Then distinguish worker restart, Continue-As-New, and offline replay.

This lab does not restart the Temporal server or recover either database from
failure. It does not prove HA, production upgrades, safe arbitrary workflow
changes, native runtime fencing, real agent execution or controller readiness.
