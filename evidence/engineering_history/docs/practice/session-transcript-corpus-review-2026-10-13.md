# Underlying session transcript corpus review — October 13, 2026

Source repository read:
`e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`. Review dates and source
timestamps are UTC.

This review corrects the scope of the earlier session failure-mode audit. It
uses the October 13 audit session only as a candidate index and provenance
record. Behavioral claims below come from earlier Codex or Claude
provider-native transcripts, or are explicitly marked as cache- or
extract-only when a native original is unavailable. No behavioral claim uses
the October 13 parent thread or its four investigator threads as proof.

## Conclusion

The retained material supports selected, source-grounded failure mechanisms;
it does not support a failure rate for the corpus. The strongest recurring
theme is still a mismatch between what an action or check establishes and what
the agent later claims: a failed gate did not stop publication, a passing test
could not detect the relevant wrong result, an amended request did not update
the acceptance list, and filtered output hid a verifier failure.

The 5,367 inventory records are a sampling frame, not 5,367 reviewed
conversations or independent user tasks. They include 3,435 identified
subagents, probes, and automated workers. Current source loss also prevents a
provider-native TraeCLI conversation review: at the review scan, all 330
surviving TraeCLI paths selected from the frozen inventory were external
execution streams, while 1,677 rows whose native/raw paths were readable at
the earlier snapshot had no surviving recorded path. A private companion
capture made on October 14 preserved 297 of those TraeCLI streams; source loss
continued between scan and capture.

A deterministic completion-control sample did not establish a silently wrong
success in the surviving native originals. That negative result is useful but
small: it covers 11 native-original apparent completions and cannot validate
the external artifacts those sessions discussed.

## What was actually reviewed

### Frozen sampling frame and current source availability

The frozen inventory is
[sessions.jsonl](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/sessions.jsonl:1).
Its snapshot coverage was:

| Provider | Records | Raw at snapshot | Cache only | History only | Prompt only | Metadata only |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| TraeCLI | 2,953 | 2,007 | 0 | 8 | 94 | 844 |
| Codex | 1,927 | 798 | 1,126 | 0 | 3 | 0 |
| Claude | 482 | 447 | 23 | 0 | 12 | 0 |
| Unresolved CLI | 5 | 5 | 0 | 0 | 0 | 0 |

“Raw at snapshot” is a historical availability label, not a statement that the
source still exists or was semantically reviewed. A fresh path check and
schema-aware scan produced this current source map:

| Provider | Current provider-native | Current external stream | Missing former raw |
| --- | ---: | ---: | ---: |
| TraeCLI | 0 | 330 | 1,677 |
| Codex | 740 | 20 | 38 |
| Claude | 447 | 0 | 0 |
| Unresolved CLI | 0 | 5 | 0 |

In total, 1,542 selected paths remain readable and 1,715 formerly raw rows have
no surviving recorded path. The readable paths contain 3.70 GB and 809,341
parsed events, including 8,418 user messages, 54,659 assistant messages,
121,981 tool calls, 87,855 tool results, and 84,215 command results. Those are
syntactic scan counts, not semantic-review counts. Two selected paths produced
parse errors.

The AgentsView cache supplied 174,351 Codex and Claude messages for 2,344
inventory sessions. After conservatively excluding parented sessions, known
permission guards, probes/fixtures, explicit workers, and rows marked
automated, 145 cached root sessions remained eligible for semantic candidate
generation: 100 Codex and 45 Claude. This is a conservative operational
classification, not proof that every retained root was human-led. TraeCLI has
no corresponding current message cache.

### Frozen originals and normalized companion corpus

The private companion corpus is
[transcript-corpus-v1](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/README.md).
It separates exact evidence from privacy-reduced analytics:

- `originals/sha256/` holds content-addressed, mode-`0600` copies of 1,504
  source files totaling 3,723,793,041 bytes;
- `originals/cache/agentsview-frozen-subset.sqlite3` is a transactionally
  consistent exact subset containing 2,344 session rows, 174,351 messages,
  138,742 tool-call rows, and 90,338 tool-result-event rows;
- [source-manifest.jsonl](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:1)
  binds original paths, hashes, byte lengths, wire formats, capture identity,
  and session references to each retained object; and
- `normalized/` provides compressed JSONL plus a queryable
  [normalized.sqlite3](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/normalized/normalized.sqlite3)
  for sessions, ordered events, turns, and linked tool exchanges.

Capture-time source fidelity was:

| Provider | Native original | External stream | AgentsView cache | Metadata only |
| --- | ---: | ---: | ---: | ---: |
| Claude | 446 | 1 | 23 | 12 |
| Codex | 760 | 0 | 1,158 | 9 |
| TraeCLI | 0 | 297 | 0 | 2,656 |
| Unresolved CLI | 0 | 0 | 0 | 5 |

