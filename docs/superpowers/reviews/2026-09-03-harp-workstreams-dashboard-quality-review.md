# Harp Workstreams dashboard final quality and security review

## Review identity

- Reviewed commit: `2b97af433de36a4ba5f3cda75f2d51abe2f82abd`
- Reviewed tree: `c30c45af45ae1c21ab10adc0ab6ac58abe0d03e5`
- Base commit: `91e57411939c7a19ac1e694aaaab1f9f3a8b244a`
- Reviewer: Independent quality reviewer / TRAE subagent
- Model: unavailable to the controller
- Verdict: PASS
- Ready: Yes

There are zero Critical findings and zero Important findings in the cumulative
feature between `91e57411939c7a19ac1e694aaaab1f9f3a8b244a` and
`2b97af433de36a4ba5f3cda75f2d51abe2f82abd`.

## Scope reviewed

This final review covers the full Workstreams dashboard feature and the related
Crouzeix evidence-path hardening that landed in the same candidate range:

- Atlas dashboard UI, routing, offline export, accessibility, and browser-only
  command queue behavior under `atlas/src/app/`, `atlas/src/content/`,
  `atlas/src/styles/`, and `atlas/tests/`.
- Python evidence and cache-path handling under
  `labs/crouzeix_proof_reproduction/`, including route validation, route
  publication, receipt validation, promotion semantics, and XDG environment
  helpers.
- Rust temporal-parity adjustments under
  `crates/harp/src/sources/crouzeix.rs` and related test coverage.

## Findings

### Critical

None.

### Important

None.

### Minor

1. The historical landed-evidence `git show` path intentionally has a larger
   stdout cap than ordinary Git commands. The exception is call-specific and
   remains bounded to the existing 1 MiB artifact limit.

## Why the candidate is ready

### Atlas dashboard and export boundary

The Atlas feature remains bounded to static, local presentation logic.
`parseWorkstreamsSnapshot()` in `atlas/src/content/workstreams.ts` validates the
snapshot contract before rendering, and the session queue in
`atlas/src/app/workstreamsSession.ts` stays fail-closed on malformed storage
state, read exceptions, and overlong command text. `AgentVelocityCard.tsx`
surfaces storage failures as visible UI state instead of silently dropping user
input.

Routing remains intentionally isolated. `AtlasApp.tsx`, `routes.ts`, and their
tests preserve the reader shell for reader routes while returning a dedicated
Workstreams shell for the dashboard. The feature does not introduce process
execution, live backend coupling, or network capability: the browser command
surface appends validated local records only.

The offline boundary is reinforced by `atlas/tests/static-export.test.mjs` and
the checked-in export receipt. The reviewed export remains self-contained, the
static scanner rejects external fetch/import/resource patterns, and the final
artifact receipt matches the exported HTML. Browser and accessibility coverage
were explicitly reported for `1440`, `1024`, and `390` widths with offline zero
Axe/network regressions.

### Route cache hardening is complete

The final route cache repair in
`labs/crouzeix_proof_reproduction/route_validation.py` closes the TOCTOU issues
found in earlier review rounds.

Accepted final properties:

- `CacheRootIdentity` binds device/inode identity for the approved cache root.
- `resolve_approved_cache_root()` binds the initial root before canonical-path
  checks, then compares the observed root identity after reopening so pre-bind
  replacement cannot slip past validation.
- `_open_root_directory()` walks the root path with no-follow semantics instead
  of trusting a later string reopen.
- `_read_rooted_bytes()` verifies the file against the same opened-root
  identity, preserving descriptor-relative reads and preventing root-boundary
  escape after validation.

The paired change in `route_publication.py` fixes canonical repository-root
handling for `/var`-style temporary roots without reopening the cache API
boundary. The final route-cache verdict for this candidate is Ready `Yes`.

### XDG cache identity (pre / during / post replacement)

The XDG exception and cache-root identity model were reviewed explicitly as a
three-step boundary:

- **Pre-replacement binding:** the cache root is identified by stable identity
  (device/inode) before any canonicalization checks. This prevents an attacker
  from swapping the root path after an early lstat but before reads.
- **During validation:** root traversal and member reads are descriptor-relative
  and no-follow, so the validated root descriptor remains the authority
  boundary.
- **Post-validation check:** the implementation compares the observed root
  identity after reopening/validation so root replacement between phases does
  not silently succeed.

This identity-based boundary is the reason the review does **not** treat a local
cached compile as equivalent to hermetic proof execution: it is a strict local
boundary, not a hermetic build system.

### LS temporal validation split is correct

The receipt validation change now separates two distinct policies instead of
overloading one rule:

- Historical committed evidence validation in
  `labs/crouzeix_proof_reproduction/ls_validation.py` uses
  `validate_committed_receipts(..., require_current_wrapper: bool = False)` so
  prior receipts remain checkable even if the current wrapper bytes have
  changed.
