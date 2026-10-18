# Real Codex launch-then-resume, run by the lead with the landed tools (2026-10-05 23:47 UTC)

Owed from ticket 20 (its Codex proof could not run while the host's /tmp was full).

Setup: throwaway repository with branches main and side; worker.py launch --runtime codex --model gpt-5.5 --effort low, workspace-write sandbox, private clone with the git directory under the run directory, allowed paths a.txt and b.txt.

- Launch task: create a.txt containing one, commit. Result: the worker created and committed a.txt (bed15a62c); the supervisor judged it ready with that Candidate; verdict DEGRADED only because the declared path b.txt did not exist yet (missing allowed output).
- Resume (worker.py resume --detach, same session): create b.txt containing two, commit. Result: the worker created and committed b.txt (e1c1b86) on top; two commits on the worker branch; main and side unchanged.
- Verdict after resume: VIOLATED with candidate-moved, because a plain resume is not a delivered review; by ticket 04's rule only a review delivered through deliver-findings or lead_launch rework makes the new commit a successor Candidate. The runtime half is what this proof was for: a Codex worker commits after a resume under its default sandbox with the private git directory writable.

Event streams: 20-lead-real-codex-launch-events.jsonl, 20-lead-real-codex-resume-events.jsonl.

Not proven here: the full rework path (lead_launch rework) with a real Codex worker; that belongs to the live trial.