The one Claude external row is an attachment-only journal whose wire format
was not recognized as a conversation. Thirty-nine previously selected paths
were missing at capture time, and three retained path contents no longer
matched their scan-time hashes. Those facts are recorded as drift rather than
silently repaired.

The normalized layer contains 5,367 session rows, 584,925 event rows, 12,624
turn rows, and 175,081 tool exchanges. It retains native Codex turn IDs;
infers Claude, external-stream, and cache turns at documented boundaries; and
links 164,412 retained results to 175,081 calls. Three malformed source lines
remain explicit `parse_error` events. Provider reasoning is represented only
by 85,917 omission markers. Normalization also omitted 2,424 process-list
outputs, performed 16,182 credential-redaction transformations, and bounded
4,228 large or deeply nested values. Those are transformation counts, not
counts of distinct secrets or incidents.

The complete private corpus occupies approximately 13 GB. Its
[validation.json](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/validation.json:1)
records matching JSONL/SQLite row counts, SQLite integrity `ok`, zero duplicate
sessions, zero orphan events, zero events without turns, contiguous per-session
event sequences, source-object hash checks, recursive private permissions, and
zero remaining strict credential-shape matches in normalized string/key
leaves. Exact originals are intentionally unredacted and must remain private;
the normalized checks do not prove the absence of every kind of sensitive
non-credential content.

### Review procedure

1. Reconcile every frozen inventory row to a currently readable path and the
   Codex/Claude message cache. Classify each selected raw path as
   provider-native or external.
2. Parse the actual provider schemas and retain only redacted message,
   command, and result tails in the working index. Credential-shaped values,
   private reasoning, and unrelated process rows are not copied into this
   report.
3. Separate root candidates from parented sessions, permission guards,
   probes, fixtures, and automated workers before searching for assistant
   corrections, user challenges, pending work, and apparent success.
4. Treat phrase matches only as leads. Promote a case only after reading the
   underlying request, action, tool result, consequence, and recovery window
   in the provider-native transcript. When the original is gone, label the
   case cache- or hash-bound-extract-only.
5. Add a completion control: for each of Codex and Claude, sort eligible
   apparent-success sessions by a stable hash of session ID and inspect the
   first 12 not already in the correction queue. This is reproducible but not
   a probability sample.

The resulting 44-session manual queue comprised 12 correction candidates,
seven challenge candidates, two apparent pending-work endings, one success
after a last nonzero command, and 24 completion controls. Some candidates were
false positives caused by task notifications, compaction summaries, review
reports, or interrupted work; they were not promoted.

## Native-transcript findings beyond the earlier selected cases

These five incident chains were read in their surviving provider-native Claude
transcripts. Each shows the surrounding action and recovery, not merely an
assistant admission.

### 1. A failing gate did not stop publication

In Torchetta, a compound shell command used `;`, so later publication steps
were not conditional on the gate. The gate returned `rc=101`, but the branch
was pushed and merge request `!16` was opened
([failed gate and push](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torchetta/8fe397a4-2b06-4b89-a7b8-ac59af700e49.jsonl:7770)).
The agent then corrected the merge-request description and recorded follow-up
work
([recovery decision](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torchetta/8fe397a4-2b06-4b89-a7b8-ac59af700e49.jsonl:7797),
[corrected description](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torchetta/8fe397a4-2b06-4b89-a7b8-ac59af700e49.jsonl:7798)).

This establishes a publication-control failure, not that the failed tests were
caused by the published change or that the merge request was merged.

### 2. Shell expansion admitted a generated output tree

In Vaso, an unquoted heredoc expanded `$VASO_BAZEL_OB` before the intended
script consumed it. The empty value defeated the exclusion and `git add -A`
committed a Bazel output base
([cause](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-vaso/556fd545-7d69-4ada-8f0d-453394340fe0.jsonl:2900)).
The tool evidence reports 109,075 files changed and 569,011 insertions
([commit inspection](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-vaso/556fd545-7d69-4ada-8f0d-453394340fe0.jsonl:2877)).
The bad tip was `2b81367`; the landing ref was restored to `a61d5f3`, and the
ancestry check showed that the target branch never contained the bad tip
([recovery and ancestry](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-vaso/556fd545-7d69-4ada-8f0d-453394340fe0.jsonl:2890)).

This is a high-consequence near miss with demonstrated containment, not a bad
landing.

### 3. A green scenario could not detect the wrong toolbox verdict

