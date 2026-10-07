# Run the autonomous Lead on the B200 host

Recorded: 2026-10-06

The owner selected a remote-first operating topology. A replaceable Lead runs
on the B200 development host and autonomously drives ticket sequencing, worker
launches, isolated implementation, tests, reviews, evidence and coherent
feature-branch commits. The current macOS session acts as Director and observer;
it sets priorities, audits state and supplies the approvals reserved below.

The Lead is a logical RoleEpoch, not a tmux pane, provider conversation or PID.
Its runtime and provider bindings must be recorded so a replacement cannot
silently duplicate authority. The Director retains approval for pass-two action
packets, landing, parking, retirement, push, merge and destructive cleanup.
