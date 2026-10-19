# Historical collaboration mechanisms

Collab authority is retired. `collab.py`, direct `cli.py`, `worktree_compat.py`,
`legacy_landing.py`, and the agent `wt.py`/`land.py` commands refuse before opening
state, changing Git, invoking checks, pushing or landing. Direct `Collab` and
`StateStore` construction also refuses.

Use `harp durable --help` for the admitted lifecycle. Current intent lives in
Git-managed `.scratch/<feature>/spec.md` and `issues/*.md`; no second Collab,
Kata or effort tracker is initialized. Old databases receive no automatic import.
Pure catalog, parser, policy and evidence helpers remain reference material.
Old tests are not selected-runtime acceptance. Source/test history, captured
evidence and licenses remain preserved through Git and `../../HARP_IMPORT.json`.
