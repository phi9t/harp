# Crouzeix Proof Reproduction Issues

> Each issue is independently reviewable. Implementation may share the current
> isolated feature worktree because the user authorized continuous execution,
> but commits must remain focused by concern.

## Shared Contract

All issues implement
[`2026-08-14-crouzeix-proof-reproduction-design.md`](../specs/2026-08-14-crouzeix-proof-reproduction-design.md)
and
[`2026-08-14-crouzeix-proof-reproduction-prd.md`](../specs/2026-08-14-crouzeix-proof-reproduction-prd.md).

Every production behavior change follows red-green-refactor. Live provider
runs are operator actions after fake-provider tests pass; they are not test
fixtures and must never run from `mise run verify`.

## Issue 1: Research And Experiment Contract

**Goal:** register the bounded source reconstruction and the approved
prospective experiment before implementing the runner.

**Owned files:**

- `knowledge/crouzeix_conjecture/reproduction_research.md`
- `docs/superpowers/specs/2026-08-14-crouzeix-proof-reproduction-design.md`
- `docs/superpowers/specs/2026-08-14-crouzeix-proof-reproduction-prd.md`
- `docs/superpowers/plans/2026-08-14-crouzeix-proof-reproduction-issues.md`

**Acceptance:**

- Prompt byte count, SHA-256, blob identity, and Git introduction point match
  the public repository and existing Harp receipt.
- Root instructions are separated from unobserved worker execution.
- Codex/ChatGPT attribution drift is explicit.
- Historical replay is rejected as an unsupported claim.
- Leakage tiers, generation/verification separation, arms, metrics, and threats
  are fixed.
- No upstream bytes are added.

**Verification:**

```sh
git diff --check
rg -n "4106|0a0c3000|MISSING|L0|L1|L2|blocked" \
  knowledge/crouzeix_conjecture/reproduction_research.md \
  docs/superpowers/specs/2026-08-14-crouzeix-proof-reproduction-*.md
```

**Commit group:** `docs: design Crouzeix proof reproduction`

## Issue 2: Strict Protocol And Run Preparation

**Goal:** implement strict experiment types, historical prompt verification,
path normalization, run preparation, and route-state enforcement.

**Owned files:**

- `labs/crouzeix_proof_reproduction/protocol.py`
- `labs/crouzeix_proof_reproduction/prepare_run.py`
- `labs/crouzeix_proof_reproduction/schemas/*.json`
- `labs/crouzeix_proof_reproduction/prompts/*.md`
- `labs/crouzeix_proof_reproduction/tests/test_protocol.py`
- `labs/crouzeix_proof_reproduction/README.md`
- `labs/crouzeix_proof_reproduction/.gitignore`

**Test-first slices:**

1. Historical prompt verifier accepts only 4,106 bytes and the pinned SHA-256.
2. Normalization replaces only the absolute output path and records both
   digests.
3. Run specifications reject unknown fields, invalid enums, unsafe paths,
   excessive budgets, and symlinks.
4. Route events reject duplicate IDs and invalid transitions.
5. Blocked routes reopen only with a byte-distinct, nonempty mechanism.
6. Promotion rejects unresolved critical findings and theorem-strength
   obligations.
7. Preparation creates a complete run tree and refuses replacement.

