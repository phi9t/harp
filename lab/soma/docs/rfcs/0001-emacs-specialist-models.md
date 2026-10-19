# RFC 0001: Emacs Specialist Models and Shared State Layer

Status: Draft
Created: 2026-09-27
Quest: Build an Emacs-native operator and completion system for the Doom-in-Ghostty/SSH/tmux workflow

## Summary

This project should start with two separate specialist models sharing one Emacs integration layer.

The important distinction is between knowing about Emacs, reliably operating a live Emacs session, and predicting the next useful edit. Those need different training examples and success criteria.

For the target Doom-in-Ghostty/SSH/tmux workflow:

| Specialist | Job | What good means |
| --- | --- | --- |
| Emacs operator | Navigate and manipulate buffers, LSP, Magit, Org, Jupyter, Dired, and processes | Accomplishes the requested state change without damaging unrelated state |
| Completion/edit model | FIM, short inline completions, eventually next-edit prediction | Produces useful, correctly scoped suggestions quickly enough to accept while typing |

Initial experiments should use a 4B-class operator with a 9B-class capacity comparison, and a 1.5B completion model with a 4B-7B comparison. These are starting hypotheses, not claims that these sizes will meet the quality bar.

The main investment should be an executable Emacs training environment and real editing data, not a large collection of Emacs question-answer pairs.

## Proposed Model Shortlist

| Role | Starting checkpoint | Why include it |
| --- | --- | --- |
| Operator, first experiment | Qwen3.5-4B | Official deployment instructions cover tool calling and a non-thinking mode. Start with text-only observations and short action outputs. |
| Operator, capacity comparison | Qwen3.5-9B | Establish whether failures persist because of the harness/data or improve substantially with more model capacity. |
| Completion, latency baseline | Qwen2.5-Coder-1.5B base | A small code-specialized base checkpoint suitable for a dedicated FIM path. |
| Completion, quality comparison | Qwen2.5-Coder-7B base | A larger checkpoint from the same family gives a useful controlled comparison. |
| Completion, alternative family | Mellum-4b-base | Oriented toward code completion, with documented FIM and additional-file context formatting. |

The licensing detail to check early: Qwen2.5-Coder-3B is tagged `qwen-research`, while the 1.5B and 7B checkpoints are tagged Apache-2.0. Do not automatically choose 3B as the obvious midpoint without reviewing that difference.

Mellum2 is another interesting operator/coding candidate, but it is 12B total parameters with 2.5B active per token. The active count does not make its resident-weight footprint equivalent to a 2.5B dense model. Treat it as a different deployment tradeoff, not the default small model.

Do not initially combine both jobs into one checkpoint. A shared base with two adapters is a reasonable later experiment, but separate models make it easier to isolate training regressions, output formats, context budgets, and serving latency.

## Operator Objective

The operator should learn a policy over Emacs state:

```text
a_t ~ pi_theta(
  a_t |
  user goal,
  current observation,
  relevant history,
  available capabilities
)
```

It should learn what to inspect, which operation to perform, when to wait, how to recover, and when it is actually finished.

Screenshots and raw key sequences should not be the primary interface. The environment should expose explicit editor and process state.

## Structured Observation

The observation should be a compact state summary with expandable details.

| State category | Information the operator should receive |
| --- | --- |
| Editor identity | Emacs instance, project, working directory, configuration/package manifest |
| Buffers | Stable IDs, names, paths, modes, modified status, revision counters |
| Interaction state | Selected window, point, region, narrowing, Evil state, active minibuffer or transient |
| Code intelligence | Relevant diagnostics, symbol information, LSP workspace status |
| Git | Repository identity, branch, staged/unstaged summaries, selected Magit sections |
| Org | Relevant heading IDs, hierarchy, properties, TODO states, source-block/session associations |
| Jupyter | Client-to-buffer association, kernel identity and incarnation, execution state, outstanding requests |
| Processes | Process IDs/handles, associated buffers, working directories, recent output, termination status |

The payload should be demand-driven. Switching buffers should not require receiving an entire Git diff and every kernel's variables.

For revision tracking, Emacs provides `buffer-chars-modified-tick`, which changes when buffer text changes but not merely when text properties change. Stale-edit checks should use that together with buffer identity.

## Operation Surface

Expose a small typed operation surface. These are proposed adapter operations, not existing package function names:

```text
editor.observe
editor.describe_command

buffer.select
buffer.read
buffer.apply_edit

lsp.find_references
lsp.preview_rename
lsp.apply_workspace_edit

magit.inspect_sections
magit.stage_selection

org.inspect_heading
org.refile_heading

jupyter.attach_existing
jupyter.associate_buffer
jupyter.execute_region
jupyter.await_request

process.send_input
process.read_output
process.interrupt
```

The adapter should invoke actual Emacs/package functionality. For example, a Magit task should not secretly become "run an unrelated shell script that makes the Git index look right."

At the same time, the model should not be forced to rediscover every task through `SPC ...` key sequences. The preferred hierarchy is:

1. Typed semantic operations for common tasks.
2. Named interactive commands for the long tail.
3. Explicitly authorized Elisp for extensions and debugging.

Command metadata and documentation should come from the installed environment. Package-specific knowledge belongs partly in retrieval, not entirely in weights.

