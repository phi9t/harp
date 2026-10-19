;;; soma-gym-bridge.el --- Minimal checked bridge for Emacs gym benchmarks -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'eieio)
(require 'json)
(require 'seq)
(require 'subr-x)

(defconst soma-gym-bridge-api-version "soma-emacs-agent-v0"
  "Version string for the stable model-facing Emacs agent interface.")

(defvar soma-gym-bridge-command-allowlist
  '(magit-stage)
  "Commands allowed through the first checked benchmark bridge slice.")

(defvar soma-gym-bridge-operation-allowlist nil
  "Optional list of operation names accepted by the request transport.

When nil, all operations implemented by `soma-gym-bridge--execute-request' are
reachable.  Benchmarks can bind or set this to publish a narrower live transport
surface without changing the shared bridge implementation.")

(defvar soma-gym-bridge-async-command-dispatch nil
  "When non-nil, queue `command.dispatch' work outside the JSONL process filter.

Batch probes keep this nil so a submitted command request returns a terminal
state.  Live daemon benchmarks enable it so package commands can run after the
transport has recorded an in-flight request that `request.status' and
`request.await' can observe.")

(defvar soma-gym-bridge-async-command-delay-seconds 0.01
  "Delay before running queued `command.dispatch' work.

A small positive delay keeps package command execution out of the network
process filter that accepted the request.")

(defvar soma-gym-bridge--requests (make-hash-table :test 'equal)
  "Model-facing bridge request records, keyed by request id.")

(defvar soma-gym-bridge--pending-actions (make-hash-table :test 'equal)
  "Queued request completion actions, keyed by request id.")

(defvar soma-gym-bridge--transport-epoch 0
  "Generation counter used to ignore stale async completions after reset.")

(defvar soma-gym-bridge--async-timers nil
  "Timers owned by the current bridge transport generation.")

(defvar soma-gym-bridge--jsonl-server nil
  "Current JSONL bridge server process, if any.")

(defvar soma-gym-bridge--jsonl-socket nil
  "Filesystem path for the current JSONL bridge server socket.")

(defvar transient--prefix)
(defvar transient--transient-map)
(defvar transient--window)
(defvar transient--buffer)
(defvar transient--suffixes)
(defvar transient-current-command)
(defvar comint-last-input-start)
(defvar comint-last-input-end)
(defvar magit-root-section)
(defvar jupyter-current-client)
(defvar jupyter-repl-interaction-mode)
(defvar dired-deletion-confirmer)
(defvar dired-no-confirm)
(defvar dired-recursive-deletes)
(defvar jupyter-display-buffer-marker)
(defvar jupyter-display-ids)
(defvar lsp--buffer-workspaces)
(defvar lsp--cur-version)
(defvar lsp--delayed-requests)
(defvar lsp--delay-timer)
(defvar lsp--position-column-function)
(defvar lsp-response-timeout)
(defvar org-confirm-babel-evaluate)
(defvar org-export-show-temporary-export-buffer)
(defvar org-src--allow-write-back)
(defvar org-src--beg-marker)
(defvar org-src--end-marker)
(defvar org-todo-keywords-1)
(defvar doom-modules)

(declare-function comint-send-input "comint")
(declare-function comint-interrupt-subjob "comint")
(declare-function dired-do-delete "dired-aux")
(declare-function dired-get-filename "dired")
(declare-function dired-get-marked-files "dired")
(declare-function dired-goto-file "dired")
(declare-function revert-buffer "files")
(declare-function jupyter-kernel-language-mode "jupyter-client")
(declare-function jupyter-repl-associate-buffer "jupyter-repl")
(declare-function jupyter-repl-connected-p "jupyter-repl")
(declare-function jupyter-repl-execute-cell "jupyter-repl")
(declare-function jupyter-repl-replace-cell-code "jupyter-repl")
(declare-function jupyter-session-id "jupyter-base")
(declare-function org-back-to-heading "org")
(declare-function org-babel-execute-src-block "ob-core")
(declare-function org-babel-get-src-block-info "ob-core")
(declare-function org-babel-result-end "ob-core")
(declare-function org-babel-where-is-src-block-result "ob-core")
(declare-function org-edit-headline "org")
(declare-function org-edit-src-code "org-src")
(declare-function org-edit-src-exit "org-src")
(declare-function org-entry-put "org")
(declare-function org-find-property "org")
(declare-function org-src-edit-buffer-p "org-src")
(declare-function org-src-source-buffer "org-src")

(defvar-local soma-gym-bridge-jupyter-connection-file nil
  "Connection file for a benchmark-owned Jupyter source buffer.")

(defvar-local soma-gym-bridge-jupyter-kernel-token nil
  "Fixture token for a benchmark-owned Jupyter source buffer.")

(defvar-local soma-gym-bridge-jupyter-kernel-pid nil
  "Kernel process id for a benchmark-owned Jupyter source buffer.")

(defvar-local soma-gym-bridge-jupyter-kernel-exe nil
  "Kernel executable path for a benchmark-owned Jupyter source buffer.")

(defun soma-gym-bridge--alist (&rest pairs)
  pairs)

(defun soma-gym-bridge--empty-object ()
  "Return a fresh empty JSON object."
  (make-hash-table :test 'equal))

(defun soma-gym-bridge--bool (value)
  (if value t :json-false))

(defun soma-gym-bridge--get (alist key)
  (alist-get key alist nil nil #'string=))

(defun soma-gym-bridge--key-string (key)
  (cond
   ((keywordp key)
    (substring (symbol-name key) 1))
   ((symbolp key)
    (symbol-name key))
   ((stringp key)
    key)
   (t
    (format "%S" key))))

(defun soma-gym-bridge--value (object key)
  (cond
   ((hash-table-p object)
    (or (gethash key object)
        (gethash (intern key) object)
        (gethash (intern (concat ":" key)) object)))
   ((and (listp object)
         (keywordp (car object)))
    (plist-get object (intern (concat ":" key))))
   ((listp object)
    (or (alist-get key object nil nil #'string=)
        (alist-get (intern key) object nil nil #'eq)
        (alist-get (intern (concat ":" key)) object nil nil #'eq)))))

(defun soma-gym-bridge--jsonish (object)
  (cond
   ((hash-table-p object)
    (let (items)
      (maphash
       (lambda (key value)
         (push (cons (soma-gym-bridge--key-string key)
                     (soma-gym-bridge--jsonish value))
               items))
       object)
      (nreverse items)))
   ((and (listp object)
         (keywordp (car object)))
    (let (items)
      (while object
        (push (cons (soma-gym-bridge--key-string (car object))
                    (soma-gym-bridge--jsonish (cadr object)))
              items)
        (setq object (cddr object)))
      (nreverse items)))
   ((and (listp object)
         (consp (car object)))
    (mapcar
     (lambda (item)
       (cons (soma-gym-bridge--key-string (car item))
             (soma-gym-bridge--jsonish (cdr item))))
     object))
   ((vectorp object)
    (mapcar #'soma-gym-bridge--jsonish object))
   (t
    object)))

(defun soma-gym-bridge--keys (object)
  (cond
   ((hash-table-p object)
    (let (keys)
      (maphash
       (lambda (key _value)
         (push (soma-gym-bridge--key-string key) keys))
       object)
      (sort keys #'string<)))
   ((and (listp object)
         (keywordp (car object)))
    (let (keys)
      (while object
        (push (soma-gym-bridge--key-string (car object)) keys)
        (setq object (cddr object)))
      (sort keys #'string<)))
   ((and (listp object)
         (consp (car object)))
    (sort (mapcar (lambda (item)
                    (soma-gym-bridge--key-string (car item)))
                  object)
          #'string<))))

(defun soma-gym-bridge--has-key-p (alist key)
  (not (null (cl-assoc key alist :test #'string=))))

(defun soma-gym-bridge--present-p (value)
  (and value (not (eq value :json-false))))

(defun soma-gym-bridge--check (name passed detail)
  (soma-gym-bridge--alist
   (cons "name" name)
   (cons "passed" (soma-gym-bridge--bool passed))
   (cons "detail" detail)))

(defun soma-gym-bridge--all-checks-pass-p (checks)
  (cl-every
   (lambda (check)
     (eq (soma-gym-bridge--get check "passed") t))
   checks))

(defun soma-gym-bridge--operation-enabled-p (operation)
  (soma-gym-bridge--bool
   (or (null soma-gym-bridge-operation-allowlist)
       (member operation soma-gym-bridge-operation-allowlist))))

(defun soma-gym-bridge--request-contract
    (side-effect-class required-params required-preconditions)
  "Return the v0 request-envelope contract for one operation."
  (soma-gym-bridge--alist
   (cons "envelope" "soma-emacs-agent-v0.request")
   (cons "required_fields" '("request_id" "operation"))
   (cons "optional_fields"
         '("api_version" "capability" "params" "preconditions"
           "timeout_ms"))
   (cons "api_version" soma-gym-bridge-api-version)
   (cons "capability" side-effect-class)
   (cons "params_mode" "typed-object")
   (cons "required_params" (or required-params nil))
   (cons "preconditions_mode"
         (if required-preconditions
             "typed-object"
           "not-required"))))

(defun soma-gym-bridge--precondition-contract (required-preconditions)
  "Return the v0 typed-precondition contract for one operation."
  (soma-gym-bridge--alist
   (cons "mode" "typed-object")
   (cons "top_level_field" "preconditions")
   (cons "params_field" "params.preconditions")
   (cons "required_fields" (or required-preconditions nil))
   (cons "conflict_error_code" "conflict")))

(defun soma-gym-bridge--result-contract (possible-states evidence)
  "Return the v0 result-envelope contract for one operation."
  (soma-gym-bridge--alist
   (cons "envelope" "soma-emacs-agent-v0.result")
   (cons "required_fields"
         '("api_version" "request_id" "operation" "state"
           "state_sequence" "result" "evidence" "observations"
           "effects" "diagnostics" "continuation"))
   (cons "states" (or possible-states nil))
   (cons "terminal_states" '("succeeded" "failed" "outcome_unknown"))
   (cons "evidence_channels" (or evidence nil))))

(defun soma-gym-bridge--operation-spec
    (operation profile maturity owner side-effect-class observation-class
               implemented route required-params required-preconditions
               possible-states evidence description)
  "Return a model-facing manifest entry for OPERATION."
  (soma-gym-bridge--alist
   (cons "operation" operation)
   (cons "api_version" soma-gym-bridge-api-version)
   (cons "profile" profile)
   (cons "maturity" maturity)
   (cons "owner" owner)
   (cons "side_effect_class" side-effect-class)
   (cons "observation_class" observation-class)
   (cons "implemented" (soma-gym-bridge--bool implemented))
   (cons "enabled" (soma-gym-bridge--bool
                    (and implemented
                         (eq (soma-gym-bridge--operation-enabled-p operation)
                             t))))
   (cons "route" route)
   (cons "required_params" (or required-params nil))
   (cons "required_preconditions" (or required-preconditions nil))
   (cons "possible_states" (or possible-states nil))
   (cons "evidence" (or evidence nil))
   (cons "request_contract"
         (soma-gym-bridge--request-contract
          side-effect-class required-params required-preconditions))
   (cons "precondition_contract"
         (soma-gym-bridge--precondition-contract required-preconditions))
   (cons "result_contract"
         (soma-gym-bridge--result-contract possible-states evidence))
   (cons "description" description)))

(defun soma-gym-bridge--operation-catalog ()
  "Return the bridge operation catalog for `api.list_capabilities'."
  (list
   (soma-gym-bridge--operation-spec
    "api.list_capabilities" "core-v0" "stable" "bridge"
    "passive_observation" "passive" t "bridge-helper"
    nil nil '("succeeded")
    '("operation_manifest" "profile_manifest" "enabled_operation_flags")
    "List the installed Soma Emacs agent interface profiles and operations.")
   (soma-gym-bridge--operation-spec
    "api.describe_operation" "core-v0" "stable" "bridge"
    "passive_observation" "passive" t "bridge-helper"
    '("operation") nil '("succeeded" "failed")
    '("owner" "preconditions" "states" "evidence" "maturity")
    "Describe one installed operation and the facts a caller must know.")
   (soma-gym-bridge--operation-spec
    "emacs.observe_session" "core-v0" "stable" "core-emacs-session"
    "passive_observation" "passive" t "bridge-helper"
    nil nil '("succeeded")
    '("api_version" "profile" "rootfs_identity" "doom_identity")
    "Observe Emacs, Doom, rootfs, and selected profile identity.")
   (soma-gym-bridge--operation-spec
    "emacs.observe_buffer" "core-v0" "stable" "core-emacs-buffer"
    "passive_observation" "passive" t "explicit-buffer-read"
    '("buffer_name") nil '("succeeded" "failed")
    '("buffer_identity" "text_owner" "point_region_narrowing")
    "Observe buffer identity and editor-local state without mutation.")
   (soma-gym-bridge--operation-spec
    "emacs.observe_frames" "core-v0" "stable" "core-emacs-frame"
    "passive_observation" "passive" t "frame-read"
    nil nil '("succeeded")
    '("frame_identity" "terminal_client_identity")
    "Observe live frame and terminal-client state.")
   (soma-gym-bridge--operation-spec
    "emacs.observe_interaction" "core-v0" "stable" "core-emacs-command-loop"
    "passive_observation" "passive" t "interaction-read"
    nil nil '("succeeded")
    '("minibuffer" "transient" "this_command" "last_command")
    "Observe prompt, transient, minibuffer, and command-loop context.")
   (soma-gym-bridge--operation-spec
    "buffer.select" "core-v0" "stable" "core-emacs-window"
    "editor_state_change" "active-diagnostic" t "selected-window-set-buffer"
    '("buffer_name") nil '("succeeded" "failed")
    '("before_after_observations" "selected_window")
    "Display an existing live buffer in the selected window without opening files or invoking package commands.")
   (soma-gym-bridge--operation-spec
    "request.status" "core-v0" "stable" "bridge-request-store"
    "passive_observation" "passive" t "request-store-read"
    '("request_id") nil '("succeeded" "failed")
    '("stored_request_state" "terminal_or_running_state")
    "Read the stored state for a submitted request id.")
   (soma-gym-bridge--operation-spec
    "request.await" "core-v0" "stable" "bridge-request-store"
    "passive_observation" "active-diagnostic" t "request-store-await"
    '("request_id") nil '("succeeded" "failed" "outcome_unknown")
    '("stored_request_state" "timeout_seconds")
    "Wait for a request to reach a terminal state without resubmitting it.")
   (soma-gym-bridge--operation-spec
    "request.respond" "core-v0" "experimental" "bridge-request-store"
    "continuation_response" "active-diagnostic" t "request-continuation"
    '("request_id" "response") nil '("succeeded" "failed")
    '("target_status" "prompt_response")
    "Respond to a request that is waiting for explicit input.")
   (soma-gym-bridge--operation-spec
    "project.observe" "coding-v0" "stable" "project"
    "passive_observation" "derived" t "project-helper"
    nil nil '("succeeded" "failed")
	    '("project_root" "vcs_identity" "search_tool_identity"
	      "rootfs_runtime")
    "Observe source project identity without shelling out arbitrarily.")
   (soma-gym-bridge--operation-spec
    "project.search" "coding-v0" "stable" "project"
    "passive_observation" "derived" t "rootfs-owned-search-tool"
    '("query") nil '("succeeded" "failed")
	    '("bounded_results" "search_tool_identity" "rootfs_runtime")
    "Search source text through a bounded rootfs-owned project search path.")
   (soma-gym-bridge--operation-spec
    "buffer.read" "coding-v0" "stable" "core-emacs-buffer"
    "passive_observation" "passive" t "explicit-buffer-read"
    '("buffer_name" "start" "end") nil '("succeeded" "failed")
    '("buffer_identity" "text_range" "text_snapshot")
    "Read a buffer range without moving point, mark, window, or revision.")
   (soma-gym-bridge--operation-spec
    "buffer.replace_range" "coding-v0" "stable" "core-emacs-buffer"
    "reversible_editor_edit" "active-diagnostic" t "checked-buffer-edit"
    '("buffer_name" "start" "end" "text")
    '("buffer_name" "text_owner_name" "buffer_file_name" "chars_tick"
      "expected_text" "point" "selected_window_buffer" "modified"
      "read_only" "minibuffer_active")
    '("succeeded" "failed")
    '("precondition_checks" "before_after_observations" "changed_range"
      "error")
    "Replace one buffer range only when buffer, text owner, range text, and relevant UI state still match.")
   (soma-gym-bridge--operation-spec
    "buffer.apply_patch" "coding-v0" "stable" "core-emacs-buffer"
    "reversible_editor_edit" "active-diagnostic" t "checked-buffer-patch"
    '("buffer_name" "patch")
    '("buffer_name" "text_owner_name" "buffer_file_name" "chars_tick"
      "point" "selected_window_buffer" "modified" "read_only"
      "minibuffer_active")
    '("succeeded" "failed")
    '("precondition_checks" "conflict_preview" "before_after_observations"
      "changed_ranges")
    "Apply a one-buffer patch after explicit conflict preview.")
   (soma-gym-bridge--operation-spec
    "buffer.save" "coding-v0" "stable" "core-emacs-buffer"
    "filesystem_write" "active-diagnostic" t "checked-buffer-save"
    '("buffer_name")
    '("buffer_name" "buffer_file_name" "chars_tick")
    '("succeeded" "failed")
    '("precondition_checks" "before_after_observations" "visited_file")
    "Save the visited file only after buffer freshness checks.")
   (soma-gym-bridge--operation-spec
    "completion.observe" "coding-v0" "stable" "completion"
    "passive_observation" "derived" t "completion-provider-read"
    '("buffer_name") nil '("succeeded" "failed")
    '("provider" "replacement_range" "candidates" "preview_state"
      "acceptance_preconditions")
    "Observe completion provider state and accept-ready context without accepting a suggestion.")
   (soma-gym-bridge--operation-spec
    "completion.accept" "coding-v0" "stable" "completion"
    "reversible_editor_edit" "active-diagnostic" t "checked-completion-accept"
    '("buffer_name" "start" "end" "text")
    '("buffer_name" "chars_tick" "point" "expected_prefix"
      "expected_suffix" "selected_window_buffer")
    '("succeeded" "failed")
    '("precondition_checks" "provider_exit_function" "before_after_observations")
    "Accept a completion only when range, point, prefix, suffix, and provider context still match.")
   (soma-gym-bridge--operation-spec
    "lsp.observe_source" "coding-v0" "stable" "lsp"
    "passive_observation" "active-diagnostic" t "lsp-source-read"
    '("buffer_name") nil '("succeeded" "failed")
    '("workspace" "client" "document_version" "diagnostics")
    "Observe LSP source, workspace, document, server, and diagnostic state; may flush or request diagnostic state.")
   (soma-gym-bridge--operation-spec
    "lsp.observe_diagnostics" "coding-v0" "stable" "lsp"
    "passive_observation" "active-diagnostic" t "lsp-diagnostics-request"
    '("buffer_name") nil '("succeeded" "failed" "outcome_unknown")
    '("precondition_checks" "diagnostics" "diagnostic_cache"
      "server_identity" "document_version")
    "Observe diagnostics as a first-class LSP operation with workspace, server, and document-owner evidence.")
   (soma-gym-bridge--operation-spec
    "lsp.rename_symbol" "coding-v0" "stable" "lsp"
    "multi_buffer_editor_edit" "active-diagnostic" t "lsp-rename"
    '("buffer_name" "new_name")
    '("buffer_name" "workspace_root" "document_version" "point")
    '("running" "succeeded" "failed" "outcome_unknown")
    '("lsp_request" "workspace_edit" "affected_files" "diagnostics")
    "Run the real LSP rename path and record workspace-edit evidence.")
   (soma-gym-bridge--operation-spec
    "comint.observe" "terminal-v0" "stable" "comint"
    "passive_observation" "passive" t "comint-state-read"
    '("buffer_name") nil '("succeeded" "failed")
    '("process_identity" "process_mark" "current_input" "buffer_tail")
    "Observe Comint process, input-boundary, and recent output state.")
   (soma-gym-bridge--operation-spec
    "comint.submit_input" "terminal-v0" "stable" "comint"
    "external_process_input" "active-diagnostic" t "comint-send-input"
    '("buffer_name" "input")
    '("buffer_name" "process_id" "process_exe" "process_rootfs_owned"
      "process_mark" "current_input_text")
    '("succeeded" "failed" "outcome_unknown")
    '("submitted_bytes" "comint_input_markers" "process_output")
    "Submit input through Comint, not raw process-send-string.")
   (soma-gym-bridge--operation-spec
    "comint.await_output" "terminal-v0" "stable" "comint"
    "passive_observation" "active-diagnostic" t "process-output-await"
    '("buffer_name")
    '("buffer_name" "process_id" "process_exe" "process_rootfs_owned"
      "process_mark")
    '("succeeded" "failed" "outcome_unknown")
    '("matched_output" "partial_output" "timeout")
    "Await output from the intended Comint process buffer.")
   (soma-gym-bridge--operation-spec
    "comint.interrupt" "terminal-v0" "stable" "comint"
    "external_process_control" "active-diagnostic" t
    "comint-interrupt-subjob"
    '("buffer_name")
    '("buffer_name" "process_id" "process_exe" "process_rootfs_owned"
      "process_mark")
    '("succeeded" "failed" "outcome_unknown")
    '("interrupt_command" "process_identity" "process_survival"
      "side_effect_reconciliation")
    "Interrupt the intended Comint subprocess and reconcile outcome.")
   (soma-gym-bridge--operation-spec
    "jupyter.observe_source" "analysis-v0" "stable" "jupyter"
    "passive_observation" "derived" t "jupyter-source-read"
    '("buffer_name") nil '("succeeded" "failed")
    '("source_buffer" "client" "kernel_incarnation" "selected_region")
    "Observe source-buffer association to an existing Jupyter client/kernel.")
   (soma-gym-bridge--operation-spec
    "jupyter.attach_existing" "analysis-v0" "stable" "jupyter"
    "editor_state_change" "active-diagnostic" t "jupyter-repl-associate-buffer"
    '("buffer_name" "client")
    '("kernel_id" "kernel_session" "jupyter_connection_file"
      "jupyter_kernel_exe" "jupyter_connection_file_rootfs_owned"
      "jupyter_kernel_exe_rootfs_owned")
    '("succeeded" "failed")
    '("precondition_checks" "client_association" "kernel_identity")
    "Associate a source buffer with an intended existing client; never restart implicitly.")
   (soma-gym-bridge--operation-spec
    "jupyter.eval_region" "analysis-v0" "stable" "jupyter"
    "kernel_execution" "active-diagnostic" t "jupyter-existing-kernel-eval"
    '("buffer_name" "start" "end")
    '("buffer_name" "chars_tick" "region_text" "client" "kernel_token"
      "jupyter_connection_file" "jupyter_kernel_exe"
      "jupyter_connection_file_rootfs_owned"
      "jupyter_kernel_exe_rootfs_owned")
    '("running" "succeeded" "failed" "outcome_unknown")
    '("request_id" "parent_header" "reply" "idle" "rootfs_runtime")
    "Execute exactly the observed source region in an associated existing kernel.")
   (soma-gym-bridge--operation-spec
    "jupyter.await_request" "analysis-v0" "stable" "jupyter"
    "passive_observation" "active-diagnostic" t "jupyter-request-await"
    '("request_id") nil '("succeeded" "failed" "outcome_unknown")
    '("reply" "iopub" "idle" "timeout")
    "Await an existing Jupyter request without blind retry.")
   (soma-gym-bridge--operation-spec
    "jupyter.observe_display" "analysis-v0" "stable" "jupyter"
    "passive_observation" "derived" t "jupyter-display-read"
    '("buffer_name" "display_id") nil '("succeeded" "failed")
    '("display_id" "display_data" "update_display_data" "repl_range")
    "Observe Jupyter display-data and update-display-data presentation.")
   (soma-gym-bridge--operation-spec
    "jupyter.eval_region_via_repl" "analysis-v0" "experimental" "jupyter"
    "kernel_execution" "active-diagnostic" t "jupyter-repl-execute-cell"
    '("buffer_name" "start" "end")
    '("buffer_name" "region_text" "client" "kernel_token"
      "jupyter_connection_file" "jupyter_kernel_exe"
      "jupyter_connection_file_rootfs_owned"
      "jupyter_kernel_exe_rootfs_owned")
    '("running" "succeeded" "failed" "outcome_unknown")
    '("request_id" "display_handlers" "repl_buffer")
    "Execute a region through emacs-jupyter REPL display handlers.")
   (soma-gym-bridge--operation-spec
    "jupyter.query_state" "analysis-v0" "benchmark-only" "jupyter"
    "kernel_execution" "active-diagnostic" t "benchmark-helper"
    '("buffer_name") nil '("succeeded" "failed" "outcome_unknown")
    '("fixture_state" "rootfs_runtime")
    "Benchmark helper for fixed Jupyter state probes; not a stable API operation.")
   (soma-gym-bridge--operation-spec
    "jupyter.restart_kernel" "harness-debug" "negative-test" "jupyter"
    "unauthorized_operation" "active-diagnostic" t "rejection-only"
    '("buffer_name") nil '("failed")
    '("rejection_reason" "restart_attempted" "shutdown_attempted"
      "launch_attempted")
    "Negative-test operation proving kernel restarts are not implicit model-facing capabilities.")
   (soma-gym-bridge--operation-spec
    "org.observe_tree" "reports-v0" "stable" "org"
    "passive_observation" "derived" t "org-structure-read"
    '("buffer_name") nil '("succeeded" "failed")
    '("headings" "ids" "properties" "source_blocks")
    "Observe Org heading, property, source-block, and result structure.")
   (soma-gym-bridge--operation-spec
    "org.update_heading" "reports-v0" "stable" "org"
    "reversible_editor_edit" "active-diagnostic" t "org-update-heading"
    '("buffer_name" "heading" "changes")
    '("org_node" "structure_fingerprint")
    '("succeeded" "failed")
    '("parent_parse" "preserved_ids_properties_children")
    "Update an Org heading while preserving structural metadata.")
   (soma-gym-bridge--operation-spec
    "org.edit_src_block" "reports-v0" "stable" "org"
    "editor_state_change" "active-diagnostic" t "org-edit-src-code"
    '("buffer_name" "source_block")
    '("buffer_name" "text_owner_name" "buffer_file_name" "chars_tick"
      "point" "modified" "read_only" "minibuffer_active" "mark_active"
      "narrowed" "org_node" "structure_fingerprint" "source_block")
    '("succeeded" "failed")
    '("source_edit_buffer" "parent_markers" "parent_parse")
    "Open or update an Org source-edit buffer with parent ownership evidence.")
   (soma-gym-bridge--operation-spec
    "org.exit_src_edit" "reports-v0" "stable" "org"
    "reversible_editor_edit" "active-diagnostic" t "org-edit-src-exit"
    '("edit_buffer_name")
    '("edit_buffer_name" "edit_buffer_chars_tick" "edit_text_sha256"
      "parent_buffer_name" "parent_text_owner_name" "parent_file_name"
      "parent_chars_tick" "parent_structure_fingerprint"
      "parent_markers" "source_block")
    '("succeeded" "failed")
    '("parent_buffer_writeback" "parent_parse")
    "Exit Org source editing and prove parent-buffer writeback.")
   (soma-gym-bridge--operation-spec
    "org.execute_babel_block" "reports-v0" "stable" "org"
    "external_process_or_kernel_execution" "active-diagnostic" t
    "org-babel-execute-src-block"
    '("buffer_name" "source_block")
    '("buffer_name" "text_owner_name" "buffer_file_name" "chars_tick"
      "point" "modified" "read_only" "minibuffer_active" "mark_active"
      "narrowed" "org_node" "structure_fingerprint" "source_block"
      "language_allowlist" "session" "default_directory")
    '("succeeded" "failed" "outcome_unknown")
    '("precondition_checks" "parent_result_block" "runtime_identity")
    "Execute a selected Org Babel block through Org with runtime and parent-result evidence.")
   (soma-gym-bridge--operation-spec
    "org.export_html" "reports-v0" "stable" "org"
    "filesystem_write" "active-diagnostic" t "org-export"
    '("buffer_name" "output_path")
    '("buffer_name" "chars_tick" "publish_directory" "export_options")
    '("succeeded" "failed")
    '("export_preview" "generated_files" "asset_policy" "export_diagnostics")
    "Preview or apply an Org HTML export with explicit output and asset policy.")
   (soma-gym-bridge--operation-spec
    "org.parent_text_write" "harness-debug" "negative-test" "org"
    "unauthorized_operation" "active-diagnostic" t "rejection-only"
    '("buffer_name" "source_block" "text") nil '("failed")
    '("rejection_reason" "direct_parent_text_write_attempted"
      "source_edit_bypassed" "written_bytes")
    "Negative-test operation proving direct parent-buffer source-block text writes are not a model-facing capability.")
   (soma-gym-bridge--operation-spec
    "magit.observe_sections" "vcs-files-v0" "stable" "magit"
    "passive_observation" "derived" t "magit-section-read"
    '("buffer_name") nil '("succeeded" "failed")
    '("repo_fingerprint" "status_generation" "selected_section")
    "Observe Magit sections, selection, region, and repository fingerprint.")
   (soma-gym-bridge--operation-spec
    "magit.stage_hunk" "vcs-files-v0" "stable" "magit"
    "git_index_write" "active-diagnostic" t "magit-stage"
    '("buffer_name" "section")
    '("repo_fingerprint" "section_identity" "this_command")
    '("succeeded" "failed" "outcome_unknown")
    '("index_diff" "worktree_diff" "post_stage_hook")
    "Stage one Magit hunk through package-owned command context.")
   (soma-gym-bridge--operation-spec
    "magit.stage_file" "vcs-files-v0" "stable" "magit"
    "git_index_write" "active-diagnostic" t "magit-stage"
    '("buffer_name" "section")
    '("repo_fingerprint" "section_identity" "this_command")
    '("succeeded" "failed" "outcome_unknown")
    '("index_diff" "worktree_diff" "post_stage_hook")
    "Stage one Magit file section through package-owned command context.")
   (soma-gym-bridge--operation-spec
    "dired.observe" "vcs-files-v0" "stable" "dired"
    "passive_observation" "derived" t "dired-state-read"
    '("buffer_name") nil '("succeeded" "failed")
    '("directory" "marks" "deletion_flags" "file_signatures")
    "Observe Dired marks, flags, directory state, and filesystem signatures.")
   (soma-gym-bridge--operation-spec
    "dired.rename_no_overwrite" "vcs-files-v0" "stable" "dired"
    "filesystem_write" "active-diagnostic" t "dired-rename-file"
    '("buffer_name" "source" "target")
    '("buffer_name" "directory" "file_signature" "target_absent"
      "collision_policy")
    '("succeeded" "failed")
    '("pre_filesystem_state" "post_filesystem_state")
    "Rename through Dired without overwriting an existing target.")
   (soma-gym-bridge--operation-spec
    "dired.delete_marked" "vcs-files-v0" "stable" "dired"
    "filesystem_write" "active-diagnostic" t "dired-do-delete"
    '("buffer_name")
    '("buffer_name" "directory" "marked_files" "file_signatures"
      "delete_policy")
    '("succeeded" "failed" "outcome_unknown")
    '("pre_filesystem_state" "post_filesystem_state")
    "Delete Dired-marked files with independent filesystem evidence.")
   (soma-gym-bridge--operation-spec
    "filesystem.rename" "harness-debug" "negative-test" "filesystem"
    "unauthorized_operation" "active-diagnostic" t "rejection-only"
    '("source_path" "target_path") nil '("failed")
    '("rejection_reason" "used_raw_filesystem_rename" "renamed_paths")
    "Negative-test operation proving raw filesystem renames are not a model-facing capability.")
   (soma-gym-bridge--operation-spec
    "emacs.goto_char" "core-v0" "experimental" "core-emacs-buffer"
    "editor_state_change" "active-diagnostic" t "checked-point-move"
    '("buffer_name" "point")
    '("buffer_name" "chars_tick" "selected_window_buffer")
    '("succeeded" "failed")
    '("precondition_checks" "before_after_observations")
    "Move point in the selected buffer after freshness checks.")
   (soma-gym-bridge--operation-spec
    "command.dispatch" "core-v0" "experimental" "core-emacs-command-loop"
    "capability_escape_hatch" "active-diagnostic" t "command-execute"
    '("buffer_name" "command")
    '("buffer_name" "selected_window_buffer" "chars_tick" "point"
      "minibuffer_active")
    '("running" "succeeded" "failed" "waiting_for_input"
      "outcome_unknown")
    '("route" "this_command" "precondition_checks" "package_evidence")
    "Run an allowlisted interactive command when command context matters.")
   (soma-gym-bridge--operation-spec
    "episode.finish" "benchmark-only" "benchmark-only" "benchmark-grader"
    "private_grader" "active-diagnostic" t "benchmark-helper"
    '("required_requests" "final_buffer_checks")
    nil '("succeeded" "failed")
    '("request_checks" "buffer_checks")
    "Benchmark episode completion helper; not a normal agent capability.")
   (soma-gym-bridge--operation-spec
    "bridge.defer_echo" "benchmark-only" "benchmark-only" "bridge"
    "async_fixture" "active-diagnostic" t "timer-fixture"
    '("value") nil '("running" "succeeded")
    '("delay_seconds" "completed_after_seconds")
    "Benchmark fixture for async request status and await behavior.")
   (soma-gym-bridge--operation-spec
    "bridge.prompt_echo" "benchmark-only" "benchmark-only" "bridge"
    "prompt_fixture" "active-diagnostic" t "prompt-fixture"
    '("prompt") nil '("waiting_for_input" "succeeded" "failed")
    '("prompt" "request_id" "prompt_response")
    "Benchmark fixture for explicit waiting-for-input lifecycle.")
   (soma-gym-bridge--operation-spec
    "direct-call" "harness-debug" "negative-test" "bridge"
    "unauthorized_operation" "active-diagnostic" t "rejection-only"
    '("requested_operation") nil '("failed")
    '("rejection_reason")
    "Negative-test operation proving unchecked direct calls are rejected.")
   (soma-gym-bridge--operation-spec
    "elisp.eval" "harness-debug" "negative-test" "bridge"
    "unauthorized_operation" "active-diagnostic" t "rejection-only"
    '("form") nil '("failed")
    '("rejection_reason" "evaluated")
    "Negative-test operation proving arbitrary Elisp evaluation is not a model-facing capability.")
   (soma-gym-bridge--operation-spec
    "process.write" "harness-debug" "negative-test" "bridge"
    "unauthorized_operation" "active-diagnostic" t "rejection-only"
    '("buffer_name" "input") nil '("failed")
    '("rejection_reason" "used_raw_process_write" "sent_bytes")
    "Negative-test operation proving raw process writes are not a model-facing capability.")
   (soma-gym-bridge--operation-spec
    "git.index_write" "harness-debug" "negative-test" "bridge"
    "unauthorized_operation" "active-diagnostic" t "rejection-only"
    '("repo" "pathspec") nil '("failed")
    '("rejection_reason" "used_raw_git_index_write" "staged_path_count")
    "Negative-test operation proving raw Git index writes are not a model-facing capability.")))

(defun soma-gym-bridge-list-capabilities ()
  "Return the model-facing operation manifest for this bridge profile."
  (let* ((operations (soma-gym-bridge--operation-catalog))
         (profiles (delete-dups
                    (mapcar
                     (lambda (entry)
                       (soma-gym-bridge--get entry "profile"))
                     operations))))
    (soma-gym-bridge--alist
     (cons "operation" "api.list_capabilities")
     (cons "state" "succeeded")
     (cons "api_version" soma-gym-bridge-api-version)
     (cons "profiles"
           '("core-v0" "coding-v0" "terminal-v0" "analysis-v0"
             "reports-v0" "vcs-files-v0"))
     (cons "all_profiles" profiles)
     (cons "operation_allowlist"
           (or soma-gym-bridge-operation-allowlist :json-false))
     (cons "operations" operations)
     (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
     (cons "insula_mode" (or (getenv "SOMA_INSULA_MODE") ""))
     (cons "insula_plan" (or (getenv "SOMA_INSULA_PLAN") "")))))

(defun soma-gym-bridge-describe-operation (operation)
  "Return the manifest entry for OPERATION, or nil if unknown."
  (and (stringp operation)
       (seq-find
        (lambda (entry)
          (string= (soma-gym-bridge--get entry "operation") operation))
        (soma-gym-bridge--operation-catalog))))

(defun soma-gym-bridge--object-alist (object)
  "Return OBJECT as a string-keyed alist, or :invalid for non-objects."
  (cond
   ((null object)
    nil)
   ((hash-table-p object)
    (soma-gym-bridge--jsonish object))
   ((and (listp object)
         (keywordp (car object)))
    (let (items)
      (while object
        (push (cons (soma-gym-bridge--key-string (car object))
                    (cadr object))
              items)
        (setq object (cddr object)))
      (nreverse items)))
   ((and (listp object)
         (cl-every #'consp object))
    object)
   (t
    :invalid)))

(defun soma-gym-bridge--alist-set (alist key value)
  "Return ALIST with string KEY set to VALUE."
  (cons (cons key value)
        (cl-remove-if
         (lambda (entry)
           (string= (soma-gym-bridge--key-string (car entry)) key))
         alist)))

(defun soma-gym-bridge--alist-remove-keys (alist keys)
  "Return ALIST without any string keys in KEYS."
  (cl-remove-if
   (lambda (entry)
     (member (soma-gym-bridge--key-string (car entry)) keys))
   (or alist nil)))

(defun soma-gym-bridge--org-parent-buffer-preconditions (preconditions)
  "Return Org parent-buffer ownership PRECONDITIONS.

Org report operations resolve and mutate an explicit parent buffer, Org node,
and source block.  A currently selected window is presentation state, so a stale
or unrelated selected window must not veto these semantic buffer operations."
  (soma-gym-bridge--alist-remove-keys
   preconditions
   '("selected_window_buffer")))

(defun soma-gym-bridge--request-validation-error (request-id operation code message)
  "Return a failed validation response for REQUEST-ID and OPERATION."
  (soma-gym-bridge--response
   request-id operation "failed" '("accepted" "failed")
   nil code message))

(defun soma-gym-bridge--missing-keys (object keys)
  "Return manifest-declared KEYS missing from OBJECT."
  (cl-remove-if
   (lambda (key)
     (soma-gym-bridge--has-key-p object key))
   (or keys nil)))

(defun soma-gym-bridge--validate-request-envelope (request-id operation request)
  "Validate and normalize REQUEST's v0 transport envelope.

Return (ok . PARAMS) on success or (error . RESPONSE) on validation failure."
  (let* ((api-version (soma-gym-bridge--get request "api_version"))
         (capability (soma-gym-bridge--get request "capability"))
         (raw-params (soma-gym-bridge--get request "params"))
         (params (soma-gym-bridge--object-alist raw-params))
         (has-top-preconditions
          (soma-gym-bridge--has-key-p request "preconditions"))
         (top-preconditions
          (and has-top-preconditions
               (soma-gym-bridge--object-alist
                (soma-gym-bridge--get request "preconditions"))))
         (has-timeout-ms
          (soma-gym-bridge--has-key-p request "timeout_ms"))
         (timeout-ms (and has-timeout-ms
                          (soma-gym-bridge--get request "timeout_ms")))
         (operation-spec (soma-gym-bridge-describe-operation operation))
         (side-effect-class
          (and operation-spec
               (soma-gym-bridge--get operation-spec "side_effect_class")))
         (maturity
          (and operation-spec
               (soma-gym-bridge--get operation-spec "maturity")))
         (profile
          (and operation-spec
               (soma-gym-bridge--get operation-spec "profile")))
         (param-preconditions nil)
         (missing-params nil)
         (missing-preconditions nil))
    (cond
     ((and api-version
           (not (and (stringp api-version)
                     (string= api-version soma-gym-bridge-api-version))))
      (cons
       'error
       (soma-gym-bridge--request-validation-error
        request-id operation "invalid_api_version"
        "api_version is not supported by this bridge")))
     ((and api-version
           operation-spec
           (or (string= (or maturity "") "benchmark-only")
               (string= (or profile "") "benchmark-only")))
      (cons
       'error
       (soma-gym-bridge--request-validation-error
        request-id operation "benchmark_only_operation"
        "benchmark-only operations are not available through the stable v0 API envelope")))
     ((and api-version
           operation-spec
           (member (or maturity "") '("experimental" "planned")))
      (cons
       'error
       (soma-gym-bridge--request-validation-error
        request-id operation "non_stable_operation"
        "experimental and planned operations are not available through the stable v0 API envelope")))
     ((eq params :invalid)
      (cons
       'error
       (soma-gym-bridge--request-validation-error
        request-id operation "invalid_params"
        "params must be a JSON object when provided")))
     ((eq top-preconditions :invalid)
      (cons
       'error
       (soma-gym-bridge--request-validation-error
        request-id operation "invalid_preconditions"
        "preconditions must be a JSON object when provided")))
     ((and capability
           (not (stringp capability)))
      (cons
       'error
       (soma-gym-bridge--request-validation-error
        request-id operation "invalid_capability"
        "capability must be a string when provided")))
     ((and (stringp capability)
           side-effect-class
           (not (string= capability side-effect-class)))
      (cons
       'error
       (soma-gym-bridge--request-validation-error
        request-id operation "capability_mismatch"
        "capability does not match the operation side-effect class")))
     ((and has-timeout-ms
           (not (and (numberp timeout-ms)
                     (>= timeout-ms 0)
                     (<= timeout-ms 120000))))
      (cons
       'error
       (soma-gym-bridge--request-validation-error
        request-id operation "invalid_timeout_ms"
        "timeout_ms must be a number between 0 and 120000")))
     (t
      (when (and (not (eq params :invalid))
                 (soma-gym-bridge--has-key-p params "preconditions"))
        (setq param-preconditions
              (soma-gym-bridge--object-alist
               (soma-gym-bridge--get params "preconditions"))))
      (cond
       ((eq param-preconditions :invalid)
        (cons
         'error
         (soma-gym-bridge--request-validation-error
          request-id operation "invalid_preconditions"
          "params.preconditions must be a JSON object when provided")))
       ((and has-top-preconditions
             (soma-gym-bridge--has-key-p params "preconditions")
             (not (equal top-preconditions param-preconditions)))
        (cons
         'error
         (soma-gym-bridge--request-validation-error
          request-id operation "precondition_mismatch"
          "top-level preconditions and params.preconditions differ")))
       (t
        (when has-top-preconditions
          (setq params
                (soma-gym-bridge--alist-set
                 params "preconditions" top-preconditions)))
        (when has-timeout-ms
          (setq params
                (soma-gym-bridge--alist-set
                 params "timeout_seconds" (/ (float timeout-ms) 1000.0))))
        (when (and api-version operation-spec)
          (setq missing-params
                (soma-gym-bridge--missing-keys
                 params
                 (soma-gym-bridge--get operation-spec "required_params")))
          (setq param-preconditions
                (soma-gym-bridge--object-alist
                 (soma-gym-bridge--get params "preconditions")))
          (setq missing-preconditions
                (soma-gym-bridge--missing-keys
                 param-preconditions
                 (soma-gym-bridge--get operation-spec
                                       "required_preconditions"))))
        (cond
         (missing-params
          (cons
           'error
           (soma-gym-bridge--request-validation-error
            request-id operation "missing_required_param"
            (format "missing required params: %s"
                    (string-join missing-params ", ")))))
         (missing-preconditions
          (cons
           'error
           (soma-gym-bridge--request-validation-error
            request-id operation "missing_required_precondition"
            (format "missing required preconditions: %s"
                    (string-join missing-preconditions ", ")))))
         (t
          (cons 'ok params)))))))))

(defun soma-gym-bridge--json-symbol (value)
  (cond
   ((symbolp value) (symbol-name value))
   ((stringp value) value)
   (t (format "%S" value))))

(defun soma-gym-bridge--safe-slot-json-symbol (object slot)
  (condition-case nil
      (if (and object
               (fboundp 'slot-boundp)
               (fboundp 'eieio-oref)
               (slot-boundp object slot))
          (soma-gym-bridge--json-symbol (eieio-oref object slot))
        :json-false)
    (error :json-false)))

(defun soma-gym-bridge--safe-slot-json-value (object slot)
  (condition-case nil
      (if (and object
               (fboundp 'slot-boundp)
               (fboundp 'eieio-oref)
               (slot-boundp object slot))
          (soma-gym-bridge--json-safe-value (eieio-oref object slot))
        :json-false)
    (error :json-false)))

(defun soma-gym-bridge--json-safe-value (value)
  (cond
   ((eq value nil) :json-false)
   ((eq value t) t)
   ((eq value :json-false) :json-false)
   ((or (stringp value) (numberp value)) value)
   ((symbolp value) (symbol-name value))
   ((markerp value) (or (marker-position value) :json-false))
   ((vectorp value)
    (mapcar #'soma-gym-bridge--json-safe-value (append value nil)))
   ((listp value)
    (mapcar #'soma-gym-bridge--json-safe-value value))
   (t (format "%S" value))))

(defun soma-gym-bridge--transient-active-prefix ()
  (condition-case nil
      (cond
       ((fboundp 'transient-active-prefix)
        (transient-active-prefix))
       ((and (boundp 'transient--prefix) transient--prefix)
        transient--prefix)
       (t nil))
    (error nil)))

(defun soma-gym-bridge--transient-suffix-observation (object index)
  (soma-gym-bridge--alist
   (cons "index" index)
   (cons "class"
         (condition-case nil
             (if (fboundp 'eieio-object-class)
                 (soma-gym-bridge--json-symbol (eieio-object-class object))
               :json-false)
           (error :json-false)))
   (cons "key" (soma-gym-bridge--safe-slot-json-value object 'key))
   (cons "command" (soma-gym-bridge--safe-slot-json-symbol object 'command))
   (cons "description"
         (soma-gym-bridge--safe-slot-json-value object 'description))
   (cons "argument"
         (soma-gym-bridge--safe-slot-json-value object 'argument))
   (cons "shortarg"
         (soma-gym-bridge--safe-slot-json-value object 'shortarg))
   (cons "value" (soma-gym-bridge--safe-slot-json-value object 'value))
   (cons "inactive"
         (soma-gym-bridge--safe-slot-json-value object 'inactive))
   (cons "inapt"
         (soma-gym-bridge--safe-slot-json-value object 'inapt))))

(defun soma-gym-bridge--transient-suffixes-observation ()
  (condition-case nil
      (if (and (soma-gym-bridge--transient-active-prefix)
               (boundp 'transient--suffixes)
               (listp transient--suffixes))
          (let ((index 0)
                suffixes)
            (dolist (object transient--suffixes)
              (cl-incf index)
              (push (soma-gym-bridge--transient-suffix-observation
                     object index)
                    suffixes))
            (nreverse suffixes))
        nil)
    (error nil)))

(defun soma-gym-bridge--transient-active-arguments ()
  (condition-case nil
      (if (and (soma-gym-bridge--transient-active-prefix)
               (boundp 'transient--suffixes)
               transient--suffixes
               (fboundp 'transient-get-value))
          (soma-gym-bridge--json-safe-value (transient-get-value))
        nil)
    (error nil)))

(defun soma-gym-bridge--number-param (params key default minimum maximum)
  (let ((value (soma-gym-bridge--get params key)))
    (when (eq value :json-false)
      (setq value nil))
    (unless (numberp value)
      (setq value default))
    (max minimum (min maximum (float value)))))

(defun soma-gym-bridge--terminal-state-p (state)
  (member state '("succeeded" "failed" "outcome_unknown")))

(defun soma-gym-bridge--publish-request-response (request-id response)
  "Store RESPONSE for REQUEST-ID without regressing a terminal request.

Some async bridge operations publish their own request state while Emacs is
reentered at a wait boundary.  If that happens before the outer transport
handler returns its initial running response, the terminal state is the
authoritative request-store state."
  (let ((current (and (stringp request-id)
                      (gethash request-id soma-gym-bridge--requests))))
    (unless (and current
                 (soma-gym-bridge--terminal-state-p
                  (soma-gym-bridge--get current "state"))
                 (not (soma-gym-bridge--terminal-state-p
                       (soma-gym-bridge--get response "state"))))
      (puthash request-id response soma-gym-bridge--requests)))
  response)

(defun soma-gym-bridge--insula-plan ()
  "Return the selected Insula plan as an alist, or nil if unavailable."
  (let ((path (getenv "SOMA_INSULA_PLAN")))
    (ignore-errors
      (soma-gym-bridge--json-read-file path))))

(defun soma-gym-bridge--path-directory-or-false (path)
  "Return PATH as a directory name when present, otherwise JSON false."
  (if (and (stringp path) (not (string-empty-p path)))
      (file-name-as-directory path)
    :json-false))

(defun soma-gym-bridge--plan-string (plan key)
  "Return string KEY from Insula PLAN, or the empty string."
  (let ((value (and (listp plan)
                    (soma-gym-bridge--get plan key))))
    (if (stringp value) value "")))

(defun soma-gym-bridge--rootfs-store-id (plan rootfs-path)
  "Return the selected rootfs store id from PLAN or ROOTFS-PATH."
  (let ((explicit (soma-gym-bridge--plan-string plan "rootfs_store_id")))
    (cond
     ((not (string-empty-p explicit))
      explicit)
     ((and (stringp rootfs-path)
           (not (string-empty-p rootfs-path)))
      (file-name-nondirectory (directory-file-name rootfs-path)))
     (t ""))))

(defun soma-gym-bridge--feature-names ()
  "Return loaded feature names as strings."
  (sort (mapcar #'soma-gym-bridge--json-symbol features) #'string<))

(defun soma-gym-bridge--doom-modules-json ()
  "Return loaded Doom module state when the current session exposes it."
  (if (and (boundp 'doom-modules) (listp doom-modules))
      (soma-gym-bridge--jsonish doom-modules)
    :json-false))

(defun soma-gym-bridge--selected-frame-session-observation ()
  "Return selected frame/window/buffer identity for session anchoring."
  (let* ((frames (frame-list))
         (frame (selected-frame))
         (frame-index (or (cl-position frame frames :test #'eq) 0))
         (window (selected-window))
         (buffer (and (window-live-p window)
                      (window-buffer window)))
         (window-id (cond
                     ((and (fboundp 'window-id)
                           (window-live-p window))
                      (window-id window))
                     ((window-live-p window)
                      frame-index)
                     (t :json-false))))
    (soma-gym-bridge--alist
     (cons "frame_id" (1+ frame-index))
     (cons "window_id" window-id)
     (cons "frame_name" (or (frame-parameter frame 'name) :json-false))
     (cons "terminal_name"
           (or (condition-case nil
                   (terminal-name (frame-terminal frame))
                 (error nil))
               :json-false))
     (cons "client_frame"
           (soma-gym-bridge--bool (frame-parameter frame 'client)))
     (cons "buffer_name" (if buffer (buffer-name buffer) :json-false)))))

(defun soma-gym-bridge-observe-session ()
  "Return passive Emacs, Doom, rootfs, and selected-frame identity."
  (let* ((plan (soma-gym-bridge--insula-plan))
         (rootfs-path (soma-gym-bridge--plan-string plan "rootfs"))
         (doom-emacs-dir
          (or (and (listp plan)
                   (soma-gym-bridge--get plan "doom_core_readonly"))
              (getenv "EMACSDIR")
              "/project/home/.emacs.d"))
         (doom-dir
          (or (and (listp plan)
                   (soma-gym-bridge--get plan "doom_user_readonly"))
              (getenv "DOOMDIR")
              "/project/home/.doom.d"))
         (editor-executable
          (let ((from-plan (soma-gym-bridge--plan-string plan "emacs")))
            (if (not (string-empty-p from-plan))
                from-plan
              (expand-file-name invocation-name invocation-directory))))
         (profiles
          (delete-dups
           (mapcar
            (lambda (entry)
              (soma-gym-bridge--get entry "profile"))
            (soma-gym-bridge--operation-catalog)))))
    (soma-gym-bridge--alist
     (cons "operation" "emacs.observe_session")
     (cons "state" "succeeded")
     (cons "session"
           (soma-gym-bridge--alist
            (cons "api_version" soma-gym-bridge-api-version)
            (cons "emacs_pid" (emacs-pid))
            (cons "daemonp" (soma-gym-bridge--bool (daemonp)))
            (cons "server_name" (if (boundp 'server-name)
                                    server-name
                                  :json-false))
            (cons "system_type" (soma-gym-bridge--json-symbol system-type))
            (cons "system_configuration" system-configuration)
            (cons "profiles" profiles)))
     (cons "rootfs"
           (soma-gym-bridge--alist
            (cons "mode" (or (getenv "SOMA_INSULA_MODE") ""))
            (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
            (cons "plan_path" (or (getenv "SOMA_INSULA_PLAN") ""))
            (cons "rootfs_path" rootfs-path)
            (cons "rootfs_store_id"
                  (soma-gym-bridge--rootfs-store-id plan rootfs-path))
            (cons "rootfs_contract"
                  (soma-gym-bridge--plan-string plan "rootfs_contract"))
            (cons "dependency_closure"
                  (soma-gym-bridge--plan-string plan "dependency_closure"))
            (cons "glibc_owner"
                  (soma-gym-bridge--plan-string plan "glibc_owner"))
            (cons "base_image"
                  (soma-gym-bridge--plan-string plan "base_image"))
            (cons "image_id"
                  (soma-gym-bridge--plan-string plan "image_id"))
            (cons "rootfs_recipe_sha256"
                  (soma-gym-bridge--plan-string
                   plan "rootfs_recipe_sha256"))))
     (cons "editor"
           (soma-gym-bridge--alist
            (cons "version" (emacs-version))
            (cons "executable" editor-executable)
            (cons "source_directory"
                  (soma-gym-bridge--path-directory-or-false
                   source-directory))
            (cons "invocation_name" invocation-name)
            (cons "invocation_directory"
                  (soma-gym-bridge--path-directory-or-false
                   invocation-directory))
            (cons "features" system-configuration-features)
            (cons "native_comp_available"
                  (soma-gym-bridge--bool
                   (and (fboundp 'native-comp-available-p)
                        (native-comp-available-p))))))
     (cons "doom"
           (soma-gym-bridge--alist
            (cons "emacs_dir"
                  (soma-gym-bridge--path-directory-or-false doom-emacs-dir))
            (cons "doom_dir"
                  (soma-gym-bridge--path-directory-or-false doom-dir))
            (cons "emacs_dir_exists"
                  (soma-gym-bridge--bool (file-directory-p doom-emacs-dir)))
            (cons "doom_dir_exists"
                  (soma-gym-bridge--bool (file-directory-p doom-dir)))
            (cons "commit" (soma-gym-bridge--plan-string plan "doom_commit"))
            (cons "modules" (soma-gym-bridge--doom-modules-json))
            (cons "loaded_features" (soma-gym-bridge--feature-names))))
     (cons "selected_frame"
           (soma-gym-bridge--selected-frame-session-observation)))))

(defun soma-gym-bridge--frame-observation (frame index)
  (let* ((terminal (frame-terminal frame))
         (window (frame-selected-window frame))
         (buffer (and (window-live-p window)
                      (window-buffer window)))
         (client (frame-parameter frame 'client)))
    (soma-gym-bridge--alist
     (cons "index" index)
     (cons "name" (or (frame-parameter frame 'name) :json-false))
     (cons "selected" (soma-gym-bridge--bool
                       (eq frame (selected-frame))))
     (cons "graphic" (soma-gym-bridge--bool
                      (display-graphic-p frame)))
     (cons "terminal_live" (soma-gym-bridge--bool
                            (terminal-live-p terminal)))
     (cons "terminal_name"
           (or (condition-case nil
                   (terminal-name terminal)
                 (error nil))
               :json-false))
     (cons "visible"
           (soma-gym-bridge--json-symbol (frame-visible-p frame)))
     (cons "client_frame" (soma-gym-bridge--bool client))
     (cons "client_repr" (if client (format "%S" client) :json-false))
     (cons "selected_window_live" (soma-gym-bridge--bool
                                   (window-live-p window)))
     (cons "selected_window_buffer"
           (if buffer (buffer-name buffer) :json-false))
     (cons "width" (frame-width frame))
     (cons "height" (frame-height frame)))))

(defun soma-gym-bridge-observe-frames ()
  "Return daemon-local frame ownership and terminal-frame evidence."
  (let ((frames nil)
        (index 0)
        (terminal-count 0)
        (graphic-count 0)
        (client-count 0))
    (dolist (frame (frame-list))
      (cl-incf index)
      (unless (display-graphic-p frame)
        (cl-incf terminal-count))
      (when (display-graphic-p frame)
        (cl-incf graphic-count))
      (when (frame-parameter frame 'client)
        (cl-incf client-count))
      (push (soma-gym-bridge--frame-observation frame index)
            frames))
    (soma-gym-bridge--alist
     (cons "operation" "emacs.observe_frames")
     (cons "state" "succeeded")
     (cons "daemonp" (soma-gym-bridge--bool (daemonp)))
     (cons "server_name" (if (boundp 'server-name)
                             server-name
                           :json-false))
     (cons "emacs_pid" (emacs-pid))
     (cons "frame_count" (length frames))
     (cons "terminal_frame_count" terminal-count)
     (cons "graphic_frame_count" graphic-count)
     (cons "client_frame_count" client-count)
     (cons "frames" (nreverse frames)))))

(defun soma-gym-bridge-observe-interaction ()
  "Return current command-loop and minibuffer interaction state."
  (let* ((selected-buffer
          (and (window-live-p (selected-window))
               (window-buffer (selected-window))))
         (minibuffer-window (active-minibuffer-window))
         (minibuffer-buffer
          (and minibuffer-window
               (window-buffer minibuffer-window)))
         (minibuffer-prompt
          (if (buffer-live-p minibuffer-buffer)
              (condition-case nil
                  (with-current-buffer minibuffer-buffer
                    (minibuffer-prompt))
                (error :json-false))
            :json-false))
         (minibuffer-contents
          (if (buffer-live-p minibuffer-buffer)
              (condition-case nil
                  (with-current-buffer minibuffer-buffer
                    (minibuffer-contents-no-properties))
                (error :json-false))
            :json-false))
         (transient-prefix
          (and (boundp 'transient--prefix) transient--prefix))
         (transient-map
          (and (boundp 'transient--transient-map) transient--transient-map))
         (transient-window
          (and (boundp 'transient--window) transient--window))
         (transient-buffer
          (and (boundp 'transient--buffer) transient--buffer))
         (transient-current
          (and (boundp 'transient-current-command) transient-current-command))
         (transient-buffer-name
          (if (buffer-live-p transient-buffer)
              (buffer-name transient-buffer)
            :json-false))
         (transient-window-buffer
          (if (window-live-p transient-window)
              (buffer-name (window-buffer transient-window))
            :json-false))
         (transient-map-installed
          (and transient-map
               (boundp 'overriding-terminal-local-map)
               (memq transient-map overriding-terminal-local-map))))
    (soma-gym-bridge--alist
     (cons "operation" "emacs.observe_interaction")
     (cons "state" "succeeded")
     (cons "daemonp" (soma-gym-bridge--bool (daemonp)))
     (cons "server_name" (if (boundp 'server-name)
                             server-name
                           :json-false))
     (cons "emacs_pid" (emacs-pid))
     (cons "selected_window_buffer"
           (if selected-buffer
               (buffer-name selected-buffer)
             :json-false))
     (cons "minibuffer_active"
           (soma-gym-bridge--bool minibuffer-window))
     (cons "minibuffer_buffer"
           (if (buffer-live-p minibuffer-buffer)
               (buffer-name minibuffer-buffer)
             :json-false))
     (cons "minibuffer_prompt" minibuffer-prompt)
     (cons "minibuffer_contents" minibuffer-contents)
     (cons "recursive_edit_depth" (recursion-depth))
     (cons "transient_active"
           (soma-gym-bridge--bool
            (or transient-prefix transient-map transient-window)))
     (cons "transient_current_command"
           (if transient-current
               (soma-gym-bridge--json-symbol transient-current)
             :json-false))
     (cons "transient_prefix_command"
           (soma-gym-bridge--safe-slot-json-symbol transient-prefix 'command))
     (cons "transient_map_active"
           (soma-gym-bridge--bool transient-map))
     (cons "transient_map_installed"
           (soma-gym-bridge--bool transient-map-installed))
     (cons "transient_buffer"
           transient-buffer-name)
     (cons "transient_window_live"
           (soma-gym-bridge--bool (window-live-p transient-window)))
     (cons "transient_window_buffer"
           transient-window-buffer)
     (cons "transient_active_arguments"
           (or (soma-gym-bridge--transient-active-arguments) []))
     (cons "transient_suffixes"
           (or (soma-gym-bridge--transient-suffixes-observation) []))
     (cons "this_command" (soma-gym-bridge--json-symbol this-command))
     (cons "last_command" (soma-gym-bridge--json-symbol last-command)))))

(defun soma-gym-bridge--object-or-empty (object)
  "Return OBJECT when it has fields, otherwise a fresh empty JSON object."
  (if (and (listp object) object)
      object
    (soma-gym-bridge--empty-object)))

(defun soma-gym-bridge--result-field-present-p (result key)
  "Return non-nil when RESULT contains KEY."
  (and (listp result)
       (soma-gym-bridge--has-key-p result key)))

(defun soma-gym-bridge--result-field (result key)
  "Return KEY from RESULT."
  (and (listp result)
       (soma-gym-bridge--get result key)))

(defun soma-gym-bridge--edit-changed-range-effect (operation result)
  "Return a typed changed-range effect extracted from RESULT."
  (when (and (string= operation "buffer.replace_range")
             (soma-gym-bridge--result-field-present-p result "start")
             (soma-gym-bridge--result-field-present-p result "end"))
    (let ((effect
           (soma-gym-bridge--alist
            (cons "start" (soma-gym-bridge--result-field result "start"))
            (cons "end" (soma-gym-bridge--result-field result "end")))))
      (when (soma-gym-bridge--result-field-present-p
             result "replacement_chars")
        (setq effect
              (append
               effect
               (list
                (cons "replacement_chars"
                      (soma-gym-bridge--result-field
                       result "replacement_chars"))))))
      effect)))

(defun soma-gym-bridge--jupyter-runtime-evidence (observation)
  "Return rootfs/runtime identity from a Jupyter OBSERVATION."
  (soma-gym-bridge--alist
   (cons "run_id"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation "run_id"))
             :json-false))
   (cons "plan"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation "plan"))
             :json-false))))

(defun soma-gym-bridge--jupyter-source-buffer-evidence (observation)
  "Return source-buffer identity evidence from a Jupyter OBSERVATION."
  (soma-gym-bridge--alist
   (cons "buffer_name"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation "buffer_name"))
             :json-false))
   (cons "text_owner_name"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation "text_owner_name"))
             :json-false))
   (cons "buffer_file_name"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation "buffer_file_name"))
             :json-false))
   (cons "major_mode"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation "major_mode"))
             :json-false))
   (cons "chars_tick"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation "chars_tick"))
             :json-false))
   (cons "selected_window_buffer"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "selected_window_buffer"))
             :json-false))))

(defun soma-gym-bridge--jupyter-client-evidence (observation)
  "Return client association evidence from a Jupyter OBSERVATION."
  (soma-gym-bridge--alist
   (cons "associated"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "jupyter_client_associated"))
             :json-false))
   (cons "class"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "jupyter_client_class"))
             :json-false))
   (cons "session_id"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "jupyter_client_session_id"))
             :json-false))
   (cons "repl_buffer"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "jupyter_repl_buffer"))
             :json-false))
   (cons "repl_interaction_mode"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "jupyter_repl_interaction_mode"))
             :json-false))))

(defun soma-gym-bridge--jupyter-kernel-evidence (observation)
  "Return kernel incarnation evidence from a Jupyter OBSERVATION."
  (soma-gym-bridge--alist
   (cons "connection_file"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "jupyter_connection_file"))
             :json-false))
   (cons "kernel_token"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "jupyter_kernel_token"))
             :json-false))
   (cons "kernel_pid"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "jupyter_kernel_pid"))
             :json-false))
   (cons "kernel_exe"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation
                                        "jupyter_kernel_exe"))
             :json-false))
   (cons "connection_file_rootfs_owned"
         (or (and (listp observation)
                  (soma-gym-bridge--get
                   observation "jupyter_connection_file_rootfs_owned"))
             :json-false))
   (cons "kernel_exe_rootfs_owned"
         (or (and (listp observation)
                  (soma-gym-bridge--get
                   observation "jupyter_kernel_exe_rootfs_owned"))
             :json-false))))

(defun soma-gym-bridge--jupyter-source-observation-evidence (observation)
  "Return typed analysis-v0 source observation evidence."
  (soma-gym-bridge--alist
   (cons "source_buffer"
         (soma-gym-bridge--jupyter-source-buffer-evidence observation))
   (cons "client"
         (soma-gym-bridge--jupyter-client-evidence observation))
   (cons "kernel_incarnation"
         (soma-gym-bridge--jupyter-kernel-evidence observation))
   (cons "selected_region"
         (or (and (listp observation)
                  (soma-gym-bridge--get observation "selected_range"))
             :json-false))
   (cons "rootfs_runtime"
         (soma-gym-bridge--jupyter-runtime-evidence observation))))

(defun soma-gym-bridge--jupyter-reply-evidence (result)
  "Return Jupyter shell reply evidence from RESULT."
  (soma-gym-bridge--alist
   (cons "status"
         (or (soma-gym-bridge--result-field result "reply_status")
             :json-false))
   (cons "parent"
         (or (soma-gym-bridge--result-field result "reply_parent")
             :json-false))))

(defun soma-gym-bridge--jupyter-iopub-evidence (result)
  "Return Jupyter IOPub evidence from RESULT."
  (soma-gym-bridge--alist
   (cons "message_types"
         (or (soma-gym-bridge--result-field result "message_types")
             nil))
   (cons "parent_ids"
         (or (soma-gym-bridge--result-field result "parent_ids")
             nil))
   (cons "stream_texts"
         (or (soma-gym-bridge--result-field result "stream_texts")
             nil))
   (cons "display_events"
         (or (soma-gym-bridge--result-field result "display_events")
             nil))
   (cons "display_update_events"
         (or (soma-gym-bridge--result-field result
                                            "display_update_events")
             nil))))

(defun soma-gym-bridge--jupyter-source-region-evidence (result)
  "Return source-region evidence from a Jupyter execution RESULT."
  (let* ((before (soma-gym-bridge--result-field result "before"))
         (selected (and (listp before)
                        (soma-gym-bridge--get before "selected_range"))))
    (soma-gym-bridge--alist
     (cons "start"
           (or (soma-gym-bridge--result-field result "start")
               :json-false))
     (cons "end"
           (or (soma-gym-bridge--result-field result "end")
               :json-false))
     (cons "text_hash"
           (or (and (listp selected)
                    (soma-gym-bridge--get selected "text_hash"))
               :json-false))
     (cons "submitted_hash"
           (or (soma-gym-bridge--result-field result "submitted_hash")
               :json-false)))))

(defun soma-gym-bridge--jupyter-kernel-execution-effect (result)
  "Return a kernel-execution effect from a completed Jupyter RESULT."
  (soma-gym-bridge--alist
   (cons "route"
         (or (soma-gym-bridge--result-field result "route")
             :json-false))
   (cons "request_id"
         (or (soma-gym-bridge--result-field result "request_id")
             :json-false))
   (cons "submitted_hash"
         (or (soma-gym-bridge--result-field result "submitted_hash")
             :json-false))
   (cons "wait_timeout_seconds"
         (or (soma-gym-bridge--result-field result "wait_timeout_seconds")
             :json-false))
   (cons "idle"
         (or (soma-gym-bridge--result-field result "idle")
             :json-false))))

(defun soma-gym-bridge--result-field-or-json-false (result key)
  "Return KEY from RESULT, preserving JSON false when KEY is absent."
  (if (soma-gym-bridge--result-field-present-p result key)
      (soma-gym-bridge--result-field result key)
    :json-false))

(defun soma-gym-bridge--core-buffer-identity-evidence (result)
  "Return buffer identity evidence from an `emacs.observe_buffer' RESULT."
  (soma-gym-bridge--alist
   (cons "buffer_name"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_name"))
   (cons "buffer_file_name"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_file_name"))
   (cons "major_mode"
         (soma-gym-bridge--result-field-or-json-false
          result "major_mode"))
   (cons "selected_window_buffer"
         (soma-gym-bridge--result-field-or-json-false
          result "selected_window_buffer"))
   (cons "minibuffer_active"
         (soma-gym-bridge--result-field-or-json-false
          result "minibuffer_active"))))

(defun soma-gym-bridge--core-buffer-text-owner-evidence (result)
  "Return buffer text-owner evidence from an `emacs.observe_buffer' RESULT."
  (let ((buffer-name
         (soma-gym-bridge--result-field-or-json-false result "buffer_name"))
        (text-owner-name
         (soma-gym-bridge--result-field-or-json-false
          result "text_owner_name")))
    (soma-gym-bridge--alist
     (cons "buffer_name" buffer-name)
     (cons "text_owner_name" text-owner-name)
     (cons "buffer_file_name"
           (soma-gym-bridge--result-field-or-json-false
            result "buffer_file_name"))
     (cons "same_buffer"
           (soma-gym-bridge--bool
            (and (stringp buffer-name)
                 (stringp text-owner-name)
                 (string= buffer-name text-owner-name)))))))

(defun soma-gym-bridge--core-point-region-narrowing-evidence (result)
  "Return point, region, and narrowing evidence from buffer RESULT."
  (soma-gym-bridge--alist
   (cons "point_min"
         (soma-gym-bridge--result-field-or-json-false result "point_min"))
   (cons "point_max"
         (soma-gym-bridge--result-field-or-json-false result "point_max"))
   (cons "point"
         (soma-gym-bridge--result-field-or-json-false result "point"))
   (cons "mark_active"
         (soma-gym-bridge--result-field-or-json-false result "mark_active"))
   (cons "narrowed"
         (soma-gym-bridge--result-field-or-json-false result "narrowed"))))

(defun soma-gym-bridge--core-buffer-state-observation (result)
  "Return the agent-facing buffer-state observation from RESULT."
  (soma-gym-bridge--alist
   (cons "buffer_name"
         (soma-gym-bridge--result-field-or-json-false result "buffer_name"))
   (cons "text_owner_name"
         (soma-gym-bridge--result-field-or-json-false
          result "text_owner_name"))
   (cons "buffer_file_name"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_file_name"))
   (cons "major_mode"
         (soma-gym-bridge--result-field-or-json-false result "major_mode"))
   (cons "chars_tick"
         (soma-gym-bridge--result-field-or-json-false result "chars_tick"))
   (cons "modified"
         (soma-gym-bridge--result-field-or-json-false result "modified"))
   (cons "read_only"
         (soma-gym-bridge--result-field-or-json-false result "read_only"))
   (cons "point_min"
         (soma-gym-bridge--result-field-or-json-false result "point_min"))
   (cons "point_max"
         (soma-gym-bridge--result-field-or-json-false result "point_max"))
   (cons "point"
         (soma-gym-bridge--result-field-or-json-false result "point"))
   (cons "mark_active"
         (soma-gym-bridge--result-field-or-json-false result "mark_active"))
   (cons "narrowed"
         (soma-gym-bridge--result-field-or-json-false result "narrowed"))
   (cons "selected_window_buffer"
         (soma-gym-bridge--result-field-or-json-false
          result "selected_window_buffer"))
   (cons "minibuffer_active"
         (soma-gym-bridge--result-field-or-json-false
          result "minibuffer_active"))))

(defun soma-gym-bridge--core-interaction-minibuffer-evidence (result)
  "Return minibuffer evidence from an `emacs.observe_interaction' RESULT."
  (soma-gym-bridge--alist
   (cons "active"
         (soma-gym-bridge--result-field-or-json-false
          result "minibuffer_active"))
   (cons "buffer"
         (soma-gym-bridge--result-field-or-json-false
          result "minibuffer_buffer"))
   (cons "prompt"
         (soma-gym-bridge--result-field-or-json-false
          result "minibuffer_prompt"))
   (cons "contents"
         (soma-gym-bridge--result-field-or-json-false
          result "minibuffer_contents"))))

(defun soma-gym-bridge--core-interaction-transient-evidence (result)
  "Return Transient evidence from an `emacs.observe_interaction' RESULT."
  (soma-gym-bridge--alist
   (cons "active"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_active"))
   (cons "current_command"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_current_command"))
   (cons "prefix_command"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_prefix_command"))
   (cons "map_active"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_map_active"))
   (cons "map_installed"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_map_installed"))
   (cons "buffer"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_buffer"))
   (cons "window_live"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_window_live"))
   (cons "window_buffer"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_window_buffer"))
   (cons "active_arguments"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_active_arguments"))
   (cons "suffixes"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_suffixes"))))

(defun soma-gym-bridge--core-interaction-observation (result)
  "Return command-loop interaction observation from RESULT."
  (soma-gym-bridge--alist
   (cons "daemonp"
         (soma-gym-bridge--result-field-or-json-false result "daemonp"))
   (cons "server_name"
         (soma-gym-bridge--result-field-or-json-false result "server_name"))
   (cons "emacs_pid"
         (soma-gym-bridge--result-field-or-json-false result "emacs_pid"))
   (cons "selected_window_buffer"
         (soma-gym-bridge--result-field-or-json-false
          result "selected_window_buffer"))
   (cons "recursive_edit_depth"
         (soma-gym-bridge--result-field-or-json-false
          result "recursive_edit_depth"))
   (cons "minibuffer_active"
         (soma-gym-bridge--result-field-or-json-false
          result "minibuffer_active"))
   (cons "transient_active"
         (soma-gym-bridge--result-field-or-json-false
          result "transient_active"))
   (cons "this_command"
         (soma-gym-bridge--result-field-or-json-false
          result "this_command"))
   (cons "last_command"
         (soma-gym-bridge--result-field-or-json-false
          result "last_command"))))

(defun soma-gym-bridge--core-editor-session-evidence (result)
  "Return editor-session evidence from an `emacs.observe_session' RESULT."
  (let ((session (soma-gym-bridge--result-field result "session"))
        (editor (soma-gym-bridge--result-field result "editor")))
    (soma-gym-bridge--alist
     (cons "api_version"
           (or (and (listp session)
                    (soma-gym-bridge--get session "api_version"))
               :json-false))
     (cons "emacs_pid"
           (or (and (listp session)
                    (soma-gym-bridge--get session "emacs_pid"))
               :json-false))
     (cons "daemonp"
           (or (and (listp session)
                    (soma-gym-bridge--get session "daemonp"))
               :json-false))
     (cons "server_name"
           (or (and (listp session)
                    (soma-gym-bridge--get session "server_name"))
               :json-false))
     (cons "system_type"
           (or (and (listp session)
                    (soma-gym-bridge--get session "system_type"))
               :json-false))
     (cons "system_configuration"
           (or (and (listp session)
                    (soma-gym-bridge--get session "system_configuration"))
               :json-false))
     (cons "profiles"
           (or (and (listp session)
                    (soma-gym-bridge--get session "profiles"))
               nil))
     (cons "version"
           (or (and (listp editor)
                    (soma-gym-bridge--get editor "version"))
               :json-false))
     (cons "executable"
           (or (and (listp editor)
                    (soma-gym-bridge--get editor "executable"))
               :json-false))
     (cons "source_directory"
           (or (and (listp editor)
                    (soma-gym-bridge--get editor "source_directory"))
               :json-false)))))

(defun soma-gym-bridge--project-root-evidence (result)
  "Return project-root evidence from RESULT."
  (soma-gym-bridge--alist
   (cons "path"
         (soma-gym-bridge--result-field-or-json-false
          result "project_root"))))

(defun soma-gym-bridge--rootfs-runtime-evidence (result)
  "Return rootfs runtime evidence from RESULT."
  (soma-gym-bridge--alist
   (cons "run_id"
         (soma-gym-bridge--result-field-or-json-false result "run_id"))
   (cons "plan"
         (soma-gym-bridge--result-field-or-json-false result "plan"))))

(defun soma-gym-bridge--project-context-observation (result)
  "Return project-context observation from a `project.observe' RESULT."
  (soma-gym-bridge--alist
   (cons "project_root"
         (soma-gym-bridge--result-field-or-json-false
          result "project_root"))
   (cons "buffer_name"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_name"))
   (cons "buffer_file_name"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_file_name"))
   (cons "default_directory"
         (soma-gym-bridge--result-field-or-json-false
          result "default_directory"))
   (cons "vcs"
         (soma-gym-bridge--result-field-or-json-false result "vcs"))
   (cons "search_tool"
         (soma-gym-bridge--result-field-or-json-false
          result "search_tool"))))

(defun soma-gym-bridge--project-bounded-results-evidence (result)
  "Return bounded search-result evidence from a `project.search' RESULT."
  (soma-gym-bridge--alist
   (cons "exit_status"
         (soma-gym-bridge--result-field-or-json-false
          result "exit_status"))
   (cons "result_count"
         (soma-gym-bridge--result-field-or-json-false
          result "result_count"))
   (cons "results_returned"
         (soma-gym-bridge--result-field-or-json-false
          result "results_returned"))
   (cons "results_truncated"
         (soma-gym-bridge--result-field-or-json-false
          result "results_truncated"))
   (cons "parse_errors"
         (or (soma-gym-bridge--result-field result "parse_errors")
             nil))))

(defun soma-gym-bridge--project-search-matches-observation (result)
  "Return search-match observation from a `project.search' RESULT."
  (soma-gym-bridge--alist
   (cons "project_root"
         (soma-gym-bridge--result-field-or-json-false
          result "project_root"))
   (cons "query"
         (soma-gym-bridge--result-field-or-json-false result "query"))
   (cons "results"
         (or (soma-gym-bridge--result-field result "results") nil))
   (cons "parse_errors"
         (or (soma-gym-bridge--result-field result "parse_errors")
             nil))))

(defun soma-gym-bridge--buffer-read-identity-evidence (result)
  "Return buffer identity evidence from a `buffer.read' RESULT."
  (soma-gym-bridge--alist
   (cons "buffer_name"
         (soma-gym-bridge--result-field-or-json-false result "buffer_name"))
   (cons "text_owner_name"
         (soma-gym-bridge--result-field-or-json-false
          result "text_owner_name"))))

(defun soma-gym-bridge--buffer-read-range-evidence (result)
  "Return text-range evidence from a `buffer.read' RESULT."
  (soma-gym-bridge--alist
   (cons "start"
         (soma-gym-bridge--result-field-or-json-false result "start"))
   (cons "end"
         (soma-gym-bridge--result-field-or-json-false result "end"))))

(defun soma-gym-bridge--buffer-read-snapshot-observation (result)
  "Return text snapshot observation from a `buffer.read' RESULT."
  (soma-gym-bridge--alist
   (cons "text"
         (soma-gym-bridge--result-field-or-json-false result "text"))
   (cons "chars_tick"
         (soma-gym-bridge--result-field-or-json-false
          result "chars_tick"))))

(defun soma-gym-bridge--org-structure-evidence (result)
  "Return structure evidence from an `org.observe_tree' RESULT."
  (soma-gym-bridge--alist
   (cons "structure_fingerprint"
         (soma-gym-bridge--result-field-or-json-false
          result "structure_fingerprint"))
   (cons "chars_tick"
         (soma-gym-bridge--result-field-or-json-false
          result "chars_tick"))))

(defun soma-gym-bridge--org-heading-index-evidence (result)
  "Return heading-index evidence from an `org.observe_tree' RESULT."
  (soma-gym-bridge--alist
   (cons "heading_count"
         (soma-gym-bridge--result-field-or-json-false
          result "heading_count"))
   (cons "headings_truncated"
         (soma-gym-bridge--result-field-or-json-false
          result "headings_truncated"))))

(defun soma-gym-bridge--org-source-block-index-evidence (result)
  "Return source-block-index evidence from an `org.observe_tree' RESULT."
  (soma-gym-bridge--alist
   (cons "source_block_count"
         (soma-gym-bridge--result-field-or-json-false
          result "source_block_count"))
   (cons "source_blocks_truncated"
         (soma-gym-bridge--result-field-or-json-false
          result "source_blocks_truncated"))))

(defun soma-gym-bridge--org-tree-observation (result)
  "Return Org tree observation from an `org.observe_tree' RESULT."
  (soma-gym-bridge--alist
   (cons "structure_fingerprint"
         (soma-gym-bridge--result-field-or-json-false
          result "structure_fingerprint"))
   (cons "heading_count"
         (soma-gym-bridge--result-field-or-json-false
          result "heading_count"))
   (cons "headings"
         (or (soma-gym-bridge--result-field result "headings") nil))
   (cons "headings_truncated"
         (soma-gym-bridge--result-field-or-json-false
          result "headings_truncated"))
   (cons "source_block_count"
         (soma-gym-bridge--result-field-or-json-false
          result "source_block_count"))
   (cons "source_blocks"
         (or (soma-gym-bridge--result-field result "source_blocks")
             nil))
   (cons "source_blocks_truncated"
         (soma-gym-bridge--result-field-or-json-false
          result "source_blocks_truncated"))))

(defun soma-gym-bridge--dired-directory-evidence (result)
  "Return directory evidence from a `dired.observe' RESULT."
  (soma-gym-bridge--alist
   (cons "path"
         (soma-gym-bridge--result-field-or-json-false
          result "directory"))
   (cons "buffer_name"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_name"))))

(defun soma-gym-bridge--dired-selection-state-evidence (result)
  "Return Dired mark and deletion-flag evidence from RESULT."
  (soma-gym-bridge--alist
   (cons "marked_files"
         (or (soma-gym-bridge--result-field result "marked_files")
             :json-false))
   (cons "deletion_flagged_files"
         (or (soma-gym-bridge--result-field
              result "deletion_flagged_files")
             :json-false))))

(defun soma-gym-bridge--dired-file-signatures-evidence (result)
  "Return filesystem-signature evidence from a `dired.observe' RESULT."
  (soma-gym-bridge--alist
   (cons "file_count"
         (soma-gym-bridge--result-field-or-json-false
          result "file_count"))
   (cons "files_returned"
         (soma-gym-bridge--result-field-or-json-false
          result "files_returned"))
   (cons "files_truncated"
         (soma-gym-bridge--result-field-or-json-false
          result "files_truncated"))
   (cons "files"
         (or (soma-gym-bridge--result-field result "files") nil))))

(defun soma-gym-bridge--dired-listing-observation (result)
  "Return Dired listing observation from a `dired.observe' RESULT."
  (soma-gym-bridge--alist
   (cons "directory"
         (soma-gym-bridge--result-field-or-json-false
          result "directory"))
   (cons "marked_files"
         (or (soma-gym-bridge--result-field result "marked_files")
             :json-false))
   (cons "deletion_flagged_files"
         (or (soma-gym-bridge--result-field
              result "deletion_flagged_files")
             :json-false))
   (cons "file_count"
         (soma-gym-bridge--result-field-or-json-false
          result "file_count"))
   (cons "files_returned"
         (soma-gym-bridge--result-field-or-json-false
          result "files_returned"))
   (cons "files_truncated"
         (soma-gym-bridge--result-field-or-json-false
          result "files_truncated"))
   (cons "files"
         (or (soma-gym-bridge--result-field result "files") nil))))

(defun soma-gym-bridge--comint-process-identity-evidence (result)
  "Return process identity evidence from a `comint.observe' RESULT."
  (soma-gym-bridge--alist
   (cons "buffer_name"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_name"))
   (cons "process_present"
         (soma-gym-bridge--result-field-or-json-false
          result "process_present"))
   (cons "process_live"
         (soma-gym-bridge--result-field-or-json-false
          result "process_live"))
   (cons "process_name"
         (soma-gym-bridge--result-field-or-json-false
          result "process_name"))
   (cons "process_status"
         (soma-gym-bridge--result-field-or-json-false
          result "process_status"))
   (cons "process_id"
         (soma-gym-bridge--result-field-or-json-false
          result "process_id"))
   (cons "process_exe"
         (soma-gym-bridge--result-field-or-json-false
          result "process_exe"))
   (cons "process_rootfs_owned"
         (soma-gym-bridge--result-field-or-json-false
          result "process_rootfs_owned"))))

(defun soma-gym-bridge--comint-input-boundary-evidence (result)
  "Return input-boundary evidence from a `comint.observe' RESULT."
  (soma-gym-bridge--alist
   (cons "process_mark"
         (soma-gym-bridge--result-field-or-json-false
          result "process_mark"))
   (cons "point_after_process_mark"
         (soma-gym-bridge--result-field-or-json-false
          result "point_after_process_mark"))
   (cons "last_input_markers_live"
         (soma-gym-bridge--result-field-or-json-false
          result "last_input_markers_live"))
   (cons "last_input_start"
         (soma-gym-bridge--result-field-or-json-false
          result "last_input_start"))
   (cons "last_input_end"
         (soma-gym-bridge--result-field-or-json-false
          result "last_input_end"))
   (cons "last_input_text"
         (soma-gym-bridge--result-field-or-json-false
          result "last_input_text"))
   (cons "current_input_text"
         (soma-gym-bridge--result-field-or-json-false
          result "current_input_text"))))

(defun soma-gym-bridge--comint-state-observation (result)
  "Return Comint state observation from a `comint.observe' RESULT."
  (soma-gym-bridge--alist
   (cons "buffer_name"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_name"))
   (cons "major_mode"
         (soma-gym-bridge--result-field-or-json-false
          result "major_mode"))
   (cons "buffer_file_name"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_file_name"))
   (cons "chars_tick"
         (soma-gym-bridge--result-field-or-json-false
          result "chars_tick"))
   (cons "modified"
         (soma-gym-bridge--result-field-or-json-false
          result "modified"))
   (cons "default_directory"
         (soma-gym-bridge--result-field-or-json-false
          result "default_directory"))
   (cons "point"
         (soma-gym-bridge--result-field-or-json-false
          result "point"))
   (cons "process_mark"
         (soma-gym-bridge--result-field-or-json-false
          result "process_mark"))
   (cons "current_input_text"
         (soma-gym-bridge--result-field-or-json-false
          result "current_input_text"))
   (cons "buffer_tail"
         (soma-gym-bridge--result-field-or-json-false
          result "buffer_tail"))
   (cons "selected_window_buffer"
         (soma-gym-bridge--result-field-or-json-false
          result "selected_window_buffer"))
   (cons "minibuffer_active"
         (soma-gym-bridge--result-field-or-json-false
          result "minibuffer_active"))))

(defun soma-gym-bridge--magit-repo-fingerprint-evidence (result)
  "Return repository fingerprint evidence from a Magit observation RESULT."
  (soma-gym-bridge--alist
   (cons "magit_repo"
         (soma-gym-bridge--result-field-or-json-false
          result "magit_repo"))
   (cons "magit_head"
         (soma-gym-bridge--result-field-or-json-false
          result "magit_head"))
   (cons "magit_status_porcelain"
         (soma-gym-bridge--result-field-or-json-false
          result "magit_status_porcelain"))
   (cons "magit_status_hash"
         (soma-gym-bridge--result-field-or-json-false
          result "magit_status_hash"))
   (cons "magit_worktree_diff_hash"
         (soma-gym-bridge--result-field-or-json-false
          result "magit_worktree_diff_hash"))
   (cons "magit_index_diff_hash"
         (soma-gym-bridge--result-field-or-json-false
          result "magit_index_diff_hash"))))

(defun soma-gym-bridge--magit-status-generation-evidence (result)
  "Return Magit view-generation evidence from an observation RESULT."
  (soma-gym-bridge--alist
   (cons "section_count"
         (soma-gym-bridge--result-field-or-json-false
          result "section_count"))
   (cons "sections_returned"
         (soma-gym-bridge--result-field-or-json-false
          result "sections_returned"))
   (cons "sections_truncated"
         (soma-gym-bridge--result-field-or-json-false
          result "sections_truncated"))
   (cons "magit_status_hash"
         (soma-gym-bridge--result-field-or-json-false
          result "magit_status_hash"))
   (cons "magit_worktree_diff_hash"
         (soma-gym-bridge--result-field-or-json-false
          result "magit_worktree_diff_hash"))
   (cons "magit_index_diff_hash"
         (soma-gym-bridge--result-field-or-json-false
          result "magit_index_diff_hash"))))

(defun soma-gym-bridge--magit-sections-observation (result)
  "Return bounded Magit section observations from RESULT."
  (soma-gym-bridge--alist
   (cons "section_count"
         (soma-gym-bridge--result-field-or-json-false
          result "section_count"))
   (cons "sections_returned"
         (soma-gym-bridge--result-field-or-json-false
          result "sections_returned"))
   (cons "sections_truncated"
         (soma-gym-bridge--result-field-or-json-false
          result "sections_truncated"))
   (cons "sections"
         (or (soma-gym-bridge--result-field result "sections") nil))))

(defun soma-gym-bridge--terminal-or-unknown-state-p (result)
  "Return non-nil when RESULT describes work that may have taken effect."
  (member (soma-gym-bridge--result-field result "state")
          '("succeeded" "outcome_unknown")))

(defun soma-gym-bridge--first-object-field (object key)
  "Return the first object in OBJECT's sequence field KEY."
  (let ((items (and (listp object)
                    (soma-gym-bridge--get object key))))
    (cond
     ((vectorp items)
      (and (> (length items) 0) (aref items 0)))
     ((consp items)
      (car items)))))

(defun soma-gym-bridge--org-tree-fingerprint-summary (tree)
  "Return a compact structure fingerprint summary from Org TREE."
  (soma-gym-bridge--alist
   (cons "buffer_name"
         (or (and (listp tree)
                  (soma-gym-bridge--get tree "buffer_name"))
             :json-false))
   (cons "structure_fingerprint"
         (or (and (listp tree)
                  (soma-gym-bridge--get tree
                                        "structure_fingerprint"))
             :json-false))
   (cons "heading_count"
         (or (and (listp tree)
                  (soma-gym-bridge--get tree "heading_count"))
             :json-false))
   (cons "source_block_count"
         (or (and (listp tree)
                  (soma-gym-bridge--get tree "source_block_count"))
             :json-false))))

(defun soma-gym-bridge--org-heading-update-effect (result)
  "Return an Org heading-update effect from RESULT, or nil."
  (when (soma-gym-bridge--terminal-or-unknown-state-p result)
    (let ((before (soma-gym-bridge--result-field result "before"))
          (after (soma-gym-bridge--result-field result "after")))
      (soma-gym-bridge--alist
       (cons "route"
             (soma-gym-bridge--result-field-or-json-false
              result "route"))
       (cons "target_position"
             (soma-gym-bridge--result-field-or-json-false
              result "target_position"))
       (cons "before_structure"
             (soma-gym-bridge--org-tree-fingerprint-summary before))
       (cons "after_structure"
             (soma-gym-bridge--org-tree-fingerprint-summary after))
       (cons "before_heading"
             (or (soma-gym-bridge--first-object-field before "headings")
                 :json-false))
       (cons "after_heading"
             (or (soma-gym-bridge--first-object-field after "headings")
                 :json-false))))))

(defun soma-gym-bridge--org-source-edit-buffer-effect (result)
  "Return an Org source-edit buffer effect from RESULT, or nil."
  (when (soma-gym-bridge--terminal-or-unknown-state-p result)
    (let ((parent-after
           (soma-gym-bridge--result-field result "parent_after_open")))
      (soma-gym-bridge--alist
       (cons "route"
             (soma-gym-bridge--result-field-or-json-false
              result "route"))
       (cons "edit_buffer_name"
             (soma-gym-bridge--result-field-or-json-false
              result "edit_buffer_name"))
       (cons "source_edit_buffer_separate"
             (soma-gym-bridge--result-field-or-json-false
              result "source_edit_buffer_separate"))
       (cons "source_block"
             (soma-gym-bridge--result-field-or-json-false
              result "source_block"))
       (cons "parent_markers"
             (soma-gym-bridge--result-field-or-json-false
              result "parent_markers"))
       (cons "parent_after_open"
             (soma-gym-bridge--org-tree-fingerprint-summary
              parent-after))))))

(defun soma-gym-bridge--org-parent-buffer-writeback-effect (result)
  "Return an Org source-edit parent writeback effect from RESULT, or nil."
  (when (soma-gym-bridge--terminal-or-unknown-state-p result)
    (let ((before-parent
           (soma-gym-bridge--result-field result "before_parent"))
          (after-parent
           (soma-gym-bridge--result-field result "after_parent")))
      (soma-gym-bridge--alist
       (cons "route"
             (soma-gym-bridge--result-field-or-json-false
              result "route"))
       (cons "edit_buffer_live_after"
             (soma-gym-bridge--result-field-or-json-false
              result "edit_buffer_live_after"))
       (cons "before_parent"
             (soma-gym-bridge--org-tree-fingerprint-summary
              before-parent))
       (cons "after_parent"
             (soma-gym-bridge--org-tree-fingerprint-summary
              after-parent))
       (cons "before_edit"
             (soma-gym-bridge--result-field-or-json-false
              result "before_edit"))))))

(defun soma-gym-bridge--org-babel-runtime-effect (result)
  "Return an Org Babel execution effect from RESULT, or nil."
  (when (soma-gym-bridge--terminal-or-unknown-state-p result)
    (let ((runtime (soma-gym-bridge--result-field result
                                                  "runtime_identity")))
      (soma-gym-bridge--alist
       (cons "route"
             (soma-gym-bridge--result-field-or-json-false
              result "route"))
       (cons "source_block"
             (soma-gym-bridge--result-field-or-json-false
              result "source_block"))
       (cons "babel_info"
             (soma-gym-bridge--result-field-or-json-false
              result "babel_info"))
       (cons "runtime_identity"
             (soma-gym-bridge--alist
              (cons "language"
                    (or (and (listp runtime)
                             (soma-gym-bridge--get runtime
                                                   "language"))
                        :json-false))
              (cons "session"
                    (or (and (listp runtime)
                             (soma-gym-bridge--get runtime
                                                   "session"))
                        :json-false))
              (cons "default_directory"
                    (or (and (listp runtime)
                             (soma-gym-bridge--get
                              runtime "default_directory"))
                        :json-false))
              (cons "emacs_pid"
                    (or (and (listp runtime)
                             (soma-gym-bridge--get runtime
                                                   "emacs_pid"))
                        :json-false))
              (cons "emacs_executable"
                    (or (and (listp runtime)
                             (soma-gym-bridge--get
                              runtime "emacs_executable"))
                        :json-false))
              (cons "rootfs_runtime"
                    (soma-gym-bridge--rootfs-runtime-evidence
                     runtime))))
       (cons "result_value"
             (soma-gym-bridge--result-field-or-json-false
              result "result_value"))
       (cons "parent_result_block"
             (soma-gym-bridge--result-field-or-json-false
              result "parent_result_block"))
       (cons "timeout_seconds"
             (soma-gym-bridge--result-field-or-json-false
              result "timeout_seconds"))))))

(defun soma-gym-bridge--org-html-export-effect (result)
  "Return an Org HTML export effect from RESULT, or nil."
  (when (and (soma-gym-bridge--terminal-or-unknown-state-p result)
             (not (string= (or (soma-gym-bridge--result-field result "mode")
                               "apply")
                           "preview")))
    (let ((after (soma-gym-bridge--result-field result "after")))
      (soma-gym-bridge--alist
       (cons "route" "org-export")
       (cons "backend" "html")
       (cons "output_path"
             (soma-gym-bridge--result-field-or-json-false
              result "output_path"))
       (cons "generated_file"
             (soma-gym-bridge--result-field-or-json-false
              result "generated_file"))
       (cons "output_signature"
             (or (and (listp after)
                      (soma-gym-bridge--get after
                                            "output_signature"))
                 :json-false))))))

(defun soma-gym-bridge--dired-filesystem-rename-effect (result)
  "Return a Dired filesystem rename effect from RESULT, or nil."
  (when (soma-gym-bridge--terminal-or-unknown-state-p result)
    (soma-gym-bridge--alist
     (cons "route" "dired-rename-file")
     (cons "source"
           (soma-gym-bridge--result-field-or-json-false result "source"))
     (cons "target"
           (soma-gym-bridge--result-field-or-json-false result "target"))
     (cons "source_path"
           (soma-gym-bridge--result-field-or-json-false
            result "source_path"))
     (cons "target_path"
           (soma-gym-bridge--result-field-or-json-false
            result "target_path"))
     (cons "source_before"
           (soma-gym-bridge--result-field-or-json-false
            result "source_before"))
     (cons "target_before"
           (soma-gym-bridge--result-field-or-json-false
            result "target_before"))
     (cons "source_after"
           (soma-gym-bridge--result-field-or-json-false
            result "source_after"))
     (cons "target_after"
           (soma-gym-bridge--result-field-or-json-false
            result "target_after")))))

(defun soma-gym-bridge--dired-filesystem-delete-effect (result)
  "Return a Dired filesystem delete effect from RESULT, or nil."
  (when (soma-gym-bridge--terminal-or-unknown-state-p result)
    (soma-gym-bridge--alist
     (cons "route" "dired-do-delete")
     (cons "marked_files"
           (soma-gym-bridge--result-field-or-json-false
            result "marked_files"))
     (cons "deletion_flagged_files_before"
           (soma-gym-bridge--result-field-or-json-false
            result "deletion_flagged_files_before"))
     (cons "pre_filesystem_state"
           (soma-gym-bridge--result-field-or-json-false
            result "pre_filesystem_state"))
     (cons "post_filesystem_state"
           (soma-gym-bridge--result-field-or-json-false
            result "post_filesystem_state")))))

(defun soma-gym-bridge--magit-stage-index-effect (result)
  "Return a Magit git-index update effect from RESULT, or nil."
  (when (soma-gym-bridge--terminal-or-unknown-state-p result)
    (let* ((before (soma-gym-bridge--result-field result "before"))
           (after (soma-gym-bridge--result-field result "after"))
           (requested-identity
            (soma-gym-bridge--result-field result "section_identity"))
           (requested-type
            (soma-gym-bridge--result-field result "section_type"))
           (selected
            (and (listp before)
                 (soma-gym-bridge--get before "selected_section"))))
      (soma-gym-bridge--alist
       (cons "route"
             (soma-gym-bridge--result-field-or-json-false result
                                                         "route"))
       (cons "command"
             (soma-gym-bridge--result-field-or-json-false result
                                                         "command"))
       (cons "this_command"
             (soma-gym-bridge--result-field-or-json-false
              result "this_command"))
       (cons "target_point"
             (soma-gym-bridge--result-field-or-json-false
              result "target_point"))
       (cons "section_type"
             (or requested-type
                 (and (listp requested-identity)
                      (soma-gym-bridge--get requested-identity "type"))
                 (and (listp selected)
                      (soma-gym-bridge--get selected "type"))
                 :json-false))
       (cons "section_identity"
             (or (and (listp requested-identity) requested-identity)
                 (and (listp selected)
                      (soma-gym-bridge--get selected
                                            "section_identity"))
                 :json-false))
       (cons "before_repo_fingerprint"
             (soma-gym-bridge--magit-repo-fingerprint-evidence
              before))
       (cons "after_repo_fingerprint"
             (soma-gym-bridge--magit-repo-fingerprint-evidence
              after))))))

(defun soma-gym-bridge--lsp-source-buffer-observation (result)
  "Return source-buffer observation fields from LSP RESULT."
  (soma-gym-bridge--alist
   (cons "buffer_name"
         (or (soma-gym-bridge--result-field result "buffer_name")
             :json-false))
   (cons "text_owner_name"
         (or (soma-gym-bridge--result-field result "text_owner_name")
             :json-false))
   (cons "buffer_file_name"
         (or (soma-gym-bridge--result-field result "buffer_file_name")
             :json-false))
   (cons "major_mode"
         (or (soma-gym-bridge--result-field result "major_mode")
             :json-false))
   (cons "chars_tick"
         (or (soma-gym-bridge--result-field result "chars_tick")
             :json-false))
   (cons "modified"
         (or (soma-gym-bridge--result-field result "modified")
             :json-false))
   (cons "read_only"
         (or (soma-gym-bridge--result-field result "read_only")
             :json-false))
   (cons "point"
         (or (soma-gym-bridge--result-field result "point")
             :json-false))
   (cons "mark_active"
         (or (soma-gym-bridge--result-field result "mark_active")
             :json-false))
   (cons "narrowed"
         (or (soma-gym-bridge--result-field result "narrowed")
             :json-false))
   (cons "selected_window_buffer"
         (or (soma-gym-bridge--result-field
              result "selected_window_buffer")
             :json-false))
   (cons "minibuffer_active"
         (or (soma-gym-bridge--result-field result "minibuffer_active")
             :json-false))
   (cons "text_hash"
         (or (soma-gym-bridge--result-field result "text_hash")
             :json-false))
   (cons "text_excerpt"
         (or (soma-gym-bridge--result-field result "text_excerpt")
             :json-false))))

(defun soma-gym-bridge--lsp-workspace-evidence (result)
  "Return workspace evidence from LSP RESULT."
  (soma-gym-bridge--alist
   (cons "present"
         (or (soma-gym-bridge--result-field result "workspace_present")
             :json-false))
   (cons "status"
         (or (soma-gym-bridge--result-field result "workspace_status")
             :json-false))
   (cons "root"
         (or (soma-gym-bridge--result-field result "workspace_root")
             :json-false))
   (cons "proc_type"
         (or (soma-gym-bridge--result-field result "workspace_proc_type")
             :json-false))))

(defun soma-gym-bridge--lsp-server-evidence-section (result)
  "Return LSP server evidence from RESULT."
  (soma-gym-bridge--alist
   (cons "id"
         (or (soma-gym-bridge--result-field result "server_id")
             :json-false))
   (cons "pid"
         (or (soma-gym-bridge--result-field result "server_pid")
             :json-false))
   (cons "executable"
         (or (soma-gym-bridge--result-field result "server_executable")
             :json-false))
   (cons "rootfs_owned"
         (or (soma-gym-bridge--result-field result "server_rootfs_owned")
             :json-false))
   (cons "cwd"
         (or (soma-gym-bridge--result-field result "server_cwd")
             :json-false))
   (cons "command"
         (or (soma-gym-bridge--result-field result "server_command")
             nil))
   (cons "cmd_proc_type"
         (or (soma-gym-bridge--result-field result "cmd_proc_type")
             :json-false))
   (cons "cmd_proc_status"
         (or (soma-gym-bridge--result-field result "cmd_proc_status")
             :json-false))))

(defun soma-gym-bridge--lsp-document-evidence (result)
  "Return document identity evidence from LSP RESULT."
  (soma-gym-bridge--alist
   (cons "uri"
         (or (soma-gym-bridge--result-field result "document_uri")
             :json-false))
   (cons "version"
         (or (soma-gym-bridge--result-field result "document_version")
             :json-false))
   (cons "position_encoding"
         (or (soma-gym-bridge--result-field result "position_encoding")
             :json-false))))

(defun soma-gym-bridge--lsp-symbol-observation (result)
  "Return point-symbol observation from LSP RESULT."
  (soma-gym-bridge--alist
   (cons "name"
         (or (soma-gym-bridge--result-field result "symbol")
             :json-false))
   (cons "source"
         (or (soma-gym-bridge--result-field result "symbol_source")
             :json-false))))

(defun soma-gym-bridge--lsp-rootfs-runtime-evidence (result)
  "Return rootfs runtime evidence from LSP RESULT."
  (soma-gym-bridge--alist
   (cons "run_id"
         (or (soma-gym-bridge--result-field result "run_id")
             :json-false))
   (cons "plan"
         (or (soma-gym-bridge--result-field result "plan")
             :json-false))))

(defun soma-gym-bridge--lsp-affected-files (result)
  "Return files affected by the LSP workspace edit RESULT."
  (let ((files nil))
    (dolist (saved (soma-gym-bridge--result-field result "saved_buffers"))
      (let ((file (and (listp saved)
                       (soma-gym-bridge--get saved "file"))))
        (when (and (stringp file) (not (string-empty-p file)))
          (push file files))))
    (nreverse (delete-dups files))))

(defun soma-gym-bridge--lsp-request-evidence (result)
  "Return LSP request evidence from a rename RESULT."
  (soma-gym-bridge--alist
   (cons "methods"
         (or (soma-gym-bridge--result-field result "request_methods")
             nil))
   (cons "requests"
         (or (soma-gym-bridge--result-field result "requests")
             nil))
   (cons "wait_timeout_seconds"
         (or (soma-gym-bridge--result-field
              result "wait_timeout_seconds")
             :json-false))))

(defun soma-gym-bridge--lsp-workspace-edit-evidence (result)
  "Return workspace-edit evidence from a rename RESULT."
  (soma-gym-bridge--alist
   (cons "applied"
         (or (soma-gym-bridge--result-field
              result "workspace_edit_applied")
             :json-false))
   (cons "sha256"
         (or (soma-gym-bridge--result-field
              result "workspace_edit_sha256")
             :json-false))
   (cons "excerpt"
         (or (soma-gym-bridge--result-field
              result "workspace_edit_excerpt")
             :json-false))))

(defun soma-gym-bridge--lsp-workspace-edit-effect (result)
  "Return a multi-buffer workspace-edit effect from a rename RESULT."
  (soma-gym-bridge--alist
   (cons "route"
         (or (soma-gym-bridge--result-field result "route")
             :json-false))
   (cons "new_name"
         (or (soma-gym-bridge--result-field result "new_name")
             :json-false))
   (cons "request_methods"
         (or (soma-gym-bridge--result-field result "request_methods")
             nil))
   (cons "applied"
         (or (soma-gym-bridge--result-field
              result "workspace_edit_applied")
             :json-false))
   (cons "sha256"
         (or (soma-gym-bridge--result-field
              result "workspace_edit_sha256")
             :json-false))
   (cons "saved_buffers"
         (or (soma-gym-bridge--result-field result "saved_buffers")
             nil))
   (cons "affected_files"
         (soma-gym-bridge--lsp-affected-files result))))

(defun soma-gym-bridge--typed-envelope-sections (operation result)
  "Project operation RESULT into typed response envelope sections.

The legacy `result' payload is preserved during the migration, but model-facing
operation evidence should also be available through the stable top-level
`evidence', `observations', and `effects' objects."
  (let (evidence observations effects)
    (when (and (stringp operation) (listp result))
      (cl-labels
          ((add-evidence
            (key)
            (when (soma-gym-bridge--result-field-present-p result key)
              (push (cons key (soma-gym-bridge--result-field result key))
                    evidence)))
           (add-evidence-value
            (key value)
            (push (cons key value) evidence))
           (add-effect
            (key value)
            (push (cons key value) effects)))
        (when (soma-gym-bridge--result-field-present-p
               result "precondition_checks")
          (push
           (cons "precondition_checks"
                 (soma-gym-bridge--result-field result "precondition_checks"))
           evidence))
        (when (soma-gym-bridge--result-field-present-p result
                                                       "conflict_preview")
          (push
           (cons "conflict_preview"
                 (soma-gym-bridge--result-field result "conflict_preview"))
           evidence))
        (when (or (soma-gym-bridge--result-field-present-p result "before")
                  (soma-gym-bridge--result-field-present-p result "after"))
          (let ((before-after nil))
            (when (soma-gym-bridge--result-field-present-p result "before")
              (push
               (cons "before"
                     (soma-gym-bridge--result-field result "before"))
               observations)
              (push
               (cons "before"
                     (soma-gym-bridge--result-field result "before"))
               before-after))
            (when (soma-gym-bridge--result-field-present-p result "after")
              (push
               (cons "after"
                     (soma-gym-bridge--result-field result "after"))
               observations)
              (push
               (cons "after"
                     (soma-gym-bridge--result-field result "after"))
               before-after))
            (push
             (cons "before_after_observations" (nreverse before-after))
             evidence)))
        (when (soma-gym-bridge--result-field-present-p result
                                                       "changed_ranges")
          (push
           (cons "changed_ranges"
                 (soma-gym-bridge--result-field result "changed_ranges"))
           effects))
        (when-let ((changed-range
                    (soma-gym-bridge--edit-changed-range-effect
                     operation result)))
          (push (cons "changed_range" changed-range) effects)
          (push (cons "changed_range" changed-range) evidence))
        (cond
	         ((string= operation "emacs.observe_buffer")
	          (add-evidence-value
	           "buffer_identity"
	           (soma-gym-bridge--core-buffer-identity-evidence result))
	          (add-evidence-value
	           "text_owner"
	           (soma-gym-bridge--core-buffer-text-owner-evidence result))
	          (add-evidence-value
	           "point_region_narrowing"
	           (soma-gym-bridge--core-point-region-narrowing-evidence
	            result))
	          (push
	           (cons "buffer_state"
	                 (soma-gym-bridge--core-buffer-state-observation
	                  result))
	           observations))
	         ((string= operation "emacs.observe_session")
	          (add-evidence-value
	           "editor_session"
	           (soma-gym-bridge--core-editor-session-evidence result))
	          (add-evidence-value
	           "rootfs_identity"
	           (or (soma-gym-bridge--result-field result "rootfs")
	               (soma-gym-bridge--empty-object)))
	          (add-evidence-value
	           "doom_identity"
	           (or (soma-gym-bridge--result-field result "doom")
	               (soma-gym-bridge--empty-object)))
	          (push
	           (cons "selected_frame"
	                 (or (soma-gym-bridge--result-field
	                      result "selected_frame")
	                     (soma-gym-bridge--empty-object)))
	           observations))
         ((string= operation "emacs.observe_interaction")
          (add-evidence-value
           "minibuffer"
           (soma-gym-bridge--core-interaction-minibuffer-evidence result))
          (add-evidence-value
           "transient"
           (soma-gym-bridge--core-interaction-transient-evidence result))
          (add-evidence "this_command")
          (add-evidence "last_command")
          (push
           (cons "interaction"
                 (soma-gym-bridge--core-interaction-observation result))
           observations))
         ((string= operation "emacs.observe_frames")
          (add-evidence-value
           "frame_identity"
           (soma-gym-bridge--alist
            (cons "daemonp"
                  (or (soma-gym-bridge--result-field result "daemonp")
                      :json-false))
            (cons "server_name"
                  (or (soma-gym-bridge--result-field result "server_name")
                      :json-false))
            (cons "emacs_pid"
                  (or (soma-gym-bridge--result-field result "emacs_pid")
                      :json-false))
            (cons "frame_count"
                  (or (soma-gym-bridge--result-field result "frame_count")
                      0))))
          (add-evidence-value
           "terminal_client_identity"
           (soma-gym-bridge--alist
            (cons "terminal_frame_count"
                  (or (soma-gym-bridge--result-field
                       result "terminal_frame_count")
                      0))
            (cons "graphic_frame_count"
                  (or (soma-gym-bridge--result-field
                       result "graphic_frame_count")
                      0))
            (cons "client_frame_count"
                  (or (soma-gym-bridge--result-field
                       result "client_frame_count")
                      0))))
          (push
           (cons "frame_summary"
                 (soma-gym-bridge--alist
                  (cons "frame_count"
                        (or (soma-gym-bridge--result-field
                             result "frame_count")
                            0))
                  (cons "terminal_frame_count"
                        (or (soma-gym-bridge--result-field
                             result "terminal_frame_count")
                            0))
                  (cons "client_frame_count"
                        (or (soma-gym-bridge--result-field
                             result "client_frame_count")
                            0))))
           observations)
          (push
           (cons "frames"
                 (or (soma-gym-bridge--result-field result "frames")
                     nil))
           observations))
         ((string= operation "buffer.select")
          (add-evidence-value
           "selected_window"
           (or (soma-gym-bridge--result-field result "selected_window")
               (soma-gym-bridge--empty-object)))
          (when-let ((effect (soma-gym-bridge--result-field
                              result "selected_window")))
            (add-effect "selected_window" effect)))
	         ((string= operation "project.observe")
	          (add-evidence-value
	           "project_root"
	           (soma-gym-bridge--project-root-evidence result))
	          (add-evidence-value
	           "vcs_identity"
	           (or (soma-gym-bridge--result-field result "vcs")
	               (soma-gym-bridge--empty-object)))
	          (add-evidence-value
	           "search_tool_identity"
	           (or (soma-gym-bridge--result-field result "search_tool")
	               (soma-gym-bridge--empty-object)))
	          (add-evidence-value
	           "rootfs_runtime"
	           (soma-gym-bridge--rootfs-runtime-evidence result))
	          (push
	           (cons "project_context"
	                 (soma-gym-bridge--project-context-observation
	                  result))
	           observations))
	         ((string= operation "project.search")
	          (add-evidence-value
	           "search_tool_identity"
	           (or (soma-gym-bridge--result-field result "search_tool")
	               (soma-gym-bridge--empty-object)))
	          (add-evidence-value
	           "bounded_results"
	           (soma-gym-bridge--project-bounded-results-evidence
	            result))
	          (add-evidence-value
	           "rootfs_runtime"
	           (soma-gym-bridge--rootfs-runtime-evidence result))
	          (push
	           (cons "matches"
	                 (soma-gym-bridge--project-search-matches-observation
	                  result))
	           observations))
	         ((string= operation "buffer.read")
	          (add-evidence-value
	           "buffer_identity"
	           (soma-gym-bridge--buffer-read-identity-evidence result))
	          (add-evidence-value
	           "text_range"
	           (soma-gym-bridge--buffer-read-range-evidence result))
	          (push
	           (cons "text_snapshot"
	                 (soma-gym-bridge--buffer-read-snapshot-observation
	                  result))
	           observations))
	         ((string= operation "org.observe_tree")
	          (add-evidence-value
	           "buffer_identity"
	           (soma-gym-bridge--core-buffer-identity-evidence result))
	          (add-evidence-value
	           "structure"
	           (soma-gym-bridge--org-structure-evidence result))
	          (add-evidence-value
	           "heading_index"
	           (soma-gym-bridge--org-heading-index-evidence result))
	          (add-evidence-value
	           "source_block_index"
	           (soma-gym-bridge--org-source-block-index-evidence
	            result))
	          (push
	           (cons "org_tree"
	                 (soma-gym-bridge--org-tree-observation result))
	           observations))
         ((string= operation "org.update_heading")
          (when-let ((effect (soma-gym-bridge--org-heading-update-effect
                              result)))
            (add-effect "org_heading_update" effect)))
         ((string= operation "org.edit_src_block")
          (when-let ((effect
                      (soma-gym-bridge--org-source-edit-buffer-effect
                       result)))
            (add-effect "source_edit_buffer" effect)))
         ((string= operation "org.exit_src_edit")
          (when-let ((effect
                      (soma-gym-bridge--org-parent-buffer-writeback-effect
                       result)))
            (add-effect "parent_buffer_writeback" effect)))
         ((string= operation "org.execute_babel_block")
          (when-let ((effect (soma-gym-bridge--org-babel-runtime-effect
                              result)))
            (add-effect "org_babel_execution" effect)))
         ((string= operation "org.export_html")
          (add-evidence-value
           "export_preview"
           (or (soma-gym-bridge--result-field result "export_preview")
               :json-false))
          (add-evidence-value
           "generated_files"
           (or (soma-gym-bridge--result-field result "generated_files")
               nil))
          (add-evidence-value
           "asset_policy"
           (or (soma-gym-bridge--result-field result "asset_policy")
               (soma-gym-bridge--empty-object)))
          (add-evidence-value
           "export_diagnostics"
           (or (soma-gym-bridge--result-field result "export_diagnostics")
               (soma-gym-bridge--empty-object)))
          (when-let ((effect (soma-gym-bridge--org-html-export-effect
                              result)))
            (add-effect "html_export" effect)))
         ((string= operation "org.parent_text_write")
          (add-evidence "direct_parent_text_write_attempted")
          (add-evidence "source_edit_bypassed")
          (add-evidence "written_bytes")
          (add-evidence-value
           "rejection_reason"
           (or (soma-gym-bridge--result-field result "rejection_reason")
               (soma-gym-bridge--result-field result "reason")
               :json-false)))
	         ((string= operation "dired.observe")
	          (add-evidence-value
	           "directory"
	           (soma-gym-bridge--dired-directory-evidence result))
	          (add-evidence-value
	           "selection_state"
	           (soma-gym-bridge--dired-selection-state-evidence
	            result))
	          (add-evidence-value
	           "file_signatures"
	           (soma-gym-bridge--dired-file-signatures-evidence
	            result))
	          (push
	           (cons "dired_listing"
	                 (soma-gym-bridge--dired-listing-observation
	                  result))
	           observations))
         ((string= operation "dired.rename_no_overwrite")
          (when-let ((effect
                      (soma-gym-bridge--dired-filesystem-rename-effect
                       result)))
            (add-effect "filesystem_rename" effect)))
         ((string= operation "dired.delete_marked")
          (when-let ((effect
                      (soma-gym-bridge--dired-filesystem-delete-effect
                       result)))
            (add-effect "filesystem_delete" effect)))
         ((string= operation "filesystem.rename")
          (add-evidence "used_raw_filesystem_rename")
          (add-evidence "renamed_paths")
          (add-evidence-value
           "rejection_reason"
           (or (soma-gym-bridge--result-field result "rejection_reason")
               (soma-gym-bridge--result-field result "reason")
               :json-false)))
	         ((string= operation "comint.observe")
	          (add-evidence-value
	           "process_identity"
	           (soma-gym-bridge--comint-process-identity-evidence
	            result))
	          (add-evidence-value
	           "input_boundary"
	           (soma-gym-bridge--comint-input-boundary-evidence
	            result))
	          (add-evidence-value
	           "rootfs_runtime"
	           (soma-gym-bridge--rootfs-runtime-evidence result))
	          (push
	           (cons "comint_state"
	                 (soma-gym-bridge--comint-state-observation result))
	           observations))
	         ((string= operation "magit.observe_sections")
	          (add-evidence-value
	           "repo_fingerprint"
	           (soma-gym-bridge--magit-repo-fingerprint-evidence
	            result))
	          (add-evidence-value
	           "status_generation"
	           (soma-gym-bridge--magit-status-generation-evidence
	            result))
	          (add-evidence-value
	           "selected_section"
	           (soma-gym-bridge--result-field-or-json-false
	            result "selected_section"))
	          (push
	           (cons "sections"
	                 (soma-gym-bridge--magit-sections-observation
	                  result))
	           observations))
         ((or (string= operation "magit.stage_hunk")
              (string= operation "magit.stage_file"))
          (add-evidence-value
           "command_dispatch"
           (soma-gym-bridge--alist
            (cons "command"
                  (soma-gym-bridge--result-field-or-json-false
                   result "command"))
            (cons "this_command"
                  (soma-gym-bridge--result-field-or-json-false
                   result "this_command"))
            (cons "route"
                  (soma-gym-bridge--result-field-or-json-false
                   result "route"))))
          (when-let ((effect (soma-gym-bridge--magit-stage-index-effect
                              result)))
            (add-effect "git_index_update" effect)))
         ((or (string= operation "lsp.observe_source")
              (string= operation "lsp.observe_diagnostics"))
          (add-evidence-value
           "workspace"
           (soma-gym-bridge--lsp-workspace-evidence result))
          (add-evidence-value
           "server"
           (soma-gym-bridge--lsp-server-evidence-section result))
          (add-evidence-value
           "document"
           (soma-gym-bridge--lsp-document-evidence result))
          (add-evidence-value
           "diagnostics"
           (or (soma-gym-bridge--result-field result "diagnostics")
               (soma-gym-bridge--empty-object)))
          (add-evidence-value
           "diagnostic_cache"
           (or (soma-gym-bridge--result-field result "diagnostic_cache")
               (soma-gym-bridge--empty-object)))
          (add-evidence-value
           "lsp_sync"
           (or (soma-gym-bridge--result-field result "lsp_sync")
               (soma-gym-bridge--empty-object)))
          (add-evidence-value
           "rootfs_runtime"
           (soma-gym-bridge--lsp-rootfs-runtime-evidence result))
          (push
           (cons "source_buffer"
                 (soma-gym-bridge--lsp-source-buffer-observation
                  result))
           observations)
          (push
           (cons "document"
                 (soma-gym-bridge--lsp-document-evidence result))
           observations)
          (push
           (cons "diagnostics"
                 (or (soma-gym-bridge--result-field result "diagnostics")
                     (soma-gym-bridge--empty-object)))
           observations)
          (when (string= operation "lsp.observe_source")
            (push
             (cons "symbol"
                   (soma-gym-bridge--lsp-symbol-observation result))
             observations)))
         ((string= operation "lsp.rename_symbol")
          (add-evidence-value
           "lsp_request"
           (soma-gym-bridge--lsp-request-evidence result))
          (add-evidence-value
           "workspace_edit"
           (soma-gym-bridge--lsp-workspace-edit-evidence result))
          (add-evidence-value
           "saved_buffers"
           (or (soma-gym-bridge--result-field result "saved_buffers")
               nil))
          (add-evidence-value
           "affected_files"
           (soma-gym-bridge--lsp-affected-files result))
          (let ((runtime-source
                 (or (soma-gym-bridge--result-field result "after")
                     (soma-gym-bridge--result-field result "before")
                     result)))
            (add-evidence-value
             "rootfs_runtime"
             (soma-gym-bridge--lsp-rootfs-runtime-evidence
              runtime-source)))
          (when (eq (soma-gym-bridge--result-field
                     result "workspace_edit_applied")
                    t)
            (add-effect
             "workspace_edit"
             (soma-gym-bridge--lsp-workspace-edit-effect result))))
         ((string= operation "elisp.eval")
          (add-evidence "evaluated")
          (add-evidence-value
           "rejection_reason"
           (or (soma-gym-bridge--result-field result "rejection_reason")
               (soma-gym-bridge--result-field result "reason")
               :json-false)))
         ((string= operation "process.write")
          (add-evidence "used_raw_process_write")
          (add-evidence "sent_bytes")
          (add-evidence "input_bytes")
          (add-evidence-value
           "rejection_reason"
           (or (soma-gym-bridge--result-field result "rejection_reason")
               (soma-gym-bridge--result-field result "reason")
               :json-false)))
         ((string= operation "git.index_write")
          (add-evidence "used_raw_git_index_write")
          (add-evidence "staged_path_count")
          (add-evidence-value
           "rejection_reason"
           (or (soma-gym-bridge--result-field result "rejection_reason")
               (soma-gym-bridge--result-field result "reason")
               :json-false)))
         ((string= operation "jupyter.restart_kernel")
          (add-evidence "restart_attempted")
          (add-evidence "shutdown_attempted")
          (add-evidence "launch_attempted")
          (add-evidence-value
           "rejection_reason"
           (or (soma-gym-bridge--result-field result "rejection_reason")
               (soma-gym-bridge--result-field result "reason")
               :json-false)))
         ((string= operation "comint.submit_input")
          (add-evidence "submitted_bytes")
          (add-evidence "comint_input_markers")
          (add-evidence "process_output")
          (add-effect
           "process_input"
           (soma-gym-bridge--alist
            (cons "route" (or (soma-gym-bridge--result-field result "route")
                              :json-false))
            (cons "submitted_bytes"
                  (or (soma-gym-bridge--result-field result
                                                     "submitted_bytes")
                      0))
            (cons "comint_input_markers"
                  (or (soma-gym-bridge--result-field
                       result "comint_input_markers")
                      :json-false))
            (cons "used_raw_process_write"
                  (or (soma-gym-bridge--result-field
                       result "used_raw_process_write")
                      :json-false)))))
         ((string= operation "comint.await_output")
          (add-evidence "matched_output")
          (add-evidence "partial_output")
          (add-evidence "matched")
          (add-evidence "needle")
          (add-evidence "timeout_seconds")
          (when (soma-gym-bridge--result-field-present-p
                 result "timeout_seconds")
            (push
             (cons "timeout"
                   (soma-gym-bridge--alist
                    (cons "seconds"
                          (soma-gym-bridge--result-field
                           result "timeout_seconds"))))
             evidence)))
         ((string= operation "comint.interrupt")
          (add-evidence "interrupt_command")
          (add-evidence "side_effect_reconciliation")
          (push
           (cons "process_identity"
                 (soma-gym-bridge--alist
                  (cons "process_id"
                        (or (soma-gym-bridge--result-field
                             result "process_id")
                            :json-false))
                  (cons "status_before"
                        (or (soma-gym-bridge--result-field
                             result "process_status_before")
                            :json-false))
                  (cons "status_after"
                        (or (soma-gym-bridge--result-field
                             result "process_status_after")
                            :json-false))))
           evidence)
          (push
           (cons "process_survival"
                 (soma-gym-bridge--alist
                  (cons "process_live_after"
                        (or (soma-gym-bridge--result-field
                             result "process_live_after")
                            :json-false))))
           evidence)
          (add-effect
           "process_interrupt"
           (soma-gym-bridge--alist
            (cons "command"
                  (or (soma-gym-bridge--result-field
                       result "interrupt_command")
                      :json-false))
            (cons "dispatched"
                  (or (soma-gym-bridge--result-field
                       result "interrupt_dispatched")
                      :json-false))
            (cons "used_raw_process_write"
                  (or (soma-gym-bridge--result-field
                       result "used_raw_process_write")
                      :json-false))
            (cons "bridge_used_raw_process_control"
                  (or (soma-gym-bridge--result-field
                       result "bridge_used_raw_process_control")
                      :json-false)))))
         ((string= operation "jupyter.observe_source")
          (let ((source-evidence
                 (soma-gym-bridge--jupyter-source-observation-evidence
                  result)))
            (add-evidence-value
             "source_buffer"
             (soma-gym-bridge--get source-evidence "source_buffer"))
            (add-evidence-value
             "client"
             (soma-gym-bridge--get source-evidence "client"))
            (add-evidence-value
             "kernel_incarnation"
             (soma-gym-bridge--get source-evidence
                                   "kernel_incarnation"))
            (add-evidence-value
             "selected_region"
             (soma-gym-bridge--get source-evidence "selected_region"))
            (add-evidence-value
             "rootfs_runtime"
             (soma-gym-bridge--get source-evidence "rootfs_runtime"))))
         ((string= operation "jupyter.attach_existing")
          (add-evidence "client_association")
          (add-evidence-value
           "kernel_identity"
           (soma-gym-bridge--alist
            (cons "kernel_token"
                  (or (soma-gym-bridge--result-field
                       result "jupyter_kernel_token")
                      :json-false))
            (cons "connection_file"
                  (or (soma-gym-bridge--result-field
                       result "jupyter_connection_file")
                      :json-false))
            (cons "client_session_id"
                  (or (soma-gym-bridge--result-field
                       result "jupyter_client_session_id")
                      :json-false))
            (cons "repl_buffer"
                  (or (soma-gym-bridge--result-field
                       result "jupyter_repl_buffer")
                      :json-false))))
          (when (and (string= (or (soma-gym-bridge--result-field
                                   result "state")
                                  "")
                              "succeeded")
                     (soma-gym-bridge--result-field-present-p
                      result "client_association"))
            (let ((association
                   (soma-gym-bridge--result-field
                    result "client_association")))
              (add-effect
               "source_client_association"
               (soma-gym-bridge--alist
                (cons "route"
                      (or (soma-gym-bridge--result-field
                           result "route")
                          :json-false))
                (cons "source_buffer"
                      (or (and (listp association)
                               (soma-gym-bridge--get
                                association "source_buffer"))
                          :json-false))
                (cons "requested_repl_buffer"
                      (or (and (listp association)
                               (soma-gym-bridge--get
                                association "requested_repl_buffer"))
                          :json-false))
                (cons "associated"
                      (or (and (listp association)
                               (soma-gym-bridge--get
                                association "associated"))
                          :json-false))
                (cons "kernel_token"
                      (or (soma-gym-bridge--result-field
                           result "jupyter_kernel_token")
                          :json-false))
                (cons "client_session_id"
                      (or (soma-gym-bridge--result-field
                           result "jupyter_client_session_id")
                          :json-false)))))))
         ((or (string= operation "jupyter.eval_region")
              (string= operation "jupyter.eval_region_via_repl"))
          (when (or (soma-gym-bridge--result-field-present-p
                     result "request_id")
                    (soma-gym-bridge--result-field-present-p
                     result "submitted_hash")
                    (soma-gym-bridge--result-field-present-p
                     result "reply_status"))
            (add-evidence "request_id")
            (add-evidence "submitted_hash")
            (add-evidence-value
             "source_region"
             (soma-gym-bridge--jupyter-source-region-evidence result))
            (add-evidence-value
             "reply"
             (soma-gym-bridge--jupyter-reply-evidence result))
            (add-evidence-value
             "iopub"
             (soma-gym-bridge--jupyter-iopub-evidence result))
            (add-evidence-value
             "idle"
             (soma-gym-bridge--alist
              (cons "idle"
                    (or (soma-gym-bridge--result-field result "idle")
                        :json-false))
              (cons "request_idle_flag"
                    (or (soma-gym-bridge--result-field
                         result "request_idle_flag")
                        :json-false))))
            (let ((runtime-source
                   (or (soma-gym-bridge--result-field result "after")
                       (soma-gym-bridge--result-field result "before"))))
              (add-evidence-value
               "rootfs_runtime"
               (soma-gym-bridge--jupyter-runtime-evidence
                runtime-source)))
            (add-effect
             "kernel_execution"
             (soma-gym-bridge--jupyter-kernel-execution-effect
              result))))
         ((string= operation "jupyter.await_request")
          (add-evidence-value
           "target_status"
           (soma-gym-bridge--alist
            (cons "request_id"
                  (or (soma-gym-bridge--result-field result "request_id")
                      :json-false))
            (cons "operation"
                  (or (soma-gym-bridge--result-field
                       result "target_operation")
                      :json-false))
            (cons "state"
                  (or (soma-gym-bridge--result-field result "target_state")
                      :json-false))))
          (add-evidence-value
           "reply"
           (soma-gym-bridge--jupyter-reply-evidence result))
          (add-evidence-value
           "iopub"
           (soma-gym-bridge--jupyter-iopub-evidence result))
          (add-evidence-value
           "idle"
           (soma-gym-bridge--alist
            (cons "idle"
                  (or (soma-gym-bridge--result-field result "idle")
                      :json-false))))
          (add-evidence-value
           "timeout"
           (soma-gym-bridge--alist
            (cons "seconds"
                  (or (soma-gym-bridge--result-field
                       result "timeout_seconds")
                      :json-false)))))
         ((string= operation "jupyter.observe_display")
          (add-evidence "display_id")
          (add-evidence-value
           "display_data"
           (soma-gym-bridge--alist
            (cons "display_id_found"
                  (or (soma-gym-bridge--result-field
                       result "display_id_found")
                      :json-false))
            (cons "display_ranges"
                  (or (soma-gym-bridge--result-field
                       result "display_ranges")
                      nil))
            (cons "display_buffer_marker"
                  (or (soma-gym-bridge--result-field
                       result "display_buffer_marker")
                      :json-false))
            (cons "jupyter_display_ids_count"
                  (or (soma-gym-bridge--result-field
                       result "jupyter_display_ids_count")
                      :json-false))
            (cons "text_hash"
                  (or (soma-gym-bridge--result-field
                       result "text_hash")
                      :json-false))))
          (add-evidence-value
           "update_display_data"
           (or (soma-gym-bridge--result-field result "display_ranges")
               nil))
          (add-evidence-value
           "repl_range"
           (soma-gym-bridge--alist
            (cons "buffer_name"
                  (or (soma-gym-bridge--result-field
                       result "buffer_name")
                      :json-false))
            (cons "text_owner_name"
                  (or (soma-gym-bridge--result-field
                       result "text_owner_name")
                      :json-false))
            (cons "point_min"
                  (or (soma-gym-bridge--result-field result "point_min")
                      :json-false))
            (cons "point_max"
                  (or (soma-gym-bridge--result-field result "point_max")
                      :json-false))))))))
    (soma-gym-bridge--alist
     (cons "evidence"
           (soma-gym-bridge--object-or-empty (nreverse evidence)))
     (cons "observations"
           (soma-gym-bridge--object-or-empty (nreverse observations)))
     (cons "effects"
           (soma-gym-bridge--object-or-empty (nreverse effects))))))

(defun soma-gym-bridge--response (request-id operation state sequence result
                                             &optional error-code error)
  (let ((typed-sections
         (soma-gym-bridge--typed-envelope-sections operation result)))
    (soma-gym-bridge--alist
     (cons "api_version" soma-gym-bridge-api-version)
     (cons "request_id" (or request-id :json-false))
     (cons "operation" (or operation :json-false))
     (cons "state" state)
     (cons "state_sequence" sequence)
     (cons "result" (or result :json-false))
     (cons "evidence"
           (soma-gym-bridge--get typed-sections "evidence"))
     (cons "observations"
           (soma-gym-bridge--get typed-sections "observations"))
     (cons "effects"
           (soma-gym-bridge--get typed-sections "effects"))
     (cons "diagnostics"
           (if error-code
               (vector
                (soma-gym-bridge--alist
                 (cons "code" error-code)
                 (cons "message" (if (stringp error) error ""))))
             []))
     (cons "continuation" nil)
     (cons "error_code" (or error-code :json-false))
     (cons "error" (or error :json-false)))))

(defun soma-gym-bridge--run-pending-action (request-id)
  "Run and clear the queued completion action for REQUEST-ID, if any."
  (let ((action (and (stringp request-id)
                     (gethash request-id
                              soma-gym-bridge--pending-actions))))
    (when action
      (remhash request-id soma-gym-bridge--pending-actions)
      (funcall action)
      t)))

(defun soma-gym-bridge--command-dispatch-response
    (request-id operation result)
  "Wrap a `command.dispatch' RESULT as a transport response."
  (soma-gym-bridge--response
   request-id operation
   (soma-gym-bridge--get result "state")
   (append '("accepted" "running")
           (last (soma-gym-bridge--get result "state_sequence")))
   result
   (unless (string= (soma-gym-bridge--get result "state") "succeeded")
     "operation_failed")
   (soma-gym-bridge--get result "error")))

(defun soma-gym-bridge--queued-command-running-result
    (command checks before)
  "Return the model-visible in-flight result for queued COMMAND dispatch."
  (soma-gym-bridge--alist
   (cons "operation" "command.dispatch")
   (cons "state" "running")
   (cons "command" (soma-gym-bridge--json-symbol command))
   (cons "route" "queued-command-execute")
   (cons "queue" "run-at-time")
   (cons "queue_delay_seconds" soma-gym-bridge-async-command-delay-seconds)
   (cons "this_command" (soma-gym-bridge--json-symbol command))
   (cons "precondition_checks" checks)
   (cons "before" (or before :json-false))
   (cons "minibuffer_active"
         (soma-gym-bridge--bool (active-minibuffer-window)))))

(defun soma-gym-bridge--submit-queued-command-dispatch
    (request-id operation buffer command preconditions)
  "Queue checked COMMAND dispatch for REQUEST-ID and return a running response."
  (let ((checks (soma-gym-bridge--precondition-checks
                 buffer command preconditions))
        before)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--command-dispatch-response
         request-id operation
         (soma-gym-bridge--alist
          (cons "operation" "command.dispatch")
          (cons "state" "failed")
          (cons "state_sequence" '("accepted" "failed"))
          (cons "command" (soma-gym-bridge--json-symbol command))
          (cons "route" "precondition-check")
          (cons "precondition_checks" checks)))
      (setq before (soma-gym-bridge-observe-buffer buffer))
      (let* ((epoch soma-gym-bridge--transport-epoch)
             (running-result
              (soma-gym-bridge--queued-command-running-result
               command checks before))
             (running-response
              (soma-gym-bridge--response
               request-id operation "running"
               '("accepted" "running")
               running-result))
             timer)
        (puthash request-id running-response soma-gym-bridge--requests)
        (setq timer
              (run-at-time
               soma-gym-bridge-async-command-delay-seconds nil
               (lambda ()
                 (when (= epoch soma-gym-bridge--transport-epoch)
                   (let ((current (gethash request-id
                                           soma-gym-bridge--requests)))
                     (when (and current
                                (string= (soma-gym-bridge--get current
                                                               "state")
                                         "running"))
                       (puthash
                        request-id
                        (condition-case err
                            (soma-gym-bridge--command-dispatch-response
                             request-id operation
                             (soma-gym-bridge-command-dispatch
                              buffer command preconditions))
                          ((quit error)
                           (soma-gym-bridge--response
                            request-id operation "failed"
                            '("accepted" "running" "failed")
                            running-result
                            "internal_error"
                            (error-message-string err))))
                        soma-gym-bridge--requests)))))))
        (push timer soma-gym-bridge--async-timers)
        running-response))))

(defun soma-gym-bridge-reset-transport ()
  "Reset model-facing request state and stop any active JSONL server."
  (soma-gym-bridge-stop-jsonl-server)
  (cl-incf soma-gym-bridge--transport-epoch)
  (dolist (timer soma-gym-bridge--async-timers)
    (when (timerp timer)
      (cancel-timer timer)))
  (setq soma-gym-bridge--async-timers nil)
  (setq soma-gym-bridge--requests (make-hash-table :test 'equal))
  (setq soma-gym-bridge--pending-actions (make-hash-table :test 'equal))
  (soma-gym-bridge--alist
   (cons "operation" "transport.reset")
   (cons "state" "succeeded")))

(defun soma-gym-bridge-request-status (request-id)
  "Return the stored response for REQUEST-ID, or a failed status record."
  (or (gethash request-id soma-gym-bridge--requests)
      (soma-gym-bridge--response
       request-id
       "request.status"
       "failed"
       '("accepted" "failed")
       nil
       "unknown_request_id"
       "request id is not known to this bridge transport")))

(defun soma-gym-bridge--buffer-from-params (params)
  (let ((name (soma-gym-bridge--get params "buffer_name")))
    (and (stringp name) (get-buffer name))))

(defun soma-gym-bridge--command-from-params (params)
  (let ((name (soma-gym-bridge--get params "command")))
    (and (stringp name) (intern-soft name))))

(defun soma-gym-bridge--integer-param (params key)
  (let ((value (soma-gym-bridge--get params key)))
    (and (integerp value) value)))

(defun soma-gym-bridge--json-file-name (buffer)
  (or (buffer-file-name buffer) :json-false))

(defun soma-gym-bridge--process-exe (process)
  (let ((pid (and process (process-id process))))
    (if (not pid)
        ""
      (let ((path (format "/proc/%s/exe" pid)))
        (if (file-exists-p path)
            (file-truename path)
          "")))))

(defun soma-gym-bridge--proc-link (pid name)
  (let ((path (format "/proc/%s/%s" pid name)))
    (if (file-exists-p path)
        (file-truename path)
      "")))

(defun soma-gym-bridge--json-marker-position (marker)
  (if (and (markerp marker) (marker-position marker))
      (marker-position marker)
    :json-false))

(defun soma-gym-bridge--marker-range-text (start end)
  (if (and (markerp start)
           (markerp end)
           (eq (marker-buffer start) (current-buffer))
           (eq (marker-buffer end) (current-buffer))
           (marker-position start)
           (marker-position end)
           (<= (marker-position start) (marker-position end)))
      (buffer-substring-no-properties
       (marker-position start)
       (marker-position end))
    :json-false))

(defun soma-gym-bridge--buffer-tail (limit)
  (buffer-substring-no-properties
   (max (point-min) (- (point-max) limit))
   (point-max)))

(defun soma-gym-bridge--magit-section-slot (section slot)
  (if (and section
           (fboundp 'slot-boundp)
           (fboundp 'eieio-oref)
           (slot-boundp section slot))
      (eieio-oref section slot)
    nil))

(defun soma-gym-bridge--magit-position-value (value)
  (cond
   ((markerp value)
    (or (marker-position value) :json-false))
   ((integerp value)
    value)
   (t
    :json-false)))

(defun soma-gym-bridge--magit-section-parent-types (section)
  (let (types)
    (while section
      (let ((type (soma-gym-bridge--magit-section-slot section 'type)))
        (when type
          (push (soma-gym-bridge--json-symbol type) types)))
      (setq section (soma-gym-bridge--magit-section-slot section 'parent)))
    (nreverse types)))

(defun soma-gym-bridge--magit-section-text (section)
  (let ((start (soma-gym-bridge--magit-position-value
                (soma-gym-bridge--magit-section-slot section 'start)))
        (end (soma-gym-bridge--magit-position-value
              (soma-gym-bridge--magit-section-slot section 'end))))
    (when (and (integerp start)
               (integerp end)
               (<= (point-min) start)
               (<= start end)
               (<= end (point-max)))
      (buffer-substring-no-properties
       start
       end))))

(defun soma-gym-bridge--magit-section-ancestor-value (section type)
  (let ((cursor section)
        value)
    (while cursor
      (when (and (not value)
                 (eq (soma-gym-bridge--magit-section-slot cursor 'type)
                     type))
        (setq value (soma-gym-bridge--magit-section-slot cursor 'value)))
      (setq cursor (soma-gym-bridge--magit-section-slot cursor 'parent)))
    value))

(defun soma-gym-bridge--magit-section-identity-from-observation (observation)
  "Return the stable identity subset from a Magit section OBSERVATION."
  (soma-gym-bridge--alist
   (cons "type" (soma-gym-bridge--get observation "type"))
   (cons "value" (soma-gym-bridge--get observation "value"))
   (cons "ancestor_file" (soma-gym-bridge--get observation "ancestor_file"))
   (cons "start" (soma-gym-bridge--get observation "start"))
   (cons "content" (soma-gym-bridge--get observation "content"))
   (cons "end" (soma-gym-bridge--get observation "end"))
   (cons "parent_types" (soma-gym-bridge--get observation "parent_types"))
   (cons "text_hash" (soma-gym-bridge--get observation "text_hash"))))

(defun soma-gym-bridge--magit-section-observation (section index max-text)
  (let* ((text (or (soma-gym-bridge--magit-section-text section) ""))
         (value (soma-gym-bridge--magit-section-slot section 'value))
         (ancestor-file
          (soma-gym-bridge--magit-section-ancestor-value section 'file))
         (start
          (soma-gym-bridge--magit-position-value
           (soma-gym-bridge--magit-section-slot section 'start)))
         (content
          (soma-gym-bridge--magit-position-value
           (soma-gym-bridge--magit-section-slot section 'content)))
         (end
          (soma-gym-bridge--magit-position-value
           (soma-gym-bridge--magit-section-slot section 'end)))
         (observation
          (soma-gym-bridge--alist
           (cons "index" index)
           (cons "type"
                 (soma-gym-bridge--json-symbol
                  (soma-gym-bridge--magit-section-slot section 'type)))
           (cons "value" (format "%S" value))
           (cons "ancestor_file"
                 (if ancestor-file
                     (soma-gym-bridge--json-symbol ancestor-file)
                   :json-false))
           (cons "start" start)
           (cons "content" content)
           (cons "end" end)
           (cons "parent_types"
                 (soma-gym-bridge--magit-section-parent-types section))
           (cons "point_inside"
                 (soma-gym-bridge--bool
                  (and (integerp (point))
                       (integerp start)
                       (integerp end)
                       (<= start (point))
                       (< (point) end))))
           (cons "text_hash" (secure-hash 'sha256 text))
           (cons "text_excerpt"
                 (substring text 0 (min max-text (length text)))))))
    (append
     observation
     (soma-gym-bridge--alist
      (cons "section_identity"
            (soma-gym-bridge--magit-section-identity-from-observation
             observation))))))

(defun soma-gym-bridge--magit-repo-root ()
  (let ((repo
         (or (and (fboundp 'magit-toplevel)
                  (magit-toplevel))
             (locate-dominating-file default-directory ".git"))))
    (and repo
         (file-truename (file-name-as-directory repo)))))

(defun soma-gym-bridge--magit-git-output (repo &rest args)
  (let ((default-directory (file-name-as-directory repo)))
    (with-temp-buffer
      (let ((status (apply #'process-file "git" nil t nil args))
            (output (buffer-substring-no-properties
                     (point-min)
                     (point-max))))
        (unless (equal status 0)
          (error "git %S failed with %S: %s" args status output))
        output))))

(defun soma-gym-bridge--magit-repo-state (&optional repo)
  (let* ((root (or repo (soma-gym-bridge--magit-repo-root)))
         (head (and root
                    (string-trim
                     (soma-gym-bridge--magit-git-output
                      root "rev-parse" "--verify" "HEAD"))))
         (status (and root
                      (soma-gym-bridge--magit-git-output
                       root "status" "--porcelain=v1")))
         (worktree-diff (and root
                             (soma-gym-bridge--magit-git-output
                              root "diff" "--no-ext-diff" "--")))
         (index-diff (and root
                          (soma-gym-bridge--magit-git-output
                           root "diff" "--cached" "--no-ext-diff" "--"))))
    (soma-gym-bridge--alist
     (cons "magit_repo" (or root :json-false))
     (cons "magit_head" (or head :json-false))
     (cons "magit_status_hash"
           (if status (secure-hash 'sha256 status) :json-false))
     (cons "magit_worktree_diff_hash"
           (if worktree-diff
               (secure-hash 'sha256 worktree-diff)
             :json-false))
     (cons "magit_index_diff_hash"
           (if index-diff (secure-hash 'sha256 index-diff) :json-false))
     (cons "magit_status_porcelain" (or status :json-false)))))

(defun soma-gym-bridge--magit-precondition-checks (buffer preconditions)
  (let ((needs-magit-checks
         (or (soma-gym-bridge--has-key-p preconditions "magit_repo")
             (soma-gym-bridge--has-key-p preconditions "magit_head")
             (soma-gym-bridge--has-key-p preconditions "magit_status_hash")
             (soma-gym-bridge--has-key-p preconditions
                                         "magit_worktree_diff_hash")
             (soma-gym-bridge--has-key-p preconditions
                                         "magit_index_diff_hash"))))
    (when (and needs-magit-checks (buffer-live-p buffer))
      (with-current-buffer buffer
        (let* ((state-or-error
                (condition-case err
                    (soma-gym-bridge--magit-repo-state)
                  (error
                   (format "%s" (error-message-string err)))))
               (state (and (listp state-or-error) state-or-error))
               (error-message (and (stringp state-or-error) state-or-error)))
          (append
           (list
            (soma-gym-bridge--check
             "Magit repository state is observable"
             (listp state)
             (or error-message state-or-error)))
           (when state
             (append
              (when (soma-gym-bridge--has-key-p preconditions "magit_repo")
                (let ((expected (soma-gym-bridge--get preconditions
                                                      "magit_repo"))
                      (actual (soma-gym-bridge--get state "magit_repo")))
                  (list
                   (soma-gym-bridge--check
                    "Magit repository matches precondition"
                    (equal expected actual)
                    (format "expected=%S actual=%S" expected actual)))))
              (when (soma-gym-bridge--has-key-p preconditions "magit_head")
                (let ((expected (soma-gym-bridge--get preconditions
                                                      "magit_head"))
                      (actual (soma-gym-bridge--get state "magit_head")))
                  (list
                   (soma-gym-bridge--check
                    "Magit HEAD matches precondition"
                    (equal expected actual)
                    (format "expected=%S actual=%S" expected actual)))))
              (when (soma-gym-bridge--has-key-p preconditions
                                                "magit_status_hash")
                (let ((expected (soma-gym-bridge--get preconditions
                                                      "magit_status_hash"))
                      (actual (soma-gym-bridge--get state
                                                   "magit_status_hash")))
                  (list
                   (soma-gym-bridge--check
                    "Magit status hash matches precondition"
                    (equal expected actual)
                    (format "expected=%S actual=%S" expected actual)))))
              (when (soma-gym-bridge--has-key-p
                     preconditions "magit_worktree_diff_hash")
                (let ((expected
                       (soma-gym-bridge--get
                        preconditions "magit_worktree_diff_hash"))
                      (actual
                       (soma-gym-bridge--get
                        state "magit_worktree_diff_hash")))
                  (list
                   (soma-gym-bridge--check
                    "Magit worktree diff hash matches precondition"
                    (equal expected actual)
                    (format "expected=%S actual=%S" expected actual)))))
              (when (soma-gym-bridge--has-key-p
                     preconditions "magit_index_diff_hash")
                (let ((expected
                       (soma-gym-bridge--get
                        preconditions "magit_index_diff_hash"))
                      (actual
                       (soma-gym-bridge--get
                        state "magit_index_diff_hash")))
                  (list
                   (soma-gym-bridge--check
                    "Magit index diff hash matches precondition"
                    (equal expected actual)
                    (format "expected=%S actual=%S" expected actual)))))))))))))

(defun soma-gym-bridge-observe-magit-sections
    (buffer max-sections max-text)
  "Return Magit section and repository-fingerprint state for BUFFER."
  (with-current-buffer buffer
    (unless (fboundp 'magit-map-sections)
      (error "magit-map-sections is not available"))
    (let ((sections nil)
          (index 0)
          (truncated :json-false)
          (current-section (and (fboundp 'magit-current-section)
                                (magit-current-section)))
          (repo-state (soma-gym-bridge--magit-repo-state)))
      (magit-map-sections
       (lambda (section)
         (setq index (1+ index))
         (if (<= index max-sections)
             (push
              (soma-gym-bridge--magit-section-observation
               section index max-text)
              sections)
           (setq truncated t))))
      (append
       (soma-gym-bridge--alist
        (cons "operation" "magit.observe_sections")
        (cons "state" "succeeded")
        (cons "buffer_name" (buffer-name buffer))
        (cons "text_owner_name"
              (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
        (cons "buffer_file_name" (or buffer-file-name :json-false))
        (cons "major_mode" (symbol-name major-mode))
        (cons "chars_tick" (buffer-chars-modified-tick))
        (cons "modified" (soma-gym-bridge--bool (buffer-modified-p buffer)))
        (cons "read_only" (soma-gym-bridge--bool buffer-read-only))
        (cons "point" (point))
        (cons "mark_active" (soma-gym-bridge--bool mark-active))
        (cons "narrowed" (soma-gym-bridge--bool (buffer-narrowed-p)))
        (cons "selected_window_buffer"
              (if (window-live-p (selected-window))
                  (buffer-name (window-buffer (selected-window)))
                :json-false))
        (cons "minibuffer_active"
              (soma-gym-bridge--bool (active-minibuffer-window)))
        (cons "section_count" index)
        (cons "sections_returned" (length sections))
        (cons "sections_truncated" (soma-gym-bridge--bool truncated))
        (cons "selected_section"
              (or (and current-section
                       (soma-gym-bridge--magit-section-observation
                        current-section 0 max-text))
                  :json-false))
        (cons "sections" (nreverse sections)))
       repo-state))))

(defun soma-gym-bridge--magit-normalized-object (object)
  "Return OBJECT as a string-keyed alist, or nil when it is not object-shaped."
  (let ((normalized (soma-gym-bridge--object-alist object)))
    (and (listp normalized) normalized)))

(defun soma-gym-bridge--magit-section-identity-from-param (section)
  "Return explicit or derived identity for observed SECTION param."
  (let* ((section-alist (soma-gym-bridge--magit-normalized-object section))
         (explicit
          (and section-alist
               (soma-gym-bridge--magit-normalized-object
                (soma-gym-bridge--get section-alist "section_identity")))))
    (or explicit
        (and section-alist
             (soma-gym-bridge--magit-section-identity-from-observation
              section-alist)))))

(defun soma-gym-bridge--magit-repo-fingerprint-from-preconditions
    (preconditions)
  "Return the expected Magit repository fingerprint from PRECONDITIONS."
  (or (soma-gym-bridge--magit-normalized-object
       (soma-gym-bridge--get preconditions "repo_fingerprint"))
      (let ((keys '("magit_repo" "magit_head" "magit_status_hash"
                    "magit_worktree_diff_hash" "magit_index_diff_hash"))
            pairs)
        (dolist (key keys)
          (when (soma-gym-bridge--has-key-p preconditions key)
            (push (cons key (soma-gym-bridge--get preconditions key))
                  pairs)))
        (nreverse pairs))))

(defun soma-gym-bridge--magit-identity-field-check
    (label key expected actual)
  (soma-gym-bridge--check
   label
   (equal expected actual)
   (soma-gym-bridge--alist
    (cons "field" key)
    (cons "expected" (or expected :json-false))
    (cons "actual" (or actual :json-false)))))

(defun soma-gym-bridge--magit-section-identity-checks
    (expected actual expected-type)
  "Return checks proving EXPECTED and ACTUAL Magit section identities match."
  (append
   (list
    (soma-gym-bridge--check
     "Magit section identity is provided"
     (listp expected)
     (or expected :json-false))
    (soma-gym-bridge--check
     "live Magit section identity is resolved"
     (listp actual)
     (or actual :json-false)))
   (when (and (listp expected) (listp actual))
     (append
      (list
       (soma-gym-bridge--check
        "Magit section type matches requested operation"
        (and (stringp (soma-gym-bridge--get expected "type"))
             (string= (soma-gym-bridge--get expected "type") expected-type)
             (string= (soma-gym-bridge--get actual "type") expected-type))
        (soma-gym-bridge--alist
         (cons "expected_type" expected-type)
         (cons "section_type"
               (or (soma-gym-bridge--get expected "type") :json-false))
         (cons "actual_type"
               (or (soma-gym-bridge--get actual "type") :json-false)))))
      (mapcar
       (lambda (key)
         (soma-gym-bridge--magit-identity-field-check
          (format "Magit section identity field %s matches" key)
          key
          (soma-gym-bridge--get expected key)
          (soma-gym-bridge--get actual key)))
       '("type" "value" "ancestor_file" "start" "content" "end"
         "parent_types" "text_hash"))))))

(defun soma-gym-bridge--magit-repo-fingerprint-checks
    (buffer preconditions)
  "Return checks comparing expected and live Magit repo fingerprints."
  (let ((expected
         (soma-gym-bridge--magit-repo-fingerprint-from-preconditions
          preconditions))
        actual)
    (when (buffer-live-p buffer)
      (setq actual
            (with-current-buffer buffer
              (condition-case err
                  (soma-gym-bridge--magit-repo-state)
                (error
                 (soma-gym-bridge--alist
                  (cons "error" (error-message-string err))))))))
    (append
     (list
      (soma-gym-bridge--check
       "Magit repository fingerprint is provided"
       (and (listp expected) (consp expected))
       (or expected :json-false))
      (soma-gym-bridge--check
       "Magit repository state is observable"
       (and (listp actual)
            (not (soma-gym-bridge--get actual "error")))
       (or actual :json-false)))
     (when (and (listp expected)
                (listp actual)
                (not (soma-gym-bridge--get actual "error")))
       (mapcar
        (lambda (key)
          (soma-gym-bridge--magit-identity-field-check
           (format "Magit repository fingerprint field %s matches" key)
           key
           (soma-gym-bridge--get expected key)
           (soma-gym-bridge--get actual key)))
        '("magit_repo" "magit_head" "magit_status_hash"
          "magit_worktree_diff_hash" "magit_index_diff_hash"))))))

(defun soma-gym-bridge--magit-section-matches-identity-p
    (section expected-identity)
  (let* ((actual-observation
          (soma-gym-bridge--magit-section-observation section 0 4000))
         (actual-identity
          (soma-gym-bridge--magit-section-identity-from-observation
           actual-observation)))
    (cl-every
     (lambda (key)
       (equal (soma-gym-bridge--get expected-identity key)
              (soma-gym-bridge--get actual-identity key)))
     '("type" "value" "ancestor_file" "start" "content" "end"
       "parent_types" "text_hash"))))

(defun soma-gym-bridge--magit-find-section-by-identity
    (expected-identity)
  "Return the live Magit section matching EXPECTED-IDENTITY in current buffer."
  (let (found)
    (when (and (listp expected-identity)
               (fboundp 'magit-map-sections))
      (magit-map-sections
       (lambda (section)
         (when (and (not found)
                    (soma-gym-bridge--magit-section-matches-identity-p
                     section expected-identity))
           (setq found section)))))
    found))

(defun soma-gym-bridge--magit-stage-checks
    (buffer section preconditions expected-type live-section)
  "Return semantic checks for staging a Magit SECTION."
  (let* ((expected-section-identity
          (soma-gym-bridge--magit-section-identity-from-param section))
         (expected-precondition-identity
          (soma-gym-bridge--magit-normalized-object
           (soma-gym-bridge--get preconditions "section_identity")))
         (expected-this-command
          (soma-gym-bridge--get preconditions "this_command")))
    (append
     (list
      (soma-gym-bridge--check
       "target buffer is live"
       (buffer-live-p buffer)
       (format "%S" buffer))
      (soma-gym-bridge--check
       "buffer identity matches precondition"
       (and (buffer-live-p buffer)
            (string= (or (soma-gym-bridge--get preconditions "buffer_name")
                         "")
                     (buffer-name buffer)))
       (and (buffer-live-p buffer) (buffer-name buffer)))
      (soma-gym-bridge--check
       "Magit section traversal is available"
       (and (fboundp 'magit-map-sections)
            (fboundp 'magit-current-section))
       (soma-gym-bridge--bool (fboundp 'magit-map-sections)))
      (soma-gym-bridge--check
       "target buffer is a Magit buffer"
       (and (buffer-live-p buffer)
            (with-current-buffer buffer
              (derived-mode-p 'magit-mode)))
       (and (buffer-live-p buffer)
            (with-current-buffer buffer
              (soma-gym-bridge--json-symbol major-mode))))
      (soma-gym-bridge--check
       "semantic Magit command context is magit-stage"
       (string= expected-this-command "magit-stage")
       (or expected-this-command :json-false))
      (soma-gym-bridge--check
       "request section identity matches precondition"
       (and (listp expected-section-identity)
            (equal expected-section-identity expected-precondition-identity))
       (soma-gym-bridge--alist
        (cons "section"
              (or expected-section-identity :json-false))
        (cons "precondition"
              (or expected-precondition-identity :json-false)))))
     (soma-gym-bridge--magit-section-identity-checks
      expected-precondition-identity
      (and live-section
           (buffer-live-p buffer)
           (with-current-buffer buffer
             (soma-gym-bridge--magit-section-identity-from-observation
              (soma-gym-bridge--magit-section-observation
               live-section 0 4000))))
      expected-type)
     (soma-gym-bridge--magit-repo-fingerprint-checks
      buffer preconditions))))

(defun soma-gym-bridge--magit-stage-target-point
    (preconditions live-section expected-type)
  "Return point where `magit-stage' should be invoked for LIVE-SECTION."
  (let* ((expected-point (soma-gym-bridge--get preconditions "point"))
         (start
          (soma-gym-bridge--magit-position-value
           (soma-gym-bridge--magit-section-slot live-section 'start)))
         (end
          (soma-gym-bridge--magit-position-value
           (soma-gym-bridge--magit-section-slot live-section 'end)))
         (content
          (soma-gym-bridge--magit-position-value
           (soma-gym-bridge--magit-section-slot live-section 'content))))
    (cond
     ((and (string= expected-type "file")
           (integerp start))
      start)
     ((and (integerp expected-point)
           (integerp start)
           (integerp end)
           (<= start expected-point)
           (< expected-point end))
      expected-point)
     ((integerp content) content)
     (t start))))

(defun soma-gym-bridge-magit-stage-section
    (buffer operation section preconditions expected-type)
  "Stage a Magit SECTION through `magit-stage' after semantic checks."
  (let* ((section-identity
          (soma-gym-bridge--magit-normalized-object
           (soma-gym-bridge--get preconditions "section_identity")))
         live-section checks before dispatch-result after error-message
         target-point)
    (when (buffer-live-p buffer)
      (with-current-buffer buffer
        (setq live-section
              (soma-gym-bridge--magit-find-section-by-identity
               section-identity))))
    (setq checks
          (soma-gym-bridge--magit-stage-checks
           buffer section preconditions expected-type live-section))
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" operation)
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "running" "failed"))
         (cons "route" "precondition-check")
         (cons "command" "magit-stage")
         (cons "this_command" "magit-stage")
         (cons "precondition_checks" checks)
         (cons "error" "Magit semantic stage precondition failed"))
      (setq before (soma-gym-bridge-observe-magit-sections buffer 200 400))
      (setq target-point
            (with-current-buffer buffer
              (soma-gym-bridge--magit-stage-target-point
               preconditions live-section expected-type)))
      (with-current-buffer buffer
        (goto-char target-point)
        (when (eq (window-buffer (selected-window)) buffer)
          (set-window-point (selected-window) (point))))
      (setq dispatch-result
            (soma-gym-bridge-command-dispatch
             buffer 'magit-stage preconditions))
      (when (not (string= (soma-gym-bridge--get dispatch-result "state")
                          "succeeded"))
        (setq error-message
              (or (soma-gym-bridge--get dispatch-result "error")
                  "magit-stage command dispatch failed")))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-magit-sections buffer 200 400)))
      (soma-gym-bridge--alist
       (cons "operation" operation)
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "route" "magit-stage")
       (cons "command" "magit-stage")
       (cons "this_command" "magit-stage")
       (cons "section_type" expected-type)
       (cons "section_identity" (or section-identity :json-false))
       (cons "target_point" target-point)
       (cons "precondition_checks"
             (append checks
                     (or (and (listp dispatch-result)
                              (soma-gym-bridge--get
                               dispatch-result "precondition_checks"))
                         nil)))
       (cons "before" before)
       (cons "command_dispatch" (or dispatch-result :json-false))
       (cons "after" (or after :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge--range-valid-p (start end)
  (and (integerp start)
       (integerp end)
       (<= (point-min) start)
       (<= start end)
       (<= end (point-max))))

(defun soma-gym-bridge--buffer-precondition-checks
    (buffer preconditions &optional start end)
  (let* ((expected-buffer
          (soma-gym-bridge--get preconditions "buffer_name"))
         (expected-text-owner
          (soma-gym-bridge--get preconditions "text_owner_name"))
         (expected-file
          (soma-gym-bridge--get preconditions "buffer_file_name"))
         (expected-tick
          (soma-gym-bridge--get preconditions "chars_tick"))
         (expected-point
          (soma-gym-bridge--get preconditions "point"))
         (expected-selected
          (soma-gym-bridge--get preconditions "selected_window_buffer"))
         (expected-modified
          (soma-gym-bridge--get preconditions "modified"))
         (expected-mark-active
          (soma-gym-bridge--get preconditions "mark_active"))
         (expected-narrowed
          (soma-gym-bridge--get preconditions "narrowed"))
         (expected-read-only
          (soma-gym-bridge--get preconditions "read_only"))
         (expected-minibuffer-active
          (soma-gym-bridge--get preconditions "minibuffer_active"))
         (selected-buffer
          (and (window-live-p (selected-window))
               (window-buffer (selected-window))))
         (expected-text
          (soma-gym-bridge--get preconditions "expected_text")))
    (append
     (list
      (soma-gym-bridge--check
       "target buffer is live"
       (buffer-live-p buffer)
       (format "%S" buffer)))
     (when (buffer-live-p buffer)
       (with-current-buffer buffer
         (append
          (list
           (soma-gym-bridge--check
            "buffer identity matches precondition"
            (or (not expected-buffer)
                (string= expected-buffer (buffer-name buffer)))
            (format "expected=%S actual=%S"
                    expected-buffer (buffer-name buffer)))
           (soma-gym-bridge--check
            "text owner matches precondition"
            (or (not expected-text-owner)
                (string= expected-text-owner
                         (buffer-name
                          (soma-gym-bridge--buffer-text-owner buffer))))
            (format "expected=%S actual=%S"
                    expected-text-owner
                    (buffer-name
                     (soma-gym-bridge--buffer-text-owner buffer))))
           (soma-gym-bridge--check
            "visited file matches precondition"
            (or (not expected-file)
                (equal expected-file (soma-gym-bridge--json-file-name buffer)))
            (format "expected=%S actual=%S"
                    expected-file
                    (soma-gym-bridge--json-file-name buffer)))
           (soma-gym-bridge--check
            "character revision matches precondition"
            (or (not expected-tick)
                (= expected-tick (buffer-chars-modified-tick)))
            (format "expected=%S actual=%S"
                    expected-tick (buffer-chars-modified-tick)))
           (soma-gym-bridge--check
            "point matches precondition"
            (or (not expected-point)
                (= expected-point (point)))
            (format "expected=%S actual=%S" expected-point (point)))
           (soma-gym-bridge--check
            "selected window matches precondition"
            (or (not expected-selected)
                (and selected-buffer
                     (eq selected-buffer buffer)
                     (string= expected-selected (buffer-name buffer))))
            (format "expected=%S actual=%S"
                    expected-selected
                    (and selected-buffer (buffer-name selected-buffer))))
           (soma-gym-bridge--check
            "modified state matches precondition"
            (or (not (soma-gym-bridge--has-key-p
                      preconditions "modified"))
                (eq (soma-gym-bridge--bool (buffer-modified-p buffer))
                    expected-modified))
            (format "expected=%S actual=%S"
                    expected-modified
                    (soma-gym-bridge--bool (buffer-modified-p buffer))))
           (soma-gym-bridge--check
            "read-only state matches precondition"
            (or (not (soma-gym-bridge--has-key-p
                      preconditions "read_only"))
                (eq (soma-gym-bridge--bool buffer-read-only)
                    expected-read-only))
            (format "expected=%S actual=%S"
                    expected-read-only
                    (soma-gym-bridge--bool buffer-read-only)))
           (soma-gym-bridge--check
            "minibuffer state matches precondition"
            (or (not (soma-gym-bridge--has-key-p
                      preconditions "minibuffer_active"))
                (eq (soma-gym-bridge--bool (active-minibuffer-window))
                    expected-minibuffer-active))
            (format "expected=%S actual=%S"
                    expected-minibuffer-active
                    (soma-gym-bridge--bool (active-minibuffer-window)))))
          (when (soma-gym-bridge--has-key-p preconditions "mark_active")
            (list
             (soma-gym-bridge--check
              "region active state matches precondition"
              (eq (soma-gym-bridge--bool mark-active)
                  expected-mark-active)
              (format "expected=%S actual=%S"
                      expected-mark-active
                      (soma-gym-bridge--bool mark-active)))))
          (when (soma-gym-bridge--has-key-p preconditions "narrowed")
            (list
             (soma-gym-bridge--check
              "narrowing state matches precondition"
              (eq (soma-gym-bridge--bool (buffer-narrowed-p))
                  expected-narrowed)
              (format "expected=%S actual=%S"
                      expected-narrowed
                      (soma-gym-bridge--bool (buffer-narrowed-p))))))
          (when (and start end)
            (list
             (soma-gym-bridge--check
              "target range is valid"
              (soma-gym-bridge--range-valid-p start end)
              (format "range=%S..%S point-min=%S point-max=%S"
                      start end (point-min) (point-max)))))
          (when (and (stringp expected-text) start end
                     (soma-gym-bridge--range-valid-p start end))
            (list
             (soma-gym-bridge--check
              "target range text matches precondition"
              (string= expected-text
                       (buffer-substring-no-properties start end))
              (format "expected=%S actual=%S"
                      expected-text
                      (buffer-substring-no-properties start end)))))))))))

(defun soma-gym-bridge-buffer-read (buffer start end)
  "Return text in BUFFER over one-based half-open range START..END."
  (cond
   ((not (buffer-live-p buffer))
    (soma-gym-bridge--alist
     (cons "operation" "buffer.read")
     (cons "state" "failed")
     (cons "error" "buffer_name does not name a live buffer")))
   ((not (with-current-buffer buffer
           (soma-gym-bridge--range-valid-p start end)))
    (soma-gym-bridge--alist
     (cons "operation" "buffer.read")
     (cons "state" "failed")
     (cons "error" "start/end do not describe a valid buffer range")))
   (t
    (with-current-buffer buffer
      (soma-gym-bridge--alist
       (cons "operation" "buffer.read")
       (cons "state" "succeeded")
       (cons "buffer_name" (buffer-name buffer))
       (cons "text_owner_name"
             (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
       (cons "start" start)
       (cons "end" end)
       (cons "text" (buffer-substring-no-properties start end))
       (cons "chars_tick" (buffer-chars-modified-tick)))))))

(defun soma-gym-bridge-buffer-select (buffer point)
  "Display live BUFFER in the selected window, optionally placing POINT."
  (let* ((window (selected-window))
         (before-selected
          (and (window-live-p window)
               (buffer-name (window-buffer window))))
         before after error-message)
    (cond
     ((not (buffer-live-p buffer))
      (soma-gym-bridge--alist
       (cons "operation" "buffer.select")
       (cons "state" "failed")
       (cons "state_sequence" '("accepted" "failed"))
       (cons "error" "buffer_name does not name a live buffer")))
     ((not (window-live-p window))
      (soma-gym-bridge--alist
       (cons "operation" "buffer.select")
       (cons "state" "failed")
       (cons "state_sequence" '("accepted" "failed"))
       (cons "error" "selected window is not live")))
     ((and point
           (not (with-current-buffer buffer
                  (<= (point-min) point (point-max)))))
      (soma-gym-bridge--alist
       (cons "operation" "buffer.select")
       (cons "state" "failed")
       (cons "state_sequence" '("accepted" "failed"))
       (cons "point" point)
       (cons "error" "point does not describe a valid buffer position")))
     (t
      (setq before (soma-gym-bridge-observe-buffer buffer))
      (condition-case err
          (progn
            (set-window-buffer window buffer)
            (when point
              (with-selected-window window
                (goto-char point)
                (set-window-point window point))))
        (error
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-buffer buffer)))
      (soma-gym-bridge--alist
       (cons "operation" "buffer.select")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "buffer_name" (buffer-name buffer))
       (cons "point" (or point :json-false))
       (cons "selected_window"
             (soma-gym-bridge--alist
              (cons "before_buffer" (or before-selected :json-false))
              (cons "after_buffer"
                    (if (window-live-p window)
                        (buffer-name (window-buffer window))
                      :json-false))
              (cons "window_live"
                    (soma-gym-bridge--bool (window-live-p window)))
              (cons "window_point"
                    (if (window-live-p window)
                        (window-point window)
                      :json-false))))
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "error" (or error-message :json-false)))))))

(defun soma-gym-bridge-goto-char (buffer point preconditions)
  "Move selected-window point to POINT in BUFFER after precondition checks."
  (let* ((selected-buffer (and (window-live-p (selected-window))
                               (window-buffer (selected-window))))
         (checks
          (append
           (soma-gym-bridge--buffer-precondition-checks
            buffer preconditions)
           (list
            (soma-gym-bridge--check
             "selected window displays target buffer"
             (and (buffer-live-p buffer)
                  selected-buffer
                  (eq selected-buffer buffer))
             (format "selected=%S target=%S"
                     (and selected-buffer (buffer-name selected-buffer))
                     (and (buffer-live-p buffer) (buffer-name buffer))))
            (soma-gym-bridge--check
             "target point is valid"
             (and (buffer-live-p buffer)
                  (integerp point)
                  (with-current-buffer buffer
                    (<= (point-min) point (point-max))))
             (format "point=%S" point)))))
         before after error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "emacs.goto_char")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "point" (or point :json-false))
         (cons "precondition_checks" checks)
         (cons "error" "goto-char precondition failed"))
      (setq before (soma-gym-bridge-observe-buffer buffer))
      (condition-case err
          (with-selected-window (selected-window)
            (goto-char point)
            (set-window-point (selected-window) (point)))
        (error
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-buffer buffer)))
      (soma-gym-bridge--alist
       (cons "operation" "emacs.goto_char")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "point" (or point :json-false))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge--org-source-block-overlap (start end)
  "Return source-block evidence when START..END overlaps an Org src block."
  (when (and (integerp start)
             (integerp end)
             (soma-gym-bridge--load-org)
             (derived-mode-p 'org-mode))
    (save-excursion
      (save-restriction
        (widen)
        (catch 'found
          (org-element-map (org-element-parse-buffer) 'src-block
            (lambda (block)
              (let ((block-begin (org-element-property :begin block))
                    (block-end (org-element-property :end block))
                    (language (org-element-property :language block))
                    (parameters (org-element-property :parameters block)))
                (when (and (integerp block-begin)
                           (integerp block-end)
                           (< start block-end)
                           (> end block-begin))
                  (throw
                   'found
                   (soma-gym-bridge--alist
                    (cons "begin" block-begin)
                    (cons "end" block-end)
                    (cons "language" (or language ""))
                    (cons "parameters" (or parameters "")))))))
            nil t)
          nil)))))

(defun soma-gym-bridge--buffer-package-owner-checks (buffer start end)
  "Return package-owner checks for generic buffer edits in BUFFER."
  (when (and (buffer-live-p buffer)
             (integerp start)
             (integerp end)
             (with-current-buffer buffer
               (derived-mode-p 'org-mode)))
    (with-current-buffer buffer
      (let ((overlap (and (soma-gym-bridge--range-valid-p start end)
                          (soma-gym-bridge--org-source-block-overlap
                           start end))))
        (list
         (soma-gym-bridge--check
          "target range does not overlap Org source block in parent buffer"
          (not overlap)
          (or overlap "no overlapping Org source block")))))))

(defun soma-gym-bridge-buffer-replace-range
    (buffer start end text preconditions)
  "Replace BUFFER text over one-based half-open range START..END with TEXT."
  (let ((checks (append
                 (soma-gym-bridge--buffer-precondition-checks
                  buffer preconditions start end)
                 (soma-gym-bridge--buffer-package-owner-checks
                  buffer start end)))
        before after error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "buffer.replace_range")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "precondition_checks" checks)
         (cons "error" "buffer edit precondition failed"))
      (setq before (soma-gym-bridge-observe-buffer buffer))
      (condition-case err
          (with-current-buffer buffer
            (atomic-change-group
              (combine-change-calls start end
                (delete-region start end)
                (goto-char start)
                (insert text))))
        (error
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-buffer buffer)))
      (soma-gym-bridge--alist
       (cons "operation" "buffer.replace_range")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "start" start)
       (cons "end" end)
       (cons "replacement_chars" (length text))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge--list-value (value)
  "Return VALUE as a list when it is JSON-array-like."
  (cond
   ((vectorp value)
    (append value nil))
   ((listp value)
    value)))

(defun soma-gym-bridge--buffer-search-locations (needle)
  "Return one-based half-open locations of literal NEEDLE in current buffer."
  (let ((locations nil)
        (length (length needle)))
    (save-excursion
      (goto-char (point-min))
      (while (search-forward needle nil t)
        (push (cons (- (point) length) (point)) locations)))
    (nreverse locations)))

(defun soma-gym-bridge--patch-plan (buffer patch)
  "Return patch validation checks and replacement plan for BUFFER and PATCH."
  (let* ((patch-object (soma-gym-bridge--object-alist patch))
         (format-name (and (listp patch-object)
                           (soma-gym-bridge--get patch-object "format")))
         (raw-hunks (and (listp patch-object)
                         (soma-gym-bridge--get patch-object "hunks")))
         (hunks (soma-gym-bridge--list-value raw-hunks))
         (patch-object-valid (listp patch-object))
         (format-valid (string= format-name "soma-context-patch-v0"))
         (hunks-valid (and (consp hunks) (<= (length hunks) 50)))
         (checks
          (list
           (soma-gym-bridge--check
            "patch is a typed object"
            patch-object-valid
            (format "%S" patch))
           (soma-gym-bridge--check
            "patch format is soma-context-patch-v0"
            format-valid
            (format "%S" format-name))
           (soma-gym-bridge--check
            "patch contains a bounded non-empty hunk list"
            hunks-valid
            (format "hunk_count=%S" (and hunks (length hunks))))))
         replacements previews)
    (when (and (buffer-live-p buffer) patch-object-valid format-valid
               hunks-valid)
      (with-current-buffer buffer
        (cl-loop
         for raw-hunk in hunks
         for index from 0
         do
         (let* ((hunk (soma-gym-bridge--object-alist raw-hunk))
                (old-text (and (listp hunk)
                               (soma-gym-bridge--get hunk "old_text")))
                (new-text (and (listp hunk)
                               (soma-gym-bridge--get hunk "new_text")))
                (locations
                 (and (stringp old-text)
                      (not (string-empty-p old-text))
                      (soma-gym-bridge--buffer-search-locations old-text)))
                (unique (= (length (or locations nil)) 1))
                (location (car locations)))
           (push
            (soma-gym-bridge--check
             (format "patch hunk %d has old_text and new_text strings"
                     index)
             (and (stringp old-text)
                  (not (string-empty-p old-text))
                  (stringp new-text))
             (format "old_type=%S new_type=%S"
                     (type-of old-text) (type-of new-text)))
            checks)
           (push
            (soma-gym-bridge--check
             (format "patch hunk %d old_text occurs exactly once" index)
             unique
             (format "matches=%S" (length (or locations nil))))
            checks)
           (push
            (soma-gym-bridge--alist
             (cons "index" index)
             (cons "old_sha256"
                   (if (stringp old-text)
                       (secure-hash 'sha256 old-text)
                     :json-false))
             (cons "new_sha256"
                   (if (stringp new-text)
                       (secure-hash 'sha256 new-text)
                     :json-false))
             (cons "match_count" (length (or locations nil)))
             (cons "start" (or (car-safe location) :json-false))
             (cons "end" (or (cdr-safe location) :json-false)))
            previews)
           (when unique
             (push
              (soma-gym-bridge--alist
               (cons "index" index)
               (cons "start" (car location))
               (cons "end" (cdr location))
               (cons "old_text" old-text)
               (cons "new_text" new-text))
              replacements)))))
      (let* ((ranges
              (sort
               (mapcar
                (lambda (replacement)
                  (cons (soma-gym-bridge--get replacement "start")
                        (soma-gym-bridge--get replacement "end")))
                replacements)
               (lambda (left right) (< (car left) (car right)))))
             (non-overlapping t)
             previous-end)
        (dolist (range ranges)
          (when (and previous-end (< (car range) previous-end))
            (setq non-overlapping nil))
          (setq previous-end (cdr range)))
        (push
         (soma-gym-bridge--check
          "patch hunk ranges do not overlap"
          (and (= (length replacements) (length hunks)) non-overlapping)
          (format "%S" ranges))
         checks)))
    (soma-gym-bridge--alist
     (cons "checks" (nreverse checks))
     (cons "previews" (nreverse previews))
     (cons "replacements" (nreverse replacements)))))

(defun soma-gym-bridge-buffer-apply-patch
    (buffer patch preconditions)
  "Apply a bounded one-buffer exact-context PATCH after freshness checks."
  (let* ((buffer-checks
          (soma-gym-bridge--buffer-precondition-checks
           buffer preconditions))
         (patch-plan (soma-gym-bridge--patch-plan buffer patch))
         (patch-checks (soma-gym-bridge--get patch-plan "checks"))
         (checks (append buffer-checks patch-checks))
         (precondition-passed
          (soma-gym-bridge--all-checks-pass-p checks))
         (replacements (soma-gym-bridge--get patch-plan "replacements"))
         before after changed-ranges error-message)
    (if (not precondition-passed)
        (soma-gym-bridge--alist
         (cons "operation" "buffer.apply_patch")
         (cons "route" "checked-buffer-patch")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "precondition_checks" checks)
         (cons "precondition_checks_passed" :json-false)
         (cons "conflict_preview"
               (or (soma-gym-bridge--get patch-plan "previews")
                   :json-false))
         (cons "changed_ranges" nil)
         (cons "error" "buffer patch precondition failed"))
      (setq before (soma-gym-bridge-observe-buffer buffer))
      (condition-case err
          (with-current-buffer buffer
            (atomic-change-group
              (combine-after-change-calls
                (dolist (replacement
                         (sort (copy-sequence replacements)
                               (lambda (left right)
                                 (> (soma-gym-bridge--get left "start")
                                    (soma-gym-bridge--get right "start")))))
                  (let ((start (soma-gym-bridge--get replacement "start"))
                        (end (soma-gym-bridge--get replacement "end"))
                        (new-text
                         (soma-gym-bridge--get replacement "new_text")))
                    (delete-region start end)
                    (goto-char start)
                    (insert new-text)
                    (push
                     (soma-gym-bridge--alist
                      (cons "index"
                            (soma-gym-bridge--get replacement "index"))
                      (cons "start" start)
                      (cons "old_end" end)
                      (cons "new_end" (+ start (length new-text))))
                     changed-ranges))))))
        (error
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-buffer buffer)))
      (soma-gym-bridge--alist
       (cons "operation" "buffer.apply_patch")
       (cons "route" "checked-buffer-patch")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "precondition_checks" checks)
       (cons "precondition_checks_passed" t)
       (cons "conflict_preview"
             (or (soma-gym-bridge--get patch-plan "previews")
                 :json-false))
       (cons "changed_ranges"
             (sort (or changed-ranges nil)
                   (lambda (left right)
                     (< (soma-gym-bridge--get left "index")
                        (soma-gym-bridge--get right "index")))))
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge--completion-provider-name (provider)
  "Return a stable printable name for completion PROVIDER."
  (cond
   ((symbolp provider) (symbol-name provider))
   ((byte-code-function-p provider) "byte-code-function")
   ((functionp provider) "function")
   (t (format "%S" provider))))

(defun soma-gym-bridge--completion-function-name (function)
  "Return a printable name for a completion metadata FUNCTION."
  (cond
   ((null function) :json-false)
   ((symbolp function) (symbol-name function))
   ((byte-code-function-p function) "byte-code-function")
   ((functionp function) "function")
   (t (format "%S" function))))

(defun soma-gym-bridge--completion-candidate-string (candidate)
  "Return CANDIDATE as a bounded JSON string."
  (cond
   ((stringp candidate) candidate)
   ((symbolp candidate) (symbol-name candidate))
   ((and (consp candidate) (stringp (car candidate))) (car candidate))
   ((and (consp candidate) (symbolp (car candidate))) (symbol-name (car candidate)))
   (t (format "%S" candidate))))

(defun soma-gym-bridge--completion-candidates
    (prefix collection predicate max-candidates)
  "Return up to MAX-CANDIDATES strings from COLLECTION matching PREFIX."
  (condition-case nil
      (seq-take
       (delete-dups
        (mapcar #'soma-gym-bridge--completion-candidate-string
                (all-completions prefix collection predicate)))
       max-candidates)
    (error nil)))

(defun soma-gym-bridge--completion-capf-result ()
  "Return the first applicable CAPF result for the current buffer and point."
  (let ((providers completion-at-point-functions)
        result provider provider-index error-message
        (index 0))
    (while (and providers (not result))
      (let ((candidate-provider (car providers)))
        (condition-case err
            (when (functionp candidate-provider)
              (let ((candidate-result (funcall candidate-provider)))
                (when candidate-result
                  (setq result candidate-result
                        provider candidate-provider
                        provider-index index))))
          (error
           (unless error-message
             (setq error-message
                   (format "%s: %s"
                           (soma-gym-bridge--completion-provider-name
                            candidate-provider)
                           (error-message-string err)))))))
      (setq providers (cdr providers)
            index (1+ index)))
    (soma-gym-bridge--alist
     (cons "provider" provider)
     (cons "provider_index" (or provider-index :json-false))
     (cons "result" result)
     (cons "error" (or error-message :json-false)))))

(defun soma-gym-bridge-observe-completion
    (buffer max-candidates prefix-chars suffix-chars)
  "Return completion-at-point provider state for BUFFER without accepting text."
  (if (not (buffer-live-p buffer))
      (soma-gym-bridge--alist
       (cons "operation" "completion.observe")
       (cons "state" "failed")
       (cons "error" "buffer_name does not name a live buffer"))
    (let (before after capf provider provider-index result start end collection
                 props prefix predicate candidates metadata expected-text
                 expected-prefix expected-suffix checks)
      (with-current-buffer buffer
        (setq before (soma-gym-bridge-observe-buffer buffer))
        (setq capf
              (save-mark-and-excursion
                (soma-gym-bridge--completion-capf-result)))
        (setq provider (soma-gym-bridge--get capf "provider"))
        (setq provider-index (soma-gym-bridge--get capf "provider_index"))
        (setq result (soma-gym-bridge--get capf "result"))
        (setq checks
              (list
               (soma-gym-bridge--check
                "target buffer is live"
                t
                (buffer-name buffer))
               (soma-gym-bridge--check
                "completion provider returned a result"
                (consp result)
                (or (soma-gym-bridge--get capf "error")
                    (format "%S" result)))))
        (when (consp result)
          (setq start (nth 0 result)
                end (nth 1 result)
                collection (nth 2 result)
                props (nthcdr 3 result))
          (setq checks
                (append
                 checks
                 (list
                  (soma-gym-bridge--check
                   "completion replacement range is valid"
                   (soma-gym-bridge--range-valid-p start end)
                   (format "range=%S..%S point-min=%S point-max=%S"
                           start end (point-min) (point-max))))))
          (when (soma-gym-bridge--range-valid-p start end)
            (setq prefix (buffer-substring-no-properties start (point)))
            (setq predicate (plist-get props :predicate))
            (setq candidates
                  (soma-gym-bridge--completion-candidates
                   prefix collection predicate max-candidates))
            (setq metadata
                  (ignore-errors
                    (completion-metadata prefix collection predicate)))
            (setq expected-text
                  (buffer-substring-no-properties start end))
            (setq expected-prefix
                  (buffer-substring-no-properties
                   (max (point-min) (- start prefix-chars))
                   start))
            (setq expected-suffix
                  (buffer-substring-no-properties
                   end
                   (min (point-max) (+ end suffix-chars))))))
        (setq after (soma-gym-bridge-observe-buffer buffer)))
      (soma-gym-bridge--alist
       (cons "operation" "completion.observe")
       (cons "state"
             (if (soma-gym-bridge--all-checks-pass-p checks)
                 "succeeded"
               "failed"))
       (cons "buffer_name" (soma-gym-bridge--get before "buffer_name"))
       (cons "text_owner_name" (soma-gym-bridge--get before "text_owner_name"))
       (cons "buffer_file_name"
             (or (soma-gym-bridge--get before "buffer_file_name")
                 :json-false))
       (cons "major_mode" (soma-gym-bridge--get before "major_mode"))
       (cons "chars_tick" (soma-gym-bridge--get before "chars_tick"))
       (cons "modified" (soma-gym-bridge--get before "modified"))
       (cons "read_only" (soma-gym-bridge--get before "read_only"))
       (cons "point" (soma-gym-bridge--get before "point"))
       (cons "mark_active" (soma-gym-bridge--get before "mark_active"))
       (cons "narrowed" (soma-gym-bridge--get before "narrowed"))
       (cons "selected_window_buffer"
             (or (soma-gym-bridge--get before "selected_window_buffer")
                 :json-false))
       (cons "minibuffer_active"
             (soma-gym-bridge--get before "minibuffer_active"))
       (cons "provider_present" (soma-gym-bridge--bool provider))
       (cons "provider"
             (if provider
                 (soma-gym-bridge--completion-provider-name provider)
               :json-false))
       (cons "provider_index" provider-index)
       (cons "replacement_range"
             (if (and (integerp start) (integerp end))
                 (soma-gym-bridge--alist
                  (cons "start" start)
                  (cons "end" end)
                  (cons "expected_text" (or expected-text "")))
               :json-false))
       (cons "candidates" (or candidates nil))
       (cons "candidate_count" (length (or candidates nil)))
       (cons "truncated" :json-false)
       (cons "provider_properties"
             (if (consp result)
                 (soma-gym-bridge--alist
                  (cons "exclusive"
                        (soma-gym-bridge--json-symbol
                         (plist-get props :exclusive)))
                  (cons "exit_function_present"
                        (soma-gym-bridge--bool
                         (plist-get props :exit-function)))
                  (cons "exit_function"
                        (soma-gym-bridge--completion-function-name
                         (plist-get props :exit-function)))
                  (cons "annotation_function_present"
                        (soma-gym-bridge--bool
                         (plist-get props :annotation-function)))
                  (cons "metadata_category"
                        (or (and metadata
                                 (soma-gym-bridge--json-symbol
                                  (completion-metadata-get metadata 'category)))
                            :json-false)))
               :json-false))
       (cons "preview_state"
             (soma-gym-bridge--alist
              (cons "displayed" :json-false)
              (cons "source" "completion-at-point-functions")
              (cons "accepted_candidate" :json-false)
              (cons "provider_exit_function_present"
                    (soma-gym-bridge--bool
                     (and (consp result)
                          (plist-get props :exit-function))))))
       (cons "acceptance_preconditions"
             (if (soma-gym-bridge--all-checks-pass-p checks)
                 (soma-gym-bridge--alist
                  (cons "buffer_name"
                        (soma-gym-bridge--get before "buffer_name"))
                  (cons "text_owner_name"
                        (soma-gym-bridge--get before "text_owner_name"))
                  (cons "buffer_file_name"
                        (or (soma-gym-bridge--get before "buffer_file_name")
                            :json-false))
                  (cons "chars_tick"
                        (soma-gym-bridge--get before "chars_tick"))
                  (cons "point" (soma-gym-bridge--get before "point"))
                  (cons "selected_window_buffer"
                        (or (soma-gym-bridge--get
                             before "selected_window_buffer")
                            :json-false))
                  (cons "modified" (soma-gym-bridge--get before "modified"))
                  (cons "mark_active"
                        (soma-gym-bridge--get before "mark_active"))
                  (cons "narrowed" (soma-gym-bridge--get before "narrowed"))
                  (cons "read_only" (soma-gym-bridge--get before "read_only"))
                  (cons "minibuffer_active"
                        (soma-gym-bridge--get before "minibuffer_active"))
                  (cons "expected_text" (or expected-text ""))
                  (cons "expected_prefix" (or expected-prefix ""))
                  (cons "expected_suffix" (or expected-suffix ""))
                  ;; Legacy aliases used by the initial completion.accept
                  ;; implementation; keep them until the accept API is fully
                  ;; migrated to the manifest field names.
                  (cons "prefix" (or expected-prefix ""))
                  (cons "suffix" (or expected-suffix "")))
               :json-false))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" after)
       (cons "error"
             (if (soma-gym-bridge--all-checks-pass-p checks)
                 :json-false
               "completion provider did not return a valid result"))))))

(defun soma-gym-bridge--completion-context-checks
    (buffer start end preconditions)
  (let ((expected-prefix
         (or (soma-gym-bridge--get preconditions "expected_prefix")
             (soma-gym-bridge--get preconditions "prefix")))
        (expected-suffix
         (or (soma-gym-bridge--get preconditions "expected_suffix")
             (soma-gym-bridge--get preconditions "suffix"))))
    (when (buffer-live-p buffer)
      (with-current-buffer buffer
        (append
         (when (stringp expected-prefix)
           (let* ((prefix-start (- start (length expected-prefix)))
                  (actual-prefix
                   (and (soma-gym-bridge--range-valid-p prefix-start start)
                        (buffer-substring-no-properties prefix-start start))))
             (list
              (soma-gym-bridge--check
               "completion prefix matches precondition"
               (and (stringp actual-prefix)
                    (string= expected-prefix actual-prefix))
               (format "expected=%S actual=%S"
                       expected-prefix
                       (or actual-prefix :json-false))))))
         (when (stringp expected-suffix)
           (let* ((suffix-end (+ end (length expected-suffix)))
                  (actual-suffix
                   (and (soma-gym-bridge--range-valid-p end suffix-end)
                        (buffer-substring-no-properties end suffix-end))))
             (list
              (soma-gym-bridge--check
               "completion suffix matches precondition"
               (and (stringp actual-suffix)
                    (string= expected-suffix actual-suffix))
               (format "expected=%S actual=%S"
                       expected-suffix
                       (or actual-suffix :json-false)))))))))))

(defun soma-gym-bridge-completion-accept
    (buffer start end text preconditions)
  "Accept a completion edit after validating BUFFER context."
  (let ((checks (append
                 (soma-gym-bridge--buffer-precondition-checks
                  buffer preconditions start end)
                 (soma-gym-bridge--completion-context-checks
                  buffer start end preconditions)))
        before after error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "completion.accept")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "precondition_checks" checks)
         (cons "error" "completion acceptance precondition failed"))
      (setq before (soma-gym-bridge-observe-buffer buffer))
      (condition-case err
          (with-current-buffer buffer
            (atomic-change-group
              (combine-change-calls start end
                (delete-region start end)
                (goto-char start)
                (insert text))))
        (error
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-buffer buffer)))
      (soma-gym-bridge--alist
       (cons "operation" "completion.accept")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "start" start)
       (cons "end" end)
       (cons "accepted_chars" (length text))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge-buffer-save (buffer preconditions)
  "Save BUFFER after checking buffer identity and revision preconditions."
  (let ((checks (soma-gym-bridge--buffer-precondition-checks
                 buffer preconditions))
        before after error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "buffer.save")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "precondition_checks" checks)
         (cons "error" "buffer save precondition failed"))
      (setq before (soma-gym-bridge-observe-buffer buffer))
      (condition-case err
          (with-current-buffer buffer
            (save-buffer))
        (error
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-buffer buffer)))
      (soma-gym-bridge--alist
       (cons "operation" "buffer.save")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge--regular-file-sha256 (path)
  (when (and (file-exists-p path)
             (file-regular-p path))
    (with-temp-buffer
      (set-buffer-multibyte nil)
      (insert-file-contents-literally path)
      (secure-hash 'sha256 (buffer-string)))))

(defun soma-gym-bridge--file-signature (path)
  "Return bounded filesystem identity evidence for PATH."
  (if (not (file-exists-p path))
      (soma-gym-bridge--alist
       (cons "exists" :json-false)
       (cons "path" path)
       (cons "truename" :json-false)
       (cons "type" "missing")
       (cons "sha256" :json-false))
    (let ((attrs (file-attributes path 'integer)))
      (soma-gym-bridge--alist
       (cons "exists" t)
       (cons "path" path)
       (cons "truename" (file-truename path))
       (cons "type"
             (cond
              ((file-directory-p path) "directory")
              ((file-regular-p path) "regular")
              (t "special")))
       (cons "size" (nth 7 attrs))
       (cons "mode" (nth 8 attrs))
       (cons "inode" (nth 10 attrs))
       (cons "device" (nth 11 attrs))
       (cons "mtime" (format "%S" (nth 5 attrs)))
       (cons "ctime" (format "%S" (nth 6 attrs)))
       (cons "sha256" (or (soma-gym-bridge--regular-file-sha256 path)
                          :json-false))))))

(defun soma-gym-bridge--call-process-capture (program &rest args)
  "Call PROGRAM with ARGS and return status plus bounded output evidence."
  (if (not (and (stringp program) (file-executable-p program)))
      (soma-gym-bridge--alist
       (cons "status" :json-false)
       (cons "stdout" "")
       (cons "error" "program is not executable"))
    (with-temp-buffer
      (let ((status (apply #'call-process program nil t nil args))
            (text (buffer-substring-no-properties (point-min) (point-max))))
        (soma-gym-bridge--alist
         (cons "status" status)
         (cons "stdout" text)
         (cons "error" :json-false))))))

(defconst soma-gym-bridge--host-projection-prefixes
  '("/nix" "/data02" "/home" "/workspace")
  "Absolute path prefixes that identify host-projected state.")

(defun soma-gym-bridge--path-under-prefix-p (path prefix)
  "Return non-nil when absolute PATH is equal to or under PREFIX."
  (let ((prefix (directory-file-name prefix)))
    (or (string= path prefix)
        (string-prefix-p (file-name-as-directory prefix) path))))

(defun soma-gym-bridge--host-projected-path-p (path)
  "Return non-nil when PATH points at host-projected state."
  (and (stringp path)
       (file-name-absolute-p path)
       (cl-some
        (lambda (prefix)
          (soma-gym-bridge--path-under-prefix-p path prefix))
        soma-gym-bridge--host-projection-prefixes)))

(defun soma-gym-bridge--rootfs-owned-runtime-path-p (path)
  "Return non-nil when PATH is an absolute selected-rootfs runtime path."
  (and (stringp path)
       (file-name-absolute-p path)
       (not (soma-gym-bridge--host-projected-path-p path))
       (let ((truename (ignore-errors (file-truename path))))
         (or (not (stringp truename))
             (not (soma-gym-bridge--host-projected-path-p truename))))))

(defun soma-gym-bridge--rootfs-owned-tool-path-p (path)
  "Return non-nil when PATH looks owned by the selected rootfs."
  (soma-gym-bridge--rootfs-owned-runtime-path-p path))

(defun soma-gym-bridge--tool-identity (name &optional version-arg)
  "Return rootfs-owned executable identity for tool NAME."
  (let* ((path (executable-find name))
         (realpath (and path (file-truename path)))
         (rootfs-owned
          (soma-gym-bridge--rootfs-owned-tool-path-p
           (or realpath path)))
         (version-result
          (and rootfs-owned
               realpath
               (apply #'soma-gym-bridge--call-process-capture
                      realpath (list (or version-arg "--version")))))
         (version-text
          (and version-result
               (soma-gym-bridge--get version-result "stdout")))
         (first-line
          (and version-text
               (car (split-string version-text "\n" t)))))
    (soma-gym-bridge--alist
     (cons "name" name)
     (cons "path" (or path :json-false))
     (cons "realpath" (or realpath :json-false))
     (cons "rootfs_owned" (soma-gym-bridge--bool rootfs-owned))
     (cons "version" (or first-line ""))
     (cons "signature"
           (if realpath
               (soma-gym-bridge--file-signature realpath)
             :json-false)))))

(defun soma-gym-bridge--project-buffer-from-params (params)
  (let ((buffer-name (soma-gym-bridge--get params "buffer_name")))
    (and (stringp buffer-name)
         (get-buffer buffer-name))))

(defun soma-gym-bridge--project-root-from-params (params)
  "Resolve project root from PARAMS without invoking arbitrary shell state."
  (let* ((explicit-root (soma-gym-bridge--get params "root"))
         (buffer (soma-gym-bridge--project-buffer-from-params params))
         (buffer-directory
          (and (buffer-live-p buffer)
               (with-current-buffer buffer
                 (or (and buffer-file-name
                          (file-name-directory buffer-file-name))
                     default-directory))))
         (candidate
          (cond
           ((stringp explicit-root) explicit-root)
           (buffer-directory
            (or (locate-dominating-file buffer-directory ".git")
                buffer-directory))
           (t default-directory))))
    (when (and (stringp candidate)
               (file-directory-p candidate))
      (file-name-as-directory (file-truename candidate)))))

(defun soma-gym-bridge--project-git-string (git root &rest args)
  (let* ((result (apply #'soma-gym-bridge--call-process-capture
                        git "-C" root args))
         (status (soma-gym-bridge--get result "status")))
    (and (integerp status)
         (= status 0)
         (string-trim (soma-gym-bridge--get result "stdout")))))

(defun soma-gym-bridge--with-project-buffer-context (buffer thunk)
  "Call THUNK in BUFFER's local process context when BUFFER is live."
  (if (buffer-live-p buffer)
      (with-current-buffer buffer
        (funcall thunk))
    (funcall thunk)))

(defun soma-gym-bridge--project-vcs-identity (root)
  "Return bounded VCS identity for ROOT."
  (let* ((git-tool (soma-gym-bridge--tool-identity "git"))
         (git (soma-gym-bridge--get git-tool "realpath"))
         (git-rootfs-owned
          (eq (soma-gym-bridge--get git-tool "rootfs_owned") t))
         (git-root
          (and git-rootfs-owned
               (stringp git)
               (soma-gym-bridge--project-git-string
                git root "rev-parse" "--show-toplevel")))
         (git-dir
          (and git-rootfs-owned
               (stringp git)
               (soma-gym-bridge--project-git-string
                git root "rev-parse" "--git-dir")))
         (branch
          (and git-rootfs-owned
               (stringp git)
               (or (soma-gym-bridge--project-git-string
                    git root "branch" "--show-current")
                   "")))
         (head
          (and git-rootfs-owned
               (stringp git)
               (or (soma-gym-bridge--project-git-string
                    git root "rev-parse" "--verify" "HEAD")
                   ""))))
    (soma-gym-bridge--alist
     (cons "type" (if git-root "git" :json-false))
     (cons "root"
           (if git-root
               (file-name-as-directory (file-truename git-root))
             :json-false))
     (cons "git_dir" (or git-dir :json-false))
     (cons "branch" (or branch ""))
     (cons "head" (or head ""))
     (cons "tool" git-tool))))

(defun soma-gym-bridge-observe-project (params)
  "Observe project identity from PARAMS."
  (let* ((root (soma-gym-bridge--project-root-from-params params))
         (buffer (soma-gym-bridge--project-buffer-from-params params)))
    (if (not root)
        (soma-gym-bridge--alist
         (cons "operation" "project.observe")
         (cons "state" "failed")
         (cons "error" "project root is not a directory"))
      (soma-gym-bridge--with-project-buffer-context
       buffer
       (lambda ()
         (let ((search-tool (soma-gym-bridge--tool-identity "rg"))
               (vcs (soma-gym-bridge--project-vcs-identity root)))
           (soma-gym-bridge--alist
            (cons "operation" "project.observe")
            (cons "state" "succeeded")
            (cons "project_root" root)
            (cons "buffer_name"
                  (if (buffer-live-p buffer) (buffer-name buffer) :json-false))
            (cons "buffer_file_name"
                  (if (buffer-live-p buffer)
                      (or buffer-file-name :json-false)
                    :json-false))
            (cons "default_directory"
                  (if (buffer-live-p buffer)
                      default-directory
                    :json-false))
            (cons "vcs" (or vcs :json-false))
            (cons "search_tool" search-tool)
            (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
            (cons "plan" (or (getenv "SOMA_INSULA_PLAN") "")))))))))

(defun soma-gym-bridge--rg-match-result (event root max-text)
  (let* ((data (soma-gym-bridge--value event "data"))
         (path-data (soma-gym-bridge--value data "path"))
         (lines-data (soma-gym-bridge--value data "lines"))
         (path-text (soma-gym-bridge--value path-data "text"))
         (line-text (or (soma-gym-bridge--value lines-data "text") ""))
         (absolute-path
          (and (stringp path-text)
               (file-truename
                (if (file-name-absolute-p path-text)
                    path-text
                  (expand-file-name path-text root))))))
    (soma-gym-bridge--alist
     (cons "path" (or absolute-path :json-false))
     (cons "line" (or (soma-gym-bridge--value data "line_number")
                      :json-false))
     (cons "absolute_offset"
           (or (soma-gym-bridge--value data "absolute_offset")
               :json-false))
     (cons "text" (substring line-text 0 (min max-text (length line-text))))
     (cons "text_truncated"
           (soma-gym-bridge--bool (> (length line-text) max-text)))
     (cons "submatches"
           (or (soma-gym-bridge--value data "submatches")
               nil)))))

(defun soma-gym-bridge--parse-rg-json-lines (text root max-results max-text)
  (let ((lines (split-string text "\n" t))
        results
        (match-count 0)
        parse-errors)
    (dolist (line lines)
      (condition-case err
          (let* ((event (let ((json-object-type 'alist)
                              (json-array-type 'list)
                              (json-key-type 'string)
                              (json-false :json-false))
                          (json-read-from-string line)))
                 (type (soma-gym-bridge--value event "type")))
            (when (string= type "match")
              (setq match-count (1+ match-count))
              (when (<= (length results) (1- max-results))
                (push (soma-gym-bridge--rg-match-result
                       event root max-text)
                      results))))
        (error
         (push (error-message-string err) parse-errors))))
    (soma-gym-bridge--alist
     (cons "match_count" match-count)
     (cons "results" (nreverse results))
     (cons "results_truncated"
           (soma-gym-bridge--bool (> match-count max-results)))
     (cons "parse_errors" (or (nreverse parse-errors) nil)))))

(defun soma-gym-bridge-project-search
    (params max-results max-text)
  "Search a project with rootfs-owned ripgrep using typed PARAMS."
  (let* ((root (soma-gym-bridge--project-root-from-params params))
         (buffer (soma-gym-bridge--project-buffer-from-params params))
         (query (soma-gym-bridge--get params "query"))
         search-tool rg search-result parsed status error-message)
    (soma-gym-bridge--with-project-buffer-context
     buffer
     (lambda ()
       (setq search-tool (soma-gym-bridge--tool-identity "rg"))
       (setq rg (soma-gym-bridge--get search-tool "realpath"))
       (cond
        ((not root)
         (setq error-message "project root is not a directory"))
        ((not (and (stringp query) (not (string-empty-p query))))
         (setq error-message "query must be a non-empty string"))
        ((not (and (stringp rg) (file-executable-p rg)))
         (setq error-message "rootfs-owned ripgrep is not available"))
        ((not (soma-gym-bridge--rootfs-owned-tool-path-p rg))
         (setq error-message "rootfs-owned ripgrep is not available")))
       (unless error-message
         (setq search-result
               (soma-gym-bridge--call-process-capture
                rg "--json" "--line-number" "--column" "--fixed-strings"
                "--hidden" "--glob" "!.git/*" "--" query root))
         (setq status (soma-gym-bridge--get search-result "status"))
         (if (not (member status '(0 1)))
             (setq error-message
                   (format "ripgrep failed with status %S" status))
           (setq parsed
                 (soma-gym-bridge--parse-rg-json-lines
                  (soma-gym-bridge--get search-result "stdout")
                  root max-results max-text))))
       (if error-message
           (soma-gym-bridge--alist
            (cons "operation" "project.search")
            (cons "state" "failed")
            (cons "project_root" (or root :json-false))
            (cons "search_tool" search-tool)
            (cons "error" error-message))
         (soma-gym-bridge--alist
          (cons "operation" "project.search")
          (cons "state" "succeeded")
          (cons "project_root" root)
          (cons "query" query)
          (cons "search_tool" search-tool)
          (cons "exit_status" status)
          (cons "result_count" (soma-gym-bridge--get parsed "match_count"))
          (cons "results_returned"
                (length (soma-gym-bridge--get parsed "results")))
          (cons "results_truncated"
                (soma-gym-bridge--get parsed "results_truncated"))
          (cons "results" (soma-gym-bridge--get parsed "results"))
          (cons "parse_errors" (soma-gym-bridge--get parsed "parse_errors"))
          (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
          (cons "plan" (or (getenv "SOMA_INSULA_PLAN") ""))))))))

(defun soma-gym-bridge--load-org ()
  "Load the Org libraries needed by the reports profile."
  (and (require 'org nil t)
       (require 'org-element nil t)
       (require 'org-src nil t)
       t))

(defun soma-gym-bridge--load-org-babel-core ()
  "Load the Org Babel libraries needed by report execution."
  (and (soma-gym-bridge--load-org)
       (require 'ob-core nil t)
       t))

(defun soma-gym-bridge--org-entry-properties (begin)
  (condition-case nil
      (save-excursion
        (goto-char begin)
        (org-entry-properties nil 'standard))
    (error nil)))

(defun soma-gym-bridge--org-heading-observation (headline index)
  "Return a JSON-ready observation for Org HEADLINE at INDEX."
  (let* ((begin (org-element-property :begin headline))
         (props (soma-gym-bridge--org-entry-properties begin))
         (id (cdr (assoc "ID" props))))
    (soma-gym-bridge--alist
     (cons "index" index)
     (cons "begin" begin)
     (cons "end" (org-element-property :end headline))
     (cons "level" (org-element-property :level headline))
     (cons "todo" (or (org-element-property :todo-keyword headline)
                      :json-false))
     (cons "title" (or (org-element-property :raw-value headline) ""))
     (cons "tags" (or (org-element-property :tags headline) nil))
     (cons "id" (or id :json-false))
     (cons "properties" props))))

(defun soma-gym-bridge--org-headings (ast max-headings)
  (let ((count 0)
        headings)
    (org-element-map ast 'headline
      (lambda (headline)
        (setq count (1+ count))
        (when (<= count max-headings)
          (push (soma-gym-bridge--org-heading-observation headline count)
                headings))))
    (soma-gym-bridge--alist
     (cons "heading_count" count)
     (cons "headings" (nreverse headings))
     (cons "headings_truncated" (soma-gym-bridge--bool
                                 (> count max-headings))))))

(defun soma-gym-bridge--org-source-block-observation (block index max-text)
  "Return a JSON-ready observation for Org source BLOCK at INDEX."
  (let* ((value (or (org-element-property :value block) ""))
         (excerpt (substring value 0 (min max-text (length value)))))
    (soma-gym-bridge--alist
     (cons "index" index)
     (cons "begin" (org-element-property :begin block))
     (cons "end" (org-element-property :end block))
     (cons "language" (or (org-element-property :language block) ""))
     (cons "parameters" (or (org-element-property :parameters block) ""))
     (cons "value_sha256" (secure-hash 'sha256 value))
     (cons "value_excerpt" excerpt)
     (cons "value_truncated" (soma-gym-bridge--bool
                              (> (length value) max-text))))))

(defun soma-gym-bridge--org-source-blocks (ast max-source-blocks max-text)
  (let ((count 0)
        blocks)
    (org-element-map ast 'src-block
      (lambda (block)
        (setq count (1+ count))
        (when (<= count max-source-blocks)
          (push
           (soma-gym-bridge--org-source-block-observation
            block count max-text)
           blocks))))
    (soma-gym-bridge--alist
     (cons "source_block_count" count)
     (cons "source_blocks" (nreverse blocks))
     (cons "source_blocks_truncated" (soma-gym-bridge--bool
                                      (> count max-source-blocks))))))

(defun soma-gym-bridge-observe-org-tree
    (buffer max-headings max-source-blocks max-text)
  "Return bounded Org structure evidence for BUFFER."
  (cond
   ((not (buffer-live-p buffer))
    (soma-gym-bridge--alist
     (cons "operation" "org.observe_tree")
     (cons "state" "failed")
     (cons "error" "buffer_name does not name a live buffer")))
   ((not (soma-gym-bridge--load-org))
    (soma-gym-bridge--alist
     (cons "operation" "org.observe_tree")
     (cons "state" "failed")
     (cons "error" "Org is not available in this Emacs")))
   (t
    (with-current-buffer buffer
      (if (not (derived-mode-p 'org-mode))
          (soma-gym-bridge--alist
           (cons "operation" "org.observe_tree")
           (cons "state" "failed")
           (cons "buffer_name" (buffer-name buffer))
           (cons "major_mode" (symbol-name major-mode))
           (cons "error" "buffer is not in org-mode"))
        (save-restriction
          (widen)
          (let* ((text (buffer-substring-no-properties
                        (point-min) (point-max)))
                 (ast (org-element-parse-buffer))
                 (heading-data
                  (soma-gym-bridge--org-headings ast max-headings))
                 (source-data
                  (soma-gym-bridge--org-source-blocks
                   ast max-source-blocks max-text)))
            (append
             (soma-gym-bridge--alist
              (cons "operation" "org.observe_tree")
              (cons "state" "succeeded")
              (cons "buffer_name" (buffer-name buffer))
              (cons "text_owner_name"
                    (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
              (cons "buffer_file_name" (or buffer-file-name :json-false))
              (cons "major_mode" (symbol-name major-mode))
              (cons "chars_tick" (buffer-chars-modified-tick))
              (cons "modified" (soma-gym-bridge--bool (buffer-modified-p)))
              (cons "read_only" (soma-gym-bridge--bool buffer-read-only))
              (cons "point" (point))
              (cons "mark_active" (soma-gym-bridge--bool mark-active))
              (cons "narrowed" (soma-gym-bridge--bool (buffer-narrowed-p)))
              (cons "selected_window_buffer"
                    (if (window-live-p (selected-window))
                        (buffer-name (window-buffer (selected-window)))
                      :json-false))
              (cons "minibuffer_active"
                    (soma-gym-bridge--bool (active-minibuffer-window)))
              (cons "structure_fingerprint" (secure-hash 'sha256 text)))
             heading-data
             source-data))))))))

(defun soma-gym-bridge--org-structure-fingerprint ()
  "Return the widened current Org buffer's text fingerprint."
  (save-restriction
    (widen)
    (secure-hash
     'sha256
     (buffer-substring-no-properties (point-min) (point-max)))))

(defun soma-gym-bridge--org-heading-present-id (heading)
  "Return HEADING's ID when it is a present JSON string."
  (let ((id (and (listp heading)
                 (soma-gym-bridge--get heading "id"))))
    (and (stringp id)
         (not (string-empty-p id))
         id)))

(defun soma-gym-bridge--org-heading-id-count (id)
  "Return the number of Org headings in the current buffer with ID."
  (let ((count 0))
    (when (and (stringp id)
               (not (string-empty-p id)))
      (save-excursion
        (save-restriction
          (widen)
          (org-element-map (org-element-parse-buffer) 'headline
            (lambda (headline)
              (let* ((begin (org-element-property :begin headline))
                     (props (soma-gym-bridge--org-entry-properties begin)))
                (when (equal id (cdr (assoc "ID" props)))
                  (setq count (1+ count)))))))))
    count))

(defun soma-gym-bridge--org-resolve-heading-position (heading)
  "Resolve observed Org HEADING to a live heading position."
  (let ((id (soma-gym-bridge--org-heading-present-id heading))
        (begin (and (listp heading)
                    (soma-gym-bridge--get heading "begin"))))
    (save-restriction
      (widen)
      (cond
       (id
        (org-find-property "ID" id))
       ((and (integerp begin)
             (<= (point-min) begin)
             (<= begin (point-max)))
        begin)))))

(defun soma-gym-bridge--org-current-heading-observation (position index)
  "Return Org heading evidence at POSITION, or nil when it is invalid."
  (condition-case nil
      (save-excursion
        (goto-char position)
        (org-back-to-heading t)
        (let ((headline (org-element-at-point)))
          (and (eq (org-element-type headline) 'headline)
               (soma-gym-bridge--org-heading-observation headline index))))
    (error nil)))

(defun soma-gym-bridge--org-heading-fields-match-p (left right)
  "Return non-nil when LEFT and RIGHT describe the same observed heading."
  (and (listp left)
       (listp right)
       (equal (soma-gym-bridge--get left "id")
              (soma-gym-bridge--get right "id"))
       (equal (soma-gym-bridge--get left "begin")
              (soma-gym-bridge--get right "begin"))
       (equal (soma-gym-bridge--get left "level")
              (soma-gym-bridge--get right "level"))
       (equal (soma-gym-bridge--get left "todo")
              (soma-gym-bridge--get right "todo"))
       (equal (soma-gym-bridge--get left "title")
              (soma-gym-bridge--get right "title"))))

(defun soma-gym-bridge--org-find-source-block (source-block max-text)
  "Return live source-block evidence matching observed SOURCE-BLOCK."
  (let ((begin (and (listp source-block)
                    (soma-gym-bridge--get source-block "begin")))
        (index 0)
        found)
    (when (integerp begin)
      (catch 'done
        (org-element-map (org-element-parse-buffer) 'src-block
          (lambda (block)
            (setq index (1+ index))
            (when (= begin (org-element-property :begin block))
              (setq found
                    (soma-gym-bridge--org-source-block-observation
                     block index max-text))
              (throw 'done found))))))
    found))

(defun soma-gym-bridge--org-source-block-fields-match-p (left right)
  "Return non-nil when LEFT and RIGHT describe the same source block."
  (and (listp left)
       (listp right)
       (equal (soma-gym-bridge--get left "begin")
              (soma-gym-bridge--get right "begin"))
       (equal (soma-gym-bridge--get left "end")
              (soma-gym-bridge--get right "end"))
       (equal (soma-gym-bridge--get left "language")
              (soma-gym-bridge--get right "language"))
       (equal (soma-gym-bridge--get left "parameters")
              (soma-gym-bridge--get right "parameters"))
       (equal (soma-gym-bridge--get left "value_sha256")
              (soma-gym-bridge--get right "value_sha256"))))

(defun soma-gym-bridge--string-list (value)
  "Return VALUE as a list of strings, or nil when VALUE is not one."
  (cond
   ((vectorp value)
    (soma-gym-bridge--string-list (append value nil)))
   ((and (listp value)
         (cl-every #'stringp value))
    value)))

(defun soma-gym-bridge--same-directory-p (left right)
  "Return non-nil when LEFT and RIGHT name the same directory."
  (and (stringp left)
       (stringp right)
       (let ((left-dir (file-name-as-directory (expand-file-name left)))
             (right-dir (file-name-as-directory (expand-file-name right))))
         (let ((left-true (ignore-errors (file-truename left-dir)))
               (right-true (ignore-errors (file-truename right-dir))))
           (and left-true right-true (equal left-true right-true))))))

(defun soma-gym-bridge--org-babel-normalize-session (session)
  "Return a JSON-ready normalized Org Babel SESSION value."
  (cond
   ((or (null session)
        (eq session :json-false)
        (and (stringp session)
             (or (string-empty-p session)
                 (string= session "none"))))
    :json-false)
   ((stringp session) session)
   (t (format "%S" session))))

(defun soma-gym-bridge--org-babel-language-feature (language)
  "Return the explicitly supported Org Babel feature for LANGUAGE."
  (cdr (assoc language
              '(("emacs-lisp" . ob-emacs-lisp)
                ("elisp" . ob-emacs-lisp)))))

(defun soma-gym-bridge--org-babel-load-language (language)
  "Load the supported Org Babel backend for LANGUAGE."
  (let ((feature (soma-gym-bridge--org-babel-language-feature language)))
    (and feature
         (require feature nil t)
         t)))

(defun soma-gym-bridge--org-babel-info-observation (info)
  "Return compact JSON-ready evidence from Org Babel INFO."
  (let* ((language (and (listp info) (nth 0 info)))
         (body (and (listp info) (nth 1 info)))
         (params (and (listp info) (nth 2 info)))
         (session (cdr (assq :session params)))
         (result-params (cdr (assq :result-params params)))
         (dir (cdr (assq :dir params))))
    (soma-gym-bridge--alist
     (cons "language" (or language :json-false))
     (cons "body_sha256"
           (if (stringp body) (secure-hash 'sha256 body) :json-false))
     (cons "session" (soma-gym-bridge--org-babel-normalize-session session))
     (cons "dir_header" (or dir :json-false))
     (cons "result_params" (or result-params nil)))))

(defun soma-gym-bridge--org-babel-result-observation ()
  "Return result-block evidence for the current Org Babel source block."
  (condition-case err
      (save-excursion
        (let ((begin (org-babel-where-is-src-block-result nil)))
          (if (not (integerp begin))
              (soma-gym-bridge--alist
               (cons "exists" :json-false)
               (cons "error" :json-false))
            (goto-char begin)
            (let* ((end (condition-case nil
                            (org-babel-result-end)
                          (error (line-end-position))))
                   (text (buffer-substring-no-properties begin end)))
              (soma-gym-bridge--alist
               (cons "exists" t)
               (cons "begin" begin)
               (cons "end" end)
               (cons "text_sha256" (secure-hash 'sha256 text))
               (cons "text_excerpt"
                     (substring text 0 (min 1000 (length text))))
               (cons "text_truncated"
                     (soma-gym-bridge--bool (> (length text) 1000))))))))
    (error
     (soma-gym-bridge--alist
      (cons "exists" :json-false)
      (cons "error" (error-message-string err))))))

(defun soma-gym-bridge--org-source-edit-parent-markers (edit-buffer)
  "Return parent marker evidence for Org source EDIT-BUFFER."
  (when (buffer-live-p edit-buffer)
    (with-current-buffer edit-buffer
      (let* ((beg (and (boundp 'org-src--beg-marker)
                       org-src--beg-marker))
             (end (and (boundp 'org-src--end-marker)
                       org-src--end-marker))
             (parent (and (markerp beg) (marker-buffer beg)))
             (beg-pos (soma-gym-bridge--json-marker-position beg))
             (end-pos (soma-gym-bridge--json-marker-position end))
             (text
              (if (and (buffer-live-p parent)
                       (markerp end)
                       (eq (marker-buffer end) parent)
                       (integerp beg-pos)
                       (integerp end-pos)
                       (<= beg-pos end-pos))
                  (with-current-buffer parent
                    (buffer-substring-no-properties beg-pos end-pos))
                nil)))
        (soma-gym-bridge--alist
         (cons "buffer_name"
               (if (buffer-live-p parent) (buffer-name parent) :json-false))
         (cons "text_owner_name"
               (if (buffer-live-p parent)
                   (buffer-name
                    (soma-gym-bridge--buffer-text-owner parent))
                 :json-false))
         (cons "buffer_file_name"
               (if (buffer-live-p parent)
                   (soma-gym-bridge--json-file-name parent)
                 :json-false))
         (cons "begin" beg-pos)
         (cons "end" end-pos)
         (cons "text_sha256"
               (if (stringp text) (secure-hash 'sha256 text) :json-false)))))))

(defun soma-gym-bridge--org-parent-markers-match-p (left right)
  "Return non-nil when LEFT and RIGHT describe the same parent marker range."
  (and (listp left)
       (listp right)
       (equal (soma-gym-bridge--get left "buffer_name")
              (soma-gym-bridge--get right "buffer_name"))
       (equal (soma-gym-bridge--get left "text_owner_name")
              (soma-gym-bridge--get right "text_owner_name"))
       (equal (soma-gym-bridge--get left "buffer_file_name")
              (soma-gym-bridge--get right "buffer_file_name"))
       (equal (soma-gym-bridge--get left "begin")
              (soma-gym-bridge--get right "begin"))
       (equal (soma-gym-bridge--get left "end")
              (soma-gym-bridge--get right "end"))
       (equal (soma-gym-bridge--get left "text_sha256")
              (soma-gym-bridge--get right "text_sha256"))))

(defun soma-gym-bridge--org-source-block-checks
    (buffer source-block preconditions)
  "Return precondition checks for opening SOURCE-BLOCK in Org BUFFER."
  (let* ((expected-node (soma-gym-bridge--get preconditions "org_node"))
         (expected-id (soma-gym-bridge--org-heading-present-id expected-node))
         (expected-source-block
          (soma-gym-bridge--get preconditions "source_block"))
         (expected-fingerprint
          (soma-gym-bridge--get preconditions "structure_fingerprint"))
         (org-available (soma-gym-bridge--load-org))
         mode-checks structure-checks id-count)
    (when (buffer-live-p buffer)
      (with-current-buffer buffer
        (setq mode-checks
              (list
               (soma-gym-bridge--check
                "Org source library is available"
                org-available
                (if org-available "loaded" "unavailable"))
               (soma-gym-bridge--check
                "buffer is in org-mode"
                (derived-mode-p 'org-mode)
                (symbol-name major-mode))))
        (when (and org-available (derived-mode-p 'org-mode))
          (save-restriction
            (widen)
            (let* ((current-fingerprint
                    (soma-gym-bridge--org-structure-fingerprint))
                   (heading-position
                    (and (listp expected-node)
                         (soma-gym-bridge--org-resolve-heading-position
                          expected-node)))
                   (current-heading
                    (and heading-position
                         (soma-gym-bridge--org-current-heading-observation
                          heading-position
                          (or (and (listp expected-node)
                                   (soma-gym-bridge--get
                                    expected-node "index"))
                              1))))
                   (current-source-block
                    (and (listp source-block)
                         (soma-gym-bridge--org-find-source-block
                          source-block 4000))))
              (setq id-count
                    (and expected-id
                         (soma-gym-bridge--org-heading-id-count expected-id)))
              (setq structure-checks
                    (list
                     (soma-gym-bridge--check
                      "structure fingerprint matches precondition"
                      (and (stringp expected-fingerprint)
                           (string= expected-fingerprint
                                    current-fingerprint))
                      (format "expected=%S actual=%S"
                              expected-fingerprint current-fingerprint))
                     (soma-gym-bridge--check
                      "org_node precondition is an object"
                      (listp expected-node)
                      (format "%S" expected-node))
                     (soma-gym-bridge--check
                      "Org heading ID is unique in buffer"
                      (or (not expected-id)
                          (and (integerp id-count)
                               (= id-count 1)))
                      (format "id=%S count=%S"
                              expected-id
                              (or id-count :json-false)))
                     (soma-gym-bridge--check
                      "live Org heading matches org_node precondition"
                      (soma-gym-bridge--org-heading-fields-match-p
                       expected-node current-heading)
                      (format "expected=%S actual=%S"
                              expected-node current-heading))
                     (soma-gym-bridge--check
                      "source_block parameter is an object"
                      (listp source-block)
                      (format "%S" source-block))
                     (soma-gym-bridge--check
                      "source_block precondition is an object"
                      (listp expected-source-block)
                      (format "%S" expected-source-block))
                     (soma-gym-bridge--check
                      "source_block parameter matches precondition"
                      (soma-gym-bridge--org-source-block-fields-match-p
                       source-block expected-source-block)
                      (format "param=%S precondition=%S"
                              source-block expected-source-block))
                     (soma-gym-bridge--check
                      "source block resolves in parent Org buffer"
                      (listp current-source-block)
                      (format "%S" current-source-block))
                     (soma-gym-bridge--check
                      "live source block matches observed identity"
                      (soma-gym-bridge--org-source-block-fields-match-p
                       source-block current-source-block)
                      (format "expected=%S actual=%S"
                              source-block current-source-block)))))))))
    (append
     (soma-gym-bridge--buffer-precondition-checks
      buffer
      (soma-gym-bridge--org-parent-buffer-preconditions preconditions))
     mode-checks
     structure-checks)))

(defun soma-gym-bridge--org-update-heading-checks
    (buffer heading changes preconditions)
  "Return precondition checks for `org.update_heading'."
  (let* ((expected-node (soma-gym-bridge--get preconditions "org_node"))
         (expected-id (soma-gym-bridge--org-heading-present-id expected-node))
         (expected-fingerprint
          (soma-gym-bridge--get preconditions "structure_fingerprint"))
         (changes-object-p (listp changes))
         (has-title (and changes-object-p
                         (soma-gym-bridge--has-key-p changes "title")))
         (title (and has-title
                     (soma-gym-bridge--get changes "title")))
         (has-todo (and changes-object-p
                        (soma-gym-bridge--has-key-p changes "todo")))
         (todo (and has-todo
                    (soma-gym-bridge--get changes "todo")))
         (org-available (soma-gym-bridge--load-org))
         mode-checks structure-checks change-checks id-count)
    (setq change-checks
          (list
           (soma-gym-bridge--check
            "changes is an object"
            changes-object-p
            (format "%S" changes))
           (soma-gym-bridge--check
            "changes includes a supported heading field"
            (and changes-object-p (or has-title has-todo))
            (format "keys=%S"
                    (and changes-object-p
                         (soma-gym-bridge--keys changes))))
           (soma-gym-bridge--check
            "title change is a single-line string"
            (or (not has-title)
                (and (stringp title)
                     (not (string-match-p "[\n\r]" title))))
            (format "%S" title))
           (soma-gym-bridge--check
            "todo change is a valid Org TODO keyword or empty"
            (or (not has-todo)
                (eq todo :json-false)
                (null todo)
                (and (stringp todo)
                     (or (string-empty-p todo)
                         (null org-todo-keywords-1)
                         (member todo org-todo-keywords-1))))
            (format "todo=%S allowed=%S"
                    todo
                    org-todo-keywords-1))))
    (when (buffer-live-p buffer)
      (with-current-buffer buffer
        (setq mode-checks
              (list
               (soma-gym-bridge--check
                "Org library is available"
                org-available
                (if org-available "loaded" "unavailable"))
               (soma-gym-bridge--check
                "buffer is in org-mode"
                (derived-mode-p 'org-mode)
                (symbol-name major-mode))))
        (when (and org-available (derived-mode-p 'org-mode))
          (save-restriction
            (widen)
            (let* ((current-fingerprint
                    (soma-gym-bridge--org-structure-fingerprint))
                   (position
                    (and (listp heading)
                         (soma-gym-bridge--org-resolve-heading-position
                          heading)))
                   (current-heading
                    (and position
                         (soma-gym-bridge--org-current-heading-observation
                          position
                          (or (and (listp heading)
                                   (soma-gym-bridge--get heading "index"))
                              1)))))
              (setq id-count
                    (and expected-id
                         (soma-gym-bridge--org-heading-id-count expected-id)))
              (setq structure-checks
                    (list
                     (soma-gym-bridge--check
                      "structure fingerprint matches precondition"
                      (and (stringp expected-fingerprint)
                           (string= expected-fingerprint
                                    current-fingerprint))
                      (format "expected=%S actual=%S"
                              expected-fingerprint current-fingerprint))
                     (soma-gym-bridge--check
                      "heading parameter is an object"
                      (listp heading)
                      (format "%S" heading))
                     (soma-gym-bridge--check
                      "org_node precondition is an object"
                      (listp expected-node)
                      (format "%S" expected-node))
                     (soma-gym-bridge--check
                      "Org heading ID is unique in buffer"
                      (or (not expected-id)
                          (and (integerp id-count)
                               (= id-count 1)))
                      (format "id=%S count=%S"
                              expected-id
                              (or id-count :json-false)))
                     (soma-gym-bridge--check
                      "heading parameter matches org_node precondition"
                      (soma-gym-bridge--org-heading-fields-match-p
                       heading expected-node)
                      (format "heading=%S org_node=%S"
                              heading expected-node))
                     (soma-gym-bridge--check
                      "Org node resolves to a live heading"
                      (listp current-heading)
                      (format "position=%S heading=%S"
                              (or position :json-false)
                              (or current-heading :json-false)))
                     (soma-gym-bridge--check
                      "live Org heading matches observed ID"
                      (or (not (soma-gym-bridge--org-heading-present-id
                                heading))
                          (and current-heading
                               (equal
                                (soma-gym-bridge--get heading "id")
                                (soma-gym-bridge--get current-heading "id"))))
                      (format "expected=%S actual=%S"
                              (and (listp heading)
                                   (soma-gym-bridge--get heading "id"))
                              (and current-heading
                                   (soma-gym-bridge--get current-heading
                                                         "id"))))
                     (soma-gym-bridge--check
                      "live Org heading begin matches observed begin"
                      (or (not (integerp
                                (and (listp heading)
                                     (soma-gym-bridge--get heading
                                                           "begin"))))
                          (and current-heading
                               (= (soma-gym-bridge--get heading "begin")
                                  (soma-gym-bridge--get current-heading
                                                        "begin"))))
                      (format "expected=%S actual=%S"
                              (and (listp heading)
                                   (soma-gym-bridge--get heading "begin"))
                              (and current-heading
                                   (soma-gym-bridge--get current-heading
                                                         "begin"))))
                     (soma-gym-bridge--check
                      "live Org heading TODO matches observed TODO"
                      (or (not (listp heading))
                          (not current-heading)
                          (equal (soma-gym-bridge--get heading "todo")
                                 (soma-gym-bridge--get current-heading
                                                       "todo")))
                      (format "expected=%S actual=%S"
                              (and (listp heading)
                                   (soma-gym-bridge--get heading "todo"))
                              (and current-heading
                                   (soma-gym-bridge--get current-heading
                                                         "todo"))))
                     (soma-gym-bridge--check
                      "live Org heading title matches observed title"
                      (or (not (listp heading))
                          (not current-heading)
                          (equal (soma-gym-bridge--get heading "title")
                                 (soma-gym-bridge--get current-heading
                                                       "title")))
                      (format "expected=%S actual=%S"
                              (and (listp heading)
                                   (soma-gym-bridge--get heading "title"))
                              (and current-heading
                                   (soma-gym-bridge--get current-heading
                                                         "title")))))))))))
    (append
     (soma-gym-bridge--buffer-precondition-checks
      buffer
      (soma-gym-bridge--org-parent-buffer-preconditions preconditions))
     mode-checks
     structure-checks
     change-checks)))

(defun soma-gym-bridge-org-update-heading
    (buffer heading changes preconditions)
  "Update an Org HEADING in BUFFER after checked structure preconditions."
  (let* ((checks (soma-gym-bridge--org-update-heading-checks
                  buffer heading changes preconditions))
         (has-title (and (listp changes)
                         (soma-gym-bridge--has-key-p changes "title")))
         (title (and has-title
                     (soma-gym-bridge--get changes "title")))
         (has-todo (and (listp changes)
                        (soma-gym-bridge--has-key-p changes "todo")))
         (todo (and has-todo
                    (soma-gym-bridge--get changes "todo")))
         before after target-position error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "org.update_heading")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "route" "org-update-heading")
         (cons "precondition_checks" checks)
         (cons "before" :json-false)
         (cons "after" :json-false)
         (cons "error" "Org heading update precondition failed"))
      (setq before (soma-gym-bridge-observe-org-tree buffer 200 200 400))
      (condition-case err
          (with-current-buffer buffer
            (save-excursion
              (save-restriction
                (widen)
                (setq target-position
                      (soma-gym-bridge--org-resolve-heading-position
                       heading))
                (unless target-position
                  (error "observed Org heading no longer resolves"))
                (goto-char target-position)
                (org-back-to-heading t)
                (atomic-change-group
                  (when has-title
                    (org-edit-headline title))
                  (when has-todo
                    (org-entry-put
                     nil "TODO"
                     (cond
                      ((or (null todo) (eq todo :json-false)) "")
                      ((stringp todo) todo)
                      (t (error "todo change must be a string or JSON false")))))))))
        (error
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-org-tree buffer 200 200 400)))
      (soma-gym-bridge--alist
       (cons "operation" "org.update_heading")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "route" "org-update-heading")
       (cons "target_position" (or target-position :json-false))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge-org-edit-src-block
    (buffer source-block preconditions)
  "Open SOURCE-BLOCK in BUFFER through Org's source-edit workflow."
  (let* ((checks (soma-gym-bridge--org-source-block-checks
                  buffer source-block preconditions))
         (begin (and (listp source-block)
                     (soma-gym-bridge--get source-block "begin")))
         before parent-after-open edit-buffer edit-buffer-name parent-markers
         error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "org.edit_src_block")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "route" "org-edit-src-code")
         (cons "precondition_checks" checks)
         (cons "edit_buffer_name" :json-false)
         (cons "parent_markers" :json-false)
         (cons "parent_after_open" :json-false)
         (cons "error" "Org source-edit precondition failed"))
      (setq before (soma-gym-bridge-observe-org-tree buffer 200 200 400))
      (condition-case err
          (with-current-buffer buffer
            (save-restriction
              (widen)
              (goto-char begin)
              (org-edit-src-code)
              (setq edit-buffer (current-buffer))
              (setq edit-buffer-name (buffer-name edit-buffer))
              (setq parent-markers
                    (soma-gym-bridge--org-source-edit-parent-markers
                     edit-buffer))))
        (error
         (setq error-message (error-message-string err))))
      (setq parent-after-open
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-org-tree buffer 200 200 400)))
      (soma-gym-bridge--alist
       (cons "operation" "org.edit_src_block")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "route" "org-edit-src-code")
       (cons "precondition_checks" checks)
       (cons "source_block" source-block)
       (cons "before" before)
       (cons "edit_buffer_name" (or edit-buffer-name :json-false))
       (cons "source_edit_buffer_separate"
             (soma-gym-bridge--bool
              (and (buffer-live-p edit-buffer)
                   (not (eq edit-buffer buffer)))))
       (cons "parent_markers" (or parent-markers :json-false))
       (cons "parent_after_open" (or parent-after-open :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge--org-source-edit-parent-buffer (edit-buffer)
  "Return the parent Org buffer for source EDIT-BUFFER, or nil."
  (when (buffer-live-p edit-buffer)
    (with-current-buffer edit-buffer
      (and (soma-gym-bridge--load-org)
           (fboundp 'org-src-edit-buffer-p)
           (org-src-edit-buffer-p)
           (condition-case nil
               (org-src-source-buffer)
             (error nil))))))

(defun soma-gym-bridge--org-exit-src-edit-checks
    (edit-buffer preconditions)
  "Return precondition checks for `org.exit_src_edit'."
  (let* ((expected-edit-buffer
          (soma-gym-bridge--get preconditions "edit_buffer_name"))
         (expected-edit-tick
          (soma-gym-bridge--get preconditions "edit_buffer_chars_tick"))
         (expected-edit-hash
          (soma-gym-bridge--get preconditions "edit_text_sha256"))
         (expected-parent-name
          (soma-gym-bridge--get preconditions "parent_buffer_name"))
         (expected-parent-owner
          (soma-gym-bridge--get preconditions "parent_text_owner_name"))
         (expected-parent-file
          (soma-gym-bridge--get preconditions "parent_file_name"))
         (expected-parent-tick
          (soma-gym-bridge--get preconditions "parent_chars_tick"))
         (expected-parent-fingerprint
          (soma-gym-bridge--get preconditions
                                "parent_structure_fingerprint"))
         (expected-markers
          (soma-gym-bridge--get preconditions "parent_markers"))
         (expected-source-block
          (soma-gym-bridge--get preconditions "source_block"))
         (parent-buffer
          (soma-gym-bridge--org-source-edit-parent-buffer edit-buffer))
         (org-available (soma-gym-bridge--load-org))
         edit-checks parent-checks source-checks)
    (setq edit-checks
          (append
           (list
            (soma-gym-bridge--check
             "edit buffer is live"
             (buffer-live-p edit-buffer)
             (format "%S" edit-buffer))
            (soma-gym-bridge--check
             "Org source library is available"
             org-available
             (if org-available "loaded" "unavailable")))
           (when (buffer-live-p edit-buffer)
             (with-current-buffer edit-buffer
               (let ((text (buffer-substring-no-properties
                            (point-min) (point-max))))
                 (list
                  (soma-gym-bridge--check
                   "edit buffer identity matches precondition"
                   (or (not expected-edit-buffer)
                       (string= expected-edit-buffer
                                (buffer-name edit-buffer)))
                   (format "expected=%S actual=%S"
                           expected-edit-buffer
                           (buffer-name edit-buffer)))
                  (soma-gym-bridge--check
                   "edit buffer is an Org source-edit buffer"
                   (and org-available
                        (fboundp 'org-src-edit-buffer-p)
                        (org-src-edit-buffer-p))
                   (format "major-mode=%S" major-mode))
                  (soma-gym-bridge--check
                   "source-edit writeback is allowed"
                   (and (boundp 'org-src--allow-write-back)
                        org-src--allow-write-back)
                   (format "%S"
                           (and (boundp 'org-src--allow-write-back)
                                org-src--allow-write-back)))
                  (soma-gym-bridge--check
                   "edit buffer character revision matches precondition"
                   (or (not expected-edit-tick)
                       (= expected-edit-tick
                          (buffer-chars-modified-tick)))
                   (format "expected=%S actual=%S"
                           expected-edit-tick
                           (buffer-chars-modified-tick)))
                  (soma-gym-bridge--check
                   "edit buffer text hash matches precondition"
                   (and (stringp expected-edit-hash)
                        (string= expected-edit-hash
                                 (secure-hash 'sha256 text)))
                   (format "expected=%S actual=%S"
                           expected-edit-hash
                           (secure-hash 'sha256 text)))))))))
    (setq parent-checks
          (append
           (list
            (soma-gym-bridge--check
             "parent Org buffer is still reachable from edit markers"
             (buffer-live-p parent-buffer)
             (format "%S" parent-buffer)))
           (when (buffer-live-p parent-buffer)
             (let ((parent-preconditions
                    (soma-gym-bridge--alist
                     (cons "buffer_name" expected-parent-name)
                     (cons "text_owner_name" expected-parent-owner)
                     (cons "buffer_file_name" expected-parent-file)
                     (cons "chars_tick" expected-parent-tick))))
               (append
                (soma-gym-bridge--buffer-precondition-checks
                 parent-buffer parent-preconditions)
                (with-current-buffer parent-buffer
                  (save-restriction
                    (widen)
                    (list
                     (soma-gym-bridge--check
                      "parent structure fingerprint matches precondition"
                      (and (stringp expected-parent-fingerprint)
                           (string= expected-parent-fingerprint
                                    (soma-gym-bridge--org-structure-fingerprint)))
                      (format "expected=%S actual=%S"
                              expected-parent-fingerprint
                              (soma-gym-bridge--org-structure-fingerprint)))))))))))
    (when (buffer-live-p parent-buffer)
      (with-current-buffer parent-buffer
        (when (and org-available (derived-mode-p 'org-mode))
          (save-restriction
            (widen)
            (let ((current-source-block
                   (and (listp expected-source-block)
                        (soma-gym-bridge--org-find-source-block
                         expected-source-block 4000)))
                  (current-markers
                   (soma-gym-bridge--org-source-edit-parent-markers
                    edit-buffer)))
              (setq source-checks
                    (list
                     (soma-gym-bridge--check
                      "parent_markers precondition is an object"
                      (listp expected-markers)
                      (format "%S" expected-markers))
                     (soma-gym-bridge--check
                      "edit-buffer parent markers match precondition"
                      (soma-gym-bridge--org-parent-markers-match-p
                       current-markers expected-markers)
                      (format "expected=%S actual=%S"
                              expected-markers current-markers))
                     (soma-gym-bridge--check
                      "source_block precondition is an object"
                      (listp expected-source-block)
                      (format "%S" expected-source-block))
                     (soma-gym-bridge--check
                      "parent source block still resolves before writeback"
                      (listp current-source-block)
                      (format "%S" current-source-block))
                     (soma-gym-bridge--check
                      "parent source block still matches observed identity"
                      (soma-gym-bridge--org-source-block-fields-match-p
                       expected-source-block current-source-block)
                      (format "expected=%S actual=%S"
                              expected-source-block current-source-block))
                     (soma-gym-bridge--check
                      "parent marker text matches observed source block"
                      (and (listp current-markers)
                           (stringp (soma-gym-bridge--get
                                     expected-source-block "value_sha256"))
                           (equal
                            (soma-gym-bridge--get current-markers
                                                  "text_sha256")
                            (soma-gym-bridge--get expected-source-block
                                                  "value_sha256")))
                      (format "markers=%S source=%S"
                              current-markers expected-source-block)))))))))
    (append edit-checks parent-checks source-checks)))

(defun soma-gym-bridge-org-exit-src-edit (edit-buffer preconditions)
  "Exit Org source EDIT-BUFFER after checked parent writeback preconditions."
  (let* ((checks (soma-gym-bridge--org-exit-src-edit-checks
                  edit-buffer preconditions))
         (parent-buffer
          (soma-gym-bridge--org-source-edit-parent-buffer edit-buffer))
         before-edit before-parent after-parent error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "org.exit_src_edit")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "route" "org-edit-src-exit")
         (cons "precondition_checks" checks)
         (cons "before_edit" :json-false)
         (cons "before_parent" :json-false)
         (cons "after_parent" :json-false)
         (cons "error" "Org source-edit exit precondition failed"))
      (setq before-edit (soma-gym-bridge-observe-buffer edit-buffer))
      (setq before-parent
            (and (buffer-live-p parent-buffer)
                 (soma-gym-bridge-observe-org-tree parent-buffer 200 200 400)))
      (condition-case err
          (with-current-buffer edit-buffer
            (org-edit-src-exit))
        (error
         (setq error-message (error-message-string err))))
      (setq after-parent
            (and (buffer-live-p parent-buffer)
                 (soma-gym-bridge-observe-org-tree parent-buffer 200 200 400)))
      (soma-gym-bridge--alist
       (cons "operation" "org.exit_src_edit")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "route" "org-edit-src-exit")
       (cons "precondition_checks" checks)
       (cons "before_edit" (or before-edit :json-false))
       (cons "before_parent" (or before-parent :json-false))
       (cons "after_parent" (or after-parent :json-false))
       (cons "edit_buffer_live_after"
             (soma-gym-bridge--bool (buffer-live-p edit-buffer)))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge--org-babel-source-checks
    (buffer source-block preconditions)
  "Return precondition checks for `org.execute_babel_block'."
  (let* ((source-checks
          (soma-gym-bridge--org-source-block-checks
           buffer source-block preconditions))
         (allowlist
          (soma-gym-bridge--string-list
           (soma-gym-bridge--get preconditions "language_allowlist")))
         (expected-session
          (soma-gym-bridge--org-babel-normalize-session
           (soma-gym-bridge--get preconditions "session")))
         (expected-default-directory
          (soma-gym-bridge--get preconditions "default_directory"))
         (begin (and (listp source-block)
                     (soma-gym-bridge--get source-block "begin")))
         (babel-core-available (soma-gym-bridge--load-org-babel-core))
         info info-error language session actual-default-directory
         backend-feature backend-loaded)
    (when (and (buffer-live-p buffer)
               babel-core-available
               (integerp begin))
      (with-current-buffer buffer
        (when (derived-mode-p 'org-mode)
          (save-restriction
            (widen)
            (when (<= (point-min) begin (point-max))
              (save-excursion
                (condition-case err
                    (progn
                      (goto-char begin)
                      (setq info (org-babel-get-src-block-info 'no-eval))
                      (setq language (and (listp info) (nth 0 info)))
                      (setq session
                            (soma-gym-bridge--org-babel-normalize-session
                             (cdr (assq :session (nth 2 info)))))
                      (setq actual-default-directory
                            (file-name-as-directory
                             (expand-file-name default-directory)))
                      (setq backend-feature
                            (soma-gym-bridge--org-babel-language-feature
                             language))
                      (setq backend-loaded
                            (and (member language allowlist)
                                 (soma-gym-bridge--org-babel-load-language
                                  language))))
                  (error
                   (setq info-error (error-message-string err))))))))))
    (append
     source-checks
     (list
      (soma-gym-bridge--check
       "Org Babel core is available"
       babel-core-available
       (if babel-core-available "loaded" "unavailable"))
      (soma-gym-bridge--check
       "language_allowlist is an explicit list of strings"
       (and allowlist t)
       (format "%S" (soma-gym-bridge--get preconditions
                                           "language_allowlist")))
      (soma-gym-bridge--check
       "Babel source block info is available"
       (listp info)
       (or info-error (format "%S" info)))
      (soma-gym-bridge--check
       "source block language matches Babel info"
       (and (stringp language)
            (string= language
                     (soma-gym-bridge--get source-block "language")))
       (format "source_block=%S info_language=%S"
               (soma-gym-bridge--get source-block "language")
               language))
      (soma-gym-bridge--check
       "source block language is explicitly allowed"
       (and (stringp language)
            (member language allowlist))
       (format "language=%S allowlist=%S" language allowlist))
      (soma-gym-bridge--check
       "source block language has a supported Babel backend"
       backend-feature
       (format "language=%S backend=%S" language backend-feature))
      (soma-gym-bridge--check
       "supported Babel backend is loaded"
       backend-loaded
       (format "language=%S backend=%S" language backend-feature))
      (soma-gym-bridge--check
       "Babel session matches precondition"
       (equal expected-session session)
       (format "expected=%S actual=%S" expected-session session))
      (soma-gym-bridge--check
       "default-directory matches precondition"
       (soma-gym-bridge--same-directory-p
        expected-default-directory actual-default-directory)
       (format "expected=%S actual=%S"
               expected-default-directory actual-default-directory))))))

(defun soma-gym-bridge--org-babel-runtime-identity (info)
  "Return runtime evidence for Org Babel INFO."
  (let* ((language (and (listp info) (nth 0 info)))
         (executor (and (stringp language)
                        (intern (concat "org-babel-execute:" language)))))
    (soma-gym-bridge--alist
     (cons "language" (or language :json-false))
     (cons "backend_feature"
           (or (and (stringp language)
                    (soma-gym-bridge--json-symbol
                     (soma-gym-bridge--org-babel-language-feature language)))
               :json-false))
     (cons "executor"
           (or (and executor
                    (soma-gym-bridge--json-symbol executor))
               :json-false))
     (cons "executor_fboundp"
           (soma-gym-bridge--bool (and executor (fboundp executor))))
     (cons "session"
           (soma-gym-bridge--org-babel-normalize-session
            (cdr (assq :session (nth 2 info)))))
     (cons "default_directory"
           (file-name-as-directory (expand-file-name default-directory)))
     (cons "emacs_pid" (emacs-pid))
     (cons "emacs_executable"
           (expand-file-name invocation-name invocation-directory))
     (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
     (cons "plan" (or (getenv "SOMA_INSULA_PLAN") "")))))

(defun soma-gym-bridge-org-execute-babel-block
    (buffer source-block preconditions timeout-seconds)
  "Execute SOURCE-BLOCK in BUFFER through Org Babel after checked preconditions."
  (let* ((checks (soma-gym-bridge--org-babel-source-checks
                  buffer source-block preconditions))
         (begin (and (listp source-block)
                     (soma-gym-bridge--get source-block "begin")))
         before after info info-observation runtime result-block result-value
         error-message timed-out completed state)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "org.execute_babel_block")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "route" "org-babel-execute-src-block")
         (cons "precondition_checks" checks)
         (cons "before" :json-false)
         (cons "after" :json-false)
         (cons "runtime_identity" :json-false)
         (cons "parent_result_block" :json-false)
         (cons "error" "Org Babel execution precondition failed"))
      (setq before (soma-gym-bridge-observe-org-tree buffer 200 200 400))
      (condition-case err
          (with-current-buffer buffer
            (save-restriction
              (widen)
              (goto-char begin)
              (setq info (org-babel-get-src-block-info 'no-eval))
              (setq info-observation
                    (soma-gym-bridge--org-babel-info-observation info))
              (setq runtime
                    (soma-gym-bridge--org-babel-runtime-identity info))
              (let ((org-confirm-babel-evaluate nil))
                (if (and (numberp timeout-seconds)
                         (> timeout-seconds 0))
                    (setq completed
                          (with-timeout (timeout-seconds
                                          (setq timed-out t)
                                          nil)
                            (setq result-value
                                  (org-babel-execute-src-block))
                            t))
                  (setq result-value (org-babel-execute-src-block))
                  (setq completed t)))
              (goto-char begin)
              (setq result-block
                    (soma-gym-bridge--org-babel-result-observation))))
        (error
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-org-tree buffer 200 200 400)))
      (setq state
            (cond
             (timed-out "outcome_unknown")
             (error-message "failed")
             (completed "succeeded")
             (t "outcome_unknown")))
      (soma-gym-bridge--alist
       (cons "operation" "org.execute_babel_block")
       (cons "state" state)
       (cons "state_sequence"
             (cond
              ((string= state "succeeded")
               '("accepted" "running" "succeeded"))
              ((string= state "outcome_unknown")
               '("accepted" "running" "outcome_unknown"))
              (t
               '("accepted" "running" "failed"))))
       (cons "route" "org-babel-execute-src-block")
       (cons "source_block" source-block)
       (cons "babel_info" (or info-observation :json-false))
       (cons "runtime_identity" (or runtime :json-false))
       (cons "result_value" (soma-gym-bridge--json-safe-value result-value))
       (cons "parent_result_block" (or result-block :json-false))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "timeout_seconds"
             (if (numberp timeout-seconds) timeout-seconds :json-false))
       (cons "error"
             (cond
              (error-message error-message)
              (timed-out "Org Babel execution timed out; outcome is unknown")
              (t :json-false)))))))

(defun soma-gym-bridge--org-export-precondition-checks
    (buffer output-path preconditions)
  (let* ((expected-buffer (soma-gym-bridge--get preconditions "buffer_name"))
         (expected-tick (soma-gym-bridge--get preconditions "chars_tick"))
         (publish-directory
          (soma-gym-bridge--get preconditions "publish_directory"))
         (export-options
          (soma-gym-bridge--get preconditions "export_options"))
         (expanded-output
          (and (stringp output-path) (expand-file-name output-path)))
         (expanded-publish
          (and (stringp publish-directory)
               (file-name-as-directory
                (expand-file-name publish-directory))))
         (inside-publish
          (and expanded-output
               expanded-publish
               (string-prefix-p expanded-publish expanded-output)))
         (resolved-publish
          (and expanded-publish
               (ignore-errors
                 (file-name-as-directory (file-truename expanded-publish)))))
         (resolved-output
          (and expanded-output
               (let ((expanded-output-dir
                      (file-name-directory expanded-output)))
                 (if (or (file-exists-p expanded-output)
                         (file-symlink-p expanded-output))
                     (ignore-errors (file-truename expanded-output))
                   (let ((resolved-output-dir
                          (and expanded-output-dir
                               (ignore-errors
                                 (file-name-as-directory
                                  (file-truename expanded-output-dir))))))
                     (and resolved-output-dir
                          (expand-file-name
                           (file-name-nondirectory expanded-output)
                           resolved-output-dir)))))))
         (resolved-inside-publish
          (and resolved-output
               resolved-publish
               (soma-gym-bridge--path-under-prefix-p
                resolved-output
                resolved-publish))))
    (append
     (list
      (soma-gym-bridge--check
       "target buffer is live"
       (buffer-live-p buffer)
       (format "%S" buffer))
      (soma-gym-bridge--check
       "output path is a string"
       (stringp output-path)
       (format "%S" output-path))
      (soma-gym-bridge--check
       "publish directory is explicit"
       (stringp publish-directory)
       (format "%S" publish-directory))
      (soma-gym-bridge--check
       "export options are explicit"
       (or (hash-table-p export-options) (listp export-options))
       (format "%S" export-options))
      (soma-gym-bridge--check
       "output path is inside publish directory"
       inside-publish
       (format "output=%S publish=%S" expanded-output expanded-publish))
      (soma-gym-bridge--check
       "output path resolves inside publish directory"
       resolved-inside-publish
       (format "output=%S publish=%S"
               resolved-output resolved-publish)))
     (when (buffer-live-p buffer)
       (with-current-buffer buffer
         (list
          (soma-gym-bridge--check
           "buffer identity matches precondition"
           (or (not expected-buffer)
               (string= expected-buffer (buffer-name buffer)))
           (format "expected=%S actual=%S"
                   expected-buffer (buffer-name buffer)))
          (soma-gym-bridge--check
           "character revision matches precondition"
           (or (not expected-tick)
               (= expected-tick (buffer-chars-modified-tick)))
           (format "expected=%S actual=%S"
                   expected-tick (buffer-chars-modified-tick)))
          (soma-gym-bridge--check
           "buffer is in org-mode"
           (derived-mode-p 'org-mode)
           (symbol-name major-mode))))))))

(defun soma-gym-bridge-org-export-html
    (buffer output-path preconditions &optional mode)
  "Preview or export BUFFER to OUTPUT-PATH with checked Org report preconditions."
  (let* ((export-mode
          (if (and (stringp mode) (not (string-empty-p mode)))
              mode
            "apply"))
         (mode-valid (member export-mode '("preview" "apply")))
         (checks (soma-gym-bridge--org-export-precondition-checks
                  buffer output-path preconditions))
         (publish-directory
          (soma-gym-bridge--get preconditions "publish_directory"))
         (export-options
          (soma-gym-bridge--get preconditions "export_options"))
         (asset-policy
          (soma-gym-bridge--alist
           (cons "mode" "org-default-html")
           (cons "external_assets" nil)
           (cons "publish_directory"
                 (or publish-directory :json-false))))
         (exporter-available nil)
         before after generated error-message)
    (cond
     ((not mode-valid)
      (soma-gym-bridge--alist
       (cons "operation" "org.export_html")
       (cons "state" "failed")
       (cons "state_sequence" '("accepted" "failed"))
       (cons "mode" (or export-mode :json-false))
       (cons "route" "org-export")
       (cons "precondition_checks" checks)
       (cons "generated_file" :json-false)
       (cons "generated_files" nil)
       (cons "asset_policy" asset-policy)
       (cons "export_diagnostics"
             (soma-gym-bridge--alist
              (cons "backend" "html")
              (cons "mode" (or export-mode :json-false))
              (cons "valid_mode" :json-false)))
       (cons "error" "Org export mode must be preview or apply")))
     ((not (soma-gym-bridge--all-checks-pass-p checks))
      (soma-gym-bridge--alist
       (cons "operation" "org.export_html")
       (cons "state" "failed")
       (cons "state_sequence" '("accepted" "failed"))
       (cons "mode" export-mode)
       (cons "route" "org-export")
       (cons "precondition_checks" checks)
       (cons "generated_file" :json-false)
       (cons "generated_files" nil)
       (cons "asset_policy" asset-policy)
       (cons "export_diagnostics"
             (soma-gym-bridge--alist
              (cons "backend" "html")
              (cons "mode" export-mode)
              (cons "precondition_checks_passed" :json-false)))
       (cons "error" "Org export precondition failed")))
     (t
      (setq before (soma-gym-bridge-observe-org-tree buffer 200 200 400))
      (setq exporter-available
            (and (soma-gym-bridge--load-org)
                 (require 'ox-html nil t)))
      (cond
       ((not exporter-available)
        (setq error-message "Org HTML exporter is not available in this Emacs"))
       ((string= export-mode "apply")
        (condition-case err
            (progn
              (make-directory (file-name-directory output-path) t)
              (with-current-buffer buffer
                (let ((org-export-show-temporary-export-buffer nil))
                  (setq generated
                        (org-export-to-file 'html output-path)))))
          (error
           (setq error-message (error-message-string err))))))
      (setq after
            (soma-gym-bridge--alist
             (cons "org_tree"
                   (and (buffer-live-p buffer)
                        (soma-gym-bridge-observe-org-tree
                         buffer 200 200 400)))
             (cons "output_signature"
                   (and (stringp output-path)
                        (soma-gym-bridge--file-signature output-path)))))
      (soma-gym-bridge--alist
       (cons "operation" "org.export_html")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "mode" export-mode)
       (cons "route"
             (if (string= export-mode "preview")
                 "org-export-preview"
               "org-export"))
       (cons "output_path" output-path)
       (cons "generated_file" (or generated :json-false))
       (cons "generated_files" (if generated (list generated) nil))
       (cons "export_preview"
             (soma-gym-bridge--alist
              (cons "backend" "html")
              (cons "mode" export-mode)
              (cons "planned_output_path" (or output-path :json-false))
              (cons "will_write" (if (string= export-mode "apply")
                                     t
                                   :json-false))
              (cons "current_output_signature"
                    (and (stringp output-path)
                         (soma-gym-bridge--file-signature output-path)))))
       (cons "asset_policy" asset-policy)
       (cons "export_diagnostics"
             (soma-gym-bridge--alist
              (cons "backend" "html")
              (cons "mode" export-mode)
              (cons "exporter_available"
                    (soma-gym-bridge--bool exporter-available))
              (cons "precondition_checks_passed" t)
              (cons "error" (or error-message :json-false))))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" after)
       (cons "error" (or error-message :json-false)))))))

(defun soma-gym-bridge--dired-safe-relative-name-p (name)
  (and (stringp name)
       (not (string-empty-p name))
       (not (file-name-absolute-p name))
       (not (file-name-directory name))
       (not (member name '("." "..")))))

(defun soma-gym-bridge--dired-buffer-directory (buffer)
  (with-current-buffer buffer
    (file-name-as-directory (file-truename default-directory))))

(defun soma-gym-bridge--dired-marked-basenames ()
  (condition-case nil
      (sort
       (mapcar #'file-name-nondirectory
               (or (dired-get-marked-files nil 'marked nil nil nil) nil))
       #'string<)
    (error nil)))

(defun soma-gym-bridge--dired-flagged-delete-basenames ()
  (let ((marker (if (boundp 'dired-del-marker) dired-del-marker ?D))
        files)
    (save-excursion
      (goto-char (point-min))
      (while (< (point) (point-max))
        (when (eq (char-after (line-beginning-position)) marker)
          (let ((file (ignore-errors (dired-get-filename nil t))))
            (when (stringp file)
              (push (file-name-nondirectory file) files))))
        (forward-line 1)))
    (sort files #'string<)))

(defun soma-gym-bridge--dired-file-signature-entry (entries name)
  "Return the expected signature entry for NAME in ENTRIES."
  (seq-find
   (lambda (entry)
     (let ((object (soma-gym-bridge--object-alist entry)))
       (and (listp object)
            (string= (or (soma-gym-bridge--get object "name") "")
                     name))))
   (if (vectorp entries) (append entries nil) entries)))

(defun soma-gym-bridge--dired-filesystem-state (directory names)
  "Return file-signature evidence for NAMES under DIRECTORY."
  (mapcar
   (lambda (name)
     (soma-gym-bridge--alist
      (cons "name" name)
      (cons "signature"
            (soma-gym-bridge--file-signature
             (expand-file-name name directory)))))
   names))

(defun soma-gym-bridge-observe-dired
    (buffer max-files)
  "Return Dired selection and filesystem-signature evidence for BUFFER."
  (cond
   ((not (buffer-live-p buffer))
    (soma-gym-bridge--alist
     (cons "operation" "dired.observe")
     (cons "state" "failed")
     (cons "error" "buffer_name does not name a live buffer")))
   (t
    (with-current-buffer buffer
      (if (not (derived-mode-p 'dired-mode))
          (soma-gym-bridge--alist
           (cons "operation" "dired.observe")
           (cons "state" "failed")
           (cons "buffer_name" (buffer-name buffer))
           (cons "major_mode" (symbol-name major-mode))
           (cons "error" "buffer is not in dired-mode"))
        (let* ((directory (file-name-as-directory
                           (file-truename default-directory)))
               (names (sort (directory-files directory nil
                                             directory-files-no-dot-files-regexp)
                            #'string<))
               (file-count (length names))
               (truncated (> file-count max-files))
               (returned (seq-take names max-files)))
          (soma-gym-bridge--alist
           (cons "operation" "dired.observe")
           (cons "state" "succeeded")
           (cons "buffer_name" (buffer-name buffer))
           (cons "text_owner_name"
                 (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
           (cons "buffer_file_name" (or buffer-file-name :json-false))
           (cons "major_mode" (symbol-name major-mode))
           (cons "chars_tick" (buffer-chars-modified-tick))
           (cons "modified" (soma-gym-bridge--bool (buffer-modified-p)))
           (cons "read_only" (soma-gym-bridge--bool buffer-read-only))
           (cons "point" (point))
           (cons "mark_active" (soma-gym-bridge--bool mark-active))
           (cons "narrowed" (soma-gym-bridge--bool (buffer-narrowed-p)))
           (cons "selected_window_buffer"
                 (if (window-live-p (selected-window))
                     (buffer-name (window-buffer (selected-window)))
                   :json-false))
           (cons "minibuffer_active"
                 (soma-gym-bridge--bool (active-minibuffer-window)))
           (cons "directory" directory)
           (cons "marked_files" (soma-gym-bridge--dired-marked-basenames))
           (cons "deletion_flagged_files"
                 (soma-gym-bridge--dired-flagged-delete-basenames))
           (cons "file_count" file-count)
           (cons "files_returned" (length returned))
           (cons "files_truncated" (soma-gym-bridge--bool truncated))
           (cons "files"
                 (mapcar
                  (lambda (name)
                    (soma-gym-bridge--alist
                     (cons "name" name)
                     (cons "signature"
                           (soma-gym-bridge--file-signature
                            (expand-file-name name directory)))))
                  returned)))))))))

(defun soma-gym-bridge--file-signature-matches-p (expected actual)
  (and (listp expected)
       (eq (soma-gym-bridge--get expected "exists") t)
       (eq (soma-gym-bridge--get actual "exists") t)
       (equal (soma-gym-bridge--get expected "truename")
              (soma-gym-bridge--get actual "truename"))
       (equal (soma-gym-bridge--get expected "size")
              (soma-gym-bridge--get actual "size"))
       (equal (soma-gym-bridge--get expected "sha256")
              (soma-gym-bridge--get actual "sha256"))))

(defun soma-gym-bridge--dired-rename-precondition-checks
    (buffer source target preconditions)
  (let* ((expected-buffer (soma-gym-bridge--get preconditions "buffer_name"))
         (expected-directory
          (soma-gym-bridge--get preconditions "directory"))
         (expected-signature
          (soma-gym-bridge--get preconditions "file_signature"))
         (expected-target-absent
          (soma-gym-bridge--get preconditions "target_absent"))
         (collision-policy
          (soma-gym-bridge--get preconditions "collision_policy"))
         (directory (and (buffer-live-p buffer)
                         (soma-gym-bridge--dired-buffer-directory buffer)))
         (source-safe (soma-gym-bridge--dired-safe-relative-name-p source))
         (target-safe (soma-gym-bridge--dired-safe-relative-name-p target))
         (source-path (and directory source-safe
                           (expand-file-name source directory)))
         (target-path (and directory target-safe
                           (expand-file-name target directory)))
         (actual-signature
          (and source-path (soma-gym-bridge--file-signature source-path))))
    (list
     (soma-gym-bridge--check
      "target buffer is live"
      (buffer-live-p buffer)
      (format "%S" buffer))
     (soma-gym-bridge--check
      "buffer identity matches precondition"
      (and (buffer-live-p buffer)
           (or (not expected-buffer)
               (string= expected-buffer (buffer-name buffer))))
      (format "expected=%S actual=%S"
              expected-buffer
              (and (buffer-live-p buffer) (buffer-name buffer))))
     (soma-gym-bridge--check
      "buffer is in dired-mode"
      (and (buffer-live-p buffer)
           (with-current-buffer buffer
             (derived-mode-p 'dired-mode)))
      (and (buffer-live-p buffer)
           (with-current-buffer buffer (symbol-name major-mode))))
     (soma-gym-bridge--check
      "Dired directory matches precondition"
      (and directory
           (stringp expected-directory)
           (string= directory
                    (file-name-as-directory
                     (file-truename expected-directory))))
      (format "expected=%S actual=%S" expected-directory directory))
     (soma-gym-bridge--check
      "source target names are safe relative basenames"
      (and source-safe target-safe)
      (format "source=%S target=%S" source target))
     (soma-gym-bridge--check
      "source file signature matches precondition"
      (soma-gym-bridge--file-signature-matches-p
       expected-signature actual-signature)
      (format "expected=%S actual=%S"
              expected-signature actual-signature))
     (soma-gym-bridge--check
      "target is absent for no-overwrite rename"
      (and (eq expected-target-absent t)
           target-path
           (not (file-exists-p target-path)))
      (format "expected_absent=%S target=%S exists=%S"
              expected-target-absent target-path
              (and target-path (file-exists-p target-path))))
     (soma-gym-bridge--check
      "collision policy is no_overwrite"
      (string= collision-policy "no_overwrite")
      (format "%S" collision-policy)))))

(defun soma-gym-bridge--dired-delete-precondition-checks
    (buffer preconditions)
  (let* ((expected-buffer (soma-gym-bridge--get preconditions "buffer_name"))
         (expected-directory
          (soma-gym-bridge--get preconditions "directory"))
         (expected-marked
          (soma-gym-bridge--string-list
           (soma-gym-bridge--get preconditions "marked_files")))
         (expected-marked-sorted
          (and expected-marked (sort (copy-sequence expected-marked)
                                     #'string<)))
         (file-signatures
          (soma-gym-bridge--get preconditions "file_signatures"))
         (delete-policy
          (soma-gym-bridge--object-alist
           (soma-gym-bridge--get preconditions "delete_policy")))
         (directory (and (buffer-live-p buffer)
                         (soma-gym-bridge--dired-buffer-directory buffer)))
         (actual-marked
          (and (buffer-live-p buffer)
               (with-current-buffer buffer
                 (soma-gym-bridge--dired-marked-basenames))))
         (marked-safe
          (and expected-marked-sorted
               (cl-every #'soma-gym-bridge--dired-safe-relative-name-p
                         expected-marked-sorted)))
         (unique-marked
          (and expected-marked-sorted
               (= (length expected-marked-sorted)
                  (length (delete-dups
                           (copy-sequence expected-marked-sorted))))))
         (signature-checks
          (mapcar
           (lambda (name)
             (let* ((entry (soma-gym-bridge--dired-file-signature-entry
                            file-signatures name))
                    (entry-object
                     (and entry (soma-gym-bridge--object-alist entry)))
                    (expected-signature
                     (and (listp entry-object)
                          (soma-gym-bridge--object-alist
                           (soma-gym-bridge--get entry-object
                                                 "signature"))))
                    (actual-signature
                     (and directory marked-safe
                          (soma-gym-bridge--file-signature
                           (expand-file-name name directory)))))
               (soma-gym-bridge--check
                (format "file signature for marked file %s matches"
                        name)
                (and (soma-gym-bridge--file-signature-matches-p
                      expected-signature actual-signature)
                     (string= (or (soma-gym-bridge--get actual-signature
                                                        "type")
                                  "")
                              "regular"))
                (format "expected=%S actual=%S"
                        expected-signature actual-signature))))
           (or expected-marked-sorted nil))))
    (append
     (list
      (soma-gym-bridge--check
       "target buffer is live"
       (buffer-live-p buffer)
       (format "%S" buffer))
      (soma-gym-bridge--check
       "buffer identity matches precondition"
       (and (buffer-live-p buffer)
            (stringp expected-buffer)
            (string= expected-buffer (buffer-name buffer)))
       (format "expected=%S actual=%S"
               expected-buffer
               (and (buffer-live-p buffer) (buffer-name buffer))))
      (soma-gym-bridge--check
       "buffer is in dired-mode"
       (and (buffer-live-p buffer)
            (with-current-buffer buffer
              (derived-mode-p 'dired-mode)))
       (and (buffer-live-p buffer)
            (with-current-buffer buffer (symbol-name major-mode))))
      (soma-gym-bridge--check
       "Dired directory matches precondition"
       (and directory
            (soma-gym-bridge--same-directory-p expected-directory
                                               directory))
       (format "expected=%S actual=%S" expected-directory directory))
      (soma-gym-bridge--check
       "marked files are explicit safe relative basenames"
       (and expected-marked-sorted marked-safe unique-marked
            (consp expected-marked-sorted))
       (format "marked=%S" expected-marked))
      (soma-gym-bridge--check
       "current Dired ordinary marks match precondition"
       (equal expected-marked-sorted actual-marked)
       (format "expected=%S actual=%S"
               expected-marked-sorted actual-marked))
      (soma-gym-bridge--check
       "delete policy selects ordinary marks only"
       (and (listp delete-policy)
            (string= (or (soma-gym-bridge--get delete-policy "selection")
                         "")
                     "marked")
            (eq (soma-gym-bridge--get
                 delete-policy "delete_by_moving_to_trash")
                :json-false)
            (eq (soma-gym-bridge--get
                 delete-policy "recursive_deletes")
                :json-false))
       (format "%S" delete-policy)))
     signature-checks)))

(defun soma-gym-bridge-dired-delete-marked (buffer preconditions)
  "Delete ordinary Dired marks through Dired after filesystem checks."
  (require 'dired)
  (require 'dired-aux)
  (let* ((expected-marked
          (soma-gym-bridge--string-list
           (soma-gym-bridge--get preconditions "marked_files")))
         (marked-names
          (and expected-marked
               (sort (copy-sequence expected-marked) #'string<)))
         (marked-names-safe
          (and marked-names
               (cl-every #'soma-gym-bridge--dired-safe-relative-name-p
                         marked-names)))
         (directory (and (buffer-live-p buffer)
                         (soma-gym-bridge--dired-buffer-directory buffer)))
         (checks (soma-gym-bridge--dired-delete-precondition-checks
                  buffer preconditions))
         (precondition-passed
          (soma-gym-bridge--all-checks-pass-p checks))
         before after pre-filesystem-state post-filesystem-state
         flagged-before error-message)
    (setq pre-filesystem-state
          (and directory marked-names marked-names-safe
               (soma-gym-bridge--dired-filesystem-state
                directory marked-names)))
    (if (not precondition-passed)
        (soma-gym-bridge--alist
         (cons "operation" "dired.delete_marked")
         (cons "route" "dired-do-delete")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "precondition_checks" checks)
         (cons "precondition_checks_passed" :json-false)
         (cons "marked_files" (or marked-names :json-false))
         (cons "pre_filesystem_state"
               (or pre-filesystem-state :json-false))
         (cons "post_filesystem_state" :json-false)
         (cons "error" "Dired delete precondition failed"))
      (setq before (soma-gym-bridge-observe-dired buffer 200))
      (setq flagged-before
            (with-current-buffer buffer
              (soma-gym-bridge--dired-flagged-delete-basenames)))
      (condition-case err
          (with-current-buffer buffer
            (let ((dired-no-confirm t)
                  (dired-deletion-confirmer (lambda (&rest _) t))
                  (delete-by-moving-to-trash nil)
                  (dired-recursive-deletes nil))
              (dired-do-delete 'marked))
            (revert-buffer))
        (error
         (setq error-message (error-message-string err))))
      (setq post-filesystem-state
            (and directory marked-names marked-names-safe
                 (soma-gym-bridge--dired-filesystem-state
                  directory marked-names)))
      (setq after (and (buffer-live-p buffer)
                       (soma-gym-bridge-observe-dired buffer 200)))
      (soma-gym-bridge--alist
       (cons "operation" "dired.delete_marked")
       (cons "route" "dired-do-delete")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "precondition_checks" checks)
       (cons "precondition_checks_passed" t)
       (cons "marked_files" (or marked-names :json-false))
       (cons "deletion_flagged_files_before" flagged-before)
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "pre_filesystem_state"
             (or pre-filesystem-state :json-false))
       (cons "post_filesystem_state"
             (or post-filesystem-state :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge-dired-rename-no-overwrite
    (buffer source target preconditions)
  "Rename SOURCE to TARGET through Dired after ownership checks."
  (let* ((checks (soma-gym-bridge--dired-rename-precondition-checks
                  buffer source target preconditions))
         (directory (and (buffer-live-p buffer)
                         (soma-gym-bridge--dired-buffer-directory buffer)))
         (source-path (and directory
                           (soma-gym-bridge--dired-safe-relative-name-p source)
                           (expand-file-name source directory)))
         (target-path (and directory
                           (soma-gym-bridge--dired-safe-relative-name-p target)
                           (expand-file-name target directory)))
         before source-before target-before source-after target-after
         error-message)
    (setq source-before
          (and source-path (soma-gym-bridge--file-signature source-path)))
    (setq target-before
          (and target-path (soma-gym-bridge--file-signature target-path)))
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "dired.rename_no_overwrite")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "source" (or source :json-false))
         (cons "target" (or target :json-false))
         (cons "precondition_checks" checks)
         (cons "source_before" (or source-before :json-false))
         (cons "target_before" (or target-before :json-false))
         (cons "error" "Dired rename precondition failed"))
      (setq before (soma-gym-bridge-observe-dired buffer 200))
      (condition-case err
          (with-current-buffer buffer
            (dired-goto-file source-path)
            (dired-rename-file source-path target-path nil)
            (revert-buffer))
        (error
         (setq error-message (error-message-string err))))
      (setq source-after
            (and source-path (soma-gym-bridge--file-signature source-path)))
      (setq target-after
            (and target-path (soma-gym-bridge--file-signature target-path)))
      (soma-gym-bridge--alist
       (cons "operation" "dired.rename_no_overwrite")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "source" (or source :json-false))
       (cons "target" (or target :json-false))
       (cons "source_path" (or source-path :json-false))
       (cons "target_path" (or target-path :json-false))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" (and (buffer-live-p buffer)
                          (soma-gym-bridge-observe-dired buffer 200)))
       (cons "source_before" (or source-before :json-false))
       (cons "target_before" (or target-before :json-false))
       (cons "source_after" (or source-after :json-false))
       (cons "target_after" (or target-after :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge--lsp-workspace ()
  (and (boundp 'lsp--buffer-workspaces)
       lsp--buffer-workspaces
       (car lsp--buffer-workspaces)))

(defun soma-gym-bridge--lsp-workspace-status (workspace)
  (if workspace
      (soma-gym-bridge--json-symbol
       (ignore-errors (lsp--workspace-status workspace)))
    "missing"))

(defun soma-gym-bridge--lsp-workspace-root (workspace)
  (or (and workspace
           (ignore-errors
             (file-name-as-directory
              (file-truename (lsp--workspace-root workspace)))))
      ""))

(defun soma-gym-bridge--lsp-position-encoding (workspace)
  (or (ignore-errors
        (lsp:server-capabilities-position-encoding?
         (lsp--workspace-server-capabilities workspace)))
      (cond
       ((and (boundp 'lsp--position-column-function)
             (eq lsp--position-column-function #'lsp--utf-16-column))
        "utf-16")
       ((and (boundp 'lsp--position-column-function)
             (eq lsp--position-column-function #'lsp--utf-8-column))
        "utf-8")
       ((and (boundp 'lsp--position-column-function)
             (eq lsp--position-column-function #'lsp--utf-32-column))
        "utf-32")
       (t "unknown"))))

(defun soma-gym-bridge--lsp-method-supported-p (method)
  (soma-gym-bridge--bool (ignore-errors (lsp-feature? method))))

(defun soma-gym-bridge--lsp-capabilities ()
  (mapcar
   (lambda (method)
     (cons method (soma-gym-bridge--lsp-method-supported-p method)))
   '("textDocument/hover"
     "textDocument/references"
     "textDocument/definition"
     "textDocument/documentSymbol"
     "textDocument/rename")))

(defun soma-gym-bridge--lsp-diagnostic-count (diagnostics)
  (cond
   ((vectorp diagnostics) (length diagnostics))
   ((listp diagnostics) (length diagnostics))
   ((hash-table-p diagnostics) (hash-table-count diagnostics))
   (diagnostics 1)
   (t 0)))

(defun soma-gym-bridge--lsp-diagnostic-evidence ()
  "Return the current lsp-mode diagnostic cache for the selected buffer."
  (let ((diagnostics
         (and (fboundp 'lsp--get-buffer-diagnostics)
              (ignore-errors (lsp--get-buffer-diagnostics)))))
    (soma-gym-bridge--alist
     (cons "count" (soma-gym-bridge--lsp-diagnostic-count diagnostics))
     (cons "raw" (if diagnostics (format "%S" diagnostics) "")))))

(defun soma-gym-bridge--lsp-diagnostic-cache-evidence (workspace)
  "Return lsp-mode diagnostic cache evidence for WORKSPACE and current buffer."
  (let* ((path (and buffer-file-name
                    (ignore-errors
                      (lsp--fix-path-casing buffer-file-name))))
         (workspace-diagnostics
          (and workspace
               (fboundp 'lsp--workspace-diagnostics)
               (ignore-errors
                 (lsp--workspace-diagnostics workspace))))
         (session-diagnostics
          (and (fboundp 'lsp-diagnostics)
               (ignore-errors (lsp-diagnostics t))))
         (workspace-current
          (and path
               (hash-table-p workspace-diagnostics)
               (gethash path workspace-diagnostics)))
         (session-current
          (and path
               (hash-table-p session-diagnostics)
               (gethash path session-diagnostics)))
         (stats
          (and path
               (fboundp 'lsp-diagnostics-stats-for)
               (ignore-errors (lsp-diagnostics-stats-for path))))
         (events
          (and (boundp 'soma-bridge-daemon--lsp-diagnostic-events)
               (symbol-value 'soma-bridge-daemon--lsp-diagnostic-events)))
         (notify-events
          (and (boundp 'soma-bridge-daemon--lsp-notify-events)
               (symbol-value 'soma-bridge-daemon--lsp-notify-events)))
         (incoming-events
          (and (boundp 'soma-bridge-daemon--lsp-incoming-events)
               (symbol-value 'soma-bridge-daemon--lsp-incoming-events))))
    (soma-gym-bridge--alist
     (cons "path" (or path ""))
     (cons "workspace_key_count"
           (if (hash-table-p workspace-diagnostics)
               (hash-table-count workspace-diagnostics)
             0))
     (cons "workspace_keys"
           (if (hash-table-p workspace-diagnostics)
               (soma-gym-bridge--keys workspace-diagnostics)
             nil))
     (cons "workspace_current_count"
           (soma-gym-bridge--lsp-diagnostic-count workspace-current))
     (cons "workspace_current_raw"
           (if workspace-current (format "%S" workspace-current) ""))
     (cons "session_key_count"
           (if (hash-table-p session-diagnostics)
               (hash-table-count session-diagnostics)
             0))
     (cons "session_keys"
           (if (hash-table-p session-diagnostics)
               (soma-gym-bridge--keys session-diagnostics)
             nil))
     (cons "session_current_count"
           (soma-gym-bridge--lsp-diagnostic-count session-current))
     (cons "stats_raw" (if stats (format "%S" stats) ""))
     (cons "publish_events"
           (if (listp events)
               (mapcar #'soma-gym-bridge--jsonish
                       (reverse (seq-take events 8)))
             nil))
     (cons "notify_events"
           (if (listp notify-events)
               (mapcar #'soma-gym-bridge--jsonish
                       (reverse (seq-take notify-events 12)))
             nil))
     (cons "incoming_events"
           (if (listp incoming-events)
               (mapcar #'soma-gym-bridge--jsonish
                       (reverse (seq-take incoming-events 12)))
             nil)))))

(defun soma-gym-bridge--lsp-pending-change-count ()
  "Return lsp-mode delayed change count visible from the current buffer."
  (if (and (boundp 'lsp--delayed-requests)
           (listp lsp--delayed-requests))
      (length lsp--delayed-requests)
    0))

(defun soma-gym-bridge--drain-events (seconds)
  "Allow timers and process filters to run for SECONDS."
  (let ((deadline (+ (float-time) seconds)))
    (while (< (float-time) deadline)
      (accept-process-output nil 0.05)
      (sit-for 0.01))))

(defun soma-gym-bridge--lsp-sync-before-observe (seconds)
  "Flush queued lsp-mode document changes and drain server responses."
  (let ((pending-before (soma-gym-bridge--lsp-pending-change-count))
        (flushed :json-false)
        (pull-requested :json-false)
        (pull-supported :json-false)
        flush-error)
    (when (fboundp 'lsp--flush-delayed-changes)
      (condition-case err
          (progn
            (lsp--flush-delayed-changes)
            (setq flushed t))
        (error
         (setq flush-error (error-message-string err)))))
    (when (and (fboundp 'lsp-diagnostics--request-pull-diagnostics)
               (fboundp 'lsp-feature?)
               (soma-gym-bridge--lsp-workspace))
      (setq pull-supported
            (soma-gym-bridge--bool
             (ignore-errors (lsp-feature? "textDocument/diagnostic"))))
      (when (eq pull-supported t)
        (condition-case err
            (progn
              (lsp-diagnostics--request-pull-diagnostics
               (soma-gym-bridge--lsp-workspace))
              (setq pull-requested t))
          (error
           (setq flush-error
                 (if flush-error
                     (concat flush-error "; pull diagnostics: "
                             (error-message-string err))
                   (concat "pull diagnostics: "
                           (error-message-string err))))))))
    (soma-gym-bridge--drain-events seconds)
    (soma-gym-bridge--alist
     (cons "pending_changes_before" pending-before)
     (cons "pending_changes_after"
           (soma-gym-bridge--lsp-pending-change-count))
     (cons "flush_attempted" (soma-gym-bridge--bool flushed))
     (cons "pull_diagnostics_supported" pull-supported)
     (cons "pull_diagnostics_requested" pull-requested)
     (cons "drain_seconds" seconds)
     (cons "flush_error" (or flush-error :json-false)))))

(defun soma-gym-bridge--named-buffer-tail (buffer-name limit)
  "Return at most LIMIT characters from BUFFER-NAME."
  (let ((buffer (and buffer-name (get-buffer buffer-name))))
    (if (buffer-live-p buffer)
        (with-current-buffer buffer
          (let* ((end (point-max))
                 (start (max (point-min) (- end limit))))
            (buffer-substring-no-properties start end)))
      "")))

(defun soma-gym-bridge--lsp-server-evidence (workspace)
  (let* ((proc (and workspace
                    (ignore-errors (lsp--workspace-cmd-proc workspace))))
         (wire-proc (and workspace
                         (ignore-errors (lsp--workspace-proc workspace))))
         (pid (and proc (ignore-errors (lsp-process-id proc))))
         (server-executable
          (if pid (soma-gym-bridge--proc-link pid "exe") ""))
         (server-rootfs-owned
          (soma-gym-bridge--rootfs-owned-tool-path-p server-executable))
         (process-name (and (processp proc) (process-name proc)))
         (stderr-buffer-name
          (and process-name (format "*%s::stderr*" process-name)))
         (stdout-buffer-name
          (and proc
               (processp proc)
               (buffer-live-p (process-buffer proc))
               (buffer-name (process-buffer proc)))))
    (soma-gym-bridge--alist
     (cons "server_id"
           (if workspace
               (soma-gym-bridge--json-symbol
                (ignore-errors
                  (lsp--client-server-id (lsp--workspace-client workspace))))
             ""))
     (cons "workspace_status"
           (soma-gym-bridge--lsp-workspace-status workspace))
     (cons "workspace_proc_type" (format "%S" (type-of wire-proc)))
     (cons "cmd_proc_type" (format "%S" (type-of proc)))
     (cons "cmd_proc_status"
           (if (and proc (processp proc))
               (soma-gym-bridge--json-symbol (process-status proc))
             ""))
     (cons "workspace_root" (soma-gym-bridge--lsp-workspace-root workspace))
     (cons "server_pid" (or pid :json-false))
     (cons "server_executable" server-executable)
     (cons "server_rootfs_owned"
           (soma-gym-bridge--bool server-rootfs-owned))
     (cons "server_cwd"
           (if pid (soma-gym-bridge--proc-link pid "cwd") ""))
     (cons "server_command"
           (if (and proc (processp proc))
               (process-command proc)
             nil))
     (cons "server_stdout_tail"
           (soma-gym-bridge--named-buffer-tail stdout-buffer-name 1200))
     (cons "server_stderr_tail"
           (soma-gym-bridge--named-buffer-tail stderr-buffer-name 1200)))))

(defun soma-gym-bridge--lsp-server-ownership-checks (workspace server)
  "Return checks proving WORKSPACE is backed by a rootfs-owned LSP server."
  (let ((executable (soma-gym-bridge--get server "server_executable"))
        (rootfs-owned (soma-gym-bridge--get server "server_rootfs_owned")))
    (list
     (soma-gym-bridge--check
      "LSP workspace is present"
      (and workspace t)
      (format "%S" workspace))
     (soma-gym-bridge--check
      "LSP server process is present"
      (soma-gym-bridge--present-p
       (soma-gym-bridge--get server "server_pid"))
      (format "%S" server))
     (soma-gym-bridge--check
      "LSP server executable is rootfs-owned"
      (eq rootfs-owned t)
      (format "executable=%S rootfs_owned=%S"
              executable rootfs-owned)))))

(defun soma-gym-bridge--lsp-ownership-error (checks)
  "Return a user-facing ownership error when CHECKS failed."
  (if (soma-gym-bridge--all-checks-pass-p checks)
      :json-false
    "LSP operation requires a live rootfs-owned LSP server"))

(defun soma-gym-bridge--lsp-symbol-at-point ()
  (let* ((rename-target (ignore-errors (lsp--get-symbol-to-rename)))
         (bounds (and (consp rename-target) (car rename-target)))
         (start (and (consp bounds) (car bounds)))
         (end (and (consp bounds) (cdr bounds))))
    (cond
     ((and (integerp start)
           (integerp end)
           (soma-gym-bridge--range-valid-p start end))
      (buffer-substring-no-properties start end))
     (t
      (or (thing-at-point 'symbol t) "")))))

(defun soma-gym-bridge--lsp-symbol-at-point-passive ()
  "Return the lexical symbol at point without sending an LSP request."
  (or (thing-at-point 'symbol t) ""))

(defun soma-gym-bridge-observe-lsp-source-passive (buffer max-text)
  "Return LSP source state for BUFFER without active server requests."
  (with-current-buffer buffer
    (let* ((workspace (soma-gym-bridge--lsp-workspace))
           (server (soma-gym-bridge--lsp-server-evidence workspace))
           (file-text
            (buffer-substring-no-properties
             (point-min)
             (min (point-max) (+ (point-min) max-text)))))
      (append
       (soma-gym-bridge--alist
        (cons "operation" "lsp.observe_source.passive")
        (cons "state" "succeeded")
        (cons "buffer_name" (buffer-name buffer))
        (cons "text_owner_name"
              (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
        (cons "buffer_file_name" (or buffer-file-name :json-false))
        (cons "major_mode" (symbol-name major-mode))
        (cons "chars_tick" (buffer-chars-modified-tick))
        (cons "modified" (soma-gym-bridge--bool (buffer-modified-p)))
        (cons "read_only" (soma-gym-bridge--bool buffer-read-only))
        (cons "point" (point))
        (cons "mark_active" (soma-gym-bridge--bool mark-active))
        (cons "narrowed" (soma-gym-bridge--bool (buffer-narrowed-p)))
        (cons "selected_window_buffer"
              (if (window-live-p (selected-window))
                  (buffer-name (window-buffer (selected-window)))
                :json-false))
        (cons "minibuffer_active"
              (soma-gym-bridge--bool (active-minibuffer-window)))
        (cons "workspace_present" (soma-gym-bridge--bool workspace))
        (cons "workspace_status"
              (soma-gym-bridge--lsp-workspace-status workspace))
        (cons "workspace_root"
              (soma-gym-bridge--lsp-workspace-root workspace))
        (cons "document_uri" (or (ignore-errors (lsp--buffer-uri)) ""))
        (cons "document_version"
              (if (boundp 'lsp--cur-version) lsp--cur-version :json-false))
        (cons "position_encoding"
              (soma-gym-bridge--lsp-position-encoding workspace))
        (cons "capabilities" (soma-gym-bridge--lsp-capabilities))
        (cons "diagnostics" (soma-gym-bridge--lsp-diagnostic-evidence))
        (cons "diagnostic_cache"
              (soma-gym-bridge--lsp-diagnostic-cache-evidence workspace))
        (cons "symbol" (soma-gym-bridge--lsp-symbol-at-point-passive))
        (cons "symbol_source" "thing-at-point")
        (cons "text_hash" (secure-hash 'sha256 file-text))
        (cons "text_excerpt" file-text)
        (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
        (cons "plan" (or (getenv "SOMA_INSULA_PLAN") "")))
       server))))

(defun soma-gym-bridge-observe-lsp-source (buffer max-text)
  "Return LSP workspace, server, document, and point-symbol state for BUFFER."
  (with-current-buffer buffer
    (let* ((workspace (soma-gym-bridge--lsp-workspace))
           (server (soma-gym-bridge--lsp-server-evidence workspace))
           (checks (soma-gym-bridge--lsp-server-ownership-checks
                    workspace server))
           (passed (soma-gym-bridge--all-checks-pass-p checks))
           (sync (if passed
                     (soma-gym-bridge--lsp-sync-before-observe 0.2)
                   (soma-gym-bridge--alist
                    (cons "skipped" t)
                    (cons "reason" "server_not_rootfs_owned"))))
           (file-text
            (buffer-substring-no-properties
             (point-min)
             (min (point-max) (+ (point-min) max-text)))))
      (append
       (soma-gym-bridge--alist
        (cons "operation" "lsp.observe_source")
        (cons "state" (if passed "succeeded" "failed"))
        (cons "state_sequence"
              (if passed
                  '("accepted" "running" "succeeded")
                '("accepted" "running" "failed")))
        (cons "buffer_name" (buffer-name buffer))
        (cons "text_owner_name"
              (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
        (cons "buffer_file_name" (or buffer-file-name :json-false))
        (cons "major_mode" (symbol-name major-mode))
        (cons "chars_tick" (buffer-chars-modified-tick))
        (cons "modified" (soma-gym-bridge--bool (buffer-modified-p)))
        (cons "read_only" (soma-gym-bridge--bool buffer-read-only))
        (cons "point" (point))
        (cons "mark_active" (soma-gym-bridge--bool mark-active))
        (cons "narrowed" (soma-gym-bridge--bool (buffer-narrowed-p)))
        (cons "selected_window_buffer"
              (if (window-live-p (selected-window))
                  (buffer-name (window-buffer (selected-window)))
                :json-false))
        (cons "minibuffer_active"
              (soma-gym-bridge--bool (active-minibuffer-window)))
        (cons "workspace_present" (soma-gym-bridge--bool workspace))
        (cons "workspace_status"
              (soma-gym-bridge--lsp-workspace-status workspace))
        (cons "workspace_root"
              (soma-gym-bridge--lsp-workspace-root workspace))
        (cons "document_uri" (or (ignore-errors (lsp--buffer-uri)) ""))
        (cons "document_version"
              (if (boundp 'lsp--cur-version) lsp--cur-version :json-false))
        (cons "position_encoding"
              (soma-gym-bridge--lsp-position-encoding workspace))
        (cons "lsp_sync" sync)
        (cons "precondition_checks" checks)
        (cons "capabilities" (if passed
                                  (soma-gym-bridge--lsp-capabilities)
                                nil))
        (cons "diagnostics" (if passed
                                 (soma-gym-bridge--lsp-diagnostic-evidence)
                               (soma-gym-bridge--alist
                                (cons "count" 0)
                                (cons "raw" ""))))
        (cons "diagnostic_cache"
              (if passed
                  (soma-gym-bridge--lsp-diagnostic-cache-evidence workspace)
                (soma-gym-bridge--empty-object)))
        (cons "symbol" (soma-gym-bridge--lsp-symbol-at-point-passive))
        (cons "symbol_source" "thing-at-point")
        (cons "text_hash" (secure-hash 'sha256 file-text))
        (cons "text_excerpt" file-text)
        (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
        (cons "plan" (or (getenv "SOMA_INSULA_PLAN") ""))
        (cons "error" (soma-gym-bridge--lsp-ownership-error checks)))
       server))))

(defun soma-gym-bridge--lsp-diagnostic-precondition-checks
    (buffer preconditions workspace)
  "Return LSP diagnostic owner and freshness checks for BUFFER."
  (let* ((actual-status (soma-gym-bridge--lsp-workspace-status workspace))
         (actual-root (soma-gym-bridge--lsp-workspace-root workspace))
         (actual-uri (or (ignore-errors (lsp--buffer-uri)) ""))
         (actual-version
          (if (boundp 'lsp--cur-version) lsp--cur-version :json-false))
         (actual-server-id
          (if workspace
              (soma-gym-bridge--json-symbol
               (ignore-errors
                 (lsp--client-server-id
                  (lsp--workspace-client workspace))))
            ""))
         (expected-root
          (soma-gym-bridge--get preconditions "workspace_root"))
         (expected-uri
          (soma-gym-bridge--get preconditions "document_uri"))
         (expected-version
          (soma-gym-bridge--get preconditions "document_version"))
         (expected-server-id
          (soma-gym-bridge--get preconditions "server_id")))
    (append
     (soma-gym-bridge--buffer-precondition-checks buffer preconditions)
     (list
      (soma-gym-bridge--check
       "LSP workspace is present"
       (and workspace t)
       (format "%S" workspace))
      (soma-gym-bridge--check
       "LSP workspace is initialized"
       (string= actual-status "initialized")
       actual-status))
     (when (soma-gym-bridge--has-key-p preconditions "workspace_root")
       (list
        (soma-gym-bridge--check
         "LSP workspace root matches precondition"
         (equal expected-root actual-root)
         (format "expected=%S actual=%S" expected-root actual-root))))
     (when (soma-gym-bridge--has-key-p preconditions "document_uri")
       (list
        (soma-gym-bridge--check
         "LSP document URI matches precondition"
         (equal expected-uri actual-uri)
         (format "expected=%S actual=%S" expected-uri actual-uri))))
     (when (soma-gym-bridge--has-key-p preconditions "document_version")
       (list
        (soma-gym-bridge--check
         "LSP document version matches precondition"
         (equal expected-version actual-version)
         (format "expected=%S actual=%S"
                 expected-version actual-version))))
     (when (soma-gym-bridge--has-key-p preconditions "server_id")
       (list
        (soma-gym-bridge--check
         "LSP server id matches precondition"
         (equal expected-server-id actual-server-id)
         (format "expected=%S actual=%S"
                 expected-server-id actual-server-id)))))))

(defun soma-gym-bridge-observe-lsp-diagnostics
    (buffer preconditions settle-seconds)
  "Return focused LSP diagnostic evidence for BUFFER."
  (with-current-buffer buffer
    (let* ((workspace (soma-gym-bridge--lsp-workspace))
           (server (soma-gym-bridge--lsp-server-evidence workspace))
           (checks
            (append
             (soma-gym-bridge--lsp-server-ownership-checks workspace server)
             (soma-gym-bridge--lsp-diagnostic-precondition-checks
              buffer preconditions workspace)))
           (passed (soma-gym-bridge--all-checks-pass-p checks))
           (sync (if passed
                     (soma-gym-bridge--lsp-sync-before-observe settle-seconds)
                   (soma-gym-bridge--alist
                    (cons "skipped" t)
                    (cons "reason" "server_not_rootfs_owned"))))
           (diagnostics (if passed
                            (soma-gym-bridge--lsp-diagnostic-evidence)
                          (soma-gym-bridge--alist
                           (cons "count" 0)
                           (cons "raw" ""))))
           (diagnostic-cache
            (if passed
                (soma-gym-bridge--lsp-diagnostic-cache-evidence workspace)
              (soma-gym-bridge--empty-object))))
      (append
       (soma-gym-bridge--alist
        (cons "operation" "lsp.observe_diagnostics")
        (cons "state" (if passed "succeeded" "failed"))
        (cons "buffer_name" (buffer-name buffer))
        (cons "text_owner_name"
              (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
        (cons "buffer_file_name" (or buffer-file-name :json-false))
        (cons "major_mode" (symbol-name major-mode))
        (cons "chars_tick" (buffer-chars-modified-tick))
        (cons "modified" (soma-gym-bridge--bool (buffer-modified-p)))
        (cons "read_only" (soma-gym-bridge--bool buffer-read-only))
        (cons "point" (point))
        (cons "mark_active" (soma-gym-bridge--bool mark-active))
        (cons "narrowed" (soma-gym-bridge--bool (buffer-narrowed-p)))
        (cons "selected_window_buffer"
              (if (window-live-p (selected-window))
                  (buffer-name (window-buffer (selected-window)))
                :json-false))
        (cons "minibuffer_active"
              (soma-gym-bridge--bool (active-minibuffer-window)))
        (cons "workspace_present" (soma-gym-bridge--bool workspace))
        (cons "workspace_status"
              (soma-gym-bridge--lsp-workspace-status workspace))
        (cons "workspace_root"
              (soma-gym-bridge--lsp-workspace-root workspace))
        (cons "document_uri" (or (ignore-errors (lsp--buffer-uri)) ""))
        (cons "document_version"
              (if (boundp 'lsp--cur-version) lsp--cur-version :json-false))
        (cons "position_encoding"
              (soma-gym-bridge--lsp-position-encoding workspace))
        (cons "lsp_sync" sync)
        (cons "precondition_checks" checks)
        (cons "diagnostics" diagnostics)
        (cons "diagnostic_cache" diagnostic-cache)
        (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
        (cons "plan" (or (getenv "SOMA_INSULA_PLAN") ""))
        (cons "error"
              (if passed
                  :json-false
                "LSP diagnostics require a live rootfs-owned LSP server and matching document preconditions")))
       server))))

(defun soma-gym-bridge--lsp-precondition-checks (buffer preconditions)
  (when (buffer-live-p buffer)
    (with-current-buffer buffer
      (let* ((workspace (soma-gym-bridge--lsp-workspace))
             (server (soma-gym-bridge--lsp-server-evidence workspace))
             (ownership-checks
              (soma-gym-bridge--lsp-server-ownership-checks workspace server))
             (ownership-passed
              (soma-gym-bridge--all-checks-pass-p ownership-checks))
             (actual-status (soma-gym-bridge--lsp-workspace-status workspace))
             (actual-root (soma-gym-bridge--lsp-workspace-root workspace))
             (actual-version
              (if (boundp 'lsp--cur-version) lsp--cur-version :json-false))
             (actual-symbol (and ownership-passed
                                 (soma-gym-bridge--lsp-symbol-at-point)))
             (expected-root
              (soma-gym-bridge--get preconditions "workspace_root"))
             (expected-version
              (soma-gym-bridge--get preconditions "document_version"))
             (expected-symbol
              (soma-gym-bridge--get preconditions "symbol")))
        (append
         ownership-checks
         (list
          (soma-gym-bridge--check
           "LSP workspace is initialized"
           (string= actual-status "initialized")
           actual-status)
          (soma-gym-bridge--check
           "LSP rename capability is advertised"
           (and ownership-passed
                (eq (soma-gym-bridge--lsp-method-supported-p
                     "textDocument/rename")
                    t))
           (if ownership-passed
               (format "%S" (soma-gym-bridge--lsp-capabilities))
             "skipped because LSP server is not rootfs-owned")))
         (when (soma-gym-bridge--has-key-p preconditions "workspace_root")
           (list
            (soma-gym-bridge--check
             "LSP workspace root matches precondition"
             (equal expected-root actual-root)
             (format "expected=%S actual=%S"
                     expected-root actual-root))))
         (when (soma-gym-bridge--has-key-p preconditions "document_version")
           (list
            (soma-gym-bridge--check
             "LSP document version matches precondition"
             (equal expected-version actual-version)
             (format "expected=%S actual=%S"
                     expected-version actual-version))))
         (when (soma-gym-bridge--has-key-p preconditions "symbol")
           (list
            (soma-gym-bridge--check
             "LSP symbol matches precondition"
             (equal expected-symbol actual-symbol)
             (format "expected=%S actual=%S"
                     expected-symbol actual-symbol)))))))))

(defun soma-gym-bridge--save-lsp-workspace-buffers (root)
  (let ((canonical-root
         (and (stringp root)
              (not (string-empty-p root))
              (file-name-as-directory (file-truename root))))
        saved)
    (dolist (buffer (buffer-list))
      (with-current-buffer buffer
        (when (and buffer-file-name
                   (buffer-modified-p)
                   canonical-root
                   (file-in-directory-p
                    (file-truename buffer-file-name)
                    canonical-root))
          (save-buffer)
          (push
           (soma-gym-bridge--alist
            (cons "buffer_name" (buffer-name buffer))
            (cons "file" (file-truename buffer-file-name))
            (cons "chars_tick" (buffer-chars-modified-tick)))
           saved))))
    (nreverse saved)))

(defun soma-gym-bridge-lsp-rename-symbol
    (buffer new-name preconditions timeout)
  "Rename the symbol at BUFFER point using real `lsp-rename'."
  (let ((checks (append
                 (soma-gym-bridge--buffer-precondition-checks
                  buffer preconditions)
                 (soma-gym-bridge--lsp-precondition-checks
                  buffer preconditions)))
        before after error-message requests workspace-edit-applied
        workspace-edit-raw workspace-edit-hash saved-buffers)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "lsp.rename_symbol")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "route" "precondition-check")
         (cons "precondition_checks" checks)
         (cons "error" "LSP rename precondition failed"))
      (setq before (soma-gym-bridge-observe-lsp-source buffer 1600))
      (with-current-buffer buffer
        (let ((orig-request (symbol-function 'lsp--send-request-async))
              (orig-apply (symbol-function 'lsp--apply-workspace-edit))
              (lsp-response-timeout timeout))
          (condition-case err
              (progn
                (when (fboundp 'lsp--flush-delayed-changes)
                  (ignore-errors (lsp--flush-delayed-changes)))
                (cl-letf (((symbol-function 'lsp--send-request-async)
                           (lambda (body callback &rest args)
                             (let ((returned
                                    (apply orig-request body callback args)))
                               (push
                                (soma-gym-bridge--alist
                                 (cons "method"
                                       (or (plist-get body :method) ""))
                                 (cons "id"
                                       (or (and (listp returned)
                                                (plist-get returned :id))
                                           :json-false))
                                 (cons "buffer" (buffer-name (current-buffer)))
                                 (cons "document_version"
                                       (if (boundp 'lsp--cur-version)
                                           lsp--cur-version
                                         :json-false)))
                                requests)
                               returned)))
                          ((symbol-function 'lsp--apply-workspace-edit)
                           (lambda (workspace-edit &optional operation)
                             (setq workspace-edit-raw
                                   (format "%S" workspace-edit))
                             (setq workspace-edit-hash
                                   (secure-hash 'sha256 workspace-edit-raw))
                             (prog1
                                 (funcall orig-apply workspace-edit operation)
                               (setq workspace-edit-applied t)))))
                  (lsp-rename new-name)
                  (setq saved-buffers
                        (soma-gym-bridge--save-lsp-workspace-buffers
                         (soma-gym-bridge--get before "workspace_root")))))
            ((quit error)
             (setq error-message (error-message-string err))))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-lsp-source buffer 1600)))
      (soma-gym-bridge--alist
       (cons "operation" "lsp.rename_symbol")
       (cons "state" (if error-message "failed" "succeeded"))
       (cons "state_sequence"
             (if error-message
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "route" "lsp-rename")
       (cons "new_name" new-name)
       (cons "wait_timeout_seconds" timeout)
       (cons "request_methods"
             (nreverse
              (delq nil
                    (mapcar
                     (lambda (request)
                       (soma-gym-bridge--get request "method"))
                     requests))))
       (cons "requests" (nreverse requests))
       (cons "workspace_edit_applied"
             (soma-gym-bridge--bool workspace-edit-applied))
       (cons "workspace_edit_sha256"
             (or workspace-edit-hash :json-false))
       (cons "workspace_edit_excerpt"
             (if workspace-edit-raw
                 (substring workspace-edit-raw
                            0
                            (min 2400 (length workspace-edit-raw)))
               :json-false))
       (cons "saved_buffers" (or saved-buffers nil))
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" (or after :json-false))
       (cons "error" (or error-message :json-false))))))

(defun soma-gym-bridge--submit-lsp-rename-symbol
    (request-id operation buffer new-name preconditions timeout)
  "Queue checked LSP rename and return a running response."
  (let* ((epoch soma-gym-bridge--transport-epoch)
         (running-result
          (soma-gym-bridge--alist
           (cons "operation" "lsp.rename_symbol")
           (cons "state" "running")
           (cons "route" "queued-lsp-rename")
           (cons "new_name" (or new-name :json-false))
           (cons "wait_timeout_seconds" timeout)))
         (running-response
          (soma-gym-bridge--response
           request-id operation "running"
           '("accepted" "running")
           running-result))
         action
         timer)
    (puthash request-id running-response soma-gym-bridge--requests)
    (setq action
          (lambda ()
            (when (= epoch soma-gym-bridge--transport-epoch)
              (let ((current (gethash request-id
                                      soma-gym-bridge--requests)))
                (when (and current
                           (string= (soma-gym-bridge--get current "state")
                                    "running"))
                  (puthash
                   request-id
                   (condition-case err
                       (let* ((result
                               (soma-gym-bridge-lsp-rename-symbol
                                buffer new-name preconditions timeout))
                              (state
                               (soma-gym-bridge--get result "state")))
                         (soma-gym-bridge--response
                          request-id operation
                          state
                          (soma-gym-bridge--get result "state_sequence")
                          result
                          (unless (string= state "succeeded")
                            "operation_failed")
                          (soma-gym-bridge--get result "error")))
                     ((quit error)
                      (soma-gym-bridge--response
                       request-id operation "failed"
                       '("accepted" "running" "failed")
                       running-result
                       "internal_error"
                       (error-message-string err))))
                   soma-gym-bridge--requests))))))
    (puthash request-id action soma-gym-bridge--pending-actions)
    (setq timer
          (run-at-time
           soma-gym-bridge-async-command-delay-seconds nil
           (lambda ()
             (soma-gym-bridge--run-pending-action request-id))))
    (push timer soma-gym-bridge--async-timers)
    running-response))

(defun soma-gym-bridge--json-read-file (path)
  (when (and path (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-gym-bridge--runtime-temp-file (prefix suffix)
  (let ((dir (or (getenv "XDG_RUNTIME_DIR")
                 (getenv "TMPDIR")
                 temporary-file-directory)))
    (make-directory dir t)
    (make-temp-file (expand-file-name prefix dir) nil suffix)))

(defun soma-gym-bridge--jupyter-current-client ()
  (and (boundp 'jupyter-current-client)
       jupyter-current-client))

(defun soma-gym-bridge--jupyter-client-session-id (client)
  (or (ignore-errors
        (jupyter-session-id (eieio-oref client 'session)))
      ""))

(defun soma-gym-bridge--jupyter-client-buffer-name (client)
  (let ((buffer (ignore-errors (eieio-oref client 'buffer))))
    (if (buffer-live-p buffer)
        (buffer-name buffer)
      :json-false)))

(defun soma-gym-bridge--jupyter-range-observation (start end max-text)
  (if (and start end (soma-gym-bridge--range-valid-p start end))
      (let ((text (buffer-substring-no-properties start end)))
        (soma-gym-bridge--alist
         (cons "start" start)
         (cons "end" end)
         (cons "text_hash" (secure-hash 'sha256 text))
         (cons "text_excerpt" (substring text 0 (min max-text (length text))))))
    :json-false))

(defun soma-gym-bridge-observe-jupyter-source
    (buffer start end max-text)
  "Return Jupyter source-buffer, client, and selected-range state for BUFFER."
  (with-current-buffer buffer
    (let* ((client (soma-gym-bridge--jupyter-current-client))
           (client-buffer (soma-gym-bridge--jupyter-client-buffer-name client))
           (connection-file soma-gym-bridge-jupyter-connection-file)
           (kernel-exe soma-gym-bridge-jupyter-kernel-exe))
      (soma-gym-bridge--alist
       (cons "operation" "jupyter.observe_source")
       (cons "state" "succeeded")
       (cons "buffer_name" (buffer-name buffer))
       (cons "text_owner_name"
             (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
       (cons "buffer_file_name" (or buffer-file-name :json-false))
       (cons "major_mode" (symbol-name major-mode))
       (cons "chars_tick" (buffer-chars-modified-tick))
       (cons "modified" (soma-gym-bridge--bool (buffer-modified-p buffer)))
       (cons "read_only" (soma-gym-bridge--bool buffer-read-only))
       (cons "point" (point))
       (cons "mark_active" (soma-gym-bridge--bool mark-active))
       (cons "narrowed" (soma-gym-bridge--bool (buffer-narrowed-p)))
       (cons "selected_window_buffer"
             (if (window-live-p (selected-window))
                 (buffer-name (window-buffer (selected-window)))
               :json-false))
       (cons "minibuffer_active"
             (soma-gym-bridge--bool (active-minibuffer-window)))
       (cons "jupyter_client_associated"
             (soma-gym-bridge--bool client))
       (cons "jupyter_client_class"
             (if client
                 (symbol-name (eieio-object-class client))
               :json-false))
       (cons "jupyter_client_session_id"
             (if client
                 (soma-gym-bridge--jupyter-client-session-id client)
               :json-false))
       (cons "jupyter_repl_buffer" client-buffer)
       (cons "jupyter_repl_interaction_mode"
             (soma-gym-bridge--bool
              (and (boundp 'jupyter-repl-interaction-mode)
                   jupyter-repl-interaction-mode)))
       (cons "jupyter_connection_file"
             (or connection-file :json-false))
       (cons "jupyter_connection_file_rootfs_owned"
             (soma-gym-bridge--bool
              (soma-gym-bridge--rootfs-owned-runtime-path-p
               connection-file)))
       (cons "jupyter_kernel_token"
             (or soma-gym-bridge-jupyter-kernel-token :json-false))
       (cons "jupyter_kernel_pid"
             (or soma-gym-bridge-jupyter-kernel-pid :json-false))
       (cons "jupyter_kernel_exe"
             (or kernel-exe :json-false))
       (cons "jupyter_kernel_exe_rootfs_owned"
             (soma-gym-bridge--bool
              (soma-gym-bridge--rootfs-owned-tool-path-p kernel-exe)))
       (cons "selected_range"
             (soma-gym-bridge--jupyter-range-observation
              start end max-text))
       (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
       (cons "plan" (or (getenv "SOMA_INSULA_PLAN") ""))))))

(defun soma-gym-bridge--jupyter-repl-client-p (client)
  (and client
       (ignore-errors
         (object-of-class-p client 'jupyter-repl-client))))

(defun soma-gym-bridge--jupyter-repl-buffer-client (repl-buffer)
  "Return the existing Jupyter client owned by REPL-BUFFER, if any."
  (and (buffer-live-p repl-buffer)
       (with-current-buffer repl-buffer
         (soma-gym-bridge--jupyter-current-client))))

(defun soma-gym-bridge--jupyter-repl-buffer-connected-p (repl-buffer)
  "Return non-nil when REPL-BUFFER's current client is connected."
  (and (buffer-live-p repl-buffer)
       (with-current-buffer repl-buffer
         (and (fboundp 'jupyter-repl-connected-p)
              (jupyter-repl-connected-p)))))

(defun soma-gym-bridge--jupyter-source-client (buffer)
  "Return BUFFER's currently associated Jupyter client, if any."
  (and (buffer-live-p buffer)
       (with-current-buffer buffer
         (soma-gym-bridge--jupyter-current-client))))

(defun soma-gym-bridge--jupyter-rootfs-checks (buffer preconditions)
  "Return rootfs ownership checks for BUFFER's Jupyter runtime metadata."
  (let ((expected-connection-file
         (soma-gym-bridge--get preconditions "jupyter_connection_file"))
        (expected-kernel-exe
         (soma-gym-bridge--get preconditions "jupyter_kernel_exe"))
        (expected-connection-rootfs-owned
         (soma-gym-bridge--get
          preconditions "jupyter_connection_file_rootfs_owned"))
        (expected-kernel-rootfs-owned
         (soma-gym-bridge--get
          preconditions "jupyter_kernel_exe_rootfs_owned"))
        connection-file kernel-exe connection-rootfs-owned
        kernel-rootfs-owned)
    (when (buffer-live-p buffer)
      (with-current-buffer buffer
        (setq connection-file soma-gym-bridge-jupyter-connection-file)
        (setq kernel-exe soma-gym-bridge-jupyter-kernel-exe)))
    (setq connection-rootfs-owned
          (soma-gym-bridge--rootfs-owned-runtime-path-p connection-file))
    (setq kernel-rootfs-owned
          (soma-gym-bridge--rootfs-owned-tool-path-p kernel-exe))
    (list
     (soma-gym-bridge--check
      "Jupyter connection file is rootfs-owned"
      (and connection-rootfs-owned
           (or (not (soma-gym-bridge--has-key-p
                     preconditions "jupyter_connection_file"))
               (equal expected-connection-file connection-file))
           (or (not (soma-gym-bridge--has-key-p
                     preconditions
                     "jupyter_connection_file_rootfs_owned"))
               (eq expected-connection-rootfs-owned t)))
      (soma-gym-bridge--alist
       (cons "expected_path" (or expected-connection-file :json-false))
       (cons "actual_path" (or connection-file :json-false))
       (cons "expected_rootfs_owned"
             (if (soma-gym-bridge--has-key-p
                  preconditions "jupyter_connection_file_rootfs_owned")
                 expected-connection-rootfs-owned
               :json-false))
       (cons "actual_rootfs_owned"
             (soma-gym-bridge--bool connection-rootfs-owned))))
     (soma-gym-bridge--check
      "Jupyter kernel executable is rootfs-owned"
      (and kernel-rootfs-owned
           (or (not (soma-gym-bridge--has-key-p
                     preconditions "jupyter_kernel_exe"))
               (equal expected-kernel-exe kernel-exe))
           (or (not (soma-gym-bridge--has-key-p
                     preconditions "jupyter_kernel_exe_rootfs_owned"))
               (eq expected-kernel-rootfs-owned t)))
      (soma-gym-bridge--alist
       (cons "expected_path" (or expected-kernel-exe :json-false))
       (cons "actual_path" (or kernel-exe :json-false))
       (cons "expected_rootfs_owned"
             (if (soma-gym-bridge--has-key-p
                  preconditions "jupyter_kernel_exe_rootfs_owned")
                 expected-kernel-rootfs-owned
               :json-false))
       (cons "actual_rootfs_owned"
             (soma-gym-bridge--bool kernel-rootfs-owned)))))))

(defun soma-gym-bridge--jupyter-attach-checks
    (buffer client-name preconditions repl-buffer repl-client)
  "Return precondition checks for attaching BUFFER to REPL-CLIENT."
  (let* ((expected-kernel-id
          (soma-gym-bridge--get preconditions "kernel_id"))
         (expected-kernel-session
          (soma-gym-bridge--get preconditions "kernel_session"))
         (expected-connection-file
          (soma-gym-bridge--get preconditions "jupyter_connection_file"))
         (expected-token
          (soma-gym-bridge--get preconditions "jupyter_kernel_token"))
         (expected-session
          (soma-gym-bridge--get preconditions "jupyter_client_session_id"))
         (expected-repl
          (soma-gym-bridge--get preconditions "jupyter_repl_buffer"))
         (source-client (soma-gym-bridge--jupyter-source-client buffer))
         (client-session
          (and repl-client
               (soma-gym-bridge--jupyter-client-session-id repl-client)))
         (client-buffer-name
          (and repl-client
               (soma-gym-bridge--jupyter-client-buffer-name repl-client)))
         source-token source-connection source-mode client-mode)
    (when (buffer-live-p buffer)
      (with-current-buffer buffer
        (setq source-token soma-gym-bridge-jupyter-kernel-token)
        (setq source-connection soma-gym-bridge-jupyter-connection-file)
        (setq source-mode major-mode)))
    (setq client-mode
          (and repl-client
               (ignore-errors
                 (jupyter-kernel-language-mode repl-client))))
    (append
     (list
      (soma-gym-bridge--check
       "target source buffer is live"
       (buffer-live-p buffer)
       (format "%S" buffer))
      (soma-gym-bridge--check
       "Jupyter client parameter names a live REPL buffer"
       (and (stringp client-name)
            (buffer-live-p repl-buffer))
       (or client-name :json-false))
      (soma-gym-bridge--check
       "Jupyter REPL buffer owns a REPL client"
       (soma-gym-bridge--jupyter-repl-client-p repl-client)
       (format "%S" repl-client))
      (soma-gym-bridge--check
       "Jupyter REPL client is connected"
       (soma-gym-bridge--jupyter-repl-buffer-connected-p repl-buffer)
       (if (buffer-live-p repl-buffer)
           (with-current-buffer repl-buffer
             (soma-gym-bridge--bool
              (and (fboundp 'jupyter-repl-connected-p)
                   (jupyter-repl-connected-p))))
         :json-false))
      (soma-gym-bridge--check
       "source buffer has no different Jupyter client"
       (or (not source-client)
           (eq source-client repl-client))
       (soma-gym-bridge--alist
        (cons "source_client"
              (if source-client
                  (soma-gym-bridge--jupyter-client-buffer-name
                   source-client)
                :json-false))
        (cons "requested_client" (or client-buffer-name :json-false))))
      (soma-gym-bridge--check
       "source mode matches Jupyter client language mode"
       (and source-mode client-mode (eq source-mode client-mode))
       (soma-gym-bridge--alist
        (cons "source_mode"
              (if source-mode
                  (soma-gym-bridge--json-symbol source-mode)
                :json-false))
        (cons "client_language_mode"
              (if client-mode
                  (soma-gym-bridge--json-symbol client-mode)
                :json-false))))
      (soma-gym-bridge--check
       "Jupyter kernel id matches source token"
       (and (stringp expected-kernel-id)
            (stringp source-token)
            (string= expected-kernel-id source-token))
       (soma-gym-bridge--alist
        (cons "expected" (or expected-kernel-id :json-false))
        (cons "actual" (or source-token :json-false))))
      (soma-gym-bridge--check
       "Jupyter kernel session matches client session"
       (and (stringp expected-kernel-session)
            (stringp client-session)
            (string= expected-kernel-session client-session))
       (soma-gym-bridge--alist
        (cons "expected" (or expected-kernel-session :json-false))
        (cons "actual" (or client-session :json-false)))))
     (soma-gym-bridge--jupyter-rootfs-checks buffer preconditions)
     (when (soma-gym-bridge--has-key-p
            preconditions "jupyter_connection_file")
       (list
        (soma-gym-bridge--check
         "Jupyter connection file matches source metadata"
         (and (stringp expected-connection-file)
              (stringp source-connection)
              (string= expected-connection-file source-connection))
         (soma-gym-bridge--alist
          (cons "expected" (or expected-connection-file :json-false))
          (cons "actual" (or source-connection :json-false))))))
     (when (soma-gym-bridge--has-key-p
            preconditions "jupyter_kernel_token")
       (list
        (soma-gym-bridge--check
         "Jupyter kernel token matches source metadata"
         (and (stringp expected-token)
              (stringp source-token)
              (string= expected-token source-token))
         (soma-gym-bridge--alist
          (cons "expected" (or expected-token :json-false))
          (cons "actual" (or source-token :json-false))))))
     (when (soma-gym-bridge--has-key-p
            preconditions "jupyter_client_session_id")
       (list
        (soma-gym-bridge--check
         "Jupyter client session id matches precondition"
         (and (stringp expected-session)
              (stringp client-session)
              (string= expected-session client-session))
         (soma-gym-bridge--alist
          (cons "expected" (or expected-session :json-false))
          (cons "actual" (or client-session :json-false))))))
     (when (soma-gym-bridge--has-key-p
            preconditions "jupyter_repl_buffer")
       (list
        (soma-gym-bridge--check
         "Jupyter REPL buffer matches precondition"
         (and (stringp expected-repl)
              (stringp client-buffer-name)
              (string= expected-repl client-buffer-name)
              (stringp client-name)
              (string= expected-repl client-name))
         (soma-gym-bridge--alist
          (cons "expected" (or expected-repl :json-false))
          (cons "param" (or client-name :json-false))
          (cons "actual" (or client-buffer-name :json-false)))))))))

(defun soma-gym-bridge-jupyter-attach-existing
    (buffer client-name preconditions)
  "Attach BUFFER to an existing Jupyter REPL CLIENT-NAME after checks."
  (let* ((repl-buffer (and (stringp client-name)
                           (get-buffer client-name)))
         (repl-client
          (soma-gym-bridge--jupyter-repl-buffer-client repl-buffer))
         (checks
          (soma-gym-bridge--jupyter-attach-checks
           buffer client-name preconditions repl-buffer repl-client))
         (before
          (and (buffer-live-p buffer)
               (soma-gym-bridge-observe-jupyter-source
                buffer nil nil 800)))
         after error-message postcondition-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "jupyter.attach_existing")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "running" "failed"))
         (cons "route" "precondition-check")
         (cons "client" (or client-name :json-false))
         (cons "precondition_checks" checks)
         (cons "before" (or before :json-false))
         (cons "after" (or before :json-false))
         (cons "jupyter_kernel_token"
               (and before
                    (soma-gym-bridge--get before "jupyter_kernel_token")))
         (cons "jupyter_client_session_id"
               (or (and repl-client
                        (soma-gym-bridge--jupyter-client-session-id
                         repl-client))
                   :json-false))
         (cons "jupyter_repl_buffer"
               (or (and repl-client
                        (soma-gym-bridge--jupyter-client-buffer-name
                         repl-client))
                   :json-false))
         (cons "error" "Jupyter attach precondition failed"))
      (condition-case err
          (with-current-buffer buffer
            (jupyter-repl-associate-buffer client-name))
        ((quit error)
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-jupyter-source
                  buffer nil nil 800)))
      (when (and (not error-message)
                 (not (and after
                           (eq (soma-gym-bridge--get
                                after "jupyter_client_associated")
                               t)
                           (string= (soma-gym-bridge--get
                                     after "jupyter_repl_buffer")
                                    client-name))))
        (setq postcondition-message
              "Jupyter attach did not produce the requested association"))
      (soma-gym-bridge--alist
       (cons "operation" "jupyter.attach_existing")
       (cons "state"
             (if (or error-message postcondition-message)
                 "failed"
               "succeeded"))
       (cons "state_sequence"
             (if (or error-message postcondition-message)
                 '("accepted" "running" "failed")
               '("accepted" "running" "succeeded")))
       (cons "route" "jupyter-repl-associate-buffer")
       (cons "client" (or client-name :json-false))
       (cons "precondition_checks" checks)
       (cons "before" (or before :json-false))
       (cons "after" (or after :json-false))
       (cons "jupyter_kernel_token"
             (or (and after
                      (soma-gym-bridge--get after "jupyter_kernel_token"))
                 :json-false))
       (cons "jupyter_connection_file"
             (or (and after
                      (soma-gym-bridge--get after "jupyter_connection_file"))
                 :json-false))
       (cons "jupyter_client_session_id"
             (or (and after
                      (soma-gym-bridge--get
                       after "jupyter_client_session_id"))
                 :json-false))
       (cons "jupyter_repl_buffer"
             (or (and after
                      (soma-gym-bridge--get after "jupyter_repl_buffer"))
                 :json-false))
       (cons "client_association"
             (soma-gym-bridge--alist
              (cons "source_buffer"
                    (if (buffer-live-p buffer)
                        (buffer-name buffer)
                      :json-false))
              (cons "associated"
                    (soma-gym-bridge--bool
                     (and after
                          (eq (soma-gym-bridge--get
                               after "jupyter_client_associated")
                              t))))
              (cons "requested_repl_buffer"
                    (or client-name :json-false))))
       (cons "error" (or error-message
                         postcondition-message
                         :json-false))))))

(defun soma-gym-bridge--jupyter-display-ranges (display-id max-text)
  "Return visible ranges tagged with Jupyter display IDs in current buffer."
  (let ((pos (point-min))
        (ranges nil))
    (while (< pos (point-max))
      (let* ((value (get-text-property pos 'jupyter-display))
             (begin (get-text-property pos 'jupyter-display-begin))
             (next (or (next-single-property-change
                        pos 'jupyter-display nil (point-max))
                       (point-max))))
        (when value
          (push
           (soma-gym-bridge--alist
            (cons "start" pos)
            (cons "end" next)
            (cons "display_id" (soma-gym-bridge--json-safe-value value))
            (cons "display_id_matches"
                  (soma-gym-bridge--bool (equal value display-id)))
            (cons "display_begin"
                  (soma-gym-bridge--bool begin))
            (cons "text_excerpt"
                  (let ((text (buffer-substring-no-properties pos next)))
                    (substring text 0 (min max-text (length text)))))
            (cons "text_hash"
                  (secure-hash
                   'sha256
                   (buffer-substring-no-properties pos next))))
           ranges))
        (setq pos next)))
    (nreverse ranges)))

(defun soma-gym-bridge-observe-jupyter-display
    (buffer display-id max-text)
  "Return Jupyter display-buffer text and display-id property evidence."
  (with-current-buffer buffer
    (let* ((text (buffer-substring-no-properties (point-min) (point-max)))
           (ranges (soma-gym-bridge--jupyter-display-ranges
                    display-id max-text))
           (matching
            (cl-remove-if-not
             (lambda (range)
               (eq (soma-gym-bridge--get range "display_id_matches") t))
             ranges)))
      (soma-gym-bridge--alist
       (cons "operation" "jupyter.observe_display")
       (cons "state" "succeeded")
       (cons "buffer_name" (buffer-name buffer))
       (cons "text_owner_name"
             (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
       (cons "buffer_file_name" (or buffer-file-name :json-false))
       (cons "major_mode" (symbol-name major-mode))
       (cons "chars_tick" (buffer-chars-modified-tick))
       (cons "modified" (soma-gym-bridge--bool (buffer-modified-p buffer)))
       (cons "read_only" (soma-gym-bridge--bool buffer-read-only))
       (cons "point_min" (point-min))
       (cons "point_max" (point-max))
       (cons "point" (point))
       (cons "display_id" (or display-id :json-false))
       (cons "display_id_found" (soma-gym-bridge--bool matching))
       (cons "display_ranges" ranges)
       (cons "display_buffer_marker"
             (if (and (boundp 'jupyter-display-buffer-marker)
                      (markerp jupyter-display-buffer-marker))
                 (marker-position jupyter-display-buffer-marker)
               :json-false))
       (cons "jupyter_display_ids_count"
             (if (and (boundp 'jupyter-display-ids)
                      (hash-table-p jupyter-display-ids))
                 (hash-table-count jupyter-display-ids)
               :json-false))
       (cons "text_excerpt"
             (substring text 0 (min max-text (length text))))
       (cons "text_hash" (secure-hash 'sha256 text))
       (cons "selected_window_buffer"
             (if (window-live-p (selected-window))
                 (buffer-name (window-buffer (selected-window)))
               :json-false))
       (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
       (cons "plan" (or (getenv "SOMA_INSULA_PLAN") ""))))))

(defun soma-gym-bridge--jupyter-precondition-checks
    (buffer preconditions)
  (when (buffer-live-p buffer)
    (with-current-buffer buffer
      (let* ((client (soma-gym-bridge--jupyter-current-client))
             (expected-connection-file
              (soma-gym-bridge--get preconditions "jupyter_connection_file"))
             (expected-token
              (soma-gym-bridge--get preconditions "jupyter_kernel_token"))
             (expected-session
              (soma-gym-bridge--get preconditions "jupyter_client_session_id"))
             (expected-repl
              (soma-gym-bridge--get preconditions "jupyter_repl_buffer"))
             (actual-session
              (and client
                   (soma-gym-bridge--jupyter-client-session-id client)))
             (actual-repl
              (and client
                   (soma-gym-bridge--jupyter-client-buffer-name client))))
        (append
         (list
          (soma-gym-bridge--check
           "Jupyter source buffer is associated with a client"
           (and client t)
           (format "%S" client)))
         (when (soma-gym-bridge--has-key-p
                preconditions "jupyter_connection_file")
           (list
            (soma-gym-bridge--check
             "Jupyter connection file matches precondition"
             (equal expected-connection-file
                    (or soma-gym-bridge-jupyter-connection-file
                        :json-false))
             (format "expected=%S actual=%S"
                     expected-connection-file
                     (or soma-gym-bridge-jupyter-connection-file
                         :json-false)))))
         (when (soma-gym-bridge--has-key-p
                preconditions "jupyter_kernel_token")
           (list
            (soma-gym-bridge--check
             "Jupyter kernel token matches precondition"
             (equal expected-token
                    (or soma-gym-bridge-jupyter-kernel-token
                        :json-false))
             (format "expected=%S actual=%S"
                     expected-token
                     (or soma-gym-bridge-jupyter-kernel-token
                         :json-false)))))
         (when (soma-gym-bridge--has-key-p
                preconditions "jupyter_client_session_id")
           (list
            (soma-gym-bridge--check
             "Jupyter client session matches precondition"
             (equal expected-session actual-session)
             (format "expected=%S actual=%S"
                     expected-session actual-session))))
         (when (soma-gym-bridge--has-key-p
                preconditions "jupyter_repl_buffer")
           (list
            (soma-gym-bridge--check
             "Jupyter REPL buffer matches precondition"
             (equal expected-repl actual-repl)
             (format "expected=%S actual=%S"
                     expected-repl actual-repl))))
         (soma-gym-bridge--jupyter-rootfs-checks
          buffer preconditions))))))

(defun soma-gym-bridge--jupyter-request-evidence (req)
  (let ((messages (and req (jupyter-request-messages req)))
        message-types
        parent-ids
        stream-texts
        display-events
        display-update-events
        reply-status
        reply-parent)
    (cl-loop for msg in messages
             for index from 0 do
      (let* ((type (jupyter-message-type msg))
             (content (jupyter-message-content msg))
             (parent-id (or (jupyter-message-id
                             (jupyter-message-parent-header msg))
                            "")))
        (push type message-types)
        (unless (string-empty-p parent-id)
          (push parent-id parent-ids))
        (when (string= type "stream")
          (push (or (plist-get content :text) "") stream-texts))
        (when (member type '("display_data" "update_display_data"))
          (let* ((data (soma-gym-bridge--value content "data"))
                 (transient (soma-gym-bridge--value content "transient"))
                 (event
                  (soma-gym-bridge--alist
                   (cons "index" index)
                   (cons "message_type" type)
                   (cons "parent_id" parent-id)
                   (cons "display_id"
                         (or (soma-gym-bridge--value transient "display_id")
                             ""))
                   (cons "data_keys" (or (soma-gym-bridge--keys data) nil))
                   (cons "application_json"
                         (or (soma-gym-bridge--jsonish
                              (soma-gym-bridge--value
                               data "application/json"))
                             :json-false))
                   (cons "text_plain"
                         (or (soma-gym-bridge--value data "text/plain")
                             "")))))
            (if (string= type "display_data")
                (push event display-events)
              (push event display-update-events))))
        (when (string= type "execute_reply")
          (setq reply-status (or (plist-get content :status) ""))
          (setq reply-parent parent-id))))
    (soma-gym-bridge--alist
     (cons "request_id" (if req (jupyter-request-id req) ""))
     (cons "inhibited_handlers"
           (if req
               (soma-gym-bridge--json-safe-value
                (jupyter-request-inhibited-handlers req))
             :json-false))
     (cons "request_idle_flag"
           (soma-gym-bridge--bool (and req (jupyter-request-idle-p req))))
     (cons "message_types" (nreverse message-types))
     (cons "parent_ids" (nreverse parent-ids))
     (cons "stream_texts" (nreverse stream-texts))
     (cons "display_events" (nreverse display-events))
     (cons "display_update_events" (nreverse display-update-events))
     (cons "reply_status" (or reply-status ""))
     (cons "reply_parent" (or reply-parent "")))))

(defun soma-gym-bridge--jupyter-request-completed-p
    (wait-returned-idle request-evidence)
  "Return non-nil when REQUEST-EVIDENCE proves the Jupyter request completed."
  (or wait-returned-idle
      (and (eq (soma-gym-bridge--get request-evidence
                                      "request_idle_flag")
               t)
           (string= (or (soma-gym-bridge--get request-evidence
                                               "reply_status")
                        "")
                    "ok"))))

(defun soma-gym-bridge-jupyter-eval-region
    (buffer start end preconditions timeout)
  "Submit a selected Jupyter region from BUFFER after checked preconditions."
  (let ((checks (append
                 (soma-gym-bridge--buffer-precondition-checks
                  buffer preconditions start end)
                 (soma-gym-bridge--jupyter-precondition-checks
                  buffer preconditions)))
        before after submitted req idle request-evidence completed
        error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "jupyter.eval_region")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "route" "precondition-check")
         (cons "precondition_checks" checks)
         (cons "error" "Jupyter eval precondition failed"))
      (setq before
            (soma-gym-bridge-observe-jupyter-source
             buffer start end 800))
      (with-current-buffer buffer
        (let ((orig (symbol-function 'jupyter-eval-string)))
          (condition-case err
              (cl-letf (((symbol-function 'jupyter-eval-string)
                         (lambda (str &optional insert rb re)
                           (setq submitted str)
                           (funcall orig str insert rb re))))
                (setq req (jupyter-eval-region nil start end)))
            ((quit error)
             (setq error-message (error-message-string err))))))
      (when req
        (setq idle (jupyter-wait-until-idle req timeout)))
      (setq request-evidence
            (soma-gym-bridge--jupyter-request-evidence req))
      (setq completed
            (soma-gym-bridge--jupyter-request-completed-p
             idle request-evidence))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-jupyter-source
                  buffer start end 800)))
      (append
       (soma-gym-bridge--alist
        (cons "operation" "jupyter.eval_region")
        (cons "state"
              (cond
               (error-message "failed")
               (completed "succeeded")
               (t "outcome_unknown")))
        (cons "state_sequence"
              (cond
               (error-message '("accepted" "running" "failed"))
               (completed '("accepted" "running" "succeeded"))
               (t '("accepted" "running" "outcome_unknown"))))
        (cons "route" "jupyter-eval-region")
        (cons "start" start)
        (cons "end" end)
        (cons "wait_timeout_seconds" timeout)
        (cons "idle" (soma-gym-bridge--bool completed))
        (cons "submitted_code" (or submitted ""))
        (cons "submitted_hash"
              (if submitted
                  (secure-hash 'sha256 submitted)
                :json-false))
        (cons "precondition_checks" checks)
        (cons "before" before)
        (cons "after" (or after :json-false))
        (cons "error" (or error-message :json-false)))
       request-evidence))))

(defun soma-gym-bridge-jupyter-eval-region-via-repl
    (buffer start end preconditions timeout)
  "Submit BUFFER's selected Jupyter region through its associated REPL.

This exercises the `jupyter-repl-client' execution path, including the
client-owned display/update handlers that maintain display IDs in the REPL
buffer."
  (let ((checks (append
                 (soma-gym-bridge--buffer-precondition-checks
                  buffer preconditions start end)
                 (soma-gym-bridge--jupyter-precondition-checks
                  buffer preconditions)))
        before after client repl-buffer selected submitted req idle
        request-evidence completed error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "jupyter.eval_region_via_repl")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "route" "precondition-check")
         (cons "precondition_checks" checks)
         (cons "error" "Jupyter eval precondition failed"))
      (setq before
            (soma-gym-bridge-observe-jupyter-source
             buffer start end 800))
      (with-current-buffer buffer
        (setq client (soma-gym-bridge--jupyter-current-client))
        (setq selected (buffer-substring-no-properties start end))
        (setq submitted (string-trim selected "[\r\n]+" "[\r\n]+"))
        (setq repl-buffer (soma-gym-bridge--jupyter-client-buffer-name client)))
      (condition-case err
          (if (not client)
              (setq error-message
                    "source buffer has no associated Jupyter client")
            (with-current-buffer (eieio-oref client 'buffer)
              (let ((inhibit-read-only t))
                (goto-char (point-max))
                (jupyter-repl-replace-cell-code selected)
                (setq req (jupyter-repl-execute-cell client)))))
        ((quit error)
         (setq error-message (error-message-string err))))
      (when req
        (setq idle (jupyter-wait-until-idle req timeout)))
      (setq request-evidence
            (soma-gym-bridge--jupyter-request-evidence req))
      (setq completed
            (soma-gym-bridge--jupyter-request-completed-p
             idle request-evidence))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-jupyter-source
                  buffer start end 800)))
      (append
       (soma-gym-bridge--alist
        (cons "operation" "jupyter.eval_region_via_repl")
        (cons "state"
              (cond
               (error-message "failed")
               (completed "succeeded")
               (t "outcome_unknown")))
        (cons "state_sequence"
              (cond
               (error-message '("accepted" "running" "failed"))
               (completed '("accepted" "running" "succeeded"))
               (t '("accepted" "running" "outcome_unknown"))))
        (cons "route" "jupyter-repl-execute-cell")
        (cons "start" start)
        (cons "end" end)
        (cons "wait_timeout_seconds" timeout)
        (cons "idle" (soma-gym-bridge--bool completed))
        (cons "selected_code" (or selected ""))
        (cons "selected_hash"
              (if selected
                  (secure-hash 'sha256 selected)
                :json-false))
        (cons "submitted_code" (or submitted ""))
        (cons "submitted_hash"
              (if submitted
                  (secure-hash 'sha256 submitted)
                :json-false))
        (cons "jupyter_repl_buffer" (or repl-buffer :json-false))
        (cons "precondition_checks" checks)
        (cons "before" before)
        (cons "after" (or after :json-false))
        (cons "error" (or error-message :json-false)))
       request-evidence))))

(defun soma-gym-bridge--submit-jupyter-eval-region-via-repl
    (request-id operation buffer start end preconditions timeout)
  "Queue checked Jupyter region evaluation through the associated REPL."
  (let* ((epoch soma-gym-bridge--transport-epoch)
         (running-result
          (soma-gym-bridge--alist
           (cons "operation" "jupyter.eval_region_via_repl")
           (cons "state" "running")
           (cons "start" (or start :json-false))
           (cons "end" (or end :json-false))
           (cons "wait_timeout_seconds" timeout)))
         (running-response
          (soma-gym-bridge--response
           request-id operation "running"
           '("accepted" "running")
           running-result))
         timer)
    (puthash request-id running-response soma-gym-bridge--requests)
    (setq timer
          (run-at-time
           soma-gym-bridge-async-command-delay-seconds nil
           (lambda ()
             (when (= epoch soma-gym-bridge--transport-epoch)
               (let ((current (gethash request-id
                                       soma-gym-bridge--requests)))
                 (when (and current
                            (string= (soma-gym-bridge--get current "state")
                                     "running"))
                   (let ((result
                          (condition-case err
                              (soma-gym-bridge-jupyter-eval-region-via-repl
                               buffer start end preconditions timeout)
                            ((quit error)
                             (soma-gym-bridge--alist
                              (cons "operation"
                                    "jupyter.eval_region_via_repl")
                              (cons "state" "failed")
                              (cons "state_sequence"
                                    '("accepted" "running" "failed"))
                              (cons "error"
                                    (error-message-string err)))))))
                     (puthash
                      request-id
                      (soma-gym-bridge--response
                       request-id operation
                       (soma-gym-bridge--get result "state")
                       (soma-gym-bridge--get result "state_sequence")
                       result
                       (cond
                        ((string= (soma-gym-bridge--get result "state")
                                  "succeeded")
                         nil)
                        ((string= (soma-gym-bridge--get result "state")
                                  "outcome_unknown")
                         "timeout")
                        (t
                         "operation_failed"))
                       (soma-gym-bridge--get result "error"))
                      soma-gym-bridge--requests))))))))
    (push timer soma-gym-bridge--async-timers)
    running-response))

(defun soma-gym-bridge--submit-jupyter-eval-region
    (request-id operation buffer start end preconditions timeout queue-delay)
  "Queue checked Jupyter region evaluation and return a running response."
  (let* ((epoch soma-gym-bridge--transport-epoch)
         (running-result
          (soma-gym-bridge--alist
           (cons "operation" "jupyter.eval_region")
           (cons "state" "running")
           (cons "start" (or start :json-false))
           (cons "end" (or end :json-false))
           (cons "wait_timeout_seconds" timeout)
           (cons "queue_delay_seconds" queue-delay)))
         (running-response
          (soma-gym-bridge--response
           request-id operation "running"
           '("accepted" "running")
           running-result))
         timer)
    (puthash request-id running-response soma-gym-bridge--requests)
    (setq timer
          (run-at-time
           queue-delay nil
           (lambda ()
             (when (= epoch soma-gym-bridge--transport-epoch)
               (let ((current (gethash request-id
                                       soma-gym-bridge--requests)))
                 (when (and current
                            (string= (soma-gym-bridge--get current "state")
                                     "running"))
                   (let ((result
                          (condition-case err
                              (soma-gym-bridge-jupyter-eval-region
                               buffer start end preconditions timeout)
                            ((quit error)
                             (soma-gym-bridge--alist
                              (cons "operation" "jupyter.eval_region")
                              (cons "state" "failed")
                              (cons "state_sequence"
                                    '("accepted" "running" "failed"))
                              (cons "error"
                                    (error-message-string err)))))))
                     (puthash
                      request-id
                      (soma-gym-bridge--response
                       request-id operation
                       (soma-gym-bridge--get result "state")
                       (soma-gym-bridge--get result "state_sequence")
                       result
                       (cond
                        ((string= (soma-gym-bridge--get result "state")
                                  "succeeded")
                         nil)
                        ((string= (soma-gym-bridge--get result "state")
                                  "outcome_unknown")
                         "timeout")
                        (t
                         "operation_failed"))
                       (soma-gym-bridge--get result "error"))
                      soma-gym-bridge--requests))))))))
    (push timer soma-gym-bridge--async-timers)
    running-response))

(defun soma-gym-bridge-jupyter-await-request
    (target-id timeout)
  "Await an existing Jupyter TARGET-ID without resubmitting kernel work."
  (let ((deadline (+ (float-time) timeout))
        status target-operation target-result target-state error-message)
    (cond
     ((not (stringp target-id))
      (soma-gym-bridge--alist
       (cons "operation" "jupyter.await_request")
       (cons "state" "failed")
       (cons "state_sequence" '("accepted" "failed"))
       (cons "route" "jupyter-request-await")
       (cons "request_id" :json-false)
       (cons "target_operation" :json-false)
       (cons "timeout_seconds" timeout)
       (cons "status" :json-false)
       (cons "error" "jupyter.await_request requires a target request_id")))
     ((not (setq status (gethash target-id soma-gym-bridge--requests)))
      (soma-gym-bridge--alist
       (cons "operation" "jupyter.await_request")
       (cons "state" "failed")
       (cons "state_sequence" '("accepted" "failed"))
       (cons "route" "jupyter-request-await")
       (cons "request_id" target-id)
       (cons "target_operation" :json-false)
       (cons "timeout_seconds" timeout)
       (cons "status" :json-false)
       (cons "error" "request id is not known to this bridge transport")))
     ((progn
        (setq target-operation (soma-gym-bridge--get status "operation"))
        (not (and (stringp target-operation)
                  (string-prefix-p "jupyter." target-operation)
                  (not (string= target-operation "jupyter.await_request")))))
      (soma-gym-bridge--alist
       (cons "operation" "jupyter.await_request")
       (cons "state" "failed")
       (cons "state_sequence" '("accepted" "failed"))
       (cons "route" "jupyter-request-await")
       (cons "request_id" target-id)
       (cons "target_operation" (or target-operation :json-false))
       (cons "timeout_seconds" timeout)
       (cons "status" status)
       (cons "error" "target request is not a Jupyter request")))
     (t
      (while (and (not (soma-gym-bridge--terminal-state-p
                        (soma-gym-bridge--get status "state")))
                  (< (float-time) deadline))
        (accept-process-output nil 0.02)
        (setq status (gethash target-id soma-gym-bridge--requests)))
      (setq target-result (soma-gym-bridge--get status "result"))
      (setq target-state (soma-gym-bridge--get status "state"))
      (if (soma-gym-bridge--terminal-state-p target-state)
          (progn
            (setq error-message
                  (cond
                   ((string= target-state "succeeded")
                    :json-false)
                   ((and (listp target-result)
                         (stringp (soma-gym-bridge--get target-result
                                                        "error")))
                    (soma-gym-bridge--get target-result "error"))
                   ((stringp (soma-gym-bridge--get status "error"))
                    (soma-gym-bridge--get status "error"))
                   ((string= target-state "outcome_unknown")
                    "target request outcome is unknown")
                   (t
                    "target request failed")))
            (soma-gym-bridge--alist
             (cons "operation" "jupyter.await_request")
             (cons "state" target-state)
             (cons "state_sequence"
                   (cond
                    ((string= target-state "succeeded")
                     '("accepted" "running" "succeeded"))
                    ((string= target-state "failed")
                     '("accepted" "running" "failed"))
                    (t
                     '("accepted" "running" "outcome_unknown"))))
             (cons "route" "jupyter-request-await")
             (cons "request_id" target-id)
             (cons "target_operation" target-operation)
             (cons "target_state" target-state)
             (cons "timeout_seconds" timeout)
             (cons "status" status)
             (cons "target_result" (or target-result :json-false))
             (cons "reply_status"
                   (or (and (listp target-result)
                            (soma-gym-bridge--get target-result
                                                  "reply_status"))
                       :json-false))
             (cons "reply_parent"
                   (or (and (listp target-result)
                            (soma-gym-bridge--get target-result
                                                  "reply_parent"))
                       :json-false))
             (cons "message_types"
                   (or (and (listp target-result)
                            (soma-gym-bridge--get target-result
                                                  "message_types"))
                       nil))
             (cons "parent_ids"
                   (or (and (listp target-result)
                            (soma-gym-bridge--get target-result
                                                  "parent_ids"))
                       nil))
             (cons "stream_texts"
                   (or (and (listp target-result)
                            (soma-gym-bridge--get target-result
                                                  "stream_texts"))
                       nil))
             (cons "idle"
                   (or (and (listp target-result)
                            (soma-gym-bridge--get target-result "idle"))
                       :json-false))
             (cons "error" error-message)))
        (soma-gym-bridge--alist
         (cons "operation" "jupyter.await_request")
         (cons "state" "outcome_unknown")
         (cons "state_sequence" '("accepted" "running" "outcome_unknown"))
         (cons "route" "jupyter-request-await")
         (cons "request_id" target-id)
         (cons "target_operation" target-operation)
         (cons "target_state" (soma-gym-bridge--get status "state"))
         (cons "timeout_seconds" timeout)
         (cons "status" status)
         (cons "target_result" (or target-result :json-false))
         (cons "error" "target request did not reach terminal state before timeout")))))))

(defun soma-gym-bridge-jupyter-query-state (buffer timeout)
  "Query fixed benchmark state from BUFFER's associated Jupyter client."
  (let ((checks (soma-gym-bridge--jupyter-rootfs-checks buffer nil))
        (before (and (buffer-live-p buffer)
                     (soma-gym-bridge-observe-jupyter-source
                      buffer nil nil 800)))
        (client nil)
        (state-path (soma-gym-bridge--runtime-temp-file
                     "soma-jupyter-state-" ".json"))
        req idle payload error-message)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (progn
          (when (file-exists-p state-path)
            (delete-file state-path))
          (soma-gym-bridge--alist
           (cons "operation" "jupyter.query_state")
           (cons "state" "failed")
           (cons "state_sequence" '("accepted" "failed"))
           (cons "route" "precondition-check")
           (cons "precondition_checks" checks)
           (cons "before" (or before :json-false))
           (cons "state_path" state-path)
           (cons "error" "Jupyter query precondition failed")))
      (with-current-buffer buffer
        (setq client (soma-gym-bridge--jupyter-current-client))
        (if (not client)
            (setq error-message
                  "source buffer has no associated Jupyter client")
          (let ((code
                 (concat
                  "import json, os, pathlib, sys\n"
                  "pathlib.Path(" (json-encode state-path) ").write_text(json.dumps({\n"
                  "    'counter': globals().get('SOMA_BENCH_COUNTER', None),\n"
                  "    'training_executions': globals().get('SOMA_TRAINING_EXECUTIONS', None),\n"
                  "    'kernel_token': globals().get('SOMA_BENCH_KERNEL_TOKEN', ''),\n"
                  "    'run_id': globals().get('SOMA_BENCH_RUN_ID', ''),\n"
                  "    'executable': sys.executable,\n"
                  "    'prefix': sys.prefix,\n"
                  "    'cwd': os.getcwd(),\n"
                  "    'plan': os.environ.get('SOMA_INSULA_PLAN', ''),\n"
                  "}, sort_keys=True) + '\\n', encoding='utf-8')\n"
                  "print('SOMA_DAEMON_JUPYTER_STATE:" state-path "')\n")))
            (condition-case err
                (setq req (jupyter-eval-string code nil nil nil))
              ((quit error)
               (setq error-message (error-message-string err)))))))
      (when req
        (setq idle (jupyter-wait-until-idle req timeout)))
      (when (file-readable-p state-path)
        (setq payload
              (condition-case nil
                  (soma-gym-bridge--json-read-file state-path)
                (error nil))))
      (when (file-exists-p state-path)
        (delete-file state-path))
      (append
       (soma-gym-bridge--alist
        (cons "operation" "jupyter.query_state")
        (cons "state"
              (cond
               (error-message "failed")
               ((and idle payload) "succeeded")
               (t "outcome_unknown")))
        (cons "state_sequence"
              (cond
               (error-message '("accepted" "running" "failed"))
               ((and idle payload) '("accepted" "running" "succeeded"))
               (t '("accepted" "running" "outcome_unknown"))))
        (cons "precondition_checks" checks)
        (cons "before" (or before :json-false))
        (cons "idle" (soma-gym-bridge--bool idle))
        (cons "state_path" state-path)
        (cons "payload" (or payload :json-false))
        (cons "error" (or error-message :json-false)))
       (soma-gym-bridge--jupyter-request-evidence req)))))

(defun soma-gym-bridge--required-request-checks (required-requests)
  (if (not (listp required-requests))
      (list
       (soma-gym-bridge--alist
        (cons "name" "required request list is valid")
        (cons "request_id" :json-false)
        (cons "expected_state" :json-false)
        (cons "actual_state" :json-false)
        (cons "terminal" :json-false)
        (cons "passed" :json-false)
        (cons "detail" "required_requests must be a list")))
    (mapcar
     (lambda (spec)
       (let* ((request-id (soma-gym-bridge--get spec "request_id"))
              (expected-state (or (soma-gym-bridge--get spec "state")
                                  "succeeded"))
              (status (and (stringp request-id)
                           (gethash request-id
                                    soma-gym-bridge--requests)))
              (actual-state (and status
                                 (soma-gym-bridge--get status "state")))
              (terminal
               (and actual-state
                    (soma-gym-bridge--terminal-state-p actual-state)))
              (passed
               (and terminal
                    (string= actual-state expected-state))))
         (soma-gym-bridge--alist
          (cons "name" "required request reached expected terminal state")
          (cons "request_id" (or request-id :json-false))
          (cons "expected_state" expected-state)
          (cons "actual_state" (or actual-state :json-false))
          (cons "terminal" (soma-gym-bridge--bool terminal))
          (cons "passed" (soma-gym-bridge--bool passed))
          (cons "detail" (or status "request id is not known")))))
     required-requests)))

(defun soma-gym-bridge--episode-buffer-check (spec)
  (let* ((buffer (soma-gym-bridge--buffer-from-params spec))
         (start (soma-gym-bridge--integer-param spec "start"))
         (end (soma-gym-bridge--integer-param spec "end"))
         (preconditions (or (soma-gym-bridge--get spec "preconditions")
                            nil))
         (checks (soma-gym-bridge--buffer-precondition-checks
                  buffer preconditions start end))
         (observation
          (and (buffer-live-p buffer)
               (soma-gym-bridge-observe-buffer buffer))))
    (soma-gym-bridge--alist
     (cons "name" "final buffer state matches evidence")
     (cons "buffer_name" (or (soma-gym-bridge--get spec "buffer_name")
                             :json-false))
     (cons "start" (or start :json-false))
     (cons "end" (or end :json-false))
     (cons "passed" (soma-gym-bridge--bool
                     (soma-gym-bridge--all-checks-pass-p checks)))
     (cons "checks" checks)
     (cons "observation" (or observation :json-false)))))

(defun soma-gym-bridge--episode-buffer-checks (final-buffer-checks)
  (if (not (listp final-buffer-checks))
      (list
       (soma-gym-bridge--alist
        (cons "name" "final buffer check list is valid")
        (cons "buffer_name" :json-false)
        (cons "passed" :json-false)
        (cons "checks" nil)
        (cons "observation" :json-false)))
    (mapcar #'soma-gym-bridge--episode-buffer-check
            final-buffer-checks)))

(defun soma-gym-bridge-episode-finish (required-requests final-buffer-checks)
  "Return episode completion evidence for REQUIRED-REQUESTS and buffers."
  (let* ((request-checks
          (soma-gym-bridge--required-request-checks required-requests))
         (buffer-checks
          (soma-gym-bridge--episode-buffer-checks final-buffer-checks))
         (request-passed (soma-gym-bridge--all-checks-pass-p
                          request-checks))
         (buffer-passed (soma-gym-bridge--all-checks-pass-p
                         buffer-checks))
         (passed (and request-passed buffer-passed)))
    (soma-gym-bridge--alist
     (cons "operation" "episode.finish")
     (cons "state" (if passed "succeeded" "failed"))
     (cons "state_sequence"
           (if passed
               '("accepted" "running" "succeeded")
             '("accepted" "running" "failed")))
     (cons "request_checks" request-checks)
     (cons "buffer_checks" buffer-checks)
     (cons "request_checks_passed" (soma-gym-bridge--bool
                                    request-passed))
     (cons "buffer_checks_passed" (soma-gym-bridge--bool
                                   buffer-passed)))))

(defun soma-gym-bridge--execute-request (request-id operation params)
  (cond
   ((string= operation "api.list_capabilities")
    (soma-gym-bridge--response
     request-id operation "succeeded"
     '("accepted" "running" "succeeded")
     (soma-gym-bridge-list-capabilities)))
   ((string= operation "api.describe_operation")
    (let* ((target-operation (soma-gym-bridge--get params "operation"))
           (description
            (soma-gym-bridge-describe-operation target-operation)))
      (if description
          (soma-gym-bridge--response
           request-id operation "succeeded"
           '("accepted" "running" "succeeded")
           description)
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "failed")
         nil
         "unknown_operation"
         "api.describe_operation requires a known operation"))))
   ((string= operation "emacs.observe_session")
    (soma-gym-bridge--response
     request-id operation "succeeded"
     '("accepted" "running" "succeeded")
     (soma-gym-bridge-observe-session)))
   ((string= operation "emacs.observe_buffer")
    (let ((buffer (soma-gym-bridge--buffer-from-params params)))
      (if (buffer-live-p buffer)
          (soma-gym-bridge--response
           request-id operation "succeeded"
           '("accepted" "running" "succeeded")
           (soma-gym-bridge-observe-buffer buffer))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))))
   ((string= operation "emacs.observe_frames")
    (soma-gym-bridge--response
     request-id operation "succeeded"
     '("accepted" "running" "succeeded")
     (soma-gym-bridge-observe-frames)))
   ((string= operation "emacs.observe_interaction")
    (soma-gym-bridge--response
     request-id operation "succeeded"
     '("accepted" "running" "succeeded")
     (soma-gym-bridge-observe-interaction)))
   ((string= operation "buffer.select")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (point (soma-gym-bridge--integer-param params "point"))
           (result (soma-gym-bridge-buffer-select buffer point)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         (cond
          ((not (buffer-live-p buffer)) "unknown_buffer")
          ((soma-gym-bridge--get result "point") "invalid_point")
          (t "operation_failed")))
       (soma-gym-bridge--get result "error"))))
   ((string= operation "project.observe")
    (let ((result (soma-gym-bridge-observe-project params)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (if (string= (soma-gym-bridge--get result "state") "succeeded")
           '("accepted" "running" "succeeded")
         '("accepted" "running" "failed"))
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "operation_failed")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "project.search")
    (let* ((max-results
            (floor (soma-gym-bridge--number-param
                    params "max_results" 50 1 1000)))
           (max-text
            (floor (soma-gym-bridge--number-param
                    params "max_text" 400 0 4000)))
           (result
            (soma-gym-bridge-project-search
             params max-results max-text)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (if (string= (soma-gym-bridge--get result "state") "succeeded")
           '("accepted" "running" "succeeded")
         '("accepted" "running" "failed"))
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "operation_failed")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "org.observe_tree")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (max-headings
            (floor (soma-gym-bridge--number-param
                    params "max_headings" 100 1 2000)))
           (max-source-blocks
            (floor (soma-gym-bridge--number-param
                    params "max_source_blocks" 100 1 2000)))
           (max-text
            (floor (soma-gym-bridge--number-param
                    params "max_text" 400 0 4000)))
           (result
            (soma-gym-bridge-observe-org-tree
             buffer max-headings max-source-blocks max-text)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (if (string= (soma-gym-bridge--get result "state") "succeeded")
           '("accepted" "running" "succeeded")
         '("accepted" "running" "failed"))
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "operation_failed")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "org.update_heading")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (heading (soma-gym-bridge--get params "heading"))
           (changes (soma-gym-bridge--get params "changes"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (result
            (soma-gym-bridge-org-update-heading
             buffer heading changes preconditions)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "conflict")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "org.edit_src_block")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (source-block (soma-gym-bridge--get params "source_block"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (result
            (soma-gym-bridge-org-edit-src-block
             buffer source-block preconditions)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "conflict")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "org.exit_src_edit")
    (let* ((edit-buffer-name (soma-gym-bridge--get params "edit_buffer_name"))
           (edit-buffer (and (stringp edit-buffer-name)
                             (get-buffer edit-buffer-name)))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (result
            (soma-gym-bridge-org-exit-src-edit
             edit-buffer preconditions)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "conflict")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "org.execute_babel_block")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (source-block (soma-gym-bridge--get params "source_block"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (timeout (soma-gym-bridge--number-param
                     params "timeout_seconds" 0 0 120))
           (result
            (soma-gym-bridge-org-execute-babel-block
             buffer source-block preconditions timeout)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (cond
        ((string= (soma-gym-bridge--get result "state") "succeeded")
         nil)
        ((string= (soma-gym-bridge--get result "state") "outcome_unknown")
         "timeout")
        (t "conflict"))
       (soma-gym-bridge--get result "error"))))
   ((string= operation "org.export_html")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (output-path (soma-gym-bridge--get params "output_path"))
           (mode (soma-gym-bridge--get params "mode"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (result
            (soma-gym-bridge-org-export-html
             buffer output-path preconditions mode)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "conflict")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "org.parent_text_write")
    (let* ((buffer-name (soma-gym-bridge--get params "buffer_name"))
           (source-block (soma-gym-bridge--get params "source_block"))
           (text (soma-gym-bridge--get params "text"))
           (reason
            "model-facing transport does not expose direct Org parent-buffer text writes"))
      (soma-gym-bridge--response
       request-id operation "failed"
       '("accepted" "failed")
       (soma-gym-bridge--alist
        (cons "operation" "org.parent_text_write")
        (cons "state" "failed")
        (cons "state_sequence" '("accepted" "failed"))
        (cons "requested_buffer" (or buffer-name :json-false))
        (cons "requested_source_block"
              (if (listp source-block) source-block :json-false))
        (cons "requested_text_bytes"
              (if (stringp text) (string-bytes text) 0))
        (cons "route" "rejection-only")
        (cons "direct_parent_text_write_attempted" :json-false)
        (cons "source_edit_bypassed" :json-false)
        (cons "written_bytes" 0)
        (cons "rejection_reason" reason)
        (cons "reason" reason))
       "unauthorized_operation"
       reason)))
   ((string= operation "dired.observe")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (max-files
            (floor (soma-gym-bridge--number-param
                    params "max_files" 200 1 2000)))
           (result
            (soma-gym-bridge-observe-dired buffer max-files)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (if (string= (soma-gym-bridge--get result "state") "succeeded")
           '("accepted" "running" "succeeded")
         '("accepted" "running" "failed"))
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "operation_failed")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "dired.rename_no_overwrite")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (source (soma-gym-bridge--get params "source"))
           (target (soma-gym-bridge--get params "target"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (result
            (soma-gym-bridge-dired-rename-no-overwrite
             buffer source target preconditions)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "conflict")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "dired.delete_marked")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (result
            (soma-gym-bridge-dired-delete-marked buffer preconditions))
           (precondition-passed
            (eq (soma-gym-bridge--get result "precondition_checks_passed")
                t)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         (if precondition-passed "operation_failed" "conflict"))
       (soma-gym-bridge--get result "error"))))
   ((string= operation "filesystem.rename")
    (let* ((source-path (soma-gym-bridge--get params "source_path"))
           (target-path (soma-gym-bridge--get params "target_path"))
           (reason
            "model-facing transport does not expose raw filesystem renames"))
      (soma-gym-bridge--response
       request-id operation "failed"
       '("accepted" "failed")
       (soma-gym-bridge--alist
        (cons "operation" "filesystem.rename")
        (cons "state" "failed")
        (cons "state_sequence" '("accepted" "failed"))
        (cons "requested_source_path" (or source-path :json-false))
        (cons "requested_target_path" (or target-path :json-false))
        (cons "route" "rejection-only")
        (cons "used_raw_filesystem_rename" :json-false)
        (cons "renamed_paths" 0)
        (cons "rejection_reason" reason)
        (cons "reason" reason))
       "unauthorized_operation"
       reason)))
   ((string= operation "comint.observe")
    (let ((buffer (soma-gym-bridge--buffer-from-params params)))
      (if (buffer-live-p buffer)
          (soma-gym-bridge--response
           request-id operation "succeeded"
           '("accepted" "running" "succeeded")
           (soma-gym-bridge-observe-comint buffer))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))))
   ((string= operation "comint.submit_input")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (input (soma-gym-bridge--get params "input"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil)))
      (cond
       ((not (buffer-live-p buffer))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))
       ((not (stringp input))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "failed")
         nil
         "invalid_input"
         "comint.submit_input requires string input"))
       (t
        (let ((result
               (soma-gym-bridge-comint-submit-input
                buffer input preconditions)))
          (soma-gym-bridge--response
           request-id operation
           (soma-gym-bridge--get result "state")
           (soma-gym-bridge--get result "state_sequence")
           result
           (unless (string= (soma-gym-bridge--get result "state")
                            "succeeded")
             "conflict")
           (soma-gym-bridge--get result "error")))))))
   ((string= operation "comint.await_output")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (needle (soma-gym-bridge--get params "needle"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (timeout (soma-gym-bridge--number-param
                     params "timeout_seconds" 2.0 0.0 120.0)))
      (if (not (buffer-live-p buffer))
          (soma-gym-bridge--response
           request-id operation "failed"
           '("accepted" "running" "failed")
           nil
           "unknown_buffer"
           "buffer_name does not name a live buffer")
        (let ((result
               (soma-gym-bridge-comint-await-output
                buffer needle preconditions timeout)))
          (soma-gym-bridge--response
           request-id operation
           (soma-gym-bridge--get result "state")
           (soma-gym-bridge--get result "state_sequence")
           result
           (unless (string= (soma-gym-bridge--get result "state")
                            "succeeded")
             (if (string= (soma-gym-bridge--get result "state")
                          "outcome_unknown")
                 "timeout"
               "conflict"))
           (soma-gym-bridge--get result "error"))))))
   ((string= operation "comint.interrupt")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil)))
      (if (not (buffer-live-p buffer))
          (soma-gym-bridge--response
           request-id operation "failed"
           '("accepted" "running" "failed")
           nil
           "unknown_buffer"
           "buffer_name does not name a live buffer")
        (let ((result
               (soma-gym-bridge-comint-interrupt
                buffer preconditions)))
          (soma-gym-bridge--response
           request-id operation
           (soma-gym-bridge--get result "state")
           (soma-gym-bridge--get result "state_sequence")
           result
           (unless (string= (soma-gym-bridge--get result "state")
                            "succeeded")
             "conflict")
           (soma-gym-bridge--get result "error"))))))
   ((string= operation "magit.observe_sections")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (max-sections
            (floor (soma-gym-bridge--number-param
                    params "max_sections" 200 1 2000)))
           (max-text
            (floor (soma-gym-bridge--number-param
                    params "max_text" 400 0 4000))))
      (if (buffer-live-p buffer)
          (let ((result
                 (soma-gym-bridge-observe-magit-sections
                  buffer max-sections max-text)))
            (soma-gym-bridge--response
             request-id operation
             (soma-gym-bridge--get result "state")
	             (or (soma-gym-bridge--get result "state_sequence")
	                 (if (string= (soma-gym-bridge--get result "state")
	                              "succeeded")
	                     '("accepted" "running" "succeeded")
	                   '("accepted" "running" "failed")))
	             result
	             (unless (string= (soma-gym-bridge--get result "state")
	                              "succeeded")
	               "operation_failed")
	             (soma-gym-bridge--get result "error")))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))))
	   ((or (string= operation "magit.stage_hunk")
	        (string= operation "magit.stage_file"))
	    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
	           (section (soma-gym-bridge--get params "section"))
	           (preconditions (or (soma-gym-bridge--get params "preconditions")
	                              nil))
	           (expected-type
	            (if (string= operation "magit.stage_hunk")
	                "hunk"
	              "file")))
	      (if (not (buffer-live-p buffer))
	          (soma-gym-bridge--response
	           request-id operation "failed"
	           '("accepted" "running" "failed")
	           nil
	           "unknown_buffer"
	           "buffer_name does not name a live buffer")
	        (let ((result
	               (soma-gym-bridge-magit-stage-section
	                buffer operation section preconditions expected-type)))
	          (soma-gym-bridge--response
	           request-id operation
	           (soma-gym-bridge--get result "state")
	           (soma-gym-bridge--get result "state_sequence")
	           result
	           (unless (string= (soma-gym-bridge--get result "state")
	                            "succeeded")
	             "conflict")
	           (soma-gym-bridge--get result "error"))))))
	   ((string= operation "jupyter.observe_source")
	    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
	           (start (soma-gym-bridge--integer-param params "start"))
	           (end (soma-gym-bridge--integer-param params "end"))
	           (max-text
	            (floor (soma-gym-bridge--number-param
	                    params "max_text" 400 0 4000))))
	      (if (buffer-live-p buffer)
	          (let ((result
	                 (soma-gym-bridge-observe-jupyter-source
	                  buffer start end max-text)))
	            (soma-gym-bridge--response
	             request-id operation
	             (soma-gym-bridge--get result "state")
	             (or (soma-gym-bridge--get result "state_sequence")
	                 (if (string= (soma-gym-bridge--get result "state")
	                              "succeeded")
	                     '("accepted" "running" "succeeded")
	                   '("accepted" "running" "failed")))
	             result
	             (unless (string= (soma-gym-bridge--get result "state")
	                              "succeeded")
	               "operation_failed")
	             (soma-gym-bridge--get result "error")))
	        (soma-gym-bridge--response
	         request-id operation "failed"
	         '("accepted" "running" "failed")
	         nil
	         "unknown_buffer"
	         "buffer_name does not name a live buffer"))))
   ((string= operation "jupyter.attach_existing")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (client-name (soma-gym-bridge--get params "client"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil)))
      (cond
       ((not (buffer-live-p buffer))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))
       ((not (and (stringp client-name)
                  (not (string-empty-p client-name))))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "failed")
         nil
         "invalid_client"
         "jupyter.attach_existing requires client to name a REPL buffer"))
       (t
        (let ((result
               (soma-gym-bridge-jupyter-attach-existing
                buffer client-name preconditions)))
          (soma-gym-bridge--response
           request-id operation
           (soma-gym-bridge--get result "state")
           (soma-gym-bridge--get result "state_sequence")
           result
           (unless (string= (soma-gym-bridge--get result "state")
                            "succeeded")
             "conflict")
           (soma-gym-bridge--get result "error")))))))
   ((string= operation "jupyter.observe_display")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (display-id (soma-gym-bridge--get params "display_id"))
           (max-text
            (floor (soma-gym-bridge--number-param
                    params "max_text" 800 0 8000))))
      (if (buffer-live-p buffer)
          (let ((result
                 (soma-gym-bridge-observe-jupyter-display
                  buffer display-id max-text)))
            (soma-gym-bridge--response
             request-id operation
             (soma-gym-bridge--get result "state")
             '("accepted" "running" "succeeded")
             result))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))))
	   ((string= operation "jupyter.eval_region")
	    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
	           (start (soma-gym-bridge--integer-param params "start"))
	           (end (soma-gym-bridge--integer-param params "end"))
	           (timeout (soma-gym-bridge--number-param
	                     params "timeout_seconds" 20.0 0.0 120.0))
		           (queue-delay
		            (soma-gym-bridge--number-param
		             params "queue_delay_seconds"
		             soma-gym-bridge-async-command-delay-seconds
		             0.0 10.0))
	           (preconditions (or (soma-gym-bridge--get params "preconditions")
	                              nil)))
	      (cond
	       ((not (buffer-live-p buffer))
	        (soma-gym-bridge--response
	         request-id operation "failed"
	         '("accepted" "running" "failed")
	         nil
	         "unknown_buffer"
	         "buffer_name does not name a live buffer"))
	       ((not (and (integerp start) (integerp end)))
	        (soma-gym-bridge--response
	         request-id operation "failed"
	         '("accepted" "failed")
	         nil
	         "invalid_range"
	         "jupyter.eval_region requires integer start/end"))
	       (t
	        (soma-gym-bridge--submit-jupyter-eval-region
		         request-id operation buffer start end preconditions timeout
		         queue-delay)))))
   ((string= operation "jupyter.eval_region_via_repl")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (start (soma-gym-bridge--integer-param params "start"))
           (end (soma-gym-bridge--integer-param params "end"))
           (timeout (soma-gym-bridge--number-param
                     params "timeout_seconds" 20.0 0.0 120.0))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil)))
      (cond
       ((not (buffer-live-p buffer))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))
       ((not (and (integerp start) (integerp end)))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "failed")
         nil
         "invalid_range"
         "jupyter.eval_region_via_repl requires integer start/end"))
       (t
        (soma-gym-bridge--submit-jupyter-eval-region-via-repl
         request-id operation buffer start end preconditions timeout)))))
   ((string= operation "jupyter.await_request")
    (let* ((target-id (soma-gym-bridge--get params "request_id"))
           (timeout (soma-gym-bridge--number-param
                     params "timeout_seconds" 0.0 0.0 120.0))
           (result
            (soma-gym-bridge-jupyter-await-request target-id timeout))
           (state (soma-gym-bridge--get result "state")))
      (soma-gym-bridge--response
       request-id operation
       state
       (soma-gym-bridge--get result "state_sequence")
       result
       (cond
        ((string= state "succeeded") nil)
        ((string= state "outcome_unknown") "timeout")
        ((not (stringp target-id)) "invalid_request_id")
        ((not (gethash target-id soma-gym-bridge--requests))
         "unknown_request_id")
        (t "conflict"))
       (soma-gym-bridge--get result "error"))))
	   ((string= operation "jupyter.query_state")
	    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
	           (timeout (soma-gym-bridge--number-param
	                     params "timeout_seconds" 20.0 0.0 120.0)))
	      (if (buffer-live-p buffer)
	          (let ((result
	                 (soma-gym-bridge-jupyter-query-state
	                  buffer timeout)))
	            (soma-gym-bridge--response
	             request-id operation
	             (soma-gym-bridge--get result "state")
	             (soma-gym-bridge--get result "state_sequence")
	             result
	             (unless (string= (soma-gym-bridge--get result "state")
	                              "succeeded")
	               (if (string= (soma-gym-bridge--get result "state")
	                            "outcome_unknown")
	                   "timeout"
	                 "operation_failed"))
	             (soma-gym-bridge--get result "error")))
	        (soma-gym-bridge--response
	         request-id operation "failed"
	         '("accepted" "running" "failed")
	         nil
	         "unknown_buffer"
	         "buffer_name does not name a live buffer"))))
   ((string= operation "jupyter.restart_kernel")
    (let* ((buffer-name (soma-gym-bridge--get params "buffer_name"))
           (reason
            "model-facing transport does not expose implicit Jupyter kernel restarts"))
      (soma-gym-bridge--response
       request-id operation "failed"
       '("accepted" "failed")
       (soma-gym-bridge--alist
        (cons "operation" "jupyter.restart_kernel")
        (cons "state" "failed")
        (cons "state_sequence" '("accepted" "failed"))
        (cons "requested_buffer" (or buffer-name :json-false))
        (cons "route" "rejection-only")
        (cons "restart_attempted" :json-false)
        (cons "shutdown_attempted" :json-false)
        (cons "launch_attempted" :json-false)
        (cons "rejection_reason" reason)
        (cons "reason" reason))
       "unauthorized_operation"
       reason)))
   ((string= operation "git.index_write")
    (let* ((repo (soma-gym-bridge--get params "repo"))
           (pathspec (soma-gym-bridge--get params "pathspec"))
           (mode (soma-gym-bridge--get params "mode"))
           (reason
            "model-facing transport does not expose raw Git index writes"))
      (soma-gym-bridge--response
       request-id operation "failed"
       '("accepted" "failed")
       (soma-gym-bridge--alist
        (cons "operation" "git.index_write")
        (cons "state" "failed")
        (cons "state_sequence" '("accepted" "failed"))
        (cons "requested_repo" (or repo :json-false))
        (cons "requested_pathspec" (or pathspec :json-false))
        (cons "requested_mode" (or mode :json-false))
        (cons "route" "rejection-only")
        (cons "used_raw_git_index_write" :json-false)
        (cons "staged_path_count" 0)
        (cons "rejection_reason" reason)
        (cons "reason" reason))
       "unauthorized_operation"
       reason)))
   ((string= operation "lsp.observe_source")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (max-text
            (floor (soma-gym-bridge--number-param
                    params "max_text" 400 0 4000))))
      (if (buffer-live-p buffer)
          (let ((result
                 (soma-gym-bridge-observe-lsp-source
                  buffer max-text)))
            (soma-gym-bridge--response
             request-id operation
             (soma-gym-bridge--get result "state")
	             (or (soma-gym-bridge--get result "state_sequence")
	                 (if (string= (soma-gym-bridge--get result "state")
	                              "succeeded")
	                     '("accepted" "running" "succeeded")
	                   '("accepted" "running" "failed")))
	             result
	             (unless (string= (soma-gym-bridge--get result "state")
	                              "succeeded")
	               "operation_failed")
	             (soma-gym-bridge--get result "error")))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))))
   ((string= operation "lsp.observe_diagnostics")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (settle-seconds
            (soma-gym-bridge--number-param
             params "settle_seconds" 0.2 0.0 5.0))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil)))
      (if (buffer-live-p buffer)
          (let ((result
                 (soma-gym-bridge-observe-lsp-diagnostics
                  buffer preconditions settle-seconds)))
            (soma-gym-bridge--response
             request-id operation
             (soma-gym-bridge--get result "state")
             (if (string= (soma-gym-bridge--get result "state")
                          "succeeded")
                 '("accepted" "running" "succeeded")
               '("accepted" "running" "failed"))
             result
             (unless (string= (soma-gym-bridge--get result "state")
                              "succeeded")
               "conflict")
             (soma-gym-bridge--get result "error")))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))))
   ((string= operation "lsp.rename_symbol")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (new-name (soma-gym-bridge--get params "new_name"))
           (timeout (soma-gym-bridge--number-param
                     params "timeout_seconds" 20.0 0.0 120.0))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil)))
      (cond
       ((not (buffer-live-p buffer))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))
       ((not (and (stringp new-name) (not (string-empty-p new-name))))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "failed")
         nil
         "invalid_new_name"
         "lsp.rename_symbol requires non-empty string new_name"))
       (t
        (soma-gym-bridge--submit-lsp-rename-symbol
         request-id operation buffer new-name preconditions timeout)))))
   ((string= operation "emacs.goto_char")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (point (soma-gym-bridge--integer-param params "point"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (result
            (soma-gym-bridge-goto-char buffer point preconditions)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "conflict")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "command.dispatch")
    (let ((buffer (soma-gym-bridge--buffer-from-params params))
          (command (soma-gym-bridge--command-from-params params))
          (preconditions (soma-gym-bridge--get params "preconditions")))
      (if (not (buffer-live-p buffer))
          (soma-gym-bridge--response
           request-id operation "failed"
           '("accepted" "running" "failed")
           nil
           "unknown_buffer"
           "buffer_name does not name a live buffer")
        (if soma-gym-bridge-async-command-dispatch
            (soma-gym-bridge--submit-queued-command-dispatch
             request-id operation buffer command (or preconditions nil))
          (soma-gym-bridge--command-dispatch-response
           request-id operation
           (soma-gym-bridge-command-dispatch
            buffer command (or preconditions nil)))))))
   ((string= operation "buffer.read")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (start (soma-gym-bridge--integer-param params "start"))
           (end (soma-gym-bridge--integer-param params "end"))
           (result (soma-gym-bridge-buffer-read buffer start end)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (if (string= (soma-gym-bridge--get result "state") "succeeded")
           '("accepted" "running" "succeeded")
         '("accepted" "running" "failed"))
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         (if (buffer-live-p buffer) "invalid_range" "unknown_buffer"))
       (soma-gym-bridge--get result "error"))))
   ((string= operation "buffer.replace_range")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (start (soma-gym-bridge--integer-param params "start"))
           (end (soma-gym-bridge--integer-param params "end"))
           (text (soma-gym-bridge--get params "text"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil)))
      (if (not (stringp text))
          (soma-gym-bridge--response
           request-id operation "failed"
           '("accepted" "failed")
           nil
           "invalid_text"
           "buffer.replace_range requires string text")
        (let ((result
               (soma-gym-bridge-buffer-replace-range
                buffer start end text preconditions)))
          (soma-gym-bridge--response
           request-id operation
           (soma-gym-bridge--get result "state")
           (soma-gym-bridge--get result "state_sequence")
           result
           (unless (string= (soma-gym-bridge--get result "state")
                            "succeeded")
             "conflict")
           (soma-gym-bridge--get result "error"))))))
   ((string= operation "buffer.apply_patch")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (patch (soma-gym-bridge--get params "patch"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (result
            (soma-gym-bridge-buffer-apply-patch
             buffer patch preconditions)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state")
                        "succeeded")
         "conflict")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "buffer.save")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil))
           (result (soma-gym-bridge-buffer-save buffer preconditions)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "conflict")
       (soma-gym-bridge--get result "error"))))
   ((string= operation "completion.accept")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (start (soma-gym-bridge--integer-param params "start"))
           (end (soma-gym-bridge--integer-param params "end"))
           (text (soma-gym-bridge--get params "text"))
           (preconditions (or (soma-gym-bridge--get params "preconditions")
                              nil)))
      (if (not (stringp text))
          (soma-gym-bridge--response
           request-id operation "failed"
           '("accepted" "failed")
           nil
           "invalid_text"
           "completion.accept requires string text")
        (let ((result
               (soma-gym-bridge-completion-accept
                buffer start end text preconditions)))
          (soma-gym-bridge--response
           request-id operation
           (soma-gym-bridge--get result "state")
           (soma-gym-bridge--get result "state_sequence")
           result
           (unless (string= (soma-gym-bridge--get result "state")
                            "succeeded")
             "conflict")
           (soma-gym-bridge--get result "error"))))))
   ((string= operation "completion.observe")
    (let* ((buffer (soma-gym-bridge--buffer-from-params params))
           (max-candidates
            (floor (soma-gym-bridge--number-param
                    params "max_candidates" 20 0 200)))
           (prefix-chars
            (floor (soma-gym-bridge--number-param
                    params "prefix_chars" 80 0 1000)))
           (suffix-chars
            (floor (soma-gym-bridge--number-param
                    params "suffix_chars" 80 0 1000))))
      (if (buffer-live-p buffer)
          (let ((result
                 (soma-gym-bridge-observe-completion
                  buffer max-candidates prefix-chars suffix-chars)))
            (soma-gym-bridge--response
             request-id operation
             (soma-gym-bridge--get result "state")
             (if (string= (soma-gym-bridge--get result "state")
                          "succeeded")
                 '("accepted" "running" "succeeded")
               '("accepted" "running" "failed"))
             result
             (unless (string= (soma-gym-bridge--get result "state")
                              "succeeded")
               "operation_failed")
             (soma-gym-bridge--get result "error")))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         nil
         "unknown_buffer"
         "buffer_name does not name a live buffer"))))
   ((string= operation "episode.finish")
    (let* ((required-requests
            (or (soma-gym-bridge--get params "required_requests")
                nil))
           (final-buffer-checks
            (or (soma-gym-bridge--get params "final_buffer_checks")
                nil))
           (result
            (soma-gym-bridge-episode-finish
             required-requests final-buffer-checks)))
      (soma-gym-bridge--response
       request-id operation
       (soma-gym-bridge--get result "state")
       (soma-gym-bridge--get result "state_sequence")
       result
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "incomplete_episode")
       (unless (string= (soma-gym-bridge--get result "state") "succeeded")
         "episode completion evidence is incomplete"))))
   ((string= operation "request.status")
    (let* ((target-id (soma-gym-bridge--get params "request_id"))
           (known (and (stringp target-id)
                       (gethash target-id soma-gym-bridge--requests)))
           (status (and (stringp target-id)
                        (soma-gym-bridge-request-status target-id))))
      (if known
          (soma-gym-bridge--response
           request-id operation "succeeded"
           '("accepted" "running" "succeeded")
           (soma-gym-bridge--alist
            (cons "operation" "request.status")
            (cons "state" "succeeded")
            (cons "status" status)))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         (soma-gym-bridge--alist
          (cons "operation" "request.status")
          (cons "state" "failed")
          (cons "status" (or status :json-false)))
         "unknown_request_id"
         "request id is not known to this bridge transport"))))
   ((string= operation "request.await")
    (let* ((target-id (soma-gym-bridge--get params "request_id"))
           (timeout (soma-gym-bridge--number-param
                     params "timeout_seconds" 0.0 0.0 120.0))
           (deadline (+ (float-time) timeout))
           status)
      (if (not (stringp target-id))
          (soma-gym-bridge--response
           request-id operation "failed"
           '("accepted" "failed")
           nil
           "invalid_request_id"
           "request.await requires a target request_id")
        (setq status (gethash target-id soma-gym-bridge--requests))
        (if (not status)
            (soma-gym-bridge--response
             request-id operation "failed"
             '("accepted" "failed")
             nil
             "unknown_request_id"
             "request id is not known to this bridge transport")
          (while (and (not (soma-gym-bridge--terminal-state-p
                            (soma-gym-bridge--get status "state")))
                      (< (float-time) deadline))
            (soma-gym-bridge--run-pending-action target-id)
            (setq status (gethash target-id soma-gym-bridge--requests))
            (accept-process-output nil 0.02)
            (sit-for 0.01)
            (setq status (gethash target-id soma-gym-bridge--requests)))
          (if (soma-gym-bridge--terminal-state-p
               (soma-gym-bridge--get status "state"))
              (soma-gym-bridge--response
               request-id operation "succeeded"
               '("accepted" "running" "succeeded")
               (soma-gym-bridge--alist
                (cons "operation" "request.await")
                (cons "state" "succeeded")
                (cons "status" status)))
            (soma-gym-bridge--response
             request-id operation "outcome_unknown"
             '("accepted" "running" "outcome_unknown")
             (soma-gym-bridge--alist
              (cons "operation" "request.await")
              (cons "state" "outcome_unknown")
              (cons "timeout_seconds" timeout)
              (cons "status" status))
             "timeout"
             "target request did not reach terminal state before timeout"))))))
   ((string= operation "bridge.defer_echo")
    (let* ((delay (soma-gym-bridge--number-param
                   params "delay_seconds" 0.1 0.0 10.0))
           (value (soma-gym-bridge--get params "value"))
           (epoch soma-gym-bridge--transport-epoch)
           timer)
      (setq timer
            (run-at-time
             delay nil
             (lambda ()
               (when (= epoch soma-gym-bridge--transport-epoch)
                 (let ((current
                        (gethash request-id soma-gym-bridge--requests)))
                   (when (and current
                              (string= (soma-gym-bridge--get current "state")
                                       "running"))
                     (puthash
                      request-id
                      (soma-gym-bridge--response
                       request-id operation "succeeded"
                       '("accepted" "running" "succeeded")
                       (soma-gym-bridge--alist
                        (cons "operation" "bridge.defer_echo")
                        (cons "state" "succeeded")
                        (cons "value" (or value :json-false))
                        (cons "completed_after_seconds" delay)))
                      soma-gym-bridge--requests)))))))
      (push timer soma-gym-bridge--async-timers)
      (soma-gym-bridge--response
       request-id operation "running"
       '("accepted" "running")
       (soma-gym-bridge--alist
        (cons "operation" "bridge.defer_echo")
        (cons "state" "running")
        (cons "value" (or value :json-false))
        (cons "delay_seconds" delay)))))
   ((string= operation "bridge.prompt_echo")
    (let ((prompt (soma-gym-bridge--get params "prompt"))
          (prefix (soma-gym-bridge--get params "value_prefix")))
      (unless (stringp prompt)
        (setq prompt "response required"))
      (unless (stringp prefix)
        (setq prefix ""))
      (soma-gym-bridge--response
       request-id operation "waiting_for_input"
       '("accepted" "running" "waiting_for_input")
       (soma-gym-bridge--alist
        (cons "operation" "bridge.prompt_echo")
        (cons "state" "waiting_for_input")
        (cons "request_id" request-id)
        (cons "prompt" prompt)
        (cons "value_prefix" prefix)))))
   ((string= operation "request.respond")
    (let* ((target-id (soma-gym-bridge--get params "request_id"))
           (prompt-response (soma-gym-bridge--get params "response"))
           (target-status (and (stringp target-id)
                               (gethash target-id
                                        soma-gym-bridge--requests)))
           (target-result (and target-status
                               (soma-gym-bridge--get target-status
                                                     "result")))
           (prefix (and target-result
                        (soma-gym-bridge--get target-result
                                              "value_prefix")))
           completed)
      (cond
       ((not (stringp target-id))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "failed")
         nil
         "invalid_request_id"
         "request.respond requires a target request_id"))
       ((not target-status)
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "failed")
         nil
         "unknown_request_id"
         "request id is not known to this bridge transport"))
       ((not (string= (soma-gym-bridge--get target-status "state")
                      "waiting_for_input"))
        (soma-gym-bridge--response
         request-id operation "failed"
         '("accepted" "running" "failed")
         (soma-gym-bridge--alist
          (cons "operation" "request.respond")
          (cons "state" "failed")
          (cons "target_status" target-status))
         "invalid_request_state"
         "target request is not waiting for input"))
       (t
        (unless (stringp prompt-response)
          (setq prompt-response ""))
        (unless (stringp prefix)
          (setq prefix ""))
        (setq completed
              (soma-gym-bridge--response
               target-id
               (soma-gym-bridge--get target-status "operation")
               "succeeded"
               '("accepted" "running" "waiting_for_input" "succeeded")
               (soma-gym-bridge--alist
                (cons "operation" (soma-gym-bridge--get target-status
                                                        "operation"))
                (cons "state" "succeeded")
                (cons "prompt_response" prompt-response)
                (cons "value" (concat prefix prompt-response))
                (cons "responded_by" request-id))))
        (puthash target-id completed soma-gym-bridge--requests)
        (soma-gym-bridge--response
         request-id operation "succeeded"
         '("accepted" "running" "succeeded")
         (soma-gym-bridge--alist
          (cons "operation" "request.respond")
          (cons "state" "succeeded")
          (cons "target_status" completed)))))))
   ((string= operation "direct-call")
    (let ((requested (soma-gym-bridge--get params "requested_operation")))
      (soma-gym-bridge--response
       request-id operation "failed"
       '("accepted" "failed")
       (soma-gym-bridge-reject-direct-call
        requested
        "model-facing transport does not expose unchecked direct calls")
       "unauthorized_operation"
       "model-facing transport does not expose unchecked direct calls")))
   ((string= operation "elisp.eval")
    (let ((form (soma-gym-bridge--get params "form")))
      (soma-gym-bridge--response
       request-id operation "failed"
       '("accepted" "failed")
       (soma-gym-bridge--alist
        (cons "operation" "elisp.eval")
        (cons "state" "failed")
        (cons "state_sequence" '("accepted" "failed"))
        (cons "requested_form" (or form :json-false))
        (cons "route" "rejection-only")
        (cons "evaluated" :json-false)
        (cons "rejection_reason"
              "model-facing transport does not expose arbitrary Elisp evaluation")
        (cons "reason"
              "model-facing transport does not expose arbitrary Elisp evaluation"))
       "unauthorized_operation"
       "model-facing transport does not expose arbitrary Elisp evaluation")))
   ((string= operation "process.write")
    (let* ((buffer-name (soma-gym-bridge--get params "buffer_name"))
           (input (soma-gym-bridge--get params "input"))
           (reason
            "model-facing transport does not expose raw process writes"))
      (soma-gym-bridge--response
       request-id operation "failed"
       '("accepted" "failed")
       (soma-gym-bridge--alist
        (cons "operation" "process.write")
        (cons "state" "failed")
        (cons "state_sequence" '("accepted" "failed"))
        (cons "requested_buffer" (or buffer-name :json-false))
        (cons "input_bytes" (if (stringp input) (string-bytes input) 0))
        (cons "sent_bytes" 0)
        (cons "route" "rejection-only")
        (cons "used_raw_process_write" :json-false)
        (cons "rejection_reason" reason)
        (cons "reason" reason))
       "unauthorized_operation"
       reason)))
   (t
    (soma-gym-bridge--response
     request-id operation "failed"
     '("accepted" "failed")
     nil
     "unsupported_operation"
     "operation is not supported by this bridge transport"))))

(defun soma-gym-bridge-submit-request (request)
  "Submit a model-facing bridge REQUEST alist and return its response.

REQUEST must have a string `request_id', string `operation', and optional
string-keyed `params' alist.  This synchronous transport slice records the same
terminal state shape an async bridge will expose later."
  (let* ((request-id (soma-gym-bridge--get request "request_id"))
         (operation (soma-gym-bridge--get request "operation"))
         validation
         params
         response)
    (cond
     ((not (and (stringp request-id) (not (string-empty-p request-id))))
      (soma-gym-bridge--response
       request-id operation "failed" '("accepted" "failed")
       nil "invalid_request_id" "request_id must be a non-empty string"))
     ((gethash request-id soma-gym-bridge--requests)
      (soma-gym-bridge--response
       request-id operation "failed" '("accepted" "failed")
       nil "duplicate_request_id" "request_id has already been submitted"))
     ((not (stringp operation))
      (setq response
            (soma-gym-bridge--response
             request-id operation "failed" '("accepted" "failed")
             nil "invalid_operation" "operation must be a string"))
      (puthash request-id response soma-gym-bridge--requests)
      response)
     ((progn
        (setq validation
              (soma-gym-bridge--validate-request-envelope
               request-id operation request))
        (eq (car validation) 'error))
      (setq response (cdr validation))
      (puthash request-id response soma-gym-bridge--requests)
      response)
     ((and soma-gym-bridge-operation-allowlist
           (not (member operation soma-gym-bridge-operation-allowlist)))
      (setq response
            (soma-gym-bridge--response
             request-id operation "failed" '("accepted" "failed")
             nil "unauthorized_operation"
             "operation is not allowed by this bridge transport profile"))
      (puthash request-id response soma-gym-bridge--requests)
      response)
     (t
      (setq params (cdr validation))
      (setq response
            (condition-case err
                (soma-gym-bridge--execute-request request-id operation params)
              (error
               (soma-gym-bridge--response
                request-id operation "failed"
                '("accepted" "running" "failed")
                nil
                "internal_error"
                (error-message-string err)))))
      (soma-gym-bridge--publish-request-response request-id response)
      response))))

(defun soma-gym-bridge--json-read (text)
  (let ((json-object-type 'alist)
        (json-array-type 'list)
        (json-key-type 'string)
        (json-false :json-false))
    (json-read-from-string text)))

(defun soma-gym-bridge--json-encode-line (object)
  (let ((json-encoding-pretty-print nil))
    (concat (json-encode object) "\n")))

(defun soma-gym-bridge--handle-jsonl-line (line)
  (condition-case err
      (soma-gym-bridge-submit-request
       (soma-gym-bridge--json-read line))
    (error
     (soma-gym-bridge--response
      nil nil "failed" '("accepted" "failed")
      nil "malformed_json" (error-message-string err)))))

(defun soma-gym-bridge--jsonl-process-pending (process)
  "Process complete JSONL request lines already buffered on PROCESS."
  (let ((pending (or (process-get process 'soma-gym-bridge-pending) ""))
        line response-line)
    (while (string-match "\\`\\([^\n]*\\)\n" pending)
      (setq line (match-string 1 pending))
      (setq pending (substring pending (match-end 0)))
      (setq response-line
            (soma-gym-bridge--json-encode-line
             (soma-gym-bridge--handle-jsonl-line line)))
      (when (process-live-p process)
        (ignore-errors
          (process-send-string process response-line))))
    (process-put process 'soma-gym-bridge-pending pending)))

(defun soma-gym-bridge--jsonl-filter (process chunk)
  (process-put process 'soma-gym-bridge-pending
               (concat (or (process-get process 'soma-gym-bridge-pending)
                           "")
                       chunk))
  (soma-gym-bridge--jsonl-process-pending process))

(defun soma-gym-bridge-start-jsonl-server (socket)
  "Start a local JSONL bridge server at Unix SOCKET."
  (soma-gym-bridge-stop-jsonl-server)
  (make-directory (file-name-directory socket) t)
  (when (file-exists-p socket)
    (delete-file socket))
  (setq soma-gym-bridge--jsonl-socket socket)
  (setq soma-gym-bridge--jsonl-server
        (make-network-process
         :name "soma-gym-bridge-jsonl"
         :server t
         :family 'local
         :service socket
         :noquery t
         :coding 'utf-8
         :filter #'soma-gym-bridge--jsonl-filter))
  (soma-gym-bridge--alist
   (cons "operation" "transport.start")
   (cons "state" "succeeded")
   (cons "socket" socket)))

(defun soma-gym-bridge-stop-jsonl-server ()
  "Stop the active JSONL bridge server, if any."
  (when (process-live-p soma-gym-bridge--jsonl-server)
    (delete-process soma-gym-bridge--jsonl-server))
  (setq soma-gym-bridge--jsonl-server nil)
  (when (and soma-gym-bridge--jsonl-socket
             (file-exists-p soma-gym-bridge--jsonl-socket))
    (delete-file soma-gym-bridge--jsonl-socket))
  (setq soma-gym-bridge--jsonl-socket nil)
  (soma-gym-bridge--alist
   (cons "operation" "transport.stop")
   (cons "state" "succeeded")))

(defun soma-gym-bridge--buffer-text-owner (buffer)
  (let ((base (buffer-base-buffer buffer)))
    (or base buffer)))

(defun soma-gym-bridge-observe-buffer (buffer)
  "Return a passive observation for BUFFER.

This function reads buffer identity and editor-local state without changing
point, mark, selected window, buffer contents, or displayed buffers."
  (with-current-buffer buffer
    (soma-gym-bridge--alist
     (cons "operation" "emacs.observe_buffer")
     (cons "state" "succeeded")
     (cons "buffer_name" (buffer-name buffer))
     (cons "text_owner_name"
           (buffer-name (soma-gym-bridge--buffer-text-owner buffer)))
     (cons "buffer_file_name" (or buffer-file-name :json-false))
     (cons "major_mode" (symbol-name major-mode))
     (cons "chars_tick" (buffer-chars-modified-tick))
     (cons "modified" (soma-gym-bridge--bool (buffer-modified-p buffer)))
     (cons "read_only" (soma-gym-bridge--bool buffer-read-only))
     (cons "point_min" (point-min))
     (cons "point_max" (point-max))
     (cons "point" (point))
     (cons "mark_active" (soma-gym-bridge--bool mark-active))
     (cons "narrowed" (soma-gym-bridge--bool (buffer-narrowed-p)))
     (cons "selected_window_buffer"
           (if (window-live-p (selected-window))
               (buffer-name (window-buffer (selected-window)))
             :json-false))
     (cons "minibuffer_active"
           (soma-gym-bridge--bool (active-minibuffer-window))))))

(defun soma-gym-bridge-observe-comint (buffer)
  "Return Comint process/input-boundary state for BUFFER."
  (with-current-buffer buffer
    (let* ((process (get-buffer-process buffer))
           (process-mark
            (and process (process-mark process)))
           (process-mark-position
            (soma-gym-bridge--json-marker-position process-mark))
           (last-input-start
            (and (boundp 'comint-last-input-start)
                 comint-last-input-start))
           (last-input-end
            (and (boundp 'comint-last-input-end)
                 comint-last-input-end))
           (last-input-start-position
            (soma-gym-bridge--json-marker-position last-input-start))
           (last-input-end-position
            (soma-gym-bridge--json-marker-position last-input-end))
           (last-input-markers-live
            (and (markerp last-input-start)
                 (markerp last-input-end)
                 (eq (marker-buffer last-input-start) buffer)
                 (eq (marker-buffer last-input-end) buffer)
                 (integerp last-input-start-position)
                 (integerp last-input-end-position)
                 (< last-input-start-position
                    last-input-end-position)))
           (process-exe
            (and process
                 (let ((path (soma-gym-bridge--process-exe process)))
                   (and (not (string-empty-p path)) path))))
           (process-rootfs-owned
            (soma-gym-bridge--rootfs-owned-tool-path-p process-exe))
           (current-input
            (if (and (integerp process-mark-position)
                     (<= process-mark-position (point-max)))
                (buffer-substring-no-properties
                 process-mark-position
                 (point-max))
              :json-false)))
      (soma-gym-bridge--alist
       (cons "operation" "comint.observe")
       (cons "state" "succeeded")
       (cons "buffer_name" (buffer-name buffer))
       (cons "major_mode" (symbol-name major-mode))
       (cons "buffer_file_name" (soma-gym-bridge--json-file-name buffer))
       (cons "chars_tick" (buffer-chars-modified-tick))
       (cons "modified" (soma-gym-bridge--bool (buffer-modified-p buffer)))
       (cons "default_directory" default-directory)
       (cons "point" (point))
       (cons "process_present" (soma-gym-bridge--bool process))
       (cons "process_live" (soma-gym-bridge--bool
                              (and process (process-live-p process))))
       (cons "process_name" (if process (process-name process) :json-false))
       (cons "process_status"
             (if process
                 (soma-gym-bridge--json-symbol (process-status process))
               :json-false))
       (cons "process_id" (or (and process (process-id process)) :json-false))
       (cons "process_exe" (or process-exe :json-false))
       (cons "process_rootfs_owned"
             (soma-gym-bridge--bool process-rootfs-owned))
       (cons "process_mark" process-mark-position)
       (cons "point_after_process_mark"
             (soma-gym-bridge--bool
              (and (integerp process-mark-position)
                   (>= (point) process-mark-position))))
       (cons "last_input_markers_live"
             (soma-gym-bridge--bool last-input-markers-live))
       (cons "last_input_start" last-input-start-position)
       (cons "last_input_end" last-input-end-position)
       (cons "last_input_text"
             (soma-gym-bridge--marker-range-text
              last-input-start last-input-end))
       (cons "current_input_text" current-input)
       (cons "buffer_tail" (soma-gym-bridge--buffer-tail 4096))
       (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
       (cons "plan" (or (getenv "SOMA_INSULA_PLAN") ""))
       (cons "selected_window_buffer"
             (if (window-live-p (selected-window))
                 (buffer-name (window-buffer (selected-window)))
               :json-false))
       (cons "minibuffer_active"
             (soma-gym-bridge--bool (active-minibuffer-window)))))))

(defun soma-gym-bridge--comint-tail-from (position &optional max-chars)
  "Return buffer text from POSITION to point max, bounded by MAX-CHARS."
  (save-restriction
    (widen)
    (let* ((end (point-max))
           (start (max (point-min)
                       (min end
                            (if (integerp position)
                                position
                              (point-min)))))
           (bounded-start
            (if (and max-chars (> (- end start) max-chars))
                (- end max-chars)
              start)))
      (buffer-substring-no-properties bounded-start end))))

(defun soma-gym-bridge--comint-submit-checks
    (buffer preconditions before)
  "Return ownership/precondition checks for Comint submit on BUFFER."
  (let* ((expected-buffer
          (soma-gym-bridge--get preconditions "buffer_name"))
         (expected-process-id
          (soma-gym-bridge--get preconditions "process_id"))
         (expected-process-exe
          (soma-gym-bridge--get preconditions "process_exe"))
         (expected-process-rootfs-owned
          (soma-gym-bridge--get preconditions "process_rootfs_owned"))
         (expected-process-mark
          (soma-gym-bridge--get preconditions "process_mark"))
         (expected-current-input
          (soma-gym-bridge--get preconditions "current_input_text"))
         (process (and (buffer-live-p buffer)
                       (get-buffer-process buffer)))
         (actual-process-id
          (and process (process-id process)))
         (actual-process-mark
          (and process
               (soma-gym-bridge--json-marker-position
                (process-mark process))))
         (actual-current-input
          (and before
               (soma-gym-bridge--get before "current_input_text")))
         (actual-process-exe
          (and before
               (soma-gym-bridge--get before "process_exe")))
         (actual-process-rootfs-owned
          (and before
               (soma-gym-bridge--get before "process_rootfs_owned"))))
    (list
     (soma-gym-bridge--check
      "target buffer is live"
      (buffer-live-p buffer)
      (format "%S" buffer))
     (soma-gym-bridge--check
      "buffer identity matches precondition"
      (and (stringp expected-buffer)
           (buffer-live-p buffer)
           (string= expected-buffer (buffer-name buffer)))
      (or expected-buffer :json-false))
     (soma-gym-bridge--check
      "buffer mode derives from comint-mode"
      (and (buffer-live-p buffer)
           (with-current-buffer buffer
             (derived-mode-p 'comint-mode)))
      (and (buffer-live-p buffer)
           (with-current-buffer buffer
             (soma-gym-bridge--json-symbol major-mode))))
     (soma-gym-bridge--check
      "Comint process is live"
      (and process (process-live-p process))
      (if process (process-status process) :json-false))
     (soma-gym-bridge--check
      "Comint process identity matches precondition"
      (and (integerp expected-process-id)
           (integerp actual-process-id)
           (= expected-process-id actual-process-id))
      (soma-gym-bridge--alist
       (cons "expected" (or expected-process-id :json-false))
       (cons "actual" (or actual-process-id :json-false))))
     (soma-gym-bridge--check
      "Comint process executable matches precondition"
      (and (stringp expected-process-exe)
           (stringp actual-process-exe)
           (string= expected-process-exe actual-process-exe))
      (soma-gym-bridge--alist
       (cons "expected" (or expected-process-exe :json-false))
       (cons "actual" (or actual-process-exe :json-false))))
     (soma-gym-bridge--check
      "Comint process executable is rootfs-owned"
      (and (eq expected-process-rootfs-owned t)
           (eq actual-process-rootfs-owned t)
           (soma-gym-bridge--rootfs-owned-tool-path-p actual-process-exe))
      (soma-gym-bridge--alist
       (cons "expected" (or expected-process-rootfs-owned :json-false))
       (cons "actual" (or actual-process-rootfs-owned :json-false))
       (cons "process_exe" (or actual-process-exe :json-false))))
     (soma-gym-bridge--check
      "Comint process mark matches precondition"
      (and (integerp expected-process-mark)
           (integerp actual-process-mark)
           (= expected-process-mark actual-process-mark))
      (soma-gym-bridge--alist
       (cons "expected" (or expected-process-mark :json-false))
       (cons "actual" (or actual-process-mark :json-false))))
     (soma-gym-bridge--check
      "current input text matches precondition"
      (and (stringp expected-current-input)
           (stringp actual-current-input)
           (string= expected-current-input actual-current-input))
      (soma-gym-bridge--alist
       (cons "expected" (or expected-current-input :json-false))
       (cons "actual" (or actual-current-input :json-false)))))))

(defun soma-gym-bridge-comint-submit-input
    (buffer input preconditions)
  "Submit INPUT to BUFFER through `comint-send-input' after checks."
  (require 'comint)
  (let* ((before (and (buffer-live-p buffer)
                      (soma-gym-bridge-observe-comint buffer)))
         (checks
          (soma-gym-bridge--comint-submit-checks
           buffer preconditions before))
         after)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "comint.submit_input")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "running" "failed"))
         (cons "route" "precondition-check")
         (cons "precondition_checks" checks)
         (cons "before" (or before :json-false))
         (cons "after" (or before :json-false))
         (cons "submitted_bytes" 0)
         (cons "used_raw_process_write" :json-false)
         (cons "error" "Comint submit preconditions failed"))
      (with-current-buffer buffer
        (let* ((process (get-buffer-process buffer))
               (process-mark (process-mark process))
               (process-position (marker-position process-mark)))
          (save-restriction
            (widen)
            (goto-char (point-max))
            (delete-region process-position (point-max))
            (insert input)
            (goto-char (point-max))
            ;; Route through Comint's command, so Comint owns input markers and
            ;; process bookkeeping instead of a raw process write.
            (let ((this-command 'comint-send-input))
              (call-interactively #'comint-send-input)))))
      (setq after (soma-gym-bridge-observe-comint buffer))
      (soma-gym-bridge--alist
       (cons "operation" "comint.submit_input")
       (cons "state" "succeeded")
       (cons "state_sequence" '("accepted" "running" "succeeded"))
       (cons "route" "comint-send-input")
       (cons "dispatch" "call-interactively")
       (cons "precondition_checks" checks)
       (cons "before" before)
       (cons "after" after)
       (cons "submitted_bytes" (string-bytes input))
       (cons "comint_input_markers"
             (soma-gym-bridge--alist
              (cons "last_input_markers_live"
                    (soma-gym-bridge--get after
                                          "last_input_markers_live"))
              (cons "last_input_start"
                    (soma-gym-bridge--get after "last_input_start"))
              (cons "last_input_end"
                    (soma-gym-bridge--get after "last_input_end"))))
       (cons "used_raw_process_write" :json-false)
       (cons "process_output"
             (or (soma-gym-bridge--get after "buffer_tail") ""))
       (cons "error" :json-false)))))

(defun soma-gym-bridge--comint-await-checks
    (buffer preconditions before)
  "Return ownership/precondition checks for awaiting Comint output."
  (let* ((expected-process-id
          (soma-gym-bridge--get preconditions "process_id"))
         (expected-process-mark
          (soma-gym-bridge--get preconditions "process_mark"))
         (expected-buffer
          (soma-gym-bridge--get preconditions "buffer_name"))
         (expected-process-exe
          (soma-gym-bridge--get preconditions "process_exe"))
         (expected-process-rootfs-owned
          (soma-gym-bridge--get preconditions "process_rootfs_owned"))
         (process (and (buffer-live-p buffer)
                       (get-buffer-process buffer)))
         (actual-process-id
          (and process (process-id process)))
         (actual-process-mark
          (and process
               (soma-gym-bridge--json-marker-position
                (process-mark process))))
         (last-input-end
          (and before
               (soma-gym-bridge--get before "last_input_end")))
         (actual-process-exe
          (and before
               (soma-gym-bridge--get before "process_exe")))
         (actual-process-rootfs-owned
          (and before
               (soma-gym-bridge--get before "process_rootfs_owned"))))
    (list
     (soma-gym-bridge--check
      "target buffer is live"
      (buffer-live-p buffer)
      (format "%S" buffer))
     (soma-gym-bridge--check
      "buffer identity matches precondition"
      (and (stringp expected-buffer)
           (buffer-live-p buffer)
           (string= expected-buffer (buffer-name buffer)))
      (or expected-buffer :json-false))
     (soma-gym-bridge--check
      "buffer mode derives from comint-mode"
      (and (buffer-live-p buffer)
           (with-current-buffer buffer
             (derived-mode-p 'comint-mode)))
      (and (buffer-live-p buffer)
           (with-current-buffer buffer
             (soma-gym-bridge--json-symbol major-mode))))
     (soma-gym-bridge--check
      "Comint process is live"
      (and process (process-live-p process))
      (if process (process-status process) :json-false))
     (soma-gym-bridge--check
      "Comint process identity matches precondition"
      (and (integerp expected-process-id)
           (integerp actual-process-id)
           (= expected-process-id actual-process-id))
      (soma-gym-bridge--alist
       (cons "expected" (or expected-process-id :json-false))
       (cons "actual" (or actual-process-id :json-false))))
     (soma-gym-bridge--check
      "Comint process executable matches precondition"
      (and (stringp expected-process-exe)
           (stringp actual-process-exe)
           (string= expected-process-exe actual-process-exe))
      (soma-gym-bridge--alist
       (cons "expected" (or expected-process-exe :json-false))
       (cons "actual" (or actual-process-exe :json-false))))
     (soma-gym-bridge--check
      "Comint process executable is rootfs-owned"
      (and (eq expected-process-rootfs-owned t)
           (eq actual-process-rootfs-owned t)
           (soma-gym-bridge--rootfs-owned-tool-path-p actual-process-exe))
      (soma-gym-bridge--alist
       (cons "expected" (or expected-process-rootfs-owned :json-false))
       (cons "actual" (or actual-process-rootfs-owned :json-false))
       (cons "process_exe" (or actual-process-exe :json-false))))
     (soma-gym-bridge--check
      "Comint output boundary is valid for the current input"
      (and (integerp expected-process-mark)
           (integerp actual-process-mark)
           (<= expected-process-mark actual-process-mark)
           (or (not (integerp last-input-end))
               (>= expected-process-mark last-input-end)))
      (soma-gym-bridge--alist
       (cons "expected_process_mark" (or expected-process-mark
                                         :json-false))
       (cons "actual_process_mark" (or actual-process-mark :json-false))
       (cons "last_input_end" (or last-input-end :json-false)))))))

(defun soma-gym-bridge-comint-await-output
    (buffer needle preconditions timeout-seconds)
  "Await NEEDLE in BUFFER output from the checked Comint boundary."
  (let* ((before (and (buffer-live-p buffer)
                      (soma-gym-bridge-observe-comint buffer)))
         (checks
          (soma-gym-bridge--comint-await-checks
           buffer preconditions before))
         (process (and (buffer-live-p buffer)
                       (get-buffer-process buffer)))
         (boundary (soma-gym-bridge--get preconditions "process_mark"))
         (deadline (+ (float-time) timeout-seconds))
         matched partial after)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "comint.await_output")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "running" "failed"))
         (cons "route" "precondition-check")
         (cons "precondition_checks" checks)
         (cons "before" (or before :json-false))
         (cons "after" (or before :json-false))
         (cons "matched" :json-false)
         (cons "matched_output" :json-false)
         (cons "partial_output" :json-false)
         (cons "timeout_seconds" timeout-seconds)
         (cons "error" "Comint await preconditions failed"))
      (while (and (not matched)
                  (< (float-time) deadline)
                  process
                  (process-live-p process))
        (with-current-buffer buffer
          (setq partial (soma-gym-bridge--comint-tail-from boundary 4096))
          (setq matched
                (if (and (stringp needle)
                         (not (string-empty-p needle)))
                    (string-match-p (regexp-quote needle) partial)
                  (> (length partial) 0))))
        (unless matched
          (accept-process-output process 0.02)))
      (with-current-buffer buffer
        (setq partial (soma-gym-bridge--comint-tail-from boundary 4096))
        (setq matched
              (or matched
                  (if (and (stringp needle)
                           (not (string-empty-p needle)))
                      (string-match-p (regexp-quote needle) partial)
                    (> (length partial) 0)))))
      (setq after (soma-gym-bridge-observe-comint buffer))
      (if matched
          (soma-gym-bridge--alist
           (cons "operation" "comint.await_output")
           (cons "state" "succeeded")
           (cons "state_sequence" '("accepted" "running" "succeeded"))
           (cons "route" "process-output-await")
           (cons "precondition_checks" checks)
           (cons "before" before)
           (cons "after" after)
           (cons "matched" t)
           (cons "needle" (or needle :json-false))
           (cons "matched_output" partial)
           (cons "partial_output" partial)
           (cons "timeout_seconds" timeout-seconds)
           (cons "error" :json-false))
        (soma-gym-bridge--alist
         (cons "operation" "comint.await_output")
         (cons "state" "outcome_unknown")
         (cons "state_sequence" '("accepted" "running" "outcome_unknown"))
         (cons "route" "process-output-await")
         (cons "precondition_checks" checks)
         (cons "before" before)
         (cons "after" after)
         (cons "matched" :json-false)
         (cons "needle" (or needle :json-false))
         (cons "matched_output" :json-false)
         (cons "partial_output" partial)
         (cons "timeout_seconds" timeout-seconds)
         (cons "error" "Comint output did not match before timeout"))))))

(defun soma-gym-bridge--comint-interrupt-checks
    (buffer preconditions before)
  "Return ownership/precondition checks for interrupting BUFFER."
  (let* ((expected-buffer
          (soma-gym-bridge--get preconditions "buffer_name"))
         (expected-process-id
          (soma-gym-bridge--get preconditions "process_id"))
         (expected-process-exe
          (soma-gym-bridge--get preconditions "process_exe"))
         (expected-process-rootfs-owned
          (soma-gym-bridge--get preconditions "process_rootfs_owned"))
         (expected-process-mark
          (soma-gym-bridge--get preconditions "process_mark"))
         (expected-current-input
          (soma-gym-bridge--get preconditions "current_input_text"))
         (process (and (buffer-live-p buffer)
                       (get-buffer-process buffer)))
         (actual-process-id
          (and process (process-id process)))
         (actual-process-mark
          (and process
               (soma-gym-bridge--json-marker-position
                (process-mark process))))
         (actual-current-input
          (and before
               (soma-gym-bridge--get before "current_input_text")))
         (actual-process-exe
          (and before
               (soma-gym-bridge--get before "process_exe")))
         (actual-process-rootfs-owned
          (and before
               (soma-gym-bridge--get before "process_rootfs_owned"))))
    (append
     (list
      (soma-gym-bridge--check
       "target buffer is live"
       (buffer-live-p buffer)
       (format "%S" buffer))
      (soma-gym-bridge--check
       "buffer identity matches precondition"
       (and (stringp expected-buffer)
            (buffer-live-p buffer)
            (string= expected-buffer (buffer-name buffer)))
       (or expected-buffer :json-false))
      (soma-gym-bridge--check
       "buffer mode derives from comint-mode"
       (and (buffer-live-p buffer)
            (with-current-buffer buffer
              (derived-mode-p 'comint-mode)))
       (and (buffer-live-p buffer)
            (with-current-buffer buffer
              (soma-gym-bridge--json-symbol major-mode))))
      (soma-gym-bridge--check
       "Comint process is live"
       (and process (process-live-p process))
       (if process (process-status process) :json-false))
      (soma-gym-bridge--check
       "Comint process identity matches precondition"
       (and (integerp expected-process-id)
            (integerp actual-process-id)
            (= expected-process-id actual-process-id))
       (soma-gym-bridge--alist
        (cons "expected" (or expected-process-id :json-false))
        (cons "actual" (or actual-process-id :json-false))))
      (soma-gym-bridge--check
       "Comint process executable matches precondition"
       (and (stringp expected-process-exe)
            (stringp actual-process-exe)
            (string= expected-process-exe actual-process-exe))
       (soma-gym-bridge--alist
        (cons "expected" (or expected-process-exe :json-false))
        (cons "actual" (or actual-process-exe :json-false))))
      (soma-gym-bridge--check
       "Comint process executable is rootfs-owned"
       (and (eq expected-process-rootfs-owned t)
            (eq actual-process-rootfs-owned t)
            (soma-gym-bridge--rootfs-owned-tool-path-p actual-process-exe))
       (soma-gym-bridge--alist
        (cons "expected" (or expected-process-rootfs-owned :json-false))
        (cons "actual" (or actual-process-rootfs-owned :json-false))
        (cons "process_exe" (or actual-process-exe :json-false))))
      (soma-gym-bridge--check
       "Comint process mark matches precondition"
       (and (integerp expected-process-mark)
            (integerp actual-process-mark)
            (= expected-process-mark actual-process-mark))
       (soma-gym-bridge--alist
        (cons "expected" (or expected-process-mark :json-false))
        (cons "actual" (or actual-process-mark :json-false)))))
     (when (stringp expected-current-input)
       (list
        (soma-gym-bridge--check
         "current input text matches precondition"
         (and (stringp actual-current-input)
              (string= expected-current-input actual-current-input))
         (soma-gym-bridge--alist
          (cons "expected" expected-current-input)
          (cons "actual" (or actual-current-input :json-false)))))))))

(defun soma-gym-bridge-comint-interrupt
    (buffer preconditions)
  "Interrupt BUFFER's current Comint subprocess after ownership checks."
  (require 'comint)
  (let* ((before (and (buffer-live-p buffer)
                      (soma-gym-bridge-observe-comint buffer)))
         (checks
          (soma-gym-bridge--comint-interrupt-checks
           buffer preconditions before))
         after dispatch-error)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "comint.interrupt")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "running" "failed"))
         (cons "route" "precondition-check")
         (cons "precondition_checks" checks)
         (cons "before" (or before :json-false))
         (cons "after" (or before :json-false))
         (cons "interrupt_command" "comint-interrupt-subjob")
         (cons "interrupt_dispatched" :json-false)
         (cons "used_raw_process_write" :json-false)
         (cons "bridge_used_raw_process_control" :json-false)
         (cons "error" "Comint interrupt preconditions failed"))
      (with-current-buffer buffer
        (condition-case err
            (let ((this-command 'comint-interrupt-subjob))
              ;; Use Comint's own command so buffer/process bookkeeping remains
              ;; package-owned instead of sending a bridge-level signal.
              (call-interactively #'comint-interrupt-subjob))
          (error
           (setq dispatch-error (error-message-string err)))))
      (setq after (soma-gym-bridge-observe-comint buffer))
      (if dispatch-error
          (soma-gym-bridge--alist
           (cons "operation" "comint.interrupt")
           (cons "state" "failed")
           (cons "state_sequence" '("accepted" "running" "failed"))
           (cons "route" "comint-interrupt-subjob")
           (cons "dispatch" "call-interactively")
           (cons "precondition_checks" checks)
           (cons "before" before)
           (cons "after" after)
           (cons "interrupt_command" "comint-interrupt-subjob")
           (cons "interrupt_dispatched" :json-false)
           (cons "process_live_after"
                 (soma-gym-bridge--get after "process_live"))
           (cons "used_raw_process_write" :json-false)
           (cons "bridge_used_raw_process_control" :json-false)
           (cons "error" dispatch-error))
        (soma-gym-bridge--alist
         (cons "operation" "comint.interrupt")
         (cons "state" "succeeded")
         (cons "state_sequence" '("accepted" "running" "succeeded"))
         (cons "route" "comint-interrupt-subjob")
         (cons "dispatch" "call-interactively")
         (cons "precondition_checks" checks)
         (cons "before" before)
         (cons "after" after)
         (cons "interrupt_command" "comint-interrupt-subjob")
         (cons "interrupt_dispatched" t)
         (cons "process_id"
               (soma-gym-bridge--get before "process_id"))
         (cons "process_status_before"
               (soma-gym-bridge--get before "process_status"))
         (cons "process_status_after"
               (soma-gym-bridge--get after "process_status"))
         (cons "process_live_after"
               (soma-gym-bridge--get after "process_live"))
         (cons "side_effect_reconciliation"
               "external_verifier_required")
         (cons "used_raw_process_write" :json-false)
         (cons "bridge_used_raw_process_control" :json-false)
         (cons "error" :json-false))))))

(defun soma-gym-bridge--precondition-checks (buffer command preconditions)
  (let* ((expected-buffer
          (soma-gym-bridge--get preconditions "buffer_name"))
         (expected-selected
          (soma-gym-bridge--get preconditions "selected_window_buffer"))
         (expected-tick
          (soma-gym-bridge--get preconditions "chars_tick"))
         (expected-point
          (soma-gym-bridge--get preconditions "point"))
         (expected-mark-active
          (soma-gym-bridge--get preconditions "mark_active"))
         (expected-narrowed
          (soma-gym-bridge--get preconditions "narrowed"))
         (expected-read-only
          (soma-gym-bridge--get preconditions "read_only"))
         (expected-region-begin
          (soma-gym-bridge--get preconditions "region_begin"))
         (expected-region-end
          (soma-gym-bridge--get preconditions "region_end"))
         (expected-minibuffer-active
          (soma-gym-bridge--get preconditions "minibuffer_active"))
         (selected-buffer
          (and (window-live-p (selected-window))
               (window-buffer (selected-window)))))
    (append
     (list
      (soma-gym-bridge--check
       "target buffer is live"
       (buffer-live-p buffer)
       (format "%S" buffer))
      (soma-gym-bridge--check
       "command is allowlisted"
       (memq command soma-gym-bridge-command-allowlist)
       (soma-gym-bridge--json-symbol command))
      (soma-gym-bridge--check
       "command is interactive"
       (commandp command)
       (soma-gym-bridge--json-symbol command)))
     (when (buffer-live-p buffer)
       (with-current-buffer buffer
         (append
          (list
           (soma-gym-bridge--check
            "buffer identity matches precondition"
            (or (not expected-buffer)
                (string= expected-buffer (buffer-name buffer)))
            (format "expected=%S actual=%S"
                    expected-buffer (buffer-name buffer)))
           (soma-gym-bridge--check
            "selected window displays target buffer"
            (or (not expected-selected)
                (and selected-buffer
                     (eq selected-buffer buffer)
                     (string= expected-selected (buffer-name buffer))))
            (format "expected=%S actual=%S"
                    expected-selected
                    (and selected-buffer (buffer-name selected-buffer))))
           (soma-gym-bridge--check
            "character revision matches precondition"
            (or (not expected-tick)
                (= expected-tick (buffer-chars-modified-tick)))
            (format "expected=%S actual=%S"
                    expected-tick (buffer-chars-modified-tick)))
           (soma-gym-bridge--check
            "point matches precondition"
            (or (not expected-point)
                (= expected-point (point)))
            (format "expected=%S actual=%S" expected-point (point)))
           (soma-gym-bridge--check
            "minibuffer state matches precondition"
            (or (not (soma-gym-bridge--has-key-p
                      preconditions "minibuffer_active"))
                (eq (soma-gym-bridge--bool (active-minibuffer-window))
                    expected-minibuffer-active))
            (format "expected=%S actual=%S"
                    expected-minibuffer-active
                    (soma-gym-bridge--bool (active-minibuffer-window)))))
          (when (soma-gym-bridge--has-key-p preconditions "mark_active")
            (list
             (soma-gym-bridge--check
              "region active state matches precondition"
              (eq (soma-gym-bridge--bool mark-active)
                  expected-mark-active)
              (format "expected=%S actual=%S"
                      expected-mark-active
                      (soma-gym-bridge--bool mark-active)))))
          (when (soma-gym-bridge--has-key-p preconditions "narrowed")
            (list
             (soma-gym-bridge--check
              "narrowing state matches precondition"
              (eq (soma-gym-bridge--bool (buffer-narrowed-p))
                  expected-narrowed)
              (format "expected=%S actual=%S"
                      expected-narrowed
                      (soma-gym-bridge--bool (buffer-narrowed-p))))))
          (when (soma-gym-bridge--has-key-p preconditions "read_only")
            (list
             (soma-gym-bridge--check
              "read-only state matches precondition"
              (eq (soma-gym-bridge--bool buffer-read-only)
                  expected-read-only)
              (format "expected=%S actual=%S"
                      expected-read-only
                      (soma-gym-bridge--bool buffer-read-only)))))
          (when (and (soma-gym-bridge--present-p expected-region-begin)
                     (soma-gym-bridge--present-p expected-region-end))
            (list
             (soma-gym-bridge--check
              "region bounds match precondition"
              (and mark-active
                   (= expected-region-begin (region-beginning))
                   (= expected-region-end (region-end)))
              (format "expected=%S..%S actual=%S..%S"
                      expected-region-begin expected-region-end
                      (and mark-active (region-beginning))
                      (and mark-active (region-end)))))))))
     (soma-gym-bridge--magit-precondition-checks buffer preconditions))))

(defun soma-gym-bridge-command-dispatch (buffer command preconditions)
  "Invoke COMMAND in BUFFER through the checked command-dispatch route.

PRECONDITIONS is a string-keyed alist.  The first bridge slice is synchronous
and batch-safe, so it returns the final operation state directly."
  (let ((checks (soma-gym-bridge--precondition-checks
                 buffer command preconditions))
        before after error-message
        lsp-before lsp-after lsp-requests
        lsp-workspace-edit-applied
        lsp-workspace-edit-raw
        lsp-workspace-edit-hash)
    (if (not (soma-gym-bridge--all-checks-pass-p checks))
        (soma-gym-bridge--alist
         (cons "operation" "command.dispatch")
         (cons "state" "failed")
         (cons "state_sequence" '("accepted" "failed"))
         (cons "command" (soma-gym-bridge--json-symbol command))
         (cons "route" "precondition-check")
         (cons "precondition_checks" checks))
      (setq before (soma-gym-bridge-observe-buffer buffer))
      (when (and (eq command 'lsp-rename)
                 (buffer-live-p buffer)
                 (fboundp 'soma-gym-bridge-observe-lsp-source))
        (setq lsp-before
              (condition-case err
                  (soma-gym-bridge-observe-lsp-source buffer 1600)
                (error
                 (soma-gym-bridge--alist
                  (cons "state" "failed")
                  (cons "error" (error-message-string err)))))))
      (condition-case err
          (with-current-buffer buffer
            (let ((this-command command))
              (if (and (eq command 'lsp-rename)
                       (fboundp 'lsp-request)
                       (fboundp 'lsp--apply-workspace-edit))
                  (let ((orig-lsp-request (symbol-function 'lsp-request))
                        (orig-apply-workspace-edit
                         (symbol-function 'lsp--apply-workspace-edit)))
                    (cl-letf (((symbol-function 'lsp-request)
                               (lambda (method params &rest args)
                                 (push
                                  (soma-gym-bridge--alist
                                   (cons "method" (or method ""))
                                   (cons "buffer" (buffer-name (current-buffer)))
                                   (cons "document_version"
                                         (if (boundp 'lsp--cur-version)
                                             lsp--cur-version
                                           :json-false)))
                                  lsp-requests)
                                 (apply orig-lsp-request method params args)))
                              ((symbol-function 'lsp--apply-workspace-edit)
                               (lambda (workspace-edit &optional operation)
                                 (setq lsp-workspace-edit-raw
                                       (format "%S" workspace-edit))
                                 (setq lsp-workspace-edit-hash
                                       (secure-hash
                                        'sha256 lsp-workspace-edit-raw))
                                 (prog1
                                     (funcall orig-apply-workspace-edit
                                              workspace-edit operation)
                                   (setq lsp-workspace-edit-applied t)))))
                      (command-execute command 'record)))
                (command-execute command 'record))))
        (error
         (setq error-message (error-message-string err))))
      (setq after
            (and (buffer-live-p buffer)
                 (soma-gym-bridge-observe-buffer buffer)))
      (when lsp-before
        (setq lsp-after
              (and (buffer-live-p buffer)
                   (condition-case err
                     (soma-gym-bridge-observe-lsp-source-passive buffer 1600)
                     (error
                      (soma-gym-bridge--alist
                       (cons "state" "failed")
                       (cons "error" (error-message-string err))))))))
      (append
       (soma-gym-bridge--alist
        (cons "operation" "command.dispatch")
        (cons "state" (if error-message "failed" "succeeded"))
        (cons "state_sequence"
              (if error-message
                  '("accepted" "running" "failed")
                '("accepted" "running" "succeeded")))
        (cons "command" (soma-gym-bridge--json-symbol command))
        (cons "route" "command-execute")
        (cons "this_command" (soma-gym-bridge--json-symbol command))
        (cons "precondition_checks" checks)
        (cons "before" before)
        (cons "after" (or after :json-false))
        (cons "error" (or error-message :json-false)))
       (when (eq command 'lsp-rename)
         (let ((requests (nreverse lsp-requests)))
           (soma-gym-bridge--alist
            (cons "lsp_request_methods"
                  (delq nil
                        (mapcar
                         (lambda (request)
                           (soma-gym-bridge--get request "method"))
                         requests)))
            (cons "lsp_requests" requests)
            (cons "lsp_workspace_edit_applied"
                  (soma-gym-bridge--bool lsp-workspace-edit-applied))
            (cons "lsp_workspace_edit_sha256"
                  (or lsp-workspace-edit-hash :json-false))
            (cons "lsp_workspace_edit_excerpt"
                  (if lsp-workspace-edit-raw
                      (substring lsp-workspace-edit-raw
                                 0
                                 (min 2400
                                      (length lsp-workspace-edit-raw)))
                    :json-false))
            (cons "lsp_before" (or lsp-before :json-false))
            (cons "lsp_after" (or lsp-after :json-false)))))))))

(defun soma-gym-bridge-reject-direct-call (operation reason)
  "Return a bridge failure for an unchecked direct-call attempt."
  (soma-gym-bridge--alist
   (cons "operation" "direct-call")
   (cons "state" "failed")
   (cons "state_sequence" '("accepted" "failed"))
   (cons "requested_operation" (soma-gym-bridge--json-symbol operation))
   (cons "reason" reason)))

(provide 'soma-gym-bridge)

;;; soma-gym-bridge.el ends here
