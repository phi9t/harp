---
title: "Deep Dive Pipeline Parallelism"
source: "https://chatgpt.com/c/6a84ddf3-06dc-83ea-af69-735755a6f185"
author:
published:
created: 2026-08-21
description: "ChatGPT conversation with 5 messages"
tags:
  - "clippings"
---
The cleanest way to understand pipeline parallelism is **not** as a sequence of named schedules. It is a constrained DAG-scheduling problem with four largely independent design choices:

1. **Optimizer semantics:** synchronous minibatch updates versus asynchronous/stale-weight updates.
2. **Stage placement:** contiguous, cyclic/interleaved, V-shaped, or replicated bidirectional pipelines.
3. **Backward granularity:** monolithic backward, or separate input-gradient and weight-gradient work.
4. **Physical overlap:** merely filling idle schedule slots, versus genuinely overlapping GPU compute with communication or another computation.

This framing matters because **Zero Bubble and DualPipe operate at different layers**:

- Zero Bubble primarily relaxes backward dependencies by separating and .
- DualPipe combines that idea with bidirectional placement and a model-specific compute/communication overlap implementation.
- A framework exposing a `DualPipeV` schedule does not, by itself, reproduce DeepSeek’s end-to-end DualPipe efficiency.

---

## 1\. A common cost model

Let:

- : number of physical pipeline ranks.
- : microbatches accumulated per optimizer step.
- : virtual pipeline stages, or model chunks, per physical rank.
- : forward time for the work assigned to one rank.
- : backward-input time, computing .
- : backward-weight time, computing .
- : full backward time.
- : activation state retained for one stage–microbatch pair.

For balanced stages, ignoring communication and kernel-launch overhead, a conventional flush pipeline has

and

Therefore,

and the fraction of wall time attributable to the schedule bubble is

Some papers instead report bubble overhead relative to useful work,

These are different denominators and should not be mixed. GPipe derives the corresponding amortized bubble and observes that it becomes small only once the number of microbatches substantially exceeds pipeline depth. [^1]

The broader real-world model is closer to

A mathematically zero-bubble schedule only eliminates the second term under its timing assumptions.

There is also an optimization constraint:

where is microbatch size and is data-parallel degree. Increasing is not free: at fixed global batch it usually requires reducing the microbatch size, which can worsen GEMM efficiency, or reducing data parallelism.

---

## 2\. The schedule landscape at a glance

| Schedule | Update semantics | Main mechanism | Idealized bubble | Principal cost |
| --- | --- | --- | --- | --- |
| **GPipe** | Synchronous | All forward, then all backward |  | activation residency |
| **1F1B-Flush** | Synchronous | Alternate F/B after warm-up | Same as GPipe | Much lower activation residency |
| **PipeDream** | Asynchronous | Never flush pipeline | No periodic flush bubble | Stale weights and weight versions |
| **Interleaved 1F1B** | Synchronous | cyclic chunks per rank | Approximately divided by | Approximately PP boundaries |
| **Looped BFS** | Synchronous | Same looped placement, stage-first ordering | Similar geometric reduction | Different activation/FSDP trade-off |
| **ZB1P** | Synchronous | Split into and |  | More demanding backward implementation |
| **ZB2P** | Synchronous | Profile-guided F/I/W scheduling | Near zero | Roughly activation budget |
| **ZB-V** | Synchronous | F/I/W split plus V placement | Zero only under matched timings | Two stages/rank; more PP boundaries |
| **Chimera** | Synchronous | Two opposite model replicas | Roughly half conventional bubbles | Two stage copies/rank and replica all-reduce |
| **DualPipe** | Synchronous | Bidirectional streams plus real F/B/comm overlap | Official formula below | Two parameter copies and custom kernels |
| **DualPipeV** | Synchronous | V-folded DualPipe | Same DualPipe schedule formula | More PP communication at fixed device count |

These formulas normalize the GPipe, PipeDream-Flush, Zero Bubble, Chimera, and DeepSeek analyses; they assume balanced stages and omit exposed communication. [^1]

