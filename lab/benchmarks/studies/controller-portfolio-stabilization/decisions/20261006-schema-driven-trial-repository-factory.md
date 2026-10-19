# Build Trial Repositories from declarative templates

Recorded: 2026-10-06

The owner approved a schema-driven Trial Repository factory rather than static
repository archives or imperative setup scripts. Each immutable Trial Template
declares seed files and modes, deterministic commit history, branches and
protected refs, work-ledger items and blockers, tests, acceptance contracts,
fault hooks and expected identities.

Materialization creates a content-addressed private Git repository and isolated
work-ledger namespace and emits a receipt binding the template identity, tool
identities, generated Git objects, refs and run namespace. Repeated
materialization of the same template under the same admitted toolchain must
produce the same semantic seed identity. Evidence is retained before teardown.
