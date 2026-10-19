# Reference: robust-training-observatory (RTO)

- Repo: `/data02/home/philip.yang/workspace/robust-training-observatory` (read-only; nothing modified, no git writes, nothing published). **No git remote is configured** (`git remote -v` prints nothing) [X].
- Commit read (master): `584d9b8da62bc27e794076cff44d27b64f688ad9` ("docs: consolidate the Arnold registration commit here as a patch", 2026-09-29 02:26 UTC), branch `master`, 29 commits, all authored as `phi9t`, co-author trailers `TRAE CLI` and `Claude Fable 5.1` [X].
- Commit read (film branch): `a7a2f4cc90bebf82ca3f5e37f52cb444b2b36096` ("feat(step): draw GEMMs in the CUTLASS layout with shared edges", 2026-09-26 20:49 UTC), branch `feat/lifecycle-film`, checked out at `../rto-film`, clean tree [X]. Merge base with master: `7acdc34893e19934757bd6bbd649fed40483da4e` (2026-09-25).
- Date of this document: 2026-09-29.
- Working tree of master at read time (not part of the commit): `CONTEXT.md` modified (uncommitted) and four **untracked** research notes (`2026-09-28-associated-repos-source-map.md` sha256 `dc180a122...`, `2026-09-28-repo-deep-dive.md` `d7aa74203...`, `2026-09-28-training-loop-integration-source-map.md` `755836212...`, `2026-09-29-controller-repos-source-map.md` `9c4c75bdf...`) [X]. They may change or vanish; section 5 says how to treat them.
- Read-only actions I executed: `git log/diff/status/rev-parse`, `node tools/bump-namespace.mjs --report`, `node --test tests/sources.test.mjs tests/geometry.test.mjs` (18 pass, 0 fail; no build, no browser), a `node -e` import of `src/*.mjs` to count content, and `grep` spot checks in the pinned Arnold checkout. I did **not** run `npm test`, the Playwright tests, `run_insula.sh`, or any GPU/network job.
- Evidence labels: **[R]** read in a file (`path:line`); **[X]** executed by me today, read-only; **[I]** inferred, not stated by any source; **[DOC]** a claim made by repo docs that I did not verify. Paths are relative to the RTO repo unless prefixed (`rto-film:` for the film worktree, `arnold:` for the sibling Arnold checkout). Line numbers are from the commits above.
- Companion references in this directory: `README.md`, `arnold.md`, `milano.md`, `ndtl.md`, `torchetta.md`, `ultron.md`. Where they already establish a fact I cite them instead of re-deriving it.

---

## 0. One-paragraph summary

RTO is a **presentation repo**, not a system. It builds one self-contained HTML page (plus, on an unmerged branch, a family of animation pages) that explains how the Arnold GPU-training platform *observes* a training run and *acts* on failures, stragglers and hangs. Every rendered claim resolves to a `{path, line span, symbol}` anchor in a **pinned commit of the Arnold monorepo** (`756a165c4`). Its discipline is the product: anchors are validated against a local checkout, the pin moves only through a ritual tool, charts are labelled illustrative fixtures, and the repo declares that it "presents; it never becomes an authority" (`AGENTS.md:5-7`). Two things ride along that are *not* the page: (a) `docs/research/`, seven notes (four untracked) that map sibling repos and record Arnold findings, and (b) `lab/agent-harness/`, an understanding-by-building experiment that executes the real Eurus Agent collectors on the B200 host, now wrapped in a home-grown "Insula/capsule" launcher with a content-addressed layer model. The capsule design is the most reusable idea in the repo; the page machinery is the most rigorously tested.

---

## 1. Problem, audience, and the pain without it

### 1.1 The stated problem and audience

- Problem: Arnold's robust-training behaviour is spread over Go services (Megavision faultdetector, ETTR, X-Ray), Python node agents (Eurus Agent collectors), the Housekeeper failover state machine, ClickHouse SQL and shell entrypoints. There is no single place that shows how a *trainer signal* becomes a *detection* becomes an *action*, with the code one click away. RTO is that place (`README.md:3-11`, `docs/adr/0002-system-walkthrough.md:7-13`) [R].
- Audience: explicitly "a model person" (`ADR 0002:9-13`): someone who reasons from the PyTorch trainer, model/optimizer state and checkpoint/trace artifacts, not from Arnold's service topology. Secondary audience is agents: `AGENTS.md` and `CONTEXT.md` are written so an agent can extend the page without breaking the evidence contract [R][I].
- Origin: modelled on `~/workspace/stratum-observatory` (checkpoint/lens/operation mapping, `CONTEXT.md:6-7,47`) and grounded on a Lark doc "Arnold observability" (revision 573; `docs/research/2026-09-24-lark-observability-grounding.md:3-6`). The Lark text is deliberately **not** copied into the app (`:9`) [R].

### 1.2 What the page teaches: Arnold's observe/act path, as the page states it

Content counts at HEAD [X, via `node -e` import]: 3 phases, 7 lines, 23 stages, **87 node cards**, 161 anchors, 35 collected signals in 8 semantic groups, 6 illustrative chart fixtures, a hero diagram of 5 lanes and 14 components. Prose numbers below are the page's claims (`src/lines.mjs`); the tests prove the *anchor location*, not the prose. I spot-checked four constants against the pinned Arnold checkout and they match [X]: `ValidTimeSecond = 660` (`arnold:megavision/faultdetector/pkg/flightrecorder/hang.go:22`), `NoDataTimeThreshold = 30` (`trialdetector.go:17`), the 50-step post-recovery window (`arnold:eurus-housekeeper/pkg/services/driver.go:440,1876`), and the 0.5% / 10-violation compare (`arnold:eurus-housekeeper/pkg/utils/training_metrics_check.go:248-249`).

**Phase 1, Data collection** (lines `pipeline`, `collection`; `src/system.mjs:19-25`, `PHASE_LINES`):
- A run is observable only once trainer state, rank/host placement, collector payloads, live streams, replay tables and alert-rule storage can be joined by **Trial, Run and time** (`PHASE_OVERVIEWS.collect.summary`, `system.mjs:20-22`). The `Correlation frame` card (Trial, Run, rank, topology) is the first card for that reason [R].
- The trainer emits two raw signals: NDTimeline CUDA timer arrays and the NCCL Flight Recorder (FR) ring of in-flight collectives (`lines.mjs` collection/instrument). The Eurus Agent `UnifiedDataCollector` owns a map of collectors per role (collection/gather).
- **Registration is conditional, and the page says so.** The `AvailableCollector` enum lists 22 collectors; the set that runs is decided at Agent start and by heartbeat: 4 always on (PySpy, dmesg OOM, HDFS client etime, cache cleaner), 12 gated by feature flag / Ray role / management level / device type / Copilot flag, 3 created from heartbeat state (LogAnalyze, CkptStepChecker, LossSpikeChecker), and 3 enum members never registered (NodeMetric via nvidia-smi, File, Log) (`lines.mjs`, collection/registry card) [R]. The signal ledger states a **wiring** gate for each of 35 signals (`system.mjs:232-`, field `wiring`) [R].
- GPU device metrics do not come from the Agent; they come from the host-side `mlsysmonitor` (30 s, Kafka to ClickHouse), per the `gpu-device` ledger group (`system.mjs:655-`) and the findings note [R].
- Store split (why detection, replay and UI have different evidence boundaries): BMQ live, ClickHouse timers/FR/events, RDS Alert/AlertRule, `arnold-x-keeper` ETTR (collection/emit "Sinks" card).

