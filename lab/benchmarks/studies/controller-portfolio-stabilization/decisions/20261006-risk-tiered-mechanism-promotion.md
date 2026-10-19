# Apply risk-tiered proof to I1 mechanism promotion

Recorded: 2026-10-06

The owner selected a risk-tiered evidence floor for promoting I1 mechanisms into
I2. Pure state and data mechanisms require deterministic behavioral tests and
independent review. Provider and runtime mechanisms require a real B200 episode
that proves disconnect, restart and reconciliation behavior. Authority and
external-effect mechanisms additionally require model-free fault injection,
stale-generation rejection, idempotent replay and retained quarantine evidence.

Every promoted mechanism must fit I2's controller-owned execution state and may
not create a second ledger or runtime authority. Uniform live testing is not
required for pure logic, and fixture-only tests cannot establish provider or
recovery claims.
