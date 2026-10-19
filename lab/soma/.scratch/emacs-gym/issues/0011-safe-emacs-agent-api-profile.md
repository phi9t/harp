# 0011: Implement The Safe Emacs Agent API Profile

Status: Closed
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0002 Soma Insula Contract Spike; 0004 Checked Elisp Bridge

## Intent

Turn RFC 0003 from design into a stable model-facing API profile. The first
profile should let coding agents operate Emacs for coding, terminal, analysis,
and reporting tasks through typed semantic operations rather than CUA, vision,
arbitrary Elisp, or raw process shortcuts.

## Requirements

- Add a machine-readable operation manifest for the selected bridge.
- Separate stable API operations from experimental operations, benchmark-only
  fixtures, and harness/debug capabilities.
- Implement `api.list_capabilities` and `api.describe_operation` so an agent can
  inspect the installed safe subset.
- Use one request/result envelope with request id, API version, operation,
  capability, params, preconditions, timeout, state, state sequence, evidence,
  effects, diagnostics, and continuation fields.
- Require operation-specific preconditions for every mutation.
- Preserve the selected Insula rootfs identity in session observations and in
  every operation that invokes external tools, kernels, language servers,
  subprocesses, export tools, or build/test commands.
- Reject direct calls, arbitrary Elisp, raw `process-send-string`, implicit
  Jupyter kernel restart, host LSP/toolchain fallback, and "same final text"
  substitutions that bypass package-owned semantics.

## First Stable Profiles

Core:

- `api.list_capabilities`
- `api.describe_operation`
- `emacs.observe_session`
- `emacs.observe_buffer`
- `emacs.observe_frames`
- `buffer.select`
- `emacs.observe_interaction`
- `request.status`
- `request.await`

Coding:

- `project.observe`
- `project.search`
- `buffer.read`
- `buffer.replace_range`
- `buffer.apply_patch`
- `buffer.save`
- `completion.observe`
- `completion.accept`
- `lsp.observe_source`
- `lsp.observe_diagnostics`
- `lsp.rename_symbol`

Terminal:

- `comint.observe`
- `comint.submit_input`
- `comint.await_output`
- `comint.interrupt`

Analysis:

- `jupyter.observe_source`
- `jupyter.attach_existing`
- `jupyter.eval_region`
- `jupyter.await_request`
- `jupyter.observe_display`

Reports:

- `org.observe_tree`
- `org.update_heading`
- `org.edit_src_block`
- `org.exit_src_edit`
- `org.execute_babel_block`
- `org.export_html`

VCS/files:

- `magit.observe_sections`
- `magit.stage_hunk`
- `magit.stage_file`
- `dired.observe`
- `dired.rename_no_overwrite`
- `dired.delete_marked`

## Verification

- A profile-level smoke benchmark runs under the selected rootfs and uses only
  stable API operations.
- The smoke benchmark edits a source file, accepts a checked completion, runs an
  LSP semantic operation, submits and awaits a Comint command, runs selected
  Jupyter analysis in an existing kernel, updates an Org report, exports HTML,
  and stages intended changes.
- Negative cases reject stale buffer/text handles, stale LSP document versions,
  wrong Comint process marks, wrong Jupyter kernel/client identity, stale Org
  source-edit parents, stale Magit sections, filesystem collisions, duplicate
  request ids, and timeout retry after `outcome_unknown`.
- Aggregate selected-rootfs benchmarks remain green after the stable profile is
  introduced.

## Current Checkpoint

As of 2026-10-01, the shared bridge implements the first API introspection and
stable-profile surface:

- `api.list_capabilities`
- `api.describe_operation`

The operation manifest separates stable, experimental, planned,
benchmark-only, and harness/debug operations. The shared bridge and live daemon
allowlist now expose:

- `api.list_capabilities`
- `api.describe_operation`
- `emacs.observe_session`
- `emacs.observe_buffer`
- `emacs.observe_frames`
- `buffer.select`
- `project.observe`
- `project.search`
- `buffer.read`
- `buffer.replace_range`
- `buffer.apply_patch`
- `buffer.save`
- `completion.observe`
- `completion.accept`
- `lsp.observe_source`
- `lsp.observe_diagnostics`
- `lsp.rename_symbol`
- `jupyter.observe_source`
- `jupyter.attach_existing`
- `jupyter.eval_region`
- `jupyter.await_request`
- `jupyter.observe_display`
- `comint.observe`
- `comint.submit_input`
- `comint.await_output`
- `comint.interrupt`
- `org.observe_tree`
- `org.update_heading`
- `org.edit_src_block`
- `org.exit_src_edit`
- `org.execute_babel_block`
- `org.export_html`
- `magit.observe_sections`
- `magit.stage_hunk`
- `magit.stage_file`
- `dired.observe`
- `dired.rename_no_overwrite`
- `dired.delete_marked`

`emacs.observe_session` reports the v0 API/session identity, Emacs pid and
daemon state, source-built Emacs executable and source directory, selected
Insula run/plan/rootfs/closure identity, rootfs-owned Doom paths and commit,
loaded feature/module state when present, and selected frame/window/buffer
context.

`emacs.observe_frames` reports daemon-local frame identity, terminal/client
ownership, selected-window buffer state, and typed passive evidence without
mutation effects.

`project.observe` reports project root, Git identity, rootfs-owned `git`
evidence, rootfs-owned `rg` evidence, run id, and selected Insula plan path.
`project.search` searches bounded source text with rootfs-owned ripgrep and
returns bounded match and structured submatch evidence.

`buffer.select` selects an existing live buffer in the selected window,
optionally sets point, records before/after buffer observations, and does not
open files, invoke package commands, or mutate buffer text.

