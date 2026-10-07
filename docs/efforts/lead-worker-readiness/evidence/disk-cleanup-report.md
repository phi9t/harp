# Disk cleanup report

Worker: disk-cleanup scratch session.

Started: `Mon Oct  5 19:28:04 UTC 2026`

Finished: `Mon Oct  5 19:37:45 UTC 2026`

Reference method: live-process checks used `/proc/*/cwd` snapshots and `ps -eo ... cmd`; I did not run recursive `lsof +D`. Raw inventories and deletion logs are in `/data02/home/philip.yang/devx/tmp/disk-cleanup/`.

## Inventory

| Path | Size | Last modified | What it is | Owner / effort | Live process check |
| --- | ---: | --- | --- | --- | --- |
| `/data02/home/philip.yang` | 2.5T | 2026-10-05 19:37:27 | home volume subtree | philip.yang | yes: cwd=243 ps=175 |
| `/data02/home/philip.yang/.vaso-estate-cuda1291` | 1.1T | 2026-10-04 03:00:02 | CUDA/rootfs/build estate | vaso/cuda1291 effort | yes: cwd=0 ps=13 |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/vaso/lines` | 630G | 2026-09-30 19:29:31 | line/capsule content store | vaso/cuda1291 | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/vaso/cache` | 58G | 2026-09-30 15:11:24 | vaso cache | vaso/cuda1291 | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/vaso/run` | 49G | 2026-10-05 15:32:11 | vaso run outputs | vaso/cuda1291 | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/agents` | 135G | 2026-10-05 13:05:37 | agent work areas | vaso/cuda1291 agents | yes: cwd=0 ps=11 |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/agents/trae` | 44G | 2026-10-05 13:52:55 | Trae agent state | vaso/cuda1291 agents | yes: cwd=0 ps=2 |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/agents/lead` | 34G | 2026-10-05 01:49:08 | lead agent state | vaso/cuda1291 agents | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/rootfs-lines` | 96G | 2026-10-05 01:03:33 | rootfs line images | vaso/cuda1291 rootfs | yes: cwd=0 ps=2 |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/rootfs-lines/cu129.unpinned-20261005` | 25G | 2026-10-04 02:47:45 | rootfs line | vaso/cuda1291 rootfs | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/rootfs-lines/cu129` | 25G | 2026-10-05 01:03:25 | rootfs line | vaso/cuda1291 rootfs | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/rootfs-lines/cu130.unpinned-20261005` | 20G | 2026-10-04 02:47:08 | rootfs line | vaso/cuda1291 rootfs | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/rootfs-lines/cu130` | 20G | 2026-10-05 01:02:05 | rootfs line | vaso/cuda1291 rootfs | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/run` | 94G | 2026-10-05 13:15:04 | probe/debug run outputs | vaso/cuda1291 | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/rootfs` | 11G | 2026-09-25 09:46:29 | rootfs tree | vaso/cuda1291 rootfs | yes: cwd=0 ps=2 |
| `/data02/home/philip.yang/workspace` | 782G | 2026-10-05 19:37:02 | workspace checkouts and worktrees | multiple repos/agents | yes: cwd=214 ps=64 |
| `/data02/home/philip.yang/workspace/monarch` | 178G | 2026-09-22 21:14:48 | repo checkout plus worktrees/rootfs cache | monarch | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/monarch/scripts/rootfs/cache/target` | 38G | 2026-08-17 08:29:38 | Rust/build target cache | monarch rootfs tooling | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/torchtitan` | 151G | 2026-09-23 02:54:37 | repo checkout/experiments/venv | torchtitan | yes: cwd=2 ps=1 |
| `/data02/home/philip.yang/workspace/torchtitan/.venv` | 13G | 2026-08-07 04:52:49 | Python virtualenv | torchtitan | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/soma` | 83G | 2026-09-29 22:55:55 | repo checkout with insula rootfs-store | soma | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/torchetta-release-builds` | 57G | 2026-09-23 06:58:53 | release build workspace/rootfs/cache | torchetta release | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/myriatron` | 30G | 2026-10-05 06:22:19 | repo checkout/worktrees/build targets | myriatron | yes: cwd=15 ps=4 |
| `/data02/home/philip.yang/workspace/myriatron/.trae/worktrees/architecture-deepening/target` | 22G | 2026-10-05 06:19:47 | Rust target build output | myriatron worktree | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/trondheim` | 27G | 2026-10-05 04:06:22 | repo checkout and insula env | trondheim | yes: cwd=12 ps=0 |
| `/data02/home/philip.yang/workspace/verl` | 21G | 2026-10-02 17:43:45 | repo checkout/worktrees/venvs | verl | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/corenius-codex-reference-design` | 21G | 2026-10-03 17:29:28 | git worktree | corenius codex/reference-design | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/corenius-codex-reference-design/crates/team-control-plane/target` | 19G | 2026-10-01 08:07:01 | Cargo target build output | corenius codex/reference-design | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/sglang` | 18G | 2026-09-22 00:19:00 | repo checkout/rootfs/venv | sglang | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-02` | 12G | 2026-10-05 18:39:46 | git worktree | corenius controller bench | yes: cwd=3 ps=0 |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-02/crates/team-control-plane/target` | 8.3G | 2026-10-05 18:47:39 | Cargo target build output | corenius controller bench | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/corenius-sqlite05-control-center` | 12G | 2026-10-04 21:03:48 | git worktree | corenius sqlite05 | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/corenius-sqlite05-control-center/crates/team-control-plane/target` | 7.9G | 2026-10-04 20:59:41 | Cargo target build output | corenius sqlite05 | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/corenius-codex-mailbox-lead/crates/team-control-plane/target` | 7.2G | 2026-10-01 12:25:56 | Cargo target build output | corenius codex/mailbox-lead | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/corenius-codex-temporal-bridge/crates/team-control-plane/target` | 6.4G | 2026-10-01 09:27:18 | Cargo target build output | corenius codex/temporal-bridge | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/workspace/corenius` | 3.2G | 2026-10-05 19:35:02 | main checkout | corenius main | yes: cwd=43 ps=22 |
| `/data02/home/philip.yang/workspace/corenius-lwr-18-interactive-launch-and-monitors` | 217M | 2026-10-05 15:58:18 | protected LWR worktree, not in rule A | corenius LWR 18 | yes: cwd=5 ps=1 |
| `/data02/home/philip.yang/workspace/corenius-lwr-19-safe-on-a-shared-repository` | 217M | 2026-10-05 18:01:55 | allowed number but live, skipped | corenius LWR 19 | yes: cwd=2 ps=1 |
| `/data02/home/philip.yang/workspace/corenius-lwr-21-scenarios-that-can-fail` | 217M | 2026-10-05 18:49:24 | protected LWR worktree, not in rule A | corenius LWR 21 | yes: cwd=4 ps=1 |
| `/data02/home/philip.yang/.cache` | 463G | 2026-10-05 06:36:59 | user cache root | mixed tool/repo caches | yes: cwd=0 ps=27 |
| `/data02/home/philip.yang/.cache/torchetta-live-proof-20261001T210845Z` | 143G | 2026-10-01 21:09:32 | torchetta live proof cache/rootfs/spack | torchetta | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.cache/torchetta-live-proof-20261001T210845Z/spack` | 129G | 2026-10-01 21:11:04 | Spack cache/store | torchetta | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.cache/waystone` | 97G | 2026-10-05 07:24:41 | Waystone cache, explicitly not deleted | waystone/sureal | yes: cwd=0 ps=4 |
| `/data02/home/philip.yang/.cache/waystone/waymo-perception` | 97G | 2026-10-05 09:05:52 | Waymo perception cached data/rootfs | waystone/sureal | yes: cwd=0 ps=4 |
| `/data02/home/philip.yang/.cache/torchetta-live-proof-20261001T045253Z` | 74G | 2026-10-01 04:56:50 | torchetta live proof cache/rootfs/spack | torchetta | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.cache/torchetta-live-proof-20261001T045253Z/spack` | 58G | 2026-10-01 05:44:16 | Spack cache/store | torchetta | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.cache/torchetta` | 56G | 2026-10-01 04:33:26 | torchetta cache/rootfs/spack/tmp | torchetta | yes: cwd=0 ps=2 |
| `/data02/home/philip.yang/.cache/torchetta-release-rehearsal-20261003T083400Z` | 53G | 2026-10-03 08:35:01 | release rehearsal cache | torchetta | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.cache/torchetta/tmp` | 18G | 2026-10-05 15:16:02 | torchetta temp cache | torchetta | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.cache/corenius-landing-audit-20261005` | 11G | 2026-10-05 06:37:06 | landing audit cache | corenius | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.cache/uv` | 8.9G | 2026-10-05 06:41:20 | uv package cache | shared Python tooling | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.cache/bazel` | 8.0G | 2026-09-29 08:09:27 | Bazel cache | shared Bazel tooling | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.cache/corenius` | 65M | 2026-10-05 08:04:51 | Corenius cache root | corenius; readiness preserved | yes: cwd=0 ps=8 |
| `/data02/home/philip.yang/devx` | 31G | 2026-09-10 21:18:40 | developer scratch root | mixed agents | yes: cwd=6 ps=48 |
| `/data02/home/philip.yang/devx/tmp` | 27G | 2026-10-05 19:39:05 | scratch/temp root; partly cleaned | mixed agents | yes: cwd=6 ps=48 |
| `/data02/home/philip.yang/devx/tmp/pytest-of-philip.yang` | 3.4G | 2026-10-05 02:53:18 | pytest temp roots | mixed tests | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/devx/tmp/claude-1018` | 2.2G | 2026-10-05 05:56:11 | Claude temp/session cache | agent tooling | yes: cwd=1 ps=0 |
| `/data02/home/philip.yang/devx/tmp/corenius-temp-policy-20261005` | 1.7G | 2026-10-05 19:27:30 | corenius temp policy scratch | corenius | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/devx/tmp/sqlite05-standards-final-ae4e6fed-wfohrlnm` | 1.4G | 2026-10-04 23:18:01 | sqlite05 review scratch | corenius sqlite05 | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/devx/tmp/sqlite05-standards-final-92429710-flm0zhez` | 1.2G | 2026-10-04 21:58:15 | sqlite05 review scratch | corenius sqlite05 | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.ollama` | 29G | 2026-08-18 16:33:50 | local model weights/cache | Ollama models | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.rustup` | 18G | 2026-05-11 23:20:17 | Rust toolchains | shared Rust tooling | no cwd/ps refs in snapshot |
| `/data02/home/philip.yang/.trae` | 16G | 2026-10-05 18:52:32 | Trae agent/session/plugin state | agent tooling; not deleted | yes: cwd=0 ps=3 |
| `/data02/home/philip.yang/.codex` | 6.2G | 2026-10-05 19:36:04 | Codex agent/session state | agent tooling; not deleted | yes: cwd=0 ps=3 |
| `/tmp` | 53G | 2026-10-05 19:39:05 | tmpfs temp tree; 380431 files before cleanup | mixed active agents | yes: cwd=2 ps=103 |
| `/tmp/corenius-kernel16-managed-handoff` | 16G | 2026-10-01 23:22:00 | /tmp worktree/scratch | corenius kernel16 | no cwd/ps refs in snapshot |
| `/tmp/corenius-target-intent26-outcome-lifecycle1` | 5.9G | 2026-10-03 23:12:54 | /tmp target/build scratch | corenius intent26 | no cwd/ps refs in snapshot |
| `/tmp/corenius-target-intent26-candidate-import2` | 5.9G | 2026-10-04 00:47:29 | /tmp target/build scratch | corenius intent26 | no cwd/ps refs in snapshot |
| `/tmp/corenius-kernel16-capacity-target` | 3.4G | 2026-10-01 23:07:27 | /tmp target/build scratch | corenius kernel16 | no cwd/ps refs in snapshot |
| `/tmp/corenius-land-pending-20261004` | 2.7G | 2026-10-05 05:55:40 | /tmp git worktree | corenius land pending | no cwd/ps refs in snapshot |
| `/tmp/corenius-monitor-wakeup` | 1.5G | 2026-10-04 18:04:55 | /tmp git worktree/scratch | corenius monitor-wakeup | no cwd/ps refs in snapshot |
| `/tmp/sqlite05-repair3-target` | 1.3G | 2026-10-05 01:32:43 | /tmp target/build scratch | corenius sqlite05 | no cwd/ps refs in snapshot |
| `docker system df` | 751.8GB images, 163.8GB containers, 713.8MB build cache | n/a | Docker daemon image/container/build cache accounting | shared container store | 17 active images, 18 running/active containers; prune not run |

Raw inventory files: `du-home-d1.txt`, `du-vaso-cuda-d2.txt`, `du-workspace-d1.txt`, `workspace-builddirs-top.txt`, `du-cache-d2.txt`, `du-devx-tmp-d2.txt`, `tmp-inventory.txt`, `container-df.txt`.

## Deleted

| Path | Size freed | Rule | Check |
| --- | ---: | --- | --- |
| `/data02/home/philip.yang/workspace/corenius-lwr-01-one-scenario-end-to-end` | 929M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-02-exact-candidate-lands` | 215M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-03-acceptance-record` | 215M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-04-rework-round` | 215M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-05-launch-by-edges-and-triage` | 215M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-06-parallel-workers-and-overlap` | 216M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-07-integration-branch` | 215M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-08-launch-access-profile` | 215M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-09-external-state-report` | 215M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-10-watching-limits-accounting` | 217M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-11-handoffs-into-briefs` | 215M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-12-codex-worker-runtime` | 215M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-13-results-table-and-notes` | 214M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-16-bookkeeping-off-code-branch` | 216M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-17-sandboxed-worker-can-commit` | 219M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/workspace/corenius-lwr-20-rework-and-resume-hold` | 217M | A | git worktree remove; git status clean; no cwd/ps reference |
| `/data02/home/philip.yang/devx/tmp/tmp.9aiylt9BLW` | 117M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.E6Xn2vv8FM` | 323M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.EpKOrBLesW` | 117M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.IdW6yWiuls` | 117M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.MD1Xgg8mtd` | 58M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.bXUhgEjFDl` | 117M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.dNM43eKlq1` | 117M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.fQWVpSzHyR` | 117M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.hGCeJBr29e` | 54M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.lrshwWr3a2` | 89M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.o0P7JAMgeC` | 117M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.vZ8ZZ3s0Zy` | 117M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/lead-gate` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/lwr-20` | 264K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/review-ib` | 80K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.0wMsQZ5Pdi` | 31M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.11cxDVNGkg` | 18M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.12DGB4dxRb` | 388K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.1ADSqKoIwz` | 468K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.1OuTialuYx` | 136K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.35MUhDs2ho` | 360K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.3CWuX7weUE` | 40K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.3uV1Q1rxGb` | 144K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.5SFGOt7fV5` | 44K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.5kfYUf9JdM` | 8.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.69DsSDK1BN` | 24K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.6ouOOd9G7C` | 444K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.7ISMjF30Ne` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.95vmbepXjb` | 8.1M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.9fPyv0Xdu7` | 900K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.9wCb3QllNc` | 28K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.9xwhQSKWh7` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.ArLDtHR16a` | 31M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.B5NPbyyZ5v` | 112K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.BMtQPRG6aA` | 424K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.BdA9nxKVY5` | 8.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.C6vZTAosCy` | 136K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.CP5vLiOKw8` | 128K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.DDCg5BBGxF` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.E4nXhUbc7z` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Egjh5mxj8p` | 136K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.EqbwM5881m` | 7.1M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.F0WWcQXPJ6` | 136K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.F5PEbmCGk7` | 84K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.FDRgmyXtLu` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.FdemlAQpRX` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.FsiSWbal2e` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.GcDTFe4qnt` | 392K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.HA4fWfsP1G` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.IF4rq5Hw5z` | 100K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.IGrT2fx0Sr` | 168K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Iae14If3UM` | 24K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.InZH9p1pwP` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.JRnMoSr19V` | 136K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Jj9bvQ02Dr` | 48K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Kgs4SAlqgk` | 392K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.KqhnHXsUBc` | 100K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.L1aEdQnIY7` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.L3A4jL2zMD` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.LGXjzfJw4o` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.N1QwuF58gm` | 44K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.NDPaWUaqdr` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.NnGj4GPqnw` | 8.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.OPYuMtuBYd` | 36K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.OiyrWv5aWH` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.OrcDUDaiYY` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Otz2MPJ8ji` | 48K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.OxaOV60wu2` | 728K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Ozs4kHOwXM` | 124K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.PfrynZuyDs` | 128K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.PwuTgmnUHO` | 7.2M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.QBk9yEXd1U` | 31M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.QMhckHcLV2` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.QtsNNlNyu4` | 40K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.R4GUqLaLLB` | 432K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.RQUEHsMQz6` | 124K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Rcd6w6hkPT` | 228K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.RhRSRPSuKq` | 136K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Sj6AYgcUne` | 432K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.TVUX5HD1Sq` | 428K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.TltK6UgRh4` | 32K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Tsq5YKLsMB` | 100K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.UeIKB9V9Y7` | 12K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.W83hXz3sIN` | 72K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.W9mltaUQCa` | 24K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.XFVzMtk5Bh` | 152K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.Y91TPbUwuq` | 1004K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.YJeyuyWWvI` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.YX2sT9EtN1` | 24K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.ZVA7QS0VcK` | 92K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.ZWTX6TUuPQ` | 732K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.a9yOXa7BPo` | 18M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.aTPRCHi00x` | 416K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.bzxmgbecAy` | 8.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.cRoIUjJJgz` | 440K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.cmjh6zm5de` | 8.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.eJuGkuBxWV` | 424K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.eSAXx20qiG` | 144K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.fVfIIXNp55` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.g81PrV4cq7` | 732K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.ga9FfYuemX` | 428K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.gl5SzQqzDQ` | 72K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.hiyDCmZF0v` | 36K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.iN9cRepj6t` | 18M | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.iNQ7xoDwAg` | 128K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.jXwEc3sCoi` | 100K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.k4bw5NmVDu` | 184K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.kuvt5rVD9X` | 8.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.lzFFTc20sM` | 140K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.m6BuGkWeJF` | 464K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.mENDNqItpo` | 432K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.mqsr02ODuJ` | 24K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.nRmcnAyaDs` | 128K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.nZlDYwWPiZ` | 8.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.osFN7yQbOZ` | 444K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.qT1rlOEOf8` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.qsRAS1Y3gQ` | 124K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.rfeaMMSra3` | 228K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.rjZssU4B7F` | 100K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.scyvb2YD9I` | 176K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.tOSHbVI2fO` | 36K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.tTDBCfdGPg` | 12K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.vQlCIgm9SU` | 128K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.vWglCQnLkV` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.vmheTQxPGR` | 36K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.wap62OdSDm` | 12K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.wvvBuANfLP` | 120K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.xBTw81SNLV` | 228K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.yAyOnZ01LD` | 428K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.yLroEaa1Zq` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.zRWed9yvFw` | 468K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.zaVe58JYqK` | 124K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.zblE0ukD3a` | 136K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp.zhhW3pQtCl` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp11qk3cv0` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp1f59g4jo` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp1jiu61at` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp1ux0zl4a` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp2fibrl17` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp4l36rycr` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp6l6o8gh5` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp6ma2iw34` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp6wz7y91q` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp71b46rwn` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp7yhz3b9m` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp82p8lj3t` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp9kh9lbtt` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmp9r_vm7u6` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpapl3p9f0` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmparvzgn87` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpbabjswgh` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpdqc3kgbk` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpept4be31` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmper67t3p_` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpf4jabpio` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmphz7934k0` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpi95wk_zo` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpic22nrwp` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmplcm6np98` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmplgm3luw4` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpltz_5fzo` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmplwov2dcj` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpmcr2smph` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpn3udo15b` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpnvfr8ogb` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpo1jgkkfx` | 8.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmppq951sp0` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpreq1jqgz` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpsk6lrwcg` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpsxvsnv80` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpte0nb38b` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmptnuvavu7` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpu255y3fw` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpu2700lps` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpukecsrik` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpunsu4_4e` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpv8ucvfpm` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpwif41z_p` | 4.0K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpwwne3y9j` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpyz8bmj4a` | 20K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/devx/tmp/tmpz5twshr1` | 16K | B | devx/tmp allowlist; no cwd/ps reference; age checked where mktemp |
| `/data02/home/philip.yang/workspace/corenius-b1/tools/agents/__pycache__` | 32K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-artifact-acceptance/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-artifact-acceptance/tools/agents/__pycache__` | 184K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-controller-bench-integration/bench/harness/__pycache__` | 84K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-controller-bench-integration/tools/agents/__pycache__` | 64K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-controller-bench-integration/tools/insula/__pycache__` | 56K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-gateway-journal/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-gateway-journal/tools/agents/__pycache__` | 184K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-intent26-contracts/.scratch/team-control-plane/ticket26/__pycache__` | 40K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-jj-preflight/docs/efforts/team-control-plane/research/probes/__pycache__` | 44K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-kata-evaluation/docs/efforts/team-control-plane/research/probes/__pycache__` | 48K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-kernel-proof/docs/efforts/team-control-plane/research/probes/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-mac-operator-dashboard/tools/agents/__pycache__` | 88K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-mac-operator-tmux/tools/agents/__pycache__` | 80K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-mac-runtime-views/tools/agents/__pycache__` | 172K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-mac-state-views/tools/agents/__pycache__` | 140K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-mailbox-lead/crates/team-control-plane/tests/fixtures/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-mailbox-lead/docs/efforts/team-control-plane/research/probes/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-mailbox-lead/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-mailbox-lead/tools/agents/__pycache__` | 184K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-reference-design/docs/efforts/team-control-plane/research/probes/__pycache__` | 108K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-reference-design/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-reference-design/tools/agents/__pycache__` | 184K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-reference-design/tools/insula/__pycache__` | 56K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-sqlite-controller-execution/.scratch/sqlite-controller/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-sqlite-controller-execution/tools/agents/__pycache__` | 104K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-supervised-environments/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-supervised-environments/tools/agents/__pycache__` | 184K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-temporal-bridge/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-codex-temporal-bridge/tools/agents/__pycache__` | 184K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-collab-spike/experiments/collab/__pycache__` | 188K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-collab-spike/experiments/collab/strategies/__pycache__` | 116K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-collab-spike/tools/__pycache__` | 68K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-efforts/tools/__pycache__` | 68K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-efforts/tools/agents/__pycache__` | 100K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-kernel-admission/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-kernel-admission/tools/agents/__pycache__` | 184K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-m1-sync/tools/__pycache__` | 72K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-mb/bench/harness/__pycache__` | 144K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-mb/bench/tmuxh/.venv/lib/python3.13/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-mb/bench/tmuxh/.venv/lib/python3.13/site-packages/libtmux/__pycache__` | 444K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-mb/bench/tmuxh/.venv/lib/python3.13/site-packages/libtmux/_internal/__pycache__` | 76K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-mb/bench/tmuxh/.venv/lib/python3.13/site-packages/libtmux/_vendor/__pycache__` | 32K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-mb/bench/tmuxh/__pycache__` | 76K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-mb/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-mb/tools/agents/__pycache__` | 232K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-mb/tools/insula/__pycache__` | 100K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/.scratch/tmp.5DYTfTvHhz/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/.scratch/tmp.5DYTfTvHhz/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/.scratch/tmp.5DYTfTvHhz/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/.scratch/tmp.Y0rUzipFVU/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/.scratch/tmp.Y0rUzipFVU/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/.scratch/tmp.Y0rUzipFVU/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/harness/__pycache__` | 80K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T002149Z-solo-b-l5-fused-op-traecli-000/workspace/python/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T002757Z-solo-m01-disjoint-utils-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T002757Z-solo-m01-disjoint-utils-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T003002Z-solo-m02-shared-interface-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T003002Z-solo-m02-shared-interface-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T003212Z-solo-m03-same-file-append-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T003212Z-solo-m03-same-file-append-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T003405Z-solo-b-l5-fused-op-traecli-000/workspace/python/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T004149Z-solo-m01-disjoint-utils-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T004149Z-solo-m01-disjoint-utils-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T004445Z-solo-m02-shared-interface-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T004445Z-solo-m02-shared-interface-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T004733Z-solo-m03-same-file-append-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T004733Z-solo-m03-same-file-append-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T004923Z-solo-b-l5-fused-op-traecli-000/workspace/python/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T005549Z-solo-m01-disjoint-utils-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T005549Z-solo-m01-disjoint-utils-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T005758Z-solo-m02-shared-interface-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T005758Z-solo-m02-shared-interface-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T010007Z-solo-m03-same-file-append-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T010007Z-solo-m03-same-file-append-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T010205Z-solo-b-l5-fused-op-traecli-000/workspace/python/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T010814Z-solo-m01-disjoint-utils-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T010814Z-solo-m01-disjoint-utils-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T011018Z-solo-m02-shared-interface-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T011018Z-solo-m02-shared-interface-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T011258Z-solo-m03-same-file-append-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T011258Z-solo-m03-same-file-append-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T011504Z-solo-b-l5-fused-op-traecli-000/workspace/python/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T012242Z-solo-m01-disjoint-utils-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T012242Z-solo-m01-disjoint-utils-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T012440Z-solo-m02-shared-interface-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T012440Z-solo-m02-shared-interface-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T012707Z-solo-m03-same-file-append-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-1/20260930T012707Z-solo-m03-same-file-append-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-hard/20260930T010252Z-solo-mh1-py-inventory-traecli-000/workspace/inventory/__pycache__` | 88K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-hard/20260930T010252Z-solo-mh1-py-inventory-traecli-000/workspace/tests/__pycache__` | 20K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-hard/20260930T014044Z-solo-mh1-py-inventory-traecli-000/workspace/inventory/__pycache__` | 80K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-hard/20260930T014044Z-solo-mh1-py-inventory-traecli-000/workspace/tests/__pycache__` | 20K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-hard/20260930T022001Z-solo-mh1-py-inventory-traecli-000/workspace/inventory/__pycache__` | 76K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/fleet-hard/20260930T022001Z-solo-mh1-py-inventory-traecli-000/workspace/tests/__pycache__` | 20K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000037Z-solo-p01-slugger-traecli-000/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000037Z-solo-p01-slugger-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000037Z-solo-p01-slugger-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000201Z-solo-p01-slugger-traecli-001/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000201Z-solo-p01-slugger-traecli-001/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000201Z-solo-p01-slugger-traecli-001/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000256Z-solo-p02-invoice-cents-traecli-000/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000256Z-solo-p02-invoice-cents-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000256Z-solo-p02-invoice-cents-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000320Z-solo-p01-slugger-traecli-002/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000320Z-solo-p01-slugger-traecli-002/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000320Z-solo-p01-slugger-traecli-002/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000347Z-solo-p03-cli-subcommand-traecli-000/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000347Z-solo-p03-cli-subcommand-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000347Z-solo-p03-cli-subcommand-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000349Z-solo-p02-invoice-cents-traecli-001/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000349Z-solo-p02-invoice-cents-traecli-001/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000349Z-solo-p02-invoice-cents-traecli-001/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000415Z-solo-p04-cache-key-normalization-traecli-000/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000415Z-solo-p04-cache-key-normalization-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000415Z-solo-p04-cache-key-normalization-traecli-000/workspace/tests/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000450Z-solo-p05-offline-wheel-conflict-traecli-000/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000450Z-solo-p05-offline-wheel-conflict-traecli-000/workspace/.venv/lib/python3.11/site-packages/alpha_calc/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000450Z-solo-p05-offline-wheel-conflict-traecli-000/workspace/.venv/lib/python3.11/site-packages/beta_calc/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000450Z-solo-p05-offline-wheel-conflict-traecli-000/workspace/.venv/lib/python3.11/site-packages/shared_math/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000450Z-solo-p05-offline-wheel-conflict-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000503Z-solo-p03-cli-subcommand-traecli-001/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000503Z-solo-p03-cli-subcommand-traecli-001/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000503Z-solo-p03-cli-subcommand-traecli-001/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000517Z-solo-p02-invoice-cents-traecli-002/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000517Z-solo-p02-invoice-cents-traecli-002/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000517Z-solo-p02-invoice-cents-traecli-002/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000530Z-solo-p04-cache-key-normalization-traecli-001/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000530Z-solo-p04-cache-key-normalization-traecli-001/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000530Z-solo-p04-cache-key-normalization-traecli-001/workspace/tests/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000604Z-solo-p03-cli-subcommand-traecli-002/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000604Z-solo-p03-cli-subcommand-traecli-002/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000604Z-solo-p03-cli-subcommand-traecli-002/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000606Z-solo-p06-typing-contract-traecli-000/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000606Z-solo-p06-typing-contract-traecli-000/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000606Z-solo-p06-typing-contract-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000637Z-solo-p05-offline-wheel-conflict-traecli-001/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000637Z-solo-p05-offline-wheel-conflict-traecli-001/workspace/.venv/lib/python3.11/site-packages/alpha_calc/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000637Z-solo-p05-offline-wheel-conflict-traecli-001/workspace/.venv/lib/python3.11/site-packages/beta_calc/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000637Z-solo-p05-offline-wheel-conflict-traecli-001/workspace/.venv/lib/python3.11/site-packages/shared_math/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000637Z-solo-p05-offline-wheel-conflict-traecli-001/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000650Z-solo-p04-cache-key-normalization-traecli-002/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000650Z-solo-p04-cache-key-normalization-traecli-002/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000650Z-solo-p04-cache-key-normalization-traecli-002/workspace/tests/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000716Z-solo-p06-typing-contract-traecli-001/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000716Z-solo-p06-typing-contract-traecli-001/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000716Z-solo-p06-typing-contract-traecli-001/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000758Z-solo-p05-offline-wheel-conflict-traecli-002/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000758Z-solo-p05-offline-wheel-conflict-traecli-002/workspace/.venv/lib/python3.11/site-packages/alpha_calc/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000758Z-solo-p05-offline-wheel-conflict-traecli-002/workspace/.venv/lib/python3.11/site-packages/beta_calc/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000758Z-solo-p05-offline-wheel-conflict-traecli-002/workspace/.venv/lib/python3.11/site-packages/shared_math/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000758Z-solo-p05-offline-wheel-conflict-traecli-002/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000800Z-solo-p06-typing-contract-traecli-002/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000800Z-solo-p06-typing-contract-traecli-002/workspace/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T000800Z-solo-p06-typing-contract-traecli-002/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-1/20260930T001226Z-solo-t03-jsonl-rollup-traecli-001/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-smoke/20260929T234428Z-solo-p01-slugger-traecli-000/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-smoke/20260929T234428Z-solo-p01-slugger-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/bench/results/pilot-smoke/20260929T234428Z-solo-p01-slugger-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/tools/agents/__pycache__` | 100K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-multi-agent-bench/tools/insula/__pycache__` | 52K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-setup-matt-pocock-skills/tools/__pycache__` | 72K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite01-storage/tools/agents/__pycache__` | 44K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite01verify-check/tools/agents/__pycache__` | 44K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite02-runtime/crates/sqlite-controller/support/__pycache__` | 24K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite02-runtime/crates/sqlite-controller/tests/support/__pycache__` | 136K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite02verify-check/crates/sqlite-controller/tests/support/__pycache__` | 244K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite03-kata/.scratch/sqlite-controller/03/native-diagnostic/crates/sqlite-controller/tests/support/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite03-kata/crates/sqlite-controller/tests/support/__pycache__` | 68K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite04-git/.scratch/sqlite-controller/04/repair1/old-selection-10b4d277df384ec8b3f8475878f1bc35/crates/sqlite-controller/tests/support/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite04-git/crates/sqlite-controller/tests/support/__pycache__` | 104K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite05-c1c2-happy/.scratch/sqlite-controller/c1c2-happy/fixtures/__pycache__` | 20K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite05-c1c2-happy/crates/sqlite-controller/tests/support/__pycache__` | 108K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite05-control-center/.scratch/sqlite-controller/05/__pycache__` | 28K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite05-control-center/crates/sqlite-controller/tests/support/__pycache__` | 464K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite05-control-center/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite05-control-center/tools/agents/__pycache__` | 184K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite05-repair3/crates/sqlite-controller/tests/support/__pycache__` | 244K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite06-happy-path/crates/sqlite-controller/tests/support/__pycache__` | 92K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-sqlite06-happy-path/tools/agents/__pycache__` | 96K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/.scratch/live-solo/results/20260930T020745Z-solo-p01-slugger-traecli-000/workspace/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/.scratch/live-solo/results/20260930T020745Z-solo-p01-slugger-traecli-000/workspace/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/.scratch/live-solo/results/20260930T020745Z-solo-p01-slugger-traecli-000/workspace/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/bench/harness/__pycache__` | 144K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/bench/tmuxh/.venv/lib/python3.13/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/bench/tmuxh/.venv/lib/python3.13/site-packages/libtmux/__pycache__` | 444K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/bench/tmuxh/.venv/lib/python3.13/site-packages/libtmux/_internal/__pycache__` | 76K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/bench/tmuxh/.venv/lib/python3.13/site-packages/libtmux/_vendor/__pycache__` | 32K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/bench/tmuxh/__pycache__` | 128K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/tools/agents/__pycache__` | 76K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-tmuxh/tools/insula/__pycache__` | 100K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-01/bench/harness/__pycache__` | 172K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-01/tools/insula/__pycache__` | 56K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-02/bench/harness/__pycache__` | 240K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-02/crates/sqlite-controller/tests/support/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-02/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-02/tools/agents/__pycache__` | 184K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-02/tools/insula/__pycache__` | 56K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-runtime/bench/harness/__pycache__` | 180K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-runtime/crates/sqlite-controller/tests/support/__pycache__` | 140K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-runtime/tools/agents/__pycache__` | 44K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-trae-controller-bench-runtime/tools/insula/__pycache__` | 56K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-c/.scratch/p05-exp-0yerK5/.venv/lib/python3.13/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-c/.scratch/p05-exp-0yerK5/.venv/lib/python3.13/site-packages/alpha_calc/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-c/.scratch/p05-exp-0yerK5/.venv/lib/python3.13/site-packages/beta_calc/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-c/.scratch/p05-exp-0yerK5/.venv/lib/python3.13/site-packages/shared_math/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-c/bench/harness/__pycache__` | 16K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-c/tools/insula/__pycache__` | 48K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-pr/bench/harness/__pycache__` | 16K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-pr/tools/insula/__pycache__` | 48K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-t/bench/harness/__pycache__` | 16K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-author-t/tools/insula/__pycache__` | 48K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-blade-author-1/bench/harness/__pycache__` | 32K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-blade-author-1/tools/insula/__pycache__` | 52K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-blade-author-2/bench/harness/__pycache__` | 32K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-blade-author-2/tools/insula/__pycache__` | 52K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-mb-claims/tools/agents/__pycache__` | 40K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-mb-evidence-2/tools/agents/__pycache__` | 40K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-mb-evidence/tools/agents/__pycache__` | 40K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-mb-msg/tools/agents/__pycache__` | 120K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-mb-tracker/tools/agents/__pycache__` | 160K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-mb-wt-core/tools/agents/__pycache__` | 68K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-mechanism/tools/agents/__pycache__` | 48K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p01-slugger-fixed/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p01-slugger-fixed/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p01-slugger-fixed/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p01-slugger-start/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p01-slugger-start/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p01-slugger-start/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p02-invoice-cents-fixed/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p02-invoice-cents-fixed/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p02-invoice-cents-fixed/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p02-invoice-cents-start/.venv/lib/python3.11/site-packages/__pycache__` | 12K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p02-invoice-cents-start/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius-worker-research-tasks/docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/verified2/p02-invoice-cents-start/tests/__pycache__` | 8.0K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius/tools/__pycache__` | 36K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/data02/home/philip.yang/workspace/corenius/tools/agents/__pycache__` | 260K | D | workspace/corenius* __pycache__; protected/live LWR paths skipped |
| `/tmp/torchetta-release-validate-1791058462673313710` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-1791058724207198903` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-1791059698009432083` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-1791059814344666315` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-abi-sha-1791058462673550965` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-abi-sha-1791058724207359428` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-abi-sha-1791059698009755537` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-abi-sha-1791059814344853873` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-allowlist-1791058462673242184` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-allowlist-1791058724207147252` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-allowlist-1791059698009285214` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-allowlist-1791059814344615423` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-bad-provenance-1791058462674526861` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-bad-provenance-1791058724208959545` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-bad-provenance-1791059698011437513` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-bad-provenance-1791059814345880511` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-cached-libtorch-1791058462673571730` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-cached-libtorch-1791058724207426942` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-cached-libtorch-1791059698010178020` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-cached-libtorch-1791059814344914265` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-consumer-runs-report-only-1791058462675061709` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-consumer-runs-report-only-1791058724209696279` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-consumer-runs-report-only-1791059698012383132` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-consumer-runs-report-only-1791059814346635627` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-desensitize-1791058462674551918` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-desensitize-1791058724209033408` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-desensitize-1791059698011532744` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-desensitize-1791059814345894667` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-distributed-level-1791058462673631456` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-distributed-level-1791058724207583989` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-distributed-level-1791059698010381309` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-distributed-level-1791059814345058095` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-empty-native-consumer-fields-1791058462673715768` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-empty-native-consumer-fields-1791058724207941482` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-empty-native-consumer-fields-1791059698010695044` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-empty-native-consumer-fields-1791059814345115963` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-failing-native-consumer-import-smoke-1791058462673912411` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-failing-native-consumer-import-smoke-1791058724208364733` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-failing-native-consumer-import-smoke-1791059698010959369` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-failing-native-consumer-import-smoke-1791059814345529733` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-flashinfer-source-jit-payload-1791058462672958885` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-flashinfer-source-jit-payload-1791058724206916374` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-flashinfer-source-jit-payload-1791059698008904088` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-flashinfer-source-jit-payload-1791059814344315085` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-fresh-thorough-nightly-1791058462673149797` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-fresh-thorough-nightly-1791058724207031123` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-fresh-thorough-nightly-1791059698009217261` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-fresh-thorough-nightly-1791059814344468803` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-hostile-cache-1791058463022911690` | 4.0K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-hostile-cache-1791058724557013340` | 4.0K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-hostile-cache-1791059698308161443` | 4.0K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-hostile-cache-1791059814698088756` | 4.0K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-hostile-parent-env-1791058463022527259` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-hostile-parent-env-1791058724556648231` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-hostile-parent-env-1791059698307735752` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-hostile-parent-env-1791059814697677165` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-incomplete-native-consumer-1791058462673948084` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-incomplete-native-consumer-1791058724208472019` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-incomplete-native-consumer-1791059698011049710` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-incomplete-native-consumer-1791059814345560580` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-level-field-1791058462673510916` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-level-field-1791058724207303884` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-level-field-1791059698009767531` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-level-field-1791059814344768927` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-native-consumer-flashinfer-arch-list-1791058462674218379` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-native-consumer-flashinfer-arch-list-1791058724208752889` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-native-consumer-flashinfer-arch-list-1791059698011121721` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-native-consumer-flashinfer-arch-list-1791059814345619384` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-provenance-1791058462674311371` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-provenance-1791058724208774542` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-provenance-1791059698011157059` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-provenance-1791059814345653384` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-smoke-probe-summary-1791058462675689622` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-smoke-probe-summary-1791058724210470530` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-smoke-probe-summary-1791059698013587369` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-smoke-probe-summary-1791059814347535635` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-verifier-report-1791058462674533550` | 52K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-verifier-report-1791058724208930501` | 52K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-verifier-report-1791059698011387304` | 52K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-missing-verifier-report-1791059814345823273` | 52K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-native-consumer-1791058462675167683` | 36K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-native-consumer-1791058724209813450` | 36K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-native-consumer-1791059698012563704` | 36K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-native-consumer-1791059814346844202` | 36K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-optional-gpu-run-1791058462672787213` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-optional-gpu-run-1791058724206824584` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-optional-gpu-run-1791059698008633605` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-optional-gpu-run-1791059814344220470` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-optional-gpu-run-first-line-1791058462673375382` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-optional-gpu-run-first-line-1791058724207227611` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-optional-gpu-run-first-line-1791059698009496514` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-optional-gpu-run-first-line-1791059814344712889` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-required-gpu-run-1791058462673857846` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-required-gpu-run-1791058724208065319` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-required-gpu-run-1791059698010721254` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-required-gpu-run-1791059814345305161` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-hash-1791058462673018427` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-hash-1791058724206930075` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-hash-1791059698008968538` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-hash-1791059814344366233` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-required-1791058462675204299` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-required-1791058724209852887` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-required-1791059698012648525` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-required-1791059814346851757` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-warning-1791058462675647538` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-warning-1791058724210412047` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-warning-1791059698013546475` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-rootfs-packages-warning-1791059814347501254` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-short-native-consumer-command-1791058462674594113` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-short-native-consumer-command-1791058724209058610` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-short-native-consumer-command-1791059698011685880` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-short-native-consumer-command-1791059814345947178` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-source-only-native-consumer-1791058462672817164` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-source-only-native-consumer-1791058724206896179` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-source-only-native-consumer-1791059698008835064` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-source-only-native-consumer-1791059814344330930` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-spack-consumer-key-1791058462673200156` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-spack-consumer-key-1791058724207058467` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-spack-consumer-key-1791059698009277145` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-spack-consumer-key-1791059814344520050` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-import-identity-1791058462674655826` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-import-identity-1791058724209098333` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-import-identity-1791059698011852584` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-import-identity-1791059814345993920` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-schema-1791058462674667786` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-schema-1791058724209193112` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-schema-1791059698011851822` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-schema-1791059814346050803` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-source-commit-1791058462674736133` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-source-commit-1791058724209244272` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-source-commit-1791059698012013805` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-source-commit-1791059814346265199` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-source-metadata-1791058462674772229` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-source-metadata-1791058724209298648` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-source-metadata-1791059698012043971` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-native-consumer-source-metadata-1791059814346344600` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-thorough-nightly-1791058462675617576` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-thorough-nightly-1791058724210368626` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-thorough-nightly-1791059698013394086` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-stale-thorough-nightly-1791059814347456139` | 60K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-top-level-native-consumer-source-1791058462675598004` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-top-level-native-consumer-source-1791058724210310273` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-top-level-native-consumer-source-1791059698013328824` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-top-level-native-consumer-source-1791059814347406542` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-unresolved-native-consumer-linkage-1791058462674844193` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-unresolved-native-consumer-linkage-1791058724209471975` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-unresolved-native-consumer-linkage-1791059698012068299` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-unresolved-native-consumer-linkage-1791059814346351933` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-arch-mismatch-1791058462675005435` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-arch-mismatch-1791058724209590196` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-arch-mismatch-1791059698012194416` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-arch-mismatch-1791059814346487188` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-cuda-mismatch-1791058462675044918` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-cuda-mismatch-1791058724209678213` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-cuda-mismatch-1791059698012383291` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-cuda-mismatch-1791059814346533365` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-hash-1791058462674885352` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-hash-1791058724209507407` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-hash-1791059698012177324` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |
| `/tmp/torchetta-release-validate-verifier-report-hash-1791059814346458838` | 56K | E | /tmp torchetta-release-validate; newest file older than 24h; no cwd/ps reference |

Skipped allowlisted candidates: `/data02/home/philip.yang/workspace/corenius-lwr-19-safe-on-a-shared-repository` and `/data02/home/philip.yang/devx/tmp/lwr-19` both had live process references; `.cache/corenius/workers/*` had no eligible old missing-worktree runs; `/tmp/torchetta-release-validate-*` from Oct 5 were newer than 24h and were left.

## Proposed, needs the owner

| Path / store | Size | What it is / how known | Regenerable and expected cost | Exact removal command |
| --- | ---: | --- | --- | --- |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/vaso/lines` | 630G | Largest subtree under `.vaso-estate-cuda1291`; line/capsule content store | Probably regenerable from Vaso workflows but expensive; expect hours and provenance risk | `rm -rf -- /data02/home/philip.yang/.vaso-estate-cuda1291/vaso/lines` |
| `docker images` | 600.3GB reclaimable of 751.8GB images | `docker system df`; shared Docker daemon has 244 images, 17 active | Re-pull/rebuild images; likely minutes to hours and may disrupt running workflows if image assumptions change | `docker image prune -a` |
| `/data02/home/philip.yang/.cache/torchetta-live-proof-20261001T210845Z` | 143G | Torchetta live-proof cache; 129G under `spack` | Regenerable by rebuilding/reinstalling release proof env; likely hours and network/toolchain cost | `rm -rf -- /data02/home/philip.yang/.cache/torchetta-live-proof-20261001T210845Z` |
| `/data02/home/philip.yang/.cache/waystone/waymo-perception` | 97G | Waystone Waymo perception cache; explicitly off-limits to this worker | Regenerable only by data/cache rebuild; likely slow and may be shared by active sureal work | `rm -rf -- /data02/home/philip.yang/.cache/waystone/waymo-perception` |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/rootfs-lines` | 96G | Rootfs line images under Vaso CUDA estate | Regenerable by rootfs build line; likely hours | `rm -rf -- /data02/home/philip.yang/.vaso-estate-cuda1291/rootfs-lines` |
| `/data02/home/philip.yang/.vaso-estate-cuda1291/run` | 94G | Vaso probe/debug run outputs; many 3-5G run dirs | Some outputs may be evidence; regenerate by rerunning probes, minutes to hours each | `rm -rf -- /data02/home/philip.yang/.vaso-estate-cuda1291/run` |
| `/data02/home/philip.yang/.cache/torchetta-live-proof-20261001T045253Z` | 74G | Torchetta live-proof cache/rootfs/spack | Regenerable by rerunning release proof setup; likely hours | `rm -rf -- /data02/home/philip.yang/.cache/torchetta-live-proof-20261001T045253Z` |
| `/data02/home/philip.yang/workspace/torchetta-release-builds` | 57G | Torchetta release build workspace/rootfs/cache | Build outputs can be regenerated; source/evidence may be valuable, owner should inspect first | `rm -rf -- /data02/home/philip.yang/workspace/torchetta-release-builds` |
| `/data02/home/philip.yang/.cache/torchetta` | 56G | Torchetta shared cache; 22G spack, 18G tmp, 9.9G rootfs | Regenerable but likely shared by release work; minutes to hours | `rm -rf -- /data02/home/philip.yang/.cache/torchetta` |
| `/data02/home/philip.yang/.cache/torchetta-release-rehearsal-20261003T083400Z` | 53G | Torchetta release rehearsal cache | Regenerable by rerunning rehearsal; likely hours | `rm -rf -- /data02/home/philip.yang/.cache/torchetta-release-rehearsal-20261003T083400Z` |
| `/data02/home/philip.yang/workspace/monarch/scripts/rootfs/cache/target` | 38G | Rust/build target cache in Monarch rootfs tooling | Regenerable by rebuild; likely tens of minutes to hours | `rm -rf -- /data02/home/philip.yang/workspace/monarch/scripts/rootfs/cache/target` |
| `/data02/home/philip.yang/.ollama` | 29G | Local model weight/cache store | Regenerable by re-pulling models; network and time dependent | `rm -rf -- /data02/home/philip.yang/.ollama` |
| `/data02/home/philip.yang/workspace/myriatron/.trae/worktrees/architecture-deepening/target` | 22G | Cargo target inside Myriatron worktree | Regenerable by cargo build; owner should confirm worktree state first | `rm -rf -- /data02/home/philip.yang/workspace/myriatron/.trae/worktrees/architecture-deepening/target` |
| `/data02/home/philip.yang/workspace/corenius-codex-reference-design/crates/team-control-plane/target` | 19G | Cargo target in Corenius codex/reference-design worktree | Regenerable; do not remove without that effort owner approval | `rm -rf -- /data02/home/philip.yang/workspace/corenius-codex-reference-design/crates/team-control-plane/target` |
| `/data02/home/philip.yang/.rustup` | 18G | Rust toolchain store | Regenerable by rustup reinstall but can break active builds until restored | `rm -rf -- /data02/home/philip.yang/.rustup` |
| `/data02/home/philip.yang/.trae` | 16G | Trae agent/session/plugin store | Not safely regenerable; archive or selective retention review needed | `rm -rf -- /data02/home/philip.yang/.trae` |
| `/data02/home/philip.yang/.cache/corenius-landing-audit-20261005` | 11G | Corenius landing audit cache/targets | Likely regenerable if audit can be rerun; confirm no lead depends on evidence | `rm -rf -- /data02/home/philip.yang/.cache/corenius-landing-audit-20261005` |
| `/tmp/corenius-kernel16-managed-handoff` | 16G tmpfs | `/tmp` worktree/scratch; likely active or retained by another effort | Unknown; inspect git status/processes before removal | `rm -rf -- /tmp/corenius-kernel16-managed-handoff` |
| `/tmp/corenius-target-intent26-outcome-lifecycle1` | 5.9G tmpfs | Corenius intent26 target/build scratch | Regenerable build output, but outside allowlist | `rm -rf -- /tmp/corenius-target-intent26-outcome-lifecycle1` |
| `/tmp/corenius-target-intent26-candidate-import2` | 5.9G tmpfs | Corenius intent26 target/build scratch | Regenerable build output, but outside allowlist | `rm -rf -- /tmp/corenius-target-intent26-candidate-import2` |

## Summary

Free space before: `/data02` 124G free, `/tmp` 547618 free inodes.

Free space after: `/data02` 126G free (`134957146112` bytes available), `/tmp` 552310 free inodes. `/tmp` file count went from 380432 to 375740.

Freed by deletion log size accounting: Rule A 4.065 GiB across 16 inactive clean LWR worktrees; Rule B 1.609 GiB across 181 allowed devx/tmp directories; Rule C 0 GiB because no cache run met both missing-worktree and age predicates; Rule D 0.011 GiB across 239 corenius `__pycache__` directories; Rule E 0.008 GiB across 160 old `/tmp/torchetta-release-validate-*` directories. Total recorded freed: 5.694 GiB. Rounded `df -h` moved from 124G to 126G during concurrent machine activity.

Five largest owner-needed opportunities: `.vaso-estate-cuda1291/vaso/lines` at 630G; Docker reclaimable images at 600.3GB; `.cache/torchetta-live-proof-20261001T210845Z` at 143G; `.cache/waystone/waymo-perception` at 97G; `.vaso-estate-cuda1291/rootfs-lines` at 96G.

Surprises: the safe allowlist was much smaller than the dominant disk users; Docker and Vaso/Torchetta/Waystone stores dominate the volume but require owner approval. The allowed `/tmp` cleanup freed little byte space but recovered about 4692 tmpfs inodes. Ticket 19 LWR paths are still live and were left intact.

DONE
