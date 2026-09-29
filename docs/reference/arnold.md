# Reference: arnold (Nonseed Arnold monorepo)

| Field | Value |
| --- | --- |
| Repo | `/data02/home/philip.yang/workspace/arnold` (upstream `code.byted.org/mlsys/arnold`) |
| Commit read | `756a165c40a9dc7a386641d869537733335a6748` (commit date 2026-09-23) |
| Branch | `master`; working tree clean at read time |
| Date written | 2026-09-29 |
| Mode | Read-only. No builds, no network, no git writes in arnold. |
| Size | 8,190 tracked files, 19,327 commits, 45 `go.mod` files (`git ls-files`, `git rev-list --count HEAD`). Most of `eurus-api` (2,991 of 3,071 files) is `vendor/`. |

Evidence labels used throughout:

- **[R]** read directly in this checkout. Cited as `path:line`.
- **[O]** taken from the observatory research notes in
  `/data02/home/philip.yang/workspace/robust-training-observatory/docs/research/` and re-checked against
  code where noted ("re-checked"). Not re-checked means "believed, not re-read".
- **[I]** inferred by me. Not stated by any file. Treat as a hypothesis.

All paths below are relative to the arnold repo root unless prefixed by another repo name. Line numbers
are for commit `756a165`. Arnold's own prose docs are in Chinese; quotations are paraphrased in English.

Scope honesty: arnold is enormous. I read the governance layer (`AGENTS.md`, `docs/index.yaml`, policies,
catalogs, check scripts) in depth, and read code only along the robust-training / ETTR / trace path
(Eurus Agent heartbeat, Housekeeper state machine, Megavision routers, ETTR stage math, Collector ETTR
client). I did not read Metis, GQC, Operator, Morningstar, `dlp_site`, `mlsysmonitor` or the Lending
stack beyond their catalog entries and READMEs. Claims about them are marked accordingly.

---

## 0. One-paragraph summary

Arnold is the Nonseed Arnold team's monorepo for a GPU training-job platform at ByteDance: task admission
and quota (GQC, Metis), a Kubernetes workload layer (Eurus Operator, Morningstar), an in-container agent
plus a per-cluster failover controller for "Robust Training" (Eurus Agent, Housekeeper), and an
observability/diagnosis stack (Megavision: X-Ray events/alerts, ETTR, trace generator, fault detector;
plus an info collector). It is simultaneously an unusually developed example of an "AI-native" repo:
a routed documentation system with per-domain authority, evidence and authorization policies, manifest
digests, and CI gates, designed so agents are the primary producers and consumers of docs
(`docs/README.md:3`). For corenius the second half matters most; the first half matters because it is
the product that every sibling repo (distributed_timeline, ndtl, torchtitan-side tooling) ultimately
feeds or is measured by.

---

## 1. Problem, audience, and the pain without it

### 1.1 What it solves

Arnold presents itself as "a monorepo hosting ML system implementations, automation skills, shared
packages, services, apps, and reviewable docs" (`README.md:3`). Reading the component catalog and
current-system docs, the concrete problems are:

1. **Admit and place training jobs across clusters.** GQC does global quota pre-check and cross-cluster
   dispatch; Metis is the in-cluster resource/scheduling service (`component-catalog/core-components.yaml`
   roles for `global-quota-checker`, `metis`; `global-quota-checker/README.md:3`; `metis/README.md:3`).
   Failure of either blocks users from submitting new tasks (`component-catalog/criticality.yaml`,
   rationale lines for `metis`, `global-quota-checker`). [R]
2. **Keep long GPU training runs alive through hardware and software faults.** "Robust Training" is an
   optional mode (env `ARNOLD_ROBUST_TRAINING == 1`, `docs/current-system/robust-training/glossary.md:18`)
   where an in-container Python agent heartbeats to a per-cluster Housekeeper that diagnoses, blocks bad
   nodes, and drives Failover/restart through a state machine
   (`eurus-housekeeper/pkg/services/driver.go:31-42`;
   `docs/current-system/robust-training/restart-and-failover/README.md:1-36`). [R]
3. **Measure whether training is actually making progress.** ETTR (effective training time ratio) is
   computed from stage timelines: `effective / total`
   (`megavision/ettr/pkg/model/stages.go:391-398`). The stage list distinguishes productive stages
   (`training`, `evaluation`, RL stages) from everything else (`stages.go:355-376`), and
   `MarkTrainingStage` relabels training that was not followed by a checkpoint save as `unsaved_*`
   (`stages.go:289-308`). [R]
4. **Diagnose hangs, stragglers and faults after the fact.** Megavision serves events/alerts,
   NDTimeline-based straggler queries, py-spy and NCCL Flight Recorder evidence
   (`megavision/trace_generator/pkg/router/router.go:14-23`;
   `docs/current-system/troubleshooting-and-performance-analysis/megavision-architecture.md:12`). [R]
5. **Let agents (and humans) maintain all of this without tribal knowledge.** The docs system exists so
   that current behavior, target design, declared human facts, environment objects and rollout status each
   have exactly one owner (`docs/README.md:7-9`, `docs/README.md:30-43`). [R]

### 1.2 For whom

- Primary owner/reader: the **Nonseed Arnold team**. A human-confirmed declared fact states the repo is
  their monorepo, that an unqualified "Arnold" means Nonseed Arnold, that "Seed Arnold" is a different
  product/team whose behavior must not be used to infer Nonseed behavior, and that Merlin frontend/backend
  do **not** belong to the team (`docs/declared-facts/global/arnold-system-boundary.md:29-35`,
  confirmed 2026-08-10 at line 6). [R]
- End users of the platform (model teams submitting jobs through Merlin) are consumers of behavior, not
  audience of the repo. The doc policy requires user-visible flows to be described from what the user
  sees first (`docs/current-system/README.md:46`). [R]
- Primary readers of docs are stated to be Arnold developers, and "AI is producer, maintainer and main
  consumer" (`docs/README.md:3`). [R]

### 1.3 What goes wrong without it

Arnold's docs describe the failure modes concretely enough to state them:

- Without Housekeeper/Agent failover, a fault (process exit, heartbeat loss, RDMA hang, panic log, loss
  spike) leaves the trial dead or hung; `MaxRetryThreshold == 0` even sends runtime faults straight to
  `Aborted` (`restart-and-failover/README.md:34`). Recovery is orchestration only: "restart succeeded"
  does not mean "training state restored"; loading a checkpoint is the training framework's job
  (`docs/current-system/robust-training/checkpoint-recovery-boundary.md:5-22`). [R]
- Without ETTR and stage logging, there is no shared definition of goodput. The repo shows the definition
  is fragile even with it (section 8). [R]
- Without the catalog/criticality registry, blast radius of a change is unknown: the criteria are three
  questions (does failure block new submissions, block running jobs, or break other core systems)
  (`component-catalog/criticality.yaml:21-25`). [R]
- Without the documentation contract, agents "recover" facts from target-design or memory. The docs system
  explicitly forbids falling back to `target-design/` to infer current behavior
  (`docs/current-system/README.md:44`). [R]
- [I] The pain the monorepo consolidation removes is source-of-truth drift: `eurus-agent` and
  `dlp_entrypoint` were switched to build from this monorepo, and old standalone repos "only express
  historical import origin" (`docs/declared-facts/global/arnold-system-boundary.md:38`). The repo-level
  reason for subtree import is preservation of source history
  (`component-catalog/core-components.yaml:2`, `README.md:8`). The motivation beyond that is my reading.