Existing projects such as `mcp-server-lib.el` and `rhblind/emacs-mcp-server` provide useful bridge pieces, including diagnostics, Elisp evaluation, and structured Org operations. They should be treated as starting points, not as the complete training environment.

## Reliability Model

Example target request:

> Rename this symbol, but preserve the unsaved changes I made in another buffer.

The adapter should require expected buffer revisions before applying edits. On mismatch, it should return a conflict for the operator to inspect, not silently overwrite newer text.

Buffer changes can use Emacs change groups for rollback. `atomic-change-group` covers its current buffer; multi-buffer transactions need lower-level change-group machinery. Neither rolls back a Git push, shell command, or Jupyter execution.

Separate capabilities into:

1. Read-only inspection.
2. Reversible editor edits.
3. Externally consequential actions.

Kernel restarts, destructive file operations, Git pushes, and arbitrary code execution need explicit capability policies. An Elisp function denylist is not a sufficient security boundary for arbitrary evaluation.

## Emacs Gym

The environment should run real Doom sessions with pinned packages, disposable repositories, test Org files, actual language servers, and disposable kernels/processes.

For interactive behavior, test a normal Emacs event loop with a terminal client where needed. Do not rely exclusively on batch-mode unit tests. The deployment target includes prompts, transients, asynchronous process output, and a human who may continue typing.

| Domain | Example training task | Executable checks |
| --- | --- | --- |
| Buffer/window management | Show the implementation and its tests side by side without losing the current edit | Correct buffers/windows; unsaved contents preserved |
| LSP | Rename a symbol across the project, excluding unrelated same-spelled identifiers | Intended references changed; project checks pass; unrelated files unchanged |
| Magit | Stage only the bug-fix hunk, leaving debug edits unstaged | Exact index contents; unrelated staged/unstaged changes preserved |
| Org | Refile a subtree while preserving its ID, properties, and children | Correct hierarchy and metadata; no duplicate subtree |
| Jupyter | Attach this Python buffer to an existing kernel and evaluate only the selected region | Correct association, kernel incarnation unchanged, expected request/result |
| Dired | Rename a selected set of files with collision handling | Exact filesystem changes; no unintended deletion or overwrite |
| Comint | Send input to the intended live process and collect its result | Correct process and working directory; input submitted once |
| Recovery | A buffer changed, a language server disconnected, or a kernel became busy | Appropriate re-observation/recovery without prohibited side effects |

Negative constraints are as important as the requested result.

"Staged the intended hunk" is insufficient when the model also staged everything else. "Produced the expected Python value" is insufficient when it destroyed the original kernel and recomputed from scratch.

## Jupyter Requirements

Jupyter should be a first-class benchmark, not an afterthought.

`emacs-jupyter` distinguishes launching a new kernel, connecting to an existing one, and associating a source buffer with a REPL client. In particular, `jupyter-repl-associate-buffer` sets the buffer's `jupyter-current-client` and enables interaction with that client's kernel.

Useful task:

> This buffer belongs to the already-running experiment kernel. Inspect `batch`, run the selected diagnostic, and show the result. Do not restart the kernel or execute the training cell.

Verify the kernel's incarnation as well as its nominal identity. Jupyter message headers distinguish sessions, and a restarted kernel should use a new kernel session ID. Replies and outputs are associated with requests through `parent_header`. These are better signals than "the REPL buffer still has the same name."

The adapter should track an execution request through completion. A timeout after submission means outcome unknown, not necessarily "nothing ran." Retrying a non-idempotent cell blindly is a serious failure.

## Operator Post-Training Sequence

Use the sequence:

```text
tool-enabled baseline
-> verified demonstration SFT
-> student-failure correction
-> selective environment RL
```

Dataset sizes and hyperparameters below are initial experimental budgets, not measured requirements.

### Stage A: Harness Baseline

Before training, evaluate the same model under:

| Condition | Purpose |
| --- | --- |
| Generic tool bridge | Establish the naive baseline |
| Structured state + typed operations + documentation retrieval | Measure the harness contribution |
| Same harness + operator SFT | Measure the specialization contribution |
| Same harness + SFT + environment RL | Measure the additional RL contribution |

This comparison matters. Otherwise, it is easy to attribute an improvement from better tools or clearer observations to post-training.

First create approximately 80-150 hand-authored workflow templates, then vary repository contents, buffer names, selected windows, dirty state, kernel state, and interruptions.

The test set must include held-out workflow compositions and projects, not just new random seeds from every training template.

### Stage B: Executed Demonstrations

A useful trajectory consists of:

```text
goal and constraints
initial observation
assistant action
actual tool result
updated observation
...
final state
independent verification result
```

Combine a modest amount of recorded human operation with a larger collection of teacher-generated trajectories.

The teacher should operate the same interface the student will use. It should not receive hidden verifier data or capabilities that disappear at deployment.

An initial target of 5,000-10,000 verified trajectories is reasonable for testing whether specialization works. Expand based on uncovered failure classes rather than aiming immediately for millions of examples.

Teacher-generated Elisp must be executed. Plausible command names and fabricated success messages are not training evidence.

### Stage C: Supervised Fine-Tuning