- Promotion-time validation in
  `labs/crouzeix_proof_reproduction/ls_promotion.py` passes
  `require_current_wrapper=True`, which blocks stale-wrapper promotion before
  mutation.

That split preserves provenance checking for historical receipts while keeping
the mutating promotion path strict. It fixes the earlier stale-wrapper gap
without weakening the ordinary evidence model.

### Refreshed three-file proof bundle

This candidate includes a refreshed local formalization evidence bundle for the
Crouzeix workstream. The review treated these artifacts as *evidence outputs*
and verified they remain consistent with the repository’s validators and receipt
paths (not as new theorem claims):

- `evidence/crouzeix_conjecture/local_formalization/build/command.json`
- `evidence/crouzeix_conjecture/local_formalization/build/stdout.log`
- `evidence/crouzeix_conjecture/local_formalization/manifest.tsv`

### XDG cache admission remains narrow

The XDG `.lake/packages` exception remains narrowly scoped. The reviewed Python
path-handling code admits only the exact XDG packages symlink for the pinned
Lean cache location, requires raw and canonical targets to agree with that
approved path, and still rejects arbitrary or nested symlink structure below the
approved packages root. Reads remain descriptor-relative and no-follow.

The surrounding helper updates in `protocol.py`,
`local_formalization_evidence.py`, `proof_evidence.py`,
`scripts/check_lean_library.sh`, `scripts/harp_xdg_env.sh`, and the task-graph
tests correctly model an existing local Lean/XDG installation. They do not
establish hermetic Lean execution on macOS, and this review does not claim that.
What they do provide is a tighter validation boundary around the supported local
environment and a clear split between validation-only paths and explicit cache
hydration paths.

### Rust change stays scoped to temporal parity

The Rust delta in `crates/harp/src/sources/crouzeix.rs` was reviewed against the
Python temporal semantics and remains scoped to parity for committed-vs-current
wrapper handling. The change does not broaden evidence admission generally and
does not undermine the stricter promotion path.

### Public-history scan and sanitization

This review incorporated the reported public-history checks and treated them as
non-negotiable safety gates:

- Scan found no requested private markers in diff/history.
- Feature tree hash unchanged through history sanitization.
- No theorem source/claim changes were introduced in this feature range.

### Historical landing evidence remains bounded

The final goal-validation repair recognizes that two landed verifier artifacts
are designed to evolve after their original program phases: the CPFR-088 local
formalization manifest and the CPFR-089 Atlas receipt. Only those two phases may
recover historical verifier bytes from the recorded landing commit. The
fallback still requires an existing commit that is an ancestor of `master`, the
exact registered artifact path, and a SHA-256 match against the immutable ledger
row.

Ordinary Git commands retain the 8 KiB stdout cap. The historical artifact
`git show` call alone uses the existing 1 MiB artifact ceiling; stderr and the
process timeout remain unchanged. The regression records a CPFR-088 artifact
larger than 8 KiB, mutates the current CPFR-088 and CPFR-089 files, and proves
that the exact committed bytes are recovered without broadening other phases.

## Evidence considered

The cumulative review incorporated the user-provided verification summary for
the final candidate:

- Atlas: `116`
- Browser: `1440/1024/390`
- Offline audit: zero Axe/network findings
- Python: `228`
- Route validation: `62 skip1`
- Route publication: `35`
- Rust: `107 fmt/clippy`
- Lean wrapper integration fixture: `23/23`; format and warning-as-error clippy passed
- All routes: `complete-local`
- Python bundle: `pass`
- Rust sources: `481/189`
- Goal-validation module: `27/27`, including real complete-goal CLI, read-only
  execution, ordinary Git bounds, and oversized historical evidence
- History tree preserved
- Private marker scan clean
- Refreshed three-file proof bundle (command/stdout/manifest) present and
  included in the candidate tree

These results are consistent with the code review conclusions above and with the
absence of any remaining open review findings in the final candidate.

## Review method

The review re-read the cumulative diff and final on-disk sources for the
feature, with focus on:

- Atlas dashboard components, presentation helpers, snapshot parsing, session
  queue storage boundaries, route selection, CSS scoping, and static export
  guards.
- The generated `atlas/dist/harp-atlas.html` and
  `atlas/dist/harp-atlas.receipt.json`.
- Python route validation, route publication, receipt validation, promotion, and
  XDG environment helpers plus their regression tests.
- Rust Crouzeix temporal-parity logic and related tests.
- Historical landed-evidence recovery, its commit/ancestry/path/digest bindings,
  and the call-specific artifact output bound.

## Final verdict

`2b97af433de36a4ba5f3cda75f2d51abe2f82abd`
(`c30c45af45ae1c21ab10adc0ab6ac58abe0d03e5`) is acceptable to ship relative to
`91e57411939c7a19ac1e694aaaab1f9f3a8b244a`.

Verdict: `PASS`

Ready: `Yes`