`completion.observe` records the first applicable CAPF provider, bounded
candidate strings, replacement range, preview state, and accept-ready
preconditions for checked completion acceptance. Those preconditions include
buffer/text owner, character revision, point, selected-window,
mark/narrowing/read-only/minibuffer state, expected range text, and expected
prefix/suffix.
`completion.accept` now accepts the `completion.observe` precondition shape and
rejects stale contexts before inserting the proposed text.

`comint.submit_input` submits through `comint-send-input`, records before/after
Comint ownership state, and rejects stale process id, stale process executable,
non-rootfs-owned process executable, stale process mark, wrong buffer name,
wrong buffer mode, or changed pending input before mutation.
`comint.await_output` waits from the checked process boundary, returns matched
output evidence on success, reports timeout as `outcome_unknown`, and rejects
wrong-owner process/buffer state, including non-rootfs-owned process
executables.
`comint.interrupt` checks the intended Comint buffer, process id, process mark,
process executable ownership, and optional pending input before dispatching
`comint-interrupt-subjob`. It records before/after Comint ownership state,
reports that no raw process write or bridge-level signal path was used, and
relies on benchmark graders to reconcile delayed external side effects.

`lsp.observe_source` flushes pending lsp-mode document changes, reads document
version, server identity, diagnostics, and diagnostic-cache/protocol evidence,
and reports passive symbol evidence without issuing repeated active symbol
requests during diagnostic polling. The daemon Pyright diagnostic fixture keeps
`lsp-diagnostics-provider` at `none`, so the API validates protocol/cache
diagnostics without requiring Flymake/Flycheck UI presentation.

`lsp.observe_diagnostics` is implemented as the stable diagnostics-only API. It
uses the same lsp-mode sync/cache path, records workspace, server, document,
position-encoding, diagnostics, diagnostic-cache, and precondition-check
evidence, and leaves source text acquisition to explicit `buffer.read` calls.

`lsp.rename_symbol` runs through the real LSP rename command path, records LSP
request/workspace-edit evidence, validates document/version context, and
verifies affected source/test files plus rootfs-owned test execution.

`jupyter.observe_source` publishes the observed Jupyter connection file,
kernel executable, and explicit `jupyter_connection_file_rootfs_owned` and
`jupyter_kernel_exe_rootfs_owned` booleans.

`jupyter.attach_existing` associates a source buffer with an intended existing
REPL/client through `jupyter-repl-associate-buffer`, rejects wrong-kernel and
wrong-owner preconditions, and never launches or restarts a kernel implicitly.
`jupyter.eval_region` and `jupyter.eval_region_via_repl` revalidate the
observed connection file and kernel executable ownership before submission, so
source-buffer metadata that points outside the selected rootfs fails before
kernel execution. The benchmark-only `jupyter.query_state` path performs the
same live ownership preflight before issuing a kernel state query.

`jupyter.await_request` awaits an existing bridge-owned Jupyter request by
request id, returns the original request status and reply/IOPub/idle evidence
through the `jupyter-request-await` route, and does not resubmit non-idempotent
code after submission.

`org.update_heading` resolves the observed Org node, preferring its `ID`
property, checks the buffer/text owner, character revision,
point/region/narrowing/read-only/minibuffer state, structure fingerprint, and
live heading TODO/title identity, then mutates through Org's headline/TODO
APIs. It rejects stale structure preconditions before editing and records
before/after Org tree evidence. Selected-window ownership is intentionally not
part of this semantic parent-buffer contract.

`org.edit_src_block` and `org.exit_src_edit` use Org's source-edit workflow
rather than direct parent-buffer text replacement. `org.edit_src_block` checks
parent buffer identity, text owner, file identity, character revision,
point/region/narrowing/read-only/minibuffer state, Org structure fingerprint,
live heading identity, and source-block hash before calling
`org-edit-src-code`; it returns the Org-created edit buffer and parent marker
evidence. `org.exit_src_edit` checks the edit-buffer revision/text hash, parent
marker range, parent buffer revision, parent structure fingerprint, and live
source-block identity before calling `org-edit-src-exit`. Stale parent drift
fails with `conflict` before writeback. The parent-buffer checks do not require
the daemon's currently selected window to display the Org buffer.

`org.execute_babel_block` executes an observed source block through
`org-babel-execute-src-block`, not by inserting results directly. The first
stable slice supports `emacs-lisp` blocks when the request provides a matching
source-block identity, parent Org node, structure fingerprint, buffer/text
owner, point/region/narrowing/read-only/minibuffer state, explicit
`language_allowlist`, expected session, and expected `default_directory`. It
records the loaded Babel backend, executor, runtime identity, returned value,
and parent `#+RESULTS:` block evidence. A stale parent buffer or stale
source-block observation fails with `conflict` before execution.

`magit.observe_sections` reports Magit section identity, selected section
state, repo fingerprint, and command-context evidence.
`magit.stage_hunk` and `magit.stage_file` resolve live sections in the target
Magit buffer, stage through Magit's package-owned command context with
`this-command` set to `magit-stage`, refresh/re-observe the status view, verify
exact index/worktree effects, and reject stale repo fingerprints or stale
section identities.

The shared response constructor now emits additive `soma-emacs-agent-v0`
envelope fields:

- `api_version`
- `state`
- `state_sequence`
- `result`
- `evidence`
- `observations`
- `effects`
- `diagnostics`
- `continuation`

The first typed-envelope migration slices are implemented for stable
core/project observations, checked buffer edits, coding-v0 LSP operations,
stable passive read/observe operations, terminal-v0 Comint operations, and
analysis-v0 Jupyter operations, plus reports-v0 and vcs-files-v0 mutating
operations:

- `emacs.observe_buffer` publishes typed `buffer_identity`, `text_owner`, and
  `point_region_narrowing` evidence, a focused `observations.buffer_state`
  object, and no effects.