**Exact verification:**

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_protocol -v
```

**Commit group:** `feat: add Crouzeix reproduction protocol`

## Issue 3: Provider Calls And Observable Orchestration

**Goal:** implement create-only TRAE CLI calls, historical and orchestrated
arms, usage accounting, and deterministic run receipts.

**Owned files:**

- `labs/crouzeix_proof_reproduction/runner.py`
- `labs/crouzeix_proof_reproduction/run_experiment.py`
- `labs/crouzeix_proof_reproduction/tests/test_runner.py`
- additions to phase prompts and strict schemas
- updates to `labs/crouzeix_proof_reproduction/README.md`

**Test-first slices:**

1. Command construction pins `--ignore-user-config`, `--ignore-rules`,
   `--ephemeral`, model, sandbox, approval, allowed tools, schema, and output
   paths.
2. Call workspaces contain no reference proof or Harp packet.
3. Existing call directories are rejected.
4. Completed, failed, timed-out, and malformed calls produce typed receipts.
5. Event parsing captures session ID and usage without treating missing fields
   as zero.
6. Historical arm makes exactly one call and validates `candidate.tex`.
7. Initial orchestrated workers run without peer outputs.
8. Controller, redirect, synthesis, critics, and repair consume only declared
   parent artifacts.
9. Promotion is deterministic and model-independent.
10. Aggregate accounting reconciles against per-call receipts.

**Fake provider:**

The test fixture is a temporary executable that:

- validates argv;
- emits the four expected JSONL event types;
- can emit malformed JSON, nonzero exit, timeout, or missing usage;
- writes schema-valid role-specific outputs; and
- records the prompt it received.

**Exact verification:**

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_runner -v
```

**Commit group:** `feat: run observable Crouzeix proof searches`

## Issue 4: Run Sealing And Offline Evidence Verification

**Goal:** publish selected attempts as deterministic, rights-aware evidence and
verify them offline.

**Owned files:**

- `labs/crouzeix_proof_reproduction/seal_run.py`
- `labs/crouzeix_proof_reproduction/tests/test_sealing.py`
- `evidence/crouzeix_conjecture_reproduction/PROVENANCE.md`
- `evidence/crouzeix_conjecture_reproduction/manifest.tsv`
- selected sealed run artifacts
- `crates/harp/src/sources/crouzeix_reproduction.rs`
- `crates/harp/src/sources.rs`
- focused Rust tests in the child verifier

**Test-first slices:**

1. Deterministic archives have stable bytes, ordering, modes, owners, and
   timestamps.
2. Archive extraction rejects traversal, symlinks, special files, duplicate
   members, digest mismatch, and decompression overflow.
3. Secret scanning rejects credentials without redacting raw evidence.
4. Seal refuses an existing destination.
5. Manifest validation rejects unknown files and missing receipts.
6. Offline source verification checks every normalized and raw digest.
7. Historical prompt bytes remain remote-only and are represented only by the
   existing receipt plus run input digest.
8. Failed and non-promoted runs remain valid evidence outcomes.

**Exact verification:**

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_sealing -v
cargo test -p harp sources::crouzeix_reproduction::tests --lib \
  -- --test-threads=1
```

**Commit group:** `evidence: capture Crouzeix reproduction attempts`

## Issue 5: Live Blind Runs

**Goal:** execute and seal one historical arm and one observable orchestration
arm under the fixed current runtime.

**Inputs:**

- historical prompt extracted from public Git commit `9df0783`;
- model `gpt-5.6-sol`;
- TRAE CLI `0.200.19(internal edition)` or the exact version observed at run
  time;
- no network, MCP, or proof-source access;
- the call and timeout ceilings in the design.

**Procedure:**

1. Prepare separate `H` and `O` run directories.
2. Inspect `run_spec.json` before execution.
3. Execute `H`; retain all outputs regardless of result.
4. Execute `O`; retain every route and failed call.
5. Seal generation outputs before reference-aware review.
6. Record call and aggregate usage accounting.
7. Do not rerun an arm under the same run ID.

**Acceptance:**

- Both attempts have complete receipts or typed failure receipts.
- No generation prompt contains proof-specific phrases from either public
  mechanism unless generated by a blind worker.
- The historical prompt delta is exactly the output path.
- The orchestrated run proves initial context independence by prompt digests
  and parent inventories.
- No mathematical success is claimed before review.

**Verification:**

```sh
python3 labs/crouzeix_proof_reproduction/seal_run.py --check RUN_DIR
```

**Commit group:** included with Issue 4 evidence after validation.

## Issue 6: Adversarial Mathematical Review

**Goal:** review sealed candidates against correctness rather than published
proof similarity.

**Owned files:**

- sealed normalized review records under
  `evidence/crouzeix_conjecture_reproduction/`
- `knowledge/crouzeix_conjecture/reproduction_results.md`

**Review order:**

1. Anonymize candidate arm labels.
2. Perform logical review.
3. Perform operator-theory review.
4. Reconcile findings without revealing treatment.
5. Freeze completeness outcome.
6. Only then classify Jin-like, Lorist-Schwenninger-like, or other mechanism.
7. Record formal status separately.

**Acceptance:**

- Every finding has ID, severity, exact locator, statement, and disposition.
- A complete outcome has no unresolved critical finding or theorem-strength
  obligation.
- An incomplete or invalid outcome names the earliest decisive gap.
- A timeout or unavailable review is `indeterminate`, not success or failure.
- Existing Lean builds remain `blocked`.

**Verification:**

```sh
python3 labs/crouzeix_proof_reproduction/seal_run.py --check RUN_DIR
rg -n "complete|incomplete|invalid|indeterminate|blocked" \
  knowledge/crouzeix_conjecture/reproduction_results.md