**Phase 2, Detection, analysis, triage** (lines `hang`, `straggler`, `ettr`, `metrics`):
- **Hang** (`lines.mjs:258-443`): BMQ FR dumps route to a per-Trial detector (a larger RunID rebuilds it). Diagnosis is admitted only when the latest trigger has dumps from every rank (`world_size`) **or** nothing has arrived for more than 30 s ("silence itself is a signal"); records older than 660 s are dropped; fewer than 2 valid timestamps or no observed NCCL work yields `NoEnoughData` instead of a guess. A rank is hung iff its uncompleted-OPID set is identical across all dumps (`reflect.DeepEqual`); if any rank advances the trial is `NotAllStuck` (slow, not hung). Classification: a bounded set of idle nodes (`idleNodes <= MaxBlockNode`) is `NoNcclOpFailure`; one idle model-parallel group is `MPGroupStuck`; all idle is `AllRankIdle` and **not** a hang. Localization: the lone MP group whose head-stack fingerprint differs; inside it, PP send-distance `(dst-src)*tp*dp` and blocker counts, or the DP "owe-count" alternative. RDMA/TensorCore activity is only a **progress proxy**, "not root-cause proof", so compute-only, dataloader and checkpoint windows do not collapse into the hang label (hang/admit `progress` card) [R].
- **Straggler** (`lines.mjs:444-528`): per-(trial, run, role, rank, step, metric) rows with parallel `start_ts[]`/`duration[]` arrays in `megatron_cuda_timer_tracing_original` (CN `lab_vc_sre_lq`, US `mlsys_monitor_maliva_2`). DP skew is `max(start+dur) - min(start)` per `dp_rank` over forward/backward compute only; step trend excludes `data-load`/`data-sync`; queries are batched (512 workload cap, 15 s sub-query, adaptive 1/2/4/8 concurrency). `initThreadIndexTableMegatron` is the *authoritative consumer* of NDTimeline metric names: a renamed upstream metric must land there [R].
- **ETTR goodput** (`lines.mjs:529-734`, 23 cards): producers (trainer `StageLogger`, container entrypoint markers, Metis pod-event reconstruction, Eurus/Housekeeper markers such as `robust` and `traffic_zero`, Arnold API writes straight to BMQ) feed a query boundary (level 2 rows, rank 0/-1, pod names head/worker/executor only), a sweep-line single-owner timeline, unknown-gap spans (real wall time that never counts as effective), `unsaved_*` renaming of training with no preceding checkpoint, an effective-stage whitelist, and `ETTR = sum(effective)/sum(total)`. Collector side: idempotent effect key `ettr:v2:platform:database:trial`, frozen versioned payload with stable `EventID` (exactly-once seam), typed not-ready retries (5 min; quarantine only at 64 attempts **and** 24 h), reviewed capacity profiles (`baseline..s16`, fail-closed if not in the TCC allowlist) [R].
- **Training metric guards** (`lines.mjs:735-803`): loss-spike confirmation (moving average vs threshold times historical minimum; "exploded" only if the last `stepThreshold` points are all spikes) and a post-recovery same-step compare of `training/loss` and `training/avg_effetive_len` (misspelled in Arnold code) over the first 50 steps; failure raises a Housekeeper **Warn**, never a rollback [R].

**Phase 3, Action triggers** (line `failover`, `lines.mjs:804-890`):
- Four outcomes are kept **separate and non-equivalent**: an X-Ray Alert for humans, a BGCP/Copilot profiling capture, a `CheckNcclHang` deterministic replay, and a restart. "An alert row is evidence, not proof that a restart happened" (`lines.mjs:833`); a hang becomes a *Robust Event* (error level, drives failover) only when the fault-node set is within the block radius and the robust config opts in, else a debug Alert Event (`lines.mjs:427`) [R].
- Restart converges on `RuntimeInfo.SetRestartWithLevel` (merges the highest diagnostic level across concurrent triggers, sets `NeedRestart`; targets Init/Spawn/Run), `DenyNodes` (fault nodes excluded from next spawn, projected by Morningstar into Trial affinity), and the Housekeeper `StatusTransMap` (Init to Spawn to Run; Run/Spawn to Failover; Failover to Check; a run that never stabilizes to Aborted at `MaxRetryThreshold`) (`lines.mjs:839-880`) [R].

### 1.3 What goes wrong without it