[![GitHub](data:image/png;base64,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)](https://ar5iv.labs.arxiv.org/html/1811.06965)

[GitHub+4 ![](https://www.google.com/s2/favicons?domain=https://ar5iv.labs.arxiv.org&sz=128)](https://ar5iv.labs.arxiv.org/html/1811.06965) 

---

## 3\. Foundational schedules

## 3.1 GPipe: fill–drain

GPipe partitions the model into contiguous stages and divides a minibatch into microbatches. It executes:

1. Forward for all microbatches.
2. Backward for all microbatches.
3. One synchronous optimizer update.

Conceptually:

```
warm-up / fill                      drain
F0 F1 F2 F3 ... Fm-1 | Bm-1 ... B2 B1 B0
```

Its strongest properties are simplicity and clean synchronous semantics: every microbatch in the minibatch uses the same parameter version, and gradients are accumulated before the optimizer update. [^1]

Its main weakness is activation lifetime. The first stage may complete all forward passes before receiving any backward work, so it must retain information for microbatches:

Activation checkpointing reduces the amount retained per microbatch, but not the number of outstanding microbatches. GPipe’s original design relies heavily on rematerialization for this reason. [^1]

### Important misconception

**GPipe and synchronous non-interleaved 1F1B have essentially the same ideal scheduling bubble.** The main advantage of 1F1B is activation memory and earlier gradient readiness, not a fundamentally smaller warm-up/cooldown bubble.

---

## 3.2 Synchronous 1F1B, usually PipeDream-Flush

A modern LLM stack generally means **PipeDream-Flush semantics** when it says “1F1B”:

1. Warm up with several forward microbatches.
2. Enter steady state, alternating one forward and one backward.
3. Drain outstanding backward operations.
4. Update weights only after the pipeline is flushed.
```
warm-up              steady state                  cooldown
F0 F1 F2 ... | B0 Fk B1 Fk+1 B2 Fk+2 ... | B... B...
```

This prioritizes backward work once it becomes available, allowing activation state to be freed earlier. The maximum number of live microbatches is bounded by approximately the pipeline depth rather than the total accumulation count:

PipeDream-Flush retains one parameter version and gives ordinary synchronous minibatch update semantics. The PipeDream-2BW paper explicitly contrasts this with GPipe: both flush, but 1F1B limits activation stashes to in-flight microbatches. [^2]

PyTorch’s current API describes `ScheduleGPipe` as fill–drain and `Schedule1F1B` as one forward and one backward during steady state. [^3]

### Why 1F1B remains the default

It is the best robustness/complexity baseline:

- Straightforward autograd.
- One model version.
- Good activation lifetime.
- Easy composition with tensor parallelism and conventional DDP/FSDP.
- Relatively few pipeline messages.
- No special equality assumptions between , , and .

Unless profiling establishes that the pipeline bubble is a major portion of step time, 1F1B is usually the schedule to beat.

---

## 3.3 Asynchronous PipeDream: remove flushes by changing the algorithm

Original PipeDream removes the periodic flush and keeps a continuous 1F1B pipeline. That can make the pipeline almost continuously occupied, but different microbatches coexist under different weight versions.

PipeDream uses **weight stashing** so a stage’s backward pass uses the same local weight version as its corresponding forward pass. This gives stage-local forward/backward consistency, but the complete model traversed by a microbatch can still comprise different effective versions across stages. Vanilla PipeDream can require up to stashed versions and applies updates under stage-dependent delay. [^2]

PipeDream-2BW reduces this to two versions, with update semantics approximately

rather than ordinary

PipeDream-Flush was introduced precisely to recover the latter semantics by accepting periodic bubbles. [^2]

This gives an essential distinction:

> **Asynchronous pipelines remove bubbles by relaxing parameter consistency. Zero-Bubble schedules try to remove bubbles while retaining synchronous update semantics.**

For frontier pretraining, where exact reproducibility, loss-scale behavior, optimizer state, and scaling-law comparability matter, this distinction is usually more important than the raw utilization number.

---

## 4\. Interleaved and looped placement

## 4.1 Interleaved 1F1B

Instead of assigning one contiguous model partition to each rank, interleaved 1F1B assigns smaller model chunks cyclically:

```
p = 4 ranks, v = 2 chunks/rank

logical stage:  0  1  2  3  4  5  6  7
physical rank:  0  1  2  3  0  1  2  3
```

Each logical chunk contains approximately of the compute that the physical rank would own in non-interleaved 1F1B. Under the balanced idealization,

and

This is why Megatron-style virtual pipeline parallelism is attractive when cannot be made much larger than .

The price is that a microbatch traverses approximately logical boundaries instead of . Most of those boundaries are cross-rank in cyclic placement, so both message count and pipeline communication volume approach a factor of increase. PyTorch calls the standard interleaved priority rule **depth first**: when multiple local stages are ready, it prioritizes the earlier microbatch. [^3]

### When it wins

Interleaved 1F1B is strong when:

- The baseline pipeline is deep.
- The number of microbatches is constrained.
- P2P latency is low.
- Activation tensors are small relative to stage compute.
- The model naturally partitions into many balanced chunks.

### When it loses

It can lose when:

- Network latency dominates small PP messages.
- Stage chunks become too small to run efficient kernels.
- The extra communication contends with TP, FSDP, or EP traffic.
- Virtual stages introduce more activation fragmentation and allocator pressure.
- Load imbalance becomes visible at the finer chunk granularity.

---

## 4.2 Looped Breadth-First Scheduling

Looped BFS uses essentially the same multi-chunk placement but changes the local priority policy.

- **Interleaved depth first:** prioritize the oldest/earliest microbatch and move it deeper through the model.
- **Looped breadth first:** prioritize the earliest logical stage and process all currently available microbatches for that stage.

PyTorch’s description is precise: Looped BFS supports multiple stages per rank, but when several local stages are ready, it prioritizes the earlier stage and runs all available microbatches for it. [^3]

This matters most when combining PP with fully sharded parameter state:

- Unshard one stage’s parameters.
- Run a group of microbatches through it.
- Reshard once.
- Move to the next local stage.

That can amortize FSDP all-gather and reshard operations over many microbatches. It also creates longer windows in which data-parallel communication can overlap computation. The Breadth-First paper’s core result is that looped breadth-first ordering provides better network overlap than looped depth-first ordering, especially at low batch size per GPU. [^4]

### Scrutiny of the reported result

The paper reports up to 43% higher throughput near its minimum batch-per-GPU regime for a 52B model. However:

- Experiments used 64 V100 GPUs.
- The breadth-first implementation and Megatron baselines were not from the same code path.
- The paper explicitly notes that its Megatron baselines did not support some data/pipeline communication overlap and sharded-DP features, potentially underestimating them. [^4]

So the result establishes that **breadth-first scheduling can be valuable**, not that Looped BFS is universally 43% faster than modern interleaved 1F1B.

---

## 5\. Zero Bubble: separating from

## 5.1 The dependency insight

Consider a linear layer

Given upstream gradient ,

and

These two computations have very different pipeline dependencies:

- , the input gradient, must be transmitted to the preceding stage. It is on the pipeline’s backward critical path.
- , the parameter gradient, is local. It only has to finish before gradient synchronization and the optimizer update.

Standard autograd treats them as one backward operation. Zero Bubble splits them into:

- : backward-input or .
- : backward-weight or .

The scheduler executes as soon as possible and moves into otherwise idle slots. The Zero Bubble paper formalizes this dependency relaxation, and PyTorch’s pipeline IR now has distinct `BACKWARD_INPUT` and `BACKWARD_WEIGHT` actions for it. [^5]

This is not free. Delaying means retaining the tensors required for , usually some combination of , , and intermediate state, for longer. PyTorch’s automatic backward partitioner explicitly saves shared values required by the deferred weight-gradient computation. [^6]

---

## 5.2 ZB-H1 and ZB1P: same theoretical activation envelope

Using as full backward time, the official ZB1P bubble expression is

Equivalently,

The second form looks unusual but makes the source of the saving explicit: compared with standard 1F1B, two copies of the weight-gradient-duration contribution disappear from the warm-up/cooldown critical path.

Under the paper’s illustrative assumption

standard 1F1B has

while ZB1P has

Thus ZB1P reduces the schedule bubble to one third under that idealization, but does not eliminate it.

The original handcrafted variant was called ZB-H1. The automatically searched ZB-1p constrains activation memory to approximately , theoretically matching the peak activation envelope of 1F1B. The PyTorch class `ScheduleInterleavedZeroBubble` implements the ZB1P variant and uses deferred weight-gradient work to fill the bubble. [^5]

### Why ZB1P is attractive

- Synchronous optimizer semantics.
- Little or no increase in pipeline communication.
- Activation memory close to 1F1B.
- No requirement to duplicate the model.
- Useful even when is not much larger than .

The Zero Bubble evaluation found ZB-1p particularly favorable against interleaved 1F1B in multi-node configurations because it reduced the bubble without interleaved 1F1B’s extra PP communication. [^5]

---

## 5.3 ZB-H2 and ZB2P: spend activation memory to approach zero

ZB-H2 performs more forward work during warm-up and pushes weight-gradient operations into the tail, transforming the schedule into a nearly bubble-free parallelogram.

Its ideal handcrafted bubble is

Under , this reaches zero, but the original handcrafted schedule requires approximately

of activation memory.

The more practical ZB-2p is an automatically searched schedule with a activation-memory constraint. In the paper’s experiments, this was approximately the least memory budget consistently sufficient to approach zero bubble. It remained efficient at smaller microbatch counts, where 1F1B, interleaved 1F1B, and ZB1P still depended strongly on increasing . [^5]

The paper also verified bitwise-identical iteration losses between its synchronous ZB schedules and 1F1B under fixed seeds. [^5]

---

## 5.4 Why automatic schedule search is important

The handcrafted diagrams generally assume:

and often omit communication. Real transformer stages violate this:

- Attention and MLP have different ratios.
- Activation recomputation changes backward timing.
- Embedding and loss stages are asymmetric.
- MoE routing introduces dynamic variation.
- P2P latency can be comparable to a small model chunk.

The Zero Bubble scheduler profiles

and searches under a specified activation-memory limit. Its heuristic schedules as much forward work as memory permits, prioritizes input-gradient work, inserts weight-gradient tasks into gaps, and uses to recycle memory when necessary. It also formulates the scheduling problem as an ILP for smaller instances. [^5]

This is the right mental model:

> Zero Bubble is a resource-constrained DAG scheduler over , , , and communication—not merely a fixed ASCII schedule.

---

## 5.5 ZB-V: change placement as well as backward ordering

ZB-V assigns exactly two logical stages to each physical rank in a V-shaped mapping:

```
p = 4 physical ranks, 8 logical stages

logical stage:  0  1  2  3 | 4  5  6  7
physical rank:  0  1  2  3 | 3  2  1  0
```

Contrast this with cyclic interleaving:

```
logical stage:  0  1  2  3 | 4  5  6  7
physical rank:  0  1  2  3 | 0  1  2  3
```

In the V mapping:

- The shallowest and deepest chunks are co-located.
- The logical forward path goes physically up the ranks and then back down.
- The backward wave begins on the physical rank that also owns the first chunk.
- Activation lifetimes and peak memory become more symmetric across ranks.

Under , ZB-V achieves zero bubble with approximately peak activation memory, about half the activation requirement of the original ZB-H2 construction. [^5]

PyTorch’s current `ScheduleZBVZeroBubble` requires exactly two stages per rank and explicitly warns that the zero-bubble property holds only when forward, backward-input, and backward-weight times are equal. For real models, a profiled greedy/search scheduler is preferable. [^3]

### Communication cost

With physical ranks:

- Standard contiguous placement has cross-rank logical boundaries.
- A -stage V has cross-rank boundaries.

So at fixed physical device count, V-shaped placement approximately doubles PP activation/gradient traffic. The schedule is trading communication for lower bubble and better activation lifetime.

---

## 5.6 V-Min, V-Half, and V-ZB: the memory–bubble Pareto frontier

A subsequent V-shape analysis makes the trade-off explicit:

- **V-Min:** very low activation memory, but residual repeating bubbles.
- **V-Half:** approximately half the activation footprint of conventional 1F1B under the paper’s abstraction, with substantially reduced bubbles.
- **V-ZB:** zero-bubble target with higher activation residency.

A particularly important experimental result is that **V-Half can outperform V-ZB** when its memory savings permit a larger microbatch and therefore higher arithmetic intensity. This is a strong example of why minimizing the theoretical bubble is not equivalent to maximizing tokens per second. [^7]

---

## 5.7 The optimizer boundary can recreate a tail bubble

Even if all work is perfectly scheduled, an optimizer step often requires global operations such as:

- Gradient-norm reduction.
- FP16/BF16 overflow checks.
- Loss-scaling decisions.
- Data-parallel gradient reductions.

The Zero Bubble paper proposes post-validation and reversible optimizer updates to move some synchronization off the critical path. It also discusses reordering deferred weight-gradient computations by parameter so that gradient communication can overlap the remaining work. [^5]

This matters in 3D parallelism: deferring may improve the PP schedule while delaying gradient readiness and degrading DP/FSDP overlap. The complete optimizer is

not PP bubble reduction in isolation.

---

## 6\. Bidirectional pipelines: Chimera

Chimera combines two model replicas mapped in opposite physical directions.

For :

```
down replica:  stage 0 -> 1 -> 2 -> 3
physical rank:       0    1    2    3

up replica:    stage 0 -> 1 -> 2 -> 3
physical rank:       3    2    1    0
```

The minibatch’s microbatches are divided between the two replicas, each of which runs a 1F1B schedule. Their operation streams are merged so that warm-up and cooldown bubbles in one direction are occupied by work from the other.

This is an important clarification:

> “Injecting microbatches from both ends” does not mean feeding token inputs into the model’s output layer. Each physical end owns stage 0 of one logical model replica.

Chimera remains synchronous. Gradients from corresponding stages of the two replicas are all-reduced before the next iteration. Every worker retains two stage parameter sets, so parameter residency is approximately doubled, although activation memory is better balanced. [^8]

Under the Chimera paper’s notation, with pipeline depth and total microbatches per worker, its ideal bubble fraction is

compared with GPipe’s

This gives roughly a 50% bubble reduction in the relevant regime. [^8]

Chimera is the conceptual bridge between simple bidirectional replication and DualPipe.

---

## 7\. DualPipe: what DeepSeek actually added

DualPipe is best understood as three stacked mechanisms.

## 7.1 Mechanism 1: bidirectional scheduling

Like Chimera, original DualPipe maintains two counter-propagating logical pipelines. Microbatches are split between directions, and each rank holds chunks associated with both streams.

The stable schedule pairs work such as:

- Forward from stream 0 with backward from stream 1.
- Forward from stream 1 with backward from stream 0.

This reduces warm-up/cooldown exposure and gives each rank ready work from both directions.

Original DualPipe keeps two copies of model parameters. DeepSeek reports that this was manageable because its very large expert-parallel degree substantially reduced per-rank parameter residency. [^9]

---

## 7.2 Mechanism 2: split backward into and

DualPipe inherits Zero Bubble’s separation of:

- Backward for inputs.
- Backward for weights.

This allows input-gradient work to propagate promptly while weight-gradient work is repositioned around communication and other computation.

But this is still not the primary distinguishing feature. ZB1P already does it.

---

## 7.3 Mechanism 3: genuine module-level compute/communication overlap

DeepSeek-V3 had a highly specific systems problem:

- 16-way pipeline parallelism.
- 64-way expert parallelism spanning eight nodes.
- Cross-node expert communication with a compute-to-communication ratio near 1:1.
- No tensor parallelism in the main training configuration. [^9]

DeepSeek decomposes each chunk into:

1. Attention.
2. Expert all-to-all dispatch.
3. MLP/expert compute.
4. Expert all-to-all combine.
5. Pipeline P2P communication.

For backward, attention and MLP are further divided into input-gradient and weight-gradient portions. These components are rearranged so that, for example, communication from one stream can run while the other stream performs compute. DeepSeek manually partitions streaming multiprocessors between communication and compute and implements topology-aware all-to-all kernels. [^9]

This is more specific than “launch NCCL asynchronously.” The intended overlap depends on complementary resource use:

- One stream is communication-bound.
- Another has tensor-core work available.
- Communication kernels use a controlled number of SMs.
- L2 and network interference are managed.
- The model’s MoE routing and cluster topology are co-designed with the kernels.

For a dense transformer where both forward and backward are dominated by saturating GEMMs, simply putting two operations on separate CUDA streams will usually not reproduce this overlap.

---

## 7.4 The official DualPipe bubble formula

DeepSeek’s notation is:

- : even number of logical pipeline stages.
- : forward-chunk time.
- : full backward-chunk time.
- : backward-for-weights time.
- : measured elapsed time for a mutually overlapped forward/backward pair.

The official bubble is

Its normalized table gives:

| Method | Parameters/device | Activations/device |
| --- | --- | --- |
| 1F1B |  |  |
| ZB1P |  |  |
| DualPipe |  |  |

[^10]

[![GitHub](data:image/png;base64,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)](https://github.com/deepseek-ai/DualPipe)

[GitHub+1](https://github.com/deepseek-ai/DualPipe)

Two important consequences follow.

### must be measured

Do not replace mechanically with . Depending on resource contention, it can be:

- Close to the maximum when overlap is excellent.
- Close to the sum when there is little effective overlap.
- Worse than expected when concurrent work interferes in HBM, L2, kernels, or network progress.

### DualPipe is not generally zero bubble

The official expression retains a residual bubble. Its primary DeepSeek objective was to reduce bubbles **and hide MoE/EP communication**, not to instantiate the mathematically pure ZB-V schedule.

---

## 7.5 The open-source implementation exposes the key limitation

The official repository states that a real application must implement a custom `overlapped_forward_backward` method for its module. [^10]

[![GitHub](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAQAAAAEACAYAAABccqhmAAAABGdBTUEAALGPC/xhBQAAACBjSFJNAAB6JgAAgIQAAPoAAACA6AAAdTAAAOpgAAA6mAAAF3CculE8AAAAeGVYSWZNTQAqAAAACAAEARoABQAAAAEAAAA+ARsABQAAAAEAAABGASgAAwAAAAEAAgAAh2kABAAAAAEAAABOAAAAAAAAAJAAAAABAAAAkAAAAAEAA6ABAAMAAAABAAEAAKACAAQAAAABAAABAKADAAQAAAABAAABAAAAAAC1Z6LoAAAACXBIWXMAABYlAAAWJQFJUiTwAAABWWlUWHRYTUw6Y29tLmFkb2JlLnhtcAAAAAAAPHg6eG1wbWV0YSB4bWxuczp4PSJhZG9iZTpuczptZXRhLyIgeDp4bXB0az0iWE1QIENvcmUgNi4wLjAiPgogICA8cmRmOlJERiB4bWxuczpyZGY9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkvMDIvMjItcmRmLXN5bnRheC1ucyMiPgogICAgICA8cmRmOkRlc2NyaXB0aW9uIHJkZjphYm91dD0iIgogICAgICAgICAgICB4bWxuczp4bXA9Imh0dHA6Ly9ucy5hZG9iZS5jb20veGFwLzEuMC8iPgogICAgICAgICA8eG1wOkNyZWF0b3JUb29sPkZpZ21hPC94bXA6Q3JlYXRvclRvb2w+CiAgICAgIDwvcmRmOkRlc2NyaXB0aW9uPgogICA8L3JkZjpSREY+CjwveDp4bXBtZXRhPgoE/1zIAAApUklEQVR4Ae2dCXxVxfXHSVgSIAkBAiGEJQRQZHEBVBT/LGqtVCzFFupfRVELtSjFhbq2inUBKspfFBVR+Cul+Ef5gyLgUkgB2ZUdAoGwQxL2EAgQkvD/Dk3+nxDyXt5y75u59537+by8l3vnzpz5zZkzZ86cOVOlilyCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAICAICAKCgCAgCAgCgoAgIAgIAoKAIOARgQiPT+SBmxBQ7aw+kSWVKv0uX8fz3FCf4pIHpd/l08n/LkFABIA7GrLayJEjYxMSEuLPnj1bq3fv3nFJSUnJZ86cSSosLKx3/vz5ukVFRQl8x1PdCL5r812Db9XZq0Rw8bOoatWqJ/ku5DuP2/ncPlitWrWjfLL5HFq8ePHBzMzMk+R7gu/j77zzzln1vlzORUAEgPParvqRI0cSDxw40IpO3eGKK65oTEdtxu8mxcXFKXT4uJo1a0ZVr169GlVTHyvauIh8Ck+fPn2OcvIjIyP31ahRYye/D6Snp+9FaOysV69eZmxsbGb9+vVPklY0B0BwwmUFczihno6lccSIEfGXXXZZ8i9+8Ys2dO7OjMTXxMXFpdDp61OpBEMqlg8dRxFAOUePHs2Ijo5eyvfaWbNm7Rk2bNh+nikBIpeBCIgAMK9Rqu7bt68ln+tSU1OvjY+PvxK1/uraXHT66uaRWzFFCKv8goKCrKioqGW7d+/edPz48ZUIs+WzZ89WwkIuQUAQKEVg4MCB0QsWLGh34sSJQXw+ZyTdiVpdwMdN12GmLssPHTr0xsmTJ2+ZMmVKUmn95VsQCEcEIjdv3tx+x44djzHCz8SwdpTeXuSmHu+pLgi4M9R5HRrC2E2bNvUZMmRITDgygNQ5DBF4/fXX62/fvv12RsGpdBBlQAv36yT2goVoPk8vWrSoLSzhaYkyDLnF/iqLDcB+jC+UsHbt2nZNmzb9TUxMzO3M5Ttipa8RoqIdUwxa0BFwmcM0Yc6WLVvm9uzZU60oyCUIOBMBjF41GNluZO77NktmO8N9qPej/mcRBgvRDO6bPn16M2e2vjOoFg3AhnZiXlujUaNGN7Nc9zvWzG/noxxv5PITAQRGEXaC7dnZ2Z/ib/CPXr167fIzC0kuCIQOATXi4yzT7dy5c1/AvCf8GPEkaSUIgOsWNKknWUYUjcBClhYNwCIwDx48eFPdunUfx41WzfFlxLcI17LZICOqsHqQAdYf41swuVu3bofKPpffgkDIEViyZElL5vmjcHzJqWQQk8cWIoCN4IdTp07dMWHChFohb3QXFSgaQICNeezYsXgs1g/hE/8Y3y0CzEZeCwIB5EkBxtVvMRaOTkxMXBJEVmH7qgiAAJo+KyurOwz3Iqr+zQG8Lq9YjABC4BAawUcZGRljO3bsKNMCi/GV7EoQ+OGHHxpj4BvNJ9dCbVaysgiB/Pz85du2bbtTGNZ3BEQD8BErrNC3kHQUO906+/iKJNODQH5ubu5/LVy4cFyfPn1y9JDgnFJFAFTSVuxiq4tl/0k8+P5I0rhKkstjQxDAf2AVqwUv4X05zxCSjCRDBICXZsHKfA3BNd5grq9Gf7kchgArM7nYBt5kw9VbV1111SmHkR8ScmXjRQUwp6WlVTt8+PD9qPtfSuevACCH3CJ4Sh00t7+2bdv270wJrnAI2SElUzSAcnDjxhvTpk2bEbjvDuWRbNgph49T/0UTSEeYP45Q/86pdbCDbtEAyqDKyN+G8FtT6fxPcVs6fxlsnP6Tjn8F0YmmslLwpHLZdnp9rKJfNIASJDEadWE1ajyOPR2tAlfyMQ8B2rgKnpujv/vuu5H9+/fPNY/C0FIkAgC8MRb1Y9R/GxVRwlSFlv+0lKaEAM5D39DuQzDy7tRChCGFhrsAiFCdn2W+92gPFWVXrjBCAEGwCKH/EJ/MMKr2RVUNWxvA4MGDq+Pc8xwj/3+DiHT+i9giPP6h43cjPuHn+HqE7bQvLDWAuXPnRnXt2vUvBOx4FlavGh7sLrX0hABCYDt2gcFs507zlMat98NOA3jrrbdq0vlHc4rN8zSqdH63crYf9UILbMVg8CmawM1+vOaKpGGlAUyePDm6b9++L9WpU+cZWi+s6u4KbrW5EtgEduH9+RCDQ9hoAmGjAYwbNy7qrrvuGoWkl85vc0dyavbYBFJq1aoVVppAWAgAHD8iH3roocfp/ENpZBn5ndpDQ0A304EmaACTiD94YwiK015EWHQG3ECH4OAzlr4vHmDaWc4ZBDAdWI9zWD88CDOcQXFgVLpeALDOfyfr/J8AT93AIJK3whiBRQweD+AstMutGLh6CsDpMrfT+T+m8aTzu5WD7a1XN7TGj9577z3X8o9rl8FWrVqV2rp164kE7Ey1l0ckdzcjwJbi1JYtW9Zs0qRJ2rx584rcVldXTgFWrlzZ6Oqrr55K57/ZbQ0m9dGCQDGHub6IcfA1LaXbWKjrBADGmxp4dr2NNfcRG3GTrMMMAfjqJHw1AI1glpuq7jobAJL698zbBrupkaQu+hGAp2IYVMbv2bPnWv3UWEeBqwQAx0r3xGL7ZxrLVfWyrrklp2AQgK8aN2zY8G87d+6MDyYfk951zRSASC/JrNl+RSOF7c4ukxjLzbSUTDGHw2uFTq9nNadXQNHPGfLKwedZXZ1fzQ8p/wgfxRBqZSWGe0pljOa3XEEiQIerQtuq6D2n+eSXZKfavB73Q342IO06iOPIfqD8L0poceyXKzQANnD8DtX/PZiheqhbgo6ezdTjsczMzE1Yic9xem11GLYepwfVhZ5EwlGn4IvQHro64Y2YpIPGUGMSbHlgehocd7OKsxyX3E2bN2/Owkc/i3xPEOX3OB56EeBbo0OHDjcS5+8dMI0Ktkx/3yeiUCZaZ2/cy7f4+66ktxCBxYsXp8IM6TCNlovDJ2ZUVp0BAwbU/vrrr1Pz8vJ+zak1UxAQ6TBQsRaCDS0UTE7SjssRpmM5eLXHxIkTm7Rr186r6/aYMWMS8NRbq6tKtOf/qO3llbW/PLcJARo+itFgii4GoNwCOvQAP6sXsXr16sZ4Kd7HKDcTxj+lkX7tReOqvYNO/NaKFSt+vm7dutr+YonQeFlXJWi7QoSACh8vlw4EcnJyBtD4+boYgKlHzmeffXZZoHUnLFktVNyf0wHmqRFQVz10lEt9txGS7SXasGWg+Kn31q5dexfa1DkddVBlIsR3LV26tFUwdZB3A0Bg2rRpTZmDbdfV8KpcOu6cp59+OjYA8i96ZcKECdVZX/45zDSDjmEXMxdBssq7gM9pL58zJWlUessvRuws6vkcGlCLi0AI8B+8PtuB2R7LCfUjQwTQhz/++GPI7U8BQnbRa441AhLD7TWMbiqsl85rFAYoRcN5K4hQGsGbb775YO3atZ8k31R/8lQ2BazTh3nvGLx7LCsrS00tjmGAzMGYls13Lp887hVgODtO3p7aviodNJ5OVQf1PJ709cm7Lr/rc9XD2BrHvbp8EijP6xy9LP2kP4+T1iymTKM5sHNF2WfB/J49e3atW2+9NY0l4OuCySfId09iC7o/MTFxZpD5yOu+ILBr164bYNAjiql0Xlj+h/hCr79p0tPTL6MTTvNSt7OMpDmo0AvpmOO3bds2ZP78+f2WLVt2PUebNeM9tfyoliM9dXJ/SVLpIwmsEs1Bm4lstLqGs/Z+SXl/gM5X+XxJ594BLRVOx2irLLSlR8nDZ4HhB4GRe/funesFq5A8AoPl7777rkSX9qPhAko6fPjw2sy9tTc4TF2oOl1AlfDhJeaVNTmg9FU61XHFwZSXzWcx98bDbH0Z4ds+8cQT9chK+45ONQ0aNWpUs927d98Cfc9A22y0hgw+atTfgFW/mw9VDjgJ5b0fkl7upRDaphi+tGVACBgYN77I6N+X0c+uebKXJr74Ecydu2TJkpvtxvjAgQM/Z4QbylFWnRnpQ77eHWD9ItesWZOCEOvHFtqUAPPw+TUMqX+5uHX0/IcM2IUtR/l8yGUHAowwau75Lz3Ne3GpNPYBpQrbUU/J0z8ECOf9B1qn8OIW0vMf2tlo/6jXm9pRm2bi4+PvBa6b9EL279KVsQ1jXbYJtIQ7DRgjL9geTMABQ+sApiTtTKDFFxocIwA2bNiQiDvow1RK+5y3BNjC5OTks76ALGnsRYBOdxyBbES0HmhJ4twJZQuw0gBrG4COEQDNmzdXwRiutg2JADLGC8wRjRxA1Rz1Csua50wimCXJ327fvv0Gk2jyRIsjBAAdrSHrz0r9N+k6jQZQujPNJLrCkRajBDHaSH1iCN5HQxjfv4wnUHEzlv87Gf07GMbZCjujGM8wfEJJjiWOWFYSDL/exQpOwG7iVtLiLS/jBYBa92/RokV/KmHK3L8Uz6h9+/bJfv9SNDR+syRrh4NRUDXCFpDIacNKCzD6Ml4ADBo06Db20d9sGoosMlVjauKUdXnT4LOUHpbejHCIKl8pbAH3E8vAaL8AowWAOtOPefbd+LIbF7mIeV4Urrj+bl8tzyPyvwUI4BdmpACgak2xXd1hQRVty8JoATBw4MAurLX3tK32QWSMI1AsqqdrT4wJApqQv4oAaEyhRtpjGjdu/OtJkyY1CDkoPhZotABISkq6k3qYCl4dVM9GPuIsyexDQIUHa2pf9sHljDHwqv79+18ZXC72vW2sAPj0008bMs/+hX1VDy5nvM+iMPSYKpyCq5yD3iYkVzQeosmmkqy2TDNQDII+Y/uakdixg6wParYKXmHsRcO+aCR4YUSU2nQEg2wzlkkgjB2de4hcZKR7sKlSqSpq3QMY2oyOskLbdh4yZEhMGPU346qKIbY9A0UT4wgrQxCaYtOUlJQ+ZW4Z89NIAUC4qFb16tUzyu23ohZDAHTq16+f2AEqAic09yI6dep0NWq28f4Y7GPpYUX4OKthNVIAINVvRWqmWF1Zq/PDBz3p2muv1RmKyuoqOSo/QqjVZCXGKVuyu6AtGhc81DgBoE75adOmjdpIYeSyTtkewhRlE/9vKHtPfocOgQ8//PA05/R9SonqZCajL3xZYjBW3moakcYJAAxrzZDqxgFVvuGYd57keKg/cVKNCIDy4ITu//McHvIlPPNXZWsLXbEBlRSBxnhHWlqa8dOVgGpn1UtEd/ktnUt7yC8YyutFdNs3qbPxWopV7WJyPuPHj49BCMzy2mBmPNzPxjajpiymaQCRLJn0RrU2zvW3bAfgNKKVHGGlQj8ZtwutLJ3h8vvRRx9VU4AXGTgOGF7nJIKm2hog1d/6GyUACKVUn2gqV/lbiVCmh8kKCb09tlWrVgdDWa6U5R0B1Ov1nBUxgYHee0K9TyOYslzTo0cPYwY4owQA+6c74TrZTG8beS+d0M/fciCJHADhHSYtT4nI+wH2I8sOHbGpEl0efPDBRJvydna2hFF62oypWsVUMD05BZPd5myU3U09WsAgtDRjT16GNE60y+9qSisYowGwplu9WbNmpkX9uaid8E6c9cYbbyy86Kb8YxQCGNm+QHyvMYqoMsRg34rmlKSbytzS+tMYAdCyZcsGGNeMspCWbRmYqiAjI2PmO++8I5GAywJj2O8rr7zyGCPsZKXDGUZaKTkReC5ei7+LERGujBEA9957b0Mi/zQvRcm0b6L/rNi6desc0+gSei5FACPtLGwB2y59YsaduLi4thgCjdhJaowAwPf/crylTHWSOI928j37uk+bwUJChTcEunTpcgB/kq+8pdH5jGlAfU4TbqmThtKyTREAERwiqQwjxiyPlAKkvnEyOcxJwDPK3pPfRiNQTCy+75kFmOoinMCAYkSQECMEALH/qqL+X24qSyGx53/wwQc7TKVP6LoUAaztKzG2mToNiMQfwNjp7qVo2nyH0b8RDbZJGW5MvFiefMxmCCR7GxDIzs5+20R+UjSxojSDmJfap7xGaAAcf53KKKsiuxp30VbHiP+/zjjChKBKEUhISPiG9jNy1YYpwOWsWNSvtBI2JzBCALBsk0o9jRQA0LW9e/fuP9ncDpK9DQgsWbJkCyNtlg1ZB50lGm8yhu/4oDMKMgMjBABGtiTqYdzpLgrbI0eOZKCdiPU/SEbT8frUqVOPogFs1lF2ZWUSISiKVaXYytLZ/dwIAdC6deuGdlc0wPyLMU4u5V1TnUoCrFZ4vEbAkFwiSxm5NwBnoOpoAUrz1XppFwDE1KuKKmSkAMD3vwDfchX1Ry6HIvDTTz9lG0p6NfhLezBT7QKAoI4xzNOSTWwkpibHp02bts9E2oQm3xBgCrCXkTbft9ShTQXfKw1Aa1AZ7QKAo79q47Zp5PZI1LQcGEjm/6HtF5aWxhRuPzaco5ZmalFmaADqRCOtewK0CwAOT6xFJ0uwCFNLs2H+uI+4f6cszVQyCykCnM2XhQA4HtJCfSyMASYOhyCtfVBr4QonvLWi0AC0O0RU1GYIgGxOKDJSfayIXrl3KQKNGjVS7sBGtiF+CrU2btyolfe1C4BevXqxIlLLyBOAGDmOYUk+dylbyR2nIMBSWxFaXIGJ9KIBROMDo/VkKe0CIDU1tSZhwLTOgzwxBw10wtMzue8MBD7//HMVgsdUIV4bL1it01/tAgBLexyspJ0OD+ws6/8egHHQ7fMMMKYactVSYJROLLV3PJZolAeg1qUQnQ0gZduPAEZm+wsJrIQa2JlqBfaqNW9pFwBIwDpURTsd1sApuRiIQETJIGMgaVUiMIBr5X2thZe0iJr/G6kBwDhapbOJHOtAmpQAMNLIDJbR0KamwNouEwSAtspXVjCqo2gnlYFk+HOCb1ZjuU3rUps3iNAAtA5+IgC8tA7SuQ5BG4zcpeiFbHlUBoFu3brF4mymfdddGZKM+ikCwEtzIJ2btW/fXus6rRfy5JEPCHDaVAKanKltiKtJRHhrAIyyWgHwxkMIgBZYabXO0bzRJ88qRwAfALXRrG7lKbWkKEQ4aXVS0q4B4GxTBPRGrtNw4GR83759tUdt0cKaLimUUPPNGGRNnQIUMMBojVysXQDgpHEMXis2kd+gLSopKSnVRNqEJt8Q6Nixo5E7TUuoVwOf1sFPuwBAQquNGlpB8MJK1dkM9B9enssjgxEYPnx4baaYHQ0m8Wx0dLRWd3PtAoBRNo95kJoGGHkxDehgyjluRgJkMFEs/9XH0czY8yaB7iwrFOE9BfjnP/95mi3BxgqAOnXqtODUYpkGGNzRPZF2zz33tEGAN/b03ID7hfXr1z+jkw7tGkB6evpZQiMV6gTBW9kYkJo2aNDgOm9p5JmZCBAN6A6MbMY6AbFCcer777/P1YmedgHAFOAMjWRy1J2qzZs3v4ngpeIQpJNT/Sx7+fLlcfHx8V39fC2kyTnANLdPnz7hrQGcOnXqJEuBRh7eUMoNGJJuGzZsWKPS/+XbfATYZt4FDaCNyZQy+KlYhVqnv9o1gJUrV+ahAew3uaHUWnKHDh16mEyj0HYRAhHE2uvB9K32RXcN+we+PwBJWpfAtQsAIracRRIeNKxtypNTDWb6dVpampHHl5cnNtz/X7duXXJcXFwfw3E4D0/t1U2jdgEAAGjYxaYLgCoxMTG3YAy8RXeDSfmVI0CYuQcYVNpWnlJrikJ4KlMrBRRuggCosmnTpmxoMdUZ6EIbKXWyZcuW/Xv06CFagG6u9VL+ihUr6hNkth/t5SWV/kesfJ3j+PI9uikxQgAAwi4+ebrBqKx8jEq/+uyzzzpXlk6e60Ogbdu2qvO310eBbyVjpMydMWOG1iVARakRAgBnG3X8llaXSF+ajdWKerGxscNHjBghWoAvgIU4zdq1a5NxrX0UAWBklOmycGAAzIKfjB/0ytJs229cgWPZevsj38Zf2CtOMWW53TYwJOOAEcjNzX3VeAYqIRB+/5iKah+AtROgWhuJnX/48GHtFlFfOA9aa2ELeG7fvn31fUkvaUKDAF51XThm8pHQlBZ8KRxWopa+tS4BqloYIQCgowif7WV8G20IVICpC1q7YWh6lp9mW5ouUOv+P5MnT45HpR7BxylCuSgzM1NNe7VfpgiAKkjvtWhHpp7gcklDsc48mK3C3S95IDdCjsDdd9/9OM5aPwt5wYEXeLBu3bprA3/dujeNEQBLly7dy65ArVsj/YGV0SYO4+Ubp0+fbubPe5LWWgTY7nsXhr+nmJoZw8uV1ZD5/34EwObK0oXiuTGgzZs3L5tG3BiKSltVBvSqJcFXUEGjrcpT8vEdgQ0bNlyJ1vgyb5ga9LPCyrAJKKNhw4ZGnlhcIcGhullQUPC2U6y4pXSyKnCeNd1X2S1o/NJTqNoxFOUwh24Gvzhi5aiUV0q+i06cOPFwKDByXBms4w4GpHPlAHPCv/modYMdB7hDCd69e3ddBO90JzBGeRoRWnkIL2OcyYyZAihexLK+kq8cB/JlTZw6xjAffcCBtDuKZAyv8cnJyR8x/ernKMJLiEUA7JwyZYoRKwCKJKMEwP79+7dgCNzmxIaF5lgMg2oKc79D6Tee7KysrAasvrwLzncZT6wHAjFYrsCR7JCHx3L7yJEjL5ZXm5z0P6ppHlL+97Sk2AQsZOetW7e2Yf/Mv5zECxXQem7nzp1GTRWN0gAUvxAhKI2G1homKRi+RTWNYU16LJrMqAkTJtQKJi95998IYBu6KSUl5VO2+Drd72Ivy8Zp0q5eEHj99dcb4Na5ugLp6bhb1OMziG7hpbryyAsCaus12tQgtKqDjmv8Cghmr8Is2UjmpcFLH7FPelwF+Dn11hrWfZ3kpVbaDFq/lyxZ0pzlsg9o9NNObfiydCPEijBgDtQKagWFGzcFUDRi7PmWL8d4BVaAa9lbVxP5ZQabP/7GPDah7AP5XSECkZzoe1+XLl3msPVa2VJc4WSFJpOzfv36JRXWWONNIwUAhsDlzJU2asTF0qKxWsfi+vkn5rFfrl69+peWZu6izDZu3HgVHWUCXnITWVZt56KqVaE+C6ZOnbrLTXWytS6MmC+UVaHc8huHoXwMnR8fOnSok60AOihzVP3LMZqORk3e55Z2LlePoj179hi5VdlIDUDxLkwxD8ca10VMYSSoicPTQ/Xq1fuW0W4s04JrHdRXLSVVLe3hRv0SO0G/Y4v106ygJFtagCGZIfCp6ta5hpDjDDLGjRsXBXB2unvmkv86NI1/5OXlvY3AeRMr7VsYaqar6QcSPLecFLflXzSCg9DxIRpBZ/YT1HRG6wRO5eDBg2uxFt6dOo9jxN9lC6iGZcqU9q3AEQvjN0v2BhRa2Z4w3TEa5G38yW9RgSSAVx35pZx2Sj81fvzxxwT1HOHwPh10t5Xle8oLbecYy4afc6TVfcuWLUuBHtcEG2HpK1LVCSE7CFX/K/w8QiJcPWEdyvvUNW/v3r1ytiQM7fcFcPXoGCutajBGndU0SA9/CMnIyLgCDeEf0HHWKjq85YOAOo/QycjJyZlOmLT7Jk2alHrDDTc4TjMYOnRoHMu57RGij6Dmf0mdM7zV263PTp48+fUTTzzhuPbzp4/YmpbO94LqFBZceatWrQr0tJga7FN4mI6ZZQEdPmdBvQsQWgcYNaczNXlywYIFv6QOlwO40lp0X0pDuaClKOcWtUNv/vz51+3YsWMAQvY1plHfIDQPU9kinyvsvoTnOKXI6K2/xquZRApq1blz5/nqfL5gOB7e2sqU4j86duwY8EYMVPTratas+R50aLHgI4DOYUTch2PRPgTDXgxn6RgUd9DhlrLMuCsYfPx9d/z48TG9e/e+B/fcbhg0G7PJRW3RbQx9Df3Ny63pwWMDWtzPEhMTnbjD1ZhmiWDOPibYwYHGOEU+vw22Vqiz7eiI64Olx6r30ZAWEBmnZbD18vd96I9gtL+Nb2UwlascAvDbedT/p/zFVdJXgACGsU6ok9nlMA7k3yys7R/Rafoxx+6K6tq4guIqvUXBbWlg7fsV0EhmLFq0KKlSgm1MgCbSGDzmBNIYbn4HrWwjxmStbWNjs4c860jU3okWMsxZBMopJPRmLNOv46KZ6m+N6Hw3oAnkWEiTX1kxx56DYEz0l2470oNjI4gXIVCmBeHX5+3AOmzzhNmvRP1WRiXLLzryFhrs1/6CixZxL4Ikz3KCKsmQUfd/2Sxj1FwbkpPBcU0lpIfFY0b/TdibXOnU5G8fsSy9CrqJQcXOXYJHsbbf6SfBVRn9/hZKrmb6shhru5HMtWXLlptLLP+hhMSostTcH61S5v5+diSfkrOkcjkMZqf32E46dAefiClJBPfFoJl8EwouhLEWED49xR/6Qp0WLJ4JBRamloF79yaZ+9vIdXSCUXY2Pur1TGK2+bXOzjmByh5ghZHSY9XIfysGv7Y2QmtJ1izb1gPDHzxWxN0PiuHPJy0BUjKpGAG20zZmjqWOEbPlQoU7k56e7u9UoAoj339C0Bk7iIKmTeTbsWJEzLuLQLwDgZVvBxYm50k7LZJDY0PAj8yDfw/Ylu4RKMtYdOYv/I3nx/sRjHyvQlfZrIL+jbBbjVppTBx5X5p3wIABtVkl+SroyjsoA9qpYOfOnb/yBR9JEyQCqnMyV59tF39gZzhDVKJe/pI5d+7cKITTf1lFFwJl9pw5c5r7S4cJ6QnucSdagC0akVX4WpkPy7KfYKj2a+poQjs5lgZ8z7vSUY9a2Yhl86JBPw4EnNmzZ9ei475CXifK5ufn7wK0kMlpaWmNAqHBhHfAIQEBoN1Zyk/cA01+AK/I60zAPaxoQO0aEWiLVfYeqnw2Kt3VgQKKEOhNJ95QWTnln6Pub+fzAJbk6oGWbcp7YPAX6ufqzUAIuSK00WGmYB5WdGAVb0AnW1a+E1n1P3mPBNCAN0wtXLiwNVuaR5JPOgLF45mHPKfPF6zFz+GVNWvWtHZLI/7000+dqNdxq9rDxHwYhObiUu6o04ndwl8X6oGq3g3GsGUqQN67Z86cGfRGm2+//bYpTka3MlI8x7beSeyRn8lnGt6H72MsewavsZ6cF+c6v3HmxHWo80ITO64VNDEF3U/Mgy6u6lBOrAyhvP5qRYNWkEcxQuA1izGpRn5RfJSKr6IQufmKZIR8rQJcHX8Lja4Y4e141d/YoKD+9ApG0rGkn+/POz6mjWDP/cOo8Vf6mN6XZIUkOsvnHJ8iX15wcJpiNlqp6Y+Dq1Ax6Qi2eTj9fFTxU+fcdYUAaN68uYqn9wz2mGyroSdSLfEcEh8k34BtAVbT5KT8sGvsYbjPdRLNldFKffayrfyFRo0anaosrenPXSEAFMiElv4JZnuBnwVWg06Um98REqy31fmGQ360yy7wy3JRXc8w+j/epEmTtW6ok2sEgGoM3DCnoG6+b3XDcLJPTHx8/LOLFy+ua3Xebs+PUGXZaFGuCInFyF+F6eYHI0eO/Mrt7ebY+rElNRbrrC3BKZD8bzgWGH2ER3DW31zHW/2oAHP+uaNHj47VB6WU7BMCrL9fgfV+sw1Mp6II3eMTEZLo/xHYtWvXNBvaIqRZolmqDWip/18pl/xw1RSgtE26d++ezu8/8Ak4AnBpXuW+axEF9y2Oebqp3H351wsCYOZoIyAd/wB+G08yldnhpZqOfORKAaBagvDdC1kV+CONd8LKlmFZMLFVq1bvc2DI9Vbm6/K8zju1fsrTFw3m8YSEhAVOrUNY040H3hDUN49uuIHqkbi4bhMh4BtrYZx9P1CcNb9XjN3nOd9q6cxUrtUASpvjk08+mYjx5sIW3dJ7VnxzUEmrlJSUSZs3b77DivwkD+MQOM/A8SbTF+VkJpeTEWCzRjRC4HVGE2ujdagMi4vzGCWe+vvf/x7nZIzspN2JGgCq/xs0r+N3ZNrZro7Km5OAo9kzMJYOa7kQgFGKWHr8jhWCWwBF+frLVQYBpwkAOv97tGl0mSrITzcgMG7cuCi7NAEYRp3qe5zjx2aonX8qIEawmM2YMaPJ9u3bmwabj+73nSQAaMMP2PtRUzdmUr5NCKjpAFtylU3AtkAVKBn5TAt+2LNnz/NEyf0VHaAJ1VE7AL3t/otUtKkOz/bgvsQIeA1D4waY8WWboAhZtk4RAHT+CQcPHgyrvf1hp67Syc5wmu0zAwcOzI+NjbXFwst6cU2MR12bNm3aNTk5We382wNjqWPEsnArPlCjRo39/D6Fj7yakVSH8RIRGE2hpzXLlwk8a0YeSmBUIf2skPXU8C5oHO3xTMOGDc+ENwxhUntlE8CveyQd8CwdTselNBD18WqTYBOSaAD2tk4xAlgZ/MJyzu/6ZUBP8uzBBx88wwafFxAAA0lzxFM6G+8r7NVHthnbCHIlWRfQ/q8x8j+PxhWWI3/YCoASxihGVZ/GXPsRPvsrYRZ57CIEGPWPEfp9KHElX6bzq+AsYXmFuwC40Oi4935BAMs7EQIrwpILwqzSqPvbMQTf37hx4w979uypIjSF7SUCoKTpb7zxxjVY3+/HGDc9bLkhDCpO519B9KjfNGjQ4OswqG6lVRQBUAai66+/PgOj2wP4CvwVFdHRO9jKVEt+ggAdvwodfwLbxO+KiYlZJ6D8GwERAOU4oUWLFmfq1KnzUk5OzgMYiLaXeyz/OhOBw3T+Pz377LPDCFF2wJlVsIfqsPMD8BVG1u+/RBNIx0L8Mkxzt6/vSTqzEMChag2j/+OM+ovMoswMakQD8NIOcXFxGWPGjHmYI66exjbgirh2XqrrqkfszThD539v1apVv8G5Sjq/q1pXQ2UIM3YNQsCWWIOMUB4vcQTyCI3HB0zdVuNC3Rc2ER+LSvqKaACVAFT6mDBja9jo0x+/9j/CYNtK78u3OQggEU4TumtiZmbmL3HDnglljo1EZA6qQsklCLDbrzXTgndhuDyPw5BFD9ygAbAp6gOL4PCYDdb9ZbRJL84j9Lbh6pK2DPcbYgQMgAOio6OVBvAYIcG+SUpKGsr/N+NRaAuWGCEdr8ayock2Tzu0sf1oZeN37NgxEaeewwE0p7wiCASOwPDhw2sTNPJeGHGFx+Ep8AdF7CJ0/AGUnAvwSuAQeHzzOEt7Y1ipuSzw1pM3BQGLECA2YBK7C1/AULjBI8v6+QChkk+eP7OIRG3ZMC+/l6oX+Fn9CpOD70k6/jTm+Sr6klyCgFkIEAAkmdBgT+FJuKlCDvbjJvPaTRMnTkw0q4b+U0MotlYIs71+VP2SpLwPrCenoW31hAJbplv+10zeEAQ8ILBu3bombDgZxlr0N2wy8jvmAAxfhOr8CNk73gagDHMqwtElvdqHG6znH0YQfloSfVmCdHrgN7ltKAJDhgyJ2bZtWx9Gr8/o1Ad94PkLSdAgJk2YMME1DE+lEujMi32tP+kyERqvsILQnqaV5WpD+VvI8h2ByPXr19/A3PVFlhHXIQxOV9QZ6PjY/Q7+mXlzHd+zdkZKVk2uoFP/Q3npVVR3MDmBtvQ9av5j6enpYtwLQbM6Xr0MAUaWFzF9+vRGvXr1aotB6zYYvh2dIY7lvuPqODO+/4nf+nrLCzUkw6FDh0Y9//zztxIX8Vbq34G616DOh6j7MpZT06ZMmZLx8MMP5xlCruvJEAGgv4mVQUupuMV8wik4heI9VXf1XVTy4UsuQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEBAFBQBAQBAQBQUAQEAQEAUFAEBAEtCHwf/yZE3EFLSc+AAAAAElFTkSuQmCC)](https://github.com/deepseek-ai/DualPipe)

[GitHub](https://github.com/deepseek-ai/DualPipe)

The linked Hugging Face code walkthrough shows what happens otherwise:

```python
if not self.overlaped_forward_backward:
    self._forward_compute_chunk(phase0)
    self._backward_compute_chunk(phase1)
    return
```

That fallback is sequential forward followed by backward. The schedule may still gain from bidirectional ordering, but it does not obtain DeepSeek’s intended internal overlap. [^11]

Therefore:

> `ScheduleDualPipeV` is an orchestration primitive. DeepSeek-level performance requires model-specific decomposition, communication kernels, stream/event management, and resource partitioning.

---

## 8\. DualPipeV: “cut DualPipe in half”

DualPipeV folds the two counter-propagating streams into one V-shaped logical model.

There are two valid normalizations, and mixing them causes much of the public confusion.

## 8.1 Hold logical stage count fixed

The official DeepSeek repository compares methods using the same number of logical PP stages:

| Method | Logical stages | Devices | Parameters/device |
| --- | --- | --- | --- |
| 1F1B |  |  |  |
| DualPipe | per replica |  |  |
| DualPipeV |  |  |  |

DualPipeV uses two logical stages per device, so it requires half as many devices for the same logical stage granularity. [^10]

[![GitHub](data:image/png;base64,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)](https://github.com/deepseek-ai/DualPipe)

[GitHub](https://github.com/deepseek-ai/DualPipe)

## 8.2 Hold physical device count fixed

For practical cluster comparison, set :

- One V-shaped model.
- Two half-sized logical chunks per rank.
- Total parameter residency per device returns to roughly the same level as standard -stage 1F1B.
- PP boundary communication approximately doubles.

Sea AI Lab’s fixed-device comparison explicitly lists:

- DualPipe: parameter residency and PP communication.
- Cut-in-half/DualPipeV: parameter residency and PP communication. [^12]

This is usually the most useful interpretation:

> At fixed cluster size, DualPipeV removes global model duplication by using twice as many, half-sized logical stages, but it pays for that with approximately twice the PP traffic.

Sea AI also characterizes cut-in-half as an **EP-specialized ZB-V schedule**. If one unties the paired forward/backward work and aggressively reorders , the structure approaches ordinary ZB-V. [^12]

---

## 9\. Assessment of the two linked articles

## 9.1 The PyTorch/TorchTitan forum post

The forum post is strong as an architecture and implementation overview. Its most important contributions are:

- Separation of `PipelineStage`, `PipelineSchedule`, and runtime.
- A declarative schedule IR.
- A compute-only trace followed by passes that insert communication.
- CSV/Chrome-trace serialization and schedule simulation.
- Distinct full-backward, backward-input, and backward-weight actions.
- Automatic autograd graph partitioning for . [^6]

Its benchmark should be interpreted carefully. For the reported 512-GPU Llama 3 405B runs:

| Schedule | Memory | WPS |
| --- | --- | --- |
| 1F1B | 65.79 GiB | 96 |
| Interleaved 1F1B | 68.44 GiB | 124 |
| ZeroBubble, without compile | 82.86 GiB | 125 |

Thus, in that particular snapshot:

- ZeroBubble was only about **0.8% faster** than interleaved 1F1B.
- It consumed about **21% more memory**.
- Both materially outperformed the reported non-interleaved 1F1B configuration. [^6]

That does not disprove Zero Bubble. It shows that implementation overhead, activation residency, compile support, communication, and the selected regime can make the theoretical schedule advantage small.

The post itself also recommends manual model splitting for better composition with other parallelisms, even though compiler-based splitting is supported. [^6]

---

## 9.2 The Hugging Face DualPipe article

The article is useful for developing intuition and following the open-source control flow, especially its discussion of the bidirectional phases and `_forward_backward_compute_chunk`.

But its quantitative notation should not be treated as ground truth.

### Error 1: is misdefined

The article says is an “activation window size.” In the official DeepSeek formula,

[^11]

### Error 2: the DualPipe table is mistyped

The article renders the DualPipe factor approximately as and the activation term as . The official expressions are

and normalized activation residency

[^11]

### Oversimplification: generic “chunked transport”

The article explains overlap largely as breaking transfers into smaller shipments. That intuition is not entirely wrong, but DeepSeek’s actual design is more structured:

- Attention.
- A2A dispatch.
- MLP.
- A2A combine.
- Separate .
- Custom communication kernels.
- Explicit SM partitioning.
- Topology-aware IB/NVLink routing.

[^9]

### Correct and valuable observation

Its code tour correctly exposes that absent a custom overlap implementation, the open-source path executes forward and backward sequentially. That is arguably the most important implementation fact in the article. [^11]

---

## 10\. What PyTorch exposes now

PyTorch 2.13 documents the following native schedules:

- `ScheduleGPipe`
- `Schedule1F1B`
- `ScheduleInterleaved1F1B`
- `ScheduleLoopedBFS`
- `ScheduleInterleavedZeroBubble`, implementing ZB1P
- `ScheduleZBVZeroBubble`
- `ScheduleDualPipeV`

`ScheduleZBVZeroBubble` requires exactly two stages per rank, and the documentation explicitly conditions its ideal zero-bubble behavior on . [^3]

The schedule runtime and IR are becoming a useful compiler boundary:

```
model partition
      ↓
compute-only F/I/W/B schedule
      ↓
insert SEND/RECV, unshard/reshard, reductions
      ↓
validate/simulate
      ↓
runtime execution
```

Current PyTorch source represents `FORWARD`, `BACKWARD_INPUT`, `BACKWARD_WEIGHT`, `FULL_BACKWARD`, and `OVERLAP_F_B` as distinct computation types. [^13]

[![GitHub](data:image/png;base64,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)](https://github.com/pytorch/pytorch/blob/main/torch/distributed/pipelining/schedules.py)

[GitHub](https://github.com/pytorch/pytorch/blob/main/torch/distributed/pipelining/schedules.py)

Composition is still an active engineering area. For example, an open PyTorch issue reports incorrect FSDP unshard/reshard timing in some PP schedule combinations and notes that not all schedules had yet moved to the same runtime path. [^14]

[![GitHub](data:image/png;base64,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)](https://github.com/pytorch/pytorch/issues/164756?utm_source=chatgpt.com)

[GitHub](https://github.com/pytorch/pytorch/issues/164756?utm_source=chatgpt.com)

---

## 11\. Which schedule should be selected?

## Dense transformer, conservative production default

Use **synchronous 1F1B-Flush**.

Move to interleaved 1F1B only when traces show substantial warm-up/cooldown bubbles and the additional P2P traffic is cheap.

## Deep pipeline with small

Try, in order:

1. Interleaved 1F1B with .
2. ZB1P.
3. ZB-V or a searched ZB schedule.

ZB1P is especially appealing when network latency makes cyclic interleaving expensive.

## FSDP/ZeRO-3 and low batch per GPU

Evaluate **Looped BFS** or a blockwise/stage-grouped schedule. Its benefit comes less from a different asymptotic bubble and more from amortizing parameter unsharding and improving communication overlap.

## Memory-constrained training

Do not assume zero-bubble is optimal. Compare:

- 1F1B with recomputation.
- ZB1P.
- V-Half or another memory-efficient V schedule.
- Larger microbatch enabled by the memory-saving schedule.

A half-memory schedule with a larger microbatch can outperform a nominally zero-bubble schedule through better arithmetic intensity. [^7]

## DeepSeek-like MoE with cross-node expert parallelism

DualPipe or DualPipeV becomes compelling when:

- EP communication is a large fraction of stage time.
- There is complementary compute available to hide it.
- Custom A2A kernels are available.
- SM allocation and network topology are under your control.
- PP traffic is relatively minor compared with EP traffic.

This is the environment for which DualPipe was designed. [^9]

For a dense model with communication-light blocks, begin with ZB1P/ZB-V rather than assuming DualPipe’s paired F/B mechanism will overlap two saturated GEMM streams efficiently.

## Relaxed optimization semantics

PipeDream or bounded-asynchrony approaches are relevant only when the training program explicitly accepts delayed/stale updates. They should not be compared directly against synchronous schedules solely on bubble rate.

---

## 12\. How I would evaluate these schedules experimentally

## 12.1 First build a stage profile

Measure separately for every logical stage:

Report:

- Maximum-to-mean stage compute ratio.
- P50/P95 timing, especially for MoE.
- Embedding, loss, and output-head asymmetry.
- TP/EP collective exposure.
- Activation memory per live microbatch.

Partition based on measured time, not equal layer count or parameter count. The Zero Bubble experiments themselves removed a transformer layer from first and final stages to compensate for embedding and loss computation. [^5]

## 12.2 Run two distinct benchmark modes

**Pure schedule comparison**

- Hold microbatch shape constant.
- Hold model partition constant.
- Vary schedule and .
- This isolates scheduling and communication effects.

**Training-relevant comparison**

- Hold global tokens per optimizer step constant.
- Let the schedules choose their best feasible microbatch size, activation checkpointing, and virtual-stage count.
- This captures arithmetic-intensity and memory trade-offs.

Both are needed. The former explains mechanisms; the latter selects production configuration.

## 12.3 Sweep the important axes

At minimum:

Compare:

- 1F1B.
- Interleaved 1F1B.
- Looped BFS when FSDP is used.
- ZB1P.
- ZB-V.
- DualPipeV only after implementing meaningful overlap.

Also sweep activation checkpointing because recomputation changes the relative values of , , and , potentially invalidating a schedule generated from a non-checkpointed profile.

## 12.4 Measure more than MFU

Record:

- End-to-end tokens/second.
- Actual schedule bubble from a trace.
- Exposed PP communication.
- Exposed TP/EP/DP communication.
- Number and sizes of PP messages.
- Peak allocated and reserved memory.
- Activation liveness by stage.
- CUDA kernel overlap and SM occupancy.
- NCCL progress while compute kernels run.
- Gradient-ready times.
- Optimizer-tail duration.

A schedule that improves nominal MFU but increases iteration variance or reduces delivered tokens per joule is not necessarily an improvement.

## 12.5 Verify numerical semantics

For synchronous alternatives:

- Same initialized weights.
- Same data and microbatch ordering.
- Correct loss scaling by .
- Controlled dropout/RNG.
- Compare per-step loss.
- Compare selected gradients.
- Compare parameters after optimizer step.

The Zero Bubble paper’s bitwise-loss comparison is the right style of validation for schedule changes. [^5]

---

## Bottom line

The progression is:

```
GPipe
  └─ synchronous, simple, high activation lifetime

1F1B-Flush
  └─ same basic bubble, much better activation lifetime

Interleaved / Looped
  └─ reduce geometric bubble through multiple chunks per rank
     at the cost of more PP boundaries

Zero Bubble
  └─ expose I and W as separate tasks
     schedule critical dX early and defer local dW

ZB-V / V families
  └─ combine split backward with stage placement
     to improve both bubbles and activation lifetime

Chimera
  └─ fill bubbles with a second opposite model replica

DualPipe
  └─ combine bidirectional streams, I/W splitting,
     and model-specific compute/communication overlap

DualPipeV
  └─ fold DualPipe into one V-shaped model at fixed devices,
     trading model duplication for roughly 2× PP traffic
```

For a general dense LLM stack, the practical progression should be:

For a DeepSeek-style MoE stack with dominant cross-node EP communication:

The decisive point is that **DualPipe’s advantage does not come from its schedule diagram alone**. It comes from the co-design of the schedule, MoE module decomposition, all-to-all kernels, SM allocation, parameter placement, and cluster topology.

---

**You said**

[pipeline\_parallel.py](https://github.com/huggingface/picotron/blob/main/picotron/pipeline_parallel/pipeline_parallel.py) ground the discussion on these

---

**ChatGPT said**

## Grounding the pipeline-parallelism taxonomy in Picotron

Picotron is an excellent executable baseline because it exposes the core mechanics without hiding them behind a scheduler compiler. In its current `main` branch, it implements exactly two schedules:

- `train_step_pipeline_afab`: **all-forward/all-backward**, structurally a GPipe-style fill–drain schedule.
- `train_step_pipeline_1f1b`: **synchronous, non-interleaved 1F1B**, equivalent in optimizer semantics to PipeDream-Flush rather than asynchronous PipeDream.

Everything more sophisticated—interleaved 1F1B, Zero Bubble, V schedules, Chimera, DualPipe—can be understood as relaxing one of Picotron’s concrete assumptions.

---

## 1\. The four concrete pieces of Picotron’s pipeline runtime

| Picotron component | What it represents theoretically |
| --- | --- |
| `PipelineParallel` | One contiguous logical stage per physical rank |
| `input_tensors` / `output_tensors` | Activation stash indexed implicitly by FIFO order |
| `model.backward(...)` | One monolithic backward operation |
| `pipeline_communicate(...)` | Blocking point-to-point pipeline edge |
| `bidirectional_pipeline_communicate(...)` | Paired send/receive on one edge, not DualPipe-style bidirectionality |
| `optimizer.step()` outside the schedule | Synchronous flush semantics; one weight version per iteration |

The stage module distributes contiguous decoder-layer ranges by layer count. Only physical PP rank 0 owns the embedding, and only physical PP rank owns the final norm and vocabulary projection. The `forward` method executes the entire local stage as one indivisible block.

This produces the topology

```
rank 0                 rank 1                  rank p-1
embedding              decoder layers         decoder layers
decoder layers   -->   ...              -->   final norm
                                                LM head
                                                loss
```

There are no virtual stages, folded stages, model replicas, or multiple chunks per rank.

The outer training loop calls one complete pipeline schedule and only then invokes `optimizer.step()`. Consequently, all microbatches in the accumulation window use the same parameter version. This is synchronous minibatch SGD/Adam semantics, not asynchronous PipeDream semantics.

---

## 2\. Picotron AFAB is GPipe-style fill–drain

The implementation is almost the literal schedule:

```python
for microbatch in microbatches:
    recv_forward()
    output = forward()
    send_forward()
    save(input, output)

for microbatch in microbatches:
    output_grad = recv_backward()
    input, output = pop_oldest()
    input_grad = backward(input, output, output_grad)
    send_backward(input_grad)
```

Picotron calls this AFAB rather than GPipe, which is a useful distinction: it implements the **schedule shape** of GPipe, but not GPipe’s activation-rematerialization machinery. It stores every input/output pair and the autograd graph reachable from every output.

Let

- : number of PP ranks.
- : `data_loader.grad_acc_steps`.
- : balanced per-stage forward time.
- : balanced per-stage backward time.

Ignoring communication,

Per rank, the useful work is

so the schedule bubble is

with wall-clock bubble fraction

### Activation residency

Every rank stores outstanding autograd graphs before beginning backward:

This is the central weakness of the implementation. `output_tensors` does not merely retain a boundary activation; each output retains the local autograd graph and the intermediate tensors needed by its stage’s backward pass.

Picotron’s backward order is FIFO:

```
F0 F1 F2 ... Fm-1 | B0 B1 B2 ... Bm-1
```

rather than LIFO. That is numerically valid because the microbatch graphs are independent. It also frees the longest-lived graph first, although peak memory has already been reached by the time backward begins.

---

## 3\. Picotron’s 1F1B schedule, derived directly from the code

The central formula in the implementation is

where is the zero-based PP rank. `w_r` is the number of forward-only warm-up microbatches for that rank. The remaining count is

The local execution order is:

1. Run warm-up forwards.
2. Repeat times:
	- Run one new forward.
		- Run backward for the oldest outstanding microbatch.
3. Run the remaining backwards.

That is precisely what the FIFO `input_tensors` and `output_tensors` queues implement.

## Concrete example:

Here `B_i` means Picotron’s complete monolithic `torch.autograd.backward` for microbatch .

| Rank | Warm-up | Steady state | Cooldown | Maximum live graphs |
| --- | --- | --- | --- | --- |
| 0 | `F0 F1 F2` | `F3 B0 F4 B1 F5 B2 F6 B3 F7 B4` | `B5 B6 B7` | 4 |
| 1 | `F0 F1` | `F2 B0 F3 B1 F4 B2 F5 B3 F6 B4 F7 B5` | `B6 B7` | 3 |
| 2 | `F0` | `F1 B0 F2 B1 F3 B2 F4 B3 F5 B4 F6 B5 F7 B6` | `B7` | 2 |
| 3 | — | `F0 B0 F1 B1 F2 B2 F3 B3 F4 B4 F5 B5 F6 B6 F7 B7` | — | 1 |

The exact maximum number of outstanding graph slots on rank is

Thus rank 0 holds at most graphs rather than , and the last stage ordinarily holds only one.

This is the primary theoretical advantage of Picotron’s 1F1B implementation.

## Important correction to the code comment

The docstring says 1F1B “improves GPU utilization.” That is not generally the fundamental difference from AFAB.

Under balanced stages, no communication, and identical ,

Synchronous non-interleaved 1F1B and fill–drain have essentially the same idealized fill/cooldown bubble. The direct advantage is lower activation residency. Throughput improves indirectly when that memory reduction permits:

- A larger microbatch.
- More microbatches.
- Less activation offload.
- Less aggressive recomputation.

Picotron’s 1F1B also combines opposite-direction transfers during steady state, which can help communication efficiency, but that is separate from the geometric schedule bubble.

## What happens when

Because of the `min(...)`, an early rank may have

That rank performs no 1F1B steady state at all; locally it degenerates to

```
all forward | all backward
```

This makes the usual rule concrete: the microbatch count needs to be at least comparable to pipeline depth for 1F1B to have a substantial steady-state region.

---

## 4\. The exact steady-state handshake

For an intermediate stage, one Picotron steady-state iteration is:

```
1. Compute forward F_j

2. With next rank:
   send activation Y_j
   receive output gradient dY_i
   wait for both

3. Pop oldest saved graph
   compute full backward B_i
   obtain input gradient dX_i

4. With previous rank:
   send dX_i
   receive activation X_{j+1}
   wait for both
```

This is encoded by:

```python
send_fwd_recv_bwd(...)
...
send_bwd_recv_fwd(...)
```

At the endpoints:

- The last stage skips `send_fwd_recv_bwd`, uses `output_tensor_grad=None`, and begins backward from the scalar loss using a gradient of one.
- The first stage skips `send_bwd_recv_fwd`; its next forward starts from the next local `input_ids`.

This is a clean implementation of the standard 1F1B communication protocol.

## “Bidirectional” here does not mean DualPipe

Picotron’s function name `bidirectional_pipeline_communicate` can be misleading.

It means:

```
send one tensor and receive another tensor
to/from the same neighboring rank
in one batch_isend_irecv call
```

It does **not** mean:

- Two model pipelines moving in opposite physical directions.
- Two model replicas.
- A V-shaped stage placement.
- Concurrent forward from one stream and backward from another stream.
- DeepSeek’s DualPipe.

It is merely full-duplex communication on a conventional linear pipeline edge.

---

## 5\. Why Picotron currently has no compute/communication overlap

The P2P helpers use asynchronous-looking primitives:

```python
dist.isend
dist.irecv
dist.batch_isend_irecv
```

but immediately do:

```python
[req.wait() for req in reqs]
torch.cuda.synchronize()
```

Therefore, no communication handle survives into the next computation. Every exchange is completed, followed by a device-wide synchronization, before Python proceeds.

For a middle stage, the steady-state critical path is therefore approximately

where:

- is the paired forward-send/backward-receive exchange.
- is the paired backward-send/forward-receive exchange.

The send and receive inside one `batch_isend_irecv` may overlap with each other, depending on the transport, but neither overlaps with or .

This distinction is critical for interpreting DualPipe:

> Replacing Picotron’s schedule order with a DualPipe-looking action sequence would not produce DualPipe’s intended overlap while these waits and global synchronizations remain.

A production runtime would instead maintain communication handles and CUDA events:

```
launch recv for activation
launch recv for gradient
run ready compute
wait only when a specific tensor is consumed
```

It would also ordinarily use dedicated streams for:

- PP sends and receives.
- TP collectives.
- EP all-to-all.
- DP/FSDP reductions.
- Forward and backward compute where genuine resource complementarity exists.

Picotron additionally does not pass an explicit PP process group into `P2POp`; its operations use global peer ranks on the default communicator. That can be correct, but a dedicated PP communicator is preferable for topology control and isolation from unrelated collective ordering.

---

## 6\. The backward operation is the boundary between 1F1B and Zero Bubble

Picotron’s complete backward engine is:

```python
torch.autograd.backward(
    output_tensor,
    grad_tensors=output_tensor_grad,
    retain_graph=False,
    create_graph=False,
)
return input_tensor.grad
```

This computes, as one indivisible action:

where:

- : input-gradient computation, producing for the preceding stage.
- : weight-gradient computation, producing for local parameters.

There is no scheduler-visible distinction between them.

## Why Picotron cannot become Zero Bubble through loop reordering alone

A Zero-Bubble schedule needs actions resembling:

```
F(microbatch, stage)
I(microbatch, stage)
W(microbatch, stage)
```

It propagates promptly because is on the inter-stage critical path, while moving into otherwise idle schedule slots.

Picotron has only:

```
F(microbatch, stage)
B(microbatch, stage)
```

To add ZB1P, `model.backward` must be replaced with something structurally like:

```python
saved_w_state, input_grad = backward_input(...)
...
backward_weight(saved_w_state)
```

This change has several consequences:

1. **The activation stash changes.** After , the runtime cannot discard everything because still needs saved activations and output gradients.
2. **FIFO queues become insufficient.** Scheduled operations can be significantly reordered, so state should be keyed by `(stage_id, microbatch_id)`.
3. **Autograd must be partitioned.** A naive sequence such as
	```python
	autograd.grad(output, input, retain_graph=True)
	autograd.grad(output, parameters)
	```
	is not generally equivalent to an efficient split; it can duplicate substantial backward computation.
4. **DP synchronization moves.** Picotron’s DP buckets currently become ready during the monolithic final backward. Under Zero Bubble, parameter gradients become ready during deferred tasks.

The practical implementation options are:

- Custom backward implementations for transformer modules.
- AOTAutograd/compiled-backward graph partitioning.
- Reusing PyTorch’s pipeline backward partitioning rather than manually splitting generic autograd.

The third path is the most sensible way to extend an educational codebase without recreating a substantial compiler subsystem.

---

## 7\. The DP/PP integration is subtle and mostly handled correctly

Picotron’s data-parallel wrapper accumulates each microbatch’s parameter gradients into `param.main_grad`. Bucket all-reduce is disabled for earlier microbatches and enabled only for the final local backward. The all-reduce itself is asynchronous and begins when all parameters in a bucket are marked ready.

The complication is that the final microbatch’s backward occurs in different schedule phases:

- On the last PP stage, , so the final backward occurs in the final steady-state iteration.
- On earlier stages, the final backward occurs during cooldown.

The code handles both cases:

```python
if num_warmup_microbatches == 0 and is_last_iteration:
    model.require_backward_grad_sync = True
```

and later:

```python
model.require_backward_grad_sync =
    ith_warmup_microbatches == num_warmup_microbatches - 1
```

The history in PR #5 shows that this behavior was added deliberately after earlier versions either failed to synchronize a stage or synchronized before the complete accumulation window was available.

This illustrates a broader principle:

> “Final accumulated microbatch” is a dependency concept, not simply “last loop iteration” in one shared loop. Different stages encounter it in different local phases.

For Zero Bubble, interleaved schedules, or V schedules, hard-coded phase checks become fragile. Gradient synchronization should instead be triggered by an explicit action such as:

```
W(stage=s, microbatch=m-1, marks_iteration_complete=True)
```

---

## 8\. A source-level gradient-scaling discrepancy

There is a notable correctness discrepancy between the non-pipeline and pipeline paths.

The non-pipeline path computes:

```python
loss = cross_entropy(...) / data_loader.grad_acc_steps
loss.backward()
```

so the accumulated gradient is the mean of the microbatch-mean gradients.

Both PP paths instead compute:

```python
output_tensor = cross_entropy(...)
logging_loss += output_tensor.item() / data_loader.grad_acc_steps
```

The division affects only `logging_loss`; the scalar passed to `model.backward` remains unscaled.

Therefore, with ,

while the no-PP path computes

Unless learning rate is compensated externally, the pipeline gradient is approximately times larger. The data-parallel bucket divides by the CP/DP process-group size, but it does not divide by . The configuration generator likewise does not apply a PP-engine-specific learning-rate adjustment.

The direct fix in both pipeline paths is:

```python
raw_loss = F.cross_entropy(
    output_tensor.flatten(0, 1),
    batch["target_ids"].to(device).flatten(),
    reduction="mean",
)

logging_loss += raw_loss.detach().item() / data_loader.grad_acc_steps
output_tensor = raw_loss / data_loader.grad_acc_steps
```

This discrepancy is source-level and does not require a performance benchmark to establish.

---

## 9\. Mapping the advanced techniques onto concrete Picotron changes

## 9.1 Interleaved 1F1B

### Picotron assumption that must change

```
one physical rank = one logical stage = one contiguous layer range
```

Interleaved 1F1B requires multiple logical chunks per physical rank. With , cyclic placement might be:

```
logical stage:  0  1  2  3  4  5  6  7
physical rank:  0  1  2  3  0  1  2  3
```

Picotron would need:

- Multiple `StageChunk` modules per rank.
- Separate activation stores per chunk.
- Logical-stage-specific first/last roles.
- Logical-stage-specific predecessor and successor mappings.
- Schedule actions keyed by chunk and microbatch.

The current scalar fields

```python
pp_prev_rank
pp_next_rank
pp_is_first_stage
pp_is_last_stage
```

are insufficient. They describe a physical rank’s position in one linear chain, not a logical stage graph.

For example, in cyclic placement the logical transition

```
stage 3 on rank 3 -> stage 4 on rank 0
```

cannot be represented by rank 3’s current `pp_next_rank=None`.

Interleaving would reduce geometric bubble by creating smaller logical stages, but it would also increase pipeline boundary count and P2P message frequency. Given Picotron’s per-exchange synchronization, interleaving should not be implemented before the communication runtime is made asynchronous.

---

## 9.2 Looped BFS

Looped BFS uses essentially the same multi-chunk ownership as interleaved 1F1B but changes the local scheduling priority.

Picotron currently follows a depth-first microbatch policy:

```
advance the oldest microbatch and then backward it
```

A breadth-first implementation would run a group of available microbatches through one local chunk before switching chunks.

This is particularly useful with FSDP because a stage can be:

```
unsharded once
used by several microbatches
resharded once
```

Picotron currently wraps stages with its own replicated `DataParallelBucket`, not FSDP. Therefore, Looped BFS’s major parameter-materialization advantage is absent in the current stack. Implementing it before FSDP would mostly be a scheduling experiment rather than a clear systems win.

---

## 9.3 ZB1P

Required changes:

```
monolithic B
    ↓
separate I and W

FIFO activation queue
    ↓
state indexed by stage and microbatch

phase-based Python loops
    ↓
explicit schedule action list

DP readiness during B
    ↓
DP readiness during W
```

Stage placement can remain exactly as in current Picotron, so PP communication volume need not increase. This makes ZB1P the most natural post-1F1B advanced schedule for this codebase.

---

## 9.4 ZB-V

For physical ranks, a two-chunk V placement is:

```
logical stage:  0  1  2  ... p-1 | p  p+1 ... 2p-1
physical rank:  0  1  2  ... p-1 | p-1 ... 1  0
```

This breaks several Picotron assumptions simultaneously.

### Rank 0 becomes both the beginning and end

Under V placement:

- Logical stage 0, containing the embedding, is on rank 0.
- Logical stage , containing the output head and loss, is also on rank 0.

Picotron currently attaches the output head to physical rank . The roles must move from rank-level booleans to logical-stage metadata.

### A rank has multiple neighbors and a local turning edge

At the V turning point:

```
logical stage p-1 -> logical stage p
```

both chunks reside on physical rank . That edge should be a local tensor handoff, not NCCL P2P.

On the descending side, rank direction reverses:

```
rank p-1 -> rank p-2 -> ... -> rank 0
```

The current global `pp_next_rank` and `pp_prev_rank` cannot express this.

### PP traffic increases

At fixed physical device count, the model has smaller logical stages rather than stages. That approximately doubles cross-rank pipeline edges, aside from the local turning edge. With Picotron’s current blocking/synchronizing communication path, this extra traffic would likely erase much of the theoretical schedule gain.

---

## 9.5 Chimera

Chimera would require two complete logical model replicas:

```
replica A: rank 0 -> rank 1 -> ... -> rank p-1
replica B: rank p-1 -> ... -> rank 1 -> rank 0
```

Each physical rank would own one stage from each replica. This entails:

- Two parameter-stage copies per rank.
- Two activation stores.
- Two microbatch streams.
- Loss computation at both physical ends.
- Gradient reconciliation between corresponding replica parameters.

Nothing in Picotron’s current `PipelineParallel` wrapper or optimizer ownership supports duplicate stage replicas. Chimera is therefore a model-placement change, not merely a new loop around the existing `forward` and `backward`.

---

## 9.6 DualPipe and DualPipeV

DualPipe requires all of the following:

1. Bidirectional logical streams or a V-folded equivalent.
2. Separate input-gradient and weight-gradient work.
3. Asynchronous communication handles.
4. Multiple CUDA streams and event dependencies.
5. Model-specific decomposition that can genuinely overlap communication and compute.
6. For DeepSeek-style benefits, substantial MoE expert-parallel communication to hide.

Picotron currently has:

```
one monolithic model.forward
one monolithic autograd backward
blocking P2P
device-wide synchronization after every exchange
dense Llama stages
```

Therefore, implementing only a DualPipe schedule diagram would produce a superficial DualPipe-like ordering, not DeepSeek’s systems result.

For a dense Picotron Llama stage, concurrently launching forward and backward GEMMs may simply make two tensor-core-heavy workloads contend for the same SMs and memory bandwidth. DualPipe’s value becomes much more plausible when one stream’s expert all-to-all communication can be hidden under another stream’s attention or expert computation.

DualPipeV would first require the same logical-stage refactor as ZB-V and then the much harder and overlap machinery.

---

## 9.7 Asynchronous PipeDream

Original PipeDream changes optimizer semantics. Picotron currently does:

```
zero_grad
run complete flushed pipeline
optimizer.step
```

Asynchronous PipeDream would require something closer to:

```
stage-local forward under weight version k
...
stage-local backward under matching weight version k
stage-local optimizer update
continue without globally draining
```

That means:

- Weight stashing or bounded weight versions.
- Optimizer state updates inside the schedule.
- Staleness semantics.
- More complicated checkpointing.
- Revised DP synchronization.

It cannot be implemented as a small variant of `train_step_pipeline_1f1b`.

---

## 10\. Additional implementation constraints exposed by the code

## Stage balance is based on layer count, not measured time

`distribute_layers` divides decoder layers as evenly as possible and gives remainder layers to earlier ranks. But:

- The first stage additionally owns the embedding.
- The last stage owns final normalization, vocabulary projection, and loss.
- TP/CP can alter the cost of different modules.
- Attention and MLP costs may not be identical across configurations.

Thus equal layer count does not imply

or balanced backward.

A practical partitioner should profile candidate boundaries and minimize the maximum stage time, subject to memory constraints:

This optimization should happen before investing in increasingly sophisticated schedules. A severe stage imbalance creates recurring idle time that Zero Bubble cannot remove.

## Fixed communication shape

The runtime uses one global shape:

```python
(
    micro_batch_size,
    seq_length_per_gpu,
    hidden_size,
)
```

for every pipeline edge.

This assumes:

- Fixed-size microbatches.
- Identical hidden width at every boundary.
- No multimodal or encoder–decoder shape transitions.
- No dynamic sequence lengths.
- No stage-specific tensor structures.

Because the DataLoader does not set `drop_last=True`, a final undersized batch is also a latent mismatch unless the per-replica dataset length is divisible by the configured microbatch size. A production runtime needs either `drop_last=True` or shape metadata preceding dynamic transfers.

## Data is independently consumed by every PP rank

Every stage calls `next(data_loader)` during each forward. The sampler shards only by `dp_rank`, so PP ranks in the same DP replica ordinarily see the same sample sequence. CP ranks then select different sequence slices.

This avoids transporting labels and position IDs through the pipeline, but it creates an implicit invariant:

```
all ranks must advance their data iterators in exactly the same microbatch order
```

Current AFAB and 1F1B satisfy this. A dynamic scheduler, replay mechanism, fault recovery path, or two-stream DualPipe implementation should use explicit microbatch IDs rather than relying only on FIFO iterator progression.

The current `_forward_step` also moves `input_ids` to the GPU on every PP stage even though non-first stages ignore them because `hidden_states` is non-null. That is unnecessary host-to-device traffic.

## Output pseudo-deallocation remains unimplemented

The code explicitly notes a TODO to deallocate the boundary output tensor after sending it.

This should be treated carefully. Megatron-style pseudo-deallocation is not simply:

```python
output_tensor.data = tiny_scalar
```

followed by ordinary `torch.autograd.backward`. The backward entrypoint must bypass or accommodate output-shape validation while preserving the graph and saved internal tensors. It is useful, but it needs a custom backward path.

## CP-only gradient synchronization deserves a dedicated test

The schedule tests `cp_dp_world_size > 1`, and `DataParallelBucket` synchronizes over the combined CP/DP group. However, `train.py` installs `DataParallelBucket` only when `dp_world_size > 1`. A configuration with

```
CP > 1
DP = 1
```

therefore does not receive that wrapper, even though replicated non-attention parameters generally require CP reduction. I did not find a separate parameter-gradient reduction in the inspected context-parallel wrapper. This is adjacent to PP rather than a pipeline-schedule problem, but it should be covered by a numerical-equivalence test.

## Physical rank topology is not abstracted

The process grid is currently arranged as:

```python
grid.view(dp_size, pp_size, cp_size, tp_size)
```

and PP neighbors are derived from adjacent `pp_rank` positions.

The repository has an open issue discussing whether rank ordering should be changed to keep higher-volume data-parallel traffic on faster links and place pipeline traffic across nodes.

The broader principle is that schedule selection and physical placement are coupled:

- Interleaved and V schedules increase PP edge count.
- TP usually requires the strongest intra-node connectivity.
- FSDP/DP communication volume can dominate PP.
- MoE EP all-to-all may dominate everything else.

A scheduler should operate on logical stages; a separate placement layer should map those stages to the physical topology.

---

## 11\. The refactor that unlocks all schedules

Rather than adding more specialized Python loops alongside `train_step_pipeline_afab` and `train_step_pipeline_1f1b`, Picotron should introduce four abstractions.

## 11.1 Logical stage

```python
@dataclass
class LogicalStage:
    stage_id: int
    module: nn.Module
    physical_rank: int
    prev_stage_id: int | None
    next_stage_id: int | None
    is_model_input: bool
    is_model_output: bool
```

This removes the incorrect equivalence between physical rank and model-stage role.

## 11.2 Explicit action IR

```python
@dataclass
class Action:
    kind: Literal[
        "RECV_F",
        "F",
        "SEND_F",
        "RECV_B",
        "B",
        "I",
        "SEND_B",
        "W",
    ]
    stage_id: int
    microbatch_id: int
```

Then schedules become action generators:

```
AFAB           -> F* then B*
1F1B           -> warm-up, alternating F/B, cooldown
Interleaved    -> same actions over multiple chunks
ZB1P           -> F/I/W ordering
ZB-V           -> F/I/W over V stage graph
DualPipeV      -> paired/overlapped actions over V stage graph
```

## 11.3 State indexed by stage and microbatch

Replace parallel FIFO lists with:

```python
slots[(stage_id, microbatch_id)] = StageState(...)
```

FIFO is sufficient only because current AFAB and 1F1B schedules have a deterministic oldest-first backward order.

## 11.4 Runtime dependency engine

The runtime should:

- Prepost receives.
- Return communication handles instead of immediately waiting.
- Wait only at tensor-consumption boundaries.
- Use CUDA events rather than device-wide synchronization.
- Distinguish local edges from cross-rank edges.
- Emit a trace with stage, microbatch, action, stream, and timestamps.
- Validate that each send has exactly one matching receive.

The backward engine can initially expose only `B`; later it can add `I/W` without changing the stage graph or communication runtime.

---

## 12\. Recommended implementation order

The most defensible progression for Picotron is:

### 1\. Establish numerical correctness

Before adding schedules:

- Fix gradient-accumulation loss scaling.
- Compare no-PP, AFAB, and 1F1B gradients from identical weights and microbatches.
- Test , , and .
- Test DP+PP so every stage synchronizes only after its final accumulated backward.
- Assert post-optimizer parameter equality, not only loss equality.

### 2\. Instrument the existing schedules

Record:

```
rank
logical stage
microbatch
F/B start and end
send/recv launch and completion
activation queue depth
gradient bucket launch and completion
```

This gives measured and exposes imbalance.

### 3\. Fix the runtime before adding more PP boundaries

- Remove unconditional `torch.cuda.synchronize()`.
- Use explicit PP groups.
- Prepost receives.
- Wait on use.
- Support per-edge tensor specifications.
- Add output pseudo-deallocation and activation recomputation where appropriate.

### 4\. Add interleaved 1F1B

This validates:

- Multiple chunks per rank.
- Logical stage topology.
- Per-chunk state.
- Nontrivial communication ordering.

It does not yet require backward graph partitioning.

### 5\. Add ZB1P

Keep the existing linear stage placement but introduce . This isolates the hardest new primitive—backward partitioning—without simultaneously changing topology.

### 6\. Add ZB-V

Once logical-stage placement and are stable, assign two V-shaped chunks per physical rank and evaluate the memory/communication frontier.

### 7\. Add DualPipeV only alongside a workload with communication worth hiding

For dense Picotron Llama training, ZB1P or ZB-V is the more natural endpoint. DualPipeV becomes justified when the framework also has:

- MoE layers.
- Large cross-node EP all-to-all.
- Custom communication kernels.
- Stream/SM resource controls.
- A measured opportunity for complementary compute/communication overlap.

---

## Final assessment

Picotron’s code makes the technique ladder unusually clear:

```
AFAB
  = retain m graphs, all F then all B

1F1B
  = retain at most min(m, p-r) graphs,
    alternate new F with oldest full B

Interleaved 1F1B
  = give each rank multiple logical chunks

Zero Bubble
  = replace full B with critical I plus deferrable W

ZB-V
  = add two V-mapped chunks per rank

DualPipeV
  = add bidirectional workload pairing and real
    module-level compute/communication overlap
```

The most important grounded conclusions are:

1. **Picotron 1F1B is primarily a memory optimization over AFAB, not an intrinsic reduction in the ideal fill/drain bubble.**
2. **Its “bidirectional” communication is paired P2P, not a bidirectional model pipeline.**
3. **Immediate waits and `torch.cuda.synchronize()` prevent compute/communication overlap.**
4. **The monolithic `torch.autograd.backward` is the precise architectural barrier to Zero Bubble.**
5. **Physical-rank-level first/last and neighbor metadata is the precise barrier to interleaved and V-shaped placement.**
6. **A source-level loss-scaling discrepancy should be corrected before comparing schedule throughput or convergence.**
7. **DualPipe should be the last step, not the next step: without MoE communication and custom overlap, its schedule alone is unlikely to recreate DeepSeek’s advantage.**

---

**You said**

from the first principle, how do I think of how to develop these pipeline parallelism algorithms and systems? what is the design space, how do we navigate this space; can we simulate the system performance given any pipeline parallelism algorithms? how do we manually or automatically discover a good system

---

**ChatGPT said**

## First-principles view: pipeline parallelism is constrained program scheduling

The most productive mental model is:

> A pipeline-parallel training algorithm is a transformation of a training computation graph into a distributed, resource-constrained, repeated execution plan.

It is not fundamentally “GPipe versus 1F1B versus Zero Bubble.” Those are particular points in a much larger space.

Given:

the system must produce:

Picotron corresponds to one particularly simple point:

Its `PipelineParallel` class assigns contiguous decoder-layer ranges to ranks; its backward is a monolithic `torch.autograd.backward`; and its communication functions immediately wait and call `torch.cuda.synchronize()`.

Every advanced algorithm we discussed can be derived by relaxing one of those decisions.

---

## 1\. Begin with the semantic contract

Before scheduling anything, define what training algorithm must be preserved.

## 1.1 Synchronous semantics

All microbatches in an optimizer step use the same model version:

GPipe, PipeDream-Flush, synchronous 1F1B, interleaved 1F1B, Zero Bubble, ZB-V, and DualPipe normally target this semantic contract.

The scheduler may reorder computations, but it must not alter:

- Which weights each forward uses.
- Which forward state each backward uses.
- Which gradients enter each optimizer step.
- Gradient scaling.
- RNG and dropout behavior.
- Collective reduction semantics.

## 1.2 Asynchronous semantics

A system such as original PipeDream permits stage-local updates while later microbatches are still in flight. That changes the algorithm:

The scheduler must now represent:

- Parameter versions.
- Optimizer-state versions.
- Forward/backward version matching.
- Stage-specific staleness.

A pure systems simulator can predict time per sample, but not necessarily time to target quality. For asynchronous algorithms, the objective must include a convergence model or actual training experiments. PipeDream explicitly combines pipelining with parameter versioning to support its asynchronous execution model. [^15]

## 1.3 Why semantics come first

Many attractive schedules are only “better” because they silently weaken semantics. You should therefore optimize in this order:

1. Fix the semantic contract.
2. Enumerate valid transformations under that contract.
3. Optimize performance within the resulting legal space.

---

## 2\. Represent one training iteration as a global DAG

Let

be a directed acyclic graph for one optimizer iteration.

A node is an executable action. An edge is a true data dependency or an intentionally imposed temporal constraint.

For logical stage and microbatch , the useful action vocabulary is:

- : forward.
- : backward for the stage input, producing .
- : backward for local weights, producing .
- , : forward activation send and receive.
- , : backward-gradient send and receive.
- : gradient all-reduce or reduce-scatter.
- : optimizer update.
- : parameter all-gather when using FSDP or ZeRO-3.
- : expert dispatch/combine all-to-all for MoE.

For a linear model, the principal dependencies include:

and in reverse:

Both and require:

- The forward state from .
- The output gradient arriving from the downstream stage.

A particular backward implementation may additionally impose:

Zero Bubble exploits the fact that is on the inter-stage critical path while is local and can often be deferred. The Zero Bubble work uses this finer action vocabulary and searches schedules using measured , a microbatch count, and a memory limit. [^16]

## The critical architectural rule

**Communication must be explicit in this graph.**

Do not represent a stage as:

```
forward_with_hidden_communication()
```

Represent it as:

```
F -> SEND -> RECV -> F
```

Otherwise the scheduler cannot reason about:

- Communication overlap.
- Link contention.
- Collective ordering.
- Buffer memory.
- Topology-sensitive placement.
- Deadlocks.

Piper adopts exactly this global-DAG approach: compute and communication are explicit nodes, and user directives transform the graph into a distributed execution plan. [^17]

---

## 3\. Every task needs a resource and memory model

A dependency DAG alone is insufficient. Two dependency-independent tasks may still be unable to execute concurrently because they contend for the same hardware.

Associate every task with:

A useful GPU resource-demand vector is:

For example:

- A large MLP GEMM may saturate tensor cores and consume considerable HBM bandwidth.
- An NCCL all-to-all may consume NIC, NVLink, copy engines, SMs, and HBM bandwidth.
- An optimizer kernel may consume HBM bandwidth but relatively few tensor-core resources.
- A P2P send may occupy a communication stream and link while using a limited number of SMs.

This explains why “put operations on separate CUDA streams” does not imply useful overlap. If both operations demand essentially the full tensor-core and HBM resources, concurrent execution may take as long as or longer than serial execution.

Piper distinguishes the high-level strategy—which determines minimum per-device compute, communication, and memory load—from the low-level execution strategy, which determines how closely the runtime approaches that bound. It also documents concrete interference between different forms of communication in composed PP/DP/EP strategies. [^17]

---

## 4\. Memory is a temporal resource

Pipeline activation memory is determined by tensor lifetime, not merely by activation size.

Suppose forward creates saved state . It remains live until its final consumer completes:

Usually:

Peak memory on device is therefore:

This is why the same compute work can have very different memory requirements under AFAB, 1F1B, ZB1P, or V-Half.

The controllable-memory pipeline work formalizes schedules as repeated building blocks and relates activation memory to the overlap of their lifespans. For stage with lifespan , repeating interval , and per-microbatch activation , it gives a bound of the form

This is a useful first-principles design tool: reduce or balance lifespans, rather than merely counting forwards in the warm-up. [^18]

---

## 5\. The full design space

A useful decomposition is the following.

| Dimension | Decisions | Main trade-off |
| --- | --- | --- |
| **Semantics** | Synchronous, bounded staleness, asynchronous | Utilization versus optimization behavior |
| **Graph granularity** | Full backward, , layer, sublayer, operator, kernel | Search freedom versus runtime/compiler complexity |
| **Partitioning** | Contiguous stages, heterogeneous cuts, graph cuts, replicated stages | Balance versus communication and parameter memory |
| **Placement** | Linear, cyclic, V-shaped, reversed replicas, topology-aware mapping | Bubble/memory balance versus communication |
| **Microbatching** | Count, size, sequence split, token-level split, variable shapes | Pipeline occupancy versus arithmetic intensity and memory |
| **Ordering** | AFAB, 1F1B, interleaved DFS, BFS, ZB, custom partial orders | Critical-path reduction versus activation lifetime |
| **Memory policy** | Retain, recompute, offload, migrate, unshard/reshard | Memory versus compute and communication |
| **Communication** | Blocking/asynchronous, bucket sizes, algorithms, channels, streams | Latency, bandwidth, overlap, contention |
| **Resource overlap** | F/B, compute/comm, DP/EP/PP collectives | Hidden latency versus interference |
| **Runtime policy** | Static schedule, dynamic list scheduler, adaptive schedule | Predictability versus robustness to variability |

There are also interactions with TP, CP, DP, EP, ZeRO, checkpointing, compiler fusion, and physical rank placement. These should not be modeled as independent knobs because they share GPU memory, streams, links, and collective communicators.

---

## 6\. Known algorithms as graph transformations

This is the easiest way to understand—and invent—pipeline algorithms.

## AFAB to 1F1B

No graph or placement change.

Only change the local ordering:

```
AFAB:
F0 F1 F2 F3 | B0 B1 B2 B3

1F1B:
F0 F1 | F2 B0 | F3 B1 | B2 B3
```

The transformation shortens activation lifetimes but does not fundamentally remove the balanced fill/drain bubble.

## 1F1B to interleaved 1F1B

Apply:

```
split each physical stage into v logical chunks
place chunks cyclically across ranks
schedule all chunks using depth-first priority
```

This reduces logical stage duration, which reduces bubble duration, but increases the number of cross-rank boundaries.

## 1F1B to Zero Bubble

Apply:

```
B -> I + W
```

Then prioritize:

```
I as soon as its output gradient is available
W when a device would otherwise idle or when memory must be released
```

The Zero Bubble paper’s heuristic performs more forward work within the memory limit, prioritizes input-gradient work, and inserts weight-gradient work into gaps or to recycle memory. It also formulates smaller scheduling instances as an ILP. [^16]

## Linear placement to V placement

Apply:

```
split model into 2p stages
place first half left-to-right
place second half right-to-left
```

For four ranks:

```
logical stage:  0 1 2 3 4 5 6 7
physical rank:  0 1 2 3 3 2 1 0
```

This co-locates stages with complementary activation lifetimes, improving memory balance. It also roughly doubles PP boundaries at fixed physical rank count.

## V schedule to DualPipeV

Add:

- Fine-grained forward/backward module decomposition.
- Multiple logical GPU streams.
- Explicit EP and DP communication nodes.
- Co-scheduling rules for forward and backward work.
- Resource-aware communication ordering.

The schedule diagram alone is insufficient; the runtime must realize the intended compute/communication overlap.

## Synchronous to PipeDream-like asynchronous execution

Apply:

```
remove global flush barrier
allow local optimizer updates
introduce parameter versions
track versioned forward states
```

This is a semantic transformation, not just a scheduling transformation.

---

## 7\. Performance lower bounds: know what “optimal” means

Before searching schedules, calculate lower bounds.

## 7.1 Critical-path lower bound

Let be any dependency path through the DAG:

No valid schedule can finish faster than its longest dependency chain.

## 7.2 Per-resource work lower bound

For device and resource :

Examples include:

- Total tensor-core work divided by tensor-core capacity.
- Total HBM bytes divided by sustainable HBM bandwidth.
- Total optimizer bytes divided by optimizer-kernel bandwidth.

## 7.3 Network lower bound

For physical link :

where is the total traffic routed across the link and is usable bandwidth.

## 7.4 Combined bound

Then define schedule efficiency:

This is more informative than “bubble ratio,” because it captures:

- Imbalanced stages.
- Exposed communication.
- Resource interference.
- Optimizer tails.
- Runtime stalls.

A schedule can have zero nominal bubble and still be far from this lower bound.

---

## 8\. Can arbitrary pipeline algorithms be simulated?

## The precise answer

**Any algorithm that can be represented as a finite or periodic task graph, together with resource, memory, and timing semantics, can be simulated.**

That includes:

- Static synchronous schedules.
- Interleaved and V-shaped schedules.
- Zero Bubble.
- Bidirectional pipelines.
- Asynchronous pipelines with parameter versions.
- Dynamic ready-queue schedulers.
- Graph pipelines with branches.
- MoE workloads with stochastic routing.
- Failure and recovery policies.

What cannot be predicted accurately without calibration is the duration and interference behavior of entirely novel kernel and communication combinations.

So there are two distinct simulators:

1. **Correctness simulator:** Is the action ordering legal and deadlock-free?
2. **Performance simulator:** How long will it take, how much memory will it consume, and what resources will be contended?

PyTorch’s schedule IR follows a compute-trace → communication-insertion → runtime architecture and includes simulation tooling primarily for schedule correctness and ordering validation. Its IR distinguishes full backward from backward-input and backward-weight actions. [^6]

---

## 9\. A discrete-event performance simulator

The cleanest implementation is an event-driven simulator.

## 9.1 Core task representation

```python
Task(
    id,
    kind,                 # F, I, W, SEND, RECV, AR, A2A, OPT...
    stage_id,
    microbatch_id,
    parameter_version,
    device_or_mesh,
    predecessors,
    resource_demands,
    duration_model,
    allocations,
    deallocations,
    stream,
)
```

## 9.2 Simulator state

```python
SimulationState(
    virtual_time,
    completed_tasks,
    ready_tasks,
    running_tasks,
    device_resources,
    link_resources,
    live_tensors,
    current_memory,
    peak_memory,
    parameter_versions,
)
```

## 9.3 Basic event loop

```
while unfinished tasks remain:

    identify tasks whose dependencies are satisfied

    launch tasks allowed by:
        schedule policy
        resource availability
        memory budget
        stream ordering
        collective ordering

    estimate the progress rate of every running task

    advance virtual time to the next task completion

    mark completed tasks
    release resources
    allocate/free tensors
    unlock successor tasks
```

For a simple non-overlapping model, the finish time is:

For realistic overlap, fixed task durations are insufficient. The instantaneous progress rate should depend on co-running tasks:

For example, an all-to-all might run at full speed alone but at 55% speed when sharing links with a data-parallel all-reduce.

---

## 10\. Communication modeling

## 10.1 First-order model

For one uncongested P2P message:

where:

- : launch and path latency.
- : achieved bandwidth.
- : message bytes.

This is sufficient for initial schedule ranking when messages are large and links are lightly contended.

## 10.2 Topology-aware model

Represent the cluster as:

where devices are vertices and links are edges with:

- Bandwidth.
- Latency.
- Duplex properties.
- NVLink, PCIe, InfiniBand, or Ethernet type.
- Shared switch or rail capacity.
- Routing policy.

Each communication operation generates one or more flows. Shared-link capacity is allocated among active flows.

## 10.3 Collective model

Collectives require:

- Algorithm choice: ring, tree, recursive doubling, hierarchical.
- Number of channels.
- Intra-node and inter-node decomposition.
- Message-size-dependent algorithm selection.
- NIC and NVLink contention.

For a research simulator, start with empirically profiled collective curves:

Only move to packet- or flow-level simulation when placement and cross-traffic are central to the research question.

---

## 11\. A fidelity ladder

Do not begin with the highest-fidelity simulator. Use a hierarchy.

## Level 0: symbolic and closed-form

Inputs:

- Balanced durations.
- Constant communication.
- Uniform memory.

Useful for:

- Rejecting obviously poor schedules.
- Deriving bubble formulas.
- Understanding scaling with .

Runtime: effectively instantaneous.

## Level 1: profile-calibrated discrete-event simulation

Inputs:

- Per-stage durations.
- P2P and collective profiles.
- Tensor lifetimes.
- Static resource exclusion rules.

Useful for:

- Stage partitioning.
- Microbatch search.
- Comparing 1F1B, interleaved, ZB, and V schedules.
- Predicting memory and makespan.

This should be the first serious implementation.

## Level 2: contention-aware simulation

Add:

- Pairwise or multi-task interference models.
- Shared HBM bandwidth.
- Shared links and NICs.
- CUDA stream ordering.
- Allocator stalls.
- Host launch overhead.

Useful for:

- DualPipe.
- PP × EP × DP composition.
- Multiple concurrent collectives.

## Level 3: trace- or framework-driven simulation

Rather than manually reimplementing the framework, execute or trace the real framework and replace GPU/network execution with simulated time.

Phantora is a contemporary example: it intercepts CUDA and communication operations, maintains per-rank virtual clocks, and simulates network execution while reusing the training framework’s code. [^19]

This improves framework fidelity but still requires calibrated kernel and network models.

## Level 4: partial-cluster execution

Run selected candidates on a small real cluster to:

- Correct simulator bias.
- Profile new overlap combinations.
- Validate ranking.
- Update the cost model.

## Level 5: production-scale validation

Only Pareto-optimal candidates should reach this level.

---

## 12\. What to profile

For each stage , microbatch shape , sequence length , and execution mode, measure:

Also measure:

- Peak workspace.
- Forward saved-state size.
- State remaining after for deferred .
- Recompute cost.
- Input and output tensor sizes.
- Gradient-ready times.
- Optimizer and gradient-reduction costs.

For communication, profile:

Then profile co-running pairs:

```
F + P2P
F + all-reduce
F + all-to-all
I + all-to-all
W + all-reduce
all-reduce + all-to-all
```

Record:

and possibly individual slowdown factors:

A pairwise interference matrix is not perfect, but it is far more realistic than assuming all asynchronous operations overlap ideally.

---

## 13\. Validate the simulator as a predictor, not merely a calculator

The main metric is not only absolute timing error. It is also **ranking quality**.

Suppose actual candidates satisfy:

A simulator that predicts all times 8% too high but preserves that order is highly useful for search. A simulator with 2% mean error but frequent ranking inversions near the Pareto frontier is less useful.

Track:

- Absolute percentage error.
- Pairwise ranking accuracy.
- Pareto-frontier recall.
- Peak-memory error.
- Critical-path attribution error.
- Sensitivity derivative error:

The simulator should also output uncertainty:

Candidates with overlapping confidence intervals should be tested on hardware.

---

## 14\. Manual discovery: follow the critical path

A disciplined manual process is:

## Step 1: construct the explicit trace

Start from Picotron’s conceptual action sequence:

```
rank, stage, microbatch, F/B, send/recv
```

Replace implicit calls with explicit actions.

## Step 2: profile and simulate the baseline

Ensure that:

Do not begin schedule invention before the baseline is understood.

## Step 3: calculate the lower bounds

Compare actual time against:

- Critical path.
- Per-device compute load.
- Network load.
- Memory-constrained best case.

## Step 4: attribute every gap

Every idle interval should be classified as one of:

| Observed behavior | Likely cause |
| --- | --- |
| Startup/cooldown idle regions | Pipeline dependency bubble |
| One stage continually busy while neighbors idle | Stage imbalance |
| GPU waits immediately before F or B | Exposed PP communication |
| Network is active but compute slows greatly | Compute/communication interference |
| Device synchronization with free theoretical resources | Runtime or stream dependency |
| Allocation stalls near memory capacity | Tensor-lifetime or allocator pressure |
| Long tail after final backward | DP reduction or optimizer synchronization |
| High variance in one stage | MoE routing or hardware stragglers |

## Step 5: apply the smallest relevant transformation

| Dominant cause | Candidate transformation |
| --- | --- |
| Dependency bubble | More microbatches, interleaving, splitting, V placement |
| Stage imbalance | Move layer boundaries, split/replicate heavy stage, apply TP locally |
| Exposed communication | Prepost receive, async P2P, topology-aware placement |
| Too many small messages | Coalesce chunks or increase bucket size |
| Long activation lifetime | 1F1B, deferred-state redesign, V placement, recompute |
| Harmful overlap | Serialize selected operations or partition resources |
| DP/EP interference | Jointly schedule collectives, change bucket granularity |
| Optimizer tail | Earlier gradient reductions, optimizer pipelining, altered synchronization |

## Step 6: re-evaluate the full Pareto frontier

Do not optimize only iteration time. Track:

A memory-saving schedule may win by enabling a larger microbatch even if its schedule has more bubbles. The controllable-memory pipeline experiments show exactly this effect: a lower-memory V schedule can outperform a nominally zero-bubble schedule because the saved memory permits better arithmetic intensity. [^18]

---

## 15\. Automatic discovery: search transformations, not source code

The search engine should operate over a small DSL or transformation grammar.

A useful grammar is:

```
SplitModel(cut_points)
Place(stage, device_or_mesh)
Replicate(stage, devices)
Shard(stage_or_op, devices, axis)
SplitBatch(num_microbatches)
SplitBackward(stage)
Order(A, B)
CoSchedule(A, B, streams, resource_partition)
Recompute(tensor)
Offload(tensor)
Bucket(comm_group, size)
SelectCollectiveAlgorithm(op, algorithm)
```

This resembles Piper’s approach: annotations identify schedulable regions, while directives such as placement, replication, sharding, splitting, and ordering transform the global training DAG. [^17]

The search result should be a transformed IR, not generated Python scheduling loops. The same IR should drive:

- Correctness validation.
- Performance simulation.
- Runtime execution.
- Chrome-trace visualization.

That prevents simulator/runtime semantic drift.

---

## 16\. Factor the search hierarchically

The full joint space is too large to search flatly.

A strong hierarchy is:

## Level A: parallelism degrees

Choose candidate combinations:

Subject to:

with adjustments for dimensions that share or nest meshes.

Prune using:

- Parameter-memory lower bounds.
- Activation-memory lower bounds.
- Minimum collective communication.
- Divisibility constraints.
- Global batch constraints.

## Level B: model partitioning

For repeated linear transformers, contiguous partitioning can be solved efficiently by dynamic programming.

If candidate cut points are , define:

Then optimize the maximum stage cost or simulated iteration cost.

Alpa uses a hierarchical formulation: dynamic programming for stage/mesh assignment and ILP for intra-operator plans within a mesh. This is a useful pattern because it converts one intractable joint search into interacting but manageable subproblems. [^20]

For non-linear models, the partition problem becomes graph partitioning. GraphPipe generalizes pipeline stages from a linear sequence to a DAG of stages and jointly optimizes graph partitioning and microbatch scheduling. [^21]

## Level C: placement

Enumerate or optimize:

- Linear placement.
- Cyclic/interleaved placement.
- V placement.
- Replicated reverse pipelines.
- Topology-aware rank mapping.

Prune placements whose communication lower bound already exceeds the current best candidate.

## Level D: schedule synthesis

Given fixed tasks and placement, find an execution order.

## Level E: low-level execution

Tune:

- Streams.
- Communication buckets.
- Collective ordering.
- Resource partitions.
- Recompute timing.
- Unshard/reshard timing.

---

## 17\. Exact schedule synthesis

For small instances, formulate scheduling as ILP or CP-SAT.

For each task , define start time .

For every dependency :

For two tasks that cannot overlap on the same exclusive resource, introduce binary :

For memory, use a cumulative-resource constraint over tensor lifetimes:

The objective is:

This works well for:

- Small stage counts.
- Small microbatch counts.
- Search over one periodic schedule block.
- Producing ground-truth optima for evaluating heuristics.

It does not scale well to hundreds of stages, hundreds of microbatches, dynamic MoE routing, and complex interference models.

---

## 18\. Heuristic schedule synthesis

For larger cases, use a memory-aware list scheduler.

At each scheduling decision:

1. Find dependency-ready tasks.
2. Remove tasks that violate memory or resource constraints.
3. Score remaining tasks.
4. Launch the best task or compatible task set.

A useful priority score is:

Typical rules emerge naturally:

- Prioritize because it unlocks upstream stages.
- Run when it fills otherwise idle time.
- Run when memory pressure is high.
- Avoid launching a forward that delays the next critical .
- Prepost receives before their consumers become ready.
- Prioritize communication on critical paths over off-critical-path reductions.

This is close to the intuition behind the Zero Bubble heuristic, which schedules forwards within the memory budget, alternates forward/input-backward work, and inserts weight-gradient work into bubbles or when memory must be reclaimed. [^16]

---

## 19\. Search periodic building blocks rather than entire iterations

Transformer training repeats nearly identical work across microbatches. Exploit this symmetry.

Instead of searching start times for all microbatches, search:

- One microbatch building block.
- Relative offsets among actions.
- A repeating interval .
- Warm-up and cooldown transformations.

A candidate building block is valid if repeated copies:

- Respect dependencies.
- Do not collide on exclusive resources.
- Stay under the memory limit.
- Preserve optimizer semantics.

This can reduce a huge scheduling space into a small offset-search space.

For example, search over:

The controllable-memory work explicitly uses repeating building blocks, squeezing, and optional reordering to derive schedule families and reason analytically about memory. [^18]

---

## 20\. Multi-fidelity automated search

A practical optimizer should be a funnel.

```
millions of symbolic candidates
        ↓
thousands passing memory/lower-bound pruning
        ↓
hundreds evaluated by discrete-event simulation
        ↓
tens evaluated by contention-aware simulation
        ↓
several tested on a small cluster
        ↓
one or two production candidates
```

## Stage 1: analytical pruning

Reject candidates using:

- Memory lower bounds.
- Critical-path bounds.
- Communication-volume bounds.
- Divisibility constraints.
- Invalid semantic transformations.

## Stage 2: cheap simulation

Use deterministic profiled durations with simple resource exclusion.

## Stage 3: detailed simulation

Use:

- Concurrent communication models.
- Pairwise interference.
- Real allocator behavior.
- Host overhead.
- Stochastic task durations.

## Stage 4: active hardware measurement

Choose candidates whose:

- Predicted performance is high.
- Uncertainty is high.
- Design differs meaningfully from existing measurements.

Use the new measurements to retrain or recalibrate the cost model.

## Stage 5: robust selection

Optimize not only expected time:

but perhaps:

especially for MoE routing variance or heterogeneous clusters.

---

## 21\. What automated method should be used where?

| Subproblem | Recommended method |
| --- | --- |
| Parallelism-degree enumeration | Enumerate factorizations plus pruning |
| Contiguous stage partitioning | Dynamic programming |
| Device-mesh and sharding choice | DP/ILP or constrained enumeration |
| Small static schedule | ILP or CP-SAT |
| Large static schedule | Critical-path list scheduling or beam search |
| Periodic transformer schedule | Building-block/offset search |
| Bucket and stream tuning | Bayesian optimization or evolutionary search |
| Placement on topology | Graph partitioning plus local search |
| Dynamic MoE scheduling | Stochastic simulation plus adaptive policy |
| Online runtime decisions | Lightweight ready-queue heuristic |
| Novel high-dimensional co-design | Hierarchical evolutionary or Bayesian search |

Reinforcement learning is usually not the first choice. It becomes more compelling when:

- The state is dynamic.
- Runtime decisions must adapt online.
- A good simulator provides cheap episodes.
- Static optimization cannot describe the policy.

For ordinary synchronous transformer schedules, DP, ILP, list scheduling, beam search, and evolutionary search are typically easier to debug and much more sample-efficient.

---

## 22\. The system architecture to build

```
Model + optimizer + semantic contract
                  │
                  ▼
         Graph capture / annotation
                  │
                  ▼
       Canonical global training IR
       compute + comm + memory + versions
                  │
         ┌────────┴────────┐
         ▼                 ▼
 Strategy transforms    Profile database
         │                 │
         └────────┬────────┘
                  ▼
       Correctness and safety validator
                  │
                  ▼
      Discrete-event performance simulator
                  │
                  ▼
       Multi-objective search optimizer
                  │
                  ▼
        Per-rank executable schedule
                  │
                  ▼
      Strategy-agnostic distributed runtime
                  │
                  ▼
        Hardware trace and calibration
                  └─────────────── feedback
```

The most important architectural property is:

> The simulator and real runtime must consume the same transformed IR.

PyTorch is moving in this direction with a schedule IR that separates schedule authoring from communication insertion and execution. Piper generalizes the idea to a global distributed training DAG that includes compute, communication, placement, streams, and memory. [^6] [^17]

---

## 23\. A concrete path from Picotron to a research platform

## Phase 1: correctness foundation

Before performance search:

- Correct gradient scaling.
- Compare AFAB and 1F1B with a non-pipeline reference.
- Compare gradients and parameters, not only loss.
- Test , uneven layer counts, DP × PP, and CP × PP.
- Track microbatch IDs explicitly.
- Verify matching send/receive operations.
- Verify optimizer and RNG semantics.

## Phase 2: replace loops with an action IR

Instead of implementing each schedule as a separate Python loop, generate:

```
Action(stage=0, mb=0, kind=F)
Action(stage=0, mb=0, kind=SEND_F)
Action(stage=1, mb=0, kind=RECV_F)
...
```

Initially support only:

```
F, FULL_B, SEND_F, RECV_F, SEND_B, RECV_B
```

Generate AFAB and 1F1B from the same IR.

## Phase 3: build the correctness simulator

Validate:

- Dependencies.
- Per-rank action order.
- Matching P2P actions.
- Collective order.
- No cyclic waits.
- Complete gradient production.
- Correct parameter-version use.

## Phase 4: add the timing and memory simulator

Profile Picotron’s current stages and reproduce:

- AFAB step time.
- 1F1B step time.
- Queue depths.
- Peak activation memory.
- Communication exposure.

## Phase 5: make the real runtime asynchronous

Picotron currently waits immediately and calls `torch.cuda.synchronize()` after P2P. Remove this architecture before implementing overlap-heavy schedules.

Introduce:

- Communication handles.
- Preposted receives.
- CUDA events.
- Dedicated PP streams.
- Wait-on-consumption rather than wait-on-launch.
- Per-edge communication buffers.

## Phase 6: support multiple logical stages per rank

Replace rank-level metadata with stage-level metadata:

```python
LogicalStage(
    stage_id,
    physical_rank,
    predecessor_stage,
    successor_stage,
    module,
)
```

This unlocks:

- Interleaved 1F1B.
- Looped BFS.
- V placement.
- DualPipeV placement.

## Phase 7: split backward

Add:

```
BACKWARD_INPUT
BACKWARD_WEIGHT
```

Keep saved state indexed by:

```
(stage_id, microbatch_id)
```

rather than FIFO lists.

PyTorch’s pipeline work demonstrates both manual and automatic partitioning of the autograd backward graph into input-gradient and weight-gradient computations. [^6]

## Phase 8: implement heuristic schedule search

Start with:

- Fixed partition and placement.
- Measured .
- Fixed .
- Memory-aware list scheduling.

Compare the search result against:

- 1F1B.
- ZB1P.
- Handcrafted V schedules.
- Small-instance ILP optima.

## Phase 9: jointly search partition, placement, and schedule

Only after the simulator has demonstrated acceptable ranking accuracy should you expand into:

- Stage boundaries.
- Virtual-stage count.
- V versus cyclic placement.
- Microbatch shape.
- TP/PP/DP/EP degrees.
- Communication buckets and stream mappings.

---

## 24\. The deepest design principle

There are three distinct layers of invention.

## Layer 1: expose more legal concurrency

Examples:

- Split full backward into .
- Split a stage into smaller chunks.
- Remove an unnecessary barrier.
- Expose communication as explicit tasks.
- Separate optimizer reductions from updates.

This changes the DAG and increases scheduling freedom.

## Layer 2: place work to improve complementary behavior

Examples:

- V-shaped placement balances activation lifetimes.
- Bidirectional placement provides work from two streams.
- Topology-aware placement keeps high-volume collectives on fast links.
- Heavy stages receive more devices or replication.

This changes where tasks and data live.

## Layer 3: schedule exposed work onto physical resources

Examples:

- Run before .
- Prepost receives.
- Overlap all-to-all with compute.
- Delay an all-reduce to avoid interfering with critical EP communication.
- Run to free memory.

This determines how close execution comes to the lower bound.

A good algorithm usually has one strong idea in each relevant layer. DualPipe, for example, is not merely an unusual ordering: it combines placement, action decomposition, and low-level resource scheduling.

---

## Final framework

Think of pipeline-parallel system development as the following optimization problem:

The most effective navigation strategy is:

1. **Represent everything in a unified task DAG.**
2. **Calculate lower bounds.**
3. **Build a calibrated discrete-event simulator.**
4. **Use traces to classify the dominant gap to the bound.**
5. **Apply graph, placement, or scheduling transformations targeted at that gap.**
6. **Search hierarchically rather than jointly.**
7. **Use exact optimization for small subproblems and heuristics for large ones.**
8. **Maintain a multi-fidelity loop from formulas to simulation to hardware.**
9. **Preserve one IR across simulation and real execution.**
10. **Select from a Pareto frontier rather than searching for a universally best schedule.**

That architecture turns pipeline parallelism from a collection of hand-drawn diagrams into a reproducible compiler and operations-research problem.

[^1]: [ar5iv.labs.arxiv.org](https://ar5iv.labs.arxiv.org/html/1811.06965)

[^2]: [proceedings.mlr.press](https://proceedings.mlr.press/v139/narayanan21a/narayanan21a.pdf)

[^3]: [docs.pytorch.org](https://docs.pytorch.org/docs/2.13/distributed.pipelining.html)

[^4]: [arxiv.org](https://arxiv.org/pdf/2211.05953)

[^5]: [arxiv.org](https://arxiv.org/html/2401.10241v1)

[^6]: [discuss.pytorch.org](https://discuss.pytorch.org/t/distributed-w-torchtitan-training-with-zero-bubble-pipeline-parallelism/214420)

[^7]: [arxiv.org](https://arxiv.org/html/2405.15362v1?utm_source=chatgpt.com)

[^8]: [arxiv.org](https://arxiv.org/html/2107.06925v5)

[^9]: [arxiv.org](https://arxiv.org/html/2412.19437)

[^10]: [github.com](https://github.com/deepseek-ai/DualPipe)

[^11]: [huggingface.co](https://huggingface.co/blog/NormalUhr/deepseek-dualpipe)

[^12]: [sail.sea.com](https://sail.sea.com/blog/articles/63?utm_source=chatgpt.com)

[^13]: [github.com](https://github.com/pytorch/pytorch/blob/main/torch/distributed/pipelining/schedules.py)

[^14]: [github.com](https://github.com/pytorch/pytorch/issues/164756?utm_source=chatgpt.com)

[^15]: [arxiv.org](https://arxiv.org/abs/1806.03377)

[^16]: [arxiv.org](https://arxiv.org/abs/2401.10241)

[^17]: [arxiv.org](https://arxiv.org/html/2606.11169v1)

[^18]: [arxiv.org](https://arxiv.org/abs/2405.15362)

[^19]: [usenix.org](https://www.usenix.org/conference/nsdi26/presentation/qin)

[^20]: [ar5iv.labs.arxiv.org](https://ar5iv.labs.arxiv.org/html/2201.12023)

[^21]: [arxiv.org](https://arxiv.org/abs/2406.17145)