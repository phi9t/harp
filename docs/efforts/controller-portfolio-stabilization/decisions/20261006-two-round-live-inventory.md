# Inventory the live estate in two observation rounds

Recorded: 2026-10-06

The owner selected a two-round live inventory rather than pausing Corenius or
accepting a best-effort single scan. Each source observation carries its time
and generation: for example a Git head and index identity, process start time,
tmux server identity, provider session generation, claim revision or artifact
digest.

The Scout Lead repeats the declared inventory. Stable facts are promoted into
the preservation record. Changed facts remain explicit drift, not an inferred
terminal state. Any Action Packet whose preconditions depend on drifted or stale
state is quarantined until a stable follow-up observation closes the uncertainty.
