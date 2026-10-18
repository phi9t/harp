# worker-in-capsule-live-and-ssh-dir

Date: 20260930
Supersedes:

## Decision

`worker.py launch --capsule` and `run.py --agent traecli` are the supported ways to run a real traecli inside a capsule. The
capsule root is a writable tmpfs, so programs may create files there; those files are ephemeral and never reach the host.

## Evidence (live, owner-approved model spend)

- Worker `capsule-smoke-20260930T000005Z`, real traecli GPT-5.5 inside `agent.json`: verdict CLEAN in 59 s, wrote the requested file in
  its worktree, cwd `/workspace`.
- Its final message said it could see `/data02/home/philip.yang/.ssh`. Checked by the lead: the directory exists inside the capsule
  as a fresh tmpfs directory (mode 700, created at startup, one 96-byte `known_hosts`), with no bind mount for it
  (`mountinfo` has none), while the host `~/.ssh` holds different files (keys, config). traecli creates it itself from the passwd
  home directory. Direct probes of the same capsule with the same flags say the path is absent until traecli runs.
- `run.py --agent traecli p01-slugger`: PASS in 74 s (first live agent run; before that two harness bugs were fixed: missing
  `--skip-git-repo-check`, agent output not kept).

## Consequences

- The `host_home_hidden` probe checks the capsule before a program runs; it cannot see files a program creates later. That is
  correct, but a report of "can see ~/.ssh" from an agent must be checked, not trusted.
- Host key material is not exposed to agents by the capsule; only traecli's own config files are bound, read-only.
