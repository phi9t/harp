# GitHub release readiness planner

## First milestone

Prepare a host-native, local-first release candidate from a verified commit.
The owner approved implementation and a subsequent GitHub landing. The GitHub
destination is not configured in this checkout and must be supplied before upload.

- [x] Prepare locked dependencies using Mise and link the existing warm Lean cache.
- [x] Implement `mise run release-candidate`: require a clean checkout, run the
  full `mise run verify` gate, build the native size-profile executable with the
  Cargo lockfile, package it, and smoke-test the extracted `harp --version`.
- [x] Emit a commit/version/target manifest, SHA-256 checksums, and release notes
  under ignored `dist/`. Keep the manifest outside the archive so its archive
  digest is not self-referential. Refuse to overwrite an existing candidate.
- [x] Document local installation and the boundary between the executable,
  repository content, external providers, and Lean dependencies.
- [x] Verify rejection of dirty inputs, failed gates, changed inputs, unsafe
  output paths, wrong executable versions, and existing output.
Release acceptance (the candidate manifest records the completed gate):

- Refresh the import receipt after staging the settled payload; run the
  full repository gate; commit the focused release-readiness slice.
- Produce and verify a candidate from that clean commit.
- Resolve the GitHub destination, authentication, and the separate publication branch before
  uploading repository history or making a public release.

Implementation lives in `scripts/release_candidate.py`; behavior tests live in
`scripts/tests/test_release_candidate.py`. `mise.toml` registers the production
entry point and its tests. `docs/releasing.md` is the operator interface.
Use `python3 -m unittest discover -s scripts/tests -p test_release_candidate.py -v`
while iterating and `mise run verify` for the final gate.

Independent review caught an archive-permission mismatch. The archive now
records mode 0755 for the executable, and smoke verification uses the archived
mode. Ten focused packaging tests cover the initial milestone, including that
regression. No Kata executable is available in this session; this planner is
the durable record until the work can be reconciled with the repository ledger.

## Deferred architecture improvements

These are follow-up work, not prerequisites for the first native candidate.

| Follow-up | Evidence and intended boundary | Acceptance evidence |
| --- | --- | --- |
| Engine activity lifecycle | Fresh dispatch in `crates/harp-engine/src/scheduler.rs` and restart handling in `recovery.rs` repeat lifecycle rules; cancellation also spans scheduler and cancel modules. Keep Engine as the single durable executor described by ADR 0002; consolidate transitions behind one lifecycle interface. | Existing execution/recovery tests remain green; tests cover crash windows, cancellation, and persisted-before-effect ordering through that interface. |
| Evidence verification | `crates/harp/src/sources.rs` combines family dispatch, reports, materialization, manifests, and Git access. Separate cohesive private verifiers while preserving the CLI/report contract. | All source families retain strict digest, provenance, license-status, and offline validation behavior; malformed-input tests exercise the public verification boundary. |
| Repository payload capture | `crates/harp/src/repository.rs` enumerates and reads tracked payload separately for forbidden-reference scanning and digesting. Capture one validated immutable payload shared by both operations. | Tests reject unsafe paths/symlinks and prove scanning and hashing use identical bytes, including when filesystem inputs change. |

## Publication boundary

The existing `feat/public-repository-sanitization` branch contains a separate
public roster, source migration, and licensing changes. It has not been merged
into this milestone. Do not infer that a passing technical gate authorizes
redistributing all captured evidence or the full historical Git object graph.
The owner must select the publication tree and license policy; preserve the
current source-specific licenses and explicit missing-license records.