In Corenius, 27 worker tests, five lead-driven tests, and nine fleet scenarios
passed. Review then found that one scenario compared its own observation to
its own expectation but never asserted the toolbox verdict
([passing evidence](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-corenius/b8dd1a72-5c0d-4e13-99aa-beecaa39b047.jsonl:570),
[review finding](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-corenius/b8dd1a72-5c0d-4e13-99aa-beecaa39b047.jsonl:595)).
An incorrect toolbox result could therefore leave the scenario green.

This is direct evidence that test volume and pass counts do not compensate for
an omitted behavioral assertion.

### 4. An amended request did not amend acceptance

Later in the same Corenius thread, review found that a worker had implemented
the original acceptance list while missing requirements added in amended
prose. The lead summarized the defect as: “an amendment changes the acceptance
list, or it is not an amendment”
([acceptance drift](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-corenius/b8dd1a72-5c0d-4e13-99aa-beecaa39b047.jsonl:1421)).

The record establishes requirements drift inside one execution chain. It does
not establish how often amended requests are missed across the corpus.

### 5. Output filtering hid a verifier failure

In TorchTitan, filtered preflight output made a verifier step appear clean even
though pyrefly had failed. A later nested gate surfaced the failure
([gate failure](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torchtitan/cb2d7426-0526-4c44-8c97-30a943533be7.jsonl:2697),
[filtering admission](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torchtitan/cb2d7426-0526-4c44-8c97-30a943533be7.jsonl:2701)).
After repair, the unfiltered rerun passed
([recovery](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torchtitan/cb2d7426-0526-4c44-8c97-30a943533be7.jsonl:2709)).

The problem was observability of the earlier command result; the later gate did
catch it before the inspected workflow completed.

## Status of the earlier ten behavioral episodes

The earlier report's ten episodes remain useful as correction-seeded existence
proofs, not as a semantic audit of 16.7 GB. Representative source checks show
why fidelity must remain attached to each case:

| Episode | Current evidence | Defensible claim |
| --- | --- | --- |
| Failure-log preservation | Native Codex original | The lead claimed both logs were preserved, a tool result later reported `initial_raw_log_retained=false`, and the lead retracted the claim. The original failed bytes were not recovered. [claim](/data02/home/philip.yang/.codex/archived_sessions/rollout-2026-10-01T01-39-09-01a0f51d-8b62-7930-862f-c4b44d7d0bb7.jsonl:7112), [diagnostic](/data02/home/philip.yang/.codex/archived_sessions/rollout-2026-10-01T01-39-09-01a0f51d-8b62-7930-862f-c4b44d7d0bb7.jsonl:7145), [retraction](/data02/home/philip.yang/.codex/archived_sessions/rollout-2026-10-01T01-39-09-01a0f51d-8b62-7930-862f-c4b44d7d0bb7.jsonl:7152) |
| Shared-tree mutation | Native Claude original | Mutation testing edited `bridge.py` in a shared checkout; the warning arrived after mutation began, and restoration later succeeded. This establishes an unsafe interval, not a corrupted landing. [mutation](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-trondheim/2ca2cccc-4bca-46f8-9412-50c93b7c61fd.jsonl:4028), [warning](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-trondheim/2ca2cccc-4bca-46f8-9412-50c93b7c61fd.jsonl:4041), [restore](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-trondheim/2ca2cccc-4bca-46f8-9412-50c93b7c61fd.jsonl:4141) |
| Redundant full test after resume | Hash-bound saved extract; native TraeCLI original missing | The extract records a successful full test, an unchanged rerun, and a second completion. It supports one redundant rerun and a result-retrieval/handoff failure, but not that compaction deleted the result. [manifest](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/source-manifest.json:100), [first completion](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/browser-source-excerpts.json:476), [rerun](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/browser-source-excerpts.json:948), [second completion](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/failure-modes/coordination/browser-source-excerpts.json:992) |

Other direct citations in the earlier report must not be treated uniformly.
Several selected TraeCLI originals have disappeared, and some intent cases
were promoted from AgentsView cache even when the inventory once listed a raw
path. Dead native links should be replaced by hash-bound extracts where those
exist; otherwise the claims should be downgraded to cache/extract evidence.

The earlier structured runtime census is a separate result: 41 explicitly
failed native turns, of which six were policy refusals, one was an unsupported
setting probe, and 34 were unexpected failures across 25 sessions. These are
turn outcomes within the retained native history snapshot, not behavioral
findings, failed user tasks, or a denominator for the 5,367-record inventory.

## Completion controls and uncorrected outcomes

The 24 stable-hash controls were selected specifically to counter the earlier
sample's bias toward visible admissions and recoveries. Manual triage found:

- 14 actual completion claims: 11 with surviving native originals and three
  cache-only;
- four reviewer failure reports rather than completion claims; and
- six partial, compaction, or interrupted records.

