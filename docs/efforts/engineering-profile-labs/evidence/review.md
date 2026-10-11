# Integrated learning lab verification

Recorded 2026-10-11T19:42:31+00:00. Source baseline:
`b7bee20e00c822feec48362a1dbd8d62fe9c3641`.

All five walkthroughs are integrated in the private Corenius clone. Fresh
database evidence is in [summary.md](summary.md); native observations and their
classification are in [the native index](../native/README.md).

## Independent review

A clean-context reviewer checked both repository Standards and the effort Spec.
The PostgreSQL/Temporal slice passed. Native review found three P2 issues:
unordered CLI acceptance, incomplete tool-item rejection and unsafe temporary
root validation. The native implementer repaired each through model-free
red/green fixtures in a separate private clone, then froze the handoff.

The integrated re-review reports Standards **pass** and Spec **pass**, with no
remaining actionable findings. It verified all 16 native content hashes,
unchanged database input hashes and all ten unchanged historical native evidence
hashes. It did not launch services or providers.

## Fresh integration checks

The parent independently executed:

```bash
python3 tools/temp_root.py -- python3 -m unittest discover -s docs/efforts/engineering-profile-labs/native -p 'test_*.py' -v
python3 tools/temp_root.py -- python3 tools/agents/tracker.py lint engineering-profile-labs
python3 tools/temp_root.py -- git diff --check
```

All 21 native tests passed. These include ordered turn acceptance, mismatched
identity rejection, unknown/missing item rejection, shared CLI/RPC tool flags,
owned temporary-root checks and replay of three retained CLI projections.
The recorded Codex CLI result remains accepted; both TraeCLI results remain
rejected. FILES.json independently matched all 16 content digests. The slice's
Ruff lint/format checks also passed before its frozen handoff.

The repaired guard cannot requalify the historical app-server record: that
projection omitted item categories. The record still proves its observed
arithmetic completion joined to thread/turn identity. No provider retry was
used to fill the missing evidence. Empty-thread persistence remains separate
from active-turn recovery.

## Handoff limits

Database results remain 16 PostgreSQL and 7 Temporal passing tests, including
11 explicit SDK replay invocations. A child worker entrypoint is intentionally
ignored by ordinary libtest and explicitly invoked by the recovery parent.
The full unrelated repository gate was not run. Deadline termination,
operational database recovery, active-turn recovery, lost-ack reconciliation
and enforced provider budgets remain unqualified.

The original Corenius checkout remains clean. Unrelated Trondheim main-checkout
simulation changes were preserved. No commit, push, merge, production migration,
deployment or GPU run occurred. Tracker tickets await accepted landing; passing
local checks do not manufacture a promotion receipt.
