# Underlying session corpus audit correction

Research date: 2026-10-13 UTC. Repository read:
`e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`.

## Conclusion

The existing material supports ten selected behavioral episodes and a bounded
census of explicit native failed turns. It does **not** yet support describing
the work as a semantic audit of the saved Codex, TraeCLI and Claude corpus.
The 2026-10-13 parent audit thread and its four child threads are audit-process
provenance and candidate indexes, not underlying behavioral evidence. Every
promoted finding must instead be reconstructed from an older provider-native
transcript/thread record, or be explicitly labeled as cache/extract-only when
the original is unavailable.

The correction is therefore methodological, not a rejection of every existing
case. Representative Codex, TraeCLI and Claude checks below corroborate three
episodes within their stated limits. The report must narrow its corpus language,
record source fidelity per claim, replace dead citations, and avoid implying
that parsing 16.7 GB amounted to semantic review.

## Verified source map

### Prior report and audit package

- The prior report is an untracked file in the detached worktree
  `/data02/home/philip.yang/workspace/corenius-session-failure-audit-20261013`.
  Its executive summary claims ten behavioral episodes and 34 unexpected
  failed turns in 25 sessions, while already disclaiming a provider ranking
  or corpus-wide behavioral rate ([draft](/data02/home/philip.yang/workspace/corenius-session-failure-audit-20261013/docs/practice/session-failure-audit-2026-10-12.md:1)).
- Saved inventory and audit artifacts live under
  `/data02/home/philip.yang/.local/share/session-audit/2026-10-13/`. The
  inventory reports 5,888 parsed candidate files and 16,740,640,497 bytes,
  including fixtures and unlinked/synthetic traces
  ([summary](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/summary.json:1)).
- The inventory is a live-store index, not a frozen transcript archive. Its
  parser combines provider state databases, AgentsView cache, native history,
  prompt history and native files
  ([inventory source](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/inventory.py:34)).
  It gives raw files precedence in the final coverage label
  ([coverage selection](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/inventory.py:170)).
- A reconciliation defect matters to the phrase "parsed 16.7 GB": a prior
  parse is reused by pathname before a stat or open verifies current identity
  ([cache branch](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/inventory.py:70)).
  The later causal review reproduced stale reuse with the real parser and
  explicitly limits what that proves
  ([causal evidence](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/causal-review/evidence/result.md:28)).

### Coverage at the saved snapshot

The 5,367 inventory records are a thread-level sampling frame, not 5,367
complete or independent tasks. Snapshot coverage was:

| Provider | Inventory records | Native/raw transcript | Cache only | History item only | Prompt only | Metadata only |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Codex | 1,927 | 798 | 1,126 | 0 | 3 | 0 |
| TraeCLI | 2,953 | 2,007 | 0 | 8 | 94 | 844 |
| Claude | 482 | 447 | 23 | 0 | 12 | 0 |
| Unresolved CLI | 5 | 5 | 0 | 0 | 0 | 0 |

These figures come from the saved method record
([provider coverage](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/methods/scope-and-denominators.json:1)).
Thus 3,257 records were classified raw at snapshot time, 1,149 were cache-only,
and 961 were history/prompt/metadata-only. "Raw" says a path was readable at
the snapshot; it does not establish completeness, current survival, or semantic
review. The inventory itself warns about live stores, replayed parent history,
schema-dependent counters and unlinked traces
([limits](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/summary.json:398)).

### The October 13 deep-dive is not the corpus

The frozen inventory includes the parent audit thread
`01a13379-fba9-7b00-9f18-90cb1a2da55e` only through 04:26:23 UTC
([inventory record](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/sessions.jsonl:2414)).
Its four specialist children began later, at 04:54-05:07 UTC, and identify that
parent in their native session metadata: [runtime](/data02/home/philip.yang/.codex/sessions/2026/10/13/rollout-2026-10-13T04-54-37-01a1339c-cf7d-7192-b2cc-fda8faecb27f.jsonl:1),
[intent](/data02/home/philip.yang/.codex/sessions/2026/10/13/rollout-2026-10-13T04-54-52-01a1339d-0c1f-7233-b9fb-535bcaf1df69.jsonl:1),
[coordination](/data02/home/philip.yang/.codex/sessions/2026/10/13/rollout-2026-10-13T04-55-09-01a1339d-4c2e-7aa0-98c1-906af6622917.jsonl:1), and
[verification](/data02/home/philip.yang/.codex/sessions/2026/10/13/rollout-2026-10-13T05-07-05-01a133a8-390a-7b90-9939-6455d7b7d79b.jsonl:1).