- `emacs.observe_session` publishes typed `editor_session`, `rootfs_identity`,
  and `doom_identity` evidence, `observations.selected_frame`, and no effects.
- `project.observe` publishes typed project-root, VCS, rootfs-owned search-tool,
  and rootfs runtime evidence, plus `observations.project_context`.
- `project.search` publishes typed rootfs-owned search-tool, bounded-result,
  and runtime evidence, plus `observations.matches`.
- `buffer.read` publishes typed `buffer_identity` and `text_range` evidence,
  `observations.text_snapshot`, and no effects.
- `org.observe_tree` publishes typed buffer identity, structure fingerprint,
  heading index, source-block index, `observations.org_tree`, and no effects.
- `dired.observe` publishes typed directory, selection-state, file-signature,
  `observations.dired_listing`, and no effects.
- `comint.observe` publishes typed process identity, input-boundary,
  rootfs-runtime evidence, `observations.comint_state`, and no effects.
- `magit.observe_sections` publishes typed repository fingerprint, status
  generation, selected-section evidence, `observations.sections`, and no
  effects.
- `buffer.replace_range` publishes `precondition_checks` and
  `before_after_observations` under top-level `evidence`, before/after buffer
  snapshots under `observations`, and the actual text mutation under
  `effects.changed_range`.
- `buffer.apply_patch` publishes `precondition_checks`, `conflict_preview`, and
  `before_after_observations` under top-level `evidence`, before/after buffer
  snapshots under `observations`, and applied hunks under
  `effects.changed_ranges`.
- `comint.submit_input` publishes submitted byte count, Comint input-marker
  state, and process output under top-level `evidence`, with the actual process
  input under `effects.process_input`.
- `comint.await_output` publishes matched and partial output plus timeout
  evidence without claiming process mutation effects.
- `comint.interrupt` publishes interrupt command, process identity/survival,
  and side-effect reconciliation under top-level `evidence`, with the
  process-control action under `effects.process_interrupt`.
- `jupyter.observe_source` publishes source buffer, associated client, kernel
  incarnation, selected region, observed connection file, observed kernel
  executable, connection/executable rootfs ownership, and rootfs runtime
  evidence without effects.
- `jupyter.attach_existing` publishes association and kernel identity evidence,
  plus `effects.source_client_association` on successful
  `jupyter-repl-associate-buffer` association.
- `jupyter.eval_region` and `jupyter.eval_region_via_repl` publish request id,
  submitted hash, source region, reply, IOPub stream/display/update messages,
  idle status, and rootfs runtime evidence. Their consequential work is
  reported as `effects.kernel_execution`, not editor text mutation.
- `jupyter.await_request` publishes target status, reply, IOPub, idle, and
  timeout evidence without resubmitting non-idempotent kernel work.
- `jupyter.observe_display` publishes display id, display/update data, and REPL
  range evidence without effects.
- The typed projection is operation-scoped: source spans such as
  `jupyter.eval_region` `start`/`end` are not reported as editor text effects.
- `org.update_heading` publishes the semantic heading mutation under
  `effects.org_heading_update`.
- `org.edit_src_block` publishes the separate source-edit buffer opening under
  `effects.source_edit_buffer`.
- `org.exit_src_edit` publishes parent-buffer writeback under
  `effects.parent_buffer_writeback`.
- `org.execute_babel_block` publishes Babel execution and rootfs runtime
  evidence under `effects.org_babel_execution`.
- `org.export_html` in preview mode validates the planned artifact path without
  writing it or publishing an effect; in apply mode it publishes the generated
  artifact under `effects.html_export`.
- `dired.rename_no_overwrite` and `dired.delete_marked` publish checked
  filesystem changes under `effects.filesystem_rename` and
  `effects.filesystem_delete`.
- `magit.stage_hunk` and `magit.stage_file` publish command-dispatch evidence
  with `this-command` and Git index changes under
  `effects.git_index_update`.
- `lsp.observe_source` publishes typed workspace, server, document,
  diagnostics/cache, LSP sync, rootfs runtime, source-buffer, and symbol
  sections without effects.
- `lsp.observe_diagnostics` publishes typed workspace, server, document,
  diagnostics/cache, LSP sync, rootfs runtime, source-buffer, and diagnostics
  observations without effects.
- `lsp.rename_symbol` publishes LSP request evidence, workspace-edit
  hash/excerpt/application evidence, saved-buffer and affected-file evidence,
  before/after observations, and the actual multi-buffer operation under
  `effects.workspace_edit`.
- Diagnostics normalize non-string legacy errors to string messages, and
  `jupyter.await_request` preserves a target request's `outcome_unknown` state
  instead of reporting top-level success while embedding unknown target status.

Operation descriptions now include machine-readable `request_contract`,
`precondition_contract`, and `result_contract` objects. These derive from the
operation manifest and describe the v0 envelope, capability, required params,
typed preconditions, possible states, required result fields, and evidence
channels.

Explicit `soma-emacs-agent-v0` requests are checked against manifest-required
params and preconditions before dispatch. Missing params fail with
`missing_required_param`; missing preconditions fail with
`missing_required_precondition`; operations whose manifest maturity or profile
is `benchmark-only` fail with `benchmark_only_operation` when invoked through
the explicit stable-v0 envelope. Experimental and planned manifest entries fail
with `non_stable_operation`, so `request.respond` stays experimental until real
minibuffer/transient/package prompt-owner semantics are proven under the
selected rootfs. Rejection-only negative-test operations may still run through
the envelope to prove that raw process, filesystem, Git, kernel, and
parent-buffer shortcuts do not perform side effects.

Current selected-rootfs evidence:

- `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/latest/summary.json` with 29/29 scenarios
  as expected in run `20261001T062920Z-2817516`, including manifest
  introspection, core-v0 session observation, coding-v0 project observation and
  source search through rootfs-owned ripgrep with structured submatch evidence,
  typed top-level `buffer.read` range/snapshot evidence,
  coding-v0 stable `buffer.apply_patch` with multi-hunk exact-context
  application and stale-state rejection,
  coding-v0 checked completion observation/acceptance with provider range,
  candidates, accept-ready preconditions, and stale completion rejection,
  terminal-v0 Comint semantic submit/await with stale process-mark rejection,
  terminal-v0 host-shadow Comint process-executable rejection for submit,
  await, and interrupt before side effects,
  terminal-v0 `comint.interrupt` with stale process-mark rejection,
  `comint-interrupt-subjob` dispatch evidence, process survival, and delayed
  side-effect reconciliation, plus typed top-level Comint evidence/effects for
  observe, submit, await, and interrupt,
  reports-v0 Org tree observation, checked heading update with stale structure
  rejection, checked source-edit buffer opening, Org source-edit writeback,
  stale source-edit parent rejection, checked `org.execute_babel_block`
  execution through `org-babel-execute-src-block` with parent result insertion
  evidence and stale source-block rejection, and HTML export, vcs-files-v0
  Dired observation, no-overwrite rename, and checked marked-delete through
  `dired-do-delete` with stale same-path replacement rejection, vcs-files-v0
  semantic Magit
  hunk/file staging with stale-section
  rejection, response-envelope checks, and
  request-envelope validation for `api_version`, `capability`, top-level
  `preconditions`, `timeout_ms`, manifest-required param/precondition
  enforcement, contract metadata for `buffer.replace_range`, the first
  stable-profile manifest gate proving the six v0 profiles have all expected
  stable operations implemented, enabled, free of `planned` operations, and
  covered by envelope contracts, a synthetic drift probe proving that gate
  detects planned/disabled stable operations, typed evidence/effects migration
  for `emacs.observe_buffer`, `emacs.observe_session`, `project.observe`,
  `project.search`, `buffer.read`, `org.observe_tree`, `dired.observe`,
  `comint.observe`, `magit.observe_sections`, checked buffer edits,
  coding-v0 LSP operations, terminal-v0 Comint operations, reports-v0
  Org update/source-edit/writeback/Babel/export effects, vcs-files-v0 Dired
  filesystem effects, and Magit command-dispatch/index-update effects, a synthetic LSP
  envelope-projection scenario
  proving typed
  `lsp.observe_source` evidence/observations and typed `lsp.rename_symbol`
  request/workspace-edit evidence/effects, timeout diagnostics with string
  messages, host-shadow negative coverage proving `project.search` rejects a
  shadowed executable `rg` outside the selected rootfs without invoking it for
  search or version probing, host-shadow negative coverage proving
  `project.observe` does not execute a shadowed executable `git` outside the
  selected rootfs for VCS probing, host-shadow negative coverage proving
  `lsp.observe_source`, `lsp.observe_diagnostics`, and `lsp.rename_symbol`
  reject a live LSP server executable outside the selected rootfs before LSP
  sync, diagnostic, or rename work,
  host-shadow negative coverage proving `comint.submit_input`,
  `comint.await_output`, and `comint.interrupt` reject a live Comint process
  executable outside the selected rootfs before process input/wait/control work,
  `jupyter.await_request` preservation of `outcome_unknown` target state, and
  recursive manifest-driven result-envelope validation over 111 captured bridge
  responses.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/latest-daemon/summary.json` with 63/63
  scenarios as expected in run `20261001T131239Z-3232868`, including live JSONL
  daemon manifest introspection, direct `emacs.observe_session`,
  `buffer.select`,
  `comint.submit_input`, `comint.await_output`, `comint.interrupt`,
  `project.observe`,
  `project.search`, `completion.observe`, `completion.accept`,
  `lsp.observe_source`, `lsp.observe_diagnostics`, `lsp.rename_symbol`,
  `jupyter.attach_existing`, `jupyter.eval_region`, `jupyter.await_request`,
  `jupyter.observe_display`,
  `magit.observe_sections`, `magit.stage_hunk`, and `magit.stage_file` requests
  over the socket, live daemon `org.update_heading`, `org.edit_src_block`,
  `org.exit_src_edit`, `org.execute_babel_block`, `org.export_html`,
  `dired.rename_no_overwrite`, and `dired.delete_marked` requests over the
  socket, stable enabled reports-v0 and vcs-files-v0 exposure in the daemon
  manifest, stable enabled `buffer.apply_patch` exposure as a
  `checked-buffer-patch` coding-v0 operation, stable enabled
  `dired.delete_marked` exposure as a `dired-do-delete` vcs-files-v0 operation,
  typed top-level sections for `emacs.observe_buffer`, `emacs.observe_session`,
  `project.observe`, `project.search`, `buffer.read`, `comint.observe`, and
  `magit.observe_sections`, and session identity,
  terminal/API Comint coexistence, structured submatch evidence, Pyright
  diagnostics through typed `lsp.observe_diagnostics` protocol/cache evidence,
  typed `lsp.observe_source` workspace/server/document/symbol evidence, semantic
  `lsp.rename_symbol` request/workspace-edit evidence and multi-buffer
  `effects.workspace_edit`,
  checked existing-kernel Jupyter source-buffer attachment with wrong-kernel
  rejection and no implicit kernel restart,
  semantic Jupyter request awaiting through the stable `jupyter-request-await`
  route without resubmission,
  typed top-level Comint submit, await, and interrupt evidence/effects over the
  JSONL path, daemon checked Comint interrupt with stale-owner rejection and
  delayed side-effect reconciliation,
  daemon checked completion stale-context rejection after terminal editing,
  daemon semantic Magit staging with stale-section rejection,
  socket-level request-envelope validation, manifest-required
  param/precondition enforcement, plus the same contract metadata and
  recursive manifest-driven result-envelope validation over 931 captured
  daemon bridge responses.
  The live reports/vcs-files scenario proves daemon Org report update,
  source-edit writeback, stale source-edit parent rejection, wrong
  Babel-session rejection with no `effects.org_babel_execution`, Babel result
  insertion, `org.export_html` preview with no artifact/effect,
  `org.export_html` apply with `effects.html_export`, Dired no-overwrite rename,
  checked marked-delete coverage, and raw filesystem rename rejection with no
  `filesystem_rename` effect. Org semantic
  parent-buffer operations no longer require the daemon's selected window to
  display the Org buffer.
  The `daemon-org-duplicate-id-rejected` scenario proves `org.update_heading`
  rejects duplicate Org ID ownership through the failed `Org heading ID is
  unique in buffer` precondition, publishes no heading-update effect, and
  preserves the Org buffer text.
  The `daemon-org-heading-text-bypass-rejected` scenario proves a generic
  checked buffer edit can make the visible Org heading line match the desired
  final text, but the task still fails when `episode.finish` requires missing
  semantic `org.update_heading` request evidence; the edit publishes only
  `changed_range`, not a semantic heading effect.
  The `daemon-org-subtree-text-bypass-rejected` scenario proves a direct
  whole-buffer text replacement can reparent an Org child subtree and pass
  final text checks, but the task still fails when `episode.finish` requires
  missing semantic subtree-move evidence; the edit publishes only
  `changed_range`, not an Org structure effect.
  The daemon async Jupyter/Magit scenario now also proves that
  `jupyter.eval_region` responses do not report editor text effects merely
  because the executed source region has `start`/`end` coordinates.
  The daemon Jupyter display-update scenario now proves typed
  `jupyter.eval_region_via_repl` evidence carries separate `display_data` and
  `update_display_data` IOPub events, parent request ids, reply status, stream
  payload, and a single `effects.kernel_execution` record. The checked
  existing-kernel attach scenario proves typed `jupyter.attach_existing`
  evidence/effects while retaining wrong-kernel rejection and no implicit
  restart behavior.
- The daemon run also includes the stable-only profile-level smoke scenario
  `The live daemon composes only stable profile operations across coding,
  terminal, analysis, reports, and VCS/file work`. It uses only stable
  operations from `core-v0`, `coding-v0`, `terminal-v0`, `analysis-v0`,
  `reports-v0`, and `vcs-files-v0`, including `buffer.select`, `buffer.read`,
  `buffer.replace_range`, `buffer.save`, `completion.observe`,
  `completion.accept`, `lsp.observe_source`, `lsp.rename_symbol`,
  `comint.submit_input`, `comint.await_output`, `jupyter.attach_existing`,
  `jupyter.eval_region`, `jupyter.await_request`, `org.update_heading`,
  `org.execute_babel_block`, `org.export_html`, `dired.rename_no_overwrite`,
  and `magit.stage_hunk`. The scenario verifies project/source inspection,
  checked buffer edit/save, checked completion acceptance, semantic LSP rename,
  Comint command execution, existing-kernel selected-region Jupyter execution,
  Org report update/Babel/HTML export, checked Dired rename, and Magit staging
  of only the intended hunk.
- A later selected-rootfs daemon run refreshed
  `benchmarks/emacs-gym/bridge/traces/jupyter-owner-green/summary.json` with
  40/40 scenarios as expected in run `20261001T065756Z-3020662`. Its
  `daemon-jupyter-kernel-ownership-rejected` scenario proves the host-shadow
  Jupyter fixture is present, `jupyter.observe_source` exposes wrong-owner
  connection/kernel metadata through rootfs-owned booleans, and
  `jupyter.attach_existing`, `jupyter.eval_region`,
  `jupyter.eval_region_via_repl`, and the benchmark-only `jupyter.query_state`
  path reject host-shadow ownership before association, execution, or state
  query. The same run records the valid real kernel executable as
  `/usr/bin/python3.12`; the rejected shadow paths are under
  `/workspace/soma/.scratch/emacs-gym/host-shadow-tools/...`.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/elisp-eval-green/summary.json` with 41/41
  scenarios as expected in run `20261001T071416Z-3115954`. Its
  `daemon-elisp-eval-rejected` scenario proves arbitrary Elisp evaluation is
  represented only by a non-stable `harness-debug`/`negative-test`
  `elisp.eval` manifest entry, and that the live daemon rejects such requests
  with `unauthorized_operation`, `evaluated=false`, and no effects.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/lsp-semantic-bypass-green-2/summary.json`
  with 43/43 scenarios as expected in run `20261001T073551Z-3254021`. Its
  `daemon-semantic-bypass-rejected` scenario proves a checked
  `buffer.replace_range` can make the final buffer text match, but
  `episode.finish` still fails when the required semantic LSP request id is
  absent: `request_checks_passed=false` and `buffer_checks_passed=true`. The
  same run adds `daemon-lsp-text-bypass-rejected`, which performs that bypass
  against a real LSP Python source buffer and requires missing semantic rename
  evidence at finish time.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/project-host-shadow-green-2/summary.json`
  with 44/44 scenarios as expected in run `20261001T074832Z-3423510`. Its
  `daemon-project-host-tool-rejected` scenario proves live daemon
  `project.search` and `project.observe` evaluate tool ownership in the
  requested buffer's process context. A host-shadow `rg` under
  `/workspace/soma/.scratch/emacs-gym/host-shadow-tools/...` is reported as
  `rootfs_owned=false`, is not invoked for version probing or search, and
  causes `project.search` to fail with
  `rootfs-owned ripgrep is not available`. A host-shadow `git` under the same
  host-projected prefix is reported as `rootfs_owned=false`, is not invoked for
  VCS probing, and leaves `project.observe` with `vcs.type=false`.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/raw-process-write-green-2/summary.json`
  with 45/45 scenarios as expected in run `20261001T080010Z-3524215`. Its
  `daemon-raw-process-write-rejected` scenario proves raw subprocess writes are
  represented only by a non-stable `harness-debug`/`negative-test`
  `process.write` manifest entry with route `rejection-only`. A direct request
  to that operation fails with `unauthorized_operation`, reports
  `used_raw_process_write=false`, `sent_bytes=0`, and leaves the Comint
  side-effect counter unchanged. The same run proves the daemon manifest keeps
  `process.write` out of the stable API, and recursive response-envelope
  validation accepts 736 captured daemon responses with no contract issues.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/jupyter-restart-green/summary.json` with
  46/46 scenarios as expected in run `20261001T080810Z-3614059`. Its
  `daemon-jupyter-restart-kernel-rejected` scenario proves direct kernel restart
  attempts are represented only by a non-stable
  `harness-debug`/`negative-test` `jupyter.restart_kernel` manifest entry with
  route `rejection-only`. A direct request fails with `unauthorized_operation`,
  reports `restart_attempted=false`, `shutdown_attempted=false`, and
  `launch_attempted=false`, emits no effects, and preserves the existing
  Jupyter kernel token, PID, executable, client session, and REPL association.
  The same run proves the daemon manifest keeps `jupyter.restart_kernel` out of
  the stable API, and recursive response-envelope validation accepts 742
  captured daemon responses with no contract issues.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/git-index-write-green/summary.json` with
  47/47 scenarios as expected in run `20261001T081521Z-3743013`. Its
  `daemon-git-index-write-rejected` scenario proves raw Git index writes are
  represented only by a non-stable `harness-debug`/`negative-test`
  `git.index_write` manifest entry with route `rejection-only`. A direct request
  fails with `unauthorized_operation`, reports
  `used_raw_git_index_write=false` and `staged_path_count=0`, emits no effects,
  preserves the index/worktree state unchanged, and does not run Magit
  post-stage hooks. The same run proves the daemon manifest keeps
  `git.index_write` out of the stable API, and recursive response-envelope
  validation accepts 746 captured daemon responses with no contract issues.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/filesystem-rename-green-2/summary.json`
  with 60/60 scenarios as expected in run `20261001T121142Z-2667734`. Its
  `daemon-filesystem-rename-rejected` scenario proves raw filesystem renames are
  represented only by a non-stable `harness-debug`/`negative-test`
  `filesystem.rename` manifest entry with route `rejection-only`. A direct
  request fails with `unauthorized_operation`, reports
  `used_raw_filesystem_rename=false` and `renamed_paths=0`, emits no
  `filesystem_rename` effect, preserves the source file, and creates no target.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/org-parent-text-write-green/summary.json`
  with 48/48 scenarios as expected in run `20261001T082640Z-4034950`. Its
  `daemon-org-parent-text-write-rejected` scenario proves direct parent-buffer
  Org source-block writes are represented only by a non-stable
  `harness-debug`/`negative-test` `org.parent_text_write` manifest entry with
  route `rejection-only`. A direct request fails with `unauthorized_operation`,
  reports `direct_parent_text_write_attempted=false`,
  `source_edit_bypassed=false`, and `written_bytes=0`, emits no effects, and
  preserves the parent Org buffer text unchanged. The same run proves the daemon
  manifest keeps `org.parent_text_write` out of the stable API, and recursive
  response-envelope validation accepts 759 captured daemon responses with no
  contract issues.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/org-buffer-replace-source-block-green/summary.json`
  with 49/49 scenarios as expected in run `20261001T085545Z-271440`. Its
  `daemon-org-buffer-replace-source-block-rejected` scenario proves stable
  generic `buffer.replace_range` rejects direct replacement of an Org parent
  source-block range with `conflict`, records the failed package-owner
  precondition `target range does not overlap Org source block in parent
  buffer`, emits no `effects.changed_range`, and preserves the parent Org text.
  The same run proves queued daemon `lsp.rename_symbol` requests can be awaited
  to terminal success after the bridge records the pending action, and recursive
  response-envelope validation accepts 773 captured daemon responses with no
  contract issues.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark-in-rootfs.sh` now
  bounds the graceful `emacsclient --eval "(kill-emacs 0)"` cleanup request and
  falls back to daemon termination if the cleanup client blocks. The
  `lsp-semantic-bypass-green-2` run exited cleanly after this change.
- `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/elisp-eval-batch-green/summary.json`
  with 29/29 scenarios as expected in run `20261001T071607Z-3131775`. Its
  `direct-call-rejected` scenario now also proves `elisp.eval` rejection is
  stored in request status and reports no evaluation.
- `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/async-terminal-publication-green-final/summary.json`
  with 30/30 scenarios as expected in run `20261001T093831Z-696416`. Its
  `async-terminal-publication-preserved` scenario proves a queued
  `lsp.rename_symbol` completion cannot be overwritten by the outer transport's
  initial `running` response: `request.await` observes the stored terminal
  state, route `lsp-rename`, and workspace-edit evidence.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/org-export-symlink-lsp-lifecycle-green-final/summary.json`
  with 50/50 scenarios as expected in run `20261001T093849Z-699151`. It proves
  the final live JSONL bridge keeps semantic LSP and terminal-dispatch LSP
  lifecycle requests awaitable to terminal success, restores existing-kernel
  Jupyter await/status behavior after removing the broad JSONL reentrancy guard,
  and rejects `org.export_html` outputs whose symlink-resolved destination
  escapes the declared publish directory. The symlink-escape scenario fails with
  `error_code=conflict`, records the failed precondition `output path resolves
  inside publish directory`, emits no `effects.html_export`, and creates no
  requested or real outside HTML artifact.
