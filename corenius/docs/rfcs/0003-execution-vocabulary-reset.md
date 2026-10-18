# RFC 0003: Fleet, Insula and workload vocabulary reset

Status: captured owner vocabulary; canonical terms adopted in Corenius documentation.
Captured: 2026-10-04 (America/Los_Angeles).
Repository commit read: `20c2cc8926d3c6875afec6986697b11a83ec0b25`.
Source: [complete supplied text](0003-execution-vocabulary-reset-source.md).
Provenance: [source manifest](0003-execution-vocabulary-reset-sources.json).
Glossary: [CONTEXT.md](../../CONTEXT.md).

The owner requested vocabulary capture. This RFC records the supplied product
boundaries; it does not report a completed implementation or sibling-repository migration.
The original source remains intact, including its clipped historical references.
Those references were not independently retrieved or verified in this capture.

## Canonical boundaries

| Term | Meaning and responsibility |
| --- | --- |
| Armada | Fleet management: machine identities, SSH routes, reachability, observed capabilities, health and ownership context |
| Insula | A live bwrap-based hermetic execution instance local to one concrete execution boundary |
| Insula contract | Immutable resolved rootfs, dependency closure, projections and runtime policy |
| rootfs | Frozen base filesystem artifact; a running instance is an Insula |
| dependency bundle | Immutable additional tools, packages and dependencies projected into an Insula |
| workspace projection | Code or project state exposed inside an Insula |
| DevX | Durable developer Insula lifecycle and developer experience |
| Metallica | Cluster placement, resources, GPU/Pod roles, membership and distributed job lifecycle |
| TorchTitan | Training semantics |
| SGLang | Serving semantics |
| tmux | Interactive terminal/session substrate, normally inside a developer Insula |
| bwrap | Linux mechanism that realizes an Insula |

Armada discovers and describes compute. DevX maintains developer Insulas.
Metallica places and coordinates cluster workloads. Insula defines each role's
process-visible execution context. Workload systems define the computation.

An Insula remains local to one execution boundary. A distributed Metallica job
can contain several Insulas across Pods and supporting roles.
DevX tools normally share the durable developer Insula; narrower build, test or
workload instances can use nested Insulas.

## Artifact and runtime identities

```text
rootfs + dependency bundles + workspace projection
       + device/host projections + Insula policy
    -> resolve immutable Insula contract
    -> realize with bwrap
    -> live Insula instance
```

Use an Insula ID for a running instance. Use content digests for its immutable
contract, rootfs and dependency bundles. These identities have different meanings.

Retire `capsule` as a separate architectural runtime noun. Use `Insula contract`
for the resolved definition and `Insula` or `Insula instance` for the live context.
The adjective `capsulated` remains available.
Existing filenames, command flags and historical source terminology remain exact
identifiers until a separately scoped compatibility change occurs.

## Hermeticity and network policy

Hermeticity concerns declared inputs and execution assumptions. Network policy
is an independent contract dimension. A hermetic Insula can use isolated or host
networking when that policy is explicitly admitted.

Use `network-isolated` for the connectivity property. Hermeticity alone establishes
neither hostile-code containment nor complete workload reproducibility.
An existing isolation requirement still needs its own qualification evidence.
This vocabulary change grants no additional mounts, devices, credentials or network access.

Developer Insulas normally host a private tmux server. Batch and cluster roles
can instead run directly supervised process trees. tmux is optional machinery,
not part of the universal Insula definition or distributed job authority.

## Superseded terminology

| Previous wording | Current wording |
| --- | --- |
| Armada execution/runtime | DevX developer lifecycle or Metallica workload execution, according to the actual responsibility |
| Armada owns Insula lifecycle | DevX or Metallica owns the product lifecycle; Insula supplies the execution substrate |
| Armada Cell execution | Record the actual workload owner and Insula role boundaries |
| capsule contract | Insula contract |
| capsule as a live-runtime object | Insula or Insula instance |
| environment as a precise runtime domain object | Insula; ordinary descriptive use of environment remains valid |

Earlier captured RFC sources remain historical evidence. This vocabulary governs
new Corenius design language where those sources assign execution ownership to Armada.

## Application to Corenius

Corenius remains the engineering-team controller. It owns admitted work, team
authority, message delivery, acceptance and recovery records.
DevX, Metallica, Armada and Insula remain distinct product or substrate boundaries.
An execution adapter must name the actual lifecycle owner rather than treating
Armada as a general workload executor.

The root-Insula proposal now uses these terms. Placement of a Corenius team in a
DevX root or nested Insula still requires an explicit deployment design.
No running worker is moved or renamed by this capture.
The retained SQLite baseline, current task contracts and their qualification gates remain in force.