For the first experiment, try LoRA ranks 32 and 64, a small learning-rate sweep around `5e-5` to `1e-4`, and roughly 4K-8K tokens per decision context, with early stopping on held-out environment success.

The supervised targets should be the operator's actions and concise user-facing reports. Observations and tool outputs should condition the model, not become text it is trained to invent.

TRL supports assistant-only loss and completion-only loss, but assistant masking depends on the chat template. Inspect token-level masks on actual multi-turn tool examples before trusting the configuration flag.

Retain a small general-code/tool-use replay set and evaluate regressions. The desired result is specialization, not an operator that can only emit a narrow JSON dialect.

Do not start with large-scale continued pretraining on every Emacs manual and package repository. Add that only when evaluations show a knowledge deficit that retrieval and trajectory training are not fixing.

### Stage D: Student-Reached States

After initial SFT, let the student attempt tasks. Collect failures such as selecting a similarly named buffer, confusing staged with unstaged changes, submitting to a busy kernel, or applying an edit against an obsolete revision.

Then have a stronger teacher or a human demonstrate recovery from that actual state.

Do not train incorrect actions as positive targets. Use corrected decisions and successful recovery continuations.

An Emacs trace explorer would be valuable here: it could let a reviewer mark the exact decision where the operator went wrong and attach the correction, rather than reviewing entire transcripts.

### Stage E: Environment RL

Begin RL only on short, resettable, objectively checkable workflows.

A safe reward ordering is:

```text
policy/invariant violations first
-> task completion second
-> efficiency only among successful executions
```

Do not allow fewer actions to compensate for losing unsaved work. Do not reward completion merely because the assistant says "done."

GRPO-style groups can sample several trajectories from equivalent initial states, but all rollouts need genuinely reset environments. Reconstructing a kernel from a fixture is usually more defensible than pretending an arbitrary live kernel can be perfectly snapshotted.

For implementation, `verl` exposes an agent loop with interleaved model/tool tokens and an explicit response mask that excludes tool outputs from model-generated-token positions. Validate the chosen checkpoint's learner, rollout engine, serialization, and log-probability agreement end to end before relying on it.

## Completion Model Objective

Train the coding assistant as an editing model, not a chat model.

For completion, the central objective is:

```text
L_FIM = -log p_theta(middle | prefix, suffix, relevant context)
```

For a Qwen-style raw FIM interface, the serialized input is conceptually:

```text
<|fim_prefix|>
code before the cursor
<|fim_suffix|>
code after the cursor
<|fim_middle|>
```

The target is only the insertion, followed by appropriate stopping behavior. The exact tokens and framing must match the checkpoint. Mellum, for example, documents a different FIM format.

## Completion Capabilities

### Cursor Completion

This is the first target: finish an expression, argument list, statement, or short block without repeating the suffix or rewriting unrelated code.

Build data from syntax-aware holes and realistic cursor positions. Include incomplete identifiers, partially typed calls, indentation boundaries, comments, strings, and syntactically broken intermediate states.

Prioritize Python, Rust, C++/CUDA, Elisp, shell, and Bazel/Starlark, weighted by actual editing frequency rather than equally.

An initial budget could be 100,000-300,000 FIM windows, with language and hole-type balancing. Some should come from code with permission to train on; some can be independently generated and execution-checked.

### Repository-Aware Completion

The model should receive compact, relevant context: imported definitions, nearby types, related tests, LSP signatures, and snippets from other buffers.

Retrieval must reconstruct what was available at the time of the edit. Training on a historical completion while retrieving the finished implementation from a later commit leaks the answer.

Use repository-level and time-based holdouts, not merely random file chunks.

### Next-Edit Prediction

This is separate from FIM:

> After I change this function signature, suggest the corresponding update at the call site.

Its inputs include recent edit history and diagnostics, and its output may be a replacement elsewhere, not an insertion at the cursor.

Add this after inline completion is useful. It can eventually share a model through explicit task conditioning, but should have separate datasets, metrics, and UI behavior.

## Editing Data

Personal editing data is more valuable than final source alone.

The most useful record is approximately:

```text
buffer before suggestion
cursor and relevant context
suggestion shown
how much was accepted
subsequent edits
what remained after continued work
latency and cancellation information
```

Acceptance alone is not correctness. A suggestion might be accepted and immediately rewritten; a good suggestion might be ignored because the user was already typing.

Optimize a combination of retained contribution, low correction effort, latency, and non-interference. A separate trigger/ranking mechanism can learn when not to show a suggestion.

Start with FIM SFT and carefully constructed preference pairs before online RL. RLinf has a relevant Qwen2.5-Coder-1.5B completion experiment, but its published demonstration used an LLM judge proxy rather than a large real-user acceptance study. Treat it as useful implementation evidence, not proof of user productivity gains.

## Tool Completion Integration

Keep LSP and kernel completion in the system.

Do not ask the small generative model to replace exact information already available from tools. Use LSP or kernel completion to supply known symbols and signatures; use the generative model to construct the expression or block that uses them.

`emacs-jupyter` already integrates with `completion-at-point`, including associated source buffers and Jupyter source blocks in Org. That is a useful existing channel.

For runtime-aware generation, expose only bounded, relevant metadata such as selected tensor names, shapes, dtypes, and devices. Avoid blindly dumping values or invoking arbitrary object representations.

