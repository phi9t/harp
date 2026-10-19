# RFC 0003: Safe Emacs Agent API

Status: Implemented
Created: 2026-09-30
Quest: Give coding agents a safe, semantic API for operating Emacs without
freezing a CUA or vision interface first

## Summary

The first model-facing Emacs interface should be an API over live Emacs state,
not a screen-control or CUA/vision task. Terminal input and screenshots remain
important validation layers, but the initial product surface should teach
agents to operate Emacs through explicit buffer, process, language-server,
Jupyter, Org, and report-building semantics.

The goal is a small, versioned operation set that lets an agent do coding,
terminal, analysis, and reporting work while preserving the editor state it did
not intend to change. Each mutating operation must take operation-specific
preconditions and return evidence about the path taken, the state changed, and
any asynchronous work still in flight.

This RFC updates RFC 0001's sketch of an operation surface using the Internals
Atlas and selected-rootfs benchmark evidence from RFC 0002. The existing bridge
operations are evidence and prototypes; this RFC defines the safer
model-facing interface they should converge toward.

## Non-Goals

- No CUA or vision operation profile yet. A future profile may use PTY keys,
  screenshots, or display capture, but this RFC is the API-first layer.
- No arbitrary Elisp or direct function call as a normal agent capability.
  Debug and extension hooks may exist for trusted harness work, not general
  policy actions.
- No claim that final text alone proves correctness. Correctness includes
  path, context, ownership, side effects, async lifecycle, and completion
  evidence.
- No package-independent "undo" promise. Filesystem changes, Git operations,
  kernel execution, process input, and debugger controls each need their own
  recovery model.
- No implicit kernel restart, host process fallback, or host toolchain fallback.
  The selected Insula rootfs remains the authority for runtime closure and ABI
  evidence.

## Design Principle

The bridge should be a deep module: a small model-facing interface with strong
contracts, backed by package-specific adapters behind the seam. The interface
is not just operation names. It includes invariants, ordering constraints,
preconditions, failure modes, timeout semantics, and evidence requirements.

```text
observe -> choose capability -> mutate with preconditions -> await/reconcile
```

Every operation belongs to an owner:

| Owner | Examples of state it owns |
| --- | --- |
| Core Emacs | buffers, text owners, windows, frames, point, region, narrowing, minibuffer, command-loop state |
| Completion | provider result, replacement range, preview/overlay, exit function, undo grouping |
| LSP | workspace, client/server process, document version, position encoding, diagnostics, workspace edits |
| Magit | repository, index/worktree, status/diff buffer, section identity, region, command context |
| Comint | process object, process mark, pending input, history, process output |
| Jupyter | source buffer association, client, kernel incarnation, request ids, display data |
| Org | outline tree, IDs/properties, source-edit parent, Babel session, export artifacts |
| Dired/files | directory buffer marks/flags and independent filesystem state |
| Debugger | session, adapter/backend, process, stack, frame, breakpoints, watches |

The agent should not be asked to infer ownership from raw buffer text. The
model-facing interface should make the current owner and required evidence
explicit.

## Request Envelope

All agent-facing requests should use one envelope:

```json
{
  "api_version": "soma-emacs-agent-v0",
  "request_id": "uuid-or-stable-task-id",
  "operation": "buffer.replace_range",
  "capability": "reversible_editor_edit",
  "params": {},
  "preconditions": {},
  "timeout_ms": 30000
}
```

Required behavior:

- `request_id` is idempotency identity. Reusing it must return the stored
  request result or a duplicate-request error, not resubmit consequential work.
- `operation` must be selected from the enabled profile allowlist.
- `capability` declares the side-effect class the caller believes it is using.
  The bridge rejects mismatches such as a raw process send under a passive
  observation capability.
- `preconditions` are typed and operation-specific. A character revision alone
  is never the universal freshness check.
- `timeout_ms` affects waiting, not whether the submitted external work
  happened. After consequential submission, timeout normally becomes
  `outcome_unknown` until reconciled.

## Result Envelope

Every operation returns the same outer result shape:

```json
{
  "api_version": "soma-emacs-agent-v0",
  "request_id": "uuid-or-stable-task-id",
  "operation": "buffer.replace_range",
  "state": "succeeded",
  "state_sequence": ["accepted", "running", "succeeded"],
  "result": {},
  "evidence": {},
  "observations": {},
  "effects": {},
  "diagnostics": [],
  "continuation": null
}
```

Allowed states are:

- `accepted`: request shape and preconditions were accepted.
- `running`: accepted work is still in flight.
- `succeeded`: completion evidence proves the intended state exists.
- `failed`: a precondition failed, the operation failed before consequential
  submission, or evidence proves the intended state does not exist.
- `waiting_for_input`: Emacs is at an explicit prompt, transient, minibuffer,
  completion, or continuation point.
- `outcome_unknown`: the adapter stopped waiting or lost the handle after a
  consequential request may have been submitted.

`evidence` should include the actual Emacs route used, package-specific handles,
and rootfs/runtime identity when external tools or kernels are involved.
`effects` should summarize changed buffers, files, processes, Git state,
kernel requests, or exported artifacts.

The current bridge keeps legacy operation payloads under `result` while adding
the v0 envelope fields around them. That is an implementation checkpoint, not
the final shape: operation-specific evidence should migrate from ad hoc result
keys into the typed `evidence`, `observations`, `effects`, and `diagnostics`
sections before the API is frozen. The migration must be operation-scoped:
source coordinates, prompt ranges, and request spans are evidence only; they
must not be projected as editor text effects unless the operation actually
mutates text.

## Stable Handles

The API should name live state through opaque handles that include enough
identity to detect drift:

| Handle | Minimum identity |
| --- | --- |
| `editor_session` | Emacs pid/daemon name, API version, profile id, selected rootfs id |
| `frame` / `window` | frame/window id, terminal/client identity when present |
| `buffer` | buffer object id, name, mode, modified state, point/mark/narrowing summary |
| `text_owner` | base text identity for direct and indirect buffers |
| `file` | truename or fixture-relative path plus mtime/hash where relevant |
| `project` | root, VCS identity, build/test profile |
| `lsp_document` | workspace id, client id, uri, document version, position encoding |
| `magit_section` | status buffer generation, section type/value/range, repo fingerprint |
| `comint_process` | buffer id, process id, process mark, cwd/env/tool identity |
| `jupyter_client` | source buffer id, client object, kernel id/session/incarnation |
| `org_node` | buffer id, heading path, ID/property identity, structural fingerprint |
| `debug_session` | backend, adapter process, target process, thread/frame ids |

Handles are not durable authority by themselves. Mutating calls must still
validate relevant surrounding text, package state, or external state before
acting.

## Observation Classes

Observation must declare the class of work it performs:

| Class | Contract |
| --- | --- |
| Passive | Reads bounded state without intentional package loading, visible display changes, or external requests |
| Derived | Runs bounded package queries, parsing, or cache reads and reports any activation or cost |
| Active diagnostic | May trigger LSP requests, process status checks, package loading, export planning, or kernel/debugger inspection |

This matters because even "read" APIs in Emacs can invoke fontification,
display, package hooks, server requests, or process filters. The API should not
pretend all reads are pure. It should state what semantic state must remain
unchanged.

## Capability Profiles

The first API should be shipped as profiles, not one global capability soup.

| Profile | Intended work | Initial operations |
| --- | --- | --- |
| `core-v0` | Inspect session state, select existing buffers, and manage stable request lifecycle | `api.list_capabilities`, `api.describe_operation`, `emacs.observe_session`, `emacs.observe_buffer`, `buffer.select`, `emacs.observe_frames`, `emacs.observe_interaction`, `request.status`, `request.await` |
| `coding-v0` | Read and edit source safely | `project.observe`, `project.search`, `buffer.read`, `buffer.replace_range`, `buffer.apply_patch`, `buffer.save`, `completion.observe`, `completion.accept`, `lsp.observe_source`, `lsp.observe_diagnostics`, `lsp.rename_symbol` |
| `terminal-v0` | Use process buffers without raw process writes | `comint.observe`, `comint.submit_input`, `comint.await_output`, `comint.interrupt` |
| `analysis-v0` | Run selected analysis code in existing kernels | `jupyter.observe_source`, `jupyter.attach_existing`, `jupyter.eval_region`, `jupyter.await_request`, `jupyter.observe_display` |
| `reports-v0` | Maintain tasks and publish reports | `org.observe_tree`, `org.update_heading`, `org.edit_src_block`, `org.exit_src_edit`, `org.execute_babel_block`, `org.export_html` |
| `vcs-files-v0` | Stage intended changes and manage files | `magit.observe_sections`, `magit.stage_hunk`, `magit.stage_file`, `dired.observe`, `dired.rename_no_overwrite`, `dired.delete_marked` |
| `debug-inspect-v0` | Observe debugger state without execution control | `debug.observe_session`, `debug.observe_stack`, `debug.observe_breakpoints`, `debug.observe_locals` |

