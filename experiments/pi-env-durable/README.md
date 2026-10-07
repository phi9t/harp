# Interrupted controller effect probe

This model-free research probe runs Pi Durable against the real Corenius Rust
SQLite controller Unix socket daemon in storage-fixture mode. It launches no
supervisor, native runtime, live provider or authentication service. It changes
no product code. Source read: Corenius `d7f674829e9844d011869b711d64fcf55313f7f1`,
Pi `ae92585d3b3e5f1e4b123d14a34314d826d8d9f5`.

From the repository root, reproduce with:

```sh
python3 experiments/pi-env-durable/interrupted-effect-probe.py \
  --upstream /path/to/pinned/pi-checkout \
  --pi-environment /path/to/prepared/offline/pi-environment
```

Prerequisites: Linux `/proc` and Unix sockets, Python 3, Node >=22.19 with
`node:sqlite` and type stripping (verified with 24.13.0), Cargo and the existing
locked offline Rust dependency cache. `--upstream` must be a checkout at the exact
Pi commit above. Every copied runtime source and package manifest is compared with `git show` at
that commit. Controller source, Cargo manifest and lock bytes are also checked
against the stated Corenius base before building. The offline environment needs `node_modules`, `package.json`, and
`package-lock.json`. The retained prior research environment is:
`/data02/home/philip.yang/workspace/corenius-research-pi-durable/.scratch/pi-durable-review`.
Its root dependency manifest pins vitest 4.1.11, typebox 1.3.27, diff 8.0.4,
partial-json 0.1.7 and esbuild 0.28.2. Preparing this cache from scratch requires
those dependencies installed with npm; the `@earendil-works/{chord,pi-ai,pi-durable,pi-telemetry}`
links must point to the corresponding `packages/{chord,ai,durable,telemetry}`
paths. The probe copies dependency bytes and copies those four packages directly
from pinned upstream into a new private scratch environment. It remaps package
links, adds `source` export conditions to the AI and telemetry manifests, and
runs Node with `--conditions=source`. It does not edit Pi runtime source.

Build artifacts and copied dependencies stay under `.scratch/pi-effect-probe/`.
The controller is built locally with `cargo build --locked --offline`. Fixture
process environments are constructed explicitly: private HOME/TMPDIR, PATH,
LANG, and newly generated fixture credentials. Provider secrets and supervisor
configuration are not inherited. This is a trusted local fixture, **not an
Insula isolation test**. The worker has no shell tool or real model provider.

Each scenario admits one Attempt and performs one `report_progress` mutation.
Before sending it, the tool uses `api.memo` to persist the complete command,
including a random idempotency key and expected revision. The first worker is
killed with SIGKILL after a proxy has read the real controller's committed reply
but before the proxy sends any reply bytes to Pi. There is no orderly Harness
close on that process. A new OS process opens the same Pi SQLite file and resumes
the original submission. It proposes a new key and intentionally wrong revision;
the durable memo must recover the original command. The tool always calls the
controller on replay; it never caches the receipt locally.

The assertions cover:

- Same task ID, tool invocation ID and command across actual process restart.
- Current authority: exactly one progress event and identical receipt JSON value.
- Revoked authority: operator cancellation between kill and restart causes the
  original saved request to fail with `authentication: revoked attempt authority`; no additional progress event.
- One occupied slot after both the successful tool result and the refused replay.
- Same key with a changed payload is refused. A fresh key at the current revision
  creates a second progress event, demonstrating the need for stable identity.
- All owned child processes and proxy threads stop, and private credential-bearing
  case directories are deleted. Copied source/dependency/build scratch remains.

The default sanitized artifact is
`docs/efforts/pi-env-durable/evidence/interrupted-effect-probe/result.json`;
`--output` overrides it. It includes real replies, effect/slot counts, Pi tool
results, identity and receipt hashes, source provenance, and cleanup outcomes.
Controller logs and databases may contain fixture secrets and are never exported.
The TS worker hashes JSON insertion order; top-level evidence hashes canonical
sorted JSON, so those hash formats are intentionally distinct.

Observed result: both scenarios passed. **Inference for adoption:** Pi Durable
can preserve a controller mutation identity across this lost-acknowledgement
crash window, provided the adapter journals the command before sending it and
re-enters Corenius authority on every recovery. `replay: "safe"` is justified here
by Corenius idempotency; it is not a property of arbitrary external effects.

This does not qualify production adoption, power-loss durability, controller
restart/backup restore, simultaneous workers, a live model, Insula, supervisor
lifecycle, or candidate/Git effects. `report_progress` is an existing transactional
controller effect; this experiment does not establish exactly-once arbitrary
filesystem, network or process effects. Pi submission `done` means its faux turn
settled, including after a refused tool; it does not mean Attempt completion or
capacity release. Recovery was manual process restart in the orchestrator.