No directly contradicted silent-success claim was established among the 11
native-original completions. One useful control disclosed rather than hid a
non-green state: a Torch Dojo session reported four end-to-end failures
([full-run disclosure](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torch-dojo/7f914975-8e0f-41b3-99c8-148cb86c93a5.jsonl:866)),
reproduced two on the previous commit and observed another as a flake
([HEAD comparison](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torch-dojo/7f914975-8e0f-41b3-99c8-148cb86c93a5.jsonl:924),
[flaky rerun](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torch-dojo/7f914975-8e0f-41b3-99c8-148cb86c93a5.jsonl:960)),
and kept the non-green suite in the final completion report
([final disclosure](/data02/home/philip.yang/.claude/projects/-data02-home-philip-yang-workspace-torch-dojo/7f914975-8e0f-41b3-99c8-148cb86c93a5.jsonl:972)).

This control does not prove the completion claims correct. Transcript review
can test whether failures and uncertainty were represented faithfully; it
usually cannot independently establish the truth of an external build,
deployment, or artifact after the fact.

## Controls suggested by the underlying records

These are design proposals derived from observed mechanisms, ordered by
consequence and evidence strength rather than estimated frequency.

| Control | Required demonstration |
| --- | --- |
| Bind every acceptance claim to an exact candidate and a behavior capable of falsifying it | A deliberately wrong toolbox result fails the scenario; changing a source anchor or expected text cannot make unchanged wrong behavior pass |
| Make publication conditional on the complete required gate | A nonzero gate result prevents push or merge-request creation even when later shell commands are present |
| Preserve raw status before summarizing or filtering output | A hidden pyrefly or equivalent verifier failure remains visible in the terminal result and final report |
| Version acceptance criteria when the request is amended | A worker cannot start or finish against the superseded list without an explicit reconciliation receipt |
| Seal attempt identity and evidence separately | A retry cannot overwrite the first failure log, and a resumed lead can retrieve the original execution's terminal result without rerunning |
| Apply one-writer isolation to verification as well as implementation | Mutation tests cannot alter a checkout another agent may inspect or snapshot |
| Tie monitors to an owned producer and terminal state | A deleted artifact directory or completed producer terminates its watchers within a bounded interval |

## Limits and required next steps

1. **No prevalence estimate.** The correction queue is phrase-seeded and the
   completion control is a small deterministic sample. It establishes
   mechanisms, not frequency or provider ranking.
2. **TraeCLI is currently unreviewable at native-conversation fidelity.** The
   297 TraeCLI paths frozen in the companion corpus are external streams,
   there is no message cache, and many formerly readable originals are gone.
   Saved extracts can preserve individual cases but cannot restore the missing
   corpus.
3. **The frozen inventory is not a frozen archive.** The collector reuses a
   prior parse by pathname before validating current content identity
   ([cache branch](/data02/home/philip.yang/.local/share/session-audit/2026-10-13/inventory.py:74))
   and compares some timestamps as strings. Its saved totals reconcile, but a
   rerun is not automatically a fresh scan.
4. **Cache fidelity is lower than native fidelity.** AgentsView is useful for
   candidate recovery, but it may omit ordering, tool payloads, provider
   metadata, or messages. Cache-only conclusions must remain labeled.
5. **Transcript truth is bounded.** A transcript can establish what was asked,
   executed, observed, claimed, and corrected. It cannot by itself validate
   every referenced external artifact or prove that an absent later event never
   occurred.
6. **Sensitive evidence should be minimized.** Only allowlisted fields,
   redacted excerpts, source identities, and hashes should be retained. Raw
   process listings and credential-shaped arguments are not needed for these
   behavioral findings.

The companion corpus now supplies the source freezing, content identity,
turn/tool normalization, and explicit fidelity strata that the next audit
needs. The remaining step is a provider, role, project, time, and
fidelity-stratified semantic sample that deliberately includes accepted
sessions with no visible correction. Only that sampling design could support
a defensible recurrence estimate.

## Working artifacts

The schema-correct scanner and its redacted outputs are under
`.scratch/underlying-transcript-audit/` in the audit worktree:

- `scan.py`: frozen-inventory reconciliation, provider parsing, role
  exclusions, candidate generation, and stable-hash control selection;
- `summary.json`: aggregate counts used above;
- `session-index.jsonl`: one source-fidelity row per inventory record; and
- `review-queue.jsonl`: the 44-session manual queue.

The generated private corpus is outside Git at
`/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/`.
Its `README.md`, `build-summary.json`, `source-manifest.jsonl`, and
`validation.json` describe use and limits. The exact build-time normalizer and
the final builder/validator are preserved separately under
`originals/control/`.

These are reproducible working artifacts, not substitutes for the cited
provider-native records. The durable claims and their limitations are recorded
in this document.