`debug-inspect-v0` is a design placeholder until the Atlas has a debugger
contract. Stepping, continuing, expression evaluation, and breakpoint mutation
must remain out of the default safe subset until the selected backend
(`dap-mode`, `gud`, or another backend) has source-grounded lifecycle probes.

`request.respond` is intentionally not part of the first stable `core-v0`
profile. The current bridge can complete a synthetic `bridge.prompt_echo`
request, which is useful for exercising waiting-for-input lifecycle plumbing,
but that does not prove ownership of real minibuffer, Transient, or package
prompt state. It remains experimental until prompt-owner preconditions,
cancellation semantics, reentry behavior, and completion evidence are covered by
selected-rootfs probes. Explicit `soma-emacs-agent-v0` requests must reject it.

## Core Operations

### `api.list_capabilities`

Returns enabled profiles, operations, side-effect classes, required
precondition fields, and whether each operation is implemented, experimental,
or benchmark-only.

### `api.describe_operation`

Returns operation documentation generated from the installed environment and
Soma contract docs. This is the primary way to make agents familiar with the
safe subset without training them to hallucinate Emacs commands.

The description should include:

- required owner and handles
- preconditions and stale-handle behavior
- expected route, such as passive read, package helper, command dispatch, or
  terminal input
- possible terminal states
- examples of valid and invalid requests

### `emacs.observe_session`

Returns session, profile, rootfs, Doom, package, feature, and selected-frame
identity. This is the first observation in an episode and the anchor for
Insula/runtime evidence.

### `emacs.observe_buffer`

Returns buffer identity, text-owner identity, file identity, mode, modified
state, point, region, narrowing, selected-window relationship, read-only state,
and relevant revision counters.

### `buffer.select`

Selects an existing live buffer in the selected window and optionally sets
point. It is a core editor-state operation, not a file-opening, package-command,
or text-mutation operation. Preconditions should name the intended live buffer
and, when point is provided, the expected valid point range. The result records
before/after buffer observations and selected-window evidence.

### `emacs.observe_interaction`

Returns minibuffer, recursive edit, completion, prompt, transient, current
command, last command, prefix arg, and command-dispatch context. It should not
replace package-specific observations such as Magit section state or Jupyter
kernel request state.

## Coding Operations

### `project.observe` and `project.search`

Agents need a source-aware way to discover files without shelling out
arbitrarily. These operations should report project root, VCS identity, ignored
paths policy, search tool identity, and bounded results. The rootfs owns the
search tools used by the operation.

### `buffer.read`

Reads text by buffer handle and range. It should preserve point, mark,
narrowing, selected window, and text revisions unless the observation class
explicitly permits derived work.

### `buffer.replace_range`

Replaces a range in one buffer with strict preconditions:

- buffer id and text-owner id match
- expected file and mode match when provided
- expected text/range/prefix/suffix match
- character revision and operation-specific UI state match
- point, region, narrowing, read-only, selected-window, and minibuffer state
  match if the caller declared them as relevant

The result reports undo grouping, before/after observations, and changed text
range. This operation is a local edit, not a semantic refactor.

### `buffer.apply_patch`

Applies a multi-hunk, one-buffer exact-context patch only after checking buffer
identity, text-owner identity, visited-file identity, character revision, point,
selected-window, modified, read-only, and minibuffer state. The first stable
patch format is `soma-context-patch-v0`: a bounded list of hunks, each with
literal `old_text` and `new_text`. Each `old_text` must occur exactly once in
the current buffer and hunk ranges must not overlap. The result reports a
conflict preview, changed ranges, and before/after observations.

This operation is the bridge between simple range edits and future multi-buffer
transactions. Multi-buffer atomicity should be a separate operation with
explicit rollback limits.

### `completion.observe` and `completion.accept`

Completion acceptance must not be reduced to raw insertion. The selected CAPF
provider, replacement range, preview/overlay state, prefix/suffix, point,
revision, and provider exit function are part of the contract.

`completion.observe` records the first applicable CAPF provider, bounded
candidate strings, the replacement range, preview state, and accept-ready
preconditions containing buffer/text owner, character revision, point,
selected-window, mark/narrowing/read-only/minibuffer state, expected range
text, and expected prefix/suffix.

`completion.accept` succeeds only if the displayed or selected suggestion is
still valid for the observed context. If raw insertion would produce the same
characters while skipping provider exit behavior, it is a different operation
and should not count as faithful completion acceptance.

### `lsp.observe_source`

Returns workspace, client/server identity, document uri, version, position
encoding, passive symbol-at-point summary, diagnostics, and server/toolchain
rootfs evidence. It may flush pending LSP document changes and drain server
responses before reading lsp-mode's protocol/cache state, but it should avoid
issuing unrelated interactive LSP requests while serving as a diagnostic wait
barrier.

### `lsp.observe_diagnostics`

Returns focused diagnostic evidence for one LSP-owned source buffer: workspace
root/status, server identity, document URI/version, position encoding, protocol
cache diagnostics, and operation-specific owner/freshness checks. It may flush
pending lsp-mode document changes and drain server responses, but it does not
return source text; callers should use `buffer.read` when they need text for a
separate guarded edit.

### `lsp.rename_symbol`

Executes the real LSP rename path, records the request, response/workspace edit,
affected files, document-version checks, saves when requested, and post-rename
diagnostics/tests. Direct text replacement is not a valid substitute.

Future coding operations should include:

- `lsp.find_references`
- `lsp.goto_definition`
- `lsp.hover`
- `lsp.code_action_preview`
- `lsp.apply_workspace_edit`
- `lsp.format_range`

## Terminal And Process Operations

Comint is the first terminal/process API because it has real editor semantics
around input boundaries, history, process marks, and process filters.

### `comint.observe`

Returns buffer/process identity, process liveness, cwd, environment, PATH/tool
resolution, process mark, point relative to mark, pending input, old input
context, recent output tail, and rootfs run identity.

### `comint.submit_input`

Submits input through `comint-send-input`, not raw `process-send-string`.
Preconditions must cover intended buffer, live process, process mark, pending
input text, point relationship, cwd/env/tool identity, and whether the agent
intends bottom input or old-history input.

### `comint.await_output`

Waits for output from the intended process and classifies partial output,
completion, timeout, and unknown outcomes. It must not read a similarly named
process buffer as evidence.

### `comint.interrupt`

Interrupts through the package-owned command path and then reconciles process
identity, partial output, exit/survival status, and whether the interrupted
command's side effect happened.

Raw terminal key dispatch belongs to a later profile. The API can still be
validated against terminal-frame evidence, but coding agents should not need
CUA or visual screen control for the first safe subset.

## Jupyter Analysis Operations

The first Jupyter API is existing-kernel only.

### `jupyter.observe_source`

Returns the source buffer, selected region, associated client, kernel identity,
kernel incarnation/session, execution status, request handles, and rootfs-owned
Python/runtime evidence. The observation publishes both the Jupyter connection
file and the kernel executable, plus explicit
`jupyter_connection_file_rootfs_owned` and
`jupyter_kernel_exe_rootfs_owned` booleans. Valid runtime connection files may
live under the selected Insula's runtime/project tmp paths; kernel executables
must resolve to the selected rootfs-owned executable closure.

### `jupyter.attach_existing`

Associates a source buffer with an intended existing REPL/client. It must never
silently launch or restart a kernel to satisfy the request.

The attach preconditions include the observed connection file, kernel
executable, and both rootfs-owned booleans. If Emacs has a live associated
client whose metadata points at a host-shadow connection file or kernel
executable, attach fails before changing the source/client association.

### `jupyter.eval_region`

Executes exactly the observed source range through the associated existing
kernel. Preconditions include source buffer identity, selected range text,
client id, kernel id/session/incarnation, rootfs runtime identity, and
non-busy or explicitly accepted busy-state policy.

