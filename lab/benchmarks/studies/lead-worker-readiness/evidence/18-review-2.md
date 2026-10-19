# Lead review 2 of ticket 18: two of three fixed in a real run; the trust override does not work

Reviewed Candidate: f127c06. I repeated the real interactive launch (traecli, one-file ticket, throwaway
repository with a mainline checkout and an integration worktree) and watched it with your tools:

- **Fixed, seen in the real run:** the worker at the folder-trust prompt was reported as
  `blocked-on-prompt` by both monitors within seconds, and `wait` returned at once with the prompt text.
  After I pressed Enter in my probe the worker did the ticket and made its marker commit at 18:53:26; `wait`
  returned `ready` within eleven seconds; `monitor-workers` showed `context=95%`; the run record has the
  session id.
- **Not fixed:** the prompt still appeared. The launch passed
  `-c projects."<workspace>".trust_level="trusted"` and the real runtime ignored it. Your acceptance evidence
  for "must not block on that prompt" is a test that the command line contains that string; it checks your
  own fixture and cannot fail. You had permission to run the real launch and did not.

I tested the two spellings against the real `traecli` (v0.207.1) in fresh directories, starting it with no
prompt and reading the first screen:

    traecli -c 'projects."/data02/home/philip.yang/devx/tmp/trustprobe.ZiNn".trust_level="trusted"' --yolo
        -> the "Folder access ... Do you trust the contents of this directory?" prompt
    traecli -c 'projects={"/data02/home/philip.yang/devx/tmp/trustprobe.XXXX"={trust_level="trusted"}}' --yolo
        -> no prompt; the normal start screen

The dotted form is split on the dots inside the path (`philip.yang`, and any `.` in a directory name), so
the key never matches. The inline-table form is honoured.

## Required

1. Pass the trust override as an inline table, `projects={"<absolute workspace path>"={trust_level="trusted"}}`,
   for traecli. Check the Codex runtime the same way if you can do so without running a model turn (start it
   in a fresh directory in a tmux pane of your own, read the first screen, kill the session); if you cannot
   check it, say so in the help and the report instead of claiming it.
2. Replace the string-contains test with one that would fail on the dotted form: the scripted interactive
   participant reads its own `-c` arguments, parses them the way the runtime does (split the key on dots
   outside quotes, or simply require the inline-table form and reject a dotted `projects.` key), and prints
   the trust prompt and waits when the override would not apply. The existing `blocked-on-prompt` test then
   covers both outcomes.
3. Run the real interactive launch yourself once after the fix (same permission as in review 1: at most
   three times, a throwaway repository on this filesystem, a tmux session of your own that you kill
   afterwards) and put in the report the two monitor lines and the `wait` output of that run. The item is
   proven only by a real run that reaches `ready` without anyone pressing a key.

## Re-integrate in the same pass

`effort/lead-worker-readiness` has moved since your base: tickets 20 and 19 landed, and this effort's
records were moved to a record branch.
- Ticket 20: `tools/agents/worker.py` (resume keeps the launch boundary, session accounting,
  acknowledgement only on a worker-authored observation), `tools/agents/lead_launch.py` (one shared rework
  counter; fresh-worker rework carries the launch profile), `tools/agents/acceptance.py`.
- Ticket 19: `tools/agents/worker.py` (other agents' branch movements are a NOTE; the run record is updated
  under a lock; `status` writes nothing without a flag; launch validates before creating anything),
  `tools/agents/land.py`, `tools/agents/acceptance.py`, `tools/agents/lead_launch.py` (claims after
  workspaces), the harness module (git environment variables stripped), `AGENTS.md`.
- On the integration branch `docs/efforts/lead-worker-readiness/` no longer exists; tickets, spec and
  evidence are on `effort/lead-worker-readiness-records` at
  `/data02/home/philip.yang/workspace/corenius-lwr-records` (read-only for you). Your report stays at its
  path in your tree and will be the only file of that directory on your branch.
- Ticket 21 is rebasing now and lands before you; it changes the harness module, `tools/agents/worker.py`,
  `tools/agents/tracker.py` and the scenario definitions.

Do items 1 to 3 now as new commits on top of f127c06. Then rebase onto the tip of
`effort/lead-worker-readiness` only after I tell you ticket 21 has landed; until then do not make the marker
commit. I know the standing brief says to finish with the marker; this instruction overrides it for this
hand-off: commit your fixes, then stop and wait.
