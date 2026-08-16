# Lean proof-engineering evidence provenance

This evidence bundle records a bounded, depth-one view of four primary routes:
Mistral's dated Leanstral 1.5 announcement, Lean4Agent arXiv revision v2, a
resolved immutable `leanprover/lean4` commit, and the Lean "Learn" landing
page. The closure accepts the Lean repository README and LICENSE plus the four
core learning routes named in `closure.tsv`. The Lean4Agent PDF is a root
capture, not a closure child. All other observed content candidates are
explicitly deferred or marked not needed; the acquisition script never follows
them.

The Lean 4 README and LICENSE are pinned to commit
`57eb1ae3d0d440f29d1f35e9699c6df4d46c2620` and use commit-scoped raw GitHub
URLs. The Mistral and Lean documentation pages are dated observations captured
on 2026-08-15, not immutable versions. The Mistral page itself displays
2026-07-02. Lean4Agent is fixed to `arXiv:2606.06523v2`; the alphaXiv route that
resolved to v1 is recorded as a revision mismatch and was not fetched.

Files beneath `artifacts/` and `metadata/` preserve the exact response bytes.
The file beneath `text/` is a derived sidecar recorded with the exact manifest
note `pdftotext -raw; PDF is source of record`. `manifest.tsv` records rights
and redistribution status per capture. Unknown page licenses remain unknown
rather than being inferred.

`acquire.sh --check` is offline and read-only. It checks byte counts, digests,
headers, receipt bindings, closure limits, symlinks, and PDF identity without
rewriting captured or derived bytes. Network acquisition is limited to the URLs
in `source_lock.tsv`, ignores ambient curl configuration, permits HTTPS sources
and HTTPS redirects only, caps every response at 16 MiB, and stages each
response as a `.part` file before renaming it into place. Refresh is rejected
after the dated lock identities expire. It is exclusive, keeps the prior bundle
until the published replacement passes its offline check, and restores the
prior bundle after failure or interruption.
