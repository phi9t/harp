# Controlled language and source findings

Recorded: 2026-10-04 UTC. Research author: background `monitor_design_critique` agent.
Scope: primary-source language findings and the source bindings for the proposed operating packet.

## Official language findings

ASD identifies Issue 9 as the January 15, 2025 release. [Official home page](https://www.asd-ste100.org/)

Rule 3.6 requires active voice; descriptive text permits passive voice when the actor is unknown.
The reference is printed page 1-3-1, PDF page 67. [Issue 9](https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf#page=67)

Rule 5.1 limits procedural sentences to 20 words.
Rule 5.2 gives one instruction per sentence, except actions that occur together.
The reference is printed page 1-5-1, PDF page 87. [Issue 9](https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf#page=87)

Rule 6.3 limits descriptive sentences to 25 words.
The reference is printed page 1-6-1, PDF page 95. [Issue 9](https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf#page=95)

Rules 1.5–1.13 permit field-specific technical nouns and verbs within defined categories and usage constraints.
They retain stable names and parts of speech; these exceptions do not permit arbitrary jargon.
The summary appears on printed page 1-1-1, PDF page 45. [Issue 9](https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf#page=45)

The FAQ says writers can apply selected principles beyond maintenance documentation.
It describes approximately 900 general dictionary words and separate technical terminology; it does not impose a universal 900-word vocabulary.
Its tools guidance does not establish this packet's compliance or certification. [Official FAQ](https://www.asd-ste100.org/STE_faq.html)

Application: use practical STE-inspired prose, explicit actors, stable terms, short sentences, and one instruction per sentence.
Preserve exact commands, hashes, API names and domain identifiers.
Label source observations, coordinator reports, and inference separately.
This packet claims neither certified STE compliance nor a measured percentage improvement.

## Retrieval and attribution

Browser opens returned HTTP 403 for the PDF and FAQ; direct HTTPS retrieval succeeded.
The author read the official downloaded PDF with `pdftotext`; citations use its printed and PDF page identities.
Downloaded PDF SHA256: `d1f4ea9e7cd6e46b47aa9057209f99e78c0e9cfc4e27a5b07895b05c1a166431`.
Downloaded FAQ SHA256: `9c7f4507fbaa83fb777dad33e1547c28665ee2f712aac0860dcb7af38452b2f6`.
The repository retains these findings and digests, not a redistributed copy of the standard.

The supplied [Karpathy primary post](https://x.com/karpathy/status/2105819303471976479) returned HTTP 403.
Attribution and post content remain unverified; no quote or policy requirement depends on that post.

## Repository source bindings

Documentation branch base: `159dd8b186630211a26d8d44967291f5700350b8`.
Canonical execution checkout HEAD read: `291f04a5abd9274e4fd0034fd36927b0a4c4d236`.
The author read CONSTITUTION.md, agentic-engineering.md, skill-orchestration.md, domain.md, CONTEXT.md, and applicable AGENTS.md.
The author read the canonical specification, tickets 05 and 06, CONTROLLED-TRIAL.md, mutable STATE.md, and the exact ticket 05 brief.
All canonical reads were read-only; the author inspected no worker transcript.

| Read artifact | SHA256 observed on 2026-10-04 |
| --- | --- |
| `docs/efforts/sqlite-controller/STATE.md` (mutable) | `784ed83324d8391bce26fed4b73cff2cc6bf9d81030172e92bb15aeecc6b3ff7` |
| `docs/efforts/sqlite-controller/SPEC.md` | `959ca8540ffe9cd958c45a8a2f289c3d8808cc88996b482bb3db686a1e331479` |
| `tickets/05-lead-control-center.md` | `db3fd71a8b6f1e38a4f957096ba89f71f2c837a7385f7f5a78049c58a85b223f` |
| `tickets/06-retained-baseline-proof.md` | `b248dff20e8ded38bd09375b8a441a23f62a98253adcbf3b8b0e122b16c59d32` |
| `CONTROLLED-TRIAL.md` | `35742c7434829af4f8ef24f3a987877fcac248a8fac10bc6c7d4eb1a427b5e96` |
| `.scratch/sqlite-controller/briefs/05-lead-control-center.md` (mutable) | `38fe6bf7150b6724c02e66bdfaed2e860ff357ffb83e2956221dfaf188165ca7` |

STATE.md's newer top-level ticket 05 status conflicts with an older active ticket 04 ownership section.
Inference: a single current projection with separately preserved history will reduce conflicting current instructions.
The author read the procedural operator receipt `incident-worker-alignment-observer-20261004.json`, recorded at `2026-10-04T19:26:12.718759+00:00`.
It records a stopped observer, excluded items reaching observer context, no assessment, and zero ledger writes.
Receipt SHA256: `078a86b2ab3cd672c3e469ca48b14af79250971120deba771d7a4f567e8c33da`.
This verifies the receipt's contents; the author did not replay the incident or inspect its excluded bodies.
The coordinator retains the local-only operator receipt; another installation must obtain it from its operator.
The operating packet records no new cursor credit from that incident.
The parent task supplied the current ticket 05 actor/thread and context thresholds; these are operating inputs, not independently verified runtime telemetry.