On the UI side, Minuet is a relevant starting point because it documents FIM-compatible endpoints and local `llama.cpp` configurations, and its current interface includes next-edit prediction. Tabbymacs provides another route through Tabby, including ghost text and incremental synchronization. Evaluate one first rather than introduce competing completion overlays.

In Doom/Evil, acceptance behavior needs deliberate ownership. TAB should not unpredictably alternate among indentation, Company/Corfu candidates, snippets, and model-generated ghost text.

## Serving Architecture

For the remote-session setup, local inference should mean close to the Emacs process, not automatically on the laptop.

When Emacs runs on the remote machine inside tmux, a nearby GPU endpoint may avoid sending editing context back across the SSH connection. A laptop model may still be attractive, but the complete latency path has to be measured.

For completion, the useful quantity is:

```text
T_suggestion =
  T_debounce
  + T_network
  + T_queue
  + T_prefill
  + T_generation
  + T_render
```

Set aspirational targets around 200 ms median and 500 ms p95 for the first useful suggestion, then adjust scope based on actual hardware and network measurements. These are product targets, not expected benchmark results.

Initial serving design:

1. Short completion outputs.
2. Bounded context.
3. Cancellation of obsolete requests.
4. Revision checks before display or acceptance.
5. Separate queues for completion and operator generations.

For the operator, default to short non-thinking actions for routine steps and separately evaluate whether extra reasoning helps difficult recovery tasks. Qwen3.5 documents disabling its default thinking behavior through serving template configuration.

On training compute, profile before estimating GPU-hours. Weight storage alone is approximately `2P` bytes in BF16 or `P/2` bytes at an idealized four bits per parameter; activations, optimizer state, quantization metadata, serving caches, and auxiliary modules are additional.

Budget engineering effort for resetting and supervising many Emacs/LSP/kernel environments, not just fitting the learner. This is a different workload from ordinary static-dataset fine-tuning.

## First Milestone

Keep the first milestone deliberately concrete.

### Operator Track

Build one reliable state/action adapter covering:

1. Buffers.
2. LSP inspection.
3. Selective Magit staging.
4. Jupyter attachment/execution.

Establish a locked evaluation suite, generate several thousand verified demonstrations, and compare Qwen3.5-4B before and after SFT under the identical harness. Use 9B to test whether remaining failures are capacity-sensitive.

### Completion Track

Connect an unmodified Qwen2.5-Coder-1.5B to Doom first, measure actual latency and suggestion behavior, then fine-tune it on realistic FIM windows.

Compare it with the unmodified model and a larger completion checkpoint. Add repository context as a separate ablation.

### Combined Track

Verify that the operator and completion system can coexist with live edits. Neither should apply stale changes, steal focus unnecessarily, submit a cell twice, or confuse buffers that share a filename.

## Scorecards

Operator scorecard:

1. Task success.
2. Preservation of unrelated state.
3. Recovery success.
4. Unauthorized-action attempts.
5. Time/actions per successful task.

Completion scorecard:

1. Latency.
2. Retained suggestions.
3. Correction effort.
4. Suffix duplication.
5. Invalid-symbol generation.
6. Functional checks where available.

Zero observed destructive failures in a small test set does not establish safety. Those cases need explicit adversarial and concurrency testing.

## Recommendation

Use Qwen3.5-4B as the first operator experiment, Qwen2.5-Coder-1.5B as the first completion experiment, and one shared Emacs state/trace layer underneath both.

The most valuable deliverable is not initially either checkpoint. It is a reproducible collection of workflows, executable checks, and realistic editing traces, because that is what reveals whether post-training has made the system genuinely better at working with the user.

## Full Reference Document

This section preserves the source reference draft in full. The RFC sections above normalize it into a quest-facing decision document; this section is the unabridged reference text to use when deriving specs, tickets, source checks, and implementation experiments.

**Yes—this is a promising specialization project, and I would start with two separate models sharing one Emacs integration layer.**

The important distinction is between **knowing about Emacs**, **reliably operating a live Emacs session**, and **predicting the next useful edit**. Those need different training examples and success criteria.

For your Doom-in-Ghostty/SSH/tmux workflow, I would aim for:

| Specialist | Its job | What “good” means |
|---|---|---|
| **Emacs operator** | Navigate and manipulate buffers, LSP, Magit, Org, Jupyter, Dired, and processes | Accomplishes the requested state change without damaging unrelated state |
| **Completion/edit model** | FIM, short inline completions, eventually next-edit prediction | Produces useful, correctly scoped suggestions quickly enough to accept while typing |

My initial experiments would use a **4B-class operator, with a 9B-class capacity comparison**, and a **1.5B completion model, with a 4B–7B comparison**. Those are starting hypotheses—not a claim that these sizes will meet your quality bar.

**The main investment should be an executable Emacs training environment and real editing data, not a large collection of Emacs question–answer pairs.**

### 1. Model choices I would actually test

Here is a concrete shortlist from the currently available checkpoints.