- `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/benchmark-only-v0-green/summary.json`
  with 30/30 scenarios as expected in run `20261001T095314Z-874310`. Its
  `request-envelope-validation` scenario proves explicit
  `soma-emacs-agent-v0` requests for benchmark-only `bridge.defer_echo` and
  `episode.finish` operations fail with `benchmark_only_operation`, while
  harness-internal async fixture calls remain available outside the stable-v0
  envelope for timeout/status tests.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/benchmark-only-v0-daemon-green/summary.json`
  with 50/50 scenarios as expected in run `20261001T095343Z-880061` with Insula
  checks passing. Its `daemon-request-envelope-validation` scenario proves the
  same benchmark-only rejection over the live JSONL daemon: explicit stable-v0
  `bridge.defer_echo` and `episode.finish` requests fail before execution with
  `benchmark_only_operation`.
- `benchmarks/emacs-gym/lsp/scripts/run-benchmark.sh` refreshed
  `benchmarks/emacs-gym/lsp/traces/soma-emacs-agent-v0-audit-green/summary.json`
  with 6/6 scenarios as expected in run `20261001T100017Z-987449` with Insula
  checks passing. It refreshes the standalone Emacs plus language-source
  evidence for Python, C++, and Rust LSP sessions, and retains negative coverage
  for direct text-edit-as-refactor, stale document version, and stale workspace
  root rejection.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/stable-smoke-apply-patch-daemon-green/summary.json`
  with 50/50 scenarios as expected in run `20261001T101147Z-1097333` with
  Insula checks passing. Its stable-profile smoke now exercises
  `buffer.apply_patch` through the live JSONL daemon and the stable-v0
  envelope; the response succeeds with `effects.changed_ranges`, and the saved
  source contains the patched return value. The existing-kernel Jupyter scenario
  records both generic `request.await` and semantic `jupyter.await_request`
  terminal success for the same original request. A manifest-vs-trace audit of
  the same run finds 40 stable enabled operations in the first six profiles and
  40 captured stable daemon response operations, with no missing stable
  operation.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed
  `benchmarks/emacs-gym/bridge/traces/wrong-text-owner-daemon-green/summary.json`
  with 51/51 scenarios as expected in run `20261001T102305Z-1198248` with
  Insula checks passing. Its `daemon-buffer-wrong-text-owner-rejected` scenario
  proves explicit stable-v0 `buffer.replace_range` requests reject corrupted
  `text_owner_name` preconditions with `conflict`, record the failed
  `text owner matches precondition` check, publish no text mutation effect, and
  preserve the daemon buffer text unchanged.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/dired-collision-daemon-green/summary.json`
  with 52/52 scenarios as expected in run `20261001T104120Z-1493347` with
  Insula checks passing. Its `daemon-dired-rename-collision-rejected` scenario
  proves stable-v0 `dired.rename_no_overwrite` rejects an existing target through
  the failed `target is absent for no-overwrite rename` precondition, publishes
  no filesystem rename effect, and preserves both source and target files.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/duplicate-mutating-daemon-green/summary.json`
  with 53/53 scenarios as expected in run `20261001T104748Z-1568405` with
  Insula checks passing. Its
  `daemon-duplicate-mutating-request-id-rejected` scenario proves a stable-v0
  `buffer.replace_range` request mutates once, a second conflicting mutating
  request with the same `request_id` fails with `duplicate_request_id`, the
  stored status remains the first successful response, and the conflicting text
  is not inserted.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/dired-unsafe-target-daemon-green/summary.json`
  with 54/54 scenarios as expected in run `20261001T105542Z-1691184` with
  Insula checks passing. Its `daemon-dired-rename-unsafe-target-rejected`
  scenario proves stable-v0 `dired.rename_no_overwrite` rejects a `../` target
  through the failed `source target names are safe relative basenames`
  precondition, publishes no filesystem rename effect, preserves the source,
  and creates no escaped file.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/dired-stale-delete-daemon-green/summary.json`
  with 55/55 scenarios as expected in run `20261001T110312Z-1823580` with
  Insula checks passing. Its
  `daemon-dired-delete-stale-signature-rejected` scenario proves stable-v0
  `dired.delete_marked` rejects an externally replaced marked file through the
  failed `file signature for marked file delete-a.txt matches` precondition,
  publishes no filesystem delete effect, preserves the replacement contents, and
  only permits the later positive delete after fresh signatures are observed.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/dired-unsafe-delete-daemon-green/summary.json`
  with 56/56 scenarios as expected in run `20261001T111156Z-2007938` with
  Insula checks passing. Its
  `daemon-dired-delete-unsafe-marked-rejected` scenario proves stable-v0
  `dired.delete_marked` rejects a `../` marked-file name through the failed
  `marked files are explicit safe relative basenames` precondition, publishes no
  filesystem delete effect, and preserves the escaped file outside the Dired
  directory. The bridge also avoids collecting delete filesystem evidence for
  unsafe marked names before returning the failed precondition result.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/org-export-direct-escape-daemon-green/summary.json`
  with 57/57 scenarios as expected in run `20261001T111825Z-2080142` with
  Insula checks passing. Its `daemon-org-export-direct-escape-rejected`
  scenario proves stable-v0 `org.export_html` rejects a direct `../` output path
  through the failed `output path is inside publish directory` precondition,
  publishes no HTML export effect, and creates no requested or resolved outside
  HTML artifact.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/lsp-stale-document-daemon-green/summary.json`
  with 58/58 scenarios as expected in run `20261001T113123Z-2258625` with
  Insula checks passing. Its
  `daemon-lsp-stale-document-version-rejected` scenario proves stable-v0
  `lsp.rename_symbol` rejects stale LSP document-version preconditions before
  applying a workspace edit, reports the failed `LSP document version matches
  precondition` check, publishes no workspace-edit effect, and preserves the
  rejected symbol name.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/org-stale-parent-daemon-green/summary.json`
  with 59/59 scenarios as expected in run `20261001T115516Z-2474499` with
  Insula checks passing. Its
  `daemon-org-source-edit-stale-parent-rejected` scenario proves stable-v0
  `org.exit_src_edit` rejects a source-edit writeback after the parent Org
  buffer changed, reports the failed `parent structure fingerprint matches
  precondition` check, publishes no parent-buffer writeback effect, preserves
  the parent mutation, and keeps the edited source-buffer text out of the parent
  document.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` refreshed the
  canonical `benchmarks/emacs-gym/bridge/traces/latest-daemon/summary.json`
  pointer with 63/63 scenarios as expected in run
  `20261001T131239Z-3232868` with Insula checks passing. The refreshed pointer
  contains the current no-host-tool-fallback and semantic-operation bypass
  negatives, includes `daemon-buffer-wrong-text-owner-rejected` and
  `daemon-dired-rename-collision-rejected`, covers duplicate mutating
  `request_id` rejection, Dired unsafe-target and stale-delete rejections, and
  Dired unsafe marked-delete rejection, stale LSP document-version rejection,
  stale Org source-edit parent rejection, duplicate Org ID heading-update
  rejection, Org heading text-substitution bypass rejection, Org subtree
  text-reparenting bypass rejection, wrong Babel-session rejection, direct and
  symlinked Org HTML export escape rejections, `org.export_html`
  preview/apply behavior, raw filesystem rename rejection, and keeps the
  `daemon-jupyter-display-update` request lifecycle green by reconciling
  generic await with semantic
  `jupyter.await_request`. A manifest-vs-trace audit of this run finds 40 stable
  enabled operations in the first six profiles and no missing stable operation
  among captured v0 daemon responses.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/frame-observe-stable-green/summary.json`
  with 63/63 scenarios as expected in run `20261001T132346Z-3347347` and Insula
  checks passing. This run promotes `emacs.observe_frames` into stable
  `core-v0`, proves the daemon manifest advertises it as `frame-read` with
  `frame_identity` and `terminal_client_identity` evidence, and exercises it
  through the stable v0 envelope in the profile smoke scenario. The response
  publishes typed passive frame evidence and observations with no effects. The
  same run validates 933 captured daemon response envelopes; a manifest-vs-trace
  audit finds 41 stable enabled operations in the first six profiles, 41
  captured stable daemon response operations, and no missing stable operation.
