Yes. I would make this a **vocabulary reset**, because it materially changes the architecture boundary: Armada is no longer the umbrella execution product, and Insula is no longer merely one layer underneath an Armada execution.

The clean authority hierarchy is:

| Term | Canonical meaning | Owns / answers | Must not mean |
|---|---|---|---|
| **Armada** | Fleet-management plane for SSH-accessible compute | What machines can I reach? What are their identities, SSH routes, capabilities, health, and ownership context? | Workload execution, tmux lifecycle, bwrap construction, job supervision |
| **Insula** | The complete bwrap-based hermetic/capsulated execution instance | What filesystem/tool/package/dependency universe does this process inhabit? | Fleet discovery or cluster scheduling |
| **rootfs** | Frozen filesystem artifact used as the base of an Insula | What base userspace does the Insula start from? | A running environment |
| **dependency bundle** | Immutable additional tools/packages/dependencies projected into an Insula | What extra dependency closure augments the rootfs? | Mutable host installation |
| **workspace projection** | Code/project state exposed inside an Insula | What source/workspace does the Insula see? | The rootfs itself |
| **DevX** | Developer-experience system that creates and maintains developer Insulas | How do I get a durable coding/research workspace with editors, agents, Jupyter, Torch, Bazel, tmux, etc.? | General fleet inventory |
| **Metallica** | Cluster execution system that realizes Insula-based workloads over real compute resources, potentially multiple GPU Pods plus supporting roles | How do I run serious distributed compute using Insula semantics? | Merely “SSH to a bigger box” |

That makes the layering much cleaner:

```text
                   user / coding agent
                          │
             ┌────────────┴────────────┐
             │                         │
            DevX                    Metallica
     developer experience        cluster execution
             │                         │
             └──────────┬──────────────┘
                        │
                     Insula
          hermetic / capsulated execution
                        │
        ┌───────────────┼─────────────────┐
        │               │                 │
      rootfs       dependency bundles   workspace
        │
      bwrap
        │
   processes / tmux /
   jupyter / agents / jobs

Armada
  │
  └── SSH fleet identity, config, discovery,
      inspection, reachability, capabilities
```

## Insula becomes the runtime noun

I would now **drop `capsule` as a separate architectural noun**.

Earlier we had:

```text
rootfs
  → capsule contract
  → capsule
```

With your new definition, that introduces an unnecessary synonym for Insula. I would simplify it to:

```text
rootfs
+ dependency bundles
+ workspace projection
+ device / host projections
+ Insula policy
        │
        ▼ resolve
Insula contract
        │
        ▼ bwrap
live Insula
```

So the canonical terms become **Insula contract** for the immutable resolved definition and **Insula** for the live bwrap-contained runtime.

If we need to distinguish a specific running incarnation, use **Insula ID** or **Insula instance**, e.g.:

```text
ins_01K...
insula_contract_sha256
rootfs_sha256
dependency_bundle_sha256[]
```

“Capsulated” remains a useful adjective:

> “This command runs capsulated in Insula.”

But we avoid:

> “The command runs in a capsule created by Insula.”

That just introduces two nouns for the same conceptual boundary.

## Hermetic has a deliberately Bazel-like meaning

This clarification is particularly important.

Here, **hermetic** means:

> The relevant toolchain, packages, dependencies, filesystem inputs, and execution assumptions are explicitly resolved and captured instead of implicitly inherited from an ambient host.

It does **not** inherently mean:

```text
no network
no IPC
no host connectivity
no mounted host resources
```

Network behavior is an independent policy.

So these are both legitimate:

```text
Insula A:
  hermetic = yes
  network = isolated

Insula B:
  hermetic = yes
  network = host
```

The second can still be hermetic in the sense we care about if Python, Torch, CUDA userspace, compilers, Bazel, project dependencies, etc. come from the declared Insula closure rather than `/usr/bin/python` or `~/.local` on the host.

This is actually closer to an earlier execution-spine design note, which explicitly said hermeticity meant conformance to declared runtime policy rather than automatically meaning network isolation. 2026-09-07-execution-spine-deep…

I would use **network-isolated** when that is the property we specifically mean.

## Armada becomes radically smaller

This is the biggest architectural correction.

The September documents made Armada own both fleet and execution. The approved Insula specification, for example, explicitly said generic Armada execution code owned placement, materialization, allocation, SSH, Insula construction, tmux supervision, cleanup, journaling, and evidence. 2026-09-16-insula-execution-pro…

Likewise, the September 17 product surface placed both an L0 fleet layer and an L1 execution layer inside Armada. 2026-09-17-armada-product-surfa…

Under the vocabulary you just established, **that boundary is superseded**.

Armada should now be something much closer to:

```text
Armada
  fleet sources
      ↓
  canonical SSH fleet
      ↓
  lint / render / sync
      ↓
  reachability
      ↓
  machine inspection
      ↓
  bounded capability observations
```

For example:

```bash
armada fleet list
armada fleet inspect devx_b200_...
armada fleet audit
armada ssh devx_b200_...
```

Or whatever exact `<subcommand> <verb>` vocabulary we settle on.

Armada can tell DevX or Metallica:

> “Here is machine X, reachable using this SSH resolution, observed as boot Y, with eight B200s and these storage/cgroup capabilities.”