Execution preconditions also require the observed Jupyter connection file and
kernel executable to be rootfs-owned. This is checked immediately before
submission; host-shadow metadata must produce `failed` or `conflict` with no
kernel execution effect.

### `jupyter.await_request`

Waits for shell reply, IOPub outputs, display data, and idle status matched by
request id and parent header. Timeout after submission is `outcome_unknown`
until reconciled; blind retry of non-idempotent code is rejected.

### `jupyter.observe_display`

Reports display ids, display-data/update ordering, rendered text ranges in the
REPL view, and source request ownership. The correct printed value from the
wrong kernel is a failure.

## Org Task And Report Operations

Org is the first reporting surface because it can own tasks, prose, source
blocks, Babel results, and HTML export in one editor-native document.

### `org.observe_tree`

Returns a bounded structural parse: headings, TODO state, IDs, properties,
tags, source blocks, result blocks, narrowed state, point/region, and file
identity.

### `org.update_heading`

Updates a heading's TODO state and title using an Org-structure precondition.
It must preserve IDs, drawers, children, source blocks, and neighboring heading
boundaries unless the request explicitly changes them. Tags, properties, and
body edits are later extensions, not part of the first stable slice.

### `org.edit_src_block` and `org.exit_src_edit`

Source editing must distinguish the edit buffer from the parent Org document.
Success requires parent-buffer writeback through the Org source-edit workflow,
not merely correct text in the temporary edit buffer.

`org.edit_src_block` validates the parent buffer, text owner, file identity,
character revision, point/region/narrowing state, Org structure
fingerprint, live heading identity, and live source-block hash before moving to
the observed source block and calling `org-edit-src-code`. It returns the
Org-created edit buffer name plus parent marker evidence. `org.exit_src_edit`
validates the edit-buffer revision and text hash, verifies that the edit buffer
is still connected to the intended parent markers/source block, rechecks the
parent buffer revision and structure fingerprint, and only then calls
`org-edit-src-exit`. Stale parent drift fails with `conflict` before writeback.

### `org.execute_babel_block`

Executes a selected Babel block with explicit session, default-directory,
language, and rootfs/runtime evidence. It must report parent-buffer result
insertion and external process/kernel ownership.

The first stable slice executes observed `emacs-lisp` blocks through
`org-babel-execute-src-block` only when the request echoes the observed source
block, parent Org node, structure fingerprint, buffer/text owner,
point/region/narrowing/read-only/minibuffer state, explicit
`language_allowlist`, expected session, and expected
`default_directory`. It disables Org's interactive confirmation prompt for the
checked execution path, records the loaded Babel backend and executor function,
and returns parent `#+RESULTS:` block evidence after Org inserts it. A stale
parent buffer, changed source block, mismatched language, unsupported backend,
wrong session, or wrong `default_directory` fails with `conflict` before
execution.

### `org.export_html`

Exports a report to HTML through Org's export pipeline with explicit output
path, backend, export options, generated artifacts, asset policy, and explicit
mode.

The stable operation has preview and apply modes:

- preview validates output path, export backend, options, source block
  execution policy, and external asset policy, then returns plan evidence
  without writing an artifact or publishing `effects.html_export`
- apply runs export and returns changed/generated files plus diagnostics under
  `effects.html_export`

Pretty HTML export can be supported without giving the agent arbitrary shell
access. CSS, themes, and publishing directories are report capabilities, not
ambient filesystem authority.

## VCS And File Operations

### `magit.observe_sections`

Returns repository identity, status/diff buffer generation, selected section,
region, diff type/scope, repo fingerprint, and command-context evidence.

### `magit.stage_hunk`, `magit.stage_file`, and `magit.stage_region_files`

These are semantic operations over Magit sections, not shell `git add`
shortcuts. They must preserve recognized Magit command identity where hooks and
package behavior depend on it, refresh the view, and verify exact index and
worktree effects.

### `dired.observe`

Returns directory identity, marks, deletion flags, active region, file
signatures, collision policy, and whether displayed state is stale relative to
the filesystem.

### `dired.rename_no_overwrite` and `dired.delete_marked`

Filesystem operations need independent pre/post filesystem checks. Dired buffer
undo is not filesystem rollback and must not be reported as such.

`dired.delete_marked` deletes only the current ordinary Dired marked set through
`dired-do-delete` with trash and recursive deletes disabled for the v0 checked
path. The request must name the Dired buffer, directory, exact marked basenames,
per-file signatures, and delete policy. The bridge rejects mismatched buffers,
wrong directories, unsafe names, mark drift, deletion-flag confusion, and
same-path file replacement before destructive deletion.

## Debugger Context

Debugger context is useful for coding agents, but the safe subset should start
with observation only because Soma does not yet have a debugger Atlas contract.

The intended direction is:

```text
debug.observe_session
debug.observe_stack
debug.select_frame
debug.observe_locals
debug.observe_breakpoints
debug.evaluate_watch_preview
```

Execution controls such as continue, step, restart, kill target, mutate
breakpoints, or evaluate arbitrary expressions should require a backend-specific
contract and a stronger capability than `debug-inspect-v0`.

The debugger contract must identify the owner of state: Emacs buffer, debugger
frontend, adapter process, target process, inferior terminal, remote target,
source mapping, and variable/object lifetime.

## Command Dispatch As Escape Hatch

`command.dispatch` remains useful, but it should not become the main API. Use
typed semantic operations when the package contract is known. Use command
dispatch only when:

- the command is allowlisted by the active profile
- preconditions include selected buffer/window, point/region, minibuffer,
  transient, package view, command identity, and expected prompt behavior
- the result records command route, `this-command`, hooks or prompt state, and
  package-specific completion evidence

Direct function calls and raw Elisp evaluation should remain harness/debug
capabilities, not learner-facing operations.

## Safety And Recovery Rules

The first API should reject stale observations by default. Automatic refresh
and mutation in one step is allowed only for operations whose contract says so.

Common rules:

- Inspect before mutation.
- Revalidate after waits and asynchronous callbacks, not only before
  submission.
- Separate reversible editor edits from externally consequential work.
- Treat timeout after consequential submission as `outcome_unknown`.
- Return conflict evidence instead of overwriting newer human edits.
- Preserve unrelated buffers, windows, selections, staged changes, kernel
  state, process state, and report artifacts.
- Never let efficiency rewards compensate for a policy or preservation
  violation.

## API Introspection For Agents

The API itself should help agents learn the safe subset:

- `api.list_capabilities` lists enabled profiles and operation maturity.
- `api.describe_operation` explains owner, path, preconditions, failure states,
  and examples.
- `emacs.describe_command` explains an installed command and its advice/hooks
  when command dispatch is allowed.
- `state.explain_handle` explains why a handle is stale or insufficient.
- `state.summarize_recent_events` gives bounded recent prompt/request/process
  history without exposing private grader data.

These operations are not a substitute for training data. They are the installed
environment's live documentation and should prevent the model from relying on
generic Emacs lore when Doom/profile/package state differs.

## First Stable Slice

The first stable API profile should include:

```text
core-v0:
  api.list_capabilities
  api.describe_operation
  emacs.observe_session
  emacs.observe_buffer
  emacs.observe_frames
  buffer.select
  emacs.observe_interaction
  request.status
  request.await

coding-v0:
  project.observe
  project.search
  buffer.read
  buffer.replace_range
  buffer.save
  completion.observe
  completion.accept
  lsp.observe_source
  lsp.observe_diagnostics
  lsp.rename_symbol

terminal-v0:
  comint.observe
  comint.submit_input
  comint.await_output
  comint.interrupt

analysis-v0:
  jupyter.observe_source
  jupyter.attach_existing
  jupyter.eval_region
  jupyter.await_request
  jupyter.observe_display

reports-v0:
  org.observe_tree
  org.update_heading
  org.edit_src_block
  org.exit_src_edit
  org.execute_babel_block
  org.export_html

vcs-files-v0:
  magit.observe_sections
  magit.stage_hunk
  magit.stage_file
  dired.observe
  dired.rename_no_overwrite
  dired.delete_marked
```

This is enough for a coding agent to inspect a project, edit buffers, apply an
LSP rename, run tests through Comint, run selected Jupyter analysis, update an
Org task/report document, export HTML, and stage intended changes without
screen-reading or generic shell shortcuts.

## Freeze Evidence

The first six `soma-emacs-agent-v0` profiles are frozen when these checks pass:

1. A machine-readable operation manifest generated by the selected bridge.
2. JSON schemas or equivalent validators for request/result envelopes and
   operation-specific preconditions.
3. A profile-level smoke benchmark that uses only the stable API, not
   benchmark-only fixtures.
4. Per-operation negative tests for stale handles, wrong owner, wrong rootfs
   runtime, duplicate request id, and timeout reconciliation.
5. A clear distinction in traces between stable API operations,
   experimental operations, benchmark-only fixtures, and harness/debug
   capabilities.

The selected-rootfs benchmark evidence below turns the package-specific
behavior into a stable contract for training and serving agents.

Current implementation checkpoint:

- `api.list_capabilities`, `api.describe_operation`, `emacs.observe_session`,
  `emacs.observe_frames`, `buffer.select`, `project.observe`, `project.search`, `org.observe_tree`,
  `org.export_html`, `org.update_heading`, `comint.observe`, `comint.submit_input`,
  `comint.await_output`, `comint.interrupt`, `lsp.observe_source`,
  `lsp.observe_diagnostics`,
  `lsp.rename_symbol`, `jupyter.observe_source`, `jupyter.attach_existing`,
  `jupyter.eval_region`, `jupyter.await_request`, `jupyter.observe_display`,
  `magit.observe_sections`, `magit.stage_hunk`,
  `magit.stage_file`, `dired.observe`, `dired.rename_no_overwrite`, and
  `dired.delete_marked` are implemented in the shared bridge and exposed by the
  live daemon allowlist. Stable checked edit operations also include
  `buffer.read`, `buffer.replace_range`, `buffer.apply_patch`, and
  `buffer.save`.
- `emacs.observe_session` reports the v0 API/session identity, Emacs pid and
  daemon state, source-built Emacs executable and source directory, selected
  Insula run/plan/rootfs/closure identity, rootfs-owned Doom paths and commit,
  loaded feature/module state when present, and selected frame/window/buffer
  context.
- `emacs.observe_frames` reports daemon-local frame identity, terminal/client
  ownership, selected-window buffer state, and typed passive evidence without
  mutation effects.
- `project.observe` reports project root, Git identity, rootfs-owned `git`
  evidence, rootfs-owned `rg` evidence, run id, and selected Insula plan path.
- `project.search` searches bounded source text with rootfs-owned ripgrep,
  returns bounded match and structured submatch evidence, and records
  rootfs-owned tool identity.
- `buffer.select` selects an existing live buffer in the selected window,
  optionally sets point, records before/after buffer observations, and does not
  open files, invoke package commands, or mutate buffer text.
- `comint.submit_input` submits through `comint-send-input`, records before
  and after Comint ownership state, and rejects stale process id, stale process
  executable, non-rootfs-owned process executable, stale process mark, wrong
  buffer name, wrong buffer mode, or changed pending input before mutating the
  process buffer.
- `comint.await_output` waits for matched output from the checked process
  boundary, reports timeout as `outcome_unknown`, and rejects wrong buffer,
  wrong process owner, non-rootfs-owned process executable, non-Comint buffers,
  and invalid output boundaries.
- `comint.interrupt` checks the intended Comint buffer, process id, process
  executable ownership, process mark, and optional pending input before
  dispatching `comint-interrupt-subjob`. It records before/after Comint state,
  reports that no raw process write or bridge-level signal path was used, and
  relies on the benchmark grader to reconcile whether delayed external side
  effects occurred.
- `lsp.observe_source` first proves the live LSP server process is owned by the
  selected rootfs, then flushes pending lsp-mode changes, reads document
  version, server identity, diagnostics, and diagnostic-cache/protocol evidence,
  and reports passive `thing-at-point` symbol evidence rather than issuing
  repeated active symbol requests during diagnostic polling. If the server
  executable resolves outside the rootfs-owned closure, observation fails closed
  and skips LSP sync/pull work.
- `lsp.observe_diagnostics` is now the stable diagnostics-only API. It uses the
  same lsp-mode sync/cache path, reports workspace/server/document ownership and
  precondition checks, rejects host-owned server processes before requesting
  diagnostics, and leaves source text acquisition to explicit `buffer.read`
  calls.
- `lsp.rename_symbol` runs through the real LSP rename command path, records
  LSP request/workspace-edit evidence, validates document/version context, and
  verifies affected Python source/test files plus rootfs-owned test execution.
  Its preconditions include live rootfs-owned LSP server ownership, so a
  host-owned server cannot satisfy rename capability or symbol checks.
- `jupyter.observe_source` publishes the observed Jupyter connection file,
  kernel executable, and explicit connection/executable rootfs-owned booleans.
  `jupyter.attach_existing` associates a source buffer with the intended
  existing Jupyter REPL/client through `jupyter-repl-associate-buffer`, rejects
  wrong-kernel and wrong-owner preconditions, and never launches or restarts a
  kernel implicitly.
- `jupyter.eval_region` and `jupyter.eval_region_via_repl` revalidate the
  observed connection file and kernel executable ownership before submission,
  so source-buffer metadata that points outside the selected rootfs fails before
  kernel execution. The benchmark-only `jupyter.query_state` path performs the
  same live ownership preflight before issuing a kernel state query.
- `jupyter.await_request` awaits an existing bridge-owned Jupyter request by
  request id, reports the target request status and original reply/IOPub/idle
  evidence through the `jupyter-request-await` route, and does not resubmit
  non-idempotent code after submission. It preserves the target request's
  terminal state: if the target is still `outcome_unknown`, semantic await also
  reports `outcome_unknown` rather than masking the unknown outcome as success.
- `org.update_heading` resolves the observed Org node, preferring its `ID`
  property, checks the buffer/text owner, character revision,
  point/region/narrowing/read-only/minibuffer state, structure fingerprint, and
  live heading TODO/title identity, then mutates through Org's headline/TODO
  APIs. It rejects stale structure preconditions before editing and records
  before/after Org tree evidence. Selected-window ownership is intentionally
  not part of this semantic parent-buffer contract.
- `org.edit_src_block` and `org.exit_src_edit` use Org's source-edit workflow,
  not direct parent-buffer text replacement. They check parent Org identity and
  source-block hashes before opening the edit buffer, record Org's parent marker
  range, validate edit-buffer revision/text and parent freshness before exit,
  write back through `org-edit-src-exit`, and reject stale parent drift before
  writeback. The parent-buffer checks do not require the daemon's currently
  selected window to display the Org buffer.
- `magit.observe_sections` reports Magit status sections with section identity,
  selected section state, repo fingerprint, and command-context evidence.
- `magit.stage_hunk` and `magit.stage_file` resolve live Magit sections in the
  target Magit buffer, stage via Magit's package-owned staging command context
  with `this-command` set to `magit-stage`, refresh/re-observe the view, verify
  exact index/worktree effects, and reject stale repo fingerprints or stale
  section identities.
- Operation descriptions now include machine-readable `request_contract`,
  `precondition_contract`, and `result_contract` objects derived from the
  operation manifest. These describe the v0 envelope, capability,
  required params, typed preconditions, possible states, required result fields,
  and evidence channels.
- Explicit `soma-emacs-agent-v0` requests are now checked against the manifest
  before dispatch. Missing manifest-required params fail with
  `missing_required_param`; missing manifest-required preconditions fail with
  `missing_required_precondition`; operations whose manifest maturity or profile
  is `benchmark-only` fail with `benchmark_only_operation` when invoked through
  the explicit stable-v0 envelope. Experimental and planned manifest entries
  fail with `non_stable_operation`, so operations such as `request.respond`
  remain available only to internal/mechanism tests until their owner contracts
  are proven. Rejection-only negative-test operations may still run through the
  envelope to prove that raw process, filesystem, Git, kernel, and parent-buffer
  shortcuts do not perform side effects.
- The bridge emits additive v0 result-envelope fields:
  `api_version`, `state`, `state_sequence`, `result`, `evidence`,
  `observations`, `effects`, `diagnostics`, and `continuation`.