- `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/request-respond-nonstable-batch-green-2/summary.json`
  with 30/30 scenarios as expected in run `20261001T134453Z-3597764` and Insula
  checks passing. Its manifest introspection and request-envelope validation
  scenarios prove `request.respond` is still implemented and enabled but
  remains `experimental` in `core-v0`, and explicit stable-v0 calls fail before
  dispatch with `non_stable_operation`.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/request-respond-nonstable-v0-green-2/summary.json`
  with 63/63 scenarios as expected in run `20261001T133813Z-3522499` and Insula
  checks passing. Its request-envelope validation scenario proves explicit
  stable-v0 `request.respond` requests fail before dispatch with
  `non_stable_operation`.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` generated
  `benchmarks/emacs-gym/bridge/traces/profile-smoke-jupyter-display-green-2/summary.json`
  with 63/63 scenarios as expected in run `20261001T143144Z-4164242` and Insula
  checks passing. Its stable-profile smoke now covers all 41 stable operations
  in `core-v0`, `coding-v0`, `terminal-v0`, `analysis-v0`, `reports-v0`, and
  `vcs-files-v0`, with no missing stable operation. The newly closed coverage
  includes `dired.delete_marked`, `magit.stage_file`, and
  `jupyter.observe_display`. The Jupyter display check observes a
  profile-owned REPL display buffer after fixture setup created real
  emacs-jupyter `display_data` and `update_display_data` state; the profile
  trace itself uses only stable operations and asserts typed passive display
  evidence with no effects.
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