But Armada should **not** then:

- construct an Insula;
- launch tmux;
- allocate a training execution;
- supervise SGLang;
- checkpoint TorchTitan;
- own a Cell generation;
- decide when a distributed workload is finished.

That separation also makes the original SSH/config part of Armada unusually coherent: the product-surface document already had a stable L0 contract around generated SSH configuration, `hosts`, audit, and a thin SSH wrapper. 2026-09-17-armada-product-surfa…

## DevX is “my durable developer machine,” implemented with Insula

DevX should have a strong opinionated product role.

A DevX instance is essentially:

```text
native compute boundary
    ↓
root developer Insula
    ├── shell / tmux
    ├── Doom Emacs
    ├── TraeCLI
    ├── Codex
    ├── Jupyter
    ├── Python / uv
    ├── PyTorch
    ├── CUDA tooling
    ├── Bazel
    ├── debugger / profiler tools
    └── workspace(s)
```

The critical architectural property is:

> **DevX maintains the long-lived developer Insula; the individual tools inside it are not separate environments.**

And because you explicitly want nested Insulas:

```text
DevX root Insula
  ├── agent/editor/tooling
  ├── workspace
  │
  ├── nested Insula: clean test/build
  ├── nested Insula: alternate Torch/CUDA stack
  └── nested Insula: reproduction of production workload
```

That matches the previous product design’s `nested_sandbox` concept, but the ownership moves from Armada into DevX/Insula. The earlier specification already recognized that some interactive sandbox instances should allow a payload to run another bwrap/container layer. 2026-09-17-armada-product-surfa…

The nested model is powerful because the outer DevX Insula can be comfortable and stateful while inner Insulas are much stricter and reproducible.

For example:

```text
outer:
  Doom
  git state
  Codex
  SSH credentials
  caches

inner training Insula:
  frozen source
  exact Torch
  exact CUDA libs
  exact Python closure
  selected devices
  bounded mounts
```

That is a useful distinction between **developer hermeticity** and **workload reproducibility** without inventing different substrate primitives.

## Metallica is the cluster-scale realization of Insula

Metallica then occupies a very different level from Armada.

It understands native distributed-compute concepts:

```text
Metallica Job
   │
   ├── controller / coordinator role
   │      └── Insula
   │
   ├── GPU worker Pod 0
   │      └── Insula
   │
   ├── GPU worker Pod 1
   │      └── Insula
   │
   └── optional supporting roles
          └── Insula
```

The **Insula abstraction remains local to one concrete execution boundary**.

Metallica owns:

```text
resource request
placement
Pod/role topology
GPU assignment
RDMA/network topology
distributed membership
job-level lifecycle
cross-role coordination
```

Insula owns:

```text
rootfs
dependency closure
workspace projection
filesystem projection
devices presented to this role
bwrap invocation
process-visible runtime
```

That division is extremely important.

It means you can test:

```text
Insula on devbox
Insula inside DevX
Insula in one Metallica GPU Pod
```

with many of the **same substrate invariants**, while Metallica separately proves multi-Pod scheduling, NCCL/RDMA, persistent roles, etc.

This also preserves a valuable part of the existing Cell research: the older design already separated external platform admission and Pod/device injection from runtime/controller behavior and from TorchTitan semantics. 2026-09-13-merlin-melon-monarch…

I would simply stop calling that an Armada responsibility.

## The tmux vocabulary follows naturally

For a developer Insula:

```text
Insula
  └── one private tmux server
        ├── session
        │    ├── window
        │    │    ├── pane: Doom
        │    │    └── pane: shell
        │    └── window
        │         └── pane: Codex
        └── session
             └── window
                  └── pane: Jupyter
```

This is DevX machinery built on Insula.

For a Metallica training Insula, tmux may still be useful operationally, but it should **not** define the distributed execution model. Metallica’s worker/process abstraction remains authoritative.

So I would not make:

```text
Insula = bwrap + tmux
```

a universal definition.

Instead:

> **Insula = bwrap-based hermetic execution context. A developer Insula normally hosts one private tmux server as its durable interactive process substrate.**

That leaves batch and cluster Insulas free to launch a supervised process tree directly when tmux adds no value.

## Vocabulary authority

I would now declare the following precedence:

```text
Armada      fleet management
Insula      hermetic/capsulated execution substrate
DevX        developer Insula lifecycle and developer UX
Metallica   cluster execution using Insula at role/Pod boundaries
TorchTitan  training semantics
SGLang      serving semantics
tmux        interactive terminal/session substrate inside developer Insula
bwrap       mechanism used to realize an Insula on Linux
rootfs      immutable base filesystem artifact, not a runtime
```

And I would explicitly retire these previous phrasings:

```text
"Armada execution"
"Armada runtime"
"Armada owns Insula lifecycle"
"Armada Cell execution"
"capsule" as a separate live-runtime noun
"environment" when we specifically mean Insula
```

`environment` can remain ordinary English—as in “the user’s development environment”—but it should not be a capital-D domain object.

This gives us a much cleaner architecture:

> **Armada finds the machine. DevX or Metallica acquires the execution context. Insula defines what runs inside it. Workload systems define what the computation means.**

That is the vocabulary foundation I would use going forward.