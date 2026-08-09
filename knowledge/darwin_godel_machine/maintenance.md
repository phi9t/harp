---
id: dgm-maintenance
title: DGM packet maintenance
type: runbook
status: active
created: 2026-08-08
updated: 2026-08-08
tags: [darwin-godel-machine, maintenance, provenance, verification]
confidence: high
canonical: ../../content/systems/dgm.md
---

# DGM packet maintenance

> This file is a learning projection. Canonical claims live under `content/`;
> primary-source captures and pinned implementation files live under
> `evidence/`.

## Ownership

The packet under `knowledge/darwin_godel_machine/` is a learning projection.
It does not own material technical claims.

Authority order:

1. `content/` owns Harp technical prose and claim boundaries.
2. `evidence/` owns captured upstream bytes, provenance, licenses, and pinned
   source snapshots.
3. `knowledge/darwin_godel_machine/` owns navigation, pedagogy, exercises, and
   source-linked learning routes.
4. generated Atlas JSON and HTML are derived from canonical `content/`.

A packet-only claim is a maintenance defect. Add or update canonical content
first, then project it here.

## Packet contract

The packet contains exactly sixteen Markdown files:

```text
darwin_godel_machine_index.md
01_orientation.md
02_paper_walkthrough.md
03_algorithm_derivation.md
04_system_architecture.md
05_repository_walkthrough.md
06_evaluation_analysis.md
07_open_endedness.md
08_safety_and_failure.md
09_critical_review.md
10_successor_design.md
learning_path.md
glossary.md
claim_evidence_crosswalk.md
source_registry.md
maintenance.md
```

Do not add `README.md`. The named index is the Obsidian and repository
entrypoint.

Every file must:

- start with YAML frontmatter;
- include the exact learning-projection authority notice;
- link back to `darwin_godel_machine_index.md`, except the index itself;
- use repository-relative links for local artifacts;
- avoid absolute local paths;
- distinguish `EVIDENCE`, `CLAIM`, `INFERENCE`, and `MISSING`; and
- retain the relevant claim ceiling and reproduction status.

## Immutable source identities

### Paper

Current packet basis:

```text
arXiv: 2505.22954v3
source ID: DGM
capture: evidence/weng/text/dgm.txt
```

The canonical source registry owns the digest and publication-state record:

[content/sources/source_registry.tsv](../../content/sources/source_registry.tsv).

### Repository

Current packet basis:

```text
source ID: DGM-REPO
revision: a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2
```

Receipts:

- [REVISION](../../evidence/implementations/dgm/REVISION)
- [REMOTE](../../evidence/implementations/dgm/REMOTE)
- [LICENSE_STATUS](../../evidence/implementations/dgm/LICENSE_STATUS)
- [implementation manifest](../../evidence/implementations/manifest.tsv)

Do not edit captured snapshot files for prose, formatting, or branding.

## When the paper changes

If a new paper version appears:

1. capture the new source through Harp's evidence workflow;
2. preserve the old bytes and receipts;
3. register the new immutable identity and digest;
4. compare sections, figures, tables, appendices, and references;
5. identify changed claims, not only changed wording;
6. update canonical `content/systems/dgm.md`;
7. update source and evidence graph records;
8. update the packet's paper walkthrough, evaluation table, safety analysis,
   claim crosswalk, and source registry;
9. keep old result values if they remain historically relevant and label the
   version; and
10. regenerate derived Atlas artifacts.

Specifically recheck:

- main scores;
- task counts;
- model assignments;
- ablations;
- costs;
- transfer results;
- safety claims;
- objective-hacking case;
- limitations; and
- future-work status.

## When the repository changes

Do not replace the snapshot in place without a new revision receipt.

For a new repository revision:

1. record the remote and commit SHA;
2. verify the source-specific license;
3. choose the narrow file set required for packet claims;
4. capture files byte-for-byte;
5. update the implementation manifest with digests and byte counts;
6. compare runtime behavior with the previous snapshot;
7. classify changes as:
   - paper-alignment change;
   - bug fix;
   - evaluator change;
   - containment change;
   - workflow change;
   - model/API change; or
   - documentation-only change;
8. update canonical implementation claims;
9. update the walkthrough and architecture;
10. preserve old snapshot evidence when needed for historical claims.

Recheck the two current release-snapshot defects:

- CLI concatenation of `score_child_prop` and `best`; and
- ascending selection in the direct `best` branch.

If fixed upstream, report:

- old behavior at the old revision;
- new behavior at the new revision; and
- whether the historical experiments used either path.

## Paper and code disagreement

When paper and source differ:

1. state both;
2. cite each source separately;
3. do not infer which produced historical results without provenance;
4. cap the paper claim at author-reported method/result;
5. cap the code claim at behavior of the pinned revision; and
6. record what evidence would reconcile them.

Examples:

- paper eligibility equation versus released candidate filtering;
- paper iteration wording versus release generation parallelism;
- paper sandbox description versus visible Docker configuration;
- paper's self-analysis language versus external diagnostic-model stage.

## Quantitative update checklist

Every result table row needs:

- benchmark;
- task scope;
- metric;
- base and treatment values;
- model role and identity;
- source locator;
- selection context;
- reproduction status; and
- any source inconsistency.

Do not silently merge:

- 50-task Polyglot search score with full-benchmark score;
- 60-task SWE-bench estimate with 200-task score;
- pass@1 with pass@2;
- search-time model with transfer model;
- one selected run with the three-run statistic;
- task-agent spend with root-tree spend.

The current Claude 3.7 SWE-bench transfer mismatch must remain visible:

```text
Figure 4 label: 59.0%
nearby prose: 59.5%
```

Resolve it only with an authoritative correction or raw result artifact.

## Reproduction update checklist

If Harp performs a reproduction, record:

- paper and source revision;
- candidate manifest;
- model endpoint and immutable identity if available;
- API observation date;
- dependency lock;
- container-image digest;
- benchmark revision;
- exact task IDs;
- metric implementation;
- random seeds;
- retries and parallelism;
- complete root-tree resource usage;
- raw artifact locations;
- expected and observed results;
- deviations from the paper;
- failures and interrupted runs; and
- independent review status.

Do not change `Not independently reproduced` to `reproduced` because:

- the code compiled;
- one task ran;
- a source path matched the paper;
- a final candidate score was copied from upstream logs; or
- a verifier passed.

Reproduction means the relevant mechanism and result were executed under a
declared protocol.

## Canonical update sequence

For a material technical change:

1. update evidence and receipts;
2. update `content/sources/source_registry.tsv`;
3. update `content/sources/evidence_graph.tsv` if relationships change;
4. update `content/systems/dgm.md`;
5. update `content/claim_evidence_ledger.md` if the canonical claim set changes;
6. update this learning packet;
7. update the claim crosswalk;
8. regenerate Atlas JSON and HTML together; and
9. run full verification.

Do not make generated JSON or HTML the source of a prose change.

## Link and source checks

Run:

```sh
cargo test -p harp --test dgm_knowledge_packet
```

The packet test verifies:

- exact file set;
- no `README.md`;
- frontmatter keys;
- authority notice;
- index backlinks;
- repository-relative links;
- absence of local absolute paths and placeholders;
- pinned implementation revision;
- claim-crosswalk schema; and
- packet source IDs.

This test checks packet integrity. It does not validate every technical claim.
Reviewers must still inspect source locators and accounting boundaries.

## Focused content checks

```sh
rg -n 'a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2|Reproduction status|objective hacking|next-cycle' \
  knowledge/darwin_godel_machine
```

Expected:

- pinned revision appears in implementation-facing documents;
- result tables retain reproduction status;
- objective hacking remains in safety and evidence routes; and
- the missing next-cycle comparison remains explicit.

The packet integrity test also rejects unresolved authoring markers and
absolute local paths. Keep that rule in the test rather than copying marker
spellings into packet prose.

## Canonical and generated checks

```sh
cargo run -p harp -- check
cargo run -p harp -- build --check
cargo run -p harp -- sources verify
```

If canonical content changed, regenerate through:

```sh
cd atlas
corepack pnpm run export:html
```

This rebuilds the canonical corpus and the offline single-file Atlas export.
Do not hand-edit:

- `atlas/src/content/generated/corpus.json`;
- `atlas/dist/harp-atlas.html`; or
- `atlas/dist/harp-atlas.receipt.json`.

## Full verification

Before committing:

```sh
mise run verify
```

Also run:

```sh
git diff --check
git status --short
```

Inspect the actual diff. A green verifier is not a substitute for confirming
that:

- every packet requirement is present;
- no unrelated work was overwritten;
- captured evidence bytes remain unchanged;
- generated artifacts correspond to canonical inputs; and
- claim ceilings remain honest.

## Review matrix

| Review question | Artifact |
|---|---|
| What changed in the paper? | Paper diff and source registry |
| What changed in code? | Snapshot diff and implementation manifest |
| Which canonical claim changed? | `content/systems/dgm.md` |
| Which packet route changed? | Packet diff |
| Are all numbers scoped? | `06_evaluation_analysis.md` |
| Are all claims traceable? | `claim_evidence_crosswalk.md` |
| Did authority boundaries change? | `04_system_architecture.md` and `08_safety_and_failure.md` |
| Did the recursive claim strengthen? | Direct next-cycle experiment receipt |
| Was anything reproduced? | Reproduction manifest and raw artifacts |
| Are generated readers current? | `build --check` and Atlas export receipt |

## Claim-promotion rule

Promote a packet statement from `MISSING` or `INFERENCE` only when the new
evidence directly measures it.

Examples:

- A higher task score does not promote successor improvement.
- A larger archive does not promote behavioral diversity.
- Docker execution does not promote sandbox security.
- A source snapshot does not promote historical experiment identity.
- A repeated author result does not promote independent reproduction.
- A candidate producing a child does not promote increasing improvement yield.

## Removing or retiring material

When a statement becomes obsolete:

1. determine whether it is historically important;
2. preserve source identity and old bytes;
3. update canonical prose with version scope;
4. remove stale teaching only after replacement links exist;
5. retain negative or contradicted evidence in the claim ledger;
6. update packet navigation; and
7. rerun link and full verification.

Do not rewrite captured evidence to make it agree with current terminology.

## Packet completion audit

Before declaring a packet update complete, answer:

1. Does every material claim have a canonical home?
2. Does every paper result name task and metric scope?
3. Does every code claim name the pinned revision?
4. Are paper, source, and packet evidence classes distinct?
5. Are source inconsistencies visible?
6. Is non-reproduction explicit?
7. Is the candidate/protected-envelope boundary current?
8. Is objective hacking still represented?
9. Is the missing next-cycle test explicit?
10. Do all sixteen files pass the integrity test?
11. Are generated Atlas artifacts current if canonical content changed?
12. Did `mise run verify` pass on the actual final tree?

If any answer is uncertain, the update is not complete.

Inspect the [source registry](source_registry.md) when evidence identities
change.

Back to the [DGM index](darwin_godel_machine_index.md).