---

## 2. Non-goals, boundaries, authority, dependents

### 2.1 What it deliberately does not do or own

- **Seed Arnold** is out of scope. Shared Merlin/Training Console/RH2 code may contain both Seed and
  Nonseed branches; one must first prove a request enters Nonseed before treating that code as evidence
  (`arnold-system-boundary.md:32`). [R]
- **Merlin frontend/backend are upstream dependencies, not Arnold code**, even when synced under
  `external/` (`arnold-system-boundary.md:33`). Citing them is "boundary evidence" only (`:36`). [R]
- **External dependency source is not committed.** `external/` is a read-only local workspace,
  git-ignored (`.gitignore`: `/external/*`), synchronized from branch tips by
  `scripts/sync_external_components.py`, explicitly not pinned to a long-term commit (`README.md:56-57`).
  Revisions that matter are recorded in manifests (`docs/current-system/robust-training/manifest.yaml`
  `external_revisions`, lines 7-11). [R]
- **Robust Training is not a generic checkpoint manager** (`checkpoint-recovery-boundary.md:5`). Only
  Cruise-style protocols get recovery-step parsing and command rewriting (`:8`). [R]
- **Megavision's fault detector reports events; it does not trigger automatic action**
  (`component-catalog/criticality.yaml`, `megavision` rationale). And the Event to Alert path is not
  connected to Situation/Karl actions or Failover: `StartSituationService` is defined
  (`megavision/pkg/service/situation_service.go:64`) but no caller exists in `megavision/`
  (`grep StartSituationService`), and Alert generation does not write `SituationChan`
  (`docs/current-system/troubleshooting-and-performance-analysis/megavision-event-alert-pipeline.md:12`). [R]
- **Housekeeper "healthy" is not "hardware healthy".** Runners mostly issue shell commands and parse logs;
  Pass/Fail/Error/Skip handling is not uniform; PreCheck status is mostly the count of banned nodes
  (`docs/target-design/housekeeper/README.md:22`). [R]
- **Docs are not runtime truth.** Merge does not prove production deployment; dynamic configs, online
  versions and outcomes need runtime evidence (`docs/README.md:45`; `AGENTS.md:11`). [R]
- **Lark mirrors** are a documented sync target but most manifests list the Lark target as `TBD`
  (`docs/current-system/lark-sync.md:6-9` and later entries). [R]

### 2.2 What it treats as authority

The authority model is the repo's central design (see section 3.4). In short:

| Question | Authority |
| --- | --- |
| Current reachable behavior | `docs/current-system/<domain>/` synthesizing code + declared facts + runtime evidence (`docs/current-system/README.md:3-12`) |
| Facts that code cannot derive (team ownership, migration completeness, product scope) | `docs/declared-facts/` with confirmer, scope, exclusions and invalidation conditions (`arnold-system-boundary.md:1-23`) |
| Target design | `docs/target-design/`, non-authoritative for current state (`docs/index.yaml:14-17`) |
| Fixed-revision behavior of external code | `docs/external-implementation-evidence/` (`docs/index.yaml:18-21`) |
| Component identity and code paths | `component-catalog/*.yaml` is "the single inventory source" (`component-catalog/README.md:31`) |
| Machine routing | `docs/index.yaml` is "the only machine index" (`AGENTS.md:17`) |
| Governance | `AGENTS.md` is the single governance source; `CLAUDE.md` only imports it (`CLAUDE.md:3`, `docs/target-design/agent/harness-architecture.md`, "Harness constitution" table) |
| Facts owned by others (platform, other teams, prod) | External owner, with revision/time recorded (`docs/agent-instructions/core/policies/evidence.md:20`) |

Notably, adapters (bytedcli, MCP, APIs) are **not** authority: "automation level is not evidence grade"
(`evidence.md:22`). [R]

### 2.3 What depends on it

- **Inside the repo:** `pkg/` (shared Go packages) is consumed by Metis and GQC; `eurus-api` (CRD schema)
  by operator/housekeeper/metis; `eurus-connector` (event routing client) by housekeeper/morningstar
  (rationales in `component-catalog/criticality.yaml`, `pkg`, `eurus-api`, `eurus-connector`). [R]
- **Outside the repo (in this workspace):**
  - `robust-training-observatory` cites arnold at a pinned commit; the pin is exactly the commit read here
    (`.../2026-09-28-arnold-findings-and-downstream-registration.md:3`). Arnold carries no trace of it
    (that note, lines 23-28). [O]
  - `distributed_timeline`'s data ends up in the ClickHouse table arnold's trace generator queries (see 6.1). [O + R]
- [I] Model teams' training jobs depend on Arnold at runtime, via `dlp_entrypoint` and Eurus Agent
  launching the user process; this is the platform's whole purpose, but I did not trace it end to end.

---

## 3. Architecture

### 3.1 Layout (what the top-level directories are)

Source: `README.md:26-47`, `component-catalog/core-components.yaml`, `ls`.

| Group | Dirs | Role (from catalog `role:` fields unless noted) |
| --- | --- | --- |
| Admission / scheduling | `global-quota-checker`, `metis`, `pkg`, `config` | GQC: global quota checker; Metis: resource and scheduling service; `pkg`: shared Go packages (bytequota, model, quota, util...) |
| Workload / control plane | `eurus-operator`, `eurus-morningstar`, `eurus-housekeeper`, `eurus-api`, `eurus-connector` | Operator: K8s controller reconciling Trial CRD; Morningstar: global dispatcher; Housekeeper: per-cluster driver; api: CRD schema; connector: event routing client (Merlin/Lark/MegaVision/Karl/BMQ) |
| In-container | `eurus-agent` (Python), `dlp_entrypoint` (shell/Python), `hdfs-fuse-scripts`, `hdfs_config`, `training_duration_metrics` (pip SDK) | Agent: heartbeat, role management, user process launch; Arnold Entrypoint: container start; SDK: ETTR stage logging |
| Observability / diagnosis | `megavision` (5 binaries), `xray-perf-backend` (Django), `arnold_info_collector`, `mlsysmonitor` (several independent monitors) | see 3.3 |
| Ops | `arnold-ops-api`, `arnold-ops-cron`, `arnold-uce-credits-auditor` | ops HTTP API, cron host, lending credits auditor |
| Legacy / deprecated | `dlp_site` (Django), `deprecated/arnold-uce-*`, `metis/deprecated/` | isolated under `legacy_components` (`criticality.yaml:165-190`) |
| Governance | `docs/`, `.agents/skills/`, `scripts/`, `.codebase/` | see sections 5 and 7 |

Criticality (from the 3-question rule): core = metis, GQC, operator, housekeeper, morningstar, agent,
entrypoint, hdfs-fuse-scripts, hdfs_config, eurus-api, eurus-connector, pkg, arnold-ops-cron.
Supporting = ops-api, info_collector, megavision, xray-perf-backend, mlsysmonitor, training_duration_metrics
(`component-catalog/criticality.yaml`, `components:` block). Megavision being *supporting* is the key fact:
observability failure must never block or corrupt jobs. [R]

### 3.2 End-to-end job path (the spine)