| Role | Starting checkpoint | Why I would include it |
|---|---|---|
| Operator, first experiment | **Qwen3.5-4B** | Its official deployment instructions cover tool calling and a non-thinking mode. I would initially use text-only observations and short action outputs. |
| Operator, capacity comparison | **Qwen3.5-9B** | Establish whether failures persist because of the harness/data or improve substantially with more model capacity. |
| Completion, latency baseline | **Qwen2.5-Coder-1.5B base** | A small code-specialized base checkpoint suitable for a dedicated FIM path. |
| Completion, quality comparison | **Qwen2.5-Coder-7B base** | A larger checkpoint from the same family provides a useful controlled comparison. |
| Completion, alternative family | **Mellum-4b-base** | Specifically oriented toward code completion, with documented FIM and additional-file context formatting. |

The Qwen operator capabilities and checkpoint availability are documented in the official model cards; the completion checkpoints and Mellum’s FIM interface are likewise available from their publishers. :chatgpt-content-reference{index="0"}

A licensing detail worth catching now: **Qwen2.5-Coder-3B is tagged `qwen-research`, whereas the 1.5B and 7B checkpoints are tagged Apache-2.0**. I would not automatically choose 3B as the obvious midpoint without reviewing that difference. :chatgpt-content-reference{index="1"}

Mellum2 is another interesting operator/coding candidate, but it is **12B total parameters with 2.5B active per token**. The active count does not make its resident-weight footprint equivalent to a 2.5B dense model. I would treat it as a different deployment trade-off, not the default “small model.” :chatgpt-content-reference{index="2"}

I would initially avoid combining both jobs into one checkpoint. A shared base with two adapters is a reasonable later experiment, but separate models make it easier to isolate training regressions, output formats, context budgets, and serving latency.

### 2. The operator should learn a policy over Emacs state

The operator’s task is approximately:

\[
a_t \sim \pi_\theta
\left(
a_t \mid
\text{user goal},
\text{current observation},
\text{relevant history},
\text{available capabilities}
\right)
\]

It should learn **what to inspect, which operation to perform, when to wait, how to recover, and when it is actually finished**.

I would not make screenshots or raw key sequences the primary interface. Your environment gives us an opportunity to expose explicit editor and process state.

#### A structured observation, not a dump of every buffer

I would design the observation around a compact state summary with expandable details:

| State category | Information the operator should receive |
|---|---|
| Editor identity | Emacs instance, project, working directory, configuration/package manifest |
| Buffers | Stable IDs, names, paths, modes, modified status, revision counters |
| Interaction state | Selected window, point, region, narrowing, Evil state, active minibuffer or transient |
| Code intelligence | Relevant diagnostics, symbol information, LSP workspace status |
| Git | Repository identity, branch, staged/unstaged summaries, selected Magit sections |
| Org | Relevant heading IDs, hierarchy, properties, TODO states, source-block/session associations |
| Jupyter | Client-to-buffer association, kernel identity and incarnation, execution state, outstanding requests |
| Processes | Process IDs/handles, associated buffers, working directories, recent output, termination status |

The exact payload should be demand-driven. Switching buffers should not require receiving an entire Git diff and every kernel’s variables.

For revision tracking, Emacs already provides `buffer-chars-modified-tick`, which changes when buffer text changes but not merely when text properties change. I would build stale-edit checks around that, together with buffer identity. :chatgpt-content-reference{index="3"}

#### Use semantic operations, but retain real Emacs behavior

I would expose a small, typed operation surface. These are **proposed adapter operations, not existing package function names**:

```text
editor.observe
editor.describe_command

buffer.select
buffer.read
buffer.apply_edit

lsp.find_references
lsp.preview_rename
lsp.apply_workspace_edit

magit.inspect_sections
magit.stage_selection

org.inspect_heading
org.refile_heading

jupyter.attach_existing
jupyter.associate_buffer
jupyter.execute_region
jupyter.await_request

process.send_input
process.read_output
process.interrupt
```

The adapter should invoke the actual relevant Emacs/package functionality. For example, a Magit task should not secretly become “run an unrelated shell script that makes the Git index look right.”

At the same time, I would not force the model to rediscover every task through `SPC …` key sequences. My preferred hierarchy is:

**Typed semantic operations for common tasks → named interactive commands for the long tail → explicitly authorized Elisp for extensions and debugging.**

Command metadata and documentation should come from the installed environment. Package-specific knowledge belongs partly in retrieval, rather than entirely in weights.

Existing projects already provide pieces of this bridge. `mcp-server-lib.el` provides infrastructure for exposing Emacs functionality, while `rhblind/emacs-mcp-server` exposes diagnostics, Elisp evaluation, and structured Org operations. These are useful starting points, but neither should be mistaken for the complete training environment described here. :chatgpt-content-reference{index="4"}

#### Reliability has to be enforced outside the model

Consider:

> “Rename this symbol, but preserve the unsaved changes I made in another buffer.”

The adapter should require expected buffer revisions before applying edits. On a mismatch, it should return a conflict for the operator to inspect—not silently overwrite newer text.

Buffer changes can use Emacs change groups for rollback. However, `atomic-change-group` covers its current buffer; multi-buffer transactions need the lower-level change-group machinery. Neither rolls back a Git push, shell command, or Jupyter execution. :chatgpt-content-reference{index="5"}

I would therefore separate:

