# Crouzeix conjecture evidence provenance

Captured at: `2026-08-14T23:44:51Z`

This bundle keeps upstream proof artifacts remote-only. `source_manifest.tsv`
records immutable identities, byte counts, SHA-256 digests, rights status, and
semantic source paths. It does not vendor manuscript, PDF, TeX, Lean, prompt,
or repository bytes.

`acquire.sh` created fresh detached checkouts for the two Jin revisions,
verified remote and Git identity, ran each revision's `Lean/verify.sh`, and
performed a metadata-only tracked-source scan. Build logs replace the checkout
root with `<source-checkout>`, abort on secret-like text or output above 2 MiB,
and collapse any detected source excerpt to metadata and digests. Scan logs
contain only path, line number, token class, and line SHA-256.

Preprints.org returned status `403` on `2026-08-14`. The
receipt is metadata-only unless status 200 produced measured bytes. No claim
maps the posted manuscript to a Git revision without byte equality.

The five prerequisite DOI records are metadata-only publisher routes. Their
presence does not prove access to full text, reproduce a theorem, or establish
the correctness of either candidate proof.
