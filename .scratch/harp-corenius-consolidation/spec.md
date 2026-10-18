# Consolidate Corenius into Harp

Status: done
Owner: Codex
Branch: codex/harp-corenius-consolidation

## Intent and authority

The owner requested consolidation of Harp and Corenius on devx_b200_dbg on
2026-10-18 and selected Harp as the destination, including the runtime,
controller, tooling and engineering practice. Preserve both existing checkouts,
uncommitted work, branches, evidence and live bindings. No push, deployment,
service startup or retirement of the original source
checkouts is authorized.

## Harp scope reduction

On 2026-10-18 the owner also requested removing the RSI research material,
retaining a brief survey of articles previously read and discarding most other
article-related material. The source import preserves all Corenius work; the Harp research product is reduced.

Retain a compact survey with source links and a short takeaway per article.
Do not carry over detailed paper deconstructions, duplicated source cards,
claim/evidence graphs, article captures, or article-specific experiments and
validation solely to preserve the old research product. Extract any operational
runtime contract before removing research prose that currently documents it.

The owner explicitly narrowed removal to RSI-related material. Preserve math,
proof work (especially the Crouzeix proof), SICP as inspiration for agentic
engineering, and other educational material. The owner subsequently explicitly requested deleting the Atlas rather than
installing its missing frontend dependencies. Preserve retained content through
Markdown, proof and publication tools. Shared corpus,
source, search and publication machinery must retain its educational/proof
consumers. The existing primary-checkout changes remain preserved.
Article deletions apply to the isolated candidate; no history rewrite or removal
of the original checkout is authorized.

## Observed baseline

- Harp base: 84fc9bec. The primary checkout has substantial unrelated changes.
- Corenius main: e045d4a6ae2a5efaa53fab93a129b3a57098f7ad.
- Corenius has 13 registered worktrees and 17 branches not merged into main.
- Main has no tracked modifications, but most linked worktrees have unfinished
  changes. Durable execution, retrospective repairs, benchmark exemplars and
  failure analyses are absent from committed main.
- Main contains 8,035 tracked files, totaling 198,679,349 bytes. Most bytes are
  historical evidence, rather than executable source.
- Harp pins Rust 1.92 and rusqlite 0.39. Corenius requires Rust 1.95 and
  rusqlite 0.38. Its workspace also resolves a parked PostgreSQL/Temporal
  experiment when testing the terminal agent.
- Corenius's assembled live two-worker baseline remains unqualified. Migration
  must not promote historical source acceptance into product qualification.
- Neither product supplies a repository-wide license for its own source.

## Additional consolidation decisions

- The owner explicitly declined PostgreSQL/Temporal adoption. Retain existing
  parked source and historical labs with the complete import, but do not enable
  those services, select them as the controller or make their fixture runs a
  prerequisite for the local product.
- Absorb the existing upstream Corenius situation rather than implementing a
  new architecture from the referenced conversations. Useful planner and native
  evidence mechanisms remain candidates for later focused adaptation.
- All original source branches and dirty worktree captures must remain
  recoverable after integration. Do not discard older source merely because
  its current implementation has been superseded.

## Proposed execution sequence

1. Inventory all source worktrees, dirty paths, divergent branches and durable
   efforts. Preserve exact source identities before promotion.
2. Import the committed operational component into an isolated Harp candidate,
   retaining native boundaries and the Corenius Cargo lock initially. Prove
   terminal-agent, controller and collaboration CLI contracts without starting
   paid providers or persistent services.
3. Integrate all Corenius branches and every registered worktree, including
   uncommitted source, with exact provenance. The owner explicitly requested
   taking all worktrees/branches and landing them into Harp. Historical versions
   are retained in ancestry; current overlapping implementations are reconciled
   instead of restoring superseded code.
4. Make Harp own the current operator documentation and engineering policy.
   Harp's local Markdown tracker, no-push rule, typed boundaries and existing
   workflows supersede retired Kata and sibling-policy synchronization.
5. Verify the entire candidate with changed-surface gates and affected consumer
   checks, and leave a reviewable local branch. Human authority controls merge,
   release and retirement of Corenius.

## Acceptance

- Harp retains only a brief article survey from its retired RSI research material.
- Crouzeix and other math/proof work, SICP and other education remain functional;
  their publication, source/evidence and validation contracts survive; the Atlas is retired.
- Harp contains the selected runtime, controller, tools and maintained practice;
  normal operation has no source/build/test/documentation dependency on another
  checkout.
- Runtime and controller ownership is explicit, with one current documented
  entry point for each operation and no contradictory policy authority.
- All unfinished work has an explicit disposition and exact provenance.
- Retained evidence is byte-preserved; licensing status remains explicit.
- Provider permissions, sandboxing, authenticated controller commands, exact
  candidate checking, cancellation and recovery are preserved.
- Required checks pass against migrated code. Existing missing qualifications
  remain stated. Source import alone does not complete the consolidation.

## Evidence and limits

All source worktrees were inspected directly; their original checkouts remain
intact. Linux import tests used existing offline toolchains/caches. The owner
authorized pinned PDF dependencies and the pinned Lean cache restoration.
No paid providers, PostgreSQL/Temporal services, delegated-cgroup qualification
or external publication were used. Retained live-runtime qualification limits
are documented in the operational ticket and integration guide.

## Candidate integration checkpoint

All 29 Corenius branch tips and all 12 dirty-worktree source snapshots are in the
combined source ancestry at 64f55c72a6b7a5489cad22a33a923eb10a2228b9. The clean
main worktree is the thirteenth inventory entry. The candidate contains 8,149
source files under corenius, with source objects and explicit adaptation records.

The import was followed by Atlas/RSI retirement and the bounded benchmark
state-path repair. Terminal/controller/parser and six benchmark suites passed
on Linux; repaired materializer/rootfs/temp-policy checks passed as well.

## Final candidate verification boundary

The candidate removes Atlas and the RSI article product, retains the 54-reading
survey, all math/proof material and SICP, and incorporates every Corenius source
branch/worktree. Independent review confirmed all 47 educational aliases resolve
and no previously compiled retained mathematics/engineering document was lost.
The proof-goal reader now accepts harp-knowledge/v1; its direct real-artifact
check passes. Corpus freshness and search are current.

The complete candidate gate passed at 838c1ca5 after the owner authorized pinned
PDF dependencies and the pinned Lean cache restoration. PDF publication and the
headless educational/proof consumers remain functional. No browser download or
Atlas dependency installation occurred.

The owner then explicitly authorized reconciliation of existing primary Harp
work and local landing. Snapshot 85fdb248a155bf4c7fea2cdfae824f86f7c031aa preserves
all 63 changed/untracked paths. The merged candidate passed its changed-surface
gate: 467 library tests, 16 CLI cases, selector/shell/hook checks, repository,
corpus and documentation checks. Local master fast-forwarded to 9ac9ce75.
See [reconciliation](primary-reconciliation.md), [verification](verification.md)
and [structure review](structure-review.md). No push or original Corenius
checkout retirement was performed.
