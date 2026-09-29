# Reference: ultron

- Repo: `/data02/home/philip.yang/workspace/ultron` (read-only; nothing modified, no commits)
- Commit read: `5061e89cc39b108ea4114acee32bbc902c4672e9` ("Restate the Ultron thesis for the refocused repository"), branch `main`, no git remote configured
- Date: 2026-09-29
- Read state: clean tracked tree; untracked `.scratch/production-release-readiness/` (contains a `gitleaks` binary and tarball; see section 8)
- HEAD moved twice during the read (`32be36b` -> `83eebdc` -> `5061e89`); another agent was refocusing the repo the same afternoon. The last hop changed only `docs/ultron-thesis.md` (new) and `docs/repository-map.md`. Code, tests and trackers cited below were read at `83eebdc` and are unchanged at `5061e89`.
- Evidence labels: **[R]** read in a file (path:line); **[X]** executed by me today, read-only; **[I]** inferred, not stated by any source.
- All paths are relative to the ultron repo unless prefixed. Line numbers are from `5061e89`.

Reuse note: `robust-training-observatory/docs/research/2026-09-29-controller-repos-source-map.md` was checked against the code. Its Ultron description (README line 3 "distributed training operating system") is stale: the README was rewritten on 2026-09-29 (commit `32be36b`). Its code claims (policy package contents, dry-run policy) verify. Its recommendation to treat Ultron as the "internal controller spine" is now contradicted by Ultron's own current framing (section 1).

---

## 0. One-paragraph summary

Ultron today is two things sharing one repo. (a) A **scribe/knowledge repo**: source-backed notes, a claim-labelling discipline, imported knowledge packets, and file-based work trackers about robust distributed training and the sibling repos. (b) A **fixture-level, dry-run policy model** (`tools/ultron_policy/`, about 3,600 lines of Python, 88 passing tests) that encodes three invariants (topology identity, update commit boundary, evidence-backed destructive action) and a conservative rollback-first decision function. It is **not** a running controller, calls no sibling repo, and no sibling repo imports it. Until 2026-09-29 it was also the canonical home of the cross-repo agentic-engineering policy and sync tool; that moved to corenius (section 7.4). Its own CONTEXT says the controller's "final shape is still to be determined" (`CONTEXT.md:6-8`).

---

## 1. Problem, audience, and the pain without it

### 1.1 The stated problem

Two problems, at two layers of ambition.

**Layer 1: what the code and design docs address (the original thesis).** Robust large-scale training needs something that decides *what safe action to take* after an incident, and no sibling repo provides it. The thesis synthesis says every repo has "real substrate" for identity, commit semantics and causal evidence, but "what does not yet exist is the transactionally safe controller that joins them and acts on them" (`.scratch/ultron-thesis/issues/17-final-synthesis.md:87-94`) [R]. The architecture doc's anchor claim: ByteRobust "makes interruption inexpensive; it does not generally eliminate interruption" (`docs/ultron-architecture.md:192`) [R]. Concrete pains it enumerates for a restart-centric system (`docs/ultron-architecture.md:182-216`) [R]:

- over-eviction of a whole pipeline group when the faulty worker cannot be pinned down;
- symptom-level communication diagnosis (stack aggregation, no per-collective state);
- MFU decline that could be hardware, sequence skew, GC, allocator, or data loading, with no counterfactual analysis;
- static topology assumptions (no per-batch or per-epoch topology);
- no semantic integrity plane for FP8 or MoE (silent corruption, router collapse, token drop).

The section 5.3 example is the sharpest statement of the operator pain: without common identity "the operator is left manually correlating 'rank 143' in one system, 'GPU UUID X' in another, and 'collective sequence 5,201' in a third" (`docs/ultron-architecture.md`, text after the `IncidentEvent` schema in 5.3) [R].

**Layer 2: what the refocused repo says it is for (2026-09-29).** Ultron is "the workspace's scriv": it catalogs, maintains and researches knowledge about followed systems, "enforces the rules for how that knowledge and agent work are kept, and tracks down the sources behind each claim" (`CONTEXT.md:3-8`) [R]. The followed systems are ndtimeline/ndtl, Arnold, Myriatron, Torchetta and Milano (`CONTEXT.md:17-19`) [R]. Three durable roles: robust-training controller, research journal, knowledge aggregator (`AGENTS.md:17-19`, `README.md:10-25`) [R].

### 1.2 For whom

- The architecture owner (the one human named as decision authority throughout `docs/agents/agentic-engineering.md`) who needs a durable record of what was learned and why. [I: no persona doc exists; inferred from workflow text and the single-author git history.]
- Coding agents working across sibling repos, who need shared vocabulary and a rule for where cross-repo intent lives.
- Reliability/platform designers, named as users in the workspace story (`docs/workspace-repository-story.md:142`, the `ultron` row) [R].

### 1.3 What goes wrong without it (concretely)

Stated or directly implied by sources:

1. Evidence from different tools cannot be joined, so a policy has nothing safe to act on. The identity validator refuses records whose load-bearing fields are silently absent, "so policy code cannot accidentally infer safety from missing identity" (`tools/ultron_policy/identity.py:29-33`) [R].
2. A recovery action can be chosen without knowing how far suspect work propagated (local, cross-rank, optimizer-committed, checkpointed). The ledger exists to make that question answerable (`tools/ultron_policy/ledger.py:27-44`) [R].
3. Research claims drift from their sources. The imported-packet rule keeps `CLAIM/EVIDENCE/INFERENCE/SPECULATION/MISSING` labels intact until an Ultron-local receipt promotes them (`knowledge/README.md:15-21`, `docs/tarocco-knowledge-sync.md`, "How Ultron should read this material") [R].
4. Cross-repo work has no single tracker and each repo invents its own agent rules (`docs/agents/agentic-engineering.md:33-35`) [R]. This one has moved to corenius.

Not documented anywhere: an incident it actually prevented. There is no production use, no incident log, no measured benefit. Every "pain" above is a design-time argument sourced to papers (ByteRobust, FT-HSDP, Mycroft, ARGUS, Aegis, etc.), not to a local outage. [R: absence confirmed by reading all trackers and `docs/`.]

---

## 2. Non-goals, boundaries, authority, dependents

### 2.1 Non-goals (stated)