**Read-only inspection**, **reversible editor edits**, and **externally consequential actions**.

Kernel restarts, destructive file operations, Git pushes, and arbitrary code execution need explicit capability policies. An Elisp function denylist is not a sufficient security boundary for arbitrary evaluation.

### 3. Build an “Emacs gym” around tasks you actually care about

The environment should run real Doom sessions with pinned packages, disposable repositories, test Org files, actual language servers, and disposable kernels/processes.

For interactive behavior, I would test a normal Emacs event loop with a terminal client where needed—not rely exclusively on batch-mode unit tests. Your deployment target includes prompts, transients, asynchronous process output, and a human who may continue typing.

The curriculum should look like this:

| Domain | Example training task | Executable checks |
|---|---|---|
| Buffer/window management | Show the implementation and its tests side by side without losing the current edit | Correct buffers/windows; unsaved contents preserved |
| LSP | Rename a symbol across the project, excluding unrelated same-spelled identifiers | Intended references changed; project checks pass; unrelated files unchanged |
| Magit | Stage only the bug-fix hunk, leaving debug edits unstaged | Exact index contents; unrelated staged/unstaged changes preserved |
| Org | Refile a subtree while preserving its ID, properties, and children | Correct hierarchy and metadata; no duplicate subtree |
| Jupyter | Attach this Python buffer to an existing kernel and evaluate only the selected region | Correct association, kernel incarnation unchanged, expected request/result |
| Dired | Rename a selected set of files with collision handling | Exact filesystem changes; no unintended deletion or overwrite |
| Comint | Send input to the intended live process and collect its result | Correct process and working directory; input submitted once |
| Recovery | A buffer changed, a language server disconnected, or a kernel became busy | Appropriate re-observation/recovery without prohibited side effects |

The **negative constraints** are as important as the requested result.

“Staged the intended hunk” is insufficient when the model also staged everything else. “Produced the expected Python value” is insufficient when it destroyed the original kernel and recomputed from scratch.

#### Jupyter deserves especially careful treatment

This is a major part of your workflow, so I would make it a first-class benchmark rather than an afterthought.

`emacs-jupyter` distinguishes launching a new kernel, connecting to an existing one, and associating a source buffer with a REPL client. In particular, `jupyter-repl-associate-buffer` sets the buffer’s `jupyter-current-client` and enables interaction with that client’s kernel. :chatgpt-content-reference{index="6"}

A useful task would be:

> “This buffer belongs to the already-running experiment kernel. Inspect `batch`, run the selected diagnostic, and show the result. Do not restart the kernel or execute the training cell.”

I would verify the kernel’s incarnation as well as its nominal identity. Jupyter message headers distinguish sessions, and a restarted kernel should use a new kernel session ID. Replies and outputs are associated with requests through `parent_header`. Those are better signals than “the REPL buffer still has the same name.” :chatgpt-content-reference{index="7"}

The adapter should track an execution request through completion. A timeout after submission means **outcome unknown**, not necessarily “nothing ran.” Retrying a non-idempotent cell blindly would be a serious failure.

### 4. Post-train the operator in stages

My proposed sequence is:

**Tool-enabled baseline → verified demonstration SFT → student-failure correction → selective environment RL.**

The dataset sizes and hyperparameters below are initial experimental budgets, not measured requirements.

#### Stage A: Establish how much the harness already solves

Before training, evaluate the same model under:

| Condition | Purpose |
|---|---|
| Generic tool bridge | Establish the naïve baseline |
| Structured state + typed operations + documentation retrieval | Measure the harness contribution |
| Same harness + operator SFT | Measure the specialization contribution |
| Same harness + SFT + environment RL | Measure the additional RL contribution |

This comparison matters. Otherwise, it is easy to attribute an improvement from better tools or clearer observations to post-training.

I would first create approximately **80–150 hand-authored workflow templates**, then vary repository contents, buffer names, selected windows, dirty state, kernel state, and interruptions.

The test set must include held-out workflow compositions and projects—not just new random seeds from every training template.

#### Stage B: Generate executed demonstrations

A useful trajectory consists of:

```text
goal and constraints
initial observation
assistant action
actual tool result
updated observation
...
final state
independent verification result
```

I would combine a modest amount of recorded human operation with a larger collection of teacher-generated trajectories.

The teacher should operate the **same interface the student will use**. It should not receive hidden verifier data or capabilities that disappear at deployment.

An initial target of **5,000–10,000 verified trajectories** seems reasonable for testing whether specialization works. I would expand based on uncovered failure classes rather than aiming immediately for millions of examples.

Crucially, teacher-generated Elisp must be executed. Plausible command names and fabricated success messages are not training evidence.

Recent work on interactive tool-agent post-training similarly combines synthesized tool-grounded dialogues, executable per-instance checkers, and subsequent verifier-based RL. That supports the overall recipe, although it does not establish performance for Doom Emacs. :chatgpt-content-reference{index="8"}

#### Stage C: Supervised fine-tuning

For the first experiment, I would try LoRA ranks **32 and 64**, a small learning-rate sweep around **\(5\times10^{-5}\) to \(10^{-4}\)**, and roughly **4K–8K tokens per decision context**, with early stopping on held-out environment success.