- The first typed-envelope migration slices are implemented for stable
  core/project observations, checked buffer edits, coding-v0 LSP operations,
  stable passive read/observe operations, terminal-v0 Comint operations, and
  analysis-v0 Jupyter operations, plus reports-v0 and vcs-files-v0 mutating
  operations.
  `emacs.observe_buffer` publishes typed buffer identity, text-owner, and
  point/region/narrowing evidence plus a focused `observations.buffer_state`
  object without effects. `emacs.observe_session` publishes typed editor
  session, selected rootfs, and Doom identity evidence plus the selected-frame
  observation without effects. `project.observe` publishes typed project root,
  VCS, rootfs-owned search-tool, and rootfs runtime evidence plus
  `observations.project_context`. `project.search` publishes typed
  rootfs-owned search-tool, bounded-result, and runtime evidence plus
  `observations.matches` without effects.
  `buffer.read` publishes typed `buffer_identity` and `text_range` evidence
  plus `observations.text_snapshot` without effects. `org.observe_tree`
  publishes typed buffer identity, structure fingerprint, heading index, and
  source-block index evidence plus `observations.org_tree` without effects.
  `dired.observe` publishes typed directory, selection-state, and
  file-signature evidence plus `observations.dired_listing` without effects.
  `comint.observe` publishes typed process identity, input-boundary, and
  rootfs-runtime evidence plus `observations.comint_state` without effects.
  `magit.observe_sections` publishes typed repository fingerprint, status
  generation, and selected-section evidence plus
  `observations.sections` without effects.
  `buffer.replace_range` publishes
  precondition checks and before/after observations under top-level `evidence`,
  before/after buffer snapshots under `observations`, and the actual text
  mutation under `effects.changed_range`. It rejects ranges that overlap Org
  source blocks in parent Org buffers, so package-owned Org source editing
  cannot be bypassed by a generic text replacement. `buffer.apply_patch`
  publishes precondition checks, conflict previews, before/after observations,
  and changed hunks through the same typed top-level sections.
  `comint.submit_input` publishes submitted byte count, Comint input-marker
  state, and process output under top-level `evidence`, with the actual process
  input under `effects.process_input`. `comint.await_output` publishes matched
  and partial output plus timeout evidence without claiming process mutation
  effects. `comint.interrupt` publishes interrupt command, process
  identity/survival, and side-effect reconciliation under top-level `evidence`,
  with the process-control action under `effects.process_interrupt`.
  `jupyter.observe_source` publishes typed source buffer, associated client,
  kernel incarnation, selected region, and rootfs runtime evidence without
  effects. `jupyter.attach_existing` publishes association and kernel identity
  evidence, then reports the source/client association as
  `effects.source_client_association` only on success. `jupyter.eval_region`
  and `jupyter.eval_region_via_repl` publish request id, submitted hash, source
  region, reply, IOPub stream/display/update messages, idle status, and
  rootfs runtime evidence, with the actual consequential action reported as
  `effects.kernel_execution`. `jupyter.await_request` publishes target status
  and reply/IOPub/idle/timeout evidence without resubmitting kernel work.
  `jupyter.observe_display` publishes display id, display/update data, and
  REPL range evidence without effects. Projection is operation-scoped, so
  Jupyter source coordinates are not reported as editor text effects.
  `org.update_heading` publishes the semantic heading mutation under
  `effects.org_heading_update`. `org.edit_src_block` publishes the separate
  source-edit buffer opening under `effects.source_edit_buffer`.
  `org.exit_src_edit` publishes parent-buffer writeback under
  `effects.parent_buffer_writeback`. `org.execute_babel_block` publishes Babel
  execution and rootfs runtime evidence under `effects.org_babel_execution`.
  `org.export_html` in preview mode validates the planned artifact path without
  writing it or publishing an effect; in apply mode it publishes the generated
  artifact under `effects.html_export`. `dired.rename_no_overwrite` and
  `dired.delete_marked` publish checked filesystem changes under
  `effects.filesystem_rename` and `effects.filesystem_delete`.
  `magit.stage_hunk` and `magit.stage_file` publish command-dispatch evidence
  with `this-command` and Git index changes under `effects.git_index_update`.
  `lsp.observe_source` publishes typed workspace, server, document,
  diagnostics/cache, LSP sync, rootfs runtime, source-buffer, and symbol
  sections without effects. `lsp.observe_diagnostics` publishes the same
  workspace/server/document/runtime evidence plus focused diagnostic
  observations without source-text mutation effects. `lsp.rename_symbol`
  publishes LSP request evidence, workspace-edit hash/excerpt/application
  evidence, saved-buffer and affected-file evidence, before/after observations,
  and the actual multi-buffer operation under `effects.workspace_edit`. Queued
  live-daemon LSP rename requests are stored as pending actions, and
  `request.await` can drive the target request to a terminal stored response
  instead of leaving successful external file edits represented as `running`.
- Response diagnostics are validated recursively and normalize non-string
  legacy errors to string diagnostic messages.
- The selected-rootfs batch bridge benchmark proves manifest introspection,
  core-v0 session observation, reports-v0 Org tree observation, checked heading
  update with stale structure rejection, checked source-edit buffer opening,
  Org source-edit writeback, stale source-edit parent rejection, checked
  `org.execute_babel_block` execution through `org-babel-execute-src-block`
  with parent result insertion evidence and stale source-block rejection, and
  HTML export,
  terminal-v0 Comint semantic submit/await with stale process-mark rejection,
  host-shadow process-executable rejection for submit/await/interrupt before
  side effects,
  terminal-v0 `comint.interrupt` with stale process-mark rejection,
  `comint-interrupt-subjob` dispatch evidence, process survival, and delayed
  side-effect reconciliation, plus typed top-level Comint evidence/effects for
  submit, await, and interrupt,
  coding-v0 `completion.observe`/`completion.accept` with exact provider range,
  candidates, accept-ready preconditions, and stale completion rejection,
  vcs-files-v0 Dired observation, no-overwrite rename, and marked-delete with
  stale same-path replacement rejection, vcs-files-v0 semantic Magit hunk/file
  staging with stale-section rejection, and the additive response envelope. It
  also proves request-envelope validation for
  `api_version`, `capability`, top-level `preconditions`, and `timeout_ms`,
  manifest-required param/precondition enforcement, contract metadata for
  `buffer.replace_range`, stable `buffer.apply_patch` with multi-hunk exact
  context and stale-state rejection, and the first stable-profile manifest gate:
  all first-profile stable operations are present, implemented, enabled, free of
  planned operations, and publish envelope contracts. The same gate includes a
  synthetic planned/disabled drift probe that fails as intended. It also proves
  recursive manifest-driven result-envelope validation over captured responses.
  The refreshed trace at
  `benchmarks/emacs-gym/bridge/traces/latest/summary.json` reports 29/29
  scenarios as expected in run `20261001T062920Z-2817516` and validates 111
  captured bridge responses. It includes typed top-level section checks for
  `emacs.observe_buffer`, `emacs.observe_session`, `project.observe`, and
  `project.search`, `buffer.read`, `org.observe_tree`, `dired.observe`,
  `comint.observe`, `magit.observe_sections`, reports-v0 Org
  update/source-edit/writeback/Babel/export effects, vcs-files-v0 Dired
  filesystem effects, and Magit command-dispatch/index-update effects, plus a
  synthetic LSP envelope-projection scenario proving
  typed `lsp.observe_source` evidence/observations and typed
  `lsp.rename_symbol` request/workspace-edit evidence/effects. It also includes
  checks that timeout diagnostics have
  string messages, legacy `:json-false` errors do not leak into
  `diagnostics[].message`, `project.search` rejects a shadowed executable
  `rg` outside the selected rootfs without invoking it for either search or
  version probing, `project.observe` does not execute a shadowed executable
  `git` outside the selected rootfs for VCS probing, LSP host-shadow coverage
  proving `lsp.observe_source`, `lsp.observe_diagnostics`, and
  `lsp.rename_symbol` reject a live server executable outside the selected
  rootfs before LSP sync, diagnostic, or rename work, Comint host-shadow
  coverage proving `comint.submit_input`, `comint.await_output`, and
  `comint.interrupt` reject a live process executable outside the selected
  rootfs before process input/wait/control work, `jupyter.await_request`
  preserves an `outcome_unknown` target state, and the stable core-v0
  `observe-session` scenario, the stable terminal-v0 `comint-semantic-api`
  and `comint-interrupt-api` scenarios, the stable coding-v0 `project-api`
  scenario with rootfs-owned ripgrep structured submatch evidence, the stable
  coding-v0 `completion-accept` and `stale-completion-rejected` scenarios, the
  stable reports-v0 `reports-vcs-files-api` Org update/source-edit/Babel/export
  path, the stable vcs-files-v0 Dired `dired.delete_marked` positive and stale
  replacement rejection path,
  and the stable vcs-files-v0 `magit-semantic-api` scenario.