- "Ultron is not the training runtime." TorchTitan, Monarch, NCCL, Torchetta and NDTimeline keep their own code and execution (`README.md:20-25`) [R].
- "Ultron should not become a general model-training, serving, kernel-search, or release-governance monorepo." Adjacent work belongs only if it feeds a robust-training control question: "what happened, which state was affected, what action is safe, and what evidence proves that" (`CONTEXT.md:21-25`) [R].
- The Policy Incident "never executes the action" (`CONTEXT.md:75-77`); Incident Replay "remains advisory" (`.scratch/ultron-incident-replay/map.md`, decisions) [R].
- Excluded from the first build: production cluster integration, live NCCL abort/revoke/shrink, Monarch RDMA or scheduler replacement, CUTLASS kernel changes, Ferric runtime containment, SDC campaigns beyond fixtures, large GPU experiments (`.scratch/ultron-build/spec.md:143-154`) [R].
- The ResilientTitan proposal says "Do not begin with RDMA checkpointing or custom CUTLASS faults" (`docs/resilient-titan-draft-proposal.md:1446-1449`) [R].

### 2.2 What was deliberately removed today (history you will see in `git log`)

Commits `5cda311`, `7a1e10b`, `32be36b`, `83eebdc`, `5061e89` (all 2026-09-29) removed the synthetic architecture lab, kernel-evaluation loop, release-readiness estate, Ollama rootfs harness, all MLU research, and the DCGM lesson course, keeping only a text DCGM guide (`docs/gpu-internals-dcgmi-learning-guide.md`) [R: `git log`, `git show --stat`]. Parked, not deleted: `.scratch/canonical-model-mastery/`, `.scratch/model-mlsys-closed-loop/`, `.scratch/manage-cutlass/` (`docs/repository-map.md:73-81`) [R]. The parked model-mastery tree encodes a Vaso-centric plan (`.scratch/canonical-model-mastery/spec.md`); do not read it as current scope.

### 2.3 What it treats as authority

| Question | Authority | Evidence |
| --- | --- | --- |
| Sibling repo code, behavior, runtime | The sibling ("Owning Repository"); Ultron "may cite it or open child work against it, but ownership stays with that repository" | `CONTEXT.md:46-49` [R] |
| Imported knowledge claims | The packet's own labels and source registry, until an Ultron-local receipt promotes them | `knowledge/README.md:15-21` [R] |
| Whether an idea becomes controller contract | Only a new workstream stating the robust-training question it answers | `docs/repository-map.md:83-93`, `CONTEXT.md:99-102` [R] |
| Architecture and merge | A human; agents "stop at the reviewed branch" | `docs/agents/agentic-engineering.md:107-110,125-126` [R] |
| Committed run truth (ResilientTitan design) | `RunCoordinator` is the only writer; the event log is truth, state is rebuildable by replay | `.scratch/resilient-titan/map.md`, decision 01 [R] (design only, unimplemented) |
| Arnold facts | Arnold. Facts about Arnold belong in Ultron research notes, "not in Arnold" | `docs/ultron-thesis.md:45` [R] (draft row labelled Inference) |

### 2.4 What depends on Ultron