```
 user -> Merlin (Training Console)               [NOT arnold; upstream]
            |  TaskGroup (ARNOLD_TASK_TYPE)
            v
        GQC (global quota pre-check, cross-cluster dispatch)
            v
        local Metis (Queue/quota admission, workload projection)
            v  creates Eurus Trial (CRD)
        Eurus Operator (K8s reconcile Trial -> pods)
            |                        ^ Morningstar: creates Runtime, writes terminal
            |                        |  state back to Trial, projects blockedNodes
            v                        |
   pod: dlp_entrypoint -> eurus-agent -- heartbeat --> Housekeeper (Runtime state, per cluster)
                            |   ^  response: role cmds, py-spy/BGCP/sexec/NDTimeline-dump triggers
                            |   |
                     user training process (rank 0 logs ETTR stages via training_duration_metrics)
```

Evidence: state ownership table `docs/current-system/scheduling/training-run-state/training-run-execution-and-state-flow.md`
("Model boundary" table: `job_run`, `job_run_trial` owned by Merlin; `task_group` by Metis; `Trial` by
Operator/Morningstar; `Runtime` by Housekeeper). Key statement: these are **four different state machines,
not one with different names**, and transient disagreement is legal (same doc, "Key conclusion" paragraph).
Agent ping: `eurus-agent/eurusagent/agent.py:231-262` posts node info to
`{driver}/api/v2/trials/{trial_id}/ping`; Housekeeper route `eurus-housekeeper/pkg/server/api_server.go:155-157`.
Housekeeper control API (block, overhaul, reset, py-spy, sexec, ndtimeline-dump, copilot profile, bgcp
trace, n-divide, file-monitor): `api_server.go:107-133`. [R]

### 3.3 Robust-training / observability data flow

```
 training process rank0 --stage log--> /var/log/tiger/arnold_ettr.log   (training_duration_metrics SDK)
 dlp_entrypoint (queue/schedule/pull_image/pod_start stages) --same logger--> same log     [R]
      log agent --> ClickHouse megavision_ettr_log            (hop is OUTSIDE the monorepo)  [O]
                         |
                         v
  ettr-service (megavision/ettr)  GET /api/v1/trial_stages, /trend, POST /ettr_list         [R]
  X-Ray (megavision/cmd/main)     /api/v1/ettr/*  (rebuilt from xray_event alert names)     [R]
                         ^
  Collector ETTR client polls /ettr/api/v1/trial_stages (arnold_info_collector)             [R]

 Agent UDC collectors --heartbeat--> Housekeeper analyzers (log analyze, RDMA/TensorCore hang,
                                     loss spike, heartbeat loss) --> Failover / block / Event
 Housekeeper/Agent --eurus-connector--> X-Ray POST /api/v1/event
      X-Ray: validate -> BRIDGE topic -> leader consume -> EventChan (8 workers) ->
             owner fill (host->pods) -> Alert (1 serial loop) -> RDS (Alert) + Analysis Topic
             Analysis Topic -> [outside monorepo consumer] -> ClickHouse xray_event
 Trace generator (megavision/trace_generator) POST /export-nd-timeline, /coll-comm-straggler...
      -> ClickHouse table megatron_cuda_timer_tracing_original (written by NDTimeline producers)
```

Evidence:
- ETTR SDK log path and rank-0-only: `training_duration_metrics/README.md` (lines "Training stage log file path" and "only RANK=0").
- Entrypoint stages: `dlp_entrypoint/exec/trace_training_duration.py:281-307`.
- ETTR router: `megavision/ettr/pkg/router/router.go:17-30`; X-Ray router: `megavision/pkg/router/router.go:20-58`.
- Collector client: `arnold_info_collector/pkg/collector/ettr/ettr.go:17-26`
  (`ettrTrialStagesPath = "/ettr/api/v1/trial_stages"`, 20 s timeout, 2 retries, 8 concurrent).
- Event to Alert flow, defaults, sliding window and Analysis Topic:
  `docs/current-system/troubleshooting-and-performance-analysis/megavision-event-alert-pipeline.md:21-90`
  with code anchors `megavision/pkg/router/v1/arnold/event.go:24-79`, `megavision/pkg/service/bridge.go:40-128`,
  `megavision/pkg/service/arnold/event_service.go:18-93`, `megavision/pkg/model/base/alert_rule.go:114-139`.
- Trace generator routes: `megavision/trace_generator/pkg/router/router.go:14-23`; ClickHouse tables in
  `megavision/trace_generator/pkg/sql/templates/ndtimeline.sql:9-12`, `:47-49` (CN
  `lab_vc_sre_lq.megatron_cuda_timer_tracing_original`; US table name per [O]); DP duration
  `max(start_t + duration_t) - min(start_t)` grouped by `dp_rank` at `ndtimeline.sql:158-165`; query batch cap
  512 workloads and 15 s sub-timeout at `megavision/trace_generator/pkg/chmodel/ndtimeline.go:187-188`.
- Five Megavision binaries from one module: `megavision/build.sh` (xray, fault-detect, py-spy-processor,
  trace-generator, ettr-service). The doc warns not to infer repo count from binary count
  (`megavision-architecture.md:28`).

Megavision as a whole is a front end (`mlsys/ml-troubleshooter-fe`, external), a BFF
(`reckon/mlsys_bff`, external), and Go/Python backends; the browser calls BFF, X-Ray, perf backend and
specialty services directly and the BFF is not a universal gateway (`megavision-architecture.md:12`).
Routing evidence was captured 2026-09-14 19:25-20:10 +08:00 and "does not prove the online artifact equals
this source" (`megavision-architecture.md:18`). [R]

### 3.4 Key abstractions and invariants

**Runtime / control-plane abstractions**

| Abstraction | Invariant | Evidence |
| --- | --- | --- |
| Eurus Trial (K8s CRD) | Workload carrier. Robust Training requires a Trial; a Trial may run without Robust Training (one-way implication, not equivalence). Never infer `ARNOLD_ROBUST_TRAINING` from "it is a Trial" or from `ROBUST_EXECUTOR_CONTROL_PLANE=eurus`. | `docs/current-system/robust-training/glossary.md:7-11`, `:17-20` |
| Runtime (CRD) | State written only by Housekeeper Driver; Morningstar creates/updates metadata and writes terminal state back to Trial but not Runtime state. | training-run-state doc, "Model boundary" table |
| Housekeeper Trial state machine | Fixed transition map `Init->{Spawn,Init}`, `Spawn->{Run,Failover}`, `Run->{Run,Failover}`, `Failover->Check`, `Check->Check`, terminal states absorb. | `eurus-housekeeper/pkg/services/driver.go:31-42` |
| `SetRestartWithLevel` | Only accepts Init/Spawn/Run; sets `NeedRestart`, keeps the *max* diagnostic level over concurrent triggers, last-writer wins on `FailoverType`; returns false and only warns in other states. 31 call sites under `pkg/`. | `eurus-housekeeper/pkg/services/runtime_info.go:395-415` [R]; count via grep |
| Housekeeper deployment | One active instance per `SHARD`; there is **no effective leader election**. | `docs/target-design/housekeeper/README.md:24` |
| Agent heartbeat | Agent aggregates role-manager and UDC (collector) reports, pings Housekeeper; response carries commands and per-feature configs (py-spy, BGCP, NDTimeline dump, sexec, loss-spike config, Ray head info). | `eurus-agent/eurusagent/agent.py:105-125`, `:231-262`; `eurus-agent/eurusagent/model/heartbeat_response.py:60-130` |
| Collector registration is gated | Registration depends on lite/non-lite, OOM flag, Ray, NVIDIA/Ascend, Python < 3.12 for file monitors. [O] (`NodeMetricCollector` never registered: re-checked, class appears only in its own file and its test: `eurus-agent/eurusagent/unified_data_collector/collectors/node_metric_collector.py:8`, `eurus-agent/tests/.../test_node_metric_collector.py:4-8`, no import in `unified_data_collector.py`.) | see 3.5 |

