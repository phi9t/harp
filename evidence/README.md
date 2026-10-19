# Evidence and provenance

This directory records where retained material came from and what was checked.
It is not an execution journal or an alternative controller database.

- Topic directories contain pinned source bytes, retrieval metadata, licenses and
  narrow excerpts used by the maintained mathematics and engineering notes.
- `implementations/` contains bounded upstream source snapshots and their
  manifests; these are reference material, not runtime dependencies.
- `crouzeix_conjecture/` contains route manifests, source identities, proof
  receipts and review records. Current verification distinguishes those original
  qualifications from fresh checks of the maintained proofs.
- `provenance/` records source preservation, absorbed Git ancestry and the exact
  inputs/results of migration qualification.
- `engineering_history/` preserves imported design and engineering records.
  Historical instructions and implementations do not define current ownership.
- `sources/` holds the shared source registry; workload-specific contracts live
  with their workload instead of in a generic content directory.

The active ownership graph is documented in
[the repository design](../docs/workstream/repo-architecture/design.md).
Keep captured artifact bytes and source-specific licensing intact. Durable run
state, credentials, private host inventories and unrestricted session captures do
not belong here. Historical receipts retain their original source identities;
they do not automatically qualify the current implementation.