Those traces explain how candidates were found and synthesized. They cannot be
counted as independent recurrence, validation of their own claims, or coverage
of the earlier corpus. The coordination method correctly describes FTS hits as
discovery counts and maps candidates back to native files
([method](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/method.json:11));
that boundary should govern the whole report.

## Representative primary-record checks

These checks deliberately avoid credential-shaped fields and unrelated process
list contents.

### Codex: preservation overclaim is supported, mechanism remains bounded

The surviving native Codex archive shows the lead say the initial failure log
was preserved
([original](/data02/home/philip.yang/.codex/archived_sessions/rollout-2026-10-01T01-39-09-01a0f51d-8b62-7930-862f-c4b44d7d0bb7.jsonl:7112)).
The later tool result reports `initial_raw_log_retained=false`
([diagnostic](/data02/home/philip.yang/.codex/archived_sessions/rollout-2026-10-01T01-39-09-01a0f51d-8b62-7930-862f-c4b44d7d0bb7.jsonl:7145)),
and the lead retracts the preservation claim
([retraction](/data02/home/philip.yang/.codex/archived_sessions/rollout-2026-10-01T01-39-09-01a0f51d-8b62-7930-862f-c4b44d7d0bb7.jsonl:7152)).
The archive still matches the saved source-manifest digest. This verifies the
overclaim and correction, but not recovery of the original failed bytes.

### TraeCLI: claim is supported by a saved extract, not a current original

The cited native TraeCLI file for `coord-01` is no longer present. The retained
extract records its original path, full-file digest, selected line numbers and
per-line digests
([extract header](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/browser-source-excerpts.json:1),
[manifest](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/source-manifest.json:100)).
Within that extract, the same full test is initially backgrounded, later
recorded completed with exit zero, then rerun unchanged and completed again
([first completion](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/browser-source-excerpts.json:476),
[rerun start](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/browser-source-excerpts.json:948),
[rerun completion](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/browser-source-excerpts.json:992)).
This supports result-retrieval/handoff failure and one redundant rerun. It does
not prove that compaction deleted the result or that a live exact-result lookup
was unavailable; the causal review preserves that limit
([lifecycle review](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/causal-review/lifecycle/result.md:40)).

### Claude: shared-checkout mutation is supported by the native original

The surviving Claude transcript shows an in-place mutation command against
`bridge.py`
([mutation](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-trondheim/2ca2cccc-4bca-46f8-9412-50c93b7c61fd.jsonl:4028)),
a warning sent after mutation began
([warning](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-trondheim/2ca2cccc-4bca-46f8-9412-50c93b7c61fd.jsonl:4041)),
and a successful restoration receipt
([restore](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-trondheim/2ca2cccc-4bca-46f8-9412-50c93b7c61fd.jsonl:4141)).
The current original matches its saved manifest digest. This verifies the unsafe
interval and recovery, not an actually corrupted snapshot or bad landing.

## Defensible corpus-level method

This is the minimum method needed before the report can make corpus-level
coverage claims.

1. **Freeze the source frame.** Create a manifest for every admissible input
   with provider, thread ID, parent/root identity, source kind, byte length,
   digest, parser/schema version and capture time. Retain allowed transcript
   bytes or a complete redacted representation before paths roll away. A live
   pathname or cache row is not a frozen source.
2. **Stratify by evidence fidelity.** Keep separate strata for native raw,
   native history item/turn, saved hash-bound extract, AgentsView cache,
   prompt-only and metadata-only. Metadata-only records contribute to missingness
   coverage, never behavioral findings. Cache/extract findings must say that the
   original was unavailable.
3. **Use the full inventory as the sampling frame.** Deduplicate by
   provider+thread and then by root task/parent-child ancestry so replayed
   parent context, reviewers and copied traces do not become recurrence. Record
   every inclusion and exclusion. Treat the October 13 audit threads as a
   discovery index outside the audited historical frame.
4. **Run two complementary reviews.** Census structured explicit turn/runtime
   failures where provider schemas permit it, with that native database as the
   denominator. Separately perform a provider/source-fidelity/role/time/project
   stratified semantic sample, including completed and accepted sessions with
   no visible correction. The old correction-seeded cases can establish
   existence only. A prevalence claim requires a probability sample and
   missingness adjustment.