Completion audit:

- The selected rootfs contract verifier passes with no errors or warnings for
  the rootfs used by the closing daemon trace.
- `api.list_capabilities` and `api.describe_operation` expose the stable
  operation manifest and operation contracts.
- Request-envelope and response-envelope validation scenarios are green under
  the live daemon.
- The first stable profiles are implemented and covered in the profile smoke:
  `core-v0`, `coding-v0`, `terminal-v0`, `analysis-v0`, `reports-v0`, and
  `vcs-files-v0`.
- The profile smoke covers all 41 stable first-six-profile operations.
- The selected-rootfs daemon suite covers the stale/wrong-owner and unsafe
  negative cases listed in this issue, plus direct call, arbitrary Elisp, raw
  process write, host tool fallback, raw Git/filesystem bypass, implicit
  Jupyter restart, and benchmark-only shortcut rejection.
- The selected-rootfs batch bridge trace at
  `benchmarks/emacs-gym/bridge/traces/v0-final-batch-green/summary.json`
  reports 30/30 scenarios as expected in run `20261001T143938Z-44198` with
  Insula checks passing. This keeps the batch manifest, request-envelope,
  response-envelope, stable-profile manifest, typed evidence/effects, and
  negative authorization gates green after the daemon profile smoke reached
  41/41 stable operation coverage.

Post-v0 maintenance:

- Keep the no-host-tool-fallback and semantic-bypass suites green when new
  subprocess-owning operations are added.

## Notes

RFC 0003 is the design source for this task:

```text
docs/rfcs/0003-emacs-agent-api.md
```

CUA and vision are intentionally out of scope for this task. They can validate
or extend the API later, but they should not block the first safe semantic
profile.
