# One-shot probe presentation

The one-shot “Corenius monitor wakeup probe” appeared as an inbound message in the exact main-lead thread `01a10084-dc3a-7743-b0c1-4694d5ecc621` on 2026-10-04, after the monitoring setup and root-Insula design turn. The lead confirmed presentation and inspected SPEC.md and OPERATIONS.md. The local CODEX_THREAD_ID matches the configured lead.

This matches the previously recorded one-shot queue probe `01a10815-d7dc-7911-8183-aa49cc389d4c`. The inbound text carries no native receipt metadata; correlation is by its distinctive probe text and exact destination, not a claimed runtime acknowledgement API.

No new scheduler was created, no probe repeated and no worker or active task changed. The separate scheduled wake `2d80dd33-f95f-4290-a3e0-63970a1cea43` remains queued in the wake journal, with receipt `01a10827-dc93-7881-951e-8ddbfc9e70b1`. This one-shot presentation is not a completed periodic worker review and does not advance the transcript cursor or finish that wake.
