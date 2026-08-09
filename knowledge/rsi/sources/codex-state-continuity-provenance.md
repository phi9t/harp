# Codex state-continuity source provenance

## Scope

This receipt supports `../codex_state_continuity_and_compaction.md`. It is an
implementation overlay outside the Weng-rooted citation closure.

## Immutable code sources

| Source ID | Repository | Revision | Local checkout | Access and execution |
|---|---|---|---|---|
| `CODEX-REPO` | `https://github.com/openai/codex` | `1e85ca099e4265bf89f4016772d299816e231bb3` | `evidence/implementations/codex-rsi/snapshot/` | Commit object inspected locally; upstream code was not executed. |
| `ARC-AGI3-BENCH` | `https://github.com/arcprize/arc-agi-3-benchmarking` | `86d72170ce3155551712a9fafd290bab471d6eee` | `evidence/implementations/arc-agi-3/snapshot/` | Clean detached checkout inspected locally; upstream code was not executed. |

## Mutable OpenAI sources

### Compaction guide

- Requested locator:
  `https://platform.openai.com/docs/guides/compaction`
- Final canonical locator:
  `https://developers.openai.com/api/docs/guides/compaction`
- Retrieval date: 2026-08-02
- Content type: HTML
- Retrieved byte count: 353,044
- SHA-256:
  `272289a464456803422a33ff5091dc3a860196dc0de3a00d2550cb0dad3eb3ee`
- Classification: mutable official API documentation
- Redaction policy: the raw mutable page is not retained as canonical prose;
  claim-level extracts and boundaries are checked into
  `codex-state-continuity-claims.tsv`.
- Parser: direct UTF-8 HTML inspection with `rg`; no scripts from the page were
  executed.

The retrieved page supported server-side compaction, standalone compaction,
opaque encrypted compaction items, `previous_response_id` chaining, and the
instruction not to manually prune a managed chain.

### ARC result article

- Locator:
  `https://openai.com/index/how-two-settings-tripled-our-arc-agi-3-scores/`
- Retrieval attempt: 2026-08-02
- Access status: blocked by HTTP 403 in this environment
- Canonical input used: user-provided technical packet in the active thread
- Claim ceiling: OpenAI-reported aggregate scores, token reduction, and
  interpretation only; no independent score reproduction

### Responses feature article

- Locator:
  `https://openai.com/index/new-tools-and-features-in-the-responses-api/`
- Retrieval attempt: 2026-08-02
- Access status: blocked by HTTP 403 in this environment
- Canonical input used: user-provided technical packet in the active thread
- Claim ceiling: company claim about encrypted-reasoning reuse, token use, and
  continuity; local Codex source independently confirms only the request and
  round-trip shape

## Closure boundary

The source packet does not establish:

- the contribution split between reasoning retention and compaction;
- server-side KV-cache or prefix-cache implementation;
- confidence intervals or run-level ARC scorecards;
- semantic fidelity across repeated compactions;
- cross-model portability of opaque checkpoints; or
- durable semantics for external side effects.

Those gaps remain explicit in
`codex-state-continuity-claims.tsv` and the companion's experimental plan.
