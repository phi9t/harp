# Lean Reconstruction Prompt Boundary

Fill only declared proof-body slots for the sealed obligation. Do not change
imports, theorem statements, local context, notation, module headers, source
maps, or ledger rows. A candidate is useful only after a fresh compile and axiom
audit classify it as `compiled`.