Source-backed (from the page's own findings and the research notes) unless marked [I]:
1. **Mis-reading proxies as verdicts.** RDMA=0 alone reads as "hang" during checkpointing or dataloader stalls. The page encodes the guard (progress proxy plus stage context). The harness even reproduced the failure shape: inside the capsule the RDMA collector returned `[0.0, 0.0]` because no NICs were visible (4.4 item 10) [X].
2. **Believing collectors exist that do not.** `NodeMetricCollector` is dead code; File and Log are stubs; `robust_training_3dtimeline` and `robust_runtime_info` routes have no emitter in the Agent tree (findings note, "collector audit") [R].
3. **Two incomparable ETTR numbers.** X-Ray `/ettr/trend` (from `xray_event` alert names) and ettr-service (from `megavision_ettr_log`) use separate effective-stage sets; the effective list exists three times; window trend divides by window length, not stage sum (same note) [R].
4. **Silent capability loss.** `dmesg` denied to non-root makes the OOM command collector return `{}`, indistinguishable from "no OOM" (`lab/agent-harness/README.md`, H6 refinement) [R].
5. **Alert conflated with action.** Arnold's own docs say Alert does not imply Situation/Karl/Failover (Lark grounding note, "Arnold evidence used"); RTO keeps them on separate cards [R].
6. [I] Without a pinned, anchor-checked walkthrough, the same knowledge lives in a Lark doc (revision-drifting) and in code that refactors underneath any prose. `npm run bump:arnold` turns "the docs are stale" into a failing test.

---

## 2. Non-goals and authority model

### 2.1 Non-goals (stated or structurally enforced)

- **Not an authority.** "It presents; it never becomes an authority" (`AGENTS.md:5-7`). Arnold is the product authority; RTO does not push to `mlsys/arnold` and records Arnold facts as research (`AGENTS.md` Rules; `ADR 0003:40-46`) [R].
- **Not production telemetry.** Charts run on hand-built fixtures labelled illustrative (`src/fixtures.mjs:1-6`; six fixtures: `opidStability`, `mpFingerprint`, `dpSkew`, `stepTrend`, `ettrLedger`, `failoverStates`) [R].
- **Not a runtime controller or monitor.** Nothing in the page reads a live run. The lab reads none either; it executes collectors against fakes.
- **Not a multi-repo citation site (yet).** Sibling-repo links stay out of `src/sources.mjs` "until the app has a tested multi-repo citation model" (`CONTEXT.md` evidence model item 9, uncommitted edit) [R].
- **Not Bazel, not Docker at run time, not a security boundary** for the capsule: the spike's non-goals include "security boundaries against a hostile candidate" (`docs/design/2026-09-29-insula-capsule-spike.md:14-15`) [R].
- **Not a place to commit generated state:** never `.cache`, `.venv`, `out/`, or py-spy output, because a dump on a shared host contains other people's processes (`AGENTS.md` Rules; `.gitignore`) [R].

### 2.2 Namespaces and pins

`src/sources.mjs:13-36` declares `NAMESPACES` [R]:

| key | policy | pinned commit | repo base | local checkout resolved by | marker file |
| --- | --- | --- | --- | --- | --- |
| `arnold` | `product` | `756a165c40a9dc7a386641d869537733335a6748` | `code.byted.org/mlsys/arnold` | `RTO_ARNOLD_DIR`, `~/workspace/arnold`, sibling | `component-catalog/core-components.yaml` |
| `distributed_timeline` | `producer` | `b43b3e197fd60f79762a0b62671b87a2e0672108` | `code.byted.org/wanghao.2023/distributed_timeline` | `RTO_DISTRIBUTED_TIMELINE_DIR`, ... | `bytedance/ndtimeline` |
| `ndtl` | `experimental` | `f7dfaa0ec7f8a24128a5d65b08106e9e2310a9a2` | `code.byted.org/philip.yang/ndtl` | `RTO_NDTL_DIR`, ... | `src/ndtl/schema.py` |

- `CARD_VISIBLE_POLICIES = ['product', 'producer']` (`sources.mjs:38`). Policies gate **visibility, not existence** (`ADR 0003:26`): `product` is the only authority for app-visible product claims; `producer` may explain provenance of a signal Arnold consumes; `experimental` may appear in research notes only. `tests/sources.test.mjs:37-43` fails if any rendered card, hero box, ledger entry, phase overview, data-model record or chart fixture cites a namespace outside those policies (`renderedIds()`, `:22-31`) [R].
- `srcUrl()` (`sources.mjs:53`) builds `<ns.base><ns.commit>/<path>#L<a>-L<b>`; links are never hand-typed. `COMMIT` and `REPO_BASE` remain as arnold-only aliases (`:49-50`) [R].
- **Actual usage today [X]:** all **161** anchors are `arnold` (roles: 90 `go_impl`, 52 `py_impl`, 12 `doc`, 5 `sql_template`, 1 `sh_impl`, 1 `ch_table`). `distributed_timeline` and `ndtl` have **zero** anchors, so their per-namespace anchor tests pass **vacuously** (they iterate an empty list) and the "experimental may not render" rejection has never been exercised on real data [I].
- Freshness today [X, `node tools/bump-namespace.mjs --report`]: `arnold` at pin, dirty=0; `distributed_timeline` at pin, dirty=7 (untracked `.scratch/`); `ndtl` **HEAD `8bc8ba711` != pin `f7dfaa0ec`**, dirty=34. The ndtl drift is one commit ("chore: ignore local worktrees"), harmless, but it shows the pin for an unused namespace is not maintained.

### 2.3 The pin-bump ritual (`tools/bump-namespace.mjs`, `tools/checkouts.mjs`)

- Only sanctioned way to move a pin (`ADR 0003:34-39`). Modes: `--report`; `<ns>` dry run validates every anchor against the checkout's HEAD; `<ns> --write` rewrites the commit literal; `--commit <sha>` requires the checkout to already be at that sha (`bump-namespace.mjs:2-8,39-45`) [R].
- Refusals: dirty **product** checkout (`:46-49`), checkout HEAD not equal to requested commit (`:40-44`), any drifted anchor (`:60-62`, exit 1). On drift it prints candidate lines for the moved symbol and **never edits an anchor**: "a human re-reads each moved span" (`:1-10`, `ADR 0003:38-39`) [R].
- `--write` uses a regex over `sources.mjs` (`:66-70`) that locates the namespace's `commit: '<old>'` literal; if the file is reformatted so the regex misses, it exits 2 rather than corrupt [R].
- `RTO_STRICT_PIN=1` makes a non-pinned or dirty product checkout fail the suite (`sources.test.mjs:70-83`); everyday runs only *report* [R].

### 2.4 Anchors as a tested invariant, and how strong the test really is

`checkAnchor` (`tools/checkouts.mjs:56-73`) returns `ok | missing_file | out_of_range | symbol_moved` [R]:
1. The file exists in the checkout; `lineStart`/`lineEnd` are within the file length.
2. The **first** symbol token (`primarySymbol`, `:48-51`, split on `; , . ( `) appears as a **substring** within `[lineStart-3, lineEnd+2]`.
3. Symbols of 2 characters or fewer are skipped (`:65`).

Consequences [I]: it detects deleted files, truncated files and moved functions; it does **not** detect a function whose body changed while its name stayed, a span that shrank around the same name, or prose that misstates behaviour. The repo-deep-dive note says the same ("intentionally heuristic ... human review remains required", "Verification contract") [R]. Also, with no checkout present that namespace's span checks are **skipped, not failed** (`sources.test.mjs:59-62`), so an off-box `npm test` can pass while every span has drifted [R].

### 2.5 Fixtures versus telemetry

- Fixtures: `src/fixtures.mjs` header: HAND-CONSTRUCTED teaching examples that mimic the *shape* of the cited computation; each names the source id it visualizes (`:1-7`); each chart is captioned "illustrative" (`src/app.mjs:90`, per the deep-dive note) [R].
- Harness output is a third category: `lab/agent-harness/out/report.json` is real collector code fed by a real host plus named fakes. The `AGENTS.md` term table defines **fixture** as "hand-built or harness-captured data ... always labeled illustrative or harness output, never production telemetry" and **fake** as a stand-in for a service the host lacks, "always named in the harness report" (`AGENTS.md:39-40`) [R]. No harness output has been fed back into `fixtures.mjs` (Run 2 plan item 5 is not done) [X].
- The film branch adds a fourth surface: `ILLUSTRATIVE = true` scripted timelines (`rto-film:src/film-script.mjs:13`), whose only real-world tie is lane/metric *names* checked against Arnold (section 6) [R].

---

## 3. Architecture of the page

### 3.1 Layers (facts, then narrative, then presentation)

`AGENTS.md` Rules: "Facts before narrative before presentation: change `src/sources.mjs`, then `src/system.mjs` / `src/lines.mjs`, then `src/app.mjs` / styles" [R].

| Layer | File | Lines | Job |
| --- | --- | --- | --- |
| Facts | `src/sources.mjs` | 1085 | `NAMESPACES`, `SOURCES` (id to `{title, path, lineStart, lineEnd, role, symbol, anchor, ns?}`), `srcUrl`, `source(id)` (throws on unknown id, `:1081`) |
| Narrative | `src/system.mjs` | 771 | `PHASES`, `PHASE_OVERVIEWS`, `SYSTEM` (hero lanes/components/edges), `PHASE_LINES`, `DATA_MODEL`, `SIGNAL_INVENTORY`, `SIGNAL_GROUPS` |
| Narrative | `src/lines.mjs` | 904 | `LINES`: line to stage to node; each node `info = makeInfo(kicker, title, body, rule, facts, sources, related, pseudocode)` (`src/scene.mjs:6`) |
| Narrative | `src/fixtures.mjs`, `src/charts.mjs` | 89, 182 | illustrative data; deterministic SVG chart builders |
| Presentation | `src/app.mjs` | 486 | render, ⌘K search, theme, share view; state in URL fragment + `localStorage`; exposes `window.RTO` for tests |
| Presentation | `src/systemviz.mjs`, `src/styles.css`, `index.html` | 204, 379, 39 | hero SVG swimlane, data-model diagrams, tokenised dark/light theme |
| Build | `build.mjs` | 69 | see 3.2 |

### 3.2 Build (`build.mjs`)

- Dependency-free Node. `FILES = ['sources','scene','fixtures','charts','system','systemviz','lines','app']` in fixed order (`build.mjs:14`), each module rewritten to an IIFE registered in `__modules`, CSS inlined into `index.html`, output `dist/index.html` (`:50-69`) [R].
- Supported ESM subset only: single-line named imports, `export const|let|function|class X`, `export { a, b }`; anything else throws "Unsupported module syntax" (`:16-44`) [R]. Master's `rewrite()` has **no** guard against `export const A = 1, B = 2;` (silently exports only `A`); the film branch added that guard after "this hid the mesh once" (`rto-film:docs/adr/0005-parallelism-gallery.md` "Consequences"; `rto-film:build.mjs`) [R].
- Deterministic: same sources produce the same bytes [DOC, `README.md`; not re-verified]. **`dist/index.html` is tracked in git** on master, and `previews/*.png` too, so `npm test` (builds, and the visual tests write screenshots to `previews/visual-verifier/`) dirties tracked files [R: `git ls-files`; `README.md` Develop][I for the dirtying].

### 3.3 Content model: line, stage, node, phase

`CONTEXT.md:11-24`, ubiquitous language [R]: **Line** = a pipeline (stratum's *checkpoint*); **Stage** = ordered step in a line (*lens*); **Node** = one function/type/decision rendered as a card (*operation*); **Source** = resolvable anchor; **Fixture** = illustrative numbers. Cards carry: lead, one-line **Rule**, optional **Logic** pseudocode (faithful paraphrase of Go/SQL for about 6 algorithmic cards), facts grid, optional chart, "Open the code".

| Phase (`PHASES`) | Lines (stages, nodes) [X] |
| --- | --- |
| `collect` | `pipeline` (4, 19); `collection` (3, 7) |
| `detect` | `hang` (5, 18); `straggler` (3, 8); `ettr` (4, 23); `metrics` (2, 4) |
| `act` | `failover` (2, 8) |

`ADR 0001` (workbench with a scene IR and `structuralReport`) was superseded by `ADR 0002` (hero swimlane plus single-column phase document); `diagrams.mjs` and `structuralReport` are retired, but `src/scene.mjs` survives as an 8-line `makeInfo` helper and stays in the build order [R].

### 3.4 Tests (18 always-on Node tests, plus 8 browser tests)

Run as `npm test` = `npm run build && node --test tests/*.test.mjs` (`package.json:9`) [R]. I ran the two non-browser files: **18 tests, 18 pass** [X].

- `tests/geometry.test.mjs` (11 tests; the name is historical, it is now a content-graph verifier): lines have known phases; `PHASE_LINES` covers every line; each phase overview has a summary of at least 80 chars, exactly 3 points, at least 2 sources including one `doc` and one code/table/schema source; node ids unique per stage; every hero component resolves a source and has valid edges; **hero components map one-to-one to a same-phase deep-dive card that cites the same source** (`:94-121`); every card is source-grounded; data-model records resolve; the signal inventory has at least 20 signals; the semantic grouping covers every signal exactly once [R].
- `tests/sources.test.mjs` (7 tests): id resolution, card-visible policy, entry well-formedness (40-hex commit, URL built from the namespace pin), per-namespace anchor validity, freshness report (`:33-83`) [R].
- `tests/visual-overflow.test.mjs` (8 tests; `playwright-core` 1.63.0 locked; system Chrome `/usr/bin/google-chrome` or `CHROME_BIN`, `:10`): 5 viewports (1440, 768, 390, 320, 812x420 landscape, `:13-17`); horizontal overflow, SVG text inside its card/viewBox, readability at narrow widths, rendered-DOM source-link grounding, ledger starts collapsed then expands, TOC auto-hides, theme polish. **Not** run by me [R]. Limits per the deep-dive note: Chromium only, viewport sampling, no pixel diff, search/URL-share/clipboard only partially covered [R].

### 3.5 Publish path

- `.agents/skills/goofy-preview-publish/SKILL.md` (83 lines; `agents/openai.yaml` is Codex UI metadata) [R]. Steps: `npm test`, `npm run build`; `bytedcli --version`, `bytedcli --json auth status`; then `bytedcli --json goofy preview deploy dist --alias robust-training-observatory --description "..." --override`; verify with `goofy preview list`; optional `curl -I` against `https://robust-training-observatory.gf-preview.bytedance.net/`.
- The skill reports deploy success, HTTP reachability and browser-render verification as **separate conclusions** (SKILL.md "Reporting") [R].
- `AGENTS.md` Rules require: "After any change to the page, rebuild and republish `dist/` to the Goofy Preview alias" [R]. Hazards in 7.3.
- Arnold side: a local Arnold commit `5f2f9190f` that registered RTO as a downstream in two Arnold READMEs was **withdrawn** (branch deleted from the Arnold checkout 2026-09-29) and kept as `docs/research/patches/0001-...patch` (apply with `git -C ~/workspace/arnold am`). Arnold therefore has no in-repo pointer to RTO (`arnold.md:485`; findings note) [R].

---

## 4. `lab/`: understanding by building

Only one experiment exists: `lab/agent-harness/`. Its premise (`README.md:11-14` there): the static signal ledger answers "what is collected" from source; the experiment answers it by **executing the production collector classes unmodified** on a real GPU host, faking only missing services, and recording the bytes produced [R].

### 4.1 The experiment (`harness.py`, 284 lines)

- 18 falsifiable hypotheses H1..H18, one per collector (NodeMetric never registered; TensorCore via `POST :12200/api/aggrByEntity` metric `1004`; RDMA via `ip a` heuristic plus `ethtool -S`; Memory; PeriodicOOM; dmesg OOM; HDFS etime; TorchElastic error.json; Ascend DCMI log; LogAnalyze; LossSpike; CkptStep; Checker; FlightRecorder dumping-signal file; PySpy; RayJob; RayActor; heartbeat assembly). Out of scope: RolloutState, BGCP, Copilot, UserCacheCleaner (README table) [R].
- Code is imported unmodified from `~/workspace/arnold/eurus-agent` (via `PYTHONPATH`; in the capsule via `RTO_ARNOLD_DIR=/workspace/candidate`, `rootfs_plan.json` env). Fakes named in the report: HTTP `:12200`, cgroup memory, Merlin `check_loss_spikes`, Lark `send_msg`, Housekeeper session, `BmqPublisher.produce` (captured instead of posted) (`harness.py:1-9` docstring, README "Setup") [R].
- **Pilot 2026-09-28** (pre-capsule, on the host): confirmed H1-H4, H8, H11, H13-H16, H18; refined H3, H6; H17 unconfirmed; two harness defects listed for run 2 (README "Pilot results") [R].
- **Run 2 plan not executed [X].** Both defects persist in the capsule run. My read of the local, gitignored `lab/agent-harness/out/report.json` (capsule `sha256:e627c0a3...`) shows `PeriodicOOMChecker: error` (`SimpleNamespace has no attribute 'raise_for_status'`) and `LogAnalyzeCollector: error` (`'list' object is not callable`; `bootstrap_complete_ranks` is called as a method, `harness.py:165`). Tally of the 22 records: 17 ok, 3 skipped, 2 error [X]. `RayActorCollector` reports `ok` with `actors_published: 0` (Ray GCS on `127.0.0.1`, not `::1`), which `INSULA.md` calls an "open item" [R][X].
- Stale text [R]: the README and the `harness.py` docstring still say `source .venv/bin/activate && python harness.py` and "specified + piloted once"; the supported entry is now `run_insula.sh`. The pilot's H15 count (2610 `PY_SPY` records from every Python process on the host) versus the capsule's 20 records is the stated evidence that the Insula fixes visibility (`INSULA.md` "Evidence") [R].

### 4.2 Terminology (AGENTS.md, the shared vocabulary)

`AGENTS.md:14-46`. The table is meant to be shared with milano, `lagrange-torch-mlu-insula` and `distributed_timeline` ("Do not introduce synonyms", `:16-18`) [R]. The four properties must be stated **separately and by name**; conflating them ("hermetic means no network") "is the confusion this table exists to prevent":

| Property | Attaches to | Meaning | Established by | Not |
| --- | --- | --- | --- | --- |
| **hermetic** | rootfs / capsule | every toolchain, interpreter and package comes from the rootfs; nothing resolves from the host except the driver interface. About *provenance of dependencies* | build time; verified by the probe | network isolation, offline |
| **isolated** | rootfs **plan** | which namespaces are private (pid, net, mount, ipc) and which host paths are bound | enforced by the Insula at launch | hermetic, sandboxing |
| **offline** | a **run** | plan gives no network beyond loopback; nothing fetched at run time | the plan (`--unshare-net`) | hermetic |
| **reproducible** | rootfs **build** | committed build script + pinned base image + pinned package versions produce the same `rootfs_digest`; the digest in `selected.json` is the proof (`AGENTS.md:37`) | rebuild and compare | deterministic, stable |

Other load-bearing terms [R]: **Insula** = the host-side execution boundary (bwrap launcher, private pid/net/proc/dev/tmp, NVIDIA driver interface), "a facility of the host; it holds no code of its own"; **capsule** = immutable content-addressed unit the Insula executes (rootfs + candidate + rootfs plan, digest recorded in `capsule.json`); **candidate** = code under test, bound read-only; **driver interface** = the only host contribution; **run** = `capsule x command -> evidence`; **sandbox** is *reserved* for TraeCode CLI's process sandbox and never used for the Insula; **fake** vs **fixture** as in 2.5; **layer** and **layer build** (below).

### 4.3 Capsule design (`AGENTS.md` "Capsule anatomy", `docs/design/2026-09-29-insula-capsule-spike.md`)

Layers ordered by **change velocity**, slowest at the bottom, so a change costs exactly its own layer's rebuild [R]:

| # | Layer | Contents | Changes | Built by | Mount | Digest |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | base rootfs | distro userland, python3.12, py-spy, ethtool, iproute2, procps | rarely | `rootfs/build_rootfs.sh` | `/` ro | `rootfs_digest` |
| 1 | spack foundation | ABI-coupled native deps (CUDA/NCCL-facing, torch) | weeks | a layer build in the Insula | `/run/insula/layers/spack` ro | `spack_digest` |
| 2 | uv overlay | fast Python deps from `uv.lock` (ray, pytest, byted packages) | days | `run_insula.sh build uv` | `/run/insula/layers/uv` ro | `uv_digest` |
| 3 | candidate | code under test, from the staged tree | every run | `git archive` | `/workspace/candidate/<name>` ro | `candidate_digest` |
| n | future layers | data, weights, configs | as needed | own build | `/run/insula/layers/<name>` | `<name>_digest` |
| - | state | evidence dir, scratch | per run | - | `/run/insula/out` rw, `/tmp` | none (evidence carries the capsule digest) |
| - | driver interface | NVIDIA libs via `ldconfig -p`, `/dev/nvidia*`, `/dev/infiniband`, host `nvidia-smi` | with host | host | ro allowlist | in `plan_digest` |

Rules: canonical **path contract** independent of host paths; **change costs its own layer**; **hermetic is per layer, not per blob** ("being baked into one filesystem was never the point"); **no installs at run time** (`pip`/`uv` shims fail fast; layer builds are the only runs with a writable layer mount, and only for the layer they produce); **do-not-shadow** across layers; digests **compose in stack order**: `capsule_digest = sha256(rootfs || spack || uv || candidate || plan)`, so "a diff of two capsules names the layer that moved" [R]. Inspiration and deliberate differences from draccus are in `docs/research/2026-09-28-draccus-as-reference-for-capsule-layering.md` (kept: path contract, Spack-vs-uv split, do-not-shadow, ldconfig driver discovery, offline as a separate switch; dropped: mutable venvs, writable bundle state, interactive shell, the "bundle" unit) [R].

### 4.4 What is implemented versus designed

Implemented and committed (commits `cd2f805`, `dadf96f`, `eca33e7`, 2026-09-28/29; the earlier `capsule.py`/`run_capsule.sh` were replaced) [R]:
- `run_insula.sh` (80 lines): `build rootfs | build uv | build capsule | run -- CMD | probe | show`; the default run is `python harness.py` [R].
- `rootfs/insula.py` (1171 lines) builds the capsule and the resolved bwrap plan: digests over rootfs tree, uv inputs+tree, a deterministic candidate tar from the **staged** tree, a canonical-JSON plan digest, and the `capsule_digest` formula (`insula.py:511-518`); staleness refusal for rootfs, uv, plan and candidate (`ensure_capsule_fresh`, `:771-800`); allowlisted binds (only `@`-placeholders resolve; driver libs from `ldconfig -p`, `:807-`); `--unshare-user/ipc/pid/uts/cgroup-try/net`, `--clearenv`, `--die-with-parent`, `ip link set lo up` inside the namespace (`:892-896,1018`); host and pin guards (`EXPECTED_HOSTNAME = "n116-077-207"`, `EXPECTED_ARNOLD_COMMIT`, refuses a dirty or moved Arnold checkout, `:30-31,211-216,286-299`) [R].
- `rootfs/probe_insula.sh` (277 lines): writes `out/probe.json` with verdicts for `hermetic`, `shadowing`, `isolated`, `offline`, `gpu`, `identity`, `py_spy`; `identity` recomputes the capsule digest from the `capsule.json` pieces and compares it to `RTO_CAPSULE_DIGEST` (`:179-197`) [R].
- Committed identity record `capsule.json` (built 2026-09-29T01:43:27Z, host `n116-077-207`, kernel `5.15.152.bsk.9-amd64`, driver `580.105.08`): capsule `sha256:e627c0a3...78c46`, rootfs `fc0ef8af...`, spack `e3b0c442...` (= `sha256("")`, the deliberately empty slot), uv `72d2007f...`, candidate `f8ca1551...` (pinned `eurus-agent` + 3 harness files), plan `43e916e7...` [R].
- Spike acceptance behaviours I could not re-execute (no Insula run, by design): a repeat `build capsule` gives the same digest; editing `uv.lock` moves only `uv_digest` and `capsule_digest`; editing `harness.py` moves only `candidate_digest` and `capsule_digest` (`INSULA.md`, "Reproducible") [DOC]. `capsule.json` was last committed with the last harness edit (`eca33e7`), so it is plausibly fresh [I].

Designed only, or not as designed (all [R] unless marked):
1. **Spack layer**: empty sentinel; no Spack build exists (`layers.toml`: `build = "none"`; design non-goal and open question).
2. **Layer sharing, extraction to a sibling `insula` repo, a layer registry**: open questions (design section 7).
3. **Base-image provenance**: the design lists "record the base image manifest digest in `selected.json`" as open. Today `selected.json` records `base_ref: "python:3.12-slim-bookworm"` (a mutable tag) and `base_kind: docker-export`; `build_rootfs.sh` runs `apt-get update && apt-get install` unpinned and `pip install uv==0.12.17 py-spy==0.4.2` (`build_rootfs.sh:230,412-417`) [R].
4. **Probe classification differs from the design.** Design section 4 says paths are classified "by its mount from `/proc/self/mountinfo`"; the code classifies by **path prefix** (`probe_insula.sh:137-160`; no `mountinfo` read), treating `/usr`, `/etc`, `/lib*` as "rootfs" by name, so a host path bind-mounted at `/etc/...` would classify as rootfs [R][I]. `shadow_owners` is 5 names (`numpy, requests, psutil, ray` uv-owned; `json` rootfs-owned), not "every importable name" [R].
5. **Capsule identity does not cover the launcher.** `HARNESS_CANDIDATE_PATHS` is a hard-coded 3-path tuple: `README.md`, `harness.py`, `rootfs/probe_insula.sh` (`insula.py:34-38`). `run_insula.sh`, `insula.py`, `build_rootfs.sh`, `layers.toml`, `INSULA.md` are **not** in `candidate_digest` (the rootfs recipe digest lands in the rootfs marker; plan, `pyproject.toml`, `uv.lock` are hashed elsewhere). The code that builds and launches the capsule can change without the capsule digest changing [R][I]. The design said the candidate is "a git archive of the staged tree of each source"; for the harness it is a fixed file list.
6. **Freshness check is unstaged-only.** `candidate_sources_dirty` runs `git diff --quiet -- <path>` (`insula.py:302-308`), which flags unstaged edits; the archive uses staged content. That is milano's "stage before you run" convention and the error text says so, but a staged-and-never-committed change is baked into the digest silently [R][I].
7. **Reproducible is claimed, not demonstrated.** `AGENTS.md:37` defines it as "committed build script, its pinned base image and pinned package versions produce the same `rootfs_digest`". With a tag base and unpinned apt, two rebuilds are not guaranteed to match; the `selected.json` digest is an *identity record of one build*, not proof of rebuildability [R][I]. There is no rebuild-and-compare gate.
8. **Docker is a build-time dependency of layer 0** (`docker build` + `docker export`, `build_rootfs.sh:395,461-463`), with a `--base-rootfs DIR` fallback. Vaso states "Docker is not a runtime dependency" for its own rootfs (`vaso/docs/rootfs-container-infra-design.md:13`). Different repos, different stance [R].
9. **No GPU lease.** Milano takes exclusive `flock` GPU leases; the harness does not. The capsule's own `nvidia-smi` capture shows other load at the same moment (GPUs 1 and 3 at 100% util, ~990 W), so H1 output is host-contaminated on a shared box [X: `report.json` `NodeMetricCollector.result`][I on cause].
10. **`--unshare-net` removes the NICs.** Inside the capsule `RdmaTrafficCollector` saw `eth_set: []` and returned `[0.0, 0.0]`, and `report.json` host block shows `infiniband: []`. So the collector that matters most for hang detection cannot be exercised for real inside the capsule; the pilot's real-NIC run predates the capsule and the two are not comparable [X]. This is the offline/isolated property colliding with the experiment's goal, and neither `INSULA.md` nor the README says so [I].
11. **Harness clobbers evidence.** `harness.py` begins with `shutil.rmtree(OUT, ...)` (`:24`); the probe writes `out/probe.json` (`probe_insula.sh:242`). The local `out/` has no `probe.json`, consistent with the harness run after the probe having wiped it [X][I]. The design promised both files carry `capsule_digest`; only what survives the last run is on disk.

### 4.5 How the capsule relates to milano's Insula (cross-check with `milano.md`)

- `AGENTS.md` says the capsule is the "same identity triple as milano's `InsulaIdentity`". Milano's `InsulaIdentity` is `(target, candidate_digest, rootfs_digest)` (`milano:milano/insula.py:7-13`); RTO's capsule is rootfs + candidate + **plan** (+ uv + spack). Not the same triple. And `milano.md:632` records that `InsulaIdentity` is **unused by milano's pipeline** [R].
- Milano's Insula **shares the network** and binds host `resolv.conf`/`hosts`; its "hermetic" is explicitly rootfs-level, "not network isolation" (`milano.md:100`, `milano/rootfs.py:118-121`). RTO's Insula definition includes a loopback-only net namespace. The word *Insula* is shared; the boundary is not. RTO's *offline* is an RTO-only property (milano's `--offline` exists but no gate uses it, `milano.md:310`) [R].
- Milano has no sealed layers: `/cache/milano` is one read-write mount holding Spack, venvs, builds and evidence (`milano.md:486`). RTO's sealed, content-addressed layer stack is its own design, not a description of milano [R].
- So the capsule is best read as **a proposal for where a milano-style Insula could go**, prototyped on a small Python-only workload. Only draccus and milano are cited as inputs; `lagrange-torch-mlu-insula` is named in `AGENTS.md` but nothing in RTO reads it [I].

### 4.6 Value and status in one line

The lab has produced real, source-corrected knowledge about the Eurus Agent (dead collectors, silent failure modes, a 30-key heartbeat) and a working, digest-stamped launcher. It has not closed its own loop: two known harness bugs persist, RDMA cannot be measured inside the capsule, fixtures were not replaced by captured shapes, and the capsule design is shared with no other repo.

---

## 5. `docs/research/*`: what each note is, and whether to trust it

Rule: research notes are "evidence but not authority" (`AGENTS.md:12`); they were written by parallel agent sessions and may be untracked (`AGENTS.md:123`). No file records a human review of any of them [I].

| # | Note | Tracked? | Date | Establishes | Reliability / status | Evidence or authority |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `2026-09-24-lark-observability-grounding.md` (38 lines) | yes | 09-24 | How the Lark "Arnold observability" doc (rev 573) shaped the page; which Arnold docs were used; four page changes (progress proxy, transport lane split, action lane split, correlation frame) | Good but not re-derivable: the Lark doc is not in the repo | evidence about the *provenance of the page's framing*; Arnold docs remain authority |
| 2 | `2026-09-28-arnold-findings-and-downstream-registration.md` (72) | yes | 09-28 | Arnold facts found while building: two ETTR definitions; triplicated effective-stage list; hard-coded tokens in `trace_training_duration.py`; `Stages.Merge` `>=` tie-break; `compareMetrics` `%s` on an int; unwired collectors; which BMQ families are emitted; the withdrawn registration commit; `check_docs_structure.py` failing on clean Arnold master | High for items cited to a file; read against Arnold `756a165` (upstream master then `f3c58fb1b`). I did not re-verify these; `arnold.md` re-checked some (`[O]` items) | evidence about Arnold, explicitly **not** authority (first paragraph says so) |
| 3 | `2026-09-28-draccus-as-reference-for-capsule-layering.md` (52) | yes | 09-28 | What was adopted from / rejected of draccus for the capsule model; term mapping table | Good; read from README/DESIGN.md/lib scripts on 09-28; I did not read draccus | design rationale (inspiration), no authority over anything |
| 4 | `2026-09-28-associated-repos-source-map.md` (355) | **untracked** | 09-28 | Roles of arnold (product), distributed_timeline (producer), ndtl (experimental); Arnold NdTimeline consumer surface (`trace_generator` routes, `NdCompleteEvent`, SQL); producer surface (`MetricPayload`, `DeviceTimerStreamRecord`, `ndtools`, semantic trace schema); ndtl v2 schema | Code paths cited `file:line` in sibling repos; NDTL claims match `ndtl.md`. Checkout table stale (C3). Recommendations partly executed (ADR 0003) | evidence + design recommendation; its "authority model" section became ADR 0003 |
| 5 | `2026-09-28-repo-deep-dive.md` (286) | **untracked** | 09-28 | Architecture and maintenance contract of RTO itself | **Partly stale**: predates the ETTR/metrics lines, namespaces and the collector audit; its line numbers (`sources.mjs:639`, `README.md:63`) no longer match; two "known drift" items are fixed (C2). Still right on the build subset, test taxonomy and heuristic anchor check | evidence about RTO at an earlier commit |
| 6 | `2026-09-28-training-loop-integration-source-map.md` (389) | **untracked** | 09-28 | Four-layer spine milano -> torchetta -> torchtitan -> ndtl; TorchTitan seams (`train.py`, `Trainer` spans, `TITAN_STRUCT_LOGGER_HANDLERS`, run evidence, FR config); `TorchTitanNdtlHandler` and `synd` staging; a "what not to overclaim" list | Detailed and cited; checkout states stale (C3). The "overclaim" list is the most useful part: Milano has no NdTimeline instrumentation, Torchetta's stream-id smoke is not an end-to-end wrapper validation, TorchTitan stable source does not vendor NDTL | evidence + design recommendation |
| 7 | `2026-09-29-controller-repos-source-map.md` (217) | **untracked** | 09-29 | Recommends ultron as "internal controller, knowledge and policy spine" and skaffen as public campaign/evidence governance; neither is a runtime owner | **Contradicted for ultron** (C1). Skaffen claims not re-read by me | recommendation; overtaken |
| p | `patches/0001-...patch` | yes | 09-28 | The withdrawn Arnold README registration, `5f2f9190f` | Faithful record of a commit | artifact |

### 5.1 Contradictions and stale claims

- **C1. Ultron's role.** Note 7 (and `CONTEXT.md` item 7, uncommitted) treat ultron as the internal controller spine. `ultron.md` (this directory, read at `5061e89`) found ultron rewritten on 2026-09-29 (`32be36b`): a scribe/knowledge repo plus a fixture-level dry-run policy model, "not a running controller"; its own CONTEXT says the controller's "final shape is still to be determined". Note 7's own caveat ("largest risk is overstating Ultron") agrees, but its headline recommendation does not. Ultron's agentic-workflow canonical source also moved to corenius that day [R: `ultron.md:11,60-75,320-330`].
- **C2. Deep-dive note versus current files.** It says `CONTEXT.md` still describes the retired scene IR/`structuralReport`, "two lines" and deferred Playwright gates. At HEAD `CONTEXT.md:41-43` already says the workbench, `structuralReport` and `diagrams.mjs` were retired and `:55` says Playwright checks are part of `npm test`; `system.mjs:7` now names `systemviz.mjs`; `lines.mjs` no longer mentions `diagrams.mjs` [X: grep]. Still true: `getLine()` silently falls back to `LINES[0]` (`lines.mjs:902-904`), and unknown chart ids fail soft in `charts.mjs` [R].
- **C3. Checkout tables and commit hashes drift.** Note 4 records ndtl `f7dfaa0` (now `8bc8ba7`, one commit later) and `distributed_timeline` dirty (still true, dirty=7); note 6 records milano `bf6ab34` (now `ed218ae`), torchetta `hardening/04-...` `4343d43` (now branch `hardening/32-b200-gpu-count-runs`, `9322aa1`), torchtitan `843948e9` (unchanged); note 7 records ultron `7bb5fe4` (now `5061e89`) [X: `git rev-parse` in siblings]. All are snapshots; each sibling reference in this directory pins its own commit.
- **C4. Which vehicle for the first real signal.** Note 6 makes TorchTitan plus the NDTL logging-handler seam the primary path; note 7 quotes ultron's worklog that the "old TorchTitan smoke framing is superseded" by `distributed_timeline/experiments/minimal_parallel_ndtimeline`, blocked on B200 availability. Both are presented without one overruling the other [R]. Do not treat either as the plan.
- **C5. "Settled 2026-09-28" contains 09-29 text.** `CONTEXT.md`'s "Multi-repo evidence model (settled 2026-09-28)" list gained items 4-9 (milano, torchetta, torchtitan, ultron, skaffen, note routing) in an **uncommitted** working-tree edit (mtime 09-29 00:21) [X: `git diff CONTEXT.md`]. The committed CONTEXT has three items.
- **C6. Recommendations already executed or not.** Note 4's "medium term: add a source namespaces abstraction" was done (ADR 0003, `aee9dc2`); its "near term: enrich the NDTimeline line with producer background and `distributed_timeline` names" was not (0 producer anchors).
- **C7. `AGENTS.md` claims about milano** (identity triple, Insula boundary): see 4.5.
- **C8. `INSULA.md` versus design acceptance.** Design section 6 criterion 3 says "the Ray actor hypothesis (H17) passes with Ray bound to `::1`"; `INSULA.md` says H17 is an **open item** with 0 actors published [R]. The spike's acceptance list is not fully met.
- **C9. ADR numbering** (film branch): see 6.3.

### 5.2 How to use these notes

Treat notes 2 and 3 as decision records; notes 4 and 6 as **maps to sibling repos, to be re-checked** against current commits and the sibling references in this directory before reuse; note 5 as history; note 7 as superseded for ultron. Do not lift a hash out of any of them.

---

## 6. The `feat/lifecycle-film` branch

### 6.1 What it adds

8 commits, 2026-09-26 06:14-20:49 UTC, on top of `7acdc34` (`git log master..HEAD`) [X]. 33 non-generated files changed, +4876/-80; including generated `dist/*.html` (13 files) and `previews/**` (157 PNGs) the diffstat is 197 files, +44939/-143 [X]. Four ADRs (0003-0006) and a README section per page:

| Page | Files | What it teaches | Data honesty |
| --- | --- | --- | --- |
| `film.html` (ADR 0003) | `film-script.mjs` (283), `film-anim.mjs`, `film-ranks.mjs`, `film-collect.mjs`, `film.mjs`, `film.css` | 97 s scripted lifecycle of one job on a 16-rank TP4 x DP4 mesh over 2 hosts, two synchronized panels (DP x TP mesh; `UnifiedDataCollector` with nine collector cards and sinks). Ends with what collectors see when a replica slows (t=57) and when rank 9 hangs (t=74; FR + py-spy dump rounds t=82..86.5) | `ILLUSTRATIVE = true`; every frame a pure function of `t`; `#t=` deep links; no verdicts, alerts or restarts animated (narrowed from three panels to two the same day) |
| `step.html` (ADR 0004) | `step-model.mjs` (28, compat view), `step-view.mjs`, `step.mjs`, `step.css` | One Megatron-style step kernel by kernel, TP4 x DP4 ZeRO-1; four views (layer stack, tensors per GPU, mesh, rank-0 CUDA streams on NDTimeline `main`/`tp collective`/`dp collective` lanes); sequence-parallel and DP-overlap toggles | timings illustrative; lane and metric names checked against Arnold |
| gallery (ADR 0005), 6 more pages | `par-model.mjs` (861), `parallel.html` template with `{{VAR}}` substitution in `build.mjs` | `ddp`, `fsdp`, `hsdp`, `fsdp-tp`, `hsdp-tp-cp`, `moe-ep` on 16 GPUs from one list scheduler (per-stream FIFO + dependency rule yields FSDP prefetch, CP ring overlap, DP bucket overlap, blocking TP/EP collectives) | durations from one constant set (work ~ 1/(tp*cp); comm ~ bytes*(N-1)/N, x2.5 across hosts): internally consistent, illustrative; `cp ring` and `ep all-to-all` lanes marked illustrative because NDTimeline has none |
| PP pages (ADR 0006), 4 more | in `par-model.mjs` | `pp-dp`, `pp-3d`, `pp-3d-vpp`, `pp-fsdp`; stage x micro-batch granularity; GPipe / 1F1B / interleaved 1F1B via `#sched=`; deadlock detection; GEMM shapes from one `gemmSpecs`, backward derived and checked | as above |

Source anchors added on the branch: `nd-fsdp-views`, `nd-rl-lanes`, `nd-megatron-alltoall` (`rto-film:src/sources.mjs`, +19 lines), all `arnold` by default [R]. `src/systemviz.mjs` was refactored (exports `layoutSystem`, `wrapWords`, `textLines`) so the film reuses the hero geometry; ADR 0003 says the hero SVG output is byte-identical [DOC]. New tests: `film` (8), `film-visual` (4), `par` (5), `par-visual` (2), `pp` (2), `step` (2), `step-visual` (4). `par.test.mjs` and `step.test.mjs` locate Arnold via `RTO_ARNOLD_DIR` or `~/workspace/arnold` **on their own** and read `generator_megatron.go` etc. to check lane/metric names (`rto-film:tests/par.test.mjs:25-34`); they do not go through `tools/checkouts.mjs` or namespaces [R].

### 6.2 State

Worktree `../rto-film` on `feat/lifecycle-film`, clean; the branch is not merged, not rebased and has no remote (the repo has none). I did not run its tests; its README claim that `npm test` covers "overview + film" is [DOC]. Whether the film pages were ever published to the alias is unknown; the skill deploys the whole `dist/`, and the branch's `dist/` has 13 pages [I].

### 6.3 Divergence from master [X]

- The branch is **8 ahead, 15 behind** master (`git log HEAD..master` lists 15 commits). Everything master gained after 2026-09-25 is absent on the branch: NAMESPACES and the bump tool, `tools/`, the ETTR/metrics lines and signal-ledger extension, `AGENTS.md`, the whole `lab/`, `docs/design/`, `docs/research/`, and the `freshness`/`bump:arnold` scripts.
- Files changed on **both** sides since the merge base (guaranteed textual conflicts or generated-file collisions): `README.md`, `CONTEXT.md`, `src/sources.mjs` (both append anchors, mechanical), `dist/index.html`, and 12 `previews/visual-verifier/*.png`. Conflicts in `dist/` and PNGs are resolved by regenerating, not merging [I].
- **ADR number collision (C9):** the branch's `docs/adr/0003-lifecycle-film.md` versus master's `0003-source-namespaces.md`. A merge needs renumbering (branch ADRs become 0004-0007), and references such as "see docs/adr/0003" in the branch's `CONTEXT.md` and README then point at the wrong record [R][I].
- **Semantic gaps on merge:** the branch's `par`/`step` tests bypass `checkouts.mjs`; the branch's README/CONTEXT lack the "presents, never authority" and terminology rules; and the rule "after any page change republish to the alias" would then apply to a `dist/` of 13 pages published with `--override`, changing what the alias serves [I].
- **Scope signal:** the branch moves from "how Arnold observes/acts" to "how PyTorch parallelism works" (DDP/FSDP/HSDP/TP/CP/EP/PP animations). Only lane/metric names tie it to Arnold; the parallelism content is external knowledge (Megatron-LM, ZeRO, PyTorch), listed "as external links, not as Arnold anchors" (`rto-film:docs/adr/0004-inside-one-step.md`). It sits at the edge of RTO's own non-goal of not being an authority [R][I].

### 6.4 Suggested handling (opinion, [I])

Do not merge as-is. If wanted: rebase on master, renumber ADRs, regenerate `dist/` and previews rather than merge them, route the film's Arnold-name checks through `tools/checkouts.mjs`, and decide separately whether the parallelism gallery belongs in this repo or in its own.

---

## 7. Collaboration practice and hazards for agents

### 7.1 What the repo prescribes (`AGENTS.md`) [R]

- Facts before narrative before presentation (change order).
- `npm test` enforces anchors and prints freshness; `RTO_STRICT_PIN=1` for release verification.
- Arnold is the authority: never push to `mlsys/arnold`; Arnold code changes, "if ever needed, stay on local worktree feature branches".
- Never commit `.cache`, `.venv`, `out/`, py-spy output.
- After any page change, rebuild and republish `dist/` to the alias `robust-training-observatory`.
- "Research notes from parallel sessions may be untracked in `docs/research/`; leave them alone" (`:123`).
- Execution boundary: anything that executes Arnold code, GPU tools or py-spy runs as a capsule via `lab/<experiment>/run_insula.sh -- <command>` on `n116-077-207`; `npm test` is plain host Node and "executes no Arnold code" (the tests do **read** the Arnold checkout, but do not execute it).

### 7.2 What the repo actually shows about how agents work in it [X/R]

- All 29 commits are by one git identity (`phi9t`); collaboration is visible only through trailers: `TRAE CLI` (page, ledger, capsule runner: `cd2f805`, `dadf96f`, `eca33e7`) and `Claude Fable 5.1` (namespaces, audit, `AGENTS.md`, terminology, design spike). The capsule commit `dadf96f` was followed by `eca33e7` ("keep harness changes to the Insula env contract"), which changed `harness.py` by +5/-25 lines [X][I that it walked back an over-reach].
- The sibling worktree `../rto-film` is a plain sibling directory, not under a `.worktrees/` convention. There is no `milano.scm`-style tool, no lock, no integration gate. Master has no remote.
- Untracked parallel-session notes are the norm: 4 of 7 research notes and a `CONTEXT.md` edit were uncommitted at read time (section 5). `AGENTS.md` tells agents to leave them alone, which also means nobody owns committing them.
- Contrast with milano (`milano.md`, `scm.py` sections): milano serializes integration and gates each push on the exact commit; RTO has one branch, direct commits and manual publishing.

### 7.3 Hazards

1. **No remote, no backup.** Every RTO commit, both branches and the capsule identity exist only on this host [X].
2. **Shared publish alias.** `--override` on the fixed alias `robust-training-observatory` replaces whatever is live for everyone with the link. Any agent, or a worktree such as `rto-film`, that runs the skill overwrites it. `AGENTS.md` **requires** republishing after any page change, which conflicts with a "publish only on human request" policy; corenius should treat the publish step as human-gated [R][I]. Publishing also needs `bytedcli` auth and network, which most agent runs should not use.
3. **Tracked generated files.** `dist/index.html` and `previews/**` PNGs are tracked; `npm test` rewrites them. `git add -A` after a test run commits binary churn and manufactures merge conflicts (the film branch already collides on 13 files) [R][I].
4. **The anchor test can pass while claims are wrong.** Off-box (no Arnold checkout) span checks skip; on-box they check a substring near `lineStart` (2.4). "Tests pass" is not "claims are accurate". Prose numbers are untested.
5. **Vacuous namespace enforcement.** `producer` and `experimental` policies have zero anchors; the first real use will be the first real test of that machinery.
6. **Pin drift is invisible from Arnold's side** (registration withdrawn by decision). Only `npm run freshness` reveals it, and only if someone runs it. The unused `ndtl` pin is already behind.
7. **Capsule identity gaps** (4.4 items 5-7): editing the launcher or `build_rootfs.sh` does not change `capsule_digest`; `reproducible` is unproven; the probe's provenance check is path-prefix based.
8. **Hostname lock.** `EXPECTED_HOSTNAME = "n116-077-207"` (`insula.py:30`), overridable with `RTO_INSULA_EXPECTED_HOSTNAME`; the capsule refuses to run elsewhere.
9. **Shared-host leakage.** A py-spy dump on the host enumerates every Python process the user can see; the capsule's private pid namespace is the mitigation. Never run `harness.py` outside `run_insula.sh`; never commit `out/` (`.gitignore` already lists it) [R].
10. **Unstaged-edit trap.** The capsule builds the candidate from the staged tree (like milano). Stage before `build capsule` or `run`, and re-run `build capsule` after changing `harness.py`, `README.md` or `probe_insula.sh` (they are in the candidate) [R].
11. **Terminology traps.** In RTO, *sandbox* is reserved for TraeCode CLI; *hermetic* never means "no network"; *Insula* is narrower or wider than milano's depending on the property (4.5). Reusing these words in corenius documents without the mapping will re-create the confusion the table exists to prevent [R].
12. **Stale text that reads as current.** `lab/agent-harness/README.md` "Procedure" and the `harness.py` docstring predate the capsule; research notes 4-7 carry hashes that have already moved.
13. **`getLine()` and `renderChart()` fail soft** (wrong line, empty chart) on unknown ids; add a test when adding an id [R].

---

## 8. Interfaces to sibling repos, open problems, verification

### 8.1 Interfaces (by kind)

Legend: **code** = imports/executes; **data** = shared schema or artifact; **vocab** = shared words; **doc** = documented in RTO notes only; **none**.

| Repo | Relationship to RTO | Kind | Evidence |
| --- | --- | --- | --- |
| **arnold** | The one real dependency. RTO cites 161 anchors at `756a165c4`, checked against `~/workspace/arnold`; the lab **executes** `eurus-agent` from it (pinned, must be clean, `insula.py:286-299`). Arnold holds no reference to RTO (registration withdrawn) and names none of the other repos (`arnold.md:470`) | code (read + execute), one-directional | `sources.mjs:14-20`; `capsule.json` candidate source; `arnold.md` |
| **distributed_timeline** | Namespace `producer` declared, pinned `b43b3e19`, 0 anchors. Named as a terminology partner in `AGENTS.md`. Note 4 maps its producer surface (`MetricPayload`, `DeviceTimerStreamRecord`, `ndtools`, semantic trace schema). Its `scripts/rootfs/selected.json` is the model for RTO's `selected.json` (design doc header) | doc + vocab; no code | `sources.mjs:21-27`; design doc lines 5-8 |
| **ndtl** | Namespace `experimental` (research notes only), 0 anchors, pin stale. The page cites Arnold's `NdCompleteEvent`; ndtl's `OperationEvent`/`StepProgress` are only compared in notes 4 and 6. `ndtl.md` here is the deeper reference | doc | `sources.mjs:28-35`; notes 4, 6 |
| **torchtitan** | "The actual training-loop authority" in `CONTEXT.md` (uncommitted). Note 6 documents the seam (`TITAN_STRUCT_LOGGER_HANDLERS`, `Trainer` spans, run evidence). RTO does not read or run it. The film branch's `pp-fsdp` page says "torchtitan-style" for layout only | doc | note 6; `rto-film:docs/adr/0006` |
| **milano** | The Insula vocabulary and the `InsulaIdentity`/`RootfsPlan`/isolation-probe ideas originate here; the capsule is a design analogue, not a client (4.5). Note 6 describes milano's runtime substrate. No code shared | vocab + design | `AGENTS.md:14-19`; `milano.md:486` |
| **torchetta** | "Production custom-libtorch authority" in notes; verifier of NDTimeline-facing NCCL stream ids. No link in code. Kairo model-ladder is mentioned as a future evidence fixture | doc | note 6; `torchetta.md` |
| **ultron** | Named "internal controller" in note 7 and `CONTEXT.md`; superseded by ultron's own framing (C1). RTO uses none of its code; ultron's `ndtimeline_adapter.py` consumes NDTimeline rows, not RTO artifacts | doc, contested | note 7; `ultron.md` |
| **skaffen** | Named only in `CONTEXT.md` and note 7 (public campaign/evidence governance). No link; not read by me | doc | note 7 |
| **vaso** | **No mention anywhere in RTO** [X: grep of `AGENTS.md`, `README.md`, `CONTEXT.md`, `docs/`, `src/`, lab docs]. Conceptual overlap only: vaso is a bwrap-rootfs development substrate (`vaso/docs/rootfs-container-infra-design.md`) and holds the cross-agent relay protocol corenius imported (`../practice/README.md`). RTO's capsule model (bwrap rootfs, ldconfig driver discovery, path contract) is closer to vaso and draccus than to milano [I] | none in RTO | grep; vaso doc |
| **stratum-observatory** | Stylistic and rigor template (checkpoint/lens/operation mapping; single-file build); not read by me | design origin | `CONTEXT.md:6-7`; `README.md:9` |
| **draccus** (github.com/phi9t/draccus) | Design input for the capsule layering; not read by me | design origin | note 3 |

### 8.2 What corenius should take from RTO (opinion, [I])

1. **The anchor discipline** is directly reusable for reference docs like this one: every claim resolves to path:line at a pinned commit, a tool moves pins, a test fails on drift. The weakness (substring proximity check, no body hash) is cheap to fix by storing a content hash of each span.
2. **The four-property vocabulary** (hermetic, isolated, offline, reproducible, each attached to a different object) is a durable contribution and worth a shared glossary entry, with the milano differences noted.
3. **Capsule = identity of what ran**: a digest that names the layer that changed, evidence stamped with it, and a probe that turns declared properties into checked ones. Milano has the ingredients (`InsulaIdentity`) but no sealed layers; RTO has the layers but a partial identity. Merging the two is an open design task.
4. **Do not copy** the publish-on-every-change rule, tracked generated artifacts, or the untracked-notes convention.

### 8.3 Open problems

- Commit or discard the four untracked notes and the `CONTEXT.md` edit; supersede note 7's ultron headline.
- Give the repo a remote; decide the fate of `feat/lifecycle-film` (rebase, renumber ADRs, or split into its own repo).
- Populate the `producer` namespace with real anchors so the policy machinery is exercised, or delete the empty namespaces. Bump or drop the `ndtl` pin.
- Strengthen `checkAnchor` (span content hash) and test prose numbers where cheap (e.g. extract constants such as 660/30/50/0.5/10).
- Capsule: cover the launcher and build scripts in the identity (or state they are out of identity); pin the base image by digest and pin apt; add a rebuild-and-compare reproducibility gate; make the probe classify by mountinfo as designed; add GPU leases; decide how RDMA can be observed under `--unshare-net` (a documented net-shared run class reported as such, or drop H3 from capsule runs).
- Lab: fix the two harness defects, bind Ray on `::1` for H17, stop `harness.py` from wiping the probe evidence, turn hypotheses into pytest, feed captured record shapes into fixtures labelled harness output, refresh README and docstring.
- Extraction question (design section 7): move launcher and probe into a shared repo once a second experiment needs them; there is currently exactly one experiment.
- No signal from a real TorchTitan or B200 training run flows into RTO; notes 6 and 7 describe the intended path but nothing implements it here.

### 8.4 How to verify this document

All commands are read-only.

```sh
R=/data02/home/philip.yang/workspace/robust-training-observatory
W=/data02/home/philip.yang/workspace

# 1. Commits read
git -C $R rev-parse master                                   # 584d9b8da62b...
git -C $W/rto-film rev-parse HEAD                             # a7a2f4cc90bb...
git -C $R merge-base master feat/lifecycle-film               # 7acdc34893e1...
git -C $R rev-list --left-right --count master...feat/lifecycle-film   # expect 15 8
git -C $R remote -v                                           # empty
git -C $R status --short                                      # M CONTEXT.md + 4 untracked notes

# 2. Namespaces, pins, anchor counts, content counts (no build, no browser)
cd $R
node tools/bump-namespace.mjs --report      # arnold 161 at pin; distributed_timeline 0; ndtl 0, HEAD != pin
node --test tests/sources.test.mjs tests/geometry.test.mjs    # 18 pass
node -e "import('./src/lines.mjs').then(m=>console.log(m.LINES.map(l=>[l.id,l.stages.length,l.stages.reduce((a,s)=>a+s.nodes.length,0)])))"   # 87 nodes total

# 3. Anchor test strength (2.4)
sed -n 56,73p tools/checkouts.mjs

# 4. Capsule identity scope and probe classification (4.4)
sed -n 34,38p lab/agent-harness/rootfs/insula.py              # HARNESS_CANDIDATE_PATHS is 3 paths
sed -n 511,518p lab/agent-harness/rootfs/insula.py            # capsule_digest formula
grep -n "mountinfo" lab/agent-harness/rootfs/probe_insula.sh  # no output
grep -n "apt-get\|pip install" lab/agent-harness/rootfs/build_rootfs.sh
cat lab/agent-harness/rootfs/selected.json                    # base_ref is a tag

# 5. Harness defects persist
grep -n "bootstrap_complete_ranks" lab/agent-harness/harness.py   # called as a method at :165
grep -c raise_for_status lab/agent-harness/harness.py             # 0

# 6. Film branch
git -C $W/rto-film log --oneline master..HEAD | wc -l              # 8
git -C $W/rto-film ls-tree --name-only HEAD docs/adr/              # 0003-lifecycle-film.md ... 0006

# 7. Spot-check page constants against pinned Arnold
grep -n "ValidTimeSecond *=" $W/arnold/megavision/faultdetector/pkg/flightrecorder/hang.go
grep -n "NoDataTimeThreshold *=" $W/arnold/megavision/faultdetector/pkg/flightrecorder/trialdetector.go

# 8. Sibling drift (5.1 C3)
for r in ndtl milano torchetta ultron torchtitan distributed_timeline; do echo $r $(git -C $W/$r rev-parse --short HEAD); done
```

Known limits of this read: I did not run `npm test`, Playwright, `build.mjs`, `run_insula.sh`, `probe`, any Docker or GPU job, so the capsule acceptance behaviours, the deterministic-build claim and the film branch's tests are unverified. I did not read the Lark doc, draccus, skaffen, stratum-observatory, or the full Arnold source behind the page prose (four constants only). The local `out/report.json` I cite is gitignored and may be regenerated or deleted. The untracked notes and the `CONTEXT.md` edit are live files; recheck the hashes in the header before relying on section 5. This repo changed on the day it was read (last commit 02:26 UTC, `.git` touched 03:13), so re-read `git log` first.
