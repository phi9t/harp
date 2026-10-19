# Reclassify the Codex controlled trial under Rung 2

Recorded: 2026-10-06

Independent critique found that the Trae-first ladder conflicted with the
existing controlled trial's Codex-only PASS condition. The owner had selected
TraeCLI for Rungs 0 and 1. This decision makes the relationship explicit. Rung 1
is a TraeCLI analogue that inherits the existing concurrency, capacity, stale
Candidate, rework, fault, evidence and acceptance scenarios while replacing its
provider requirement.

The original Codex lead-plus-two-workers PASS condition moves to Rung 2. Existing
Codex evidence remains Codex evidence and is never relabeled as TraeCLI proof.
After Codex parity, Rung 2 adds mixed-provider workers. This supersedes only the
provider ordering in `docs/efforts/sqlite-controller/CONTROLLED-TRIAL.md`.
