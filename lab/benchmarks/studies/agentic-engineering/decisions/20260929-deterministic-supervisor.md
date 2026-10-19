# Supervision is deterministic; a Haiku subagent only relays

Date: 20260929
Supersedes: 

## Decision

`worker.py supervise` blocks on the process, checks git and process state each cycle, and writes the verdict. The Haiku supervisor (F3) runs it and reports the verdict verbatim.

## Why

In trial 1 the LLM supervisor never blocked, sent repeated interim reports, gave wrong timings and once declared completion while the process was alive.

## Consequences

The lead keeps its own PID watch.
