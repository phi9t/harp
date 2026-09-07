# Harp domain glossary

## Evaluation

**Access boundary** — The rules governing what an evaluated agent may inspect,
retrieve, execute, or transmit while attempting a task, including internet,
tool, repository, and evaluator access. It is distinct from contamination
history.

**Contamination history** — The relationship between benchmark task or solution
material and a model's training, tuning, or prior exposure. It is distinct from
the access available during an evaluation attempt.

**Historical-public-task control** — A benchmark with publicly available,
historically sourced tasks used to make explicit how newer evaluations differ
in provenance, access policy, evaluator construction, and rollout disclosure.
It is not presumed contamination-free merely because it has a pinned revision.

**Full benchmark row** — A comparison entry backed by both a dated primary
reader source and at least one pinned methodological artifact: task schema,
verifier, access-policy document, run-data artifact, or dataset revision. A
benchmark that lacks this pair is a first-party reader route with a narrower
claim ceiling, not a complete comparison row.

**Evaluator disclosure** — The independently inspectable parts of an
evaluation: task specification and repository state; verifier/test logic;
allowed tool and internet policy; rollout trace, cost, and step accounting;
and model/agent configuration. Disclosure is assessed per component, not as a
single “open” or “closed” label.

**Pre-exposure resistance** — Resistance to task or solution material having
entered a model's training, tuning, or prior context before an evaluation.

**Live-evaluation resistance** — Resistance to an agent obtaining
task-relevant help during a rollout through internet, search, repository,
evaluator, or tool access.

**Access-policy enforcement evidence** — A documented mechanism or audit trail
showing that an access rule constrained an evaluation attempt. A stated
internet or tool policy alone is not enforcement evidence.

**Agent visibility** — The task, repository, evaluator, tool, and policy
material the evaluated agent can inspect during an attempt.

**Auditor visibility** — The task, repository, evaluator, policy, and rollout
material an independent reviewer can inspect after an attempt, including
through a controlled disclosure route. It is distinct from agent visibility.

## Proof Search

**Expert attempt** — One ticketed execution of a mathematical expert task. An
attempt may fail or produce unusable output without creating a mathematical
node.
_Avoid_: Node, expert node

**Expert result** — A schema-valid expert response that has not yet passed the
mathematical-node admission predicate. Raw malformed provider output is attempt
evidence, not an expert result.
_Avoid_: Node, archive node

**Mathematical node** — An immutable admitted mathematical artifact with
lineage, provenance, and a mathematical payload. Evaluation scores, child
counts, selection history, and review outcomes are not part of the node.
_Avoid_: Expert call, attempt, archive entry, candidate node

**Admission decision** — The immutable determination that an expert result
either creates one mathematical node or is rejected with a typed reason.
_Avoid_: Evaluation, scoring

**Node evaluation** — An independent probe-based assessment that references an
immutable mathematical node. It does not modify the node.
_Avoid_: Node state

**Archive entry** — The immutable pairing of a mathematical node with its
reconciled proof-progress evaluation. DGM selection operates on archive
entries, while child counts and selection eligibility are derived projections.
_Avoid_: Mathematical node, attempt

**Candidate projection** — A content-addressed review input derived from an
archive entry whose node contains a candidate proof and whose probes all pass.
It is a qualification for independent review, not proof certification.
_Avoid_: Complete proof, completed node

## Agentic execution analysis

**Operator analysis** — Private reconstruction and inspection of recorded agent
execution so a human operator can understand, debug, and extract lessons. It is
not a publishable postmortem and not a research Investigation.
_Avoid_: Postmortem, Investigation, chat replay

**Execution session** — The v1 unit of operator analysis: one recorded agent
runtime identified by a session id and backed by a primary rollout artifact.
Multi-day span is allowed; cross-session stitching is out of scope for v1.
_Avoid_: Episode, Investigation, thread (as the analysis unit)
_Related later_: Episode (optional multi-session span) may be introduced only
after Execution session ingest is reliable.

**Rollout artifact** — The primary on-disk recording of an Execution session.
It is the transcript authority for operator analysis. Normalized viewer
databases are not substitutes for it in v1.
_Avoid_: AgentsView archive (as v1 authority), chat export

**Trace dialect** — A named family of Rollout artifact shapes (for example a
TraeCLI dual-stream rollout versus a TraeCLI response-item rollout, or a
non-TraeCLI agent format). The Session analyzer is dialect-pluggable; a dialect
is *supported* only when it meets the Index trust bar with its own fixtures.
_Avoid_: Agent brand name alone, “TraeCLI” as the only forever format

**Session analyzer** — Harp-owned tooling whose job is operator analysis of an
Execution session from its Rollout artifact via Trace dialects. It is separate
from postmortem publication capture and projection. v1 is a dedicated Rust
crate (Session index + CLI), not a `harp research` subcommand. Product
direction is multi-dialect; v1 exit criterion is TraeCLI-complete (all known
TraeCLI eras trustworthy), with other dialects detected/refused until ready.
_Avoid_: Postmortem tool (as the v1 product), AgentsView, `harp` main CLI
(as the v1 home)

**Session index** — A derived, re-computable local projection of an Execution
session built from its Rollout artifact for operator queries. It is not the
system of record; the Rollout artifact remains authoritative.
_Avoid_: Archive (as authority), transcript database (as authority)

**Index trust bar** — An Execution session index is trusted for operator use
only when (a) re-indexing the same Rollout artifact reproduces the same index
digests and agreed recount invariants, and (b) CI passes on synthetic fixtures
that cover known mechanical hazards for that Trace dialect. Live private
rollouts are soak inputs, not repository fixtures.
_Avoid_: “Looks right in the UI”, AgentsView health row parity alone

**Oversize line spill** — When a JSONL line exceeds the analyzer threshold, its
bytes are written to a content-addressed sidecar and the Session index stores
only a typed stub (identity, offsets, digest). The build remains complete;
full materialization is explicit and opt-in.
_Avoid_: Silent skip, hard-fail-as-the-only-policy

**Session index store** — The machine-local directory tree that holds Session
indexes and oversize spills for operator analysis, under Harp’s XDG data home
(`$XDG_DATA_HOME/harp/...`, defaulting to `~/.local/share/harp/...`). It is
private machine state, not part of the repository vault.
_Avoid_: In-repo `.build` as the primary store, TraeCLI sessions directory as
the Harp index home

**Tool-result reference** — A Session index entry that records a structured
pointer to an externalized tool-result artifact (path, presence, size, digest
when available) without storing the blob body. Body materialization is
out-of-band and opt-in.
_Avoid_: Full blob ingest (v1), ignoring externalization

**Model-visible history** — The conversation and tool I/O the model context
accumulated, recorded as `history_mutation` items (`append` grows it; `replace`
rewrites it on compaction). Default operator “story” views use this layer.
_Avoid_: event_msg alone as the story, merged fake chat log

**Runtime telemetry** — Harness-observed execution facts recorded primarily as
`event_msg` payloads (exec status, token samples, collab wait/spawn, task
lifecycle, UI `user_message`/`agent_message`). Used for “what ran?” queries.
_Avoid_: history_mutation alone as proof a process exited

**Stream divergence** — A counted disagreement or one-sided silence between
Model-visible history and Runtime telemetry for the same Execution session
(missing joins, empty outputs, compaction rewrites, absent turn_ids). The
Session index must surface divergence metrics rather than hide them in a merge.
_Avoid_: Silent coalesce of the two streams