The supervised targets should be the operator’s actions and concise user-facing reports. Observations and tool outputs should condition the model, not become text it is trained to invent.

TRL supports assistant-only loss and completion-only loss, but assistant masking depends on the chat template. I would inspect token-level masks on actual multi-turn tool examples before trusting the configuration flag. :chatgpt-content-reference{index="9"}

I would also retain a small general-code/tool-use replay set and evaluate regressions. The desired result is specialization, not an operator that can only emit a narrow JSON dialect.

I would **not** start with large-scale continued pretraining on every Emacs manual and package repository. Add that only when evaluations show a knowledge deficit that retrieval and trajectory training are not fixing.

#### Stage D: Train on states the student actually reaches

This may be the highest-value phase.

After initial SFT, let the student attempt tasks. Collect failures such as selecting a similarly named buffer, confusing staged with unstaged changes, submitting to a busy kernel, or applying an edit against an obsolete revision.

Then have a stronger teacher or you demonstrate recovery **from that actual state**.

Do not train incorrect actions as positive targets. Use corrected decisions and successful recovery continuations.

This also makes your proposed Emacs trace explorer valuable: it could let you mark the exact decision where the operator went wrong and attach the correction, rather than reviewing entire transcripts.

#### Stage E: Environment RL only where the reward is credible

I would begin RL on short, resettable, objectively checkable workflows.

A safe reward ordering is:

**Policy/invariant violations first → task completion second → efficiency only among successful executions.**

Do not allow “fewer actions” to compensate for losing unsaved work. Do not reward completion merely because the assistant says “done.”

GRPO-style groups can sample several trajectories from equivalent initial states, but all rollouts need genuinely reset environments. Reconstructing a kernel from a fixture is usually a more defensible experimental reset than pretending that an arbitrary live kernel can be perfectly snapshotted.

For implementation, `verl` exposes an agent loop with interleaved model/tool tokens and an explicit response mask that excludes tool outputs from model-generated-token positions. It is a plausible foundation, but I would first validate the chosen checkpoint’s learner, rollout engine, serialization, and log-probability agreement end to end. :chatgpt-content-reference{index="10"}

### 5. Train the coding assistant as an editing model, not a chat model

For completion, the central objective is:

\[
\mathcal L_{\mathrm{FIM}}
=
-\log p_\theta
\left(
\text{middle}
\mid
\text{prefix},\text{suffix},\text{relevant context}
\right)
\]

For a Qwen-style raw FIM interface, the serialized input is conceptually:

```text
<|fim_prefix|>
code before the cursor
<|fim_suffix|>
code after the cursor
<|fim_middle|>
```

The target is **only the insertion**, followed by the appropriate stopping behavior. The exact tokens and framing must match the checkpoint: Mellum, for example, documents a different FIM format. :chatgpt-content-reference{index="11"}

I would distinguish three capabilities.

#### A. Cursor completion

This is the first target: finish an expression, argument list, statement, or short block without repeating the suffix or rewriting unrelated code.

Build data from syntax-aware holes and realistic cursor positions. Include incomplete identifiers, partially typed calls, indentation boundaries, comments, strings, and syntactically broken intermediate states.

For your work, I would prioritize Python, Rust, C++/CUDA, Elisp, shell, and Bazel/Starlark, weighted by actual editing frequency rather than equally.

An initial budget could be **100,000–300,000 FIM windows**, with language and hole-type balancing. Some should come from code you have permission to train on; some can be independently generated and execution-checked.

#### B. Repository-aware completion

The model should receive compact, relevant context: imported definitions, nearby types, related tests, LSP signatures, and snippets from other buffers.

But retrieval must reconstruct **what was available at the time of the edit**. Training on a historical completion while retrieving the finished implementation from a later commit would leak the answer.

I would use repository-level and time-based holdouts, not merely random file chunks.

#### C. Next-edit prediction

This is a separate task from FIM:

> “After I change this function signature, suggest the corresponding update at the call site.”

Its inputs include recent edit history and diagnostics, and its output may be a replacement elsewhere—not an insertion at the cursor.

I would add this after inline completion is useful. It can eventually share a model through explicit task conditioning, but should have separate datasets, metrics, and UI behavior.

#### Personal editing data is more valuable than final source alone

The most useful record is approximately:

```text
buffer before suggestion
cursor and relevant context
suggestion shown
how much was accepted
subsequent edits
what remained after continued work
latency and cancellation information
```

Acceptance alone is not correctness. A suggestion might be accepted and immediately rewritten; a good suggestion might be ignored because you were already typing.

I would optimize a combination of retained contribution, low correction effort, latency, and non-interference. A separate trigger/ranking mechanism can learn when not to show a suggestion.

I would start with FIM SFT and carefully constructed preference pairs before online RL. RLinf has a relevant Qwen2.5-Coder-1.5B completion experiment, but its published demonstration used an LLM judge as a proxy rather than a large real-user acceptance study. That makes it useful implementation evidence, not proof of user productivity gains. :chatgpt-content-reference{index="12"}

### 6. Keep LSP and kernel completion in the system

I would not ask the small generative model to replace exact information already available from your tools.

For example, use LSP or kernel completion to supply known symbols and signatures; use the generative model to construct the expression or block that uses them.