```

**Commit group:** `docs: report Crouzeix reproduction results`

## Issue 7: Packet, Atlas, Search, And Product Contracts

**Goal:** register research, protocol, and results without weakening the
existing Crouzeix packet contract.

**Owned files:**

- `knowledge/crouzeix_conjecture/reproduction_research.md`
- `knowledge/crouzeix_conjecture/reproduction_protocol.md`
- `knowledge/crouzeix_conjecture/reproduction_results.md`
- `knowledge/crouzeix_conjecture/crouzeix_conjecture_index.md`
- `knowledge/crouzeix_conjecture/source_registry.md`
- `knowledge/crouzeix_conjecture/claim_evidence_ledger.md`
- `crates/harp/tests/crouzeix_conjecture_knowledge_packet.rs`
- `crates/harp/src/corpus/mod.rs`
- `crates/harp/src/corpus/tests.rs`
- `crates/harp/tests/cli.rs`
- `docs/product-contract.md`
- generated Atlas corpus and offline export
- `docs/import-receipt.md`

**Test-first slices:**

1. Extend the exact packet roster and fail before registration.
2. Add stable reproduction claim IDs and require reader routes.
3. Require source-registry entries for historical prompt and local run.
4. Require result language to use prospective/attempted rather than exact
   historical replay.
5. Require formal statuses from the closed enum.
6. Register every new document and regenerate derived artifacts.
7. Update exact document, evidence, search, and import counts.
8. Refresh payload digest last.

**Exact verification:**

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet \
  -- --test-threads=1
cargo test -p harp corpus::tests --lib -- --test-threads=1
cd atlas && corepack pnpm run test
mise run verify
```

**Commit group:** `docs: integrate Crouzeix reproduction study`

## Issue 8: Independent Review And Landing

**Goal:** review the exact feature diff along mathematical, evidence, security,
and repository-contract axes, repair findings, and land locally.

**Review base:** the feature branch's merge base with current `master`.

**Required checks:**

- source and licensing boundaries;
- generation/reference leakage;
- command and filesystem safety;
- event and usage accounting;
- deterministic archive behavior;
- mathematical claim calibration;
- packet and generated-output parity;
- commit trailers;
- clean worktrees; and
- full release gate.

**Landing:**

1. Merge current `master` into the feature branch if it advanced.
2. Resolve registries and counts by union.
3. Regenerate Atlas and search outputs.
4. Refresh `docs/import-receipt.md` last.
5. Run `mise run verify`.
6. Confirm primary checkout is clean and an ancestor.
7. Fast-forward local `master`.
8. Do not push.

**Commit trailer:**

```text
Co-authored-by: TRAE CLI <noreply@bytedance.com>
```
