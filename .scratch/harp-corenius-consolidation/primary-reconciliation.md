# Existing primary work: reconciliation

Owner instruction: reconcile existing Harp work and land, 2026-10-18.
Source snapshot: 85fdb248a155bf4c7fea2cdfae824f86f7c031aa, parent 84fc9bec.
All 63 changed/untracked paths are preserved exactly in that snapshot; it is a
merge parent of the integration candidate. The original primary files and index
were left unchanged until verified landing.

## Dispositions

- Retain the sanitizer CLI/module and explicit verification task. Repair symlink
  and hardlink mutation, atomic file replacement, line-ending preservation,
  idempotent counts, and staged-index hook behavior. The hook is optional and is
  not installed. Repository-wide scanning is opt-in; it does not rewrite captured
  Corenius/evidence bytes or become an unconditional import gate.
- Retain redactions on surviving maintained documents and the Crouzeix, FLT and
  multi-agent engineering plans. Preserve malformed sanitizer edits to historical
  math/training examples in the snapshot, but retain their valid candidate forms.
- Keep Atlas/RSI-only paths retired. Redactions of their obsolete registries and
  projections do not resurrect the product. Regenerate the surviving corpus and
  import receipt instead of merging stale exports.
- Preserve the two educational clippings byte-for-byte under
  evidence/education_clippings/artifacts with digests and explicit unknown
  license status. One source is a private conversation; no external publication
  or new source verification is claimed.
- Preserve the raw planning transcript under evidence/session_notes/artifacts,
  as historical session material rather than contributor instructions.
- The zero-byte duplicate knowledge/knowledge note is editor scratch; it remains
  recoverable in the snapshot rather than maintained as a second proof source.
- Preserve the historical proof execution ledger over redactions of its recorded
  identities. The proposed redactions remain in the source snapshot.
- Retain the verified proof-lab source over two behavior-identical string-splitting
  edits. No proof, receipt identity or captured evidence byte changed.

## Verification

Independent review found no actionable defect in the bounded sanitizer repair.
Fifteen sanitizer cases, four staged-hook cases, a real CLI hook smoke check and
strict library Clippy passed in the repair worktree. The integrated CLI additionally
checks stdin redaction/UTF-8/CRLF and read-only checking followed by idempotent scrub.
The final changed-surface integration gate is pending.