`emacs-jupyter` already integrates with `completion-at-point`, including associated source buffers and Jupyter source blocks in Org. That is a useful existing channel rather than something to discard. :chatgpt-content-reference{index="13"}

For runtime-aware generation, I would expose only bounded, relevant metadata—such as selected tensor names, shapes, dtypes, and devices. Avoid blindly dumping values or invoking arbitrary object representations.

On the UI side, **Minuet** is an especially relevant starting point: it documents FIM-compatible endpoints and local llama.cpp configurations, and its current interface also includes next-edit prediction. **Tabbymacs** provides another route through Tabby, including ghost text and incremental synchronization. I would evaluate one first rather than introduce competing completion overlays. :chatgpt-content-reference{index="14"}

In Doom/Evil, the acceptance behavior needs deliberate ownership. TAB should not unpredictably alternate among indentation, Company/Corfu candidates, snippets, and model-generated ghost text.

### 7. Serving architecture and compute

For your remote-session setup, **“local inference” should mean close to the Emacs process—not automatically on the laptop**.

When Emacs runs on the remote machine inside tmux, a nearby GPU endpoint may avoid sending editing context back across the SSH connection. A laptop model may still be attractive, but the complete latency path has to be measured.

For completion, the useful quantity is:

\[
T_{\text{suggestion}}
=
T_{\text{debounce}}
+
T_{\text{network}}
+
T_{\text{queue}}
+
T_{\text{prefill}}
+
T_{\text{generation}}
+
T_{\text{render}}
\]

I would set aspirational targets around **200 ms median and 500 ms p95 for the first useful suggestion**, then adjust scope based on actual hardware and network measurements. These are product targets, not expected benchmark results.

My initial serving design would use short completion outputs, bounded context, cancellation of obsolete requests, and revision checks before display or acceptance. Completion requests should not sit behind long operator generations in the same queue.

For the operator, I would default to short non-thinking actions for routine steps and separately evaluate whether extra reasoning helps difficult recovery tasks. Qwen3.5 documents disabling its default thinking behavior through the serving template configuration. :chatgpt-content-reference{index="15"}

On training compute, I would profile before estimating GPU-hours. Weight storage alone is approximately \(2P\) bytes in BF16 or \(P/2\) bytes at an idealized four bits per parameter; activations, optimizer state, quantization metadata, serving caches, and auxiliary modules are additional.

For this project, I would budget engineering effort for **resetting and supervising many Emacs/LSP/kernel environments**, not just fitting the learner. That is a different workload from ordinary static-dataset fine-tuning.

### 8. The first experiment I would run

I would keep the first milestone deliberately concrete.

**Operator track:** build one reliable state/action adapter covering buffers, LSP inspection, selective Magit staging, and Jupyter attachment/execution. Establish a locked evaluation suite, generate several thousand verified demonstrations, and compare Qwen3.5-4B before and after SFT under the identical harness. Use 9B to test whether remaining failures are capacity-sensitive.

**Completion track:** connect an unmodified Qwen2.5-Coder-1.5B to Doom first, measure the actual latency and suggestion behavior, then fine-tune it on realistic FIM windows. Compare it with the unmodified model and a larger completion checkpoint. Add repository context as a separate ablation.

**Combined track:** verify that the operator and completion system can coexist with your edits. Neither should apply stale changes, steal focus unnecessarily, submit a cell twice, or confuse buffers that share a filename.

The operator scorecard should report task success, preservation of unrelated state, recovery success, unauthorized-action attempts, and time/actions per successful task. The completion scorecard should report latency, retained suggestions, correction effort, suffix duplication, invalid-symbol generation, and functional checks where available.

Zero observed destructive failures in a small test set would not establish safety; those cases need explicit adversarial and concurrency testing.

**My recommendation is therefore: Qwen3.5-4B as the first operator experiment, Qwen2.5-Coder-1.5B as the first completion experiment, and one shared Emacs state/trace layer underneath both.** The most valuable deliverable is not initially either checkpoint. It is a reproducible collection of your workflows, executable checks, and realistic editing traces—because that is what lets you tell whether post-training has made the system genuinely better at working with you.

## Source Checks To Complete

The draft this RFC is based on cited external model cards, package docs, and tool documentation. Before implementation depends on them, verify and pin:

1. Qwen3.5 model card claims for tool calling, non-thinking mode, and checkpoint availability.
2. Qwen2.5-Coder 1.5B/3B/7B licenses and FIM interfaces.
3. Mellum and Mellum2 model cards, FIM formats, and active/resident parameter implications.
4. Emacs `buffer-chars-modified-tick` behavior.
5. Emacs change-group and `atomic-change-group` rollback behavior.
6. `mcp-server-lib.el` and `rhblind/emacs-mcp-server` operation surfaces.
7. `emacs-jupyter` association, `completion-at-point`, and kernel/session behavior.
8. Jupyter protocol request/reply correlation through headers and `parent_header`.
9. TRL assistant/completion loss masking behavior with the selected chat template.
10. `verl` agent-loop masking, rollout, serialization, and log-probability behavior.
11. RLinf completion experiment details and its judge/evaluation setup.
12. Minuet and Tabbymacs interfaces for FIM, next-edit prediction, ghost text, and synchronization.