- The selected-rootfs daemon benchmark proves the same manifest path over the
  live JSONL daemon, directly exercises `emacs.observe_session`,
  `buffer.select`, `comint.submit_input`, `comint.await_output`, `comint.interrupt`,
  `project.observe`, `project.search`, `completion.observe`, `completion.accept`,
  `lsp.observe_source`, `lsp.observe_diagnostics`, `lsp.rename_symbol`,
  `jupyter.attach_existing`, `jupyter.eval_region`, `jupyter.await_request`,
  `jupyter.observe_display`,
  `magit.observe_sections`, `magit.stage_hunk`, and `magit.stage_file` over the
  socket, directly exercises `org.update_heading`, `org.edit_src_block`,
  `org.exit_src_edit`, `org.execute_babel_block`, `org.export_html`,
  `dired.rename_no_overwrite`, and `dired.delete_marked` over the socket,
  verifies that these reports-v0 and vcs-files-v0 operations are published as
  stable enabled operations in the daemon manifest, verifies that
  `buffer.apply_patch` is published as a stable enabled coding-v0
  `checked-buffer-patch` operation, and proves the same request-envelope
  validation at the
  socket boundary, manifest-required param/precondition
  enforcement, plus the same contract metadata and recursive manifest-driven
  result-envelope validation over live JSONL responses. The refreshed trace at
  `benchmarks/emacs-gym/bridge/traces/latest-daemon/summary.json` reports 63/63
  scenarios as expected in run `20261001T131239Z-3232868` and validates 931
  captured daemon bridge responses, including typed `emacs.observe_buffer`,
  `emacs.observe_session`, `project.observe`, and `project.search` top-level
  sections, typed `buffer.read`, `comint.observe`, and
  `magit.observe_sections` top-level sections, session identity, terminal/API
  Comint coexistence, typed `comint.submit_input`,
  `comint.await_output`, and `comint.interrupt` evidence/effects over the
  JSONL path, structured project submatch evidence, Pyright diagnostics through
  typed `lsp.observe_diagnostics` protocol/cache evidence, typed
  `lsp.observe_source` workspace/server/document/symbol evidence, semantic
  `lsp.rename_symbol` request/workspace-edit evidence and multi-buffer
  `effects.workspace_edit`, semantic existing-kernel
  Jupyter request awaiting without resubmission, and daemon semantic Magit
  staging with stale section rejection. It also includes live daemon Org report
  update, source-edit writeback, stale source-edit parent rejection, wrong
  Babel-session rejection with no `effects.org_babel_execution`, Babel result
  insertion, `org.export_html` preview with no artifact/effect, `org.export_html`
  apply with `effects.html_export`, Dired no-overwrite rename, and checked
  marked-delete coverage, plus raw filesystem rename rejection as a non-stable
  `harness-debug` operation with no `filesystem_rename` effect. It also includes
  checked existing-kernel
  Jupyter source-buffer attachment with wrong-kernel rejection and no implicit
  kernel restart, selected-region Jupyter execution with typed rootfs-owned
  kernel-execution evidence, display/update-display IOPub capture in typed
  evidence, Jupyter display observation without mutation effects, and the daemon
  `completion.observe` stale-completion path, where terminal editing changes
  the observed context and `completion.accept` rejects the stale request as a
  conflict without inserting the proposed text. The daemon async Jupyter/Magit
  scenario also proves that `jupyter.eval_region` responses do not expose
  editor text effects merely because they name the executed source range.
  `daemon-org-source-edit-stale-parent-rejected` proves `org.exit_src_edit`
  rejects a parent Org document changed after source-edit opening through the
  failed `parent structure fingerprint matches precondition` check, emits no
  `effects.parent_buffer_writeback`, preserves the parent mutation, and leaves
  the edited source-buffer text outside the parent document.
  `daemon-org-duplicate-id-rejected` proves `org.update_heading` rejects an Org
  buffer where the target ID resolves to multiple headings through the failed
  `Org heading ID is unique in buffer` check, emits no
  `effects.org_heading_update`, and preserves the Org buffer text.
  `daemon-org-heading-text-bypass-rejected` proves a direct checked
  `buffer.replace_range` can make the visible Org heading line match the desired
  final text, but `episode.finish` still rejects the task when the required
  semantic `org.update_heading` request evidence is missing; the text edit
  publishes only `effects.changed_range`, not `effects.org_heading_update`.
  `daemon-org-subtree-text-bypass-rejected` proves a direct whole-buffer text
  replacement can reparent an Org child subtree so the final buffer text
  matches, but `episode.finish` still rejects the task when required semantic
  subtree-move evidence is absent; the edit publishes only
  `effects.changed_range`, not an Org structure effect.
- The same selected-rootfs daemon run includes the stable-only profile-level
  smoke scenario `The live daemon composes only stable profile operations across
  coding, terminal, analysis, reports, and VCS/file work`. It uses only
  operations published as stable in `core-v0`, `coding-v0`, `terminal-v0`,
  `analysis-v0`, `reports-v0`, and `vcs-files-v0`, including `buffer.select`,
  `buffer.read`, `buffer.replace_range`, `buffer.save`,
  `completion.observe`, `completion.accept`, `lsp.observe_source`,
  `lsp.rename_symbol`, `comint.submit_input`, `comint.await_output`,
  `jupyter.attach_existing`, `jupyter.eval_region`, `jupyter.await_request`,
  `org.update_heading`, `org.execute_babel_block`, `org.export_html`,
  `dired.rename_no_overwrite`, and `magit.stage_hunk`. The scenario verifies
  source/project inspection, checked buffer edit/save, checked completion
  acceptance, semantic LSP rename, Comint command execution, existing-kernel
  selected-region Jupyter execution, Org report update/Babel/HTML export,
  checked Dired rename, and Magit staging of only the intended hunk.