**Observability abstractions**

| Abstraction | Invariant / behavior | Evidence |
| --- | --- | --- |
| Stage (ETTR) | `{trial_id, stage_name, start_ts, end_ts, duration, priority, ...}`. Overlaps are resolved by highest priority. | `megavision/ettr/pkg/model/stages.go:24-52`; `Merge` at `:95-137` |
| ETTR | `effective / total`, where total is the sum of stage durations for per-trial ETTR but the **window length** for range/trend ETTR. | `stages.go:378-398`; `megavision/ettr/pkg/model/trial_stages.go:390-392` |
| Effective stage set | training, evaluation, and 11 RL stage names; must match the frontend's ETTR logic (comment cites the front-end file). | `stages.go:355-376` |
| Unsaved training | Training stages not followed by `save_checkpoint[_async]` before a restart marker are renamed `unsaved_*`, hence excluded from effective time. | `stages.go:289-308` |
| Alert | Per-trial dedup view of Events, not a threshold trigger. First event creates an Alert immediately; merge iff same trial, same category+name, `abs(new.ts-(old.ts+old.duration)) <= dedup_duration` (default 10 min, sliding) and every `separate_keys` expression equal. Default rule has no `separate_keys`. | event-alert-pipeline doc `:78-90`; `megavision/pkg/model/base/alert_rule.go:114-139` |
| Event | Events from `driver-0` are dropped; host-only events fan out to every running pod on the host; host with no running pod gets `trial_id=NO_RUNNING_POD` and goes only to the Analysis Topic. | event-alert-pipeline doc `:39-50` |
| NdCompleteEvent | Go projection over the NDTimeline ClickHouse table (rank, dp_rank, tp_rank, stage, model_chunk, step, metric, start_ts, duration, tensor/sequence/PG fields, extra_tag). | `megavision/trace_generator/pkg/chmodel/ndtimeline.go:41-56` [R+O] |

### 3.5 Agent-side collection (what actually reaches the platform)

From the observatory audit [O], with the two checks I repeated marked:

- GPU device metrics do not come from the Agent; they come from `mlsysmonitor/src/mlsysmonitorCloudNative`
  (30 s, host metrics agent, Kafka to ClickHouse `arnold_monitor`). [O, not re-checked]
- Agent-side BMQ message families actually emitted: PY_SPY, MEMORY_USAGE, RAY_ACTOR, ISOLATED_PRECHECK, and
  Flight Recorder / Ray job publishers; `ROLLOUT_STATE` publishing is commented out; routes
  `robust_training_3dtimeline` and `robust_runtime_info` are configured but no emitter found. [O]
  Re-check: those two route names appear in only four non-vendor files
  (`xray-perf-backend/src/utils/oneservice/client.py`, `xray-perf-backend/src/topology/views.py`,
  `eurus-connector/pkg/client/bmq/message_type.go`, `eurus-agent/eurusagent/common/bmq_publisher.py`).
  I did not open them to confirm "no emitter".
- Housekeeper to X-Ray event families go through `eurus-connector`, not the Agent
  (`eurus-connector/pkg/client/megavision/client.go:23-45`: buffered channel 10,000, 20 workers,
  5 s HTTP timeout, `MaxRetryCount: 1` marked `TODO`). [R]

---

## 4. Key design decisions, trade-offs, known failures

### 4.1 Decisions documented in the repo

| Decision | Rationale and alternative | Evidence |
| --- | --- | --- |
| Monorepo via `git subtree add`, preserving history; per-component source branch/commit and explicit `adaptations` / `excluded_paths` | Keeps reviewability and provenance; private `seed.key` files are excluded from imports (3 entries at `core-components.yaml:101-102`, `:130-131`, plus header text `component-catalog/README.md:16-22`) | `component-catalog/README.md:5-22` |
| External deps as a synced, uncommitted view | Avoids vendoring other teams' code; cost is no long-term pin, so manifests record `external_revisions` | `README.md:56-57`, `docs/current-system/robust-training/manifest.yaml:7-11` |
| One fact, one authority owner; separate layers for current / target / declared / inventory / rollout | Prevents drift and "which doc is true"; target design may duplicate a contract for reviewability but never gets current-state authority | `docs/README.md:7-9`, `:30-45` |
| Human input only enters as narrowly scoped Declared Facts with confirmer, scope, exclusions, invalidation; conflicts with code are preserved, not overridden | Humans are authority for what code cannot derive, but cannot silently override code | `evidence.md:67-88`; example `arnold-system-boundary.md:1-23` |
| Five-state context sufficiency: `sufficient`, `runtime-evidence-required`, `declared-facts-required`, `conflicting-evidence`, `unknown` | Forces agents to stop rather than extrapolate; "usually true" is banned | `evidence.md:26-51` |
| Structured reach-out payload: decision, scope, evidence_checked, missing_fact, alternatives, impact, requested_owner | Do not push retrieval work back to humans | `evidence.md:53-65` |
| Authorization by intent, not grammar ("can you fix it?" is a fix request; explain/review/status is read-only) | Removes ambiguity about what an agent may do | `AGENTS.md:7`; `docs/agent-instructions/core/policies/authorization.md` bullet 1 |
| Session capabilities only if time scope, action class and no-confirm semantics are all stated; excludes stop/cancel/rollback/delete/approve/bypass | Lets a human grant bounded autonomy without permanent grants | `authorization.md:24-32` |
| Design depth proportional to risk: full seven-section Target Design vs short note vs none; design ships in the same MR as implementation | Avoids both undesigned cross-component changes and ceremony for typos | `docs/agent-instructions/core/policies/design-first.md:18-29`, `:43` |
| Policies are "machine-enforced" only if a script blocks bad candidates and a failing-path test exists; everything else is a manual gate | Honest labelling of what is checked | `docs/agent-instructions/core/policies/policy-evolution.md:19` |
| New mandatory rules need a verified root cause and a checkable recurrence; do not add a global rule for one failure or model habit | Controls governance growth | `policy-evolution.md` ("Select the minimal correction") |
| `AGENTS.md` is a small bootloader; policies are loaded by task route from `docs/index.yaml`; simple explanation and typo tasks skip doc preloading | Human attention and token budget over completeness | `AGENTS.md:13-17`; `docs/target-design/agent/harness-architecture.md` ("Goals and invariants") |
| Two hosts, one governance source: Codex reads `AGENTS.md` natively; Claude Code imports it via `@AGENTS.md` in `CLAUDE.md`, no duplicated rules | Avoids two divergent rule sets | `CLAUDE.md:1-7` |
| Housekeeper: one active instance per shard, no leader election | Simplicity; cost is an operational invariant that must be maintained | `docs/target-design/housekeeper/README.md:24` |
| Honest scoping of Robust Training: orchestration, not checkpoint management | Prevents overclaiming to users | `checkpoint-recovery-boundary.md:5-22` |

