# Independent design critique and disposition

Reviewer: separate-context /root/monitor_design_critique, read-only, 2026-10-04 UTC.
Scope: proposed monitor design and initial manual report, not product implementation.

The reviewer identified three blocking contracts. Each has been added to SPEC.md:

1. Overlapping live/backfill batches could duplicate progress/finding/repair/correction effects. Resolution: shared transactional effect identities, explicit adjudication of ambiguous finding matches, and overlap tests in both publication orders.
2. Cited model judgments did not define trusted confirmation/approval. Resolution: all analytical classifications are advisory; enforcement requires controller-verifiable facts or a separately authenticated authorized decision, then current-intent/epoch validation at durable command admission. Injected claims of authorization or exhausted repair counts are explicit negatives.
3. Mutable artifact retrieval and source deletion could invalidate review evidence. Resolution: immutable source/test/artifact identity, retained content-addressed inputs, concurrent-change detection, bounded retention with explicit expiry, and source-deletion tests.

The reviewer confirmed that formal-turn/action distinction, investigation-versus-repetition assessment, manual operational limits and the initial historical coverage gap were represented honestly. This resolution records design changes; it does not claim the monitoring subsystem is implemented or qualified.