- A later selected-rootfs daemon evidence run at
  `benchmarks/emacs-gym/bridge/traces/jupyter-owner-green/summary.json`
  reports 40/40 scenarios as expected in run `20261001T065756Z-3020662`. Its
  `daemon-jupyter-kernel-ownership-rejected` scenario proves that
  `jupyter.observe_source` exposes host-shadow connection/kernel metadata as
  non-rootfs-owned, and that `jupyter.attach_existing`,
  `jupyter.eval_region`, `jupyter.eval_region_via_repl`, and the
  benchmark-only `jupyter.query_state` path reject that wrong-owner metadata
  before association, execution, or state query.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/elisp-eval-green/summary.json` reports
  41/41 scenarios as expected in run `20261001T071416Z-3115954`. Its
  `daemon-elisp-eval-rejected` scenario proves the daemon has an explicit
  non-stable, rejection-only `elisp.eval` manifest entry and rejects arbitrary
  Elisp evaluation with `unauthorized_operation`, `evaluated=false`, and no
  effects.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/lsp-semantic-bypass-green-2/summary.json`
  reports 43/43 scenarios as expected in run `20261001T073551Z-3254021`. Its
  `daemon-semantic-bypass-rejected` scenario first performs a checked
  `buffer.replace_range` that makes the final buffer text match, then submits
  `episode.finish` with a required semantic `lsp.rename_symbol` request id that
  was never produced. The finish response fails with
  `request_checks_passed=false` while `buffer_checks_passed=true`, proving that
  matching text alone cannot satisfy a task whose contract requires semantic
  operation evidence. The same run also includes
  `daemon-lsp-text-bypass-rejected`, which performs the direct substitution in
  a real LSP Python source buffer and requires missing semantic rename evidence
  at finish time.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/project-host-shadow-green-2/summary.json`
  reports 44/44 scenarios as expected in run `20261001T074832Z-3423510`. Its
  `daemon-project-host-tool-rejected` scenario proves live daemon
  `project.search` and `project.observe` evaluate project tool ownership in the
  requested buffer's process context. A host-shadow `rg` under
  `/workspace/soma/.scratch/emacs-gym/host-shadow-tools/...` is reported as
  `rootfs_owned=false`, is not invoked for version probing or search, and
  causes `project.search` to fail with
  `rootfs-owned ripgrep is not available`. A host-shadow `git` under the same
  host-projected prefix is reported as `rootfs_owned=false`, is not invoked for
  VCS probing, and leaves `project.observe` with `vcs.type=false`.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/raw-process-write-green-2/summary.json`
  reports 45/45 scenarios as expected in run `20261001T080010Z-3524215`. Its
  `daemon-raw-process-write-rejected` scenario proves raw subprocess writes are
  represented only by a non-stable `harness-debug`/`negative-test`
  `process.write` manifest entry with route `rejection-only`. A direct request
  to that operation fails with `unauthorized_operation`, reports
  `used_raw_process_write=false`, `sent_bytes=0`, and preserves the Comint
  side-effect counter unchanged. The same run's manifest scenario checks that
  `process.write` is not stable, and its recursive response-envelope validation
  accepts 736 captured daemon responses with no contract issues.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/jupyter-restart-green/summary.json`
  reports 46/46 scenarios as expected in run `20261001T080810Z-3614059`. Its
  `daemon-jupyter-restart-kernel-rejected` scenario proves direct kernel
  restart attempts are represented only by a non-stable
  `harness-debug`/`negative-test` `jupyter.restart_kernel` manifest entry with
  route `rejection-only`. A direct request fails with `unauthorized_operation`,
  reports `restart_attempted=false`, `shutdown_attempted=false`, and
  `launch_attempted=false`, emits no effects, and preserves the existing
  Jupyter kernel token, PID, executable, client session, and REPL association.
  The same run's recursive response-envelope validation accepts 742 captured
  daemon responses with no contract issues.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/git-index-write-green/summary.json`
  reports 47/47 scenarios as expected in run `20261001T081521Z-3743013`. Its
  `daemon-git-index-write-rejected` scenario proves raw Git index writes are
  represented only by a non-stable `harness-debug`/`negative-test`
  `git.index_write` manifest entry with route `rejection-only`. A direct request
  fails with `unauthorized_operation`, reports
  `used_raw_git_index_write=false` and `staged_path_count=0`, emits no effects,
  preserves the index/worktree state unchanged, and does not run Magit
  post-stage hooks. The same run's recursive response-envelope validation
  accepts 746 captured daemon responses with no contract issues.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/filesystem-rename-green-2/summary.json`
  reports 60/60 scenarios as expected in run `20261001T121142Z-2667734`. Its
  `daemon-filesystem-rename-rejected` scenario proves raw filesystem renames
  are represented only by a non-stable `harness-debug`/`negative-test`
  `filesystem.rename` manifest entry with route `rejection-only`. A direct
  request fails with `unauthorized_operation`, reports
  `used_raw_filesystem_rename=false` and `renamed_paths=0`, emits no
  `filesystem_rename` effect, preserves the source file, and creates no target.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/org-parent-text-write-green/summary.json`
  reports 48/48 scenarios as expected in run `20261001T082640Z-4034950`. Its
  `daemon-org-parent-text-write-rejected` scenario proves direct parent-buffer
  source-block writes are represented only by a non-stable
  `harness-debug`/`negative-test` `org.parent_text_write` manifest entry with
  route `rejection-only`. A direct request fails with `unauthorized_operation`,
  reports `direct_parent_text_write_attempted=false`,
  `source_edit_bypassed=false`, and `written_bytes=0`, emits no effects, and
  preserves the parent Org buffer text unchanged. The same run's recursive
  response-envelope validation accepts 759 captured daemon responses with no
  contract issues.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/org-buffer-replace-source-block-green/summary.json`
  reports 49/49 scenarios as expected in run `20261001T085545Z-271440`. Its
  `daemon-org-buffer-replace-source-block-rejected` scenario proves stable
  generic `buffer.replace_range` rejects direct replacement of an Org parent
  source-block range with `conflict`, records the failed package-owner
  precondition `target range does not overlap Org source block in parent
  buffer`, emits no `effects.changed_range`, and preserves the parent Org text.
  The same run proves queued daemon `lsp.rename_symbol` requests can be awaited
  to terminal success after the bridge records the pending action, and recursive
  response-envelope validation accepts 773 captured daemon responses with no
  contract issues.
- The selected-rootfs batch trace at
  `benchmarks/emacs-gym/bridge/traces/async-terminal-publication-green-final/summary.json`
  reports 30/30 scenarios as expected in run `20261001T093831Z-696416`. Its
  `async-terminal-publication-preserved` scenario proves a queued
  `lsp.rename_symbol` completion cannot be overwritten by the outer transport's
  initial `running` response: `request.await` observes the stored terminal
  state, route `lsp-rename`, and workspace-edit evidence.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/org-export-symlink-lsp-lifecycle-green-final/summary.json`
  reports 50/50 scenarios as expected in run `20261001T093849Z-699151`. It
  proves the final live JSONL bridge keeps semantic LSP and terminal-dispatch
  LSP lifecycle requests awaitable to terminal success, restores existing-kernel
  Jupyter await/status behavior after removing the broad JSONL reentrancy guard,
  and rejects `org.export_html` outputs whose symlink-resolved destination
  escapes the declared publish directory. The symlink-escape scenario fails with
  `error_code=conflict`, records the failed precondition `output path resolves
  inside publish directory`, emits no `effects.html_export`, and creates no
  requested or real outside HTML artifact.