- **Code dependents: none found.** No sibling imports `tools.ultron_policy` [I: I grepped only Ultron's own tree and a few sibling READMEs/AGENTS files; I did not exhaustively grep every sibling.] `git grep` of Ultron for milano/torchetta/arnold/ndtl outside `knowledge/` and `vendor/` finds only prose in `CONTEXT.md`, `README.md`, `docs/workspace-repository-story.md`, `docs/research/workspace-focus-audit-2026-09-12.md`, `docs/ultron-thesis.md` [X].
- **Process dependents (legacy):** every repo in `docs/agents/agentic-workflow-repos.json` carries a managed block in its instruction file that says `CONSTITUTION.md` and `docs/agents/agentic-engineering.md` are mandatory (`tools/sync_agentic_workflow.py:18-28`) [R]. Those files in sibling repos were generated from Ultron. Their header still says "Canonical source: ultron/docs/agents/agentic-engineering.md" (`docs/agents/agentic-engineering.md:1`) [R], while corenius now claims to be canonical (section 7.4).
- **Doc dependents:** `robust-training-observatory` cites Ultron in research notes only, and explicitly does not make it an app-visible source namespace (`robust-training-observatory/docs/research/2026-09-29-controller-repos-source-map.md`, intro) [R].
- **Branch convention dependents:** siblings carry an `ultron/mainline` branch; `--check` reports `MISSING-BRANCH` when absent (`tools/sync_agentic_workflow.py:361-391,419-500`; `docs/agents/agentic-engineering.md:37-44`) [R].

---

## 3. Architecture

### 3.1 Repository layout by role

```
ultron/
  README.md CONTEXT.md AGENTS.md RESOURCES.md CONSTITUTION.md   identity, glossary, entry points
  docs/
    repository-map.md          scope map and promotion rule (start here)
    ultron-thesis.md           restated thesis, draft (new today)
    ultron-architecture.md     2274 lines, ByteRobust-anchored technology map + "Ultron" design
    resilient-titan-draft-proposal.md   1468 lines, layered controller research program (draft)
    workspace-repository-story.md       398 lines, portfolio roadmap + repo inventory (PARTLY STALE)
    tarocco-knowledge-sync.md  sync receipt for imported packets
    agents/                    workflow policy + tracker rules (canonical source moved to corenius)
    research/                  journal (parked entries + 2 controller-adjacent audits)
  tools/
    ultron_policy/            13 modules, ~3,600 LOC  (the only executable "controller")
    sync_agentic_workflow.py  531 LOC, policy distribution (legacy home)
    host_sample_native/       C++ Bazel no-BPF-load preflight stub
  tests/ultron_policy/  (13 files, 67 tests)   tests/test_sync_agentic_workflow.py (21 tests)
  knowledge/tarocco/ (imported)  knowledge/tla-distributed-training/ (course)   ~59 MB
  .scratch/<workstream>/       map.md, spec.md, issues/NN-*.md, fixtures, notes  ~119 MB
  vendor/vendor.toml           one pinned partial-clone manifest (qianchuan_100b); contents git-ignored
  .worktrees/                  16 registered worktrees, ~679 MB (section 8)
```
[R: `ls`, `git worktree list`, `du`]

### 3.2 The policy pipeline (the only real data flow)

```
 native artifact (JSON dict)               typed evidence            ledger record (synthetic!)
 ndtimeline row ----+                      EvidenceRecord               UpdateLedger
 NCCL prof v7 evt --+--> *_adapter.py --->  {class, source,   --+
 py-spy/gdb dump ---+     (fixture-only)     confidence,          |     decide_action(ledger_record,
 profiler / DCGM ---+                        identity envelope,   +---> evidence list)
 straggler output --+                        payload,                       |
 host-sample agg ---+                        direct_policy_input}           v
                                                                     PolicyDecision(action, rationale)
 framework evidence --> side table (user_tag -> identity) -> NCCL adapter        (never executed)
```
[R: `tools/ultron_policy/*.py`; the framework-to-NCCL join is `framework_side_table_adapter.py:19-60`, `nccl_profiler_adapter.py:19-92`.]

Driver: `FixtureIncidentRunner` loads a JSON manifest of incidents, builds evidence, constructs a ledger by replaying all five states up to the requested one, calls `decide_action`, and compares to `expected_action` (`tools/ultron_policy/incident_runner.py:66-171`) [R].

### 3.3 Key abstractions and invariants (as enforced by code)

**Topology identity envelope** (`identity.py:26-83`) [R]. Five groups, all required: `run` (job/run/attempt ids concrete; code/config/container hash present), `logical` (global_rank, world_size concrete; hsdp_replica, dp, pp, tp, cp, ep, process_group id and generation present), `physical` (host, device, gpu_uuid, nic, port, fabric_plane, rack, switch_path present), `temporal` (step, topology_epoch concrete; microbatch, active_replica_set, process_group_generation present), `artifact` (at least one of checkpoint_generation, collective_sequence, framework_op_id, kernel_id, metric_window, evidence_source_id). "Present" means a concrete value **or** a marker `{"state": "unknown"|"not_applicable", "reason": <non-empty>}` (`identity.py:9,246-267`). Invariant: absence is an error; ignorance must be written down with a reason. This is the most reusable idea in the repo.

**Evidence record** (`evidence.py:17-96`) [R]. Eight classes: framework, collective, numerical_contract, controller_fleet, stack_snapshot, hardware_telemetry, straggler_attribution, host_sample_aggregate; four confidence levels (direct_causal, substrate, operator_supplied, conservative_assumption); four source stabilities (public-stable, public-versioned, internal-wrapped, diagnostic-only). Invariants: `direct_policy_input` is mandatory and boolean; `internal-wrapped` sources may never be direct policy input (`evidence.py:76-79`); each class has required payload keys (`evidence.py:131-190`); identity must validate.

**Update ledger** (`ledger.py:27-176`) [R]. States: never_contributed, local_only, cross_rank_visible, optimizer_committed, durably_checkpointed. Append-only, in-memory. The first record for a (replica, step, epoch) must be `never_contributed`, and every later record must advance by exactly one state (`ledger.py:130-139`). Context is mandatory (active_replica_set, process_group_generation, data_cursor, rng_marker, model_version, optimizer_version; `ledger.py:11-18`), plus `collective_sequence` from cross-rank state onward, `consumed_collective is True` for commit, `checkpoint_generation` for durable.

**Decision function** (`policy.py:28-92`) [R], evaluated in this order:
1. any `numerical_contract` evidence present: `ROLLBACK` (line 38-43);
2. ledger state durably_checkpointed / optimizer_committed / cross_rank_visible: `ROLLBACK` (45-62);
3. `controller_fleet` evidence with `remove_replica` eligibility and a `replica-*` domain: `REMOVE_REPLICA`; with `quarantine_domain` eligibility: `QUARANTINE_DOMAIN` (63-82);
4. ledger never_contributed / local_only: `DISCARD_LOCAL` (84-89);
5. otherwise `GLOBAL_STOP` (90).

**Cross-adapter join.** Framework evidence owns run/rank/step identity; the communication context owns NIC/fabric/process-group generation; NCCL profiler events join through an opaque nonzero `userProfilerTag` looked up in a side table. NCCL communicator and sequence ids are "substrate evidence" and "do not prove framework identity without the opaque user tag join" (`nccl_profiler_adapter.py:19-29`) [R]. `join_key` (job, run, attempt, rank, step, microbatch, epoch) is the broad join used in tests (`evidence.py:99-119`).

**Adapters** [R for existence and headers; contents beyond the headers only skimmed]:

| Module | LOC | Input | Policy use |
| --- | --- | --- | --- |
| `framework_side_table_adapter.py` | 83 | framework evidence + comm contexts | builds tag table |
| `nccl_profiler_adapter.py` | 138 | `ncclProfiler_v7` collective event | `direct_policy_input: True` (line 64) |
| `ndtimeline_adapter.py` | 312 | `DeviceTimerStreamRecord.to_dict()`-shaped row | `diagnostic-only`, `False` (63,66) |
| `stack_snapshot_adapter.py` | 610 | py-spy dump / gdb transcript / skip / failure | not direct |
| `profiler_dcgm_adapter.py` | 459 | profiler window, DCGM window, unavailable | not direct |
| `straggler_analysis_adapter.py` | 388 | Ultron trace <-> straggler_analysis I/O | `False` (149,168) |
| `host_sample_adapter.py` | 599 | eBPF/perf host-sample aggregate + no-load preflight | must be `False` (164-165) |

### 3.4 What the code does NOT do (verified, and surprising)

1. **Most evidence never reaches the decision.** `decide_action` looks only at `numerical_contract` and `controller_fleet` classes (`policy.py:38,63`). `collective`, `framework`, `stack_snapshot`, `hardware_telemetry`, `straggler_attribution` and `host_sample_aggregate` evidence are validated and then ignored. In the e2e fixture, the `collective_entry_ambiguous_commit` incident yields `rollback` purely because the ledger says `cross_rank_visible`; the NCCL async-error evidence is inert (`.scratch/ultron-build/fixtures/e2e/policy_incidents.json`) [R]. **[I]** Roughly 2,400 of 3,600 LOC are adapters whose outputs cannot change any action today.
2. **`direct_policy_input` and `confidence` are never read by policy.** grep finds `direct_policy_input` only in validation and adapters, `confidence` only when cloning records in the runner [X: `grep` of `tools tests`]. The "internal-wrapped cannot be direct" rule is enforced at validation, not at decision time.
3. **Evidence is not checked against the ledger record.** `decide_action(ledger_record, evidence)` never compares identity between them. `join_key` is called only from tests (`tests/ultron_policy/test_incident_runner.py:56`, `test_evidence.py:64-68`, `test_framework_side_table_adapter.py:44`) [X]. The runner uses a hard-coded synthetic key (`job_id="job-ultron-fixture"`, `incident_runner.py:145`) and fabricates ledger history by replaying every state up to the target (`incident_runner.py:152-155`) [R]. So INV1 (identity) and INV2 (ledger) are each enforced in isolation, not jointly.
4. **"Unknown identity forces conservative policy" is documented but not implemented.** Ticket resolution: "safety-load-bearing unknowns force conservative policy" (`.scratch/ultron-build/spec.md:197-198`); `policy.py` contains no reference to `unknown` markers [X: grep returned nothing].
5. **`GLOBAL_STOP` is unreachable with the current enum.** All five ledger states are handled by rules 2 or 4, so rule 5 cannot fire [I: from reading `policy.py:45-90` against `ledger.py:27-33`]. `CONTINUE` is defined (`policy.py:13`) and listed in the spec's action set, but no rule returns it [R].
6. **The ledger has no terminal or abort state.** Transitions are strictly +1 from `never_contributed`; discarded, rolled-back or superseded work cannot be recorded [I: from `ledger.py:130-139`]. This is a gap for the "remove replica, catch up, rejoin at a new epoch" story the architecture doc tells (`docs/ultron-architecture.md`, 5.4-5.5).
7. **Real ndtimeline rows are rejected by the adapter.** Producer: `distributed_timeline/bytedance/ndtimeline/handlers/parser_handler.py:52-78,100,107`. `run_id` is declared `int` and `extra_tag` is `List[str]` (each element a JSON string like `{"microbatch": n}`, lines 327, 438-442). The adapter requires `run_id` to be a non-empty string (`ndtimeline_adapter.py:256-260`) and `extra_tag` to be a mapping (`ndtimeline_adapter.py:55,219-227`). **[X]** I ran the adapter on a synthetic span: `extra_tag=[]` raised "extra_tag must be an object"; `run_id=7` raised "run_id must be a non-empty string"; `extra_tag={}` passed. The unit fixture uses a dict and a string (`tests/ultron_policy/test_ndtimeline_adapter.py:151-183`), so tests pass while the documented input shape (`to_dict()` output) does not. The microbatch that `extra_tag` actually carries is also discarded: the adapter always emits `microbatch: unknown` (`ndtimeline_adapter.py:112-115`). Not tested against a real ndtimeline artifact [I]. The worklog says the real GPU ladder was blocked by B200 occupancy (`.scratch/workstream-worklog/map.md`, frontier).

---

## 4. Key design decisions, trade-offs, rejected alternatives, lessons

### 4.1 Decisions with recorded alternatives

| Decision | Chosen | Rejected / deferred | Source |
| --- | --- | --- | --- |
| First implementation locus | Ultron-local dry-run contracts and fixtures first; TorchTitan/NCCL only as seam analyses | Starting in TorchTitan (strongest locus but "not the safest first place to define the policy contracts") | `.scratch/ultron-build/spec.md:61-64`, critique in `issues/01-independent-critique.md` [R] |
| Public test seam | Pure library API `tools.ultron_policy`, fixtures under `.scratch/ultron-build/fixtures/` | live or mocked integration seams | `spec.md:194-196` [R] |
| Identity semantics | Concrete / `unknown` / `not_applicable`, each explicit with reason | optional fields, null | `identity.py:9,246-267`; ticket 12 [R] |
| NCCL join | `ncclCollConfig_t.userProfilerTag` + profiler ABI v7 (public-versioned) | RAS adapter first; bare communicator/sequence equality | `.scratch/ultron-build/issues/08-nccl-evidence-adapter-spike.md`, Answer [R] |
| Source stability classes | public-stable / public-versioned / internal-wrapped / diagnostic-only; internal-wrapped never direct | treating any parsed internal record as policy input | ticket 08 acceptance; `evidence.py:76-79` [R] |
| Observability first source | ndtimeline as structured span source, raw parsed records over Chrome trace / private MQ / ClickHouse | trace-viewer artifacts | `.scratch/workstream-worklog/map.md`, decision 01 [R] |
| Stack sampling | `py-spy dump --json` first, `gdb -batch` fallback; `perf` low-rate as native candidate; eBPF/libbpf only via a separate no-load schema first | BCC (prototype only), always-on ARGUS clone now | worklog decisions 02, 17, 19 [R] |
| Straggler analysis | offline what-if stage over normalized traces; wrapped as non-policy evidence needing corroboration | always-on detector | worklog decisions 04, 12 [R] |
| Incident replay | deep verified-bundle module + separate evaluator + label-free bundles (approach B) | (A) extend `FixtureIncidentRunner` (labels sit beside proposer input); (C) signed content-addressed archive service | `.scratch/ultron-incident-replay/spec.md:36-88` [R] |
| ResilientTitan baseline | Approach B: Monarch supervision + TorchFT in-band + TorchTitan semantics + DCP anchor + host guardians | (A) external supervisor only (kept as baseline); (C) Monarch-native training runtime (too big) | `docs/resilient-titan-draft-proposal.md:84-133` [R] |
| Authority for committed run truth | single writer `RunCoordinator`, event-sourced, adapters cannot commit | adapters committing epochs or generations | `.scratch/resilient-titan/map.md`, decision 01 [R] |
| Execution governance for experiments | Python in `uv` venv inside a `bwrap` rootfs; native code only via Bazel | ad hoc host runs | `.scratch/workstream-worklog/map.md`, Route [R] |

### 4.2 Design principles that recur

- **Conservative under uncertainty.** Numerical or cross-rank uncertainty resolves to rollback or global stop, never continuation (`CONTEXT.md:129-132`; `policy.py:38-62`) [R].
- **Smallest safe domain**, but only after evidence supports it (`docs/ultron-architecture.md` 5.5; hypothesis H2 in the proposal) [R].
- **Independent liveness.** A watchdog in the same control flow as a blocked NCCL call cannot detect the block; use a separate process (proposal H3, "Four important corrections" 1) [R].
- **Provenance beats convenience.** Adapters take `artifact_sha256`, `parser_version`, raw source fields, and uncertainty markers (`ndtimeline_adapter.py:110-160`) [R].
- **Claims carry labels.** Imported packets keep CLAIM/EVIDENCE/INFERENCE/SPECULATION/MISSING (`knowledge/README.md:15-21`) [R]. This document follows the same idea.
- **No blanket normalization of imported artifacts** because some are digest-bearing (`docs/tarocco-knowledge-sync.md`, Maintenance) [R].

### 4.3 Lessons learned and recorded failures

- **Critique caught real design errors.** In ultron-build, separate-context critique found: spec marked ready before decision tickets resolved; blocker graph over-serialized local schema work behind cross-repo trackers; spikes could be read as authorization to edit siblings; hot-path claims lacked proof classes. Fix: tickets 10-14 and the `static-seam-only | fixture-proven | performance-proven` classification (`.scratch/ultron-build/issues/01-independent-critique.md`, Answer; `issues/07-...md` acceptance) [R].
- **Incident-replay critique** required labels to bind immutable bundle content, full topology joins for replayed evidence, recursive immutability, an implementation digest on policies, and descriptor-relative no-follow filesystem traversal (`.scratch/ultron-incident-replay/map.md`, decisions) [R]. The spec still concedes the firewall is "a capability seam, not cryptographic secrecy inside one developer checkout" (`spec.md:533-546`) [R].
- **Environment failure that is on record:** the governed rootfs rerun of low-rate sampling produced `collector_unavailable` for both `perf` and `py-spy` because neither exists in the rootfs, so no governed overhead claim is promoted (worklog decision 21) [R]. The ndtimeline GPU ladder was blocked on 2026-08-27 because all 8 B200s were occupied by other users' vLLM workers; `fsdp_tp_pp_torch` never launched (worklog, 2026-08-27 entries) [R].
- **Managed-repo health (2026-09-07):** the policy check failed with 10 drifted paths; only 3 of 8 managed repos used `ultron/mainline` as default; 43 registered worktrees; no public repo release-ready (`docs/research/managed-repo-health-2026-09-07.md`, Executive status) [R; snapshot is 3 weeks old, [I] current state unknown].
- **Documentation rot from the refocus.** The workspace story still cites `README.md:12-18` and `README.md:3-18` for claims the rewritten README no longer contains, and its roadmap still treats the parked model-mastery plan as canonical (`docs/workspace-repository-story.md:8-10,142,398`) [R]. It is labelled "useful scribe context, but not controller authority" (`docs/repository-map.md:45-46`) [R].

---

## 5. Build, test, release, verify (trust and evidence model)

### 5.1 Build and test

- No package manifest. Python modules import as `tools.ultron_policy.*` from the repo root (namespace packages: no `__init__.py` in `tests/`, `tests/ultron_policy/`, or `tools/`) [X: `ls`].
- Documented verification: `PYTHONDONTWRITEBYTECODE=1 python -m unittest discover -s tests -p 'test*.py'` (`README.md:62-68`) and `bazel test //tools/host_sample_native:preflight_info_test` (`README.md:70-73`) [R].
- **[X] That documented command runs only 21 tests** (Python 3.14.5 from a Spack view). Discovery does not descend into `tests/ultron_policy/` because it is not an importable package. Running the policy modules explicitly (`python3 -m unittest tests.ultron_policy.test_policy ...`, all 13 files) ran **67 tests, all OK**. So the headline verification command silently skips the controller tests. I did not run Bazel, and `pytest` is not installed in that interpreter.
- Bazel 9.2.0, bzlmod, single dep `rules_cc` (`MODULE.bazel`, `.bazelversion`, `.bazelrc`). The only native target is a preflight stub that prints `bpf_load=disabled` and touches no BPF (`tools/host_sample_native/preflight_info.cc`) [R].
- Tests are behavior-through-public-seam style (real fixtures, no mocks) per the ultron-build spec seam decision [R]; four e2e incidents cover local discard, collective-entry ambiguity, replica removal, domain quarantine (`fixtures/e2e/policy_incidents.json`). Numerical-uncertainty and committed-contamination behavior are covered in `tests/ultron_policy/test_policy.py` [I: not read line by line].

### 5.2 Trust and evidence model

What exists (all repo-local, all fixture-level):

- **Typed, validated evidence with stability class and confidence.** See 3.3.
- **Provenance fields:** `artifact_sha256`, parser/converter/analyzer versions, `input_trace_hash`, `raw_source_fields` preserved verbatim (`ndtimeline_adapter.py`, `straggler_analysis` payload requirements in `evidence.py:167-174`) [R].
- **Safe-failure records:** stack snapshot statuses (`ok, skipped, collector_unavailable, attach_denied, timeout, partial, error`), host-sample statuses including `permission_denied`, `bpf_unprivileged_disabled`, `seccomp_denied`, `btf_missing`, `not_validated_in_rootfs` (`stack_snapshot_adapter.py:16-19`, `host_sample_adapter.py:49-65`); a no-load preflight builds a record without touching BPF or perf syscalls (`host_sample_adapter.py:358`) [R].
- **Tracker discipline as the audit trail:** spec + numbered tickets with `Status:`, `Blocked by:`, and an `## Answer` that must cite file:line (`docs/agents/issue-tracker.md`) [R]. Ticket status counts: ultron-build 16/16 resolved; ultron-thesis 17/17; workstream-worklog 21 resolved, 4 open, 1 blocked, 1 ready-for-human; resilient-titan 11 resolved, 11 ready-for-human; incident-replay 1 resolved, 7 open [X: grep of `Status:` lines].
- **Review gate (process):** ticket-level Standards + Spec review, then `roborev review --branch` or a separate-context reviewer, at most three repair iterations, human commit/push/PR/merge (`docs/agents/agentic-engineering.md:96-110`; `.scratch/ultron-build/issues/14-independent-verifier-path.md`, Answer) [R].

What does **not** exist:

- No signing, no release process, no CI in this repo [I: no `.github`, no CI config found by `ls -a`]. The release-readiness estate was removed today; an untracked leftover remains.
- No content-addressed or append-only evidence store. The incident-replay spec that would add SHA-256-verified bundles is `review-ready` but unimplemented: ticket 08 (written-spec approval) is the gate and no `tools/` module for it exists (`.scratch/ultron-incident-replay/map.md`, Frontier) [R].
- No real-artifact validation of any adapter (section 3.4 item 7).

Repo-level truth rule worth copying: **a note is not controller authority until a workstream or policy test promotes the claim** (`docs/research/README.md:4-6`) [R].

---

## 6. Interfaces with other repos

Legend: "calls" = executable dependency; "reads" = design-time reading; "shape" = an input shape the code assumes.

### 6.1 Summary matrix

| Repo | Ultron calls it? | It calls Ultron? | Shared vocabulary | Real contract in code | Status |
| --- | --- | --- | --- | --- | --- |
| distributed_timeline (ndtimeline) | no | no | span/metric/rank/step/pg_id/seq_id | `ndtimeline_adapter.py` assumes `DeviceTimerStreamRecord.to_dict()` | shape mismatch, section 3.4 item 7 |
| ndtl | no | no | Update, phase, microbatch, collective, work | none (no adapter) | nominal; `docs/ultron-thesis.md:43` calls "ndtimeline, ndtl" the first evidence source but only the ndtimeline adapter exists |
| torchtitan | no (seam analysis only) | no | run evidence, step, attempt id, checkpoint artifact | none in code; static seam recorded in a ticket | `static-seam-only` |
| nccl | no | no | `userProfilerTag`, profiler ABI v7, communicator, sequence | `nccl_profiler_adapter.py` on fixtures | fixture-only |
| milano | no | no | none | none | listed as "followed"; relation is an Inference |
| torchetta | no | no (AGENTS.md links to workflow docs) | none | none | listed as "followed"; relation is an Inference |
| arnold | no | no | ETTR, robust training, X-Ray | none | nominal; see 6.5 |
| vaso | no | no | rootfs, bwrap | none | parked plan only |
| monarch | no | no | actor mesh, supervision | none | design proposal only |

### 6.2 distributed_timeline / ndtimeline

The adapter is the most concrete cross-repo interface and it reads a producer shape (`stage`, `dp_rank`, `tp_rank`, `ip`, `pg_id`, `seq_id`, `extra_tag`, `run_id`) taken from `DeviceTimerStreamRecord.to_dict()` (`distributed_timeline/bytedance/ndtimeline/handlers/parser_handler.py:87-110`) [R]. The worklog promotes `distributed_timeline/experiments/minimal_parallel_ndtimeline` as the research vehicle and records a repo-local workflow doc there (`.scratch/workstream-worklog/map.md`, Frontier) [R]. Ultron supplies what ndtimeline lacks: `job_id`, `attempt_id`, `topology_epoch`, `process_group_generation`, hashes are caller arguments (`ndtimeline_adapter.py:15-38`) [R], and everything not in the row becomes an explicit `unknown` marker. Fields ndtimeline cannot give: `hsdp_replica`, `microbatch` (although `extra_tag` carries it), physical placement.

### 6.3 ndtl

ndtl is a separate small Python package that writes JSONL timing records via an `Observatory` with `update`, `phase`, `microbatch`, `work`, `collective`, `p2p` context managers plus a Rust `synd` sidecar around TorchTitan (`ndtl/README.md`, first 60 lines) [R]. Its vocabulary (updates, work, collectives, microbatches) is closer to Ultron's Update Ledger than ndtimeline is. **[I]** It is the natural producer for a real ledger and framework-evidence adapter, but no Ultron adapter, ticket or mention beyond the CONTEXT/README lists exists. I did not read ndtl's schema in depth, so field-level fit is unknown.

### 6.4 TorchTitan and NCCL (seam analyses)

Child trackers exist at `torchtitan/.scratch/ultron-build/` and `nccl/.scratch/ultron-build/` [X: `ls`]. TorchTitan seam (ticket 07 Answer) [R]: `RunEvidence` resolves run/attempt identity and config hash (`torchtitan/observability/run_evidence.py:114,208,264`); `bind_distributed` captures mesh axes and device UUID (`:164`); `train_step` is the optimizer boundary and the candidate PREPARED/COMMITTED seam (`torchtitan/trainer.py:868,876`); checkpoint artifacts are declared/completed through `record_artifact` (`torchtitan/components/checkpoint.py:1128,1146`). Verdict `static-seam-only`; hot-path overhead not measured. Not re-verified against the current torchtitan tree by me. NCCL: profiler adapter first; the framework assigns a per-operation `userProfilerTag`; `ncclCommRevoke/Abort/Shrink` are action vocabulary only and "Phase 1 must not execute them" (ticket 08 Answer) [R].

### 6.5 Arnold

Arnold appears only as a name in the followed list and as a draft "Inference" row: "the platform where jobs run; a source of operational facts about placement and failure. Its facts belong in research notes here, not in Arnold" (`docs/ultron-thesis.md:45`) [R]. The corresponding real work lives in `robust-training-observatory`, whose note records the decision not to push to `mlsys/arnold`, that Arnold carries no trace of the downstream registration, and that pin drift is detected only from the observatory side (`robust-training-observatory/docs/research/2026-09-28-arnold-findings-and-downstream-registration.md`) [R]. Arnold facts there that matter for an Ultron identity/evidence model: two non-comparable ETTR definitions (X-Ray `/ettr/trend` vs ettr-service reading `megavision_ettr_log`); the effective-stage list defined three times; `robust_training_3dtimeline` and `robust_runtime_info` routes configured with no emitter found in the Agent tree [R, from the observatory note; not re-verified against Arnold]. Ultron has no adapter, no fixture, and no ticket for any Arnold artifact.

### 6.6 Milano and Torchetta

Ultron never mentions Milano or Torchetta in code or trackers. The only substantive statements: Torchetta is "Build/release engineers reproduce ByteDance libtorch training-serving forks... Owns build-system evidence, not libtorch source" (`docs/workspace-repository-story.md:140`), classified "Keep supporting" (`docs/research/workspace-focus-audit-2026-09-12.md`, the `torchetta` row, line 101), and the draft thesis row: "Their pinned toolchains and artifacts could supply the `code_hash` and `container_hash` identity fields" labelled **Inference** (`docs/ultron-thesis.md:44`) [R]. That is a plausible mapping: identity requires `run.code_hash` and `run.container_hash` (`identity.py:47-48`), and both repos produce content-addressed artifacts and digests (Milano's `AGENTS.md`; Torchetta's README). **[I]** No verification that the digest formats align. Torchetta's `AGENTS.md:44-50` points to `docs/agents/agentic-engineering.md` as "Ultron-style workflow discipline," so the process link is real; note neither Milano nor Torchetta is in the managed-repo manifest (`docs/agents/agentic-workflow-repos.json`, `repositories`) [R].

### 6.7 Vaso, Monarch, others

Vaso: the parked M1 plan names it the mandatory execution envelope (`.scratch/canonical-model-mastery/spec.md`, "Vaso execution qualification") [R]; current scope does not depend on it. Monarch: proposal role only (out-of-band supervision, `SPMDActor` boundary; `docs/resilient-titan-draft-proposal.md:28-50`) [R]; no Monarch child tracker exists [X: `ls monarch/.scratch/ultron-build` failed], consistent with the build spec marking Monarch "evidence-only" (`spec.md:199-200`).

### 6.8 Shared vocabulary (glossary terms that other repos would need to adopt)

Topology Identity, Evidence Record, Evidence Adapter, Update Ledger, Policy Incident, Incident Replay, Owning Repository, Workstream, Parked Research (`CONTEXT.md:27-102`) [R]. Identity groups: run / logical / physical / temporal / artifact. Update states: never_contributed, local_only, cross_rank_visible, optimizer_committed, durably_checkpointed. Actions: continue, discard_local, rollback, remove_replica, quarantine_domain, global_stop. Evidence stability: public-stable, public-versioned, internal-wrapped, diagnostic-only.

---

## 7. Relevance to agentic engineering

### 7.1 How agents work in it today

- Entry: `AGENTS.md` (30 lines): tracker location, triage labels, domain-doc layout, scope paragraph, and a managed block making `CONSTITUTION.md` and `docs/agents/agentic-engineering.md` mandatory (`AGENTS.md:1-30`) [R]. No `CLAUDE.md`; `.codex/` is empty [X].
- Scope guard: "Start with `docs/repository-map.md` before adding new top-level docs, trackers, or code. Park adjacent... material unless it answers a specific robust-training controller question" (`AGENTS.md:15-21`) [R].
- Work model: gated path with wayfinder maps (`map.md` with Notes / Decisions-so-far / Fog), specs, numbered tickets, `Status:` and `Blocked by:` lines; a frontier rule (open, unblocked, unclaimed, lowest number first) (`docs/agents/issue-tracker.md`) [R].
- Isolation: one worktree per ticket. Evidence of heavy use: 16 registered worktrees, ~679 MB, branches `codex/*` and `feature/*` [X: `git worktree list`, `du`]. Agents named in branch prefixes: Codex and Claude [I].

### 7.2 Historical role (only as history)

Until 2026-09-29 Ultron was the canonical source of the cross-repo agent policy: `CONSTITUTION.md` (Clanker Constitution v2026.08.11, CC BY 4.0, `CONSTITUTION.md:1`), the Agentic Engineering Workflow v2026.09.02.1 (`docs/agents/agentic-engineering.md:1`), the Matt-Superpowers crosswalk (`docs/agents/skill-orchestration.md:1`), tracker templates, `agentic-workflow-repos.json` (v2026.09.07.1, nine repositories: ultron, torchtitan, monarch, megatronlm, nccl, ferric_continuum, vaso, miles_diffusion, cutlass), and `tools/sync_agentic_workflow.py` (check/apply modes, marker-delimited managed block, atomic writes, symlink/VCS-path refusal, ensure-exists `ultron/mainline` branch). Corenius commit `0ebc58c` ("Absorb agentic-engineering policy and sync tool from Ultron", 2026-09-29 02:57) made corenius the canonical source and explicitly left "Ultron's copies, sync --apply, and the ultron-named marker and branch... for a separate migration" (commit message) [R]. Do not re-document the policy here; read corenius.

### 7.3 Mechanisms worth reusing in corenius (independent of Ultron's controller)

1. **Explicit-unknown identity markers.** `{state: unknown|not_applicable, reason}` with a validator that rejects absence. Directly applicable to agent-run provenance (model, repo commit, tool versions, sandbox, session id): "I do not know" is recordable, "I forgot" is an error (`identity.py:246-267`).
2. **Evidence class + stability + `direct_policy_input` gate.** A way to say some observations are diagnostic only and must not drive an action. Caveat: Ultron enforces it at validation only (3.4 item 2). Enforce at the decision seam.
3. **Append-only, strictly ordered visibility ledger** with mandatory context per state. Maps to agent work: proposed, locally verified, reviewed, integrated, released. Add a terminal/abort state (3.4 item 6).
4. **Label-free bundle + separate oracle + advisory replay** for evaluating agent policies without leaking answers (`.scratch/ultron-incident-replay/spec.md:89-133,533-546`). Approach A was rejected precisely because labels sat next to proposer-visible inputs.
5. **Proof classification for claims** (`static-seam-only | fixture-proven | performance-proven`; stability classes) to stop "plausible" from becoming "verified".
6. **Authority boundary block at the top of every spec** naming what the tracker does *not* authorize (commits, pushes, PRs, cluster access, destructive action, sibling edits) and "child ticket is necessary but not sufficient" for sibling changes (`.scratch/ultron-build/issues/01-independent-critique.md`, Answer) [R].
7. **Claim labels preserved on import** and "promotion needs a local receipt" (`knowledge/README.md:15-21`).
8. **Parked-not-deleted with a promotion rule** (`docs/repository-map.md:83-93`) plus the day-of-refocus commit series showing the pattern at work.
9. **Managed marker block + `--check` drift report** for propagating a small policy pointer while leaving surrounding guidance intact (`tools/sync_agentic_workflow.py:18-28`, tests `tests/test_sync_agentic_workflow.py`, 21 tests passing [X]). Already absorbed into corenius.
10. **Independent verification with a repair budget** (max three iterations, then escalate as attention-needed) (`docs/agents/agentic-engineering.md:99-106`) [R].

### 7.4 Hazards for agents

1. **Split-brain policy source.** Ultron's `agentic-engineering.md` still says "Canonical source: ultron" and Ultron's manifest, if run with `--apply`, would rewrite siblings with Ultron-worded text over corenius's (`tools/sync_agentic_workflow.py:419-500`; diff of manifests: corenius adds a `corenius` entry and `source_repository: corenius`) [X: `diff` of the two manifests]. **[I]** Do not run Ultron's sync `--apply`. Resolve the migration first.
2. **False assurance from tests.** The documented test command skips 67 of 88 tests (section 5.1). An agent that reports "tests pass" from README's command has not exercised the policy.
3. **Fixture-green is not integration-green.** Adapter tests use hand-authored fixtures (section 3.4 item 7). Treat every adapter as unvalidated against its producer.
4. **Authority creep in prose.** The 2,274-line architecture doc and 1,468-line proposal are written in a directive, first-person voice ("My proposed synthesis below is called Ultron", `docs/ultron-architecture.md:32`; "Your corrected mapping is directionally right", `docs/resilient-titan-draft-proposal.md:9`). They are research synthesis and a draft proposal ("not yet an approved implementation specification", proposal lines 3-7), not specifications. The architecture doc's own naming note says "ByteRobust++" was retired and Ultron is "not a system described in any single paper" (`ultron-architecture.md:5-8`).
5. **Stale cross-references.** `docs/workspace-repository-story.md`, the observatory's controller note, and `docs/research/managed-repo-health-2026-09-07.md` (eight repos, now nine) describe a repo that no longer exists in that form. Check the date and commit before quoting.
6. **Scope pull.** Large, well-written parked trees (`canonical-model-mastery`, `model-mlsys-closed-loop`) look like live plans. `AGENTS.md` says park them; the repo map lists them as parked (`docs/repository-map.md:73-81`).
7. **Untracked binary and secrets scanner in tree.** `.scratch/production-release-readiness/tools/gitleaks-8.30.1/{gitleaks,*.tar.gz}` is untracked; `git add -A` would commit an executable. `.scratch/production-release-readiness/estate-ledger.json` is untracked and unclassified (`docs/research/managed-repo-health-2026-09-07.md`, Ultron section, recommends classifying it) [R].
8. **Concurrent editing.** The repo changed under me three times in one read (section header). Read `git log -1` before and after; do not assume a stable base.
9. **Human-only merge and Git authority** is a hard rule: "Skill text never grants Git authority" (`docs/agents/agentic-engineering.md:127-128`) [R].
10. **Heavy dirs.** `.worktrees/` (679 MB) and `.scratch/` (119 MB, including `workstream-worklog/artifacts/ticket18-perf-*.data`) will slow a naive `grep -r`. Use `git grep`.

---

## 8. Open problems, debt, uncertainties

### 8.1 Declared open (by Ultron itself)

- Controller role: real component, research vehicle, or reduced to evidence/adapter layer (`docs/ultron-thesis.md:48-50`, "Open decisions" 1) [R].
- Whether the monarch/torchtitan/nccl/cutlass/ferric anchor set stays in the thesis or moves to history (open decision 2).
- Whether `docs/ultron-architecture.md`, still framed as a ByteRobust-anchored training OS, is kept as background or rewritten (open decision 3).
- The Torchetta/Milano/Arnold rows are untested Inferences (open decision 4).
- ResilientTitan fog: whether Phase 1 avoids TorchTitan source edits; HostGuardian transport; TorchFT contract; commit matrix for Monarch/TorchFT/TorchTitan/PyTorch/CUDA/NCCL (`.scratch/resilient-titan/map.md`, "Not yet specified") [R]. All 11 implementation tickets are `ready-for-human`; none started.
- Incident replay: written-spec approval (ticket 08) pending; sealed-oracle CI storage and production-evidence privacy/redaction/retention deferred (`.scratch/ultron-incident-replay/map.md`, Fog) [R].
- Workstream-worklog: ndtimeline 1/2/4/8-GPU ladder unfinished (`fsdp_tp_pp_torch` never launched); tickets 24-27 open/blocked; sampling tools absent in rootfs [R].

### 8.2 Debt I found (not declared by Ultron)

1. Policy implementation lags its own spec: unknown-identity conservatism, `CONTINUE`, joint identity-to-ledger check, use of `confidence` and `direct_policy_input`, unreachable `GLOBAL_STOP` (section 3.4).
2. Adapter/producer mismatch for ndtimeline (`extra_tag`, `run_id`), verified by execution on synthetic input only (3.4 item 7).
3. Test command in README skips the controller tests (5.1).
4. Two "canonical" policy sources (7.4 item 1) and a manifest naming `ultron/mainline` while the main integration point is moving to corenius.
5. `docs/workspace-repository-story.md` cites README lines that no longer exist and preserves the pre-refocus roadmap (4.3).
6. `CONTEXT.md:17-19` and `README.md:4-6` list five "followed systems" with no code, no tickets and no imported knowledge for Milano, Torchetta or Arnold; the Myriatron entry has no record in the manifest either. The scope has been asserted before the work exists [I].
7. `docs/ultron-architecture.md` (2,274 lines) and its 18 source deep-dives are unrelated to the current "scriv" framing and unlabeled per claim level; the Tarocco note says maintained claims now live in `knowledge/tarocco/` and to upgrade claims there first (`ultron-architecture.md:10-15`) [R].
8. No remote configured, so nothing is backed up off-host and no CI exists (`managed-repo-health-2026-09-07.md` Ultron section confirms "No configured GitHub remote"; `git remote -v` empty today [X]).
9. Imported knowledge snapshot came from a macOS path (`/Users/bytedance/workspace/tarocco`) at commit `accee92e...` "with pre-existing dirty knowledge changes", so it is not a clean export (`docs/tarocco-knowledge-sync.md`, Snapshot) [R]. Some referenced asset directories were absent at sync and not fabricated.
10. Worktree sprawl: 16 registered worktrees, several at stale base commits (`3dd253a`, `424ba7c`, `019422d`); `ultron/mainline` worktree is at `3dd253a`, behind `main` `5061e89` [X].
11. Bazel target is a stub; the eBPF collector itself does not exist, only schema and no-load preflight [R].

### 8.3 Uncertainties in this document

- I read Ultron's code fully for `identity.py`, `evidence.py`, `ledger.py`, `policy.py`, `incident_runner.py`; only headers, signatures and key branches for the other adapters (`stack_snapshot`, `profiler_dcgm`, `straggler_analysis`, `host_sample`). Claims about those are limited to what I cite.
- I did not read the 2,274-line architecture doc end to end (read sections 1-3, 5, headings for the rest) nor the 59 MB Tarocco packets.
- I read sibling repos only shallowly: Milano `AGENTS.md`, Torchetta/ndtl/distributed_timeline READMEs, `parser_handler.py`, `git log`. Statements about TorchTitan/NCCL/Arnold internals are quoted from Ultron or observatory notes and were not re-verified.
- "No sibling imports Ultron" was checked by targeted grep, not exhaustively.
- Whether any human or agent uses `tools.ultron_policy` outside tests is unknown.

---

## How to verify this document

Run from `/data02/home/philip.yang/workspace/ultron` (read-only commands; use `PYTHONDONTWRITEBYTECODE=1`).

1. Commit and state: `git rev-parse HEAD` (expect `5061e89...` or a descendant; if newer, diff `tools/` and `tests/` against `83eebdc`), `git status --short`, `git remote -v` (empty), `git worktree list | wc -l` (17 lines incl. main).
2. README test command runs 21 tests: `python3 -m unittest discover -s tests -p 'test*.py' 2>&1 | tail -3`.
3. Policy tests run 67: `python3 -m unittest $(ls tests/ultron_policy/test_*.py | sed 's|/|.|g; s|\.py$||') 2>&1 | tail -3`.
4. Policy ignores most evidence and flags: `sed -n 28,92p tools/ultron_policy/policy.py`; `grep -n 'direct_policy_input\|confidence' tools/ultron_policy/policy.py` (expect no matches); `grep -n unknown tools/ultron_policy/policy.py` (expect none); `grep -rn join_key tools` (definition only).
5. Unreachable `GLOBAL_STOP`: compare `ledger.py:27-33` (five states) against `policy.py:45-89`.
6. ndtimeline mismatch: read `../distributed_timeline/bytedance/ndtimeline/handlers/parser_handler.py:52-78,327,438-442` and `tools/ultron_policy/ndtimeline_adapter.py:55,219-227,256-260`; then run the adapter with `extra_tag=[]` and `run_id=7` on a minimal span (see section 3.4 item 7 for the required keyword arguments).
7. Manifest/split-brain: `diff docs/agents/agentic-workflow-repos.json ../corenius-collab-spike/docs/agents/agentic-workflow-repos.json`; `git -C ../corenius-collab-spike show 0ebc58c --stat`.
8. Ticket counts: `for d in ultron-build resilient-titan ultron-incident-replay workstream-worklog ultron-thesis; do grep -h -m1 '^Status:' .scratch/$d/issues/*.md | sort | uniq -c; done`.
9. Untracked binary: `git status --short`; `find .scratch/production-release-readiness -type f`.
10. Scope statements: `sed -n 1,30p CONTEXT.md`, `sed -n 1,25p README.md`, `sed -n 60,100p docs/repository-map.md`.
11. Stale citation check: `sed -n 8,12p docs/workspace-repository-story.md` versus `sed -n 1,20p README.md`.
12. Observatory note's stale claim: `sed -n 62,66p ../robust-training-observatory/docs/research/2026-09-29-controller-repos-source-map.md` versus current README lines 1-20.
