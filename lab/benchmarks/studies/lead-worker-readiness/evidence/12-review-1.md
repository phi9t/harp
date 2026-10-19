# Lead review 1 of ticket 12: changes required before it can land

Reviewed Candidate: the tip of your branch at 10:47Z. The runtime option, the shared command adapter, the
per-runtime observability text and the resume receipt are right and stay. I checked your launch and resume
arguments against the real `codex exec --help`, `codex exec resume --help` and `traecli exec resume --help`
on this machine: every flag you use exists. Do items 1 to 3 now. Do NOT rebase yet; two tickets that
change the same files land before you and I will tell you when.

## 1. Your Codex sample is not the format `codex exec --json` emits (blocking)

Your sample comes from a stored session record (a rollout file), whose records wrap items in `payload` with
type `CommandExecution`. That was my ticket's wording and it pointed you at the wrong source. I ran one
real `codex exec --json` (codex-cli 0.160.0) with a one-command prompt. The stream had six lines of these
shapes:

    {"type":"thread.started","thread_id":"<uuid>"}
    {"type":"turn.started"}
    {"type":"item.started","item":{"id":"...","type":"command_execution","command":"/bin/bash -lc 'echo corenius-probe'","aggregated_output":"","exit_code":null,"status":"in_progress"}}
    {"type":"item.completed","item":{"id":"...","type":"command_execution","command":"/bin/bash -lc 'echo corenius-probe'","aggregated_output":"corenius-probe\n","exit_code":0,"status":"completed"}}
    {"type":"item.completed","item":{"id":"...","type":"agent_message","text":"done"}}
    {"type":"turn.completed","usage":{...}}

(Field values other than the command, output and types are from memory of the run; the key names for the
command item were `aggregated_output, command, exit_code, id, status, type`, and `thread.started` had exactly
`thread_id` and `type`.) I fed that real stream to your `event_commands` and `event_session_id`: the session
id was found, and the command was returned twice, once for `item.started` and once for `item.completed`.

Required:
- Add a second sample in this real stream shape (state in the test that it was captured by the lead from a
  real `codex exec --json` run of codex-cli 0.160.0 on 2026-10-05) and make the fake Codex binary emit this
  shape. Keep the rollout-shaped sample only if you say what it is for.
- Count a command once: take it from `item.completed` (and from `item.started` only if no completion for
  that item id ever arrives, so a command that is still running or was killed is still seen).
- Add `usage` from `turn.completed` to what the adapter can report, since ticket 13's result rows want
  model usage where the runtime exposes it. Recording it in the run record is enough.

## 2. Resume blocks for at most two minutes and then crashes (should fix)

`resume` runs the runtime synchronously with `timeout=120` and does not catch the timeout. A real rework
round takes many minutes, so every real resume would end in an unhandled `TimeoutExpired` traceback with
the worker killed mid-turn. For this ticket: catch the timeout, record it in the receipt as timed out with
the partial event file kept, and exit DEGRADED with a clear message; make the default timeout the run's
wall-clock cap. Add under `## For later tickets`: `- to 04: resume runs in the foreground; a rework round needs
it started detached and supervised like launch (cap, quiet time, hand-off detection).`

## 3. An unexplained change to ticket 09's finding text (should fix)

You added `external_change_labels` and changed the external-state finding detail from full paths to labels
relative to the protected root. That is ticket 09's behaviour, it is not in your ticket, and your report
does not mention it. With two protected roots the labels are ambiguous. Either revert it, or say in the
report exactly what failed without it and keep the root in the label.

## Protocol

New commits on top of your current tip; no rewriting. Update the report. When items 1 to 3 are done and
your gate passes, commit them and then STOP. Do not make the `ticket 12: ready for review` commit yet: your
branch will be stale by then and that commit would signal a Candidate that cannot land. Wait for my
message telling you to re-integrate; the marker commit comes after that rebase.
