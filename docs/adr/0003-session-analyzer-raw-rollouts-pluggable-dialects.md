# Own a Session analyzer over raw rollouts with pluggable Trace dialects

Harp will analyze long-horizon agent execution from **raw Rollout artifacts**,
not from viewer databases, using a dedicated **Session analyzer** (Rust crate)
with **pluggable Trace dialects**. For TraeCLI dual-stream sessions,
**Model-visible history** (`history_mutation`) is the default story layer and
**Runtime telemetry** (`event_msg`) answers what ran; both are indexed and
**Stream divergence** is surfaced rather than silently merged.

## Status

Accepted.

## Context

Operator work such as the TraeCLI session
`01a011a1-5880-7251-b88a-e75c11512157` produces multi-day, multi-agent rollouts
where AgentsView’s archive can report an empty transcript while the on-disk
JSONL remains complete, and where publication-oriented postmortem capture
correctly strips private detail but cannot serve interactive reconstruction at
seed scale. A bounded investigation under
`knowledge/investigations/local/long-horizon-trace-analysis/20260904-traecli-long-horizon/`
documented mechanical hazards (dual streams, compaction replace, oversize
lines, externalized tool blobs). Grilling settled product shape before
implementation.

## Decision

1. **Primary job** is private **operator analysis** of an **Execution session**.
   Publishable postmortem projection and research Investigations are separate
   products.
2. **Transcript authority** is the **Rollout artifact** (raw TraeCLI JSONL and
   sibling artifacts). AgentsView may be audited later; it is not v1 authority.
3. **Session analyzer** is a **new Rust crate** (Session index + CLI), not a
   `harp research` subcommand and not an evolution of the postmortem tool.
   Shared libraries with postmortem may come later once both shapes stabilize.
4. **Trace dialects** are pluggable. Product direction is multi-agent; **v1 exit
   criterion** is TraeCLI-complete (all known TraeCLI eras meet the Index trust
   bar). Unsupported dialects are detected and refused with typed errors.
5. **Index trust bar**: reparse digest/invariant equality on the same artifact,
   plus CI synthetic fixtures. Live private rollouts are soak inputs only.
6. **Oversize JSONL lines** spill to content-addressed sidecars with typed
   stubs; builds stay complete.
7. **Session index store** lives under Harp XDG data home
   (`$XDG_DATA_HOME/harp/...`), not in git and not as the primary layout under
   `~/.trae`.
8. **Tool-result bodies** are **reference-only** in the v1 index.
9. **Dual-stream split** (TraeCLI): Model-visible history vs Runtime telemetry
   as above; no silent merge.
10. **v1 success** is a trustworthy mechanical index + CLI. Timeline browser and
    lesson extraction follow only after that bar.

## Considered options (rejected for v1)

- Treat AgentsView SQLite/MCP as transcript authority — false-empty on the
  seed class.
- Evolve postmortem private-capture into the operator analyzer — publication
  claim ceilings and hard event/line caps fight reconstruction.
- Require multi-agent format completeness before TraeCLI dual-stream is
  trustworthy — dilutes the trust bar.
- Prefer `event_msg` alone as the story, or silently coalesce streams — loses
  either model-visible compaction or runtime exec truth.

## Consequences

- Dialect deep-dive specs (starting with TraeCLI dual-stream) are prerequisites
  to codec implementation.
- Private indexes and spills must stay out of the repository vault.
- Future UI/lesson features must query the Session index layers explicitly
  rather than assuming a single chat transcript.
