# Approved versioned local proof evidence

Owner approved implementation, full verification and landing after the initial
immutable-evidence blocker. The original bundle and all route receipts/reviews
remain unchanged. New execution is local compilation evidence, not a new
independent mathematical review.

## Wire and filesystem contract

Keep the legacy `evidence/crouzeix_conjecture/local_formalization/` untouched.
New immutable bundles live under `local_formalization_generations/<id>/` beneath
the same evidence root, where id is exactly 32 lowercase hexadecimal characters.
An optional `local_formalization.current.json` selects the active bundle. Without
it, existing legacy validation remains unchanged. A malformed pointer fails;
it never falls back to legacy evidence.

The strict pointer has exactly `schema_version`, `active_generation` and
`generations`. Schema is `crouzeix-local-formalization-selection/v1`.
Each generations entry has exactly `id` and `bundle_sha256`. IDs are unique;
`legacy` must occur exactly once and maps to the original bundle. The active ID
must occur in the catalog. New entries use the hexadecimal ID above. At most
128 entries are accepted. The pointer is bounded to 64 KiB and is a regular,
single-link file; unknown fields, duplicate JSON keys and symlinks are rejected.

Bundle digest is SHA-256 of UTF-8 records `relative-path<TAB>sha256<LF>` for every
regular file in the bundle, sorted lexicographically by relative POSIX path.
Paths contain neither tabs nor newlines. Reject symlinks, hardlinks and special
files, with the existing bounded bundle inventory limits. Historical bundles
are checked for byte integrity, not against today's build inputs. Only the
selected bundle undergoes the existing full live source/build/audit validation.
The exact evidence roster includes all and only catalogued bundles and pointer.

## Publication behavior

`publish-local` remains create-only and backwards compatible. Add `refresh-local`
which captures a fresh Crouzeix aggregate build and six declaration audits via
the existing executor, validates unchanged input snapshots, publishes the new
bundle create-only, then atomically selects it under the publication lock.
Manifest member locators name the actual generation, never the legacy directory.
Existing route evidence and candidate commit checks remain enforced.

Failure before selecting preserves the old pointer and every old bundle.
If the new immutable bundle is already visible when selection fails, report a
recovery-required result with its exact path; never silently delete evidence or
report success. Reject unregistered preexisting generation directories. No
dependency hydration, evidence rewriting or proof-status widening is permitted.

## Implementation and acceptance plan

1. Python publisher/selector seam: failing tests for two refreshes preserving
   legacy bytes, fresh executor calls, bad pointers, symlinks, tampering,
   concurrent publication and failed selection. Implement bounded generation
   support by reusing the current executor and publication safety checks.
2. Rust verifier seam: failing fixture tests for selected fresh evidence alongside
   stale historical evidence, invalid active IDs/digests, unknown generation
   directories and active source drift. Preserve all existing legacy tests.
3. CLI seam: expose refresh-local with typed committed/recovery diagnostics; test
   dispatch without real Lean. Integrate both languages against the contract.
4. Review Standards and Spec independently; run focused Python/Rust tests. Freeze
   code, preflight the pinned warm cache, capture one real fresh generation and
   verify it through the Rust CLI. Preserve a hash inventory of legacy bytes.
5. Regenerate textbook publication and Atlas/PDF outputs if canonical inputs
   changed. Refresh the import receipt last, run mise run verify, commit only
   after success, fast-forward master and push normally. Keep whole-book open.
