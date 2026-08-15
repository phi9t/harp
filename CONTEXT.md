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
