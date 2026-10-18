# Lead review 1 of ticket 18: a real interactive launch showed three defects

Reviewed Candidate: 901a770. The interactive launch mode, the brief generated from one template with
ticket-derived lines marked, both monitors, the session command and the scenario are the right shape and
stay. Your tests use a fake runtime, as instructed. I ran the real thing once: a throwaway repository with a
mainline checkout and an integration worktree, an effort with one ticket ("create hello.txt containing
hello"), and

    lead_launch.py --repo <main checkout> launch demo 01 --interactive --tmux-session lwr-probe \
        --runtime traecli --model gpt-5.5 --reasoning-effort low

The launch created the session and the window and returned a run record. Then:

## 1. A real interactive worker in a new private clone stops at the runtime's folder-trust prompt (blocking)

The window showed, and stayed on for seven minutes until I pressed Enter myself:

    Folder access
    <...>/r-worker-demo-01-hello
    Do you trust the contents of this directory? Working with untrusted contents comes with higher risk of
    prompt injection. Trusting the directory allows project-local config, hooks, and exec policies to load.
    > 1. Yes, continue
      2. No, quit
    enter continue  |  esc quit

A worker's workspace is now a private clone in a directory the runtime has never seen (ticket 17), so every
interactive launch meets this prompt. During those seven minutes `monitor-tickets` said `01 running`,
`monitor-workers` said `01 running ... context=-`, and `wait` returned
`{'triggered': False, 'trigger': None, 'states': {'01': 'running'}}`. Nothing told the lead the worker was
sitting at a prompt.

Required:
- The interactive launch must not block on that prompt. Find the supported way for each runtime to start in
  a directory as trusted (a configuration override on the command line is the likely one; check the
  runtime's help and its configuration reference) and use it, recorded in the run record. If a runtime
  offers no way, the launch command says so and the monitors must show the state below.
- A new worker state, derived from the pane: `blocked-on-prompt`, when the pane shows a runtime prompt that
  waits for a key (this trust prompt; an approval prompt). `wait` returns on it with the prompt text.
- A test with a scripted participant that prints this exact prompt and waits.

## 2. A finished, idle interactive worker is reported as `finishing` for ever, so `wait` never returns (blocking)

After I accepted the prompt the worker did the ticket in about a minute and made the marker commit
`ticket 01: ready for review` on a clean tree at 17:33. Five minutes later, with the session idle:

    monitor-tickets:  01 finishing ahead=1 dirty=0 outside=0 diff=0 waits=- age=5 minutes ago
    wait --timeout 380:  {'triggered': False, 'trigger': None, 'states': {'01': 'finishing'}}

A capture of that pane (247 lines) contains no busy indicator at all; its last lines are the worker's final
message, the input box and the footer `GPT-5.5 · Context 95% left · ... Full Access (shift+tab to cycle)`.
So your busy-or-idle test reads a real traecli pane as busy. The lead's interim script decides it this way
and it has been right all day: the session is busy if and only if the visible pane contains the text
`esc to interrupt`. Use an observation that is true of the real pane, say in the code what it is for each
runtime, and add the real idle pane's last lines as a fixture: the capture is at
`~/.cache/corenius/readiness/lead-worker-readiness/18-interactive-launch-and-monitors/probe-pane-after-ready.txt`
(you may copy the last 30 lines into a test fixture).

Also in that run: `monitor-workers` printed the whole footer line as `context=`; show the remaining context
(`95%`) and nothing else. `wait` exited 0 on timeout; ticket 21 owns that exit code, leave it.

## 3. The session transcript is not recorded for a real interactive session (should fix)

The run record of that real run (`probe-run.json` in the same directory) has no session id and no
transcript path, though the acceptance item requires one "once the runtime's session id is known". An
interactive session emits no JSON event stream, so the id never arrives the way your fake supplies it. The
runtime did write its transcript:
`~/.trae/cli/sessions/2026/10/05/rollout-2026-10-05T17-32-31-<session id>.jsonl`, whose first record names
the session's working directory. Find it by that: the newest session file under the runtime's session
directory whose first record's working directory is the worker's workspace and whose start time is after
the launch. Record the path and the session id, and say in the help for which runtimes this is done.

## What worked in the real run, for your report

The generated brief was followed: the worker created the file, wrote a report at
`docs/efforts/demo/evidence/01-report.md`, and made the marker commit on a clean tree; the run record has
`workspace_kind: private-clone`, the marker hand-off and the tmux target.

## Protocol

New commits on top of 901a770; no rewriting. Update the report. When items 1 to 3 are done and your gate
passes, commit and then STOP without the marker commit: tickets 20 has landed and 19 lands before you, and
both change `tools/agents/worker.py` and `tools/agents/lead_launch.py`. Wait for my message telling you to
re-integrate; the `ticket 18: ready for review` commit comes after that rebase. For item 1 you may run the
real interactive launch above yourself, at most three times, in a throwaway repository on this filesystem,
with a tmux session name of your own that you kill afterwards; this permission is part of this review and
overrides the general rule in your brief about model workers, for exactly that.