No dedicated rejected-alternatives log was found; Target Designs carry a `## Key decisions` section with alternatives and costs (`design-first.md:29-38`), of which I read only a handful. Grafana dashboards are deliberately kept out of `docs/` as deployable assets (`docs/README.md:52`). [R]

### 4.2 Known failures, defects and lessons (each cited)

1. **Credential exposure via an agent tool output.** On 2026-08-24 a local diagnostic pipeline printed raw
   RDS payloads (containing historical task env vars and credential fields) into an agent tool output;
   treated as potential leak, rotation pending, status "Open; owner action required" as of 2026-08-28
   (`docs/issue/security/codebase-12-diagnostic-credential-exposure.md:1-25`). Lesson: agent
   diagnostics need output redaction by default; tie to `production-safety.md:25` ("keep only necessary
   summaries; full results in a temp dir") and `authorization.md:15` ("never write tokens... to repo, docs
   or stable logs"). [R]
2. **Two non-comparable ETTR definitions.** X-Ray `/ettr/trend` rebuilds stages from `xray_event` alert
   names (`megavision/pkg/model/train_stage.go:206-230` `isSaved`; `global_ettr.go:88`
   `ComputeGlobalETTRMetrics`); ettr-service reads `megavision_ettr_log` (`megavision/ettr/pkg/model/stages.go`).
   Effective sets and unsaved rules are separate implementations. [O, function locations re-checked]
3. **Effective-stage list exists in three places** (`GetEffectiveStages`, `MarkTrainingStage`, and the
   frontend, per the code comment at `stages.go:356`). Adding a stage in one place silently skews ETTR. [R for two Go copies; frontend per comment]
4. **ETTR denominator differs by API.** Per-trial ETTR divides by the sum of stage durations
   (`stages.go:391-398`); range ETTR divides by window length because "total is the sum of stages and may be
   less than the real range" (`trial_stages.go:390-392`). A trend number is a different measurement. [R]
5. **Priority tie-break uses `>=`**: same-priority overlapping stages (`robust` vs `training`, both priority 1
   per [O]) resolve by event order (`stages.go:124`). [R for `>=`; priorities per O]
6. **Embedded API tokens in shipped scripts.** `dlp_entrypoint/exec/trace_training_duration.py:39-41` holds
   two hard-coded auth tokens (CN and non-CN) selected by URL substring. Values not reproduced here. [R]
   [O] adds that it assumes `driver-0` / `executor-0` pod names so multi-role or Ray trials may lose
   queue stages.
7. **Spelling frozen as contract.** Metric name `training/avg_effetive_len` (sic) at
   `eurus-housekeeper/pkg/utils/training_metrics_check.go:26`; documented as deliberate [O].
8. **Docs gate red on clean master.** [O] reports `check_docs_structure.py` needs PyYAML and fails on
   clean master with a manifest digest drift for `troubleshooting-and-performance-analysis`; Arnold policy
   forbids digest-only updates. Not re-run by me (I ran nothing).
9. **Reachability baseline is noisy by design.** First reachability baseline (commit `de14038`, 2026-08-18)
   found 901 unreachable functions in the 17 of 25 modules that analyzed successfully, with a
   prominent "high false positive rate, requires human confirmation before deletion" banner
   (`docs/ad-hoc/2026-08-18-arnold-reachability-baseline/README.md:1-6, 24-30`). [R]
10. **Alert does not reach automated action.** Documented gap (see 2.1); rule configuration for Situation
    actions does not prove any action fires (`megavision-event-alert-pipeline.md:12`). [R]
11. **Analysis Topic to ClickHouse consumer and log-agent to `megavision_ettr_log` hop are outside the
    monorepo**, so only names are provable (`megavision-event-alert-pipeline.md:18`;
    [O] findings note). [R/O]
12. **Issue registry is dominated by GQC/Metis defects** (17 numbered Codebase issues, most "Open" or
    "Resolved in code; production not re-verified"): HTTP status semantics, truncated LQ TaskGroups,
    readiness 502, ghost quota, Tide schema contract (`docs/issue/README.md`, table rows #1-#17). [R]

---

## 5. Build, test, release, verification (trust and evidence model)

### 5.1 Build and release (as documented; not executed)

- Each service has its own `build.sh` and `go.mod` (45 `go.mod`); SCM builds produce `output/`;
  TCE deploys. E.g. Housekeeper: "SCM `567257` builds `eurus-housekeeper/build.sh` from `mlsys/arnold:master`;
  `repo_info` update is replace semantics, so the full online dependency list must be preserved"
  (`docs/environment-inventory/eurus-housekeeper/README.md`, "Service and artifact boundary" section;
  `docs/agent-instructions/operations/policies/production-safety.md:17`). [R]
- `megavision/build.sh` runs `go test ./...` (with `-tags router` on `stage-env`) then builds five
  binaries and copies their `run*.sh` (`megavision/build.sh:1-50`). [R]
- `dlp_entrypoint` / `eurus-agent` are built from the monorepo as the source of truth; this does **not**
  mean the two artifacts release atomically (`arnold-system-boundary.md:38-39`). [R]
- Release production-safety rules: confirm branch and full commit pushed before SCM build; stop on ByteSIB
  block; only execute platform-returned `allow_actions`; build/deploy success does not prove effect, read
  back online version and exercise a real business path (`production-safety.md:12-19`). [R]

### 5.2 Test and CI

- CI pipeline defined in `.codebase/pipelines/docs.yaml:1-24`: installs PyYAML 6.0.2, byte-compiles the docs
  checkers, runs `check_docs_structure.py --self-test`, `check_docs_structure_delta.py --self-test`,
  `test_check_review_attention.py`, then the delta check against the MR target branch (or the full check
  if none), then `check_component_docs.py --self-test`. Timeout 10, retry 1. [R]
  Note this pipeline validates **docs structure**; I found no repo-wide pipeline for Go/Python tests in
  `.codebase/pipelines/` (only `docs.yaml`). Tests run in per-component `build.sh` (as in Megavision) and
  in separate CI checks not visible here. [I]
- `scripts/check_docs_structure.py` is 3,767 lines and validates index, manifests, path digests, external
  revisions, Markdown reachability, duplicate prose, question-first openings, dashboard registry,
  environment inventory schema and CODEOWNERS
  (function list `scripts/check_docs_structure.py:155-3310`). Legacy non-conforming design docs are
  quarantined in a "only shrink, never grow" `docs/quality-baseline.yaml` (`design-first.md:43`;
  `quality-baseline.yaml:1-3`). [R]
- `scripts/check_component_docs.py` enforces that changes to a manifest's `primary_code_paths` /
  `evidence_paths` receive doc review (`scripts/check_component_docs.py:1-2, 16-24`). [R]
- `scripts/gen_codeowners.py` generates `.codebase/CODEOWNERS` from `criticality.yaml`
  (`component-catalog/README.md:10-15`); review and merge requirements come from Codebase branch rules,
  not from criticality (`criticality.yaml:5-6`, header). [R]
- Dashboards: `check_arnold_dashboards.py`, `check_eurus_dashboards.py`, `check_gqc_metis_dashboards.py`
  plus `generate_arnold_dashboards.py`, i.e. Grafana JSON is generated and validated as code. [R, names only]
- **Harness compatibility tests** run real agent CLIs (Codex, Claude Code) on three narrow prompt cases
  (`bootstrap`, `global-route`, `change-route`) and compare JSON answers to expected values
  (`scripts/run_agent_harness_compat.py:20-56`). E.g. the bootstrap case must return
  `{"entry":"AGENTS.md","index":"docs/index.yaml"}`; "user asks to fix parsing error" must select
  `fix_and_verify`; "explain the error" must be `answer_only`; "cross-component auth protocol" must select a
  `full` design; "create fix MR" must **not** authorize merging protected branches. This is an unusual and
  reusable idea: unit tests for agent behavior under the governance layer. Explicitly opt-in, not in CI
  (docstring: "Explicitly run..."). [R]
- **Philosophy experiments** (`scripts/run_philosophy_experiment.py`, 1,133 lines; `docs/philosophy/*`) record
  model-perspective answers per harness fingerprint and model version to observe harness influence; they are
  explicitly non-authoritative and default-unrouted (`docs/README.md:47`;
  `docs/philosophy/ai-native-monorepo/README.md` "Interpretation boundary"). [R]

### 5.3 Trust and evidence model

Four-dimension evidence contract (`evidence.md:9-18`): Authority, Scope, Revision/Time, Verification. Missing
any dimension means the evidence may be used only within its known range, never extrapolated. Applied
patterns:

- **Time-bounded snapshots.** Dynamic facts such as CN AlertRule counts (1,258 records, 2026-07-22 14:57:01
  +08:00) and LogAnalyze rules (143 rule lines from 6 TCC online versions, 2026-07-22 16:31:04 +08:00) are
  stored as snapshots with environment and query time, and are explicitly "not a replacement for runtime RDS
  and TCC" (`docs/current-system/troubleshooting-and-performance-analysis/README.md:13-14, 19-20`). [R]
- **Manifest digests.** Each current-system domain has `manifest.yaml` with `monorepo_paths_digest`,
  `external_revisions`, `reviewed_at`, `primary_code_paths`, `evidence_paths`
  (`docs/current-system/robust-training/manifest.yaml:1-50`). Digest drift forces re-review even if
  conclusions do not change; you must then advance digest and `reviewed_at` (`docs/current-system/README.md:45`). [R]
- **Three separate claims** that must never be merged: code passes verification; deployment reached steady
  state; online behavior took effect (`AGENTS.md:11`). [R]
- **Readback.** Every external write requires readback of target, environment, region, object and online
  version before and after (`production-safety.md:7-10`); empty query results do not prove absence, verify
  data existence and query-path health separately (`:27`). [R]
- **No signing.** [I] I found no signature or attestation mechanism; trust is by review, owner routing,
  manifest digest, and CI structure checks. Contrast with milano (section 6).

---

## 6. Interfaces with other repos in the workspace

**Headline: arnold contains no textual reference to milano, torchetta, ultron, ndtl, vaso, torchtitan or
`distributed_timeline`.** `git grep -il` for each name, excluding `external/`, returns no hits, except
"ndtimeline" and one unrelated `vaso` match inside vendored Go HTML code
(`eurus-api/vendor/golang.org/x/net/html/doctype.go`). All linkage below is therefore either data-level
(shared table/API/message names) or asserted by the *sibling* repos' notes. [R for the grep]

| Repo | Relationship to arnold | Direction | Evidence and status |
| --- | --- | --- | --- |
| **distributed_timeline** (`bytedance.ndtimeline` package) | Producer of NDTimeline data (timers, tags, streamer) that lands in ClickHouse `megatron_cuda_timer_tracing_original`; arnold's trace generator is a read-only consumer. Arnold also patches Flight Recorder integration: `dlp_entrypoint/tools/flight_recorder_patch.py:6` imports `bytedance.ndtimeline.EmergencyServer`; `dlp_entrypoint/init_env/set_torch.sh:19-20` sets `NDTIMELINE_ENABLE_FLIGHT_RECORDER_CPP_SERVER=1` by default. Housekeeper can trigger `ndtimeline-dump` (`api_server.go:121`) and the Agent receives `NDTimelineResponse` (`heartbeat_response.py:90-95`). | producer -> (CH) -> arnold reads; arnold -> producer only via env var and dump trigger | [R] for arnold side; producer side [O] (`.../associated-repos-source-map.md:29-33`). Contract is the ClickHouse column set (`DefaultCols`, `chmodel/ndtimeline.go:52`) and MetricPayload semantics owned by the producer. |
| **ndtl** | Experimental successor vocabulary (JSONL manifests, heartbeats, step progress, collector health). No arnold code uses it. | none (yet) | [O] only, `associated-repos-source-map.md:39-47`. [I] It is a candidate to replace arnold's liveness inference (heartbeat loss, hang analyzers) with explicit progress records; nothing in arnold indicates that plan. |
| **torchtitan** | Training loop that would emit the timers/ETTR stages arnold consumes. Arnold's ETTR SDK is `training_duration_metrics` (`bytedance.trainingmetrics`), a pip package with rank-0 stage logging. No arnold code names torchtitan. Effective-stage names (`training`, `evaluation`, RL stages) are the shared vocabulary any loop must emit for ETTR to be meaningful (`stages.go:355-376`). | loop -> log -> arnold | [R] vocabulary; [I] the torchtitan integration seam is a *proposal* from the observatory notes, not something arnold defines. |
| **milano** | Independent CUDA-backed `torch_accel` runtime/build/release harness. No dependency either way. Relevance is methodological: milano's content-addressed, append-only evidence and single-entry pipeline are the strong-trust counterpart to arnold's review-and-digest model. | none | [O] classification in `.../associated-repos-source-map.md` / CONTEXT; arnold side [R] (grep). Comparison in 7.2 is [I]. |
| **torchetta** | Custom libtorch/rootfs/release evidence. No arnold link. | none | [O] (observatory `CONTEXT.md` "Associated source"). |
| **ultron** | Internal controller/knowledge/policy spine for robust-training observability, per the observatory's recommendation. No arnold link. Conceptually overlaps arnold's own docs authority system. | none | [O] `.../2026-09-29-controller-repos-source-map.md:12-30`. [I] Overlap: arnold's `docs/index.yaml` + policies are already a working controller/knowledge spine *inside one repo*; ultron would be the cross-repo one. |
| **vaso** | Not found in arnold. I have no evidence of any relationship. | unknown | [R] grep negative; not investigated in any note I read. |
| **robust-training-observatory** (not in the requested list, but the only actual consumer) | Read-only downstream visualization that pins arnold at `756a165` with line-level anchors; a downstream pointer commit was deliberately withdrawn from arnold, so arnold maintainers have no in-repo notice of it. | arnold -> observatory | [O] `.../2026-09-28-arnold-findings-and-downstream-registration.md:3-28`. |

Shared vocabulary that would have to be preserved across repos (all [R] in arnold):
`Trial`, `Runtime`, `Housekeeper`, `Morningstar`, stage names (`training`, `save_checkpoint`,
`queue_on_arnold`, `dlp_entrypoint`, `unsaved_*`), `NdCompleteEvent` fields (`rank`, `dp_rank`, `tp_rank`,
`step`, `metric`, `extra_tag`), `separate_keys`/`dedup_duration` for alerts, `trial_id`/`worker`/`host`
event ownership, and the Seed vs Nonseed qualifier.

Who-calls-whom summary:
- Agent -> Housekeeper (HTTP v2 ping). Housekeeper -> Agent only in the heartbeat response.
- Housekeeper, Morningstar -> Megavision X-Ray via `eurus-connector` (HTTP + BMQ).
- Collector -> ETTR service (HTTP polling). Browser/BFF -> X-Ray, ETTR, perf backend, trace generator.
- Megavision BFF -> Morningstar (URL from a Metis TCC key `cluster_to_morningstar_url`) -> `sexec`,
  `file_monitor`, `megavision/profile` (`megavision-architecture.md:83`). [R]
- Nothing in arnold calls any other workspace repo.

---

## 7. Relevance to agentic engineering

### 7.1 How agents work in it today

- **Entry.** `AGENTS.md` (35 lines) tells agents: respond in Simplified Chinese (line 1); understand goal
  and evidence first, verify problem premises, propose lower-cost paths (`:5-6`); judge authorization from
  intent (`:7`); complete implementation, affected verification and self-introduced fixes in one go
  (`:8`); preserve user changes, no branch switching or index edits in a shared working copy, use a
  separate worktree for modifying tasks (`:9`); never print credentials or query data the user said they
  cannot access (`:10`); report code-verified vs deployed vs live separately (`:11`). [R]
- **Routing.** `docs/index.yaml` (681 lines) defines `task_routes` (read-only-analysis, documentation-change,
  code-change, mr-review, operations, `docs/index.yaml:64-116`) and `conditional_instructions` triggered by
  semantics rather than path (scm-tce-release, text-delivery, design-preparation, mr-preparation,
  policy-maintenance, `:117-146`). `scripts/route_code_change_policies.py --task code-change --show-route`
  prints the route without dumping the index (`AGENTS.md:19-21`); with changed-file hints it suggests
  extra policies, but "path hints do not replace semantic judgement" (`change.md:10`). [R]
- **Domains.** Agent instructions are organized by task domain (core, documentation, code-change,
  operations), not as a global policy catalog (`docs/agent-instructions/README.md:3-22`); templates must
  declare `derived_from` in `templates/index.yaml`. [R]
- **Skills.** `.agents/skills/` holds seven repo skills (add-arnold-scheduled-accelerator,
  arnold-core-upgrade-pipeline, arnold-targeted-lending, arnold-taskinfo-dev, feature-e2e-verify,
  shuorenhua, tce-status-query); Claude Code reuses them through `CLAUDE.md:7`. `feature-e2e-verify`
  requires a full `target_commit` in every spec, fixed during execution
  (`.agents/skills/feature-e2e-verify/SKILL.md:8-9`). [R]
- **Delivery gates.** Pre-review gate: triggered policies loaded; relevant tests and static checks pass, with
  pre-existing environment failures reported separately; `check_docs_structure.py` passes; component-doc check
  if manifest paths hit; docs propagation, Lark sync and observability decisions made; zero secrets/personal
  files/unrelated diff (`docs/agent-instructions/code-change/workflows/change.md:49-56`). MR descriptions
  express the final diff, not the journey (`git-and-mr.md`, MR bullets). Merge to protected branches requires
  explicit authorization per MR (`git-and-mr.md`, penultimate bullets). [R]
- **Verification dimensions** to consider on every change: contract, UX (Nyx), state convergence, lifecycle
  (owner, cancel, timeout, panic, drain, shutdown), persistence (idempotence, conflict, partial failure),
  external effects proved by readback, observability, docs (`change.md:26-35`). [R]
- **Agents inside the product.** `arnold_info_collector` ships a TaskInfo recognizer that uses "skills" and
  MCP evidence, with a shared `semantic-contract.md` treated as core runtime asset, not ordinary docs
  (`docs/target-design/arnold_info_collector/README.md`, "Runtime skill source assets" row). This is an
  LLM-in-the-loop production feature with its own audit artifacts. [R for existence, not internals]

### 7.2 Mechanisms worth reusing in corenius

1. **Single bootloader + machine index + task routes.** Small `AGENTS.md`, `CLAUDE.md` = `@AGENTS.md`,
   routing in one YAML. Cost model: loads only what the task needs. [R]
2. **Context sufficiency states and reach-out payload** (`evidence.md`). Directly portable; gives agents a
   vocabulary for "stop" that is not "ask a vague question". [R]
3. **Declared facts with invalidation conditions.** Best-in-class way to record human-owned facts (team
   ownership, product boundary) so they expire deliberately. [R]
4. **Layered knowledge with single owners** (current / target / declared / inventory / rollout / ad-hoc), and
   the rule that target design can never prove current state. [R]
5. **Manifests with path digests** binding docs to code: digest drift = mandatory review. A cheap
   staleness detector. [R]
6. **Machine-enforced vs manual-gate labelling** (`policy-evolution.md:19`). [R]
7. **Proportional design-first** rule with objective triggers (cross-component contract, authz boundary,
   persistent state ownership, irreversible migration, failure-recovery strategy, governance change). [R]
8. **Agent-behavior compat tests** (`run_agent_harness_compat.py`): tiny, deterministic prompts with expected
   JSON, run against each supported host. Would catch a governance regression that no unit test sees. [R]
9. **Session capability grants** scoped by time, action class and no-confirm semantics; recorded in the
   agent's output but not persisted. [R]
10. **Ad-hoc investigations with mandatory time boundary** and a "false-positive-prone" banner pattern for
    weak evidence (`docs/ad-hoc/2026-08-18-arnold-reachability-baseline/README.md`). [R]
11. **Criticality by three failure questions** to prioritize review and ownership, kept separate from approval
    rules (`criticality.yaml:21-25`, header). [R]
12. **Question-first topic docs**: each current-system topic opens with 1-5 concrete questions it answers, so
    an agent can decide whether to read on (`docs/current-system/README.md:47`). [R]
13. **Public/private evidence hygiene** (from the observatory decision to keep arnold-facts in the consumer's
    repo rather than pushing upstream): consider a corenius rule of "record facts about a sibling where you may
    write; link to it, cite by pinned line". [O + I]

What not to copy blindly [I]: the sheer weight. `check_docs_structure.py` at 3,767 lines and a 681-line
index are justified by team scale and one repo with ~800 docs; corenius should adopt the *contracts*
(single owner, sufficiency states, digest drift), not the whole checker.

### 7.3 Hazards for agents working in or against arnold

1. **Credentials in data and source.** Env vars in historical task payloads include credentials
   (issue #12); shipped scripts embed tokens (`trace_training_duration.py:39-41`); private `seed.key` files
   are excluded from imports but exist in source repos (`core-components.yaml:101, 130`). Never print raw
   payloads. [R]
2. **Seed vs Nonseed confusion.** Using Seed or shared Merlin/RH2 code as Nonseed evidence is explicitly
   disallowed (`arnold-system-boundary.md:31-37`). [R]
3. **Read-only vs mutating UI/API.** Megavision looks like a read-only page but supports rule edits,
   Toolbox commands, profiling and NDivide restarts with server-side effects; "reading the page does not
   authorize executing these" (`megavision-architecture.md:38`). Housekeeper's HTTP API includes
   block/reset/overhaul (`api_server.go:107-133`). [R]
4. **Production side effects of "diagnostics".** Profiling is a production action requiring PSM, environment,
   sample type and duration confirmation (`production-safety.md:26`). [R]
5. **Shared working copy.** Do not switch branches or stage in a shared copy; use a worktree (`AGENTS.md:9`).
   Contrast with milano's per-agent worktree tooling. [R]
6. **Stale docs.** Target designs and legacy `arnold_info_collector` docs are explicitly "investigation clues,
   not current facts" (`docs/target-design/arnold_info_collector/README.md:3`); ~30+ legacy design docs are
   listed in `quality-baseline.yaml`. [R]
7. **Snapshots expire.** Rule and config snapshots carry query time; using them later as facts is a
   category error (`troubleshooting-and-performance-analysis/README.md:19-20`). [R]
8. **Digest-only doc updates are forbidden**, and the docs gate may fail on clean master ([O]). An agent must
   not "fix" it mechanically.
9. **Ownership is thin.** Owners are single individuals: "single owner reflects current team size, not the
   end state" (`criticality.yaml:33`). Escalation paths for an agent are therefore narrow. [R]
10. **Language.** Instruction and docs are Chinese; `AGENTS.md:1` mandates Simplified Chinese responses.
    A corenius agent writing to arnold must follow this; a corenius doc quoting arnold should translate
    with the original term preserved. [R]
11. **Unauthenticated-looking endpoint.** The trace generator router mounts CORS only and listens on `:8080`
    (`megavision/trace_generator/pkg/router/router.go:11-12, 27`), while the X-Ray router adds
    `RequestAuthAllowList` (`megavision/pkg/router/router.go:15`). Auth may be at the gateway; unverified. [R for code, I for risk]

---

## 8. Open problems, debt, uncertainties

### 8.1 Product / engineering debt

- **ETTR unification.** Two implementations, three stage lists, two denominators (4.2 items 2-4). No single
  owner doc found for "the" ETTR definition. [R]
- **Situation/Karl auto-action path is dormant.** Code exists; not started (2.1). Whether that is intended
  is not documented in what I read. [R / I]
- **Agent collectors partially dead or stubbed.** `NodeMetricCollector` unregistered; File and Log
  collectors are stubs; `ROLLOUT_STATE` publish commented out; two configured routes without emitters
  [O; first one re-checked].
- **No real leader election in Housekeeper**; correctness depends on operational sharding invariants
  (`docs/target-design/housekeeper/README.md:24`). [R]
- **Critical blind spots outside the monorepo:** log-agent to ClickHouse, Analysis Topic consumer, frontend
  and BFF, several Megavision services (`/dev/rl-dashboard`, AlphaSeed, `mlsys.karl.vantage`, log rules,
  HDFS dumper) whose source repos are located or unknown (`megavision-architecture.md:54-60`). [R]
- **Dead-code uncertainty.** 901 candidate unreachable functions; 8 of 25 modules failed analysis for
  environment reasons (reachability README). [R]
- **Open issue backlog** (GQC/Metis/Lending/Security), some requiring production migration or owner action
  (`docs/issue/README.md`). [R]
- **Many legacy docs** not yet migrated to the current/target split (`docs/quality-baseline.yaml`). [R]
- **Frozen typos** as contracts (`avg_effetive_len`). [R]

### 8.2 Trust-model gaps

- No cryptographic evidence: docs claims are validated by structure and digest, not by attestation. [I]
- Runtime truth is intentionally outside the repo; the repo defers to readback, but readbacks are ad hoc
  (via bytedcli and platform UIs) and stored as prose snapshots. [R]
- Harness compat tests are opt-in and only three narrow cases; they are not in `.codebase/pipelines/`. [R]

### 8.3 Uncertainties about this document

- I did not read: Metis, GQC, Operator, Morningstar, `dlp_site`, `mlsysmonitor`, Lending/UCE, Ray, network,
  storage, or `xray-perf-backend` internals. Section 3.2 for those is from catalog roles and docs headers.
- Whether Arnold has Go/Python unit-test gates beyond per-component `build.sh` is unknown; only `docs.yaml`
  lives under `.codebase/pipelines/`. Real CI may be configured on the platform.
- [O] items are from a note dated 2026-09-28 at the same commit; I re-checked a subset (marked). Others are
  believed but not re-read.
- The comparison with milano's trust model in 5.3 and 7.2 draws on the observatory's description and
  the milano `AGENTS.md` instructions given to my session, not on a fresh read of milano.
- I do not know which of arnold's practices are actually followed day to day versus documented.
  Policy text is design intent; the code and CI enforce a subset (docs structure).

---

## How to verify this document

Run from `/data02/home/philip.yang/workspace/arnold` (read-only commands):

1. Confirm the pin: `git rev-parse HEAD` returns `756a165c40a9dc7a386641d869537733335a6748`;
   `git branch --show-current` returns `master`; `git status --short` is empty.
2. Governance entry: `cat -n AGENTS.md CLAUDE.md` (35 and 7 lines; `CLAUDE.md:3` is `@AGENTS.md`).
3. Catalog and criticality: `sed -n 15,40p component-catalog/criticality.yaml`;
   `grep -n excluded_paths component-catalog/core-components.yaml`.
4. Housekeeper state machine: `sed -n 31,42p eurus-housekeeper/pkg/services/driver.go`;
   `sed -n 395,415p eurus-housekeeper/pkg/services/runtime_info.go`;
   `grep -rn "SetRestartWithLevel(" eurus-housekeeper/pkg | wc -l` (31 at this commit).
5. Housekeeper API surface: `sed -n 103,133p eurus-housekeeper/pkg/server/api_server.go`.
6. ETTR: `sed -n 289,308p megavision/ettr/pkg/model/stages.go`; `sed -n 355,398p` same file;
   `sed -n 388,392p megavision/ettr/pkg/model/trial_stages.go`; `sed -n 120,128p` stages.go for the `>=`.
7. Situation service unused: `grep -rn "StartSituationService" megavision --include='*.go'` shows only the
   definition (`situation_service.go:64`).
8. Trace generator: `sed -n 11,28p megavision/trace_generator/pkg/router/router.go`;
   `sed -n 5,12p megavision/trace_generator/pkg/sql/templates/ndtimeline.sql`;
   `sed -n 186,189p megavision/trace_generator/pkg/chmodel/ndtimeline.go`.
9. Agent heartbeat: `sed -n 231,262p eurus-agent/eurusagent/agent.py`;
   `grep -rn NodeMetricCollector eurus-agent --include='*.py'` shows only its own file and its test.
10. Embedded tokens (do not print values): `grep -n "token" dlp_entrypoint/exec/trace_training_duration.py | cut -c1-60`.
11. No sibling references: `for t in milano torchetta ultron ndtl vaso distributed_timeline torchtitan; do git grep -il "$t" -- . ':!external'; done`
    returns nothing except one vendored HTML file for `vaso`.
12. CI: `cat .codebase/pipelines/docs.yaml`; `ls .codebase/pipelines` shows only `docs.yaml`.
13. Compat tests: `sed -n 20,56p scripts/run_agent_harness_compat.py`.
14. Docs authority statements: `sed -n 5,9p docs/README.md`; `sed -n 26,51p docs/agent-instructions/core/policies/evidence.md`;
    `sed -n 19p docs/agent-instructions/core/policies/policy-evolution.md`.
15. Observatory cross-check of [O] claims:
    `/data02/home/philip.yang/workspace/robust-training-observatory/docs/research/2026-09-28-arnold-findings-and-downstream-registration.md`
    and `2026-09-28-associated-repos-source-map.md`. If arnold `HEAD` has moved beyond `756a165`, line numbers
    here may drift (upstream `f3c58fb1b` was reported two Metis-only commits ahead on 2026-09-28).
