# Harp theorem assurance execution record

## Candidate and authority

Base commit: `2e5d57da339d83b90bb631e0b1939958bd7369a6`.
Branch: `codex/harp-mathematics-spec`.
Kata issue: `dp5z`, related to `00zs` and `64xv`.
Implementation authorized on 2026-09-07. No external outreach, merge, push or deployment.

Current state: `locally-audited`. Focused mathematical and correspondence
reviews, compiler evidence and local repository/presentation acceptance passed.
An agent review is not external human review.

## Baseline execution

- Pinned toolchain file: `leanprover/lean4:v4.32.1`.
- Canonical cache ancestry, packages symlink and Mathlib.olean checked before Lake.
- Feature cache linked to the primary checkout's canonical .lake.
- `mise run lean-env`: exit 0.
- `scripts/check_lean_library.sh CrouzeixHarp`: exit 0, 3395 jobs,
  66 seconds reported by the maintained runner. Existing deprecation and
  unnecessary-simpa warnings were replayed.
- Execution was local and non-hermetic. No dependency hydration or update.
- Temporary baseline log: `/tmp/harp-math-baseline-lean.log`.
  This local path is a convenience, not retained immutable proof evidence.

## Inspection checklist

The implementation owner inspected the following source contracts. A final
disposition also requires the independent mathematical and correspondence
reviews and the new compile checks.

| Check | Source owners inspected | Review disposition |
| --- | --- | --- |
| Norm and statement | Euclidean, Definitions, Statements | Expanded comparison compiled; both agent reviews passed |
| Positive weights and mass | PositiveCubature, FiniteMeasureCubature, FiniteAtomicDilation | Both agent reviews passed after boundary-data repair |
| Embedding and moments | FiniteAtomicL2Dilation, BoundaryEmbedding, CompressionMoments | Both agent reviews passed |
| Fixed core and horizon witnesses | FiniteHorizonDilation, FiniteAtomicL2Dilation | Both agent reviews passed |
| Recurrence and displacement sign | FiniteHorizonOperatorRecurrence, FiniteHorizonPerturbation | Both agent reviews passed after explicit-identity repair |
| Normalization and limits | Harp.MainTheorem, Limiting, OuterApproximationLimit | Both agent reviews passed |
| Shared support and transported scope | Harp.Consequences and explicit LS imports | Both agent reviews passed after pole-restriction repair |

Technical exposition belongs in the canonical manuscript, not this record.

## Repository acceptance

Fresh compiler evidence, the immutable selected-bundle refresh and source
verification passed. Managed-document registration, actual desktop/phone reader
checks and rebuilt PDF checks also passed; see remainder-review.md for the
shared integration evidence. The final mise run verify exited 0, including
the 8831-job aggregate Lean build and repository verification. The proof-lab
suite had two documented environment/state-conditioned skips; details are in
remainder-review.md. No terminal proof or historical evidence was changed.
External human review remains pending and is not part of local completion.

## Focused acceptance evidence

The implementation agent observed missing-declaration failures for the
initial six audit clients and subsequent norm/nilpotent clients before
implementing their proofs. Final focused builds passed:

- lake build CrouzeixTextbook.HarpStatementAudit: exit 0, 2705 jobs.
- lake build CrouzeixTextbook.HarpStatementAuditTests: exit 0, 2706 jobs.

The independent specification reviewer separately ran lake env lean on
both files with combined exit 0. The 31 printed theorem axiom closures
contained only propext, Classical.choice and Quot.sound. That reviewer
inspected the project import closure through NumericalRange, Statements,
Definitions and Euclidean to Mathlib and found no terminal import or project
axiom. A separate quality reviewer inspected the proofs and rejection
fixtures and accepted them without findings. All execution was local,
non-hermetic.

Reviewed source SHA-256 values:

- HarpStatementAudit.lean:
  6570dc268d7d6e602abf957feef9273cab8c4d2546374d015196bb4ac27e7f85
- HarpStatementAuditTests.lean:
  8fa4a15dada9f6baf8e93cc1a21fe81fcb1282a8b1899c0097d52d1953a9b8dc

The manuscript's mathematical reviewer required explicit boundary/companion
construction, the recurrence identity before square completion, and the
rational pole restrictions. All were added and re-reviewed successfully.
The subsequent quality/correspondence reviewer required continuity of point,
normal and speed and the resulting integrability of F. The repaired prose
passed re-review. These are agent reviews, not human peer review.

The parent ran the existing corpus registration baseline, exit 0, and
observed the new supplement test fail first for a missing registered audit
document. After registration it correctly failed on the not-yet-implemented
operator application source link. Once the actual application existed, the
test passed, exit 0. The fresh maintained textbook root build then compiled
3721 jobs, including every new fixture, and published six ledgers; its
canonical-input check passed. The three focused Atlas suites passed 27 tests.

A fresh integration reviewer inspected all six new Lean files, both
supplements and cited providers, registration, root imports, PDF changes and
the unchanged terminal/coverage/exercise files. Source review and diff check
passed. This was inspection, not an additional compiler or full-gate run.
