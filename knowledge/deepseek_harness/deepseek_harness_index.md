---
id: deepseek-harness-index
title: DeepSeek Harness design-decision study
type: research-index
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, dsh, cordis, harness-engineering, agentic-runtime]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, CORDIS-PAPER, DSH-SMOKE]
---

# DeepSeek Harness design-decision study

Mode: `DESIGN DECISION STUDY`.

This packet studies DeepSeek Harness (`dsh`) as a public, pinned implementation
of a composition-first agent harness. It is separate from
[[knowledge/rsi/deepseek_harness_deep_dive|the RSI-facing overlay]], which keeps
the short recursive-self-improvement interpretation. This packet is the larger
design-decision study.

## Evidence boundary

**EVIDENCE - pinned implementation.** DeepSeek Harness is captured at
`47f943859bef60e4160492346772ded9b24f765a` under
[[evidence/implementations/deepseek_harness/snapshot/README.md|the local DSH snapshot]]
([README lines 5-11](../../evidence/implementations/deepseek_harness/snapshot/README.md#L5)).

**EVIDENCE - cited paper.** The Cordis paper cited by DSH is captured from
`cordiverse/paper` tag `v8` at commit
`948a07b369c62adb3b12e102458be5c18dfb69b9`. The local text extraction lives at
[[evidence/cordis_paper/text/cordis-paper-v8.txt|Cordis paper text]]
([abstract lines 13-27](../../evidence/cordis_paper/text/cordis-paper-v8.txt#L13)).

**EVIDENCE - local smoke.** Keyless local checks are recorded in
[[evidence/deepseek_harness_study/README.md|the DSH study receipt]]. They verify
source identity, package surfaces, documented examples, and keyless tool-flow
mechanics. They do not prove model-provider behavior, benchmark scores, or
security enforcement on every platform.

## Reader routes

- [[knowledge/deepseek_harness/01_cordis_spatiotemporal_composition|Cordis spatiotemporal composition]]
- [[knowledge/deepseek_harness/02_profiles_bundles_and_patch_layers|Profiles, bundles, and patch layers]]
- [[knowledge/deepseek_harness/03_events_effects_and_reversible_lifecycle|Events, effects, and reversible lifecycle]]
- [[knowledge/deepseek_harness/04_turn_session_and_model_visible_log|Turn, session, and model-visible log]]
- [[knowledge/deepseek_harness/05_capability_seams|Capability seams]]
- [[knowledge/deepseek_harness/06_tool_registry_and_policy_pipeline|Tool registry and policy pipeline]]
- [[knowledge/deepseek_harness/07_sandbox_permission_and_filesystem_boundaries|Sandbox, permission, and filesystem boundaries]]
- [[knowledge/deepseek_harness/08_subagents_and_continuable_children|Subagents and continuable children]]
- [[knowledge/deepseek_harness/09_persistence_sdk_and_runtime_packaging|Persistence, SDK, and runtime packaging]]
- [[knowledge/deepseek_harness/10_self_modification_and_dynamic_cordis|Self-modification and dynamic Cordis]]
- [[knowledge/deepseek_harness/source_registry|Source registry]]
- [[knowledge/deepseek_harness/claim_evidence_ledger|Claim evidence ledger]]

## Design-decision map

| Decision | One-line reading | Primary support | Local check |
|-|-|-|-|
| Cordis as substrate | DSH treats all product behavior as plugins over a shared context. | `DSH-C001`, `DSH-C002` | package and docs identity |
| Ordered profiles | Boot state is a patchable plugin tree, not a hard-coded product core. | `DSH-C003` | package scripts and config examples |
| Reversible lifecycle | Registrations are effects that unwind with their owning plugin. | `DSH-C004` | Cordis tutorial inspection |
| Event domains | Durable facts, live interception, and capability policies use different event domains. | `DSH-C005` | docs and generated catalog inspection |
| Event-sourced sessions | Model-visible context is reconstructed from the append-only session log. | `DSH-C006`, `DSH-C007` | source walk of `agent.ts` and session docs |
| Capability seams | Swappable providers are service definitions plus providers plus consumers. | `DSH-C008` | capability graph inspection |
| Tool policy pipeline | Tools separate model-visible schema from host execution and policy waterfalls. | `DSH-C009`, `DSH-C010` | keyless tool-flow tutorial check |
| Shared sandbox policy | File and shell capabilities resolve the same per-call policy. | `DSH-C011` | source walk of policy/fs/shell packages |
| Continuable subagents | Child agents are durable sessions with explicit activation and parent authority. | `DSH-C012` | subagent docs and type surface inspection |
| Runtime packaging | API, persistence, SDK, and generated contracts are explicit runtime faces. | `DSH-C013`, `DSH-C014` | package and script inspection |
| Dynamic Cordis | HMR and dynamic composition are treated as first-class runtime operations. | `DSH-C015` | tutorial and Cordis paper inspection |

## Claim ceiling

This packet can support:

- present-day architectural claims about the pinned DSH source tree;
- source-backed interpretation of DSH's Cordis-based design choices;
- keyless local verification of package surfaces and documented mechanics; and
- RSI-relevant analysis of what DSH would make easier for harness evolution.

This packet cannot support:

- DeepSWE leaderboard or model-performance claims;
- provider-authenticated runs, paid model executions, or UI behavior not locally
  exercised;
- platform-wide security guarantees for every sandbox backend; or
- evidence that DSH already implements autonomous recursive self-improvement.
