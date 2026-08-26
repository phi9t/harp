# LeanDojo organization refresh

Status: bounded research refresh, not an accepted architecture

Observation date: 2026-08-25 (America/Los_Angeles)

Prior brief: [`research.md`](research.md), snapshot 2026-08-23

## Scope and evidence boundary

This refresh reuses the exact 15-repository LeanDojo organization census in
the prior brief. It does not add repositories, adjacent systems, or sources.

For each repository, the default branch and current `HEAD` were resolved from
the official Git remote with:

```text
git ls-remote --symref https://github.com/lean-dojo/<repository>.git HEAD
```

The `ref:` record identified the default branch and the full 40-hex `HEAD`
record identified its observed commit. All 15 default branches resolved to
`main`. Each result was compared exactly with the immutable 2026-08-23 revision
recorded in the prior brief.

The inspection rule was: inspect implementation or author-paper artifacts only
for repositories whose default-branch SHA changed, with the existing cap of 12
artifacts per changed core repository and three per changed peripheral
repository. No SHA changed. The detailed content-inspection set is therefore
empty, and no source or paper link was followed beyond the census.

The commit links in the table are official immutable primary-source locations.
The default-branch mapping is a dated Git-remote observation; `main` may move
after the observation date.

## Exact repository census

| Repository | 2026-08-23 SHA | 2026-08-25 default-branch SHA | Changed? |
|---|---|---|---:|
| ReProver | [`fd6d99c01e8bd8fd8f3fd1de4cd0bc4a7f158eaa`](https://github.com/lean-dojo/ReProver/tree/fd6d99c01e8bd8fd8f3fd1de4cd0bc4a7f158eaa) | [`fd6d99c01e8bd8fd8f3fd1de4cd0bc4a7f158eaa`](https://github.com/lean-dojo/ReProver/tree/fd6d99c01e8bd8fd8f3fd1de4cd0bc4a7f158eaa) | No |
| LeanDojoWebsite | [`95aa617c239806ecee6b4da0c8c4c9ea76edf17a`](https://github.com/lean-dojo/LeanDojoWebsite/tree/95aa617c239806ecee6b4da0c8c4c9ea76edf17a) | [`95aa617c239806ecee6b4da0c8c4c9ea76edf17a`](https://github.com/lean-dojo/LeanDojoWebsite/tree/95aa617c239806ecee6b4da0c8c4c9ea76edf17a) | No |
| LeanDojo | [`7a9f600b250a89a62e991c05ffe81ba033018e6e`](https://github.com/lean-dojo/LeanDojo/tree/7a9f600b250a89a62e991c05ffe81ba033018e6e) | [`7a9f600b250a89a62e991c05ffe81ba033018e6e`](https://github.com/lean-dojo/LeanDojo/tree/7a9f600b250a89a62e991c05ffe81ba033018e6e) | No |
| LeanDojoChatGPT | [`1fbb7506dca60e0bdc031d423014718715f63ccd`](https://github.com/lean-dojo/LeanDojoChatGPT/tree/1fbb7506dca60e0bdc031d423014718715f63ccd) | [`1fbb7506dca60e0bdc031d423014718715f63ccd`](https://github.com/lean-dojo/LeanDojoChatGPT/tree/1fbb7506dca60e0bdc031d423014718715f63ccd) | No |
| LeanCopilot | [`60738032a1398a4e83524107424a11322a753609`](https://github.com/lean-dojo/LeanCopilot/tree/60738032a1398a4e83524107424a11322a753609) | [`60738032a1398a4e83524107424a11322a753609`](https://github.com/lean-dojo/LeanCopilot/tree/60738032a1398a4e83524107424a11322a753609) | No |
| LeanAgent | [`13f63bd8ed42057dbf909791ba39ea6f8759c690`](https://github.com/lean-dojo/LeanAgent/tree/13f63bd8ed42057dbf909791ba39ea6f8759c690) | [`13f63bd8ed42057dbf909791ba39ea6f8759c690`](https://github.com/lean-dojo/LeanAgent/tree/13f63bd8ed42057dbf909791ba39ea6f8759c690) | No |
| LeanMillenniumPrizeProblems | [`fd5207106c8c13c40cd4eeb0acb169c2c4e58aeb`](https://github.com/lean-dojo/LeanMillenniumPrizeProblems/tree/fd5207106c8c13c40cd4eeb0acb169c2c4e58aeb) | [`fd5207106c8c13c40cd4eeb0acb169c2c4e58aeb`](https://github.com/lean-dojo/LeanMillenniumPrizeProblems/tree/fd5207106c8c13c40cd4eeb0acb169c2c4e58aeb) | No |
| lean4code | [`3215b7e833e16b1be5a0a689277358208fe4f97b`](https://github.com/lean-dojo/lean4code/tree/3215b7e833e16b1be5a0a689277358208fe4f97b) | [`3215b7e833e16b1be5a0a689277358208fe4f97b`](https://github.com/lean-dojo/lean4code/tree/3215b7e833e16b1be5a0a689277358208fe4f97b) | No |
| LeanDojo-v2 | [`baed5eae6e87a65a446d9f54af07aab2154e7599`](https://github.com/lean-dojo/LeanDojo-v2/tree/baed5eae6e87a65a446d9f54af07aab2154e7599) | [`baed5eae6e87a65a446d9f54af07aab2154e7599`](https://github.com/lean-dojo/LeanDojo-v2/tree/baed5eae6e87a65a446d9f54af07aab2154e7599) | No |
| LeanProgress | [`328ca46ede8ab9e3506805ba7658859af74aa7be`](https://github.com/lean-dojo/LeanProgress/tree/328ca46ede8ab9e3506805ba7658859af74aa7be) | [`328ca46ede8ab9e3506805ba7658859af74aa7be`](https://github.com/lean-dojo/LeanProgress/tree/328ca46ede8ab9e3506805ba7658859af74aa7be) | No |
| TorchLean | [`4caafbb692139908da104a4cf1c3699a54acd4bc`](https://github.com/lean-dojo/TorchLean/tree/4caafbb692139908da104a4cf1c3699a54acd4bc) | [`4caafbb692139908da104a4cf1c3699a54acd4bc`](https://github.com/lean-dojo/TorchLean/tree/4caafbb692139908da104a4cf1c3699a54acd4bc) | No |
| BRIDGE | [`152ffbeaa1d9d320208b95b28fc39cf824b8780b`](https://github.com/lean-dojo/BRIDGE/tree/152ffbeaa1d9d320208b95b28fc39cf824b8780b) | [`152ffbeaa1d9d320208b95b28fc39cf824b8780b`](https://github.com/lean-dojo/BRIDGE/tree/152ffbeaa1d9d320208b95b28fc39cf824b8780b) | No |
| QuantumLean-Bench | [`dac8c9da3ccdbb0319b8e9dc76b94c11f7dcf39b`](https://github.com/lean-dojo/QuantumLean-Bench/tree/dac8c9da3ccdbb0319b8e9dc76b94c11f7dcf39b) | [`dac8c9da3ccdbb0319b8e9dc76b94c11f7dcf39b`](https://github.com/lean-dojo/QuantumLean-Bench/tree/dac8c9da3ccdbb0319b8e9dc76b94c11f7dcf39b) | No |
| ITPEval | [`30dc5fe5f0ec7d6e6fae4b2d7eb3d56c6da7fb2a`](https://github.com/lean-dojo/ITPEval/tree/30dc5fe5f0ec7d6e6fae4b2d7eb3d56c6da7fb2a) | [`30dc5fe5f0ec7d6e6fae4b2d7eb3d56c6da7fb2a`](https://github.com/lean-dojo/ITPEval/tree/30dc5fe5f0ec7d6e6fae4b2d7eb3d56c6da7fb2a) | No |
| LeanProfiler | [`ecbcb4bef3bbeb7ea8c2e6e48d196b49b75192f4`](https://github.com/lean-dojo/LeanProfiler/tree/ecbcb4bef3bbeb7ea8c2e6e48d196b49b75192f4) | [`ecbcb4bef3bbeb7ea8c2e6e48d196b49b75192f4`](https://github.com/lean-dojo/LeanProfiler/tree/ecbcb4bef3bbeb7ea8c2e6e48d196b49b75192f4) | No |

Accounting boundary: 15 census repositories classified, 15 unchanged, zero
changed, zero detailed implementation or paper artifacts inspected in this
refresh. This reconciles exactly to the fixed 15-repository corpus.

## Material deltas

There are no material source deltas within the allowed corpus. Every official
default-branch SHA still names the same immutable source tree inspected for the
2026-08-23 brief, as shown by the per-repository primary-source links above.

Consequently, this refresh supplies no new upstream evidence for changing the
brief's findings about interaction contracts, environment identity, proof
search, tracing, trust separation, evaluation, or observability. This is a
statement about unchanged repository bytes, not a claim that project activity,
hosted services, releases, papers, or external dependencies are unchanged.

## Corrections to the existing brief

No substantive correction to the LeanDojo project analysis is supported by
this refresh because the inspected source identities did not change.

Two date-boundary clarifications should accompany future use of the brief:

1. The statement that the organization API returned 15 public repositories is
   specifically a 2026-08-23 observation. This refresh intentionally reused
   that census and did not re-query organization membership.
2. Release counts, latest-release labels, repository settings, and paper
   availability in `research.md` remain observations from its stated snapshot.
   An unchanged source SHA does not independently refresh those metadata.

## Implications for Harp v1

The refresh does not alter the existing Harp v1 direction:

- Keep Harp as the sole durable executor and evidence/promotion authority.
  Nothing in the unchanged LeanDojo corpus warrants a second workflow engine
  or state store.
- Treat LeanDojo and LeanDojo-v2 as pinned component and protocol references,
  not as a wholesale infrastructure dependency. Any reused tracing,
  interaction, retrieval, or search mechanism still needs a Harp-owned
  environment and evidence boundary.
- Continue the Crouzeix golden trace before generalizing the runtime. The audit
  adds no upstream change that should reorder route certification, independent
  review, transactional publication, and offline receipt reproduction.
- Preserve the two-lane boundary: fast interactive observations may reuse
  workers, while certification consumes sealed inputs in a fresh environment
  and emits a distinct receipt.
- Make no source update, dependency migration, or adapter rewrite solely on the
  basis of this refresh.

These are bounded continuity implications. They do not promote the provisional
architecture in `research.md` to an accepted design.

## MISSING evidence and uncertainty

- Current LeanDojo organization membership was not refreshed; the exact prior
  census was a fixed input to this task.
- Tags, releases, issues, discussions, GitHub repository settings, hosted
  services, and non-source metadata were not inspected.
- Author papers were not revisited because no repository entered the changed
  inspection set. A paper revision can change without moving a repository's
  default branch.
- Dependency repositories, submodules, package registries, model artifacts,
  and remote datasets were outside the fixed census.
- The `git ls-remote` observations were not promoted into Harp's immutable
  `evidence/` plane. This document is a dated research record, not a certified
  upstream snapshot or availability receipt.
- Default branches are mutable. The table preserves the exact commits observed
  on 2026-08-25, but it cannot establish their state after that date.
- No local Lean build, Crouzeix route audit, Harp verifier run, or repository
  status refresh was performed. This file does not update any local proof or
  claim status.