5. **Promote candidates only from underlying records.** FTS, deep-dive prose,
   model admissions and heuristic outcomes generate hypotheses. Promotion
   requires the underlying expectation, action, tool/result, consequence and
   recovery window, with exact item/line IDs and source digest. When only a
   cache or saved extract survives, preserve that limitation in the finding.
6. **Track coverage per claim.** Publish counts for original-inspected,
   cache-inspected, extract-inspected and unreviewable records; identify malformed
   and multi-metadata files; distinguish file parsing from semantic review.
   Independent verification samples each provider and source-fidelity stratum.
7. **Redact before persistence.** Retain only allowlisted evidence fields and
   digests. Exclude credential-shaped values, encrypted/private reasoning and
   unrelated host process rows. The causal review already states this required
   boundary
   ([retention proposal](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/causal-review/RESULT.md:57)).

## Report claims that must change

1. **Executive scope.** Change "the audit documents 10 behavioral episodes" to
   "the audit documents 10 correction-seeded selected episodes." Keep the 34
   unexpected failed-turn count explicitly tied to the native structured-turn
   subset; it is not a corpus task-failure count.
2. **16.7 GB statement.** Replace "parsed 16.7 GB" with language that the
   inventory syntactically classified/accounted for 16.7 GB at a live snapshot,
   with some rows reused by pathname. State directly that this was not semantic
   review and did not freeze all source bytes.
3. **Investigator provenance.** State that the October 13 parent/child deep-dive
   generated candidate indexes and hypotheses. It is not itself evidence of
   older provider behavior; every finding must cite its underlying provider
   record and fidelity.
4. **Behavioral breadth and priorities.** The old sample selected episodes with
   visible correction/recovery. The later causal report acknowledges this bias
   and calls for no-correction controls and stratification
   ([breadth limit](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/causal-review/RESULT.md:69)).
   Until that review runs, priorities are design proposals derived from selected
   cases, not corpus recurrence or risk rankings.
5. **Dead TraeCLI citations.** Replace direct links to vanished TraeCLI JSONL
   paths. For `coord-01`, cite the manifest-bound extract and say the original
   is unavailable. The intent evidence for `I01`, `I02` and `I05` records native
   paths but not the coordination package's full-file/per-line hash chain
   ([intent evidence](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/intent/evidence.json:1)).
   Freeze and hash the surviving native-history items, or downgrade those cases
   to saved-excerpt evidence.
6. **Cache versus original.** `I03` and `I04` were promoted from AgentsView
   cached messages/tool results
   ([I03](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/intent/evidence.json:179),
   [I04](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/intent/evidence.json:251))
   even though the inventory lists raw native paths for those sessions. Recheck
   and cite the originals, or label the claims cache-derived and explain why the
   originals were not used.
7. **Independent verification wording.** Replace "checked ... against original
   sources" and unqualified "no blocking finding" with its actual bounded
   scope: two of five intent cases, all five coordination cases, and the 41
   native failed-turn IDs; `I04` used retained AgentsView data. The verification
   record itself says no exhaustive semantic audit occurred
   ([verification scope](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/verification.json:4),
   [limits](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/verification.json:229)).
8. **Snapshot language.** Date every coverage table and distinguish
   "readable at 2026-10-13 snapshot" from "currently readable." The causal
   package confirms missing primary TraeCLI transcripts and unresolved deletion
   cause
   ([unresolved sources](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/causal-review/RESULT.md:75)).

## Missing evidence and inference boundary

Verified here: the inventory/source strata and their snapshot counts; the
deep-dive parent/child chronology; current survival and digest match of the six
Codex/Claude files in the coordination manifest; absence of the selected native
TraeCLI files at their recorded paths; and the three representative incident
chains above.

Inferred/proposed: the stratified design is a defensible way to convert the
saved inventory into corpus-level evidence. It has not been executed by this
research note, so no new prevalence, recurrence or provider-comparison result
is claimed.

Missing: immutable originals for the selected TraeCLI cases; a hash-bound full
source frame for all 5,367 records; semantic review of a stratified
no-correction/control sample; a complete task-success denominator; outgoing
wire provenance for the `VALIDATED` failures; and writer/resource telemetry for
ENOSPC/EAGAIN. These gaps must remain explicit rather than being filled from the
October 13 deep-dive narrative.
