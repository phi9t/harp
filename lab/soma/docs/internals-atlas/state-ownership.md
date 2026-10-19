# State Ownership

The gym state model must represent ownership boundaries, not just visible text.

## Ownership Graph

Minimum objects:

- Emacs instance: binary, startup mode, profile, environment, process id,
  terminal frame, and command-loop state.
- Buffer object: buffer name, major mode, local variables, modified flag,
  modification ticks, point, mark, active region, narrowing, overlays, undo
  state, and process association.
- Text owner: base buffer or shared underlying text identity. Indirect buffers
  are distinct buffer objects that share text with a base buffer.
- Visited file: truename, buffer-file-name, file attributes, save state, and
  external modification state.
- Window: selected window, displayed buffer, window point, window history,
  dedicated/side/popup parameters, and minibuffer relation.
- Marker/range: buffer identity, position, insertion type, captured surrounding
  text, and semantic validation target.
- Package view: Magit section tree, Org source-edit buffer, Dired mark state,
  completion overlay, Jupyter client, LSP workspace, or Comint process state.
- External object: Git index/worktree, filesystem entries, language server,
  subprocess, Jupyter kernel, or tmux/terminal state.

## Required Distinctions

Current buffer is not selected window:

- `with-current-buffer` is a dynamic binding used for Lisp operations.
- `switch-to-buffer` changes display/window state and should be treated as a
  user-visible action.
- Probe `current-buffer-selected-window` showed that the current buffer can be
  `soma-probe-b` while the selected window still displays `soma-probe-a`.

Buffer identity is not text identity:

- Probe `base-indirect-buffers` showed that editing an indirect buffer changed
  the base buffer text.
- A checker cannot claim "only buffer A changed" unless it knows whether another
  buffer shares A's text owner.

Position is not semantic identity:

- Probe `marker-movement` showed that a marker stayed live and moved while the
  local text context changed.
- Delayed edits must validate buffer identity, target-range text, prefix/suffix,
  and expected revision, not only marker liveness.

Character revision is not full state revision:

- Probe `narrowing-region-without-text-change` showed that narrowing and active
  region state changed while `buffer-chars-modified-tick` did not.
- Text properties, overlays, point, region, selected window, package selection,
  and completion overlays need operation-specific guards.

Undo is scoped:

- Text undo can reverse some buffer mutations.
- Dired undo restores Dired buffer presentation but does not generally undo
  filesystem deletions.
- Multi-buffer, Git, process, and kernel side effects require explicit
  operation-specific recovery, not a generic undo action.

## Observation Record

Each learner-visible observation should include stable IDs for:

- Emacs instance and action profile
- selected frame/window
- current buffer
- displayed buffer per window
- buffer object and base-buffer/text-owner identity
- visited file identity
- buffer revision fields relevant to the requested action
- active minibuffer/transient/completion state
- package-owned handles, with generation counters when available

Do not include every buffer dump by default. Observation should be demand-driven,
but every operation must name the identities and freshness predicates it relies
on.

## Mutation Preconditions

Text replacement:

- buffer object id
- text-owner id
- expected character tick
- expected target range text
- prefix/suffix validation for delayed suggestions
- narrowing and read-only state check

Visible editor action:

- selected window id
- displayed buffer id
- point/region/Evil or non-Evil mode state
- active minibuffer or transient status

Package action:

- package view id and generation
- selected section/heading/client/process/completion overlay id
- package-specific readiness state
- external object identity, such as repository, process pid, or kernel id

External action:

- explicit capability
- idempotence classification
- completion evidence
- recovery or outcome_unknown handling
