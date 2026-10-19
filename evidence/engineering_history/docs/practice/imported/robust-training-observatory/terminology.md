## Terminology

Use these words exactly; they are shared with the sibling repos under `~/workspace` (milano,
lagrange-torch-mlu-insula, distributed_timeline). Do not introduce synonyms. When describing a
capsule or a run, state the four properties separately and by name: **hermetic** (dependency
provenance), **isolated** (namespaces and binds), **offline** (no network at run time),
**reproducible** (rootfs digest from a committed build). Conflating them, in particular using
"hermetic" to mean "no network", is the confusion this table exists to prevent.

| Term | Meaning | Not |
| --- | --- | --- |
| **Insula** | The execution boundary in which a capsule runs: the host-side bubblewrap launcher, the namespaces it creates (pid, net with loopback only, private /proc /dev /tmp) and the NVIDIA driver interface it contributes. Capitalized, singular, proper noun: "runs inside the Insula". The Insula is a facility of the host; it holds no code of its own. | container, jail, sandbox, VM |
| **capsule** | The immutable, content-addressed unit that the Insula executes: one **rootfs** + one **candidate** + one **rootfs plan**, identified by the sha256 over their digests and recorded in `capsule.json` (`capsule_digest`, `rootfs_digest`, `candidate_digest`, `plan_digest`, `owner`). Building a capsule needs no GPU; running it needs the Insula. "Build the capsule, run it in the Insula." Same identity triple as milano's `InsulaIdentity`. | environment, image, the Insula itself |
| **rootfs** | The owned filesystem tree under `lab/<experiment>/.cache/rootfs-store/rootfs-<sha256>`, built reproducibly by a committed script and chosen by `rootfs/selected.json` (`digest`, `owner`, `rootfs_path`, `schema_version`, `store_root`). `.cache` is never committed; `selected.json` is. | image, base image (that is only its build input) |
| **bubblewrap / bwrap** | The launcher mechanism that mounts the rootfs and enters namespaces. It is not the boundary itself. | the sandbox |
| **rootfs plan** | The inspectable, serialized list of binds and namespaces that bwrap is given (`rootfs_plan.json`, after milano's `RootfsPlan`); part of the capsule, so a plan change changes the capsule digest. Every bind source must be the rootfs, the capsule's own state, the candidate, or an allowlisted driver path. | bwrap args (that is its encoding) |
| **candidate** | The immutable code under test inside a capsule, bound read-only at run time: here the pinned `eurus-agent` checkout plus the harness, archived from the staged tree like milano does. | source dir, repo mount |
| **driver interface** | The only host contribution to an Insula: NVIDIA driver libraries, `nvidia-smi`, `/dev/nvidia*`, and network configuration files, per milano's allowlist. | host libs, host PATH |
| **run** / **evidence** | One execution of a capsule in the Insula with a command: `capsule × command → evidence`. Evidence (`out/report.json`, logs) leaves through the mounted `out/` and carries the capsule digest so it can be tied back to exactly what ran. | result, output |
| **harness** | The Python driver of an experiment (`lab/<experiment>/harness.py`), part of the candidate. Host-agnostic: it does not know whether it runs inside the Insula. | runner, launcher (that is `run_insula.sh`) |
| **hermetic** | A property of the **rootfs / capsule**: every toolchain, interpreter, package and dependency the run uses comes from the rootfs; nothing resolves from the host except the driver interface. Hermeticity is about *provenance of dependencies*, verified by inspecting what the runtime resolves (milano's `isolation probe`). It says nothing about the network. "The capsule is hermetic" is a statement about where its bytes came from. | network isolation, offline, air-gapped |
| **isolation** | A property of the **rootfs plan**: which namespaces are private (pid, net, mount, ipc) and which host paths are bound. Isolation is enforced by the Insula at launch; hermeticity is established when the rootfs is built. A capsule can be isolated but not hermetic (host paths bound in) or hermetic but weakly isolated (shares the host pid namespace). | hermetic, sandboxing |
| **offline** | A property of a **run**: the plan gives no network beyond loopback, so nothing is fetched at run time. A hermetic capsule is what makes an offline run possible; the two are still separate facts and are reported separately. | hermetic |
| **reproducible** | A property of the **rootfs build**: the committed build script, its pinned base image and pinned package versions produce the same `rootfs_digest`; the digest in `selected.json` is the proof. | deterministic (fine for the harness code), stable |
| **sandbox** | Reserved for TraeCode CLI's own process sandbox (`traecli sandbox`). Never used for the Insula, the capsule or the plan. | — |
| **fake** | A stand-in for a service the host lacks (local metrics agent, cgroup API, Merlin, Lark, Housekeeper), always named in the harness report. Production code paths are never faked. | mock (fine in test code, not in prose), stub |
| **fixture** | Hand-built or harness-captured data that drives a chart or a collector input, always labeled illustrative or harness output, never production telemetry. | sample data, real data |
| **namespace** (source) | A cited repository with its own pinned commit and evidence policy in `NAMESPACES` (`product` Arnold, `producer` distributed_timeline, `experimental` ndtl). | repo link, upstream |
| **anchor** / **source id** | One entry in `SOURCES`: path, line span, symbol, namespace. Links are built from it, never hand-typed. | citation, reference |
| **pin** | A namespace's commit. It moves only through `tools/bump-namespace.mjs`. | version, HEAD |
| **line / stage / node** | The page's content model: a pipeline, an ordered step in it, one card. | section, chapter |
| **signal ledger** | The folded inventory of collected signals, each with owner, cadence, **wiring** gate, sink and sources. | metrics list |
| **wiring** | The condition under which a collector is actually registered on a node (always / device / management level / Ray role / feature flag / heartbeat-created / never). | enabled |
| **layer** | One content-addressed, read-only-at-run component of a capsule, owned by one build step and ordered by how fast it changes: base rootfs, spack foundation, uv overlay, candidate, plus any future layer (data, weights, config). See *Capsule anatomy*. | image layer (Docker's, which we do not use), venv |
| **base rootfs** / **spack foundation** / **uv overlay** / **state** | The named layers of a capsule and its one writable area; see *Capsule anatomy* below. | overlay (as a Docker term), site-packages |
| **layer build** | A run whose only writable mount is the layer being produced, sealed by its digest on success. Cost is paid once per layer change, never per run. | rebuild the image |

### Capsule anatomy: layers ordered by change velocity

A capsule is an ordered stack of content-addressed layers plus the host's driver interface. Layers
are ordered by how often they change, slowest at the bottom, so that a change costs exactly the
rebuild of its own layer and nothing below it. Each layer has one owner, one build step, one
digest, and is read-only at run time. The model borrows draccus's "path contract enforced by a
namespace", its Spack-versus-uv split, and its do-not-shadow rule (see
`docs/research/2026-09-28-draccus-as-reference-for-capsule-layering.md`); it differs in that every
layer is sealed and content-addressed, and nothing is installed during a run.

| Layer | Name | Contents | Changes | Built by | Mounted at run | Digest |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | **base rootfs** | Distro userland, system interpreter, apt tools (ethtool, iproute2, procps, py-spy if packaged) from a pinned image export, copied into the owned store. | rarely (base image bump) | host-side `rootfs/build_rootfs.sh` | read-only at `/` | `rootfs_digest` |
| 1 | **spack foundation** | ABI-coupled, hermetically built dependencies for training infra: CUDA/NCCL-facing libs, torch and friends, native tools. This is where hermetic native packages are built up for the training infra, and it stays that way. | slowly (weeks) | a layer build inside the Insula, writable only at its own store dir | read-only at `/run/insula/layers/spack` (flat view) | `spack_digest` |
| 2 | **uv overlay** | Fast-moving Python packages: the agent's pip deps, ray, pytest, internal-index packages. Built from a committed `uv.lock` against the interpreter below it, with wheel hashes recorded. | often (days) | a layer build inside the Insula from the lock, offline once the wheel cache is warm | read-only at `/run/insula/layers/uv` (a venv with `--system-site-packages` over the layers below) | `uv_digest` |
| 3 | **candidate** | The code under test, archived from the staged tree. | every run | `git archive` on the host | read-only at `/workspace/candidate/<name>` | `candidate_digest` |
| n | *(future layers)* | Data, model weights, configs, fixtures that a run needs and that have their own velocity and owner. Added as layers, never smuggled into state or the candidate. | as needed | their own build | `/run/insula/layers/<name>` | `<name>_digest` |
| — | **state** | The only writable area: evidence dir, scratch, caches. Never part of identity; evidence carries the capsule digest. | per run | — | read-write at `/run/insula/out`, `/tmp` | none |
| — | **driver interface** | NVIDIA driver libs discovered via `ldconfig -p`, `/dev/nvidia*`, `/dev/nvidia-caps`, `/dev/infiniband`, host `nvidia-smi` at `/run/insula/driver/bin/nvidia-smi`, network config files. | with the host | the host | read-only, allowlisted | in `plan_digest` |

Rules that follow from the table:

- **Path contract.** Inside the capsule, paths are canonical and independent of where anything sits
  on the host: `/` is the base rootfs, `/run/insula/layers/<layer>` each sealed layer,
  `/workspace/candidate/…` the candidate, `/run/insula/out` evidence, `/run/insula/driver` the
  driver interface. Scripts and reports use only these.
- **Change costs its own layer.** A new pip dependency rebuilds the uv overlay (cheap, lock-driven);
  a native library rebuilds the spack foundation (expensive, rare); a base image bump rebuilds the
  base rootfs. Nothing forces a rebuild of a layer below the one that changed.
- **Hermetic is per layer, not per blob.** A layer is hermetic when everything in it resolves from
  itself, the layers below it, or the driver interface, and its build inputs (lock, index, wheel
  hashes, spack concretization) are recorded with its digest. The capsule is hermetic when every
  layer is. Being baked into one filesystem was never the point.
- **No installs at run time.** Layers are built by layer builds and sealed; a run mounts them
  read-only. `pip`/`uv` inside a run fail fast. Layer builds are the only runs allowed a writable
  layer mount, and only for the layer they produce.
- **Do-not-shadow across layers.** A higher layer must not provide what a lower layer already
  provides (the uv overlay must not install the interpreter, torch or numpy that spack owns; the
  candidate must not vendor anything the overlay owns). The Insula probe classifies every resolved
  module and shared object by the mount it lives on and fails on shadowing or host leakage; that
  probe is the operational meaning of "hermetic".
- **Digests compose in stack order.** `capsule_digest = sha256(rootfs ‖ spack ‖ uv ‖ candidate ‖
  plan)` with future layers inserted at their position. A diff of two capsules therefore names the
  layer that moved.

What we deliberately do not take from draccus: writable bundle state under `/opt/...` at run
time, mutable per-project venvs edited in place, and interactive shells as a first-class mode.
Those serve a development environment; a capsule serves a sealed, replayable experiment. For the
current agent-harness capsule the spack foundation layer is empty: the harness needs no native
ML deps, so the stack is base rootfs → uv overlay → candidate, and the slot stays reserved.

