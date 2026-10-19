# 0013: Add Org Report And HTML Export API Benchmark

Status: In progress
Type: Follow-up task
Depends on: 0007 Comint Process Benchmark; 0006 Existing-Kernel Jupyter Benchmark; 0011 Safe Emacs Agent API Profile

## Intent

Exercise the reporting side of the safe Emacs API: maintain an Org task/report
document, run selected analysis, insert results, and export a publishable HTML
artifact without granting arbitrary shell or filesystem authority.

## Requirements

- Implement or stabilize report-profile operations:
  - `org.observe_tree`
  - `org.update_heading`
  - `org.edit_src_block`
  - `org.exit_src_edit`
  - `org.execute_babel_block`
  - `org.export_html`
- Split HTML export into preview and apply phases.
- Make export output path, backend, options, CSS/theme assets, source-block
  execution policy, and generated artifacts explicit.
- Ensure Babel execution uses rootfs-owned runtimes, kernels, working
  directories, and process evidence.
- Verify parent Org document state, not temporary source-edit buffer state.

## Verification

- A scripted policy updates a TODO/report heading, edits a source block through
  the Org source-edit workflow, executes the block in the intended rootfs-owned
  runtime or existing Jupyter kernel, inserts results in the parent Org buffer,
  exports HTML, and returns generated artifact evidence.
- Negative cases reject stale heading/source-edit handles, duplicate IDs,
  visible-text-only subtree movement, wrong Babel session/kernel, host runtime
  fallback, unsafe output paths, and generated files outside the declared
  publish directory.
- The generated HTML artifact is independently checked for expected content,
  declared assets, and absence of undeclared filesystem writes.

## Current Checkpoint

As of 2026-10-01, the shared daemon bridge exposes stable `reports-v0`
operations for:

- `org.observe_tree`
- `org.update_heading`
- `org.edit_src_block`
- `org.exit_src_edit`
- `org.execute_babel_block`
- `org.export_html`

The selected-rootfs daemon benchmark includes
`daemon-reports-vcs-files-semantic-api`, which verifies that the live daemon can
observe the Org tree, update a heading by semantic preconditions, open and exit
an Org source-edit buffer, reject stale source-edit parent state, execute a
Babel block with rootfs runtime evidence, reject a wrong Babel session before
execution without publishing `effects.org_babel_execution`, preview an HTML
export plan without creating `report-preview.html` or publishing
`effects.html_export`, apply HTML export with `effects.html_export`, and check
the generated report artifact.

The canonical selected-rootfs daemon trace at
`benchmarks/emacs-gym/bridge/traces/latest-daemon/summary.json` reports 63/63
scenarios as expected in run `20261001T131239Z-3232868`, with Insula checks
passing and 931 daemon response envelopes validated. The
`daemon-org-duplicate-id-rejected` record proves `org.update_heading` rejects a
buffer where the target ID appears on multiple headings through the failed
`Org heading ID is unique in buffer` precondition, emits no heading-update
effect, and preserves the Org buffer text. The
`daemon-org-heading-text-bypass-rejected` record proves a direct checked
`buffer.replace_range` can make the visible Org heading line match the desired
final text, but `episode.finish` rejects the task because the required semantic
`org.update_heading` request evidence is absent; the direct edit publishes only
`changed_range`, not a heading-update effect. The
`daemon-org-subtree-text-bypass-rejected` record proves a direct whole-buffer
text replacement can reparent an Org child subtree and pass final text checks,
but `episode.finish` rejects the task because required semantic subtree-move
evidence is absent; the direct edit publishes only `changed_range`, not an Org
structure effect. The targeted preview/apply
trace at `benchmarks/emacs-gym/bridge/traces/org-export-preview-green/summary.json`
reports 60/60 scenarios as expected in run `20261001T122412Z-2769786`; its
`daemon-reports-vcs-files-semantic-api` record shows preview mode returning
`mode=preview`, route `org-export-preview`, no effects, and no preview artifact,
while apply mode writes `report.html` and publishes `effects.html_export`.

This checkpoint does not close the ticket. Remaining work includes richer
declared CSS/theme asset policy, wrong Jupyter-backed report kernel rejection,
host runtime fallback coverage for Babel or Jupyter-backed report execution,
and generated asset absence checks.