- The selected-rootfs batch bridge trace at
  `benchmarks/emacs-gym/bridge/traces/benchmark-only-v0-green/summary.json`
  reports 30/30 scenarios as expected in run `20261001T095314Z-874310`. Its
  `request-envelope-validation` scenario proves explicit
  `soma-emacs-agent-v0` requests for benchmark-only `bridge.defer_echo` and
  `episode.finish` operations fail with `benchmark_only_operation`, while
  harness-internal async fixture calls remain available outside the stable-v0
  envelope for timeout/status tests.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/benchmark-only-v0-daemon-green/summary.json`
  reports 50/50 scenarios as expected in run `20261001T095343Z-880061` with
  Insula checks passing. Its `daemon-request-envelope-validation` scenario
  proves the same benchmark-only rejection over the live JSONL daemon:
  explicit stable-v0 `bridge.defer_echo` and `episode.finish` requests fail
  before execution with `benchmark_only_operation`.
- The selected-rootfs LSP trace at
  `benchmarks/emacs-gym/lsp/traces/soma-emacs-agent-v0-audit-green/summary.json`
  reports 6/6 scenarios as expected in run `20261001T100017Z-987449` with
  Insula checks passing. It refreshes the standalone Emacs plus language-source
  evidence for Python, C++, and Rust LSP sessions, and retains negative coverage
  for direct text-edit-as-refactor, stale document version, and stale workspace
  root rejection.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/stable-smoke-apply-patch-daemon-green/summary.json`
  reports 50/50 scenarios as expected in run `20261001T101147Z-1097333` with
  Insula checks passing. Its stable-profile smoke now exercises
  `buffer.apply_patch` through the live JSONL daemon and the stable-v0
  envelope; the response succeeds with `effects.changed_ranges`, and the saved
  source contains the patched return value. The existing-kernel Jupyter scenario
  records both generic `request.await` and semantic `jupyter.await_request`
  terminal success for the same original request. A manifest-vs-trace audit of
  the same run finds 40 stable enabled operations in the first six profiles and
  40 captured stable daemon response operations, with no missing stable
  operation.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/wrong-text-owner-daemon-green/summary.json`
  reports 51/51 scenarios as expected in run `20261001T102305Z-1198248` with
  Insula checks passing. Its `daemon-buffer-wrong-text-owner-rejected` scenario
  proves explicit stable-v0 `buffer.replace_range` requests reject a corrupted
  `text_owner_name` precondition with `conflict`, record the failed
  `text owner matches precondition` check, emit no `effects.changed_range`, and
  preserve the daemon buffer text unchanged.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/dired-collision-daemon-green/summary.json`
  reports 52/52 scenarios as expected in run `20261001T104120Z-1493347` with
  Insula checks passing. Its `daemon-dired-rename-collision-rejected` scenario
  proves stable-v0 `dired.rename_no_overwrite` rejects an existing target through
  the failed `target is absent for no-overwrite rename` precondition, emits no
  `effects.filesystem_rename`, and preserves both source and target file
  contents.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/duplicate-mutating-daemon-green/summary.json`
  reports 53/53 scenarios as expected in run `20261001T104748Z-1568405` with
  Insula checks passing. Its
  `daemon-duplicate-mutating-request-id-rejected` scenario proves the first
  `buffer.replace_range` request for a new `request_id` mutates once, a second
  conflicting mutating request with the same `request_id` fails with
  `duplicate_request_id`, stored status remains the first successful response,
  and the conflicting text is not inserted.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/dired-unsafe-target-daemon-green/summary.json`
  reports 54/54 scenarios as expected in run `20261001T105542Z-1691184` with
  Insula checks passing. Its `daemon-dired-rename-unsafe-target-rejected`
  scenario proves stable-v0 `dired.rename_no_overwrite` rejects a `../` target
  through the failed `source target names are safe relative basenames`
  precondition, emits no `effects.filesystem_rename`, preserves the source, and
  creates no escaped file.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/dired-stale-delete-daemon-green/summary.json`
  reports 55/55 scenarios as expected in run `20261001T110312Z-1823580` with
  Insula checks passing. Its
  `daemon-dired-delete-stale-signature-rejected` scenario proves stable-v0
  `dired.delete_marked` rejects an externally replaced marked file through the
  failed `file signature for marked file delete-a.txt matches` precondition,
  emits no `effects.filesystem_delete`, preserves the replacement contents, and
  only permits the later positive delete after fresh signatures are observed.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/dired-unsafe-delete-daemon-green/summary.json`
  reports 56/56 scenarios as expected in run `20261001T111156Z-2007938` with
  Insula checks passing. Its
  `daemon-dired-delete-unsafe-marked-rejected` scenario proves stable-v0
  `dired.delete_marked` rejects a `../` marked-file name through the failed
  `marked files are explicit safe relative basenames` precondition, emits no
  `effects.filesystem_delete`, and preserves the escaped file outside the Dired
  directory. The bridge also avoids collecting delete filesystem evidence for
  unsafe marked names before returning the failed precondition result.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/org-export-direct-escape-daemon-green/summary.json`
  reports 57/57 scenarios as expected in run `20261001T111825Z-2080142` with
  Insula checks passing. Its `daemon-org-export-direct-escape-rejected`
  scenario proves stable-v0 `org.export_html` rejects a direct `../` output path
  through the failed `output path is inside publish directory` precondition,
  emits no `effects.html_export`, and creates no requested or resolved outside
  HTML artifact.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/lsp-stale-document-daemon-green/summary.json`
  reports 58/58 scenarios as expected in run `20261001T113123Z-2258625` with
  Insula checks passing. Its
  `daemon-lsp-stale-document-version-rejected` scenario proves stable-v0
  `lsp.rename_symbol` rejects stale LSP document-version preconditions before
  applying a workspace edit, reports the failed `LSP document version matches
  precondition` check, emits no `effects.workspace_edit`, and preserves the
  rejected symbol name.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/org-stale-parent-daemon-green/summary.json`
  reports 59/59 scenarios as expected in run `20261001T115516Z-2474499` with
  Insula checks passing. Its
  `daemon-org-source-edit-stale-parent-rejected` scenario proves stable-v0
  `org.exit_src_edit` rejects a source-edit writeback after the parent Org
  buffer changed, records the failed `parent structure fingerprint matches
  precondition` check, emits no `effects.parent_buffer_writeback`, preserves the
  parent mutation, and keeps the edited source-buffer text out of the parent
  document.
- The canonical selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/latest-daemon/summary.json` now reports
  63/63 scenarios as expected in run `20261001T131239Z-3232868` with Insula
  checks passing. It includes the current no-host-tool-fallback and semantic
  bypass negatives, the wrong-text-owner, Dired collision, and duplicate
  mutating request-id rejections, the Dired unsafe-target and stale-delete
  rejections, Dired unsafe marked-delete rejection, stale LSP document-version
  rejection, stale Org source-edit parent rejection, duplicate Org ID heading
  update rejection, Org heading text-substitution bypass rejection, Org subtree
  text-reparenting bypass rejection, wrong Babel-session rejection, direct and
  symlinked Org HTML export escape rejections,
  `org.export_html` preview/apply behavior, raw filesystem rename rejection, and a
  refreshed `daemon-jupyter-display-update` scenario that reconciles a generic
  `request.await` timeout through semantic `jupyter.await_request` before
  judging the original kernel execution. A
  manifest-vs-trace audit of this run finds 40 stable enabled operations in the
  first six profiles and no missing stable operation among captured v0 daemon
  responses.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/frame-observe-stable-green/summary.json`
  reports 63/63 scenarios as expected in run `20261001T132346Z-3347347` with
  Insula checks passing. It promotes `emacs.observe_frames` into stable
  `core-v0`, proves the daemon manifest advertises it as `frame-read` with
  `frame_identity` and `terminal_client_identity` evidence, and exercises it
  through the stable v0 envelope inside the profile smoke scenario. The
  response publishes typed passive frame evidence and observations with no
  effects. The same run validates 933 captured daemon response envelopes and
  its manifest-vs-trace audit finds 41 stable enabled operations in the first
  six profiles, 41 captured stable daemon response operations, and no missing
  stable operation.
- The selected-rootfs batch and daemon traces at
  `benchmarks/emacs-gym/bridge/traces/request-respond-nonstable-batch-green-2/summary.json`
  and
  `benchmarks/emacs-gym/bridge/traces/request-respond-nonstable-v0-green-2/summary.json`
  report 30/30 and 63/63 scenarios as expected in runs
  `20261001T134453Z-3597764` and `20261001T133813Z-3522499`, with Insula
  checks passing. They prove `request.respond` remains implemented but
  experimental in the `core-v0` manifest, and explicit stable-v0
  `request.respond` requests fail before dispatch with
  `non_stable_operation`.
- The selected-rootfs daemon trace at
  `benchmarks/emacs-gym/bridge/traces/profile-smoke-jupyter-display-green-2/summary.json`
  reports 63/63 scenarios as expected in run `20261001T143144Z-4164242` with
  Insula checks passing. Its stable-profile smoke now covers all 41 stable
  operations in the first six profiles with no missing operation. The smoke
  path exercises `lsp.observe_diagnostics`, `comint.interrupt`,
  `org.edit_src_block`, `org.exit_src_edit`, `dired.delete_marked`,
  `magit.stage_file`, and `jupyter.observe_display` through the stable v0
  envelope. The Jupyter display check observes a profile-owned REPL display
  buffer after fixture setup created real emacs-jupyter `display_data` and
  `update_display_data` state; the profile trace itself uses only stable
  operations and asserts typed passive display evidence with no effects.
- `scripts/insula/validate-emacs-rootfs-contract.py
  .scratch/emacs-gym/insula/rootfs-store/content/soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
  passes with no errors or warnings for the selected rootfs used by that trace.
  The contract records base image
  `ghcr.io/pytorch/pytorch:2.13.0-cuda13.2-cudnn9-runtime@sha256:7492928e093d67276716440161f694a0e4ea27796d599d64b23cb76cdd665e71`,
  glibc owner `container-image-rootfs`, dynamic loader
  `/lib64/ld-linux-x86-64.so.2`, source-built GNU Emacs
  `/opt/soma/emacs/bin/emacs` version 30.2, no missing required tools, and
  rootfs tree hash
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`.
- The selected-rootfs batch bridge trace at
  `benchmarks/emacs-gym/bridge/traces/v0-final-batch-green/summary.json`
  reports 30/30 scenarios as expected in run `20261001T143938Z-44198` with
  Insula checks passing. This keeps the batch manifest, request-envelope,
  response-envelope, stable-profile manifest, typed evidence/effects, and
  negative authorization gates green after the daemon profile smoke reached
  41/41 stable operation coverage.
- The selected-rootfs batch bridge trace at
  `benchmarks/emacs-gym/bridge/traces/elisp-eval-batch-green/summary.json`
  reports 29/29 scenarios as expected in run `20261001T071607Z-3131775`. Its
  `direct-call-rejected` scenario now also proves `elisp.eval` rejection is
  stored in request status and reports no evaluation.
- Post-v0 maintenance: keep the no-host-tool-fallback and semantic-bypass
  suites green when new subprocess-owning operations are added.

## Post-v0 Questions

- Which debugger backend should be studied first for `debug-inspect-v0`:
  `dap-mode`, `gud`, or a narrower project-specific debugger path?
- Org HTML export is part of `reports-v0` for `soma-emacs-agent-v0`; a later
  `publish-v0` profile can add richer asset policies and multi-artifact
  publishing.
- How much multi-buffer edit atomicity is required before LSP workspace edits
  can be exposed beyond `lsp.rename_symbol`?
- Should `command.dispatch` be available to trained policies, or only to
  scripted reference policies and recovery teachers?
- What is the minimum event history needed for agents to recover from prompts,
  process output, kernel timeouts, and stale display without leaking private
  benchmark answers?
