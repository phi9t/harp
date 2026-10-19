;;; run-benchmark.el --- Bridge transport benchmark for the Emacs gym -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'comint)
(require 'dired)
(require 'dired-aux)
(require 'eieio)
(require 'json)
(require 'seq)
(require 'subr-x)

(add-to-list
 'load-path
 (expand-file-name "../../elisp"
                   (file-name-directory (or load-file-name buffer-file-name))))
(require 'soma-gym-bridge)

(defvar soma-bridge-benchmark-output-dir nil)
(defvar soma-bridge-benchmark-results nil)

(defconst soma-bridge-bench--first-profile-names
  '("core-v0" "coding-v0" "terminal-v0" "analysis-v0"
    "reports-v0" "vcs-files-v0"))

(defconst soma-bridge-bench--expected-stable-profile-operations
  '(("core-v0"
     "api.list_capabilities" "api.describe_operation"
     "emacs.observe_session" "emacs.observe_buffer"
	     "emacs.observe_frames" "buffer.select"
	     "emacs.observe_interaction" "request.status"
	     "request.await")
    ("coding-v0"
     "project.observe" "project.search" "buffer.read"
     "buffer.replace_range" "buffer.apply_patch" "buffer.save"
     "completion.observe" "completion.accept" "lsp.observe_source"
     "lsp.observe_diagnostics" "lsp.rename_symbol")
    ("terminal-v0"
     "comint.observe" "comint.submit_input" "comint.await_output"
     "comint.interrupt")
    ("analysis-v0"
     "jupyter.observe_source" "jupyter.attach_existing"
     "jupyter.eval_region" "jupyter.await_request"
     "jupyter.observe_display")
    ("reports-v0"
     "org.observe_tree" "org.update_heading" "org.edit_src_block"
     "org.exit_src_edit" "org.execute_babel_block" "org.export_html")
    ("vcs-files-v0"
     "magit.observe_sections" "magit.stage_hunk" "magit.stage_file"
     "dired.observe" "dired.rename_no_overwrite"
     "dired.delete_marked")))

(defvar magit-auto-revert-mode)
(defvar magit-display-buffer-function)
(defvar magit-display-buffer-noselect)
(defvar magit-post-stage-hook)
(defvar magit-refresh-buffer-hook)

(defvar soma-bridge-bench--magit-post-stage-runs nil)
(defvar soma-bridge-bench--magit-refresh-runs nil)

(defun soma-bridge-bench--alist (&rest pairs)
  pairs)

(defun soma-bridge-bench--bool (value)
  (if value t :json-false))

(defun soma-bridge-bench--get (alist key)
  (cond
   ((hash-table-p alist)
    (gethash key alist))
   ((listp alist)
    (alist-get key alist nil nil #'string=))))

(defun soma-bridge-bench--without-key (alist key)
  (cl-remove-if
   (lambda (entry)
     (string= (car entry) key))
   alist))

(defun soma-bridge-bench--check (name passed detail)
  (soma-bridge-bench--alist
   (cons "name" name)
   (cons "passed" (soma-bridge-bench--bool passed))
   (cons "detail" detail)))

(defun soma-bridge-bench--all-checks-pass-p (checks)
  (and (consp checks)
       (cl-every
        (lambda (check)
          (eq (soma-bridge-bench--get check "passed") t))
        checks)))

(defun soma-bridge-bench--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-bridge-bench--read-json-file (path)
  (when (and path (not (string-empty-p path)) (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-bridge-bench--plan ()
  (soma-bridge-bench--read-json-file (or (getenv "SOMA_INSULA_PLAN") "")))

(defun soma-bridge-bench--make-buffer (name text)
  (let ((buffer (get-buffer-create name)))
    (with-current-buffer buffer
      (erase-buffer)
      (insert text)
      (goto-char (point-min))
      (set-buffer-modified-p nil))
    (set-window-buffer (selected-window) buffer)
    buffer))

(defun soma-bridge-bench--preconditions (buffer)
  (with-current-buffer buffer
    (soma-bridge-bench--alist
     (cons "buffer_name" (buffer-name buffer))
     (cons "selected_window_buffer" (buffer-name buffer))
     (cons "chars_tick" (buffer-chars-modified-tick))
     (cons "point" (point))
     (cons "mark_active" :json-false)
     (cons "narrowed" (soma-bridge-bench--bool (buffer-narrowed-p)))
     (cons "read_only" (soma-bridge-bench--bool buffer-read-only))
     (cons "minibuffer_active" :json-false))))

(defun soma-bridge-bench--buffer-edit-preconditions
    (observed expected-text)
  (soma-bridge-bench--alist
   (cons "buffer_name" (soma-bridge-bench--get observed "buffer_name"))
   (cons "text_owner_name"
         (soma-bridge-bench--get observed "text_owner_name"))
   (cons "buffer_file_name"
         (soma-bridge-bench--get observed "buffer_file_name"))
   (cons "chars_tick" (soma-bridge-bench--get observed "chars_tick"))
   (cons "expected_text" expected-text)
   (cons "point" (soma-bridge-bench--get observed "point"))
   (cons "selected_window_buffer"
         (soma-bridge-bench--get observed "selected_window_buffer"))
   (cons "modified" (soma-bridge-bench--get observed "modified"))
   (cons "read_only" (soma-bridge-bench--get observed "read_only"))
   (cons "minibuffer_active"
         (soma-bridge-bench--get observed "minibuffer_active"))
   (cons "mark_active" (soma-bridge-bench--get observed "mark_active"))
   (cons "narrowed" (soma-bridge-bench--get observed "narrowed"))))

(defun soma-bridge-bench--org-preconditions
    (buffer observe-result heading &optional source-block)
  (let ((preconditions
         (soma-bridge-bench--alist
          (cons "buffer_name" (buffer-name buffer))
          (cons "text_owner_name"
                (soma-bridge-bench--get observe-result "text_owner_name"))
          (cons "buffer_file_name"
                (soma-bridge-bench--get observe-result "buffer_file_name"))
          (cons "chars_tick"
                (soma-bridge-bench--get observe-result "chars_tick"))
          (cons "point" (soma-bridge-bench--get observe-result "point"))
          (cons "modified" (soma-bridge-bench--get observe-result
                                                    "modified"))
          (cons "read_only" (soma-bridge-bench--get observe-result
                                                    "read_only"))
          (cons "minibuffer_active"
                (soma-bridge-bench--get observe-result
                                        "minibuffer_active"))
          (cons "mark_active"
                (soma-bridge-bench--get observe-result "mark_active"))
          (cons "narrowed"
                (soma-bridge-bench--get observe-result "narrowed"))
          (cons "org_node" heading)
          (cons "structure_fingerprint"
                (soma-bridge-bench--get observe-result
                                        "structure_fingerprint")))))
    (if source-block
        (append preconditions
                (list (cons "source_block" source-block)))
      preconditions)))

(defun soma-bridge-bench--buffer-full-text (buffer)
  (with-current-buffer buffer
    (save-restriction
      (widen)
      (buffer-substring-no-properties (point-min) (point-max)))))

(defun soma-bridge-bench--write-file (path text)
  (make-directory (file-name-directory path) t)
  (with-temp-file path
    (insert text)))

(defun soma-bridge-bench--read-file (path)
  (when (file-exists-p path)
    (with-temp-buffer
      (insert-file-contents path)
      (buffer-substring-no-properties (point-min) (point-max)))))

(defun soma-bridge-bench--file-lines (path)
  (if (not (file-exists-p path))
      nil
    (split-string (string-trim (or (soma-bridge-bench--read-file path) ""))
                  "\n" t)))

(defun soma-bridge-bench--has-exact-line-p (text line)
  "Return non-nil if TEXT contains LINE as a complete line."
  (member line (split-string (or text "") "\n" t)))

(defun soma-bridge-bench--file-sha256 (path)
  (when (file-exists-p path)
    (with-temp-buffer
      (set-buffer-multibyte nil)
      (insert-file-contents-literally path)
      (secure-hash 'sha256 (buffer-string)))))

(defun soma-bridge-bench--file-signature (path)
  (if (not (file-exists-p path))
      (soma-bridge-bench--alist
       (cons "exists" :json-false)
       (cons "path" path))
    (let ((attrs (file-attributes path 'integer)))
      (soma-bridge-bench--alist
       (cons "exists" t)
       (cons "path" path)
       (cons "truename" (file-truename path))
       (cons "size" (nth 7 attrs))
       (cons "mode" (nth 8 attrs))
       (cons "inode" (nth 10 attrs))
       (cons "device" (nth 11 attrs))
       (cons "sha256" (soma-bridge-bench--file-sha256 path))))))

(defun soma-bridge-bench--call-git (directory &rest args)
  (let ((default-directory (file-name-as-directory directory)))
    (with-temp-buffer
      (let ((status (apply #'call-process "git" nil t nil args)))
        (soma-bridge-bench--alist
         (cons "status" status)
         (cons "output" (buffer-substring-no-properties
                         (point-min) (point-max))))))))

(defun soma-bridge-bench--git (repo &rest args)
  (let* ((result (apply #'soma-bridge-bench--call-git repo args))
         (status (soma-bridge-bench--get result "status"))
         (output (soma-bridge-bench--get result "output")))
    (unless (equal status 0)
      (error "git %S failed with %S: %s" args status output))
    output))

(defun soma-bridge-bench--git-lines (repo &rest args)
  (sort (split-string (string-trim (apply #'soma-bridge-bench--git repo args))
                      "\n" t)
        #'string<))

(defun soma-bridge-bench--precondition-check-named (response name)
  (let* ((result (soma-bridge-bench--get response "result"))
         (checks (and (listp result)
                      (soma-bridge-bench--get result "precondition_checks"))))
    (and (listp checks)
         (seq-find
          (lambda (check)
            (string= (soma-bridge-bench--get check "name") name))
          checks))))

(defun soma-bridge-bench-append-token ()
  (interactive)
  (goto-char (point-max))
  (insert " done"))

(defun soma-bridge-bench--observe-request (id buffer)
  (soma-bridge-bench--alist
   (cons "request_id" id)
   (cons "operation" "emacs.observe_buffer")
   (cons "params"
         (soma-bridge-bench--alist
          (cons "buffer_name" (buffer-name buffer))))))

(defun soma-bridge-bench--command-request (id buffer command)
  (soma-bridge-bench--alist
   (cons "request_id" id)
   (cons "operation" "command.dispatch")
   (cons "params"
         (soma-bridge-bench--alist
          (cons "buffer_name" (buffer-name buffer))
          (cons "command" (symbol-name command))
          (cons "preconditions"
                (soma-bridge-bench--preconditions buffer))))))

(defun soma-bridge-bench--buffer-read-request (id buffer start end)
  (soma-bridge-bench--alist
   (cons "request_id" id)
   (cons "operation" "buffer.read")
   (cons "params"
         (soma-bridge-bench--alist
          (cons "buffer_name" (buffer-name buffer))
          (cons "start" start)
          (cons "end" end)))))

(defun soma-bridge-bench--buffer-replace-request
    (id buffer start end text preconditions)
  (soma-bridge-bench--alist
   (cons "request_id" id)
   (cons "operation" "buffer.replace_range")
   (cons "params"
         (soma-bridge-bench--alist
          (cons "buffer_name" (buffer-name buffer))
          (cons "start" start)
          (cons "end" end)
          (cons "text" text)
          (cons "preconditions" preconditions)))))

(defun soma-bridge-bench--buffer-apply-patch-request
    (id buffer patch preconditions)
  (soma-bridge-bench--alist
   (cons "api_version" "soma-emacs-agent-v0")
   (cons "request_id" id)
   (cons "operation" "buffer.apply_patch")
   (cons "capability" "reversible_editor_edit")
   (cons "preconditions" preconditions)
   (cons "params"
         (soma-bridge-bench--alist
          (cons "buffer_name" (buffer-name buffer))
          (cons "patch" patch)))))

(defun soma-bridge-bench--completion-accept-request
    (id buffer start end text preconditions)
  (soma-bridge-bench--alist
   (cons "request_id" id)
   (cons "operation" "completion.accept")
   (cons "params"
         (soma-bridge-bench--alist
          (cons "buffer_name" (buffer-name buffer))
          (cons "start" start)
          (cons "end" end)
          (cons "text" text)
          (cons "preconditions" preconditions)))))

(defun soma-bridge-bench--completion-observe-request
    (id buffer &optional max-candidates)
  (soma-bridge-bench--api-request
   id "completion.observe"
   (soma-bridge-bench--alist
    (cons "buffer_name" (buffer-name buffer))
    (cons "max_candidates" (or max-candidates 8))
    (cons "prefix_chars" 80)
    (cons "suffix_chars" 80))))

(defun soma-bridge-bench--buffer-save-request (id buffer preconditions)
  (soma-bridge-bench--alist
   (cons "request_id" id)
   (cons "operation" "buffer.save")
   (cons "params"
         (soma-bridge-bench--alist
          (cons "buffer_name" (buffer-name buffer))
          (cons "preconditions" preconditions)))))

(defun soma-bridge-bench--episode-finish-request
    (id required-requests final-buffer-checks)
  (soma-bridge-bench--alist
   (cons "request_id" id)
   (cons "operation" "episode.finish")
   (cons "params"
         (soma-bridge-bench--alist
          (cons "required_requests" required-requests)
          (cons "final_buffer_checks" final-buffer-checks)))))

(defun soma-bridge-bench--api-request (id operation &optional params)
  (soma-bridge-bench--alist
   (cons "request_id" id)
   (cons "operation" operation)
   (cons "params" (or params nil))))

(defun soma-bridge-bench--find-operation (manifest operation)
  (seq-find
   (lambda (entry)
     (string= (soma-bridge-bench--get entry "operation") operation))
   (soma-bridge-bench--get manifest "operations")))

(defun soma-bridge-bench--first-profile-manifest-gate (manifest)
  "Return profile-completeness gate evidence for MANIFEST."
  (let ((operations (or (soma-bridge-bench--get manifest "operations") nil))
        no-stable-profiles
        missing-expected-stable-operations
        planned-operations
        stable-unimplemented-operations
        stable-disabled-operations
        stable-missing-contract-operations)
    (dolist (profile soma-bridge-bench--first-profile-names)
      (let* ((profile-operations
              (seq-filter
               (lambda (entry)
                 (string= (or (soma-bridge-bench--get entry "profile") "")
                          profile))
               operations))
             (stable-operations
              (seq-filter
               (lambda (entry)
                 (string= (or (soma-bridge-bench--get entry "maturity") "")
                          "stable"))
               profile-operations))
             (stable-operation-names
              (mapcar
               (lambda (entry)
                 (soma-bridge-bench--get entry "operation"))
               stable-operations))
             (expected-operations
              (cdr (assoc profile
                          soma-bridge-bench--expected-stable-profile-operations))))
        (unless stable-operations
          (push profile no-stable-profiles))
        (dolist (operation expected-operations)
          (unless (member operation stable-operation-names)
            (push (format "%s:%s" profile operation)
                  missing-expected-stable-operations)))
        (dolist (entry profile-operations)
          (let* ((operation (or (soma-bridge-bench--get entry "operation")
                                "<missing-operation>"))
                 (maturity (soma-bridge-bench--get entry "maturity"))
                 (route (soma-bridge-bench--get entry "route"))
                 (stable-p (string= (or maturity "") "stable"))
                 (request-contract
                  (soma-bridge-bench--get entry "request_contract"))
                 (precondition-contract
                  (soma-bridge-bench--get entry "precondition_contract"))
                 (result-contract
                  (soma-bridge-bench--get entry "result_contract"))
                 (required-result-fields
                  (and result-contract
                       (soma-bridge-bench--get
                        result-contract "required_fields"))))
            (when (or (string= (or maturity "") "planned")
                      (string= (or route "") "planned"))
              (push operation planned-operations))
            (when stable-p
              (unless (eq (soma-bridge-bench--get entry "implemented") t)
                (push operation stable-unimplemented-operations))
              (unless (eq (soma-bridge-bench--get entry "enabled") t)
                (push operation stable-disabled-operations))
              (unless (and (consp request-contract)
                           (consp precondition-contract)
                           (consp result-contract)
                           (cl-every
                            (lambda (field)
                              (member field required-result-fields))
                            '("api_version" "request_id" "operation"
                              "state" "state_sequence" "result"
                              "evidence" "observations" "effects"
                              "diagnostics" "continuation")))
                (push operation stable-missing-contract-operations)))))))
    (soma-bridge-bench--alist
     (cons "profiles" soma-bridge-bench--first-profile-names)
     (cons "expected_stable_operations"
           soma-bridge-bench--expected-stable-profile-operations)
     (cons "no_stable_profiles" (nreverse no-stable-profiles))
     (cons "missing_expected_stable_operations"
           (nreverse missing-expected-stable-operations))
     (cons "planned_operations" (nreverse planned-operations))
     (cons "stable_unimplemented_operations"
           (nreverse stable-unimplemented-operations))
     (cons "stable_disabled_operations"
           (nreverse stable-disabled-operations))
     (cons "stable_missing_contract_operations"
           (nreverse stable-missing-contract-operations))
     (cons "passed"
           (soma-bridge-bench--bool
            (and (null no-stable-profiles)
                 (null missing-expected-stable-operations)
                 (null planned-operations)
                 (null stable-unimplemented-operations)
                 (null stable-disabled-operations)
                 (null stable-missing-contract-operations)))))))

(defun soma-bridge-bench--make-comint-shell (name)
  (let* ((buffer (get-buffer-create (format " *soma-bridge-%s*" name)))
         (_ (make-comint-in-buffer name buffer "sh" nil))
         (process (get-buffer-process buffer)))
    (set-process-query-on-exit-flag process nil)
    (set-window-buffer (selected-window) buffer)
    (with-current-buffer buffer
      (goto-char (process-mark process)))
    (accept-process-output process 0.2)
    (cons buffer process)))

(defun soma-bridge-bench--make-comint-program (name program &rest args)
  "Create a Comint buffer named NAME running PROGRAM with ARGS."
  (let* ((buffer (get-buffer-create (format " *soma-bridge-%s*" name)))
         (_ (apply #'make-comint-in-buffer name buffer program nil args))
         (process (get-buffer-process buffer)))
    (set-process-query-on-exit-flag process nil)
    (set-window-buffer (selected-window) buffer)
    (with-current-buffer buffer
      (goto-char (process-mark process)))
    (accept-process-output process 0.2)
    (cons buffer process)))

(defun soma-bridge-bench--cleanup-comint-shell (cell)
  (let ((buffer (car cell))
        (process (cdr cell)))
    (when (process-live-p process)
      (delete-process process))
    (when (buffer-live-p buffer)
      (kill-buffer buffer))))

(defun soma-bridge-bench--comint-command (label counter)
  (format
   "printf 'SOMA_BRIDGE_COMINT_SENT:%s\\n'; printf 'SOMA_BRIDGE_COMINT_RUN:%%s\\n' \"$SOMA_INSULA_RUN_ID\"; printf '%s\\n' >> %s; printf 'SOMA_BRIDGE_COMINT_DONE:%s\\n'"
   label
   label
   (shell-quote-argument counter)
   label))

(defun soma-bridge-bench--comint-interrupt-command (counter)
  (format
   "printf 'SOMA_BRIDGE_COMINT_INTERRUPT_START\\n'; sleep 5 && { printf 'finished\\n' >> %s; printf 'SOMA_BRIDGE_COMINT_INTERRUPT_DONE\\n'; }"
   (shell-quote-argument counter)))

(defun soma-bridge-bench--comint-submit-request
    (id buffer input preconditions)
  (soma-bridge-bench--api-request
   id "comint.submit_input"
   (soma-bridge-bench--alist
    (cons "buffer_name" (buffer-name buffer))
    (cons "input" input)
    (cons "preconditions" preconditions))))

(defun soma-bridge-bench--comint-await-request
    (id buffer needle preconditions &optional timeout-seconds)
  (soma-bridge-bench--api-request
   id "comint.await_output"
   (soma-bridge-bench--alist
    (cons "buffer_name" (buffer-name buffer))
    (cons "needle" needle)
    (cons "timeout_seconds" (or timeout-seconds 2.0))
    (cons "preconditions" preconditions))))

(defun soma-bridge-bench--comint-interrupt-request
    (id buffer preconditions)
  (soma-bridge-bench--api-request
   id "comint.interrupt"
   (soma-bridge-bench--alist
    (cons "buffer_name" (buffer-name buffer))
    (cons "preconditions" preconditions))))

(defun soma-bridge-bench--doom-straight-build-dir ()
  (expand-file-name
   (format "~/.emacs.d/.local/straight/build-%d.%d"
           emacs-major-version emacs-minor-version)))

(defun soma-bridge-bench--add-straight-build-paths ()
  (let ((build (soma-bridge-bench--doom-straight-build-dir)))
    (when (file-directory-p build)
      (dolist (dir (directory-files build t directory-files-no-dot-files-regexp))
        (when (file-directory-p dir)
          (add-to-list 'load-path dir))))))

(defun soma-bridge-bench--load-magit ()
  (soma-bridge-bench--add-straight-build-paths)
  (require 'magit-status)
  (require 'magit-apply)
  (require 'magit-section))

(defun soma-bridge-bench--magit-target-content
    (&optional selected other external)
  (let (lines)
    (dotimes (idx 30)
      (let ((line (1+ idx)))
        (push
         (cond
          ((= line 5)
           (if selected "target-05-selected" "target-05-base"))
          ((= line 25)
           (cond
            (external "target-25-external")
            (other "target-25-other")
            (t "target-25-base")))
          (t
           (format "target-%02d-base" line)))
         lines)))
    (concat (string-join (nreverse lines) "\n") "\n")))

(defun soma-bridge-bench--magit-index-file (repo file)
  (soma-bridge-bench--git repo "show" (concat ":" file)))

(defun soma-bridge-bench--magit-worktree-file (repo file)
  (soma-bridge-bench--read-file (expand-file-name file repo)))

(defun soma-bridge-bench--make-magit-repo (name)
  (let ((repo (make-temp-file (format "soma-bridge-magit-%s-" name) t)))
    (soma-bridge-bench--git repo "init" "-q")
    (soma-bridge-bench--git repo "config" "user.name" "Soma Bridge Benchmark")
    (soma-bridge-bench--git repo "config" "user.email"
                              "soma-bridge-bench@example.invalid")
    (soma-bridge-bench--write-file
     (expand-file-name "target.txt" repo)
     (soma-bridge-bench--magit-target-content nil nil nil))
    (soma-bridge-bench--write-file
     (expand-file-name "debug.txt" repo)
     "debug-base\n")
    (soma-bridge-bench--write-file
     (expand-file-name "staged.txt" repo)
     "staged-base\n")
    (soma-bridge-bench--git repo "add" ".")
    (soma-bridge-bench--git repo "commit" "-q" "-m" "initial")
    repo))

(defun soma-bridge-bench--setup-magit-unrelated-state (repo)
  (soma-bridge-bench--write-file
   (expand-file-name "staged.txt" repo)
   "staged-expected\n")
  (soma-bridge-bench--git repo "add" "staged.txt")
  (soma-bridge-bench--write-file
   (expand-file-name "debug.txt" repo)
   "debug-base\ndebug-worktree-only\n"))

(defun soma-bridge-bench--setup-magit-target-two-hunks (repo)
  (soma-bridge-bench--setup-magit-unrelated-state repo)
  (soma-bridge-bench--write-file
   (expand-file-name "target.txt" repo)
   (soma-bridge-bench--magit-target-content t t nil)))

(defun soma-bridge-bench--setup-magit-target-one-hunk (repo)
  (soma-bridge-bench--setup-magit-unrelated-state repo)
  (soma-bridge-bench--write-file
   (expand-file-name "target.txt" repo)
   (soma-bridge-bench--magit-target-content t nil nil)))

(defun soma-bridge-bench--magit-stale-external-edit (repo)
  (soma-bridge-bench--write-file
   (expand-file-name "target.txt" repo)
   (soma-bridge-bench--magit-target-content t nil t)))

(defun soma-bridge-bench--display-magit-buffer (buffer)
  (set-window-buffer (selected-window) buffer)
  (selected-window))

(defun soma-bridge-bench--magit-status-buffer (repo)
  (let ((default-directory (file-name-as-directory repo))
        (magit-display-buffer-function #'soma-bridge-bench--display-magit-buffer)
        (magit-display-buffer-noselect nil)
        (magit-auto-revert-mode nil))
    (magit-status-setup-buffer repo)))

(defun soma-bridge-bench--magit-section-slot (section slot)
  (eieio-oref section slot))

(defun soma-bridge-bench--magit-section-text (section)
  (buffer-substring-no-properties
   (soma-bridge-bench--magit-section-slot section 'start)
   (soma-bridge-bench--magit-section-slot section 'end)))

(defun soma-bridge-bench--magit-section-target-point (section needle)
  (save-excursion
    (goto-char (soma-bridge-bench--magit-section-slot section 'content))
    (if (search-forward needle
                        (soma-bridge-bench--magit-section-slot section 'end)
                        t)
        (match-beginning 0)
      (soma-bridge-bench--magit-section-slot section 'content))))

(defun soma-bridge-bench--magit-section-ancestor-file-p (section file)
  (let ((cursor section)
        found)
    (while cursor
      (when (and (eq (soma-bridge-bench--magit-section-slot cursor 'type)
                     'file)
                 (equal (soma-bridge-bench--magit-section-slot cursor 'value)
                        file))
        (setq found t))
      (setq cursor (soma-bridge-bench--magit-section-slot cursor 'parent)))
    found))

(defun soma-bridge-bench--magit-find-section (buffer predicate)
  (let (found)
    (with-current-buffer buffer
      (magit-map-sections
       (lambda (section)
         (when (and (not found) (funcall predicate section))
           (setq found section)))))
    found))

(defun soma-bridge-bench--magit-find-hunk-section (buffer file needle)
  (soma-bridge-bench--magit-find-section
   buffer
   (lambda (section)
     (and (eq (soma-bridge-bench--magit-section-slot section 'type) 'hunk)
          (soma-bridge-bench--magit-section-ancestor-file-p section file)
          (string-match-p (regexp-quote needle)
                          (with-current-buffer buffer
                            (soma-bridge-bench--magit-section-text section)))))))

(defun soma-bridge-bench--magit-find-file-section (buffer file)
  (soma-bridge-bench--magit-find-section
   buffer
   (lambda (section)
     (and (eq (soma-bridge-bench--magit-section-slot section 'type) 'file)
          (equal (soma-bridge-bench--magit-section-slot section 'value)
                 file)))))

(defun soma-bridge-bench--find-magit-observed-section
    (observation type file needle)
  (seq-find
   (lambda (section)
     (and (string= (or (soma-bridge-bench--get section "type") "") type)
          (string= (or (soma-bridge-bench--get section "ancestor_file") "")
                   file)
          (or (not needle)
              (string-match-p
               (regexp-quote needle)
               (or (soma-bridge-bench--get section "text_excerpt") "")))))
   (soma-bridge-bench--get observation "sections")))

(defun soma-bridge-bench--magit-repo-fingerprint (observation)
  (soma-bridge-bench--alist
   (cons "magit_repo" (soma-bridge-bench--get observation "magit_repo"))
   (cons "magit_head" (soma-bridge-bench--get observation "magit_head"))
   (cons "magit_status_hash"
         (soma-bridge-bench--get observation "magit_status_hash"))
   (cons "magit_worktree_diff_hash"
         (soma-bridge-bench--get observation "magit_worktree_diff_hash"))
   (cons "magit_index_diff_hash"
         (soma-bridge-bench--get observation "magit_index_diff_hash"))))

(defun soma-bridge-bench--magit-section-identity (section)
  (or (soma-bridge-bench--get section "section_identity")
      (soma-bridge-bench--alist
       (cons "type" (soma-bridge-bench--get section "type"))
       (cons "value" (soma-bridge-bench--get section "value"))
       (cons "ancestor_file" (soma-bridge-bench--get section "ancestor_file"))
       (cons "start" (soma-bridge-bench--get section "start"))
       (cons "content" (soma-bridge-bench--get section "content"))
       (cons "end" (soma-bridge-bench--get section "end"))
       (cons "parent_types" (soma-bridge-bench--get section "parent_types"))
       (cons "text_hash" (soma-bridge-bench--get section "text_hash")))))

(defun soma-bridge-bench--magit-stage-preconditions
    (observation section)
  (append
   observation
   (soma-bridge-bench--alist
    (cons "repo_fingerprint"
          (soma-bridge-bench--magit-repo-fingerprint observation))
    (cons "section_identity"
          (soma-bridge-bench--magit-section-identity section))
    (cons "this_command" "magit-stage"))))

(defun soma-bridge-bench--magit-stage-request
    (id operation buffer section preconditions)
  (soma-bridge-bench--alist
   (cons "api_version" "soma-emacs-agent-v0")
   (cons "request_id" id)
   (cons "operation" operation)
   (cons "capability" "git_index_write")
   (cons "params"
         (soma-bridge-bench--alist
          (cons "buffer_name" (buffer-name buffer))
          (cons "section" section)
          (cons "preconditions" preconditions)))))

(defun soma-bridge-bench--capture-magit-post-stage-hook ()
  (push (format "%S" this-command)
        soma-bridge-bench--magit-post-stage-runs))

(defun soma-bridge-bench--capture-magit-refresh-hook ()
  (push (buffer-name (current-buffer))
        soma-bridge-bench--magit-refresh-runs))

(defun soma-bridge-bench--with-magit-capture (thunk)
  (let ((soma-bridge-bench--magit-post-stage-runs nil)
        (soma-bridge-bench--magit-refresh-runs nil)
        (magit-post-stage-hook
         '(soma-bridge-bench--capture-magit-post-stage-hook))
        (magit-refresh-buffer-hook
         '(soma-bridge-bench--capture-magit-refresh-hook))
        (magit-auto-revert-mode nil))
    (funcall thunk)
    (soma-bridge-bench--alist
     (cons "post_stage_hook_runs"
           (nreverse soma-bridge-bench--magit-post-stage-runs))
     (cons "refresh_buffer_hook_runs"
           (nreverse soma-bridge-bench--magit-refresh-runs)))))

(defun soma-bridge-bench--magit-repo-state (repo)
  (soma-bridge-bench--alist
   (cons "cached_names"
         (soma-bridge-bench--git-lines repo "diff" "--cached" "--name-only"))
   (cons "unstaged_names"
         (soma-bridge-bench--git-lines repo "diff" "--name-only"))
   (cons "status_porcelain"
         (soma-bridge-bench--git-lines repo "status" "--porcelain=v1"))
   (cons "target_index"
         (ignore-errors
           (soma-bridge-bench--magit-index-file repo "target.txt")))
   (cons "target_worktree"
         (soma-bridge-bench--magit-worktree-file repo "target.txt"))
   (cons "debug_index"
         (ignore-errors
           (soma-bridge-bench--magit-index-file repo "debug.txt")))
   (cons "debug_worktree"
         (soma-bridge-bench--magit-worktree-file repo "debug.txt"))
   (cons "staged_index"
         (ignore-errors
           (soma-bridge-bench--magit-index-file repo "staged.txt")))
   (cons "staged_worktree"
         (soma-bridge-bench--magit-worktree-file repo "staged.txt"))))

(defun soma-bridge-bench--string-contains-p (string needle)
  (and (stringp string)
       (string-match-p (regexp-quote needle) string)))

(defun soma-bridge-bench--cleanup-magit-repo (repo)
  (dolist (buffer (buffer-list))
    (when (with-current-buffer buffer
            (or (derived-mode-p 'magit-mode)
                (string-prefix-p "magit" (buffer-name buffer))
                (string-prefix-p "*magit" (buffer-name buffer))))
      (kill-buffer buffer)))
  (when (and repo (file-directory-p repo))
    (delete-directory repo t)))

(defun soma-bridge-bench--object-p (value)
  (or (hash-table-p value)
      (and (listp value)
           (consp (car value)))))

(defun soma-bridge-bench--sequence-values (value)
  (cond
   ((vectorp value) (append value nil))
   ((listp value) value)
   (t nil)))

(defun soma-bridge-bench--has-key-p (object key)
  (cond
   ((hash-table-p object)
    (let ((sentinel (make-symbol "missing")))
      (not (eq (gethash key object sentinel) sentinel))))
   ((listp object)
    (not (null (assoc key object))))))

(defun soma-bridge-bench--diagnostics-valid-p (diagnostics)
  (and (or (vectorp diagnostics) (consp diagnostics))
       (cl-every
        (lambda (diagnostic)
          (and (soma-bridge-bench--object-p diagnostic)
               (stringp (soma-bridge-bench--get diagnostic "code"))
               (stringp (soma-bridge-bench--get diagnostic "message"))))
        (soma-bridge-bench--sequence-values diagnostics))))

(defun soma-bridge-bench--response-like-p (value)
  (and (soma-bridge-bench--object-p value)
       (soma-bridge-bench--has-key-p value "request_id")
       (soma-bridge-bench--has-key-p value "operation")
       (soma-bridge-bench--has-key-p value "state")
       (soma-bridge-bench--has-key-p value "state_sequence")))

(defun soma-bridge-bench--response-slot-p (key)
  (and (stringp key)
       (or (string= key "response")
           (string-suffix-p "_response" key)
           (string= key "status")
           (string= key "target_status"))))

(defun soma-bridge-bench--response-contract-issues (manifest path response)
  (let* ((operation (soma-bridge-bench--get response "operation"))
         (state (soma-bridge-bench--get response "state"))
         (state-sequence
          (soma-bridge-bench--sequence-values
           (soma-bridge-bench--get response "state_sequence")))
         (operation-spec
          (and (stringp operation)
               (soma-bridge-bench--find-operation manifest operation)))
         (result-contract
          (and operation-spec
               (soma-bridge-bench--get operation-spec "result_contract")))
         (required-fields
          (and result-contract
               (soma-bridge-bench--get result-contract "required_fields")))
         (allowed-states
          (append
           (or (and result-contract
                    (soma-bridge-bench--get result-contract "states"))
               nil)
           (or (and result-contract
                    (soma-bridge-bench--get result-contract
                                            "terminal_states"))
               nil)))
         (allowed-sequence-states
          (append allowed-states '("accepted" "running" "waiting_for_input")))
         issues)
    (cl-labels
        ((issue (code message &optional detail)
           (push
            (soma-bridge-bench--alist
             (cons "path" path)
             (cons "request_id"
                   (or (soma-bridge-bench--get response "request_id")
                       :json-false))
             (cons "operation" (or operation :json-false))
             (cons "state" (or state :json-false))
             (cons "code" code)
             (cons "message" message)
             (cons "detail" (or detail :json-false)))
            issues)))
      (unless (string= (or (soma-bridge-bench--get response "api_version") "")
                       "soma-emacs-agent-v0")
        (issue "invalid_api_version"
               "response must declare soma-emacs-agent-v0"))
      (unless (stringp (soma-bridge-bench--get response "request_id"))
        (issue "invalid_request_id" "response request_id must be a string"))
      (unless (stringp operation)
        (issue "invalid_operation" "response operation must be a string"))
      (unless operation-spec
        (issue "unknown_operation"
               "response operation must be present in the operation manifest"))
      (dolist (field required-fields)
        (unless (soma-bridge-bench--has-key-p response field)
          (issue "missing_required_field"
                 "response is missing a manifest-required result-envelope field"
                 field)))
      (unless (member state allowed-states)
        (issue "invalid_state"
               "response state must be declared by the operation result contract"
               (soma-bridge-bench--alist
                (cons "allowed_states" allowed-states))))
      (unless state-sequence
        (issue "invalid_state_sequence"
               "response state_sequence must be a non-empty list or vector"))
      (when (and state-sequence (not (string= (car (last state-sequence))
                                             state)))
        (issue "invalid_state_sequence"
               "response state_sequence must end in the current response state"
               state-sequence))
      (dolist (sequence-state state-sequence)
        (unless (member sequence-state allowed-sequence-states)
          (issue "invalid_state_sequence"
                 "response state_sequence contains a state outside the lifecycle contract"
                 (soma-bridge-bench--alist
                  (cons "sequence_state" sequence-state)
                  (cons "allowed_states" allowed-sequence-states)))))
      (unless (soma-bridge-bench--object-p
               (soma-bridge-bench--get response "evidence"))
        (issue "invalid_evidence" "response evidence must be an object"))
      (unless (soma-bridge-bench--object-p
               (soma-bridge-bench--get response "observations"))
        (issue "invalid_observations" "response observations must be an object"))
      (unless (soma-bridge-bench--object-p
               (soma-bridge-bench--get response "effects"))
        (issue "invalid_effects" "response effects must be an object"))
      (unless (soma-bridge-bench--diagnostics-valid-p
               (soma-bridge-bench--get response "diagnostics"))
        (issue "invalid_diagnostics"
               "response diagnostics must be an array of code/message objects")))
    (nreverse issues)))

(defun soma-bridge-bench--collect-response-validations (manifest records)
  (let (validations)
    (cl-labels
        ((walk (value path &optional key)
           (when (and (soma-bridge-bench--response-like-p value)
                      (or (soma-bridge-bench--response-slot-p key)
                          (soma-bridge-bench--has-key-p
                           value "api_version")))
             (push
              (soma-bridge-bench--alist
               (cons "path" path)
               (cons "request_id"
                     (or (soma-bridge-bench--get value "request_id")
                         :json-false))
               (cons "operation"
                     (or (soma-bridge-bench--get value "operation")
                         :json-false))
               (cons "state"
                     (or (soma-bridge-bench--get value "state")
                         :json-false))
               (cons "issues"
                     (soma-bridge-bench--response-contract-issues
                      manifest path value)))
              validations))
           (cond
            ((hash-table-p value)
             (maphash
              (lambda (child-key child-value)
                (walk child-value
                      (format "%s.%s" path child-key)
                      child-key))
              value))
            ((and (listp value) (not (null value)) (consp (car value)))
             (dolist (entry value)
               (let ((child-key (car entry)))
                 (walk (cdr entry)
                       (format "%s.%s" path child-key)
                       child-key))))
            ((or (vectorp value) (listp value))
             (cl-loop for child across (vconcat value)
                      for index from 0
                      do (walk child
                               (format "%s[%d]" path index)))))))
      (cl-loop for record in records
               for index from 0
               do (walk record
                        (format "records[%d]" index))))
    (nreverse validations)))

(defun soma-bridge-bench--scenario-response-envelope-validation ()
  (let* ((records (reverse soma-bridge-benchmark-results))
         (manifest (soma-gym-bridge-list-capabilities))
         (validations
          (soma-bridge-bench--collect-response-validations
           manifest records))
         (issues
          (apply #'append
                 (mapcar
                  (lambda (validation)
                    (soma-bridge-bench--get validation "issues"))
                  validations)))
         (paths (mapcar
                 (lambda (validation)
                   (soma-bridge-bench--get validation "path"))
                 validations))
         (checks
          (list
           (soma-bridge-bench--check
            "captured bridge responses are discovered recursively"
            (> (length validations) 20)
            paths)
           (soma-bridge-bench--check
            "nested stored request responses are included"
            (seq-some
             (lambda (path)
               (or (string-match-p "\\.status\\'" path)
                   (string-match-p "\\.target_status\\'" path)))
             paths)
            paths)
           (soma-bridge-bench--check
            "all captured responses satisfy the manifest result envelope"
            (null issues)
            issues))))
    (soma-bridge-bench--alist
     (cons "validated_response_count" (length validations))
     (cons "validated_responses" validations)
     (cons "issues" issues)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-envelope-observe ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-observe" "alpha\n"))
         (request (soma-bridge-bench--observe-request "observe-1" buffer))
         (response (soma-gym-bridge-submit-request request))
         (status (soma-gym-bridge-request-status "observe-1"))
         (result (soma-bridge-bench--get response "result"))
	         (evidence (soma-bridge-bench--get response "evidence"))
	         (observations (soma-bridge-bench--get response "observations"))
	         (effects (soma-bridge-bench--get response "effects"))
         (checks
          (list
           (soma-bridge-bench--check
            "request id is echoed"
            (string= (soma-bridge-bench--get response "request_id") "observe-1")
            (format "%S" response))
           (soma-bridge-bench--check
            "request reached succeeded state"
            (string= (soma-bridge-bench--get response "state") "succeeded")
            (format "%S" response))
           (soma-bridge-bench--check
            "state sequence records accepted running succeeded"
            (equal (soma-bridge-bench--get response "state_sequence")
                   '("accepted" "running" "succeeded"))
            (format "%S" (soma-bridge-bench--get response "state_sequence")))
           (soma-bridge-bench--check
            "response exposes the stable v0 envelope sections"
            (and (string= (soma-bridge-bench--get response "api_version")
                          "soma-emacs-agent-v0")
	                 (or (hash-table-p evidence) (consp evidence))
	                 (or (hash-table-p observations) (consp observations))
	                 (or (hash-table-p effects) (consp effects))
                 (vectorp (soma-bridge-bench--get response "diagnostics"))
                 (= (length (soma-bridge-bench--get response "diagnostics"))
                    0)
                 (assoc "continuation" response))
            (format "%S" response))
           (soma-bridge-bench--check
            "observe result names target buffer"
            (and (string= (soma-bridge-bench--get result "operation")
                          "emacs.observe_buffer")
                 (string= (soma-bridge-bench--get result "buffer_name")
                          (buffer-name buffer)))
            (format "%S" result))
	           (soma-bridge-bench--check
	            "emacs.observe_buffer publishes typed buffer identity and state sections"
	            (let ((buffer-identity
	                   (soma-bridge-bench--get evidence "buffer_identity"))
	                  (text-owner
	                   (soma-bridge-bench--get evidence "text_owner"))
	                  (point-region
	                   (soma-bridge-bench--get evidence
	                                            "point_region_narrowing"))
	                  (buffer-state
	                   (soma-bridge-bench--get observations "buffer_state")))
	              (and (string= (soma-bridge-bench--get
	                              buffer-identity "buffer_name")
	                             (buffer-name buffer))
	                   (string= (soma-bridge-bench--get
	                             text-owner "text_owner_name")
	                            (buffer-name buffer))
	                   (= (or (soma-bridge-bench--get
	                           point-region "point_min")
	                          0)
	                      1)
	                   (= (or (soma-bridge-bench--get
	                           point-region "point_max")
	                          0)
	                      (1+ (buffer-size buffer)))
	                   (string= (soma-bridge-bench--get
	                             buffer-state "major_mode")
	                            "fundamental-mode")
	                   (eq (soma-bridge-bench--get
	                        buffer-state "modified")
	                       :json-false)
	                   (and (hash-table-p effects)
	                        (= (hash-table-count effects) 0))))
	            (format "%S" response))
           (soma-bridge-bench--check
            "status returns stored completed request"
            (and (string= (soma-bridge-bench--get status "request_id")
                          "observe-1")
                 (string= (soma-bridge-bench--get status "state")
                          "succeeded"))
            (format "%S" status)))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "status" status)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-api-manifest-introspection ()
  (soma-gym-bridge-reset-transport)
  (let* ((list-request
          (soma-bridge-bench--api-request
           "api-list-1" "api.list_capabilities"))
         (list-response
          (soma-gym-bridge-submit-request list-request))
         (manifest (soma-bridge-bench--get list-response "result"))
         (session-entry
          (and (listp manifest)
               (soma-bridge-bench--find-operation
                manifest "emacs.observe_session")))
         (replace-entry
          (and (listp manifest)
               (soma-bridge-bench--find-operation
                manifest "buffer.replace_range")))
         (completion-observe-entry
          (and (listp manifest)
               (soma-bridge-bench--find-operation
                manifest "completion.observe")))
         (direct-entry
          (and (listp manifest)
               (soma-bridge-bench--find-operation manifest "direct-call")))
         (request-respond-entry
          (and (listp manifest)
               (soma-bridge-bench--find-operation
                manifest "request.respond")))
         (elisp-eval-entry
          (and (listp manifest)
               (soma-bridge-bench--find-operation manifest "elisp.eval")))
         (lsp-entry
          (and (listp manifest)
               (soma-bridge-bench--find-operation
                manifest "lsp.observe_source")))
         (lsp-diagnostics-entry
          (and (listp manifest)
               (soma-bridge-bench--find-operation
                manifest "lsp.observe_diagnostics")))
         (lsp-rename-entry
          (and (listp manifest)
               (soma-bridge-bench--find-operation
                manifest "lsp.rename_symbol")))
         (first-profile-gate
          (and (listp manifest)
               (soma-bridge-bench--first-profile-manifest-gate manifest)))
         (synthetic-drift-gate
          (and
           (listp manifest)
           (soma-bridge-bench--first-profile-manifest-gate
            (soma-bridge-bench--alist
             (cons
              "operations"
              (append
               (soma-bridge-bench--get manifest "operations")
               (list
                (soma-bridge-bench--alist
                 (cons "operation" "synthetic.planned_operation")
                 (cons "profile" "coding-v0")
                 (cons "maturity" "planned")
                 (cons "implemented" :json-false)
                 (cons "enabled" :json-false)
                 (cons "route" "planned"))
                (soma-bridge-bench--alist
                 (cons "operation" "synthetic.disabled_stable_operation")
                 (cons "profile" "terminal-v0")
                 (cons "maturity" "stable")
                 (cons "implemented" :json-false)
                 (cons "enabled" :json-false)
                 (cons "route" "synthetic-route")))))))))
         (describe-request
          (soma-bridge-bench--api-request
           "api-describe-1" "api.describe_operation"
           (soma-bridge-bench--alist
            (cons "operation" "buffer.replace_range"))))
         (describe-response
          (soma-gym-bridge-submit-request describe-request))
         (description
          (soma-bridge-bench--get describe-response "result"))
         (request-contract
          (and (listp description)
               (soma-bridge-bench--get description "request_contract")))
         (precondition-contract
          (and (listp description)
               (soma-bridge-bench--get description "precondition_contract")))
         (result-contract
          (and (listp description)
               (soma-bridge-bench--get description "result_contract")))
         (profiles
          (and (listp manifest)
               (soma-bridge-bench--get manifest "profiles")))
         (checks
          (list
           (soma-bridge-bench--check
            "api.list_capabilities succeeds through request envelope"
            (string= (soma-bridge-bench--get list-response "state")
                     "succeeded")
            (format "%S" list-response))
           (soma-bridge-bench--check
            "manifest declares soma emacs agent api version"
            (and (listp manifest)
                 (string= (soma-bridge-bench--get manifest "operation")
                          "api.list_capabilities")
                 (string= (soma-bridge-bench--get manifest "api_version")
                          "soma-emacs-agent-v0"))
            (format "%S" manifest))
           (soma-bridge-bench--check
            "manifest publishes stable first-slice profiles"
            (and (member "core-v0" profiles)
                 (member "coding-v0" profiles)
                 (member "terminal-v0" profiles)
                 (member "analysis-v0" profiles)
                 (member "reports-v0" profiles)
                 (member "vcs-files-v0" profiles))
            (format "%S" profiles))
           (soma-bridge-bench--check
            "first stable profiles contain their expected stable operations"
            (and first-profile-gate
                 (null (soma-bridge-bench--get
                        first-profile-gate
                        "missing_expected_stable_operations"))
                 (null (soma-bridge-bench--get
                        first-profile-gate
                        "no_stable_profiles")))
            (format "%S" first-profile-gate))
           (soma-bridge-bench--check
            "first stable profiles contain no planned operations"
            (and first-profile-gate
                 (null (soma-bridge-bench--get
                        first-profile-gate "planned_operations")))
            (format "%S" first-profile-gate))
           (soma-bridge-bench--check
            "all first-profile stable operations are implemented and enabled"
            (and first-profile-gate
                 (null (soma-bridge-bench--get
                        first-profile-gate
                        "stable_unimplemented_operations"))
                 (null (soma-bridge-bench--get
                        first-profile-gate
                        "stable_disabled_operations")))
            (format "%S" first-profile-gate))
           (soma-bridge-bench--check
            "all first-profile stable operations publish envelope contracts"
            (and first-profile-gate
                 (null (soma-bridge-bench--get
                        first-profile-gate
                        "stable_missing_contract_operations")))
            (format "%S" first-profile-gate))
           (soma-bridge-bench--check
            "first-profile manifest gate detects synthetic planned and disabled stable drift"
            (and synthetic-drift-gate
                 (not (eq (soma-bridge-bench--get synthetic-drift-gate
                                                  "passed")
                          t))
                 (member "synthetic.planned_operation"
                         (soma-bridge-bench--get
                          synthetic-drift-gate "planned_operations"))
                 (member "synthetic.disabled_stable_operation"
                         (soma-bridge-bench--get
                          synthetic-drift-gate
                          "stable_unimplemented_operations"))
                 (member "synthetic.disabled_stable_operation"
                         (soma-bridge-bench--get
                          synthetic-drift-gate
                          "stable_disabled_operations"))
                 (member "synthetic.disabled_stable_operation"
                         (soma-bridge-bench--get
                          synthetic-drift-gate
                          "stable_missing_contract_operations")))
            (format "%S" synthetic-drift-gate))
           (soma-bridge-bench--check
            "emacs.observe_session is a stable enabled core operation"
            (and session-entry
                 (string= (soma-bridge-bench--get session-entry "profile")
                          "core-v0")
                 (string= (soma-bridge-bench--get session-entry "maturity")
                          "stable")
                 (eq (soma-bridge-bench--get session-entry "implemented")
                     t)
                 (eq (soma-bridge-bench--get session-entry "enabled") t)
                 (member "rootfs_identity"
                         (soma-bridge-bench--get session-entry "evidence"))
                 (member "doom_identity"
                         (soma-bridge-bench--get session-entry "evidence")))
            (format "%S" session-entry))
           (soma-bridge-bench--check
            "buffer.replace_range is classified as stable coding edit"
            (and replace-entry
                 (string= (soma-bridge-bench--get replace-entry "profile")
                          "coding-v0")
                 (string= (soma-bridge-bench--get replace-entry "maturity")
                          "stable")
                 (string= (soma-bridge-bench--get
                           replace-entry "side_effect_class")
                          "reversible_editor_edit"))
            (format "%S" replace-entry))
           (soma-bridge-bench--check
            "completion.observe is a stable provider-context observation"
            (and completion-observe-entry
                 (string= (soma-bridge-bench--get
                           completion-observe-entry "profile")
                          "coding-v0")
                 (string= (soma-bridge-bench--get
                           completion-observe-entry "maturity")
                          "stable")
                 (eq (soma-bridge-bench--get
                      completion-observe-entry "implemented")
                     t)
                 (eq (soma-bridge-bench--get
                      completion-observe-entry "enabled")
                     t)
                 (member "replacement_range"
                         (soma-bridge-bench--get
                          completion-observe-entry "evidence"))
                 (member "preview_state"
                         (soma-bridge-bench--get
                          completion-observe-entry "evidence")))
            (format "%S" completion-observe-entry))
           (soma-bridge-bench--check
            "manifest does not classify direct-call as stable"
            (or (not direct-entry)
                (not (string= (soma-bridge-bench--get
                               direct-entry "maturity")
                              "stable")))
            (format "%S" direct-entry))
           (soma-bridge-bench--check
            "manifest keeps request.respond experimental until prompt-owner semantics are proven"
            (and request-respond-entry
                 (string= (soma-bridge-bench--get
                           request-respond-entry "profile")
                          "core-v0")
                 (string= (soma-bridge-bench--get
                           request-respond-entry "maturity")
                          "experimental")
                 (string= (soma-bridge-bench--get
                           request-respond-entry "route")
                          "request-continuation"))
            (format "%S" request-respond-entry))
           (soma-bridge-bench--check
            "manifest does not classify arbitrary Elisp eval as stable"
            (and elisp-eval-entry
                 (not (string= (soma-bridge-bench--get
                                elisp-eval-entry "maturity")
                               "stable"))
                 (string= (soma-bridge-bench--get
                           elisp-eval-entry "route")
                          "rejection-only"))
            (format "%S" elisp-eval-entry))
           (soma-bridge-bench--check
            "lsp.observe_source is classified as active diagnostic"
            (and lsp-entry
                 (string= (soma-bridge-bench--get
                           lsp-entry "observation_class")
                          "active-diagnostic"))
            (format "%S" lsp-entry))
           (soma-bridge-bench--check
            "lsp.observe_diagnostics is a stable diagnostics-only coding operation"
            (and lsp-diagnostics-entry
                 (string= (soma-bridge-bench--get
                           lsp-diagnostics-entry "profile")
                          "coding-v0")
                 (string= (soma-bridge-bench--get
                           lsp-diagnostics-entry "maturity")
                          "stable")
                 (eq (soma-bridge-bench--get
                      lsp-diagnostics-entry "implemented")
                     t)
                 (eq (soma-bridge-bench--get
                      lsp-diagnostics-entry "enabled")
                     t)
                 (member "diagnostic_cache"
                         (soma-bridge-bench--get
                          lsp-diagnostics-entry "evidence")))
            (format "%S" lsp-diagnostics-entry))
           (soma-bridge-bench--check
            "lsp.rename_symbol is a stable workspace-edit coding operation"
            (and lsp-rename-entry
                 (string= (soma-bridge-bench--get
                           lsp-rename-entry "profile")
                          "coding-v0")
                 (string= (soma-bridge-bench--get
                           lsp-rename-entry "maturity")
                          "stable")
                 (eq (soma-bridge-bench--get
                      lsp-rename-entry "implemented")
                     t)
                 (eq (soma-bridge-bench--get
                      lsp-rename-entry "enabled")
                     t)
                 (string= (soma-bridge-bench--get
                           lsp-rename-entry "route")
                          "lsp-rename")
                 (member "lsp_request"
                         (soma-bridge-bench--get
                          lsp-rename-entry "evidence"))
                 (member "workspace_edit"
                         (soma-bridge-bench--get
                          lsp-rename-entry "evidence")))
            (format "%S" lsp-rename-entry))
           (soma-bridge-bench--check
            "api.describe_operation succeeds for a stable operation"
            (string= (soma-bridge-bench--get describe-response "state")
                     "succeeded")
            (format "%S" describe-response))
           (soma-bridge-bench--check
            "operation description exposes owner, preconditions, states, and evidence"
            (and (listp description)
                 (string= (soma-bridge-bench--get description "operation")
                          "buffer.replace_range")
                 (string= (soma-bridge-bench--get description "owner")
                          "core-emacs-buffer")
                 (member "buffer_name"
                         (soma-bridge-bench--get
                          description "required_preconditions"))
                 (member "chars_tick"
                         (soma-bridge-bench--get
                          description "required_preconditions"))
                 (member "failed"
                         (soma-bridge-bench--get
                          description "possible_states"))
                 (member "before_after_observations"
                         (soma-bridge-bench--get
                          description "evidence")))
            (format "%S" description))
           (soma-bridge-bench--check
            "operation description exposes machine-readable envelope contracts"
            (and (listp request-contract)
                 (string= (soma-bridge-bench--get
                           request-contract "envelope")
                          "soma-emacs-agent-v0.request")
                 (string= (soma-bridge-bench--get
                           request-contract "capability")
                          "reversible_editor_edit")
                 (member "buffer_name"
                         (soma-bridge-bench--get
                          request-contract "required_params"))
                 (listp precondition-contract)
                 (string= (soma-bridge-bench--get
                           precondition-contract "mode")
                          "typed-object")
                 (member "chars_tick"
                         (soma-bridge-bench--get
                          precondition-contract "required_fields"))
                 (listp result-contract)
                 (string= (soma-bridge-bench--get
                           result-contract "envelope")
                          "soma-emacs-agent-v0.result")
                 (member "state_sequence"
                         (soma-bridge-bench--get
                          result-contract "required_fields"))
                 (member "failed"
                         (soma-bridge-bench--get
                          result-contract "states"))
                 (member "diagnostics"
                         (soma-bridge-bench--get
                          result-contract "required_fields"))
                 (member "before_after_observations"
                         (soma-bridge-bench--get
                          result-contract "evidence_channels")))
            (format "%S" description)))))
    (soma-bridge-bench--alist
     (cons "list_request" list-request)
     (cons "list_response" list-response)
     (cons "describe_request" describe-request)
     (cons "describe_response" describe-response)
     (cons "first_profile_manifest_gate" first-profile-gate)
     (cons "synthetic_drift_gate" synthetic-drift-gate)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-lsp-envelope-projection ()
  (soma-gym-bridge-reset-transport)
  (let* ((source-result
          (soma-bridge-bench--alist
           (cons "operation" "lsp.observe_source")
           (cons "state" "succeeded")
           (cons "buffer_name" "core.py")
           (cons "text_owner_name" "core.py")
           (cons "buffer_file_name" "/project/tmp/lsp/core.py")
           (cons "major_mode" "python-mode")
           (cons "chars_tick" 7)
           (cons "modified" :json-false)
           (cons "read_only" :json-false)
           (cons "point" 42)
           (cons "mark_active" :json-false)
           (cons "narrowed" :json-false)
           (cons "selected_window_buffer" "core.py")
           (cons "minibuffer_active" :json-false)
           (cons "workspace_present" t)
           (cons "workspace_status" "initialized")
           (cons "workspace_root" "/project/tmp/lsp")
           (cons "document_uri" "file:///project/tmp/lsp/core.py")
           (cons "document_version" 5)
           (cons "position_encoding" "utf-16")
           (cons "lsp_sync"
                 (soma-bridge-bench--alist
                  (cons "flushed_delayed_changes" t)
                  (cons "settle_seconds" 0.2)))
           (cons "capabilities"
                 (soma-bridge-bench--alist
                  (cons "textDocument/rename" t)))
           (cons "diagnostics"
                 (soma-bridge-bench--alist
                  (cons "count" 0)
                  (cons "raw" "")))
           (cons "diagnostic_cache"
                 (soma-bridge-bench--alist
                  (cons "workspace" "/project/tmp/lsp")
                  (cons "documents" nil)))
           (cons "symbol" "compute_value")
           (cons "symbol_source" "thing-at-point")
           (cons "text_hash" "source-hash")
           (cons "text_excerpt" "def compute_value(x):\n    return x\n")
           (cons "server_id" "pylsp")
           (cons "server_pid" 1234)
           (cons "server_executable" "/usr/bin/pylsp")
           (cons "server_cwd" "/project/tmp/lsp")
           (cons "server_command" '("pylsp"))
           (cons "workspace_proc_type" "real")
           (cons "cmd_proc_type" "real")
           (cons "cmd_proc_status" "run")
           (cons "run_id" "synthetic-run")
           (cons "plan" "/project/run/insula-plan.json")))
         (source-response
          (soma-gym-bridge--response
           "lsp-source-synthetic" "lsp.observe_source" "succeeded"
           '("accepted" "running" "succeeded")
           source-result))
         (source-evidence
          (soma-bridge-bench--get source-response "evidence"))
         (source-observations
          (soma-bridge-bench--get source-response "observations"))
         (source-effects
          (soma-bridge-bench--get source-response "effects"))
         (rename-result
          (soma-bridge-bench--alist
           (cons "operation" "lsp.rename_symbol")
           (cons "state" "succeeded")
           (cons "state_sequence" '("accepted" "running" "succeeded"))
           (cons "route" "lsp-rename")
           (cons "new_name" "renamed_value")
           (cons "wait_timeout_seconds" 30.0)
           (cons "request_methods" '("textDocument/rename"))
           (cons "requests"
                 (list
                  (soma-bridge-bench--alist
                   (cons "method" "textDocument/rename")
                   (cons "id" 99)
                   (cons "buffer" "core.py")
                   (cons "document_version" 5))))
           (cons "workspace_edit_applied" t)
           (cons "workspace_edit_sha256" "workspace-edit-hash")
           (cons "workspace_edit_excerpt" "(:changes ...)")
           (cons "saved_buffers"
                 (list
                  (soma-bridge-bench--alist
                   (cons "buffer_name" "core.py")
                   (cons "file" "/project/tmp/lsp/core.py")
                   (cons "chars_tick" 8))))
           (cons "precondition_checks"
                 (list
                  (soma-bridge-bench--check
                   "LSP document version matches precondition" t "ok")))
           (cons "before" source-result)
           (cons "after"
                 (append
                  (soma-bridge-bench--alist
                   (cons "symbol" "renamed_value")
                   (cons "document_version" 6))
                  (soma-bridge-bench--without-key
                   (soma-bridge-bench--without-key source-result "symbol")
                   "document_version")))
           (cons "error" :json-false)))
         (rename-response
          (soma-gym-bridge--response
           "lsp-rename-synthetic" "lsp.rename_symbol" "succeeded"
           '("accepted" "running" "succeeded")
           rename-result))
         (rename-evidence
          (soma-bridge-bench--get rename-response "evidence"))
         (rename-observations
          (soma-bridge-bench--get rename-response "observations"))
         (rename-effects
          (soma-bridge-bench--get rename-response "effects"))
         (workspace-effect
          (and (listp rename-effects)
               (soma-bridge-bench--get rename-effects "workspace_edit")))
         (checks
          (list
           (soma-bridge-bench--check
            "lsp.observe_source projects typed workspace and server evidence"
            (and (listp source-evidence)
                 (listp (soma-bridge-bench--get source-evidence "workspace"))
                 (string= (soma-bridge-bench--get
                           (soma-bridge-bench--get source-evidence "workspace")
                           "status")
                          "initialized")
                 (listp (soma-bridge-bench--get source-evidence "server"))
                 (string= (soma-bridge-bench--get
                           (soma-bridge-bench--get source-evidence "server")
                           "id")
                          "pylsp")
                 (listp (soma-bridge-bench--get source-evidence "document"))
                 (listp (soma-bridge-bench--get source-evidence "diagnostics"))
                 (listp (soma-bridge-bench--get
                         source-evidence "diagnostic_cache"))
                 (listp (soma-bridge-bench--get source-evidence "lsp_sync"))
                 (listp (soma-bridge-bench--get
                         source-evidence "rootfs_runtime")))
            (format "%S" source-evidence))
           (soma-bridge-bench--check
            "lsp.observe_source projects typed observations without effects"
            (and (listp source-observations)
                 (listp (soma-bridge-bench--get
                         source-observations "source_buffer"))
                 (string= (soma-bridge-bench--get
                           (soma-bridge-bench--get
                            source-observations "source_buffer")
                           "buffer_name")
                          "core.py")
                 (listp (soma-bridge-bench--get source-observations
                                                "document"))
                 (listp (soma-bridge-bench--get source-observations
                                                "symbol"))
                 (null (hash-table-keys source-effects)))
            (format "%S %S" source-observations source-effects))
           (soma-bridge-bench--check
            "lsp.rename_symbol projects typed request and workspace-edit evidence"
            (and (listp rename-evidence)
                 (listp (soma-bridge-bench--get rename-evidence
                                                "lsp_request"))
                 (member
                  "textDocument/rename"
                  (soma-bridge-bench--get
                   (soma-bridge-bench--get rename-evidence "lsp_request")
                   "methods"))
                 (listp (soma-bridge-bench--get rename-evidence
                                                "workspace_edit"))
                 (eq (soma-bridge-bench--get
                      (soma-bridge-bench--get rename-evidence
                                             "workspace_edit")
                      "applied")
                     t)
                 (listp (soma-bridge-bench--get rename-evidence
                                                "saved_buffers"))
                 (listp (soma-bridge-bench--get rename-evidence
                                                "rootfs_runtime")))
            (format "%S" rename-evidence))
           (soma-bridge-bench--check
            "lsp.rename_symbol projects a multi-buffer workspace-edit effect"
            (and (listp workspace-effect)
                 (string= (soma-bridge-bench--get workspace-effect "route")
                          "lsp-rename")
                 (string= (soma-bridge-bench--get workspace-effect
                                                  "new_name")
                          "renamed_value")
                 (eq (soma-bridge-bench--get workspace-effect "applied") t)
                 (member "textDocument/rename"
                         (soma-bridge-bench--get workspace-effect
                                                "request_methods"))
                 (listp (soma-bridge-bench--get workspace-effect
                                                "saved_buffers"))
                 (member "/project/tmp/lsp/core.py"
                         (soma-bridge-bench--get workspace-effect
                                                "affected_files"))
                 (not (soma-bridge-bench--get rename-effects
                                              "changed_range")))
            (format "%S" rename-effects))
           (soma-bridge-bench--check
            "lsp.rename_symbol preserves before and after observations"
            (and (listp rename-observations)
                 (listp (soma-bridge-bench--get rename-observations
                                                "before"))
                 (listp (soma-bridge-bench--get rename-observations
                                                "after"))
                 (string= (soma-bridge-bench--get
                           (soma-bridge-bench--get rename-observations
                                                  "before")
                           "symbol")
                          "compute_value")
                 (string= (soma-bridge-bench--get
                           (soma-bridge-bench--get rename-observations
                                                  "after")
                           "symbol")
                          "renamed_value"))
            (format "%S" rename-observations)))))
    (soma-bridge-bench--alist
     (cons "source_response" source-response)
     (cons "rename_response" rename-response)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-lsp-server-ownership-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((tmp (make-temp-file "soma-bridge-lsp-host-owner-" t))
         (subject-file (expand-file-name "subject.py" tmp))
         (host-parent
          (expand-file-name
           ".scratch/emacs-gym/host-shadow-tools/"
           (file-name-as-directory
            (or (locate-dominating-file default-directory ".git")
                default-directory))))
         (host-dir (make-temp-file
                    (expand-file-name "soma-bridge-host-lsp-" host-parent)
                    t))
         (host-lsp-path (expand-file-name "pylsp" host-dir))
         (source-sleep (or (executable-find "sleep")
                           (error "sleep executable is required")))
         (call-log nil)
         buffer fake-process fake-workspace observe-request observe-response
         observe-result diagnostics-request diagnostics-response diagnostics-result
         rename-request rename-response await-request await-response
         rename-final rename-result server-evidence checks)
    (unwind-protect
        (progn
          (make-directory host-parent t)
          (soma-bridge-bench--write-file
           subject-file
           "def compute_value(x: int) -> int:\n    return x + 22\n")
          (copy-file source-sleep host-lsp-path t)
          (set-file-modes host-lsp-path #o755)
          (setq fake-process
                (start-process
                 "soma-host-shadow-pylsp" nil host-lsp-path "60"))
          (set-process-query-on-exit-flag fake-process nil)
          (setq buffer (find-file-noselect subject-file))
          (set-window-buffer (selected-window) buffer)
          (with-current-buffer buffer
            (setq default-directory (file-name-as-directory tmp))
            (goto-char (point-min))
            (search-forward "compute_value")
            (goto-char (match-beginning 0))
            (setq-local lsp--cur-version 1)
            (setq fake-workspace
                  (list (cons 'root (file-name-as-directory
                                      (file-truename tmp)))
                        (cons 'process fake-process)))
            (setq-local lsp--buffer-workspaces (list fake-workspace)))
          (cl-letf (((symbol-function 'lsp--workspace-cmd-proc)
                     (lambda (workspace) (alist-get 'process workspace)))
                    ((symbol-function 'lsp--workspace-proc)
                     (lambda (workspace) (alist-get 'process workspace)))
                    ((symbol-function 'lsp-process-id)
                     (lambda (process) (process-id process)))
                    ((symbol-function 'lsp--workspace-status)
                     (lambda (_workspace) 'initialized))
                    ((symbol-function 'lsp--workspace-root)
                     (lambda (workspace) (alist-get 'root workspace)))
                    ((symbol-function 'lsp--workspace-client)
                     (lambda (_workspace) 'host-shadow-client))
                    ((symbol-function 'lsp--client-server-id)
                     (lambda (_client) 'pylsp))
                    ((symbol-function 'lsp-feature?)
                     (lambda (method)
                       (and (string= method "textDocument/rename") t)))
                    ((symbol-function 'lsp--buffer-uri)
                     (lambda ()
                       (concat "file://" (file-truename subject-file))))
                    ((symbol-function 'lsp--get-symbol-to-rename)
                     (lambda ()
                       (push "symbol-request" call-log)
                       (cons (cons (point) (+ (point) 13))
                             "compute_value")))
                    ((symbol-function 'lsp--flush-delayed-changes)
                     (lambda ()
                       (push "flush-delayed-changes" call-log)))
                    ((symbol-function 'lsp-diagnostics--request-pull-diagnostics)
                     (lambda (&rest _)
                       (push "pull-diagnostics" call-log)))
                    ((symbol-function 'lsp-rename)
                     (lambda (&rest _)
                       (push "rename" call-log))))
            (setq observe-request
                  (soma-bridge-bench--api-request
                   "lsp-host-observe-1" "lsp.observe_source"
                   (soma-bridge-bench--alist
                    (cons "buffer_name" (buffer-name buffer))
                    (cons "max_text" 400))))
            (setq observe-response
                  (soma-gym-bridge-submit-request observe-request))
            (setq observe-result
                  (soma-bridge-bench--get observe-response "result"))
            (setq diagnostics-request
                  (soma-bridge-bench--api-request
                   "lsp-host-diagnostics-1" "lsp.observe_diagnostics"
                   (soma-bridge-bench--alist
                    (cons "buffer_name" (buffer-name buffer))
                    (cons "settle_seconds" 0.0))))
            (setq diagnostics-response
                  (soma-gym-bridge-submit-request diagnostics-request))
            (setq diagnostics-result
                  (soma-bridge-bench--get diagnostics-response "result"))
            (let ((soma-gym-bridge-async-command-delay-seconds 0.01))
              (setq rename-request
                    (soma-bridge-bench--api-request
                     "lsp-host-rename-1" "lsp.rename_symbol"
                     (soma-bridge-bench--alist
                      (cons "buffer_name" (buffer-name buffer))
                      (cons "new_name" "compute_answer")
                      (cons "timeout_seconds" 0.1)
                      (cons "preconditions" observe-result))))
              (setq rename-response
                    (soma-gym-bridge-submit-request rename-request))
              (setq await-request
                    (soma-bridge-bench--api-request
                     "lsp-host-rename-await-1" "request.await"
                     (soma-bridge-bench--alist
                      (cons "request_id" "lsp-host-rename-1")
                      (cons "timeout_seconds" 1.0))))
              (setq await-response
                    (soma-gym-bridge-submit-request await-request)))
            (setq rename-final
                  (or (soma-bridge-bench--get
                       (soma-bridge-bench--get await-response "result")
                       "status")
                      (soma-gym-bridge-request-status "lsp-host-rename-1")))
            (setq rename-result
                  (soma-bridge-bench--get rename-final "result"))
            (setq server-evidence
                  (soma-bridge-bench--get
                   (soma-bridge-bench--get observe-response "evidence")
                   "server"))
            (setq checks
                  (list
                   (soma-bridge-bench--check
                    "lsp.observe_source rejects a live server executable outside the selected rootfs"
                    (let ((server-executable
                           (or (soma-bridge-bench--get server-evidence
                                                       "executable")
                               "")))
                      (and (string= (soma-bridge-bench--get observe-response
                                                           "state")
                                    "failed")
                           (eq (soma-bridge-bench--get server-evidence
                                                       "rootfs_owned")
                               :json-false)
                           (not (soma-gym-bridge--rootfs-owned-tool-path-p
                                 server-executable))))
                    (format "%S" observe-response))
                   (soma-bridge-bench--check
                    "lsp.observe_diagnostics rejects a live server executable outside the selected rootfs"
                    (and (string= (soma-bridge-bench--get
                                   diagnostics-response "state")
                                  "failed")
                         (string-match-p
                          "rootfs-owned LSP server"
                          (or (soma-bridge-bench--get diagnostics-result
                                                      "error")
                              "")))
                    (format "%S" diagnostics-response))
                   (soma-bridge-bench--check
                    "lsp.rename_symbol fails before LSP requests on a host-owned server"
                    (and (string= (soma-bridge-bench--get rename-final "state")
                                  "failed")
                         (string= (soma-bridge-bench--get rename-result
                                                         "route")
                                  "precondition-check")
                         (not (member "rename" call-log)))
                    (format "%S" rename-final))
                   (soma-bridge-bench--check
                    "host-owned LSP rejection does not flush, pull diagnostics, or request rename"
                    (not (seq-intersection
                          call-log
                          '("flush-delayed-changes" "pull-diagnostics"
                            "symbol-request" "rename")
                          #'string=))
                    (format "%S" call-log)))))
          (soma-bridge-bench--alist
           (cons "host_lsp_path" host-lsp-path)
           (cons "observe_request" observe-request)
           (cons "observe_response" observe-response)
           (cons "diagnostics_request" diagnostics-request)
           (cons "diagnostics_response" diagnostics-response)
           (cons "rename_request" rename-request)
           (cons "rename_response" rename-response)
           (cons "await_request" await-request)
           (cons "await_response" await-response)
           (cons "call_log" call-log)
           (cons "checks" checks)))
      (when (process-live-p fake-process)
        (delete-process fake-process))
      (when (buffer-live-p buffer)
        (kill-buffer buffer))
      (ignore-errors (delete-directory tmp t))
      (ignore-errors (delete-directory host-dir t)))))

(defun soma-bridge-bench--scenario-observe-session ()
  (soma-gym-bridge-reset-transport)
  (let* ((request
          (soma-bridge-bench--api-request
           "session-observe-1" "emacs.observe_session"))
         (response (soma-gym-bridge-submit-request request))
         (status (soma-gym-bridge-request-status "session-observe-1"))
         (result (soma-bridge-bench--get response "result"))
	         (evidence (soma-bridge-bench--get response "evidence"))
	         (observations (soma-bridge-bench--get response "observations"))
	         (effects (soma-bridge-bench--get response "effects"))
         (session (and (listp result)
                       (soma-bridge-bench--get result "session")))
         (rootfs (and (listp result)
                      (soma-bridge-bench--get result "rootfs")))
         (doom (and (listp result)
                    (soma-bridge-bench--get result "doom")))
         (editor (and (listp result)
                      (soma-bridge-bench--get result "editor")))
         (selected (and (listp result)
                        (soma-bridge-bench--get result "selected_frame")))
         (plan (soma-bridge-bench--plan))
         (checks
          (list
           (soma-bridge-bench--check
            "emacs.observe_session succeeds through the v0 envelope"
            (and (string= (soma-bridge-bench--get response "api_version")
                          "soma-emacs-agent-v0")
                 (string= (soma-bridge-bench--get response "state")
                          "succeeded")
                 (equal (soma-bridge-bench--get response "state_sequence")
                        '("accepted" "running" "succeeded")))
            (format "%S" response))
	           (soma-bridge-bench--check
	            "emacs.observe_session publishes typed session, rootfs, Doom, and frame sections"
	            (let ((editor-session
	                   (soma-bridge-bench--get evidence "editor_session"))
	                  (rootfs-identity
	                   (soma-bridge-bench--get evidence "rootfs_identity"))
	                  (doom-identity
	                   (soma-bridge-bench--get evidence "doom_identity"))
	                  (selected-frame
	                   (soma-bridge-bench--get observations "selected_frame")))
	              (and (string= (soma-bridge-bench--get
	                              editor-session "api_version")
	                             "soma-emacs-agent-v0")
	                   (string-match-p
	                    "GNU Emacs 30\\.2"
	                    (or (soma-bridge-bench--get
	                         editor-session "version")
	                        ""))
	                   (string= (soma-bridge-bench--get
	                             rootfs-identity "mode")
	                            "selected-container-rootfs")
	                   (string= (soma-bridge-bench--get
	                             doom-identity "doom_dir")
	                            "/project/home/.doom.d/")
	                   (stringp (soma-bridge-bench--get
	                             selected-frame "buffer_name"))
	                   (and (hash-table-p effects)
	                        (= (hash-table-count effects) 0))))
	            (format "%S" response))
           (soma-bridge-bench--check
            "session observation echoes run id and selected rootfs mode"
            (and (string= (soma-bridge-bench--get result "operation")
                          "emacs.observe_session")
                 (string= (soma-bridge-bench--get rootfs "run_id")
                          (or (getenv "SOMA_INSULA_RUN_ID") ""))
                 (string= (soma-bridge-bench--get rootfs "mode")
                          "selected-container-rootfs")
                 (string= (soma-bridge-bench--get rootfs "dependency_closure")
                          "rootfs_owned")
                 (string= (soma-bridge-bench--get rootfs "glibc_owner")
                          "container-image-rootfs"))
            (format "%S" rootfs))
           (soma-bridge-bench--check
            "session observation reports the selected Insula plan identity"
            (and (string= (soma-bridge-bench--get rootfs "plan_path")
                          (or (getenv "SOMA_INSULA_PLAN") ""))
                 (string= (soma-bridge-bench--get rootfs "rootfs_path")
                          (or (soma-bridge-bench--get plan "rootfs") ""))
                 (string= (soma-bridge-bench--get rootfs "rootfs_store_id")
                          (or (soma-bridge-bench--get plan "rootfs_store_id")
                              (and (soma-bridge-bench--get plan "rootfs")
                                   (file-name-nondirectory
                                    (directory-file-name
                                     (soma-bridge-bench--get plan "rootfs"))))
                              "")))
            (format "%S" rootfs))
           (soma-bridge-bench--check
            "session observation reports source-built Emacs identity"
            (and (stringp (soma-bridge-bench--get editor "version"))
                 (string-match-p "GNU Emacs 30\\.2"
                                 (soma-bridge-bench--get editor "version"))
                 (string= (soma-bridge-bench--get editor "executable")
                          "/opt/soma/emacs/bin/emacs")
                 (string= (soma-bridge-bench--get editor "source_directory")
                          "/opt/soma/build/emacs-30.2/"))
            (format "%S" editor))
           (soma-bridge-bench--check
            "session observation reports rootfs-owned Doom identity"
            (and (string= (soma-bridge-bench--get doom "emacs_dir")
                          "/project/home/.emacs.d/")
                 (string= (soma-bridge-bench--get doom "doom_dir")
                          "/project/home/.doom.d/")
                 (stringp (soma-bridge-bench--get doom "commit"))
                 (= (length (soma-bridge-bench--get doom "commit")) 40)
                 (eq (soma-bridge-bench--get doom "emacs_dir_exists") t)
                 (eq (soma-bridge-bench--get doom "doom_dir_exists") t))
            (format "%S" doom))
           (soma-bridge-bench--check
            "session observation reports selected frame and buffer context"
            (and (numberp (soma-bridge-bench--get selected "frame_id"))
                 (numberp (soma-bridge-bench--get selected "window_id"))
                 (stringp (soma-bridge-bench--get selected "buffer_name"))
                 (string= (soma-bridge-bench--get session "api_version")
                          "soma-emacs-agent-v0"))
            (format "%S" selected))
           (soma-bridge-bench--check
            "session observation status is stored by request id"
            (and (string= (soma-bridge-bench--get status "request_id")
                          "session-observe-1")
                 (string= (soma-bridge-bench--get status "state")
                          "succeeded"))
            (format "%S" status)))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "status" status)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-request-envelope-validation ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-envelope-validation" "alpha\n"))
         (observed (soma-gym-bridge-observe-buffer buffer))
         (full-preconditions
          (soma-bridge-bench--alist
           (cons "buffer_name"
                 (soma-bridge-bench--get observed "buffer_name"))
           (cons "text_owner_name"
                 (soma-bridge-bench--get observed "text_owner_name"))
           (cons "buffer_file_name"
                 (soma-bridge-bench--get observed "buffer_file_name"))
           (cons "chars_tick"
                 (soma-bridge-bench--get observed "chars_tick"))
           (cons "expected_text" "alpha")
           (cons "point" (soma-bridge-bench--get observed "point"))
           (cons "selected_window_buffer"
                 (soma-bridge-bench--get observed "selected_window_buffer"))
           (cons "modified" (soma-bridge-bench--get observed "modified"))
           (cons "read_only" (soma-bridge-bench--get observed "read_only"))
           (cons "minibuffer_active"
                 (soma-bridge-bench--get observed "minibuffer_active"))))
         (valid-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-valid-1")
           (cons "operation" "emacs.observe_buffer")
           (cons "capability" "passive_observation")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer))))))
         (valid-response
          (soma-gym-bridge-submit-request valid-request))
         (invalid-version-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v999")
           (cons "request_id" "envelope-invalid-version-1")
           (cons "operation" "emacs.observe_buffer")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer))))))
         (invalid-version-response
          (soma-gym-bridge-submit-request invalid-version-request))
         (capability-mismatch-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-capability-mismatch-1")
           (cons "operation" "emacs.observe_buffer")
           (cons "capability" "kernel_execution")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer))))))
         (capability-mismatch-response
          (soma-gym-bridge-submit-request capability-mismatch-request))
         (invalid-preconditions-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-invalid-preconditions-1")
           (cons "operation" "buffer.replace_range")
           (cons "preconditions" "not-an-object")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer))
                  (cons "start" 1)
                  (cons "end" 6)
                  (cons "text" "omega\n")))))
         (invalid-preconditions-response
          (soma-gym-bridge-submit-request invalid-preconditions-request))
         (missing-param-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-missing-param-1")
           (cons "operation" "buffer.replace_range")
           (cons "capability" "reversible_editor_edit")
           (cons "preconditions" full-preconditions)
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer))
                  (cons "start" 1)
                  (cons "end" 6)))))
         (missing-param-response
          (soma-gym-bridge-submit-request missing-param-request))
         (missing-precondition-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-missing-precondition-1")
           (cons "operation" "buffer.replace_range")
           (cons "capability" "reversible_editor_edit")
           (cons "preconditions"
                 (soma-bridge-bench--without-key
                  full-preconditions "chars_tick"))
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer))
                  (cons "start" 1)
                  (cons "end" 6)
                  (cons "text" "omega")))))
         (missing-precondition-response
          (soma-gym-bridge-submit-request missing-precondition-request))
         (benchmark-only-defer-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-benchmark-only-defer-1")
           (cons "operation" "bridge.defer_echo")
           (cons "capability" "async_fixture")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "delay_seconds" 0.01)
                  (cons "value" "must-not-run-as-v0")))))
         (benchmark-only-defer-response
          (soma-gym-bridge-submit-request benchmark-only-defer-request))
         (benchmark-only-finish-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-benchmark-only-finish-1")
           (cons "operation" "episode.finish")
           (cons "capability" "private_grader")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "required_requests" nil)
                  (cons "final_buffer_checks" nil)))))
         (benchmark-only-finish-response
          (soma-gym-bridge-submit-request benchmark-only-finish-request))
         (experimental-respond-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-experimental-respond-1")
           (cons "operation" "request.respond")
           (cons "capability" "continuation_response")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "request_id" "envelope-prompt-1")
                  (cons "response" "must-not-run-as-v0")))))
         (experimental-respond-response
          (soma-gym-bridge-submit-request experimental-respond-request))
         (defer-request
          (soma-bridge-bench--alist
           (cons "request_id" "envelope-defer-1")
           (cons "operation" "bridge.defer_echo")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "delay_seconds" 0.01)
                  (cons "value" "timeout-ms-ok")))))
         (defer-response
          (soma-gym-bridge-submit-request defer-request))
         (await-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-await-1")
           (cons "operation" "request.await")
           (cons "capability" "passive_observation")
           (cons "timeout_ms" 1000)
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "request_id" "envelope-defer-1")))))
         (await-response
          (soma-gym-bridge-submit-request await-request))
         (timeout-defer-request
          (soma-bridge-bench--alist
           (cons "request_id" "envelope-timeout-defer-1")
           (cons "operation" "bridge.defer_echo")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "delay_seconds" 0.2)
                  (cons "value" "timeout-diagnostic-ok")))))
         (timeout-defer-response
          (soma-gym-bridge-submit-request timeout-defer-request))
         (timeout-await-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-timeout-await-1")
           (cons "operation" "request.await")
           (cons "capability" "passive_observation")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "request_id" "envelope-timeout-defer-1")
                  (cons "timeout_seconds" 0.0)))))
         (timeout-await-response
          (soma-gym-bridge-submit-request timeout-await-request))
         (false-error-diagnostic-response
          (soma-gym-bridge--response
           "envelope-false-error-diagnostic-1"
           "request.await"
           "outcome_unknown"
           '("accepted" "running" "outcome_unknown")
           nil
           "timeout"
           :json-false))
         (jupyter-outcome-target-response
          (soma-gym-bridge--response
           "envelope-jupyter-outcome-target-1"
           "jupyter.eval_region"
           "outcome_unknown"
           '("accepted" "running" "outcome_unknown")
           (soma-bridge-bench--alist
            (cons "operation" "jupyter.eval_region")
            (cons "state" "outcome_unknown")
            (cons "request_id" "kernel-request-1")
            (cons "reply_status" "ok")
            (cons "stream_texts" '("SOMA_SELECTED_JSON:{}"))
            (cons "error" :json-false))
           "timeout"
           :json-false))
         (_stored-jupyter-outcome-target
          (puthash "envelope-jupyter-outcome-target-1"
                   jupyter-outcome-target-response
                   soma-gym-bridge--requests))
         (jupyter-outcome-await-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-jupyter-outcome-await-1")
           (cons "operation" "jupyter.await_request")
           (cons "capability" "passive_observation")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "request_id" "envelope-jupyter-outcome-target-1")
                  (cons "timeout_seconds" 0.0)))))
         (jupyter-outcome-await-response
          (soma-gym-bridge-submit-request jupyter-outcome-await-request))
         (invalid-timeout-request
          (soma-bridge-bench--alist
           (cons "api_version" "soma-emacs-agent-v0")
           (cons "request_id" "envelope-invalid-timeout-1")
           (cons "operation" "request.await")
           (cons "timeout_ms" -1)
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "request_id" "envelope-defer-1")))))
         (invalid-timeout-response
          (soma-gym-bridge-submit-request invalid-timeout-request))
         (checks
          (list
           (soma-bridge-bench--check
            "v0 request envelope with matching capability succeeds"
            (string= (soma-bridge-bench--get valid-response "state")
                     "succeeded")
            (format "%S" valid-response))
           (soma-bridge-bench--check
            "unsupported api_version is rejected"
            (and (string= (soma-bridge-bench--get
                           invalid-version-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get
                           invalid-version-response "error_code")
                          "invalid_api_version"))
            (format "%S" invalid-version-response))
           (soma-bridge-bench--check
            "capability mismatch is rejected"
            (and (string= (soma-bridge-bench--get
                           capability-mismatch-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get
                           capability-mismatch-response "error_code")
                          "capability_mismatch"))
            (format "%S" capability-mismatch-response))
           (soma-bridge-bench--check
            "invalid top-level preconditions are rejected"
            (and (string= (soma-bridge-bench--get
                           invalid-preconditions-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get
                           invalid-preconditions-response "error_code")
                          "invalid_preconditions"))
            (format "%S" invalid-preconditions-response))
           (soma-bridge-bench--check
            "explicit v0 requests reject missing manifest-required params"
            (and (string= (soma-bridge-bench--get
                           missing-param-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get
                           missing-param-response "error_code")
                          "missing_required_param"))
            (format "%S" missing-param-response))
           (soma-bridge-bench--check
            "explicit v0 requests reject missing manifest-required preconditions"
            (and (string= (soma-bridge-bench--get
                           missing-precondition-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get
                           missing-precondition-response "error_code")
                          "missing_required_precondition"))
            (format "%S" missing-precondition-response))
           (soma-bridge-bench--check
            "explicit v0 requests reject benchmark-only async helpers"
            (and (string= (soma-bridge-bench--get
                           benchmark-only-defer-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get
                           benchmark-only-defer-response "error_code")
                          "benchmark_only_operation"))
            (format "%S" benchmark-only-defer-response))
           (soma-bridge-bench--check
            "explicit v0 requests reject benchmark-only episode graders"
            (and (string= (soma-bridge-bench--get
                           benchmark-only-finish-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get
                           benchmark-only-finish-response "error_code")
                          "benchmark_only_operation"))
            (format "%S" benchmark-only-finish-response))
           (soma-bridge-bench--check
            "explicit v0 requests reject experimental request continuation"
            (and (string= (soma-bridge-bench--get
                           experimental-respond-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get
                           experimental-respond-response "error_code")
                          "non_stable_operation"))
            (format "%S" experimental-respond-response))
           (soma-bridge-bench--check
            "timeout_ms drives request.await timeout semantics"
            (and (string= (soma-bridge-bench--get defer-response "state")
                          "running")
                 (string= (soma-bridge-bench--get await-response "state")
                          "succeeded")
                 (string= (soma-bridge-bench--get
                           (soma-bridge-bench--get
                            (soma-bridge-bench--get await-response "result")
                            "status")
                           "state")
                          "succeeded"))
            (format "%S" await-response))
           (soma-bridge-bench--check
            "request.await timeout diagnostic has a string message"
            (let ((diagnostics
                   (soma-bridge-bench--get timeout-await-response
                                           "diagnostics")))
              (and (string= (soma-bridge-bench--get
                             timeout-defer-response "state")
                            "running")
                   (string= (soma-bridge-bench--get
                             timeout-await-response "state")
                            "outcome_unknown")
                   (vectorp diagnostics)
                   (= (length diagnostics) 1)
                   (string= (soma-bridge-bench--get
                             (aref diagnostics 0) "code")
                            "timeout")
                   (stringp (soma-bridge-bench--get
                             (aref diagnostics 0) "message"))))
            (format "%S" timeout-await-response))
           (soma-bridge-bench--check
            "false legacy errors become string diagnostic messages"
            (let ((diagnostics
                   (soma-bridge-bench--get
                    false-error-diagnostic-response "diagnostics")))
              (and (string= (soma-bridge-bench--get
                             false-error-diagnostic-response "state")
                            "outcome_unknown")
                   (vectorp diagnostics)
                   (= (length diagnostics) 1)
                   (string= (soma-bridge-bench--get
                             (aref diagnostics 0) "code")
                            "timeout")
                   (stringp (soma-bridge-bench--get
                             (aref diagnostics 0) "message"))))
            (format "%S" false-error-diagnostic-response))
           (soma-bridge-bench--check
            "jupyter.await_request preserves outcome_unknown target state"
            (let* ((result
                    (soma-bridge-bench--get
                     jupyter-outcome-await-response "result"))
                   (status (soma-bridge-bench--get result "status"))
                   (diagnostics
                    (soma-bridge-bench--get
                     jupyter-outcome-await-response "diagnostics")))
              (and (string= (soma-bridge-bench--get
                             jupyter-outcome-await-response "state")
                            "outcome_unknown")
                   (string= (soma-bridge-bench--get
                             jupyter-outcome-await-response "error_code")
                            "timeout")
                   (string= (soma-bridge-bench--get result "target_state")
                            "outcome_unknown")
                   (string= (soma-bridge-bench--get status "state")
                            "outcome_unknown")
                   (vectorp diagnostics)
                   (= (length diagnostics) 1)
                   (stringp (soma-bridge-bench--get
                             (aref diagnostics 0) "message"))))
            (format "%S" jupyter-outcome-await-response))
           (soma-bridge-bench--check
            "invalid timeout_ms is rejected"
            (and (string= (soma-bridge-bench--get
                           invalid-timeout-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get
                           invalid-timeout-response "error_code")
                          "invalid_timeout_ms"))
            (format "%S" invalid-timeout-response)))))
    (soma-bridge-bench--alist
     (cons "valid_request" valid-request)
     (cons "valid_response" valid-response)
     (cons "invalid_version_request" invalid-version-request)
     (cons "invalid_version_response" invalid-version-response)
     (cons "capability_mismatch_request" capability-mismatch-request)
     (cons "capability_mismatch_response" capability-mismatch-response)
     (cons "invalid_preconditions_request" invalid-preconditions-request)
     (cons "invalid_preconditions_response" invalid-preconditions-response)
     (cons "missing_param_request" missing-param-request)
     (cons "missing_param_response" missing-param-response)
     (cons "missing_precondition_request" missing-precondition-request)
     (cons "missing_precondition_response" missing-precondition-response)
     (cons "benchmark_only_defer_request" benchmark-only-defer-request)
     (cons "benchmark_only_defer_response" benchmark-only-defer-response)
     (cons "benchmark_only_finish_request" benchmark-only-finish-request)
     (cons "benchmark_only_finish_response" benchmark-only-finish-response)
     (cons "experimental_respond_request" experimental-respond-request)
     (cons "experimental_respond_response" experimental-respond-response)
     (cons "defer_request" defer-request)
     (cons "defer_response" defer-response)
     (cons "await_request" await-request)
     (cons "await_response" await-response)
     (cons "timeout_defer_request" timeout-defer-request)
     (cons "timeout_defer_response" timeout-defer-response)
     (cons "timeout_await_request" timeout-await-request)
     (cons "timeout_await_response" timeout-await-response)
     (cons "false_error_diagnostic_response" false-error-diagnostic-response)
     (cons "jupyter_outcome_target_response" jupyter-outcome-target-response)
     (cons "jupyter_outcome_await_request" jupyter-outcome-await-request)
     (cons "jupyter_outcome_await_response" jupyter-outcome-await-response)
     (cons "invalid_timeout_request" invalid-timeout-request)
     (cons "invalid_timeout_response" invalid-timeout-response)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-project-api ()
  (soma-gym-bridge-reset-transport)
  (let* ((tmp (make-temp-file "soma-bridge-project-api-" t))
         (src-dir (expand-file-name "src" tmp))
         (subject-file (expand-file-name "src/subject.py" tmp))
         (other-file (expand-file-name "README.md" tmp))
         (plan (soma-bridge-bench--plan))
         (plan-rg (soma-bridge-bench--get
                   (soma-bridge-bench--get plan "tool_inventory")
                   "rg"))
         subject-buffer manifest observe-entry search-entry
         observe-request observe-response observe-result
         search-request search-response search-result
         host-rg-parent host-rg-dir host-rg-path host-rg-marker
	         host-rg-request host-rg-response host-rg-result
	         host-git-dir host-git-path host-git-marker
	         host-git-request host-git-response host-git-result host-git-vcs
	         checks)
    (unwind-protect
        (progn
          (make-directory src-dir t)
          (soma-bridge-bench--write-file
           subject-file
           (concat
            "def project_probe():\n"
            "    return 'soma-project-needle'\n"))
          (soma-bridge-bench--write-file
           other-file
           "# Soma project API fixture\n")
          (soma-bridge-bench--call-git tmp "init" "-q")
          (soma-bridge-bench--call-git tmp "add" "src/subject.py" "README.md")
          (setq subject-buffer (find-file-noselect subject-file))
          (set-window-buffer (selected-window) subject-buffer)
          (setq manifest (soma-gym-bridge-list-capabilities))
          (setq observe-entry
                (soma-bridge-bench--find-operation
                 manifest "project.observe"))
          (setq search-entry
                (soma-bridge-bench--find-operation
                 manifest "project.search"))
          (setq observe-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "project-observe-1")
                 (cons "operation" "project.observe")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name subject-buffer))
                        (cons "root" tmp)))))
          (setq observe-response
                (soma-gym-bridge-submit-request observe-request))
          (setq observe-result
                (soma-bridge-bench--get observe-response "result"))
          (setq search-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "project-search-1")
                 (cons "operation" "project.search")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "root" tmp)
                        (cons "query" "soma-project-needle")
                        (cons "max_results" 10)
                        (cons "max_text" 200)))))
          (setq search-response
                (soma-gym-bridge-submit-request search-request))
          (setq search-result
                (soma-bridge-bench--get search-response "result"))
          (setq host-rg-parent
                (expand-file-name
                 ".scratch/emacs-gym/host-shadow-tools/"
                 (file-name-as-directory
                  (or (locate-dominating-file default-directory ".git")
                      default-directory))))
          (make-directory host-rg-parent t)
          (setq host-rg-dir
                (make-temp-file
                 (expand-file-name "soma-bridge-host-rg-" host-rg-parent)
                 t))
          (setq host-rg-path (expand-file-name "rg" host-rg-dir))
          (setq host-rg-marker (expand-file-name "called" host-rg-dir))
          (soma-bridge-bench--write-file
           host-rg-path
           (format
            "#!/bin/sh\nprintf 'called\\n' >> %s\nprintf 'HOST_RG_SHOULD_NOT_RUN\\n'\nexit 0\n"
            (shell-quote-argument host-rg-marker)))
          (set-file-modes host-rg-path #o755)
          (setq host-rg-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "project-search-host-rg-rejected")
                 (cons "operation" "project.search")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "root" tmp)
                        (cons "query" "soma-project-needle")
                        (cons "max_results" 10)
                        (cons "max_text" 200)))))
          (let ((exec-path (list host-rg-dir))
                (process-environment
                 (cons (concat "PATH=" host-rg-dir)
                       process-environment)))
            (setq host-rg-response
                  (soma-gym-bridge-submit-request host-rg-request)))
          (setq host-rg-result
                (soma-bridge-bench--get host-rg-response "result"))
          (setq host-git-dir
                (make-temp-file
                 (expand-file-name "soma-bridge-host-git-" host-rg-parent)
                 t))
          (setq host-git-path (expand-file-name "git" host-git-dir))
          (setq host-git-marker (expand-file-name "called" host-git-dir))
          (soma-bridge-bench--write-file
           host-git-path
           (format
            "#!/bin/sh\nprintf 'called %%s\\n' \"$*\" >> %s\nexit 0\n"
            (shell-quote-argument host-git-marker)))
          (set-file-modes host-git-path #o755)
          (setq host-git-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "project-observe-host-git-rejected")
                 (cons "operation" "project.observe")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "root" tmp)))))
          (let ((exec-path (list host-git-dir))
                (process-environment
                 (cons (concat "PATH=" host-git-dir)
                       process-environment)))
            (setq host-git-response
                  (soma-gym-bridge-submit-request host-git-request)))
          (setq host-git-result
                (soma-bridge-bench--get host-git-response "result"))
          (setq host-git-vcs
                (soma-bridge-bench--get host-git-result "vcs"))
          (setq checks
                (list
                 (soma-bridge-bench--check
                  "project operations are stable and implemented"
                  (and (string= (soma-bridge-bench--get
                                 observe-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            observe-entry "implemented")
                           t)
                       (string= (soma-bridge-bench--get
                                 search-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            search-entry "implemented")
	                           t)
	                       (member "search_tool_identity"
	                               (soma-bridge-bench--get
	                                observe-entry "evidence"))
	                       (member "rootfs_runtime"
	                               (soma-bridge-bench--get
	                                observe-entry "evidence"))
	                       (member "rootfs_runtime"
	                               (soma-bridge-bench--get
	                                search-entry "evidence")))
                  (format "%S" (list observe-entry search-entry)))
                 (soma-bridge-bench--check
                  "project.observe succeeds through the v0 envelope"
                  (string= (soma-bridge-bench--get observe-response "state")
                           "succeeded")
                  (format "%S" observe-response))
                 (soma-bridge-bench--check
                  "project.observe reports Git project and rootfs search tool"
                  (and (listp observe-result)
                       (string= (file-name-as-directory
                                 (file-truename tmp))
                                (soma-bridge-bench--get
                                 observe-result "project_root"))
                       (string= (soma-bridge-bench--get
                                 (soma-bridge-bench--get
                                  observe-result "vcs")
                                 "type")
                                "git")
                       (string= (soma-bridge-bench--get
                                 (soma-bridge-bench--get
                                  observe-result "search_tool")
                                 "path")
                                (soma-bridge-bench--get plan-rg "path")))
                  (format "%S" observe-result))
	                 (soma-bridge-bench--check
	                  "project.observe publishes typed project context sections"
	                  (let* ((evidence (soma-bridge-bench--get
	                                    observe-response "evidence"))
	                         (observations (soma-bridge-bench--get
	                                        observe-response "observations"))
	                         (effects (soma-bridge-bench--get
	                                   observe-response "effects"))
	                         (project-root (soma-bridge-bench--get
	                                        evidence "project_root"))
	                         (vcs-identity (soma-bridge-bench--get
	                                        evidence "vcs_identity"))
	                         (search-tool (soma-bridge-bench--get
	                                       evidence "search_tool_identity"))
	                         (runtime (soma-bridge-bench--get
	                                   evidence "rootfs_runtime"))
	                         (project-context (soma-bridge-bench--get
	                                           observations
	                                           "project_context")))
	                    (and (string= (soma-bridge-bench--get
	                                    project-root "path")
	                                   (file-name-as-directory
	                                    (file-truename tmp)))
	                         (string= (soma-bridge-bench--get
	                                   vcs-identity "type")
	                                  "git")
	                         (string= (soma-bridge-bench--get
	                                   search-tool "path")
	                                  (soma-bridge-bench--get
	                                   plan-rg "path"))
	                         (string= (soma-bridge-bench--get
	                                   runtime "run_id")
	                                  (or (getenv "SOMA_INSULA_RUN_ID") ""))
	                         (string= (soma-bridge-bench--get
	                                   project-context "buffer_name")
	                                  (buffer-name subject-buffer))
	                         (and (hash-table-p effects)
	                              (= (hash-table-count effects) 0))))
	                  (format "%S" observe-response))
                 (soma-bridge-bench--check
                  "project.search succeeds through rootfs-owned ripgrep"
                  (and (string= (soma-bridge-bench--get search-response
                                                        "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get
                                 (soma-bridge-bench--get
                                  search-result "search_tool")
                                 "path")
                                (soma-bridge-bench--get plan-rg "path")))
                  (format "%S" search-result))
	                 (soma-bridge-bench--check
	                  "project.search publishes typed search evidence and match observations"
	                  (let* ((evidence (soma-bridge-bench--get
	                                    search-response "evidence"))
	                         (observations (soma-bridge-bench--get
	                                        search-response "observations"))
	                         (effects (soma-bridge-bench--get
	                                   search-response "effects"))
	                         (search-tool (soma-bridge-bench--get
	                                       evidence "search_tool_identity"))
	                         (bounded-results (soma-bridge-bench--get
	                                           evidence "bounded_results"))
	                         (runtime (soma-bridge-bench--get
	                                   evidence "rootfs_runtime"))
	                         (matches (soma-bridge-bench--get
	                                   observations "matches"))
	                         (results (soma-bridge-bench--get
	                                   matches "results")))
	                    (and (string= (soma-bridge-bench--get
	                                    search-tool "path")
	                                   (soma-bridge-bench--get plan-rg
	                                                           "path"))
	                         (= (or (soma-bridge-bench--get
	                                 bounded-results "result_count")
	                                0)
	                            1)
	                         (= (or (soma-bridge-bench--get
	                                 bounded-results "results_returned")
	                                0)
	                            1)
	                         (eq (soma-bridge-bench--get
	                              bounded-results "results_truncated")
	                             :json-false)
	                         (string= (soma-bridge-bench--get
	                                   runtime "run_id")
	                                  (or (getenv "SOMA_INSULA_RUN_ID") ""))
	                         (string= (soma-bridge-bench--get
	                                   matches "query")
	                                  "soma-project-needle")
	                         (string= (soma-bridge-bench--get
	                                   (car results) "path")
	                                  (file-truename subject-file))
	                         (and (hash-table-p effects)
	                              (= (hash-table-count effects) 0))))
	                  (format "%S" search-response))
                 (soma-bridge-bench--check
                  "project.search returns bounded source match evidence"
                  (let ((results (soma-bridge-bench--get
                                  search-result "results")))
                    (and (= (or (soma-bridge-bench--get
                                 search-result "result_count")
                                0)
                            1)
                         (string= (soma-bridge-bench--get
                                   (car results) "path")
                                  (file-truename subject-file))
                         (= (or (soma-bridge-bench--get
                                 (car results) "line")
                                0)
                            2)
                         (string-match-p
                          "soma-project-needle"
                          (or (soma-bridge-bench--get
                               (car results) "text")
                              ""))))
                  (format "%S" search-result))
                 (soma-bridge-bench--check
                  "project.search returns structured submatch evidence"
                  (let* ((results (soma-bridge-bench--get
                                   search-result "results"))
                         (submatches (soma-bridge-bench--get
                                      (car results) "submatches"))
                         (first-submatch (car submatches))
                         (match (soma-bridge-bench--get
                                 first-submatch "match")))
                    (and (consp submatches)
                         (listp first-submatch)
                         (string= (or (soma-bridge-bench--get
                                       match "text")
                                      "")
                                  "soma-project-needle")
                         (= (or (soma-bridge-bench--get
                                 first-submatch "start")
                                -1)
                            12)
                         (= (or (soma-bridge-bench--get
                                 first-submatch "end")
                                -1)
                            31)))
	                  (format "%S" search-result))
                 (soma-bridge-bench--check
                  "project.search rejects executable rg outside the selected rootfs"
                  (and (string= (soma-bridge-bench--get host-rg-response
                                                        "state")
                                "failed")
                       (string= (soma-bridge-bench--get host-rg-result
                                                        "error")
                                "rootfs-owned ripgrep is not available")
                       (eq (soma-bridge-bench--get
                            (soma-bridge-bench--get host-rg-result
                                                     "search_tool")
                            "rootfs_owned")
                           :json-false))
	                  (format "%S" host-rg-response))
                 (soma-bridge-bench--check
                  "project.search does not execute rejected rg for version probing"
                  (let ((search-tool (soma-bridge-bench--get
                                      host-rg-result "search_tool")))
                    (and (not (file-exists-p host-rg-marker))
                         (string= (or (soma-bridge-bench--get
                                       search-tool "version")
                                      "")
                                  "")))
	                  (format "%S" host-rg-response))
                 (soma-bridge-bench--check
                  "project.observe does not execute rejected git for VCS probing"
                  (let ((git-tool (soma-bridge-bench--get
                                   host-git-vcs "tool")))
                    (and (string= (soma-bridge-bench--get
                                   host-git-response "state")
                                  "succeeded")
                         (not (file-exists-p host-git-marker))
                         (eq (soma-bridge-bench--get
                              host-git-vcs "type")
                             :json-false)
                         (eq (soma-bridge-bench--get
                              git-tool "rootfs_owned")
                             :json-false)
                         (string= (or (soma-bridge-bench--get
                                       git-tool "version")
                                      "")
                                  "")))
	                  (format "%S" host-git-response))))
          (soma-bridge-bench--alist
           (cons "observe_request" observe-request)
           (cons "observe_response" observe-response)
           (cons "search_request" search-request)
           (cons "search_response" search-response)
           (cons "host_rg_path" host-rg-path)
	           (cons "host_rg_marker" host-rg-marker)
           (cons "host_rg_request" host-rg-request)
           (cons "host_rg_response" host-rg-response)
	           (cons "host_git_path" host-git-path)
	           (cons "host_git_marker" host-git-marker)
	           (cons "host_git_request" host-git-request)
	           (cons "host_git_response" host-git-response)
           (cons "checks" checks)))
      (when (buffer-live-p subject-buffer)
        (kill-buffer subject-buffer)))))

(defun soma-bridge-bench--scenario-reports-vcs-files-api ()
  (soma-gym-bridge-reset-transport)
  (let* ((tmp (make-temp-file "soma-bridge-reports-files-" t))
         (report-file (expand-file-name "report.org" tmp))
         (export-file (expand-file-name "published/report.html" tmp))
         (source-file (expand-file-name "source.txt" tmp))
         (renamed-file (expand-file-name "renamed.txt" tmp))
         (collision-source-file (expand-file-name "collision-source.txt" tmp))
         (collision-file (expand-file-name "collision.txt" tmp))
         (delete-a-file (expand-file-name "delete-a.txt" tmp))
         (delete-b-file (expand-file-name "delete-b.txt" tmp))
         (flagged-decoy-file (expand-file-name "flagged-decoy.txt" tmp))
         (stale-dir (expand-file-name "stale-delete" tmp))
         stale-file stale-before-signature
         report-buffer dired-buffer stale-dired-buffer
         manifest org-observe-entry org-update-entry org-edit-entry
         org-exit-entry org-babel-entry org-export-entry
         dired-observe-entry dired-rename-entry dired-delete-entry
         org-observe-request
         org-observe-response org-observe-result org-heading
         org-source-block org-update-preconditions org-update-request
         org-update-response org-update-result org-after-update-request
         org-after-update-response org-after-update-result org-after-heading
         org-after-source-block org-stale-update-request
         org-stale-update-response org-stale-update-result
         org-edit-preconditions org-edit-request org-edit-response
         org-edit-result org-edit-buffer-name org-edit-buffer
         org-edit-buffer-opened-p org-edit-buffer-separate-p
         org-edit-buffer-initial-text
         org-edit-buffer-observe-request org-edit-buffer-observe-response
         org-edit-buffer-observe-result org-edit-replace-request
         org-edit-replace-response org-edit-replace-result
         org-exit-preconditions org-exit-request org-exit-response
         org-exit-result org-after-exit-request org-after-exit-response
         org-after-exit-result org-after-exit-heading
         org-after-exit-source-block org-stale-edit-request
         org-stale-edit-response org-stale-edit-result
         org-stale-edit-buffer-name org-stale-edit-buffer
         org-stale-edit-buffer-observe-request
         org-stale-edit-buffer-observe-response
         org-stale-edit-buffer-observe-result
         org-stale-replace-request org-stale-replace-response
         org-stale-exit-preconditions org-stale-exit-request
         org-stale-exit-response org-stale-exit-result
         org-before-babel-request org-before-babel-response
         org-before-babel-result org-before-babel-heading
         org-before-babel-source-block org-babel-preconditions
         org-babel-request org-babel-response org-babel-result
         org-after-babel-request org-after-babel-response
         org-after-babel-result org-after-babel-source-block
         org-stale-babel-request org-stale-babel-response
         org-stale-babel-result
         org-export-request org-export-response
         org-export-result dired-observe-request dired-observe-response
         dired-observe-result dired-rename-request dired-rename-response
         dired-rename-result collision-request collision-response
         collision-result dired-delete-request dired-delete-response
         dired-delete-result stale-delete-request stale-delete-response
         stale-delete-result checks)
    (unwind-protect
        (progn
          (soma-bridge-bench--write-file
           report-file
           (concat
            "#+title: Soma API Report\n"
            "* TODO Summary\n"
            ":PROPERTIES:\n"
            ":ID: report-summary\n"
            ":END:\n"
            "The bridge should preserve Org structure.\n\n"
            "#+begin_src emacs-lisp :results output replace\n"
            "(princ \"ok\\n\")\n"
            "#+end_src\n"))
          (soma-bridge-bench--write-file source-file "source\n")
          (soma-bridge-bench--write-file
           collision-source-file "collision-source\n")
          (soma-bridge-bench--write-file collision-file "collision-original\n")
          (soma-bridge-bench--write-file delete-a-file "delete-a\n")
          (soma-bridge-bench--write-file delete-b-file "delete-b\n")
          (soma-bridge-bench--write-file
           flagged-decoy-file "flagged-decoy\n")
          (setq stale-file (expand-file-name "replace-me.txt" stale-dir))
          (soma-bridge-bench--write-file stale-file "original-stale\n")
          (setq report-buffer (find-file-noselect report-file))
          (with-current-buffer report-buffer
            (org-mode)
            (goto-char (point-min))
            (set-buffer-modified-p nil))
          (set-window-buffer (selected-window) report-buffer)
          (setq dired-buffer (dired-noselect tmp))
          (with-current-buffer dired-buffer
            (revert-buffer))
          (setq stale-dired-buffer (dired-noselect stale-dir))
          (with-current-buffer stale-dired-buffer
            (revert-buffer))
          (setq manifest (soma-gym-bridge-list-capabilities))
          (setq org-observe-entry
                (soma-bridge-bench--find-operation
                 manifest "org.observe_tree"))
          (setq org-update-entry
                (soma-bridge-bench--find-operation
                 manifest "org.update_heading"))
          (setq org-edit-entry
                (soma-bridge-bench--find-operation
                 manifest "org.edit_src_block"))
          (setq org-exit-entry
                (soma-bridge-bench--find-operation
                 manifest "org.exit_src_edit"))
          (setq org-babel-entry
                (soma-bridge-bench--find-operation
                 manifest "org.execute_babel_block"))
          (setq org-export-entry
                (soma-bridge-bench--find-operation
                 manifest "org.export_html"))
          (setq dired-observe-entry
                (soma-bridge-bench--find-operation
                 manifest "dired.observe"))
          (setq dired-rename-entry
                (soma-bridge-bench--find-operation
                 manifest "dired.rename_no_overwrite"))
          (setq dired-delete-entry
                (soma-bridge-bench--find-operation
                 manifest "dired.delete_marked"))
          (setq org-observe-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-observe-1")
                 (cons "operation" "org.observe_tree")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "max_headings" 20)
                        (cons "max_source_blocks" 20)))))
          (setq org-observe-response
                (soma-gym-bridge-submit-request org-observe-request))
          (setq org-observe-result
                (soma-bridge-bench--get org-observe-response "result"))
          (setq org-heading
                (car (soma-bridge-bench--get org-observe-result "headings")))
          (setq org-source-block
                (car (soma-bridge-bench--get org-observe-result
                                             "source_blocks")))
          (setq org-update-preconditions
                (soma-bridge-bench--alist
                 (cons "buffer_name" (buffer-name report-buffer))
                 (cons "text_owner_name"
                       (soma-bridge-bench--get org-observe-result
                                               "text_owner_name"))
                 (cons "buffer_file_name"
                       (soma-bridge-bench--get org-observe-result
                                               "buffer_file_name"))
                 (cons "chars_tick"
                       (soma-bridge-bench--get org-observe-result
                                               "chars_tick"))
                 (cons "point"
                       (soma-bridge-bench--get org-observe-result "point"))
                 (cons "selected_window_buffer"
                       (soma-bridge-bench--get org-observe-result
                                               "selected_window_buffer"))
                 (cons "modified"
                       (soma-bridge-bench--get org-observe-result "modified"))
                 (cons "read_only"
                       (soma-bridge-bench--get org-observe-result "read_only"))
                 (cons "minibuffer_active"
                       (soma-bridge-bench--get org-observe-result
                                               "minibuffer_active"))
                 (cons "mark_active"
                       (soma-bridge-bench--get org-observe-result
                                               "mark_active"))
                 (cons "narrowed"
                       (soma-bridge-bench--get org-observe-result "narrowed"))
                 (cons "org_node" org-heading)
                 (cons "structure_fingerprint"
                       (soma-bridge-bench--get org-observe-result
                                               "structure_fingerprint"))))
          (setq org-update-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-update-heading-1")
                 (cons "operation" "org.update_heading")
                 (cons "capability" "reversible_editor_edit")
                 (cons "preconditions" org-update-preconditions)
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "heading" org-heading)
                        (cons "changes"
                              (soma-bridge-bench--alist
                               (cons "todo" "DONE")
                               (cons "title" "Updated Summary")))))))
          (setq org-update-response
                (soma-gym-bridge-submit-request org-update-request))
          (setq org-update-result
                (soma-bridge-bench--get org-update-response "result"))
          (setq org-after-update-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-after-update-observe-1")
                 (cons "operation" "org.observe_tree")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "max_headings" 20)
                        (cons "max_source_blocks" 20)))))
          (setq org-after-update-response
                (soma-gym-bridge-submit-request org-after-update-request))
          (setq org-after-update-result
                (soma-bridge-bench--get org-after-update-response "result"))
          (setq org-after-heading
                (car (soma-bridge-bench--get org-after-update-result
                                             "headings")))
          (setq org-after-source-block
                (car (soma-bridge-bench--get org-after-update-result
                                             "source_blocks")))
          (setq org-stale-update-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-update-heading-stale-1")
                 (cons "operation" "org.update_heading")
                 (cons "capability" "reversible_editor_edit")
                 (cons "preconditions" org-update-preconditions)
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "heading" org-heading)
                        (cons "changes"
                              (soma-bridge-bench--alist
                               (cons "title" "Stale Rewrite")))))))
          (setq org-stale-update-response
                (soma-gym-bridge-submit-request org-stale-update-request))
          (setq org-stale-update-result
                (soma-bridge-bench--get org-stale-update-response "result"))
          (setq org-edit-preconditions
                (soma-bridge-bench--org-preconditions
                 report-buffer org-after-update-result org-after-heading
                 org-after-source-block))
          (setq org-edit-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-edit-src-block-1")
                 (cons "operation" "org.edit_src_block")
                 (cons "capability" "editor_state_change")
                 (cons "preconditions" org-edit-preconditions)
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "source_block" org-after-source-block)))))
          (setq org-edit-response
                (soma-gym-bridge-submit-request org-edit-request))
          (setq org-edit-result
                (soma-bridge-bench--get org-edit-response "result"))
          (setq org-edit-buffer-name
                (soma-bridge-bench--get org-edit-result "edit_buffer_name"))
          (setq org-edit-buffer
                (and (stringp org-edit-buffer-name)
                     (get-buffer org-edit-buffer-name)))
          (setq org-edit-buffer-opened-p (buffer-live-p org-edit-buffer))
          (setq org-edit-buffer-separate-p
                (and (buffer-live-p org-edit-buffer)
                     (not (eq org-edit-buffer report-buffer))))
          (setq org-edit-buffer-initial-text
                (if (buffer-live-p org-edit-buffer)
                    (with-current-buffer org-edit-buffer
                      (buffer-substring-no-properties
                       (point-min) (point-max)))
                  ""))
          (setq org-edit-buffer-observe-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-edit-buffer-observe-1")
                 (cons "operation" "emacs.observe_buffer")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (or org-edit-buffer-name ""))))))
          (setq org-edit-buffer-observe-response
                (soma-gym-bridge-submit-request
                 org-edit-buffer-observe-request))
          (setq org-edit-buffer-observe-result
                (soma-bridge-bench--get org-edit-buffer-observe-response
                                        "result"))
          (setq org-edit-replace-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-edit-buffer-replace-1")
                 (cons "operation" "buffer.replace_range")
                 (cons "capability" "reversible_editor_edit")
                 (cons "preconditions"
                       (soma-bridge-bench--buffer-edit-preconditions
                        org-edit-buffer-observe-result
                        (if (buffer-live-p org-edit-buffer)
                            (with-current-buffer org-edit-buffer
                              (buffer-substring-no-properties
                               (point-min) (point-max)))
                          "")))
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (or org-edit-buffer-name ""))
                        (cons "start"
                              (or (soma-bridge-bench--get
                                   org-edit-buffer-observe-result
                                   "point_min")
                                  1))
                        (cons "end"
                              (or (soma-bridge-bench--get
                                   org-edit-buffer-observe-result
                                   "point_max")
                                  1))
                        (cons "text"
                              "(princ \"source-edit\\n\")\n")))))
          (setq org-edit-replace-response
                (soma-gym-bridge-submit-request org-edit-replace-request))
          (setq org-edit-replace-result
                (soma-bridge-bench--get org-edit-replace-response "result"))
          (setq org-exit-preconditions
                (soma-bridge-bench--alist
                 (cons "edit_buffer_name" (or org-edit-buffer-name ""))
                 (cons "edit_buffer_chars_tick"
                       (soma-bridge-bench--get
                        (soma-bridge-bench--get org-edit-replace-result
                                                "after")
                        "chars_tick"))
                 (cons "edit_text_sha256"
                       (secure-hash 'sha256 "(princ \"source-edit\\n\")\n"))
                 (cons "parent_buffer_name" (buffer-name report-buffer))
                 (cons "parent_text_owner_name"
                       (soma-bridge-bench--get org-after-update-result
                                               "text_owner_name"))
                 (cons "parent_file_name"
                       (soma-bridge-bench--get org-after-update-result
                                               "buffer_file_name"))
                 (cons "parent_chars_tick"
                       (soma-bridge-bench--get org-after-update-result
                                               "chars_tick"))
                 (cons "parent_structure_fingerprint"
                       (soma-bridge-bench--get org-after-update-result
                                               "structure_fingerprint"))
                 (cons "parent_markers"
                       (soma-bridge-bench--get org-edit-result
                                               "parent_markers"))
                 (cons "source_block" org-after-source-block)))
          (setq org-exit-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-exit-src-edit-1")
                 (cons "operation" "org.exit_src_edit")
                 (cons "capability" "reversible_editor_edit")
                 (cons "preconditions" org-exit-preconditions)
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "edit_buffer_name"
                              (or org-edit-buffer-name ""))))))
          (setq org-exit-response
                (soma-gym-bridge-submit-request org-exit-request))
          (setq org-exit-result
                (soma-bridge-bench--get org-exit-response "result"))
          (setq org-after-exit-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-after-exit-observe-1")
                 (cons "operation" "org.observe_tree")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "max_headings" 20)
                        (cons "max_source_blocks" 20)))))
          (setq org-after-exit-response
                (soma-gym-bridge-submit-request org-after-exit-request))
          (setq org-after-exit-result
                (soma-bridge-bench--get org-after-exit-response "result"))
          (setq org-after-exit-heading
                (car (soma-bridge-bench--get org-after-exit-result
                                             "headings")))
          (setq org-after-exit-source-block
                (car (soma-bridge-bench--get org-after-exit-result
                                             "source_blocks")))
          (setq org-stale-edit-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-edit-src-block-stale-1")
                 (cons "operation" "org.edit_src_block")
                 (cons "capability" "editor_state_change")
                 (cons "preconditions"
                       (soma-bridge-bench--org-preconditions
                        report-buffer org-after-exit-result
                        org-after-exit-heading org-after-exit-source-block))
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "source_block"
                              org-after-exit-source-block)))))
          (setq org-stale-edit-response
                (soma-gym-bridge-submit-request org-stale-edit-request))
          (setq org-stale-edit-result
                (soma-bridge-bench--get org-stale-edit-response "result"))
          (setq org-stale-edit-buffer-name
                (soma-bridge-bench--get org-stale-edit-result
                                        "edit_buffer_name"))
          (setq org-stale-edit-buffer
                (and (stringp org-stale-edit-buffer-name)
                     (get-buffer org-stale-edit-buffer-name)))
          (setq org-stale-edit-buffer-observe-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-stale-edit-buffer-observe-1")
                 (cons "operation" "emacs.observe_buffer")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name"
                              (or org-stale-edit-buffer-name ""))))))
          (setq org-stale-edit-buffer-observe-response
                (soma-gym-bridge-submit-request
                 org-stale-edit-buffer-observe-request))
          (setq org-stale-edit-buffer-observe-result
                (soma-bridge-bench--get
                 org-stale-edit-buffer-observe-response "result"))
          (setq org-stale-replace-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-stale-edit-buffer-replace-1")
                 (cons "operation" "buffer.replace_range")
                 (cons "capability" "reversible_editor_edit")
                 (cons "preconditions"
                       (soma-bridge-bench--buffer-edit-preconditions
                        org-stale-edit-buffer-observe-result
                        (if (buffer-live-p org-stale-edit-buffer)
                            (with-current-buffer org-stale-edit-buffer
                              (buffer-substring-no-properties
                               (point-min) (point-max)))
                          "")))
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name"
                              (or org-stale-edit-buffer-name ""))
                        (cons "start"
                              (or (soma-bridge-bench--get
                                   org-stale-edit-buffer-observe-result
                                   "point_min")
                                  1))
                        (cons "end"
                              (or (soma-bridge-bench--get
                                   org-stale-edit-buffer-observe-result
                                   "point_max")
                                  1))
                        (cons "text"
                              "(princ \"stale writeback\\n\")\n")))))
          (setq org-stale-replace-response
                (soma-gym-bridge-submit-request org-stale-replace-request))
          (when (buffer-live-p org-stale-edit-buffer)
            (with-current-buffer report-buffer
              (save-excursion
                (goto-char (point-max))
                (insert "\n# human parent drift\n"))))
          (setq org-stale-exit-preconditions
                (soma-bridge-bench--alist
                 (cons "edit_buffer_name"
                       (or org-stale-edit-buffer-name ""))
                 (cons "edit_buffer_chars_tick"
                       (soma-bridge-bench--get
                        (soma-bridge-bench--get
                         (soma-bridge-bench--get org-stale-replace-response
                                                 "result")
                         "after")
                        "chars_tick"))
                 (cons "edit_text_sha256"
                       (secure-hash 'sha256
                                    "(princ \"stale writeback\\n\")\n"))
                 (cons "parent_buffer_name" (buffer-name report-buffer))
                 (cons "parent_text_owner_name"
                       (soma-bridge-bench--get org-after-exit-result
                                               "text_owner_name"))
                 (cons "parent_file_name"
                       (soma-bridge-bench--get org-after-exit-result
                                               "buffer_file_name"))
                 (cons "parent_chars_tick"
                       (soma-bridge-bench--get org-after-exit-result
                                               "chars_tick"))
                 (cons "parent_structure_fingerprint"
                       (soma-bridge-bench--get org-after-exit-result
                                               "structure_fingerprint"))
                 (cons "parent_markers"
                       (soma-bridge-bench--get org-stale-edit-result
                                               "parent_markers"))
                 (cons "source_block" org-after-exit-source-block)))
          (setq org-stale-exit-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-exit-src-edit-stale-1")
                 (cons "operation" "org.exit_src_edit")
                 (cons "capability" "reversible_editor_edit")
                 (cons "preconditions" org-stale-exit-preconditions)
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "edit_buffer_name"
                              (or org-stale-edit-buffer-name ""))))))
          (setq org-stale-exit-response
                (soma-gym-bridge-submit-request org-stale-exit-request))
          (setq org-stale-exit-result
                (soma-bridge-bench--get org-stale-exit-response "result"))
          (when (buffer-live-p org-stale-edit-buffer)
            (kill-buffer org-stale-edit-buffer))
          (set-window-buffer (selected-window) report-buffer)
          (setq org-before-babel-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-before-babel-observe-1")
                 (cons "operation" "org.observe_tree")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "max_headings" 20)
                        (cons "max_source_blocks" 20)))))
          (setq org-before-babel-response
                (soma-gym-bridge-submit-request org-before-babel-request))
          (setq org-before-babel-result
                (soma-bridge-bench--get org-before-babel-response "result"))
          (setq org-before-babel-heading
                (car (soma-bridge-bench--get org-before-babel-result
                                             "headings")))
          (setq org-before-babel-source-block
                (car (soma-bridge-bench--get org-before-babel-result
                                             "source_blocks")))
          (setq org-babel-preconditions
                (append
                 (soma-bridge-bench--org-preconditions
                  report-buffer org-before-babel-result
                  org-before-babel-heading org-before-babel-source-block)
                 (list
                  (cons "language_allowlist" '("emacs-lisp"))
                  (cons "default_directory"
                        (file-name-as-directory tmp))
                  (cons "session" :json-false))))
          (setq org-babel-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-execute-babel-1")
                 (cons "operation" "org.execute_babel_block")
                 (cons "capability" "external_process_or_kernel_execution")
                 (cons "preconditions" org-babel-preconditions)
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "source_block"
                              org-before-babel-source-block)))))
          (setq org-babel-response
                (soma-gym-bridge-submit-request org-babel-request))
          (setq org-babel-result
                (soma-bridge-bench--get org-babel-response "result"))
          (setq org-after-babel-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-after-babel-observe-1")
                 (cons "operation" "org.observe_tree")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "max_headings" 20)
                        (cons "max_source_blocks" 20)))))
          (setq org-after-babel-response
                (soma-gym-bridge-submit-request org-after-babel-request))
          (setq org-after-babel-result
                (soma-bridge-bench--get org-after-babel-response "result"))
          (setq org-after-babel-source-block
                (car (soma-bridge-bench--get org-after-babel-result
                                             "source_blocks")))
          (setq org-stale-babel-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-execute-babel-stale-1")
                 (cons "operation" "org.execute_babel_block")
                 (cons "capability" "external_process_or_kernel_execution")
                 (cons "preconditions" org-babel-preconditions)
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "source_block"
                              org-after-exit-source-block)))))
          (setq org-stale-babel-response
                (soma-gym-bridge-submit-request org-stale-babel-request))
          (setq org-stale-babel-result
                (soma-bridge-bench--get org-stale-babel-response "result"))
          (setq org-export-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "reports-org-export-1")
                 (cons "operation" "org.export_html")
                 (cons "capability" "filesystem_write")
                 (cons "preconditions"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "chars_tick"
                              (with-current-buffer report-buffer
                                (buffer-chars-modified-tick)))
                        (cons "publish_directory"
                              (file-name-as-directory
                               (expand-file-name "published" tmp)))
                        (cons "export_options"
                              (soma-bridge-bench--alist
                               (cons "backend" "html")
                               (cons "body_only" :json-false)))))
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name report-buffer))
                        (cons "output_path" export-file)))))
          (setq org-export-response
                (soma-gym-bridge-submit-request org-export-request))
          (setq org-export-result
                (soma-bridge-bench--get org-export-response "result"))
          (setq dired-observe-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "vcs-dired-observe-1")
                 (cons "operation" "dired.observe")
                 (cons "capability" "passive_observation")
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name dired-buffer))
                        (cons "max_files" 50)))))
          (setq dired-observe-response
                (soma-gym-bridge-submit-request dired-observe-request))
          (setq dired-observe-result
                (soma-bridge-bench--get dired-observe-response "result"))
          (setq dired-rename-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "vcs-dired-rename-1")
                 (cons "operation" "dired.rename_no_overwrite")
                 (cons "capability" "filesystem_write")
                 (cons "preconditions"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name dired-buffer))
                        (cons "directory" (file-name-as-directory
                                           (file-truename tmp)))
                        (cons "file_signature"
                              (soma-bridge-bench--file-signature source-file))
                        (cons "target_absent" t)
                        (cons "collision_policy" "no_overwrite")))
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name dired-buffer))
                        (cons "source" "source.txt")
                        (cons "target" "renamed.txt")))))
          (setq dired-rename-response
                (soma-gym-bridge-submit-request dired-rename-request))
          (setq dired-rename-result
                (soma-bridge-bench--get dired-rename-response "result"))
          (setq collision-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "vcs-dired-collision-1")
                 (cons "operation" "dired.rename_no_overwrite")
                 (cons "capability" "filesystem_write")
                 (cons "preconditions"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name dired-buffer))
                        (cons "directory" (file-name-as-directory
                                           (file-truename tmp)))
                        (cons "file_signature"
                              (soma-bridge-bench--file-signature
                               collision-source-file))
                        (cons "target_absent" t)
                        (cons "collision_policy" "no_overwrite")))
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name dired-buffer))
                        (cons "source" "collision-source.txt")
                        (cons "target" "collision.txt")))))
          (setq collision-response
                (soma-gym-bridge-submit-request collision-request))
          (setq collision-result
                (soma-bridge-bench--get collision-response "result"))
          (with-current-buffer dired-buffer
            (revert-buffer)
            (dired-goto-file delete-a-file)
            (dired-mark 1)
            (dired-goto-file delete-b-file)
            (dired-mark 1)
            (dired-goto-file flagged-decoy-file)
            (dired-flag-file-deletion 1))
          (setq dired-delete-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "vcs-dired-delete-marked-1")
                 (cons "operation" "dired.delete_marked")
                 (cons "capability" "filesystem_write")
                 (cons "preconditions"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name dired-buffer))
                        (cons "directory" (file-name-as-directory
                                           (file-truename tmp)))
                        (cons "marked_files"
                              '("delete-a.txt" "delete-b.txt"))
                        (cons "file_signatures"
                              (list
                               (soma-bridge-bench--alist
                                (cons "name" "delete-a.txt")
                                (cons "signature"
                                      (soma-bridge-bench--file-signature
                                       delete-a-file)))
                               (soma-bridge-bench--alist
                                (cons "name" "delete-b.txt")
                                (cons "signature"
                                      (soma-bridge-bench--file-signature
                                       delete-b-file)))))
                        (cons "delete_policy"
                              (soma-bridge-bench--alist
                               (cons "selection" "marked")
                               (cons "delete_by_moving_to_trash" :json-false)
                               (cons "recursive_deletes" :json-false)))))
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name dired-buffer))))))
          (setq dired-delete-response
                (soma-gym-bridge-submit-request dired-delete-request))
          (setq dired-delete-result
                (soma-bridge-bench--get dired-delete-response "result"))
          (with-current-buffer stale-dired-buffer
            (revert-buffer)
            (dired-goto-file stale-file)
            (dired-mark 1))
          (setq stale-before-signature
                (soma-bridge-bench--file-signature stale-file))
          (delete-file stale-file)
          (soma-bridge-bench--write-file
           stale-file "external-replacement\n")
          (setq stale-delete-request
                (soma-bridge-bench--alist
                 (cons "api_version" "soma-emacs-agent-v0")
                 (cons "request_id" "vcs-dired-delete-stale-1")
                 (cons "operation" "dired.delete_marked")
                 (cons "capability" "filesystem_write")
                 (cons "preconditions"
                       (soma-bridge-bench--alist
                        (cons "buffer_name" (buffer-name stale-dired-buffer))
                        (cons "directory" (file-name-as-directory
                                           (file-truename stale-dir)))
                        (cons "marked_files" '("replace-me.txt"))
                        (cons "file_signatures"
                              (list
                               (soma-bridge-bench--alist
                                (cons "name" "replace-me.txt")
                                (cons "signature"
                                      stale-before-signature))))
                        (cons "delete_policy"
                              (soma-bridge-bench--alist
                               (cons "selection" "marked")
                               (cons "delete_by_moving_to_trash" :json-false)
                               (cons "recursive_deletes" :json-false)))))
                 (cons "params"
                       (soma-bridge-bench--alist
                        (cons "buffer_name"
                              (buffer-name stale-dired-buffer))))))
          (setq stale-delete-response
                (soma-gym-bridge-submit-request stale-delete-request))
          (setq stale-delete-result
                (soma-bridge-bench--get stale-delete-response "result"))
          (setq checks
                (list
                 (soma-bridge-bench--check
                  "reports and files operations are stable and implemented"
                  (and (string= (soma-bridge-bench--get
                                 org-observe-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            org-observe-entry "implemented")
                           t)
                       (string= (soma-bridge-bench--get
                                 org-update-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            org-update-entry "implemented")
                           t)
                       (string= (soma-bridge-bench--get
                                 org-edit-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            org-edit-entry "implemented")
                           t)
                       (string= (soma-bridge-bench--get
                                 org-exit-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            org-exit-entry "implemented")
                           t)
                       (string= (soma-bridge-bench--get
                                 org-babel-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            org-babel-entry "implemented")
                           t)
                       (string= (soma-bridge-bench--get
                                 org-export-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            org-export-entry "implemented")
                           t)
                       (string= (soma-bridge-bench--get
                                 dired-observe-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            dired-observe-entry "implemented")
                           t)
                       (string= (soma-bridge-bench--get
                                 dired-rename-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            dired-rename-entry "implemented")
                           t)
                       (string= (soma-bridge-bench--get
                                 dired-delete-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get
                            dired-delete-entry "implemented")
                           t))
                  (format "%S"
                          (list org-observe-entry org-update-entry
                                org-edit-entry org-exit-entry
                                org-babel-entry org-export-entry
                                dired-observe-entry dired-rename-entry
                                dired-delete-entry)))
                 (soma-bridge-bench--check
                  "org.observe_tree succeeds through the v0 envelope"
                  (string= (soma-bridge-bench--get org-observe-response
                                                   "state")
                           "succeeded")
                  (format "%S" org-observe-response))
                 (soma-bridge-bench--check
                  "org.observe_tree reports heading and source block ownership"
                  (and (listp org-observe-result)
                       (= (or (soma-bridge-bench--get
                               org-observe-result "heading_count")
                              0)
                          1)
                       (= (or (soma-bridge-bench--get
                               org-observe-result "source_block_count")
                              0)
                          1)
                       (string= (soma-bridge-bench--get
                                 (car (soma-bridge-bench--get
                                       org-observe-result "headings"))
                                 "id")
                                "report-summary"))
                  (format "%S" org-observe-result))
	                 (soma-bridge-bench--check
	                  "org.observe_tree publishes typed structure evidence and tree observation"
	                  (let* ((evidence (soma-bridge-bench--get
	                                    org-observe-response "evidence"))
	                         (observations (soma-bridge-bench--get
	                                        org-observe-response
	                                        "observations"))
	                         (effects (soma-bridge-bench--get
	                                   org-observe-response "effects"))
	                         (buffer-identity (soma-bridge-bench--get
	                                           evidence "buffer_identity"))
	                         (structure (soma-bridge-bench--get
	                                     evidence "structure"))
	                         (heading-index (soma-bridge-bench--get
	                                         evidence "heading_index"))
	                         (source-index (soma-bridge-bench--get
	                                        evidence "source_block_index"))
	                         (tree (soma-bridge-bench--get observations
	                                                        "org_tree")))
	                    (and (string= (soma-bridge-bench--get
	                                    buffer-identity "buffer_name")
	                                   (buffer-name report-buffer))
	                         (string= (soma-bridge-bench--get
	                                   structure "structure_fingerprint")
	                                  (soma-bridge-bench--get
	                                   org-observe-result
	                                   "structure_fingerprint"))
	                         (= (or (soma-bridge-bench--get
	                                 heading-index "heading_count")
	                                0)
	                            1)
	                         (= (or (soma-bridge-bench--get
	                                 source-index "source_block_count")
	                                0)
	                            1)
	                         (string= (soma-bridge-bench--get
	                                   (car (soma-bridge-bench--get
	                                         tree "headings"))
	                                   "id")
	                                  "report-summary")
	                         (and (hash-table-p effects)
	                              (= (hash-table-count effects) 0))))
	                  (format "%S" org-observe-response))
                 (soma-bridge-bench--check
                  "org.update_heading succeeds through the v0 envelope"
                  (and (string= (soma-bridge-bench--get org-update-response
                                                        "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get org-update-result
                                                        "route")
                                "org-update-heading"))
                  (format "%S" org-update-response))
                 (soma-bridge-bench--check
                  "org.update_heading preserves ID and source block"
                  (and (string= (soma-bridge-bench--get org-after-update-response
                                                        "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get org-after-heading
                                                        "todo")
                                "DONE")
                       (string= (soma-bridge-bench--get org-after-heading
                                                        "title")
                                "Updated Summary")
                       (string= (soma-bridge-bench--get org-after-heading
                                                        "id")
                                "report-summary")
                       (string= (soma-bridge-bench--get org-after-source-block
                                                        "value_sha256")
                                (soma-bridge-bench--get org-source-block
                                                        "value_sha256")))
                  (format "%S" org-after-update-result))
                 (soma-bridge-bench--check
                  "org.update_heading rejects stale structure preconditions"
                  (and (string= (soma-bridge-bench--get
                                 org-stale-update-response "state")
                                "failed")
                       (string= (soma-bridge-bench--get
                                 org-stale-update-response "error_code")
                                "conflict")
                       (not (string-match-p
                             "Stale Rewrite"
                             (with-current-buffer report-buffer
                               (buffer-substring-no-properties
                                (point-min) (point-max))))))
                  (format "%S" org-stale-update-result))
                 (soma-bridge-bench--check
                  "org.edit_src_block opens a separate checked source-edit buffer"
                  (and (string= (soma-bridge-bench--get org-edit-response
                                                        "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get org-edit-result
                                                        "route")
                                "org-edit-src-code")
                       (eq (soma-bridge-bench--get
                            org-edit-result "source_edit_buffer_separate")
                           t)
                       org-edit-buffer-opened-p
                       org-edit-buffer-separate-p
                       (string-match-p
                        "(princ \"ok\\\\n\")"
                        org-edit-buffer-initial-text))
                  (format "%S" org-edit-result))
                 (soma-bridge-bench--check
                  "parent Org buffer is unchanged before source-edit exit"
                  (and (string= (soma-bridge-bench--get
                                 org-edit-buffer-observe-response "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get
                                 org-edit-replace-response "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get
                                 org-after-update-result
                                 "structure_fingerprint")
                                (soma-bridge-bench--get
                                 (soma-bridge-bench--get
                                  org-edit-result "parent_after_open")
                                 "structure_fingerprint")))
                  (format "%S" (list org-edit-buffer-observe-response
                                     org-edit-replace-response
                                     org-edit-result)))
                 (soma-bridge-bench--check
                  "org.exit_src_edit writes back through Org source-edit workflow"
                  (and (string= (soma-bridge-bench--get org-exit-response
                                                        "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get org-exit-result
                                                        "route")
                                "org-edit-src-exit")
                       (string= (soma-bridge-bench--get
                                 org-after-exit-response "state")
                                "succeeded")
                       (eq (soma-bridge-bench--get
                            org-exit-result "edit_buffer_live_after")
                           :json-false)
                       (string= (soma-bridge-bench--get
                                 org-after-exit-heading "title")
                                "Updated Summary")
                       (string= (soma-bridge-bench--get
                                 org-after-exit-heading "id")
                                "report-summary")
                       (string= (soma-bridge-bench--get
                                 org-after-exit-source-block "value_sha256")
                                (secure-hash
                                 'sha256
                                 "  (princ \"source-edit\\n\")\n")))
                  (format "%S" (list org-exit-response
                                     org-after-exit-response)))
                 (soma-bridge-bench--check
                  "org.exit_src_edit rejects stale parent before writeback"
                  (and (string= (soma-bridge-bench--get
                                 org-stale-exit-response "state")
                                "failed")
                       (string= (soma-bridge-bench--get
                                 org-stale-exit-response "error_code")
                                "conflict")
                       (not (string-match-p
                             "stale writeback"
                             (with-current-buffer report-buffer
                               (buffer-substring-no-properties
                                (point-min) (point-max))))))
                  (format "%S" org-stale-exit-result))
                 (soma-bridge-bench--check
                  "org.execute_babel_block runs through Org Babel and inserts results"
                  (and (string= (soma-bridge-bench--get org-babel-response
                                                        "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get org-babel-result
                                                        "route")
                                "org-babel-execute-src-block")
                       (string= (soma-bridge-bench--get
                                 org-after-babel-response "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get
                                 org-after-babel-source-block
                                 "value_sha256")
                                (soma-bridge-bench--get
                                 org-before-babel-source-block
                                 "value_sha256"))
                       (string-match-p
                        "#\\+RESULTS:"
                        (with-current-buffer report-buffer
                          (buffer-substring-no-properties
                           (point-min) (point-max))))
                       (string-match-p
                        "source-edit"
                        (with-current-buffer report-buffer
                          (buffer-substring-no-properties
                           (point-min) (point-max)))))
                  (format "%S" (list org-babel-response
                                     org-after-babel-response)))
                 (soma-bridge-bench--check
                  "org.execute_babel_block rejects stale source-block state"
                  (and (string= (soma-bridge-bench--get
                                 org-stale-babel-response "state")
                                "failed")
                       (string= (soma-bridge-bench--get
                                 org-stale-babel-response "error_code")
                                "conflict"))
                  (format "%S" org-stale-babel-result))
                 (soma-bridge-bench--check
                  "org.export_html writes the requested HTML artifact"
                  (and (string= (soma-bridge-bench--get org-export-response
                                                        "state")
                                "succeeded")
                       (file-exists-p export-file)
                       (string-match-p
                        "<html"
                        (or (soma-bridge-bench--read-file export-file) "")))
                  (format "%S" org-export-result))
                 (soma-bridge-bench--check
                  "reports-v0 mutators publish typed semantic effects"
                  (let* ((update-effect
                          (soma-bridge-bench--get
                           (soma-bridge-bench--get
                            org-update-response "effects")
                           "org_heading_update"))
                         (source-edit-effect
                          (soma-bridge-bench--get
                           (soma-bridge-bench--get
                            org-edit-response "effects")
                           "source_edit_buffer"))
                         (writeback-effect
                          (soma-bridge-bench--get
                           (soma-bridge-bench--get
                            org-exit-response "effects")
                           "parent_buffer_writeback"))
                         (babel-effect
                          (soma-bridge-bench--get
                           (soma-bridge-bench--get
                            org-babel-response "effects")
                           "org_babel_execution"))
                         (export-effect
                          (soma-bridge-bench--get
                           (soma-bridge-bench--get
                            org-export-response "effects")
                           "html_export")))
                    (and
                     (soma-bridge-bench--object-p update-effect)
                     (string= (soma-bridge-bench--get
                               update-effect "route")
                              "org-update-heading")
                     (= (or (soma-bridge-bench--get
                             update-effect "target_position")
                            0)
                        (soma-bridge-bench--get
                         org-update-result "target_position"))
                     (string= (soma-bridge-bench--get
                               (soma-bridge-bench--get
                                update-effect "after_heading")
                               "title")
                              "Updated Summary")
                     (soma-bridge-bench--object-p source-edit-effect)
                     (string= (soma-bridge-bench--get
                               source-edit-effect "route")
                              "org-edit-src-code")
                     (eq (soma-bridge-bench--get
                          source-edit-effect
                          "source_edit_buffer_separate")
                         t)
                     (stringp (soma-bridge-bench--get
                               source-edit-effect "edit_buffer_name"))
                     (soma-bridge-bench--object-p writeback-effect)
                     (string= (soma-bridge-bench--get
                               writeback-effect "route")
                              "org-edit-src-exit")
                     (eq (soma-bridge-bench--get
                          writeback-effect "edit_buffer_live_after")
                         :json-false)
                     (soma-bridge-bench--object-p babel-effect)
                     (string= (soma-bridge-bench--get
                               babel-effect "route")
                              "org-babel-execute-src-block")
                     (string= (soma-bridge-bench--get
                               (soma-bridge-bench--get
                                (soma-bridge-bench--get
                                 babel-effect "runtime_identity")
                                "rootfs_runtime")
                               "run_id")
                              (getenv "SOMA_INSULA_RUN_ID"))
                     (soma-bridge-bench--object-p
                      (soma-bridge-bench--get
                       babel-effect "parent_result_block"))
                     (soma-bridge-bench--object-p export-effect)
                     (string= (soma-bridge-bench--get
                               export-effect "route")
                              "org-export")
                     (string= (soma-bridge-bench--get
                               export-effect "generated_file")
                              export-file)
                     (eq (soma-bridge-bench--get
                          (soma-bridge-bench--get
                           export-effect "output_signature")
                          "exists")
                         t)))
                  (format "%S"
                          (list org-update-response org-edit-response
                                org-exit-response org-babel-response
                                org-export-response)))
                 (soma-bridge-bench--check
                  "dired.observe succeeds and reports filesystem signatures"
                  (and (string= (soma-bridge-bench--get dired-observe-response
                                                        "state")
                                "succeeded")
                       (>= (or (soma-bridge-bench--get
                                dired-observe-result "file_count")
                               0)
                           4))
                  (format "%S" dired-observe-result))
	                 (soma-bridge-bench--check
	                  "dired.observe publishes typed directory, selection, and filesystem sections"
	                  (let* ((evidence (soma-bridge-bench--get
	                                    dired-observe-response "evidence"))
	                         (observations (soma-bridge-bench--get
	                                        dired-observe-response
	                                        "observations"))
	                         (effects (soma-bridge-bench--get
	                                   dired-observe-response "effects"))
	                         (directory (soma-bridge-bench--get
	                                     evidence "directory"))
	                         (selection-state (soma-bridge-bench--get
	                                           evidence "selection_state"))
	                         (filesystem (soma-bridge-bench--get
	                                      evidence "file_signatures"))
	                         (listing (soma-bridge-bench--get
	                                   observations "dired_listing")))
	                    (and (string= (soma-bridge-bench--get
	                                    directory "path")
	                                   (file-name-as-directory
	                                    (file-truename tmp)))
	                         (eq (soma-bridge-bench--get
	                              selection-state "marked_files")
	                             :json-false)
	                         (>= (or (soma-bridge-bench--get
	                                  filesystem "file_count")
	                                 0)
	                             4)
	                         (consp (soma-bridge-bench--get
	                                 listing "files"))
	                         (and (hash-table-p effects)
	                              (= (hash-table-count effects) 0))))
	                  (format "%S" dired-observe-response))
                 (soma-bridge-bench--check
                  "dired.rename_no_overwrite moves the requested file only"
                  (and (string= (soma-bridge-bench--get dired-rename-response
                                                        "state")
                                "succeeded")
                       (not (file-exists-p source-file))
                       (file-exists-p renamed-file)
                       (string= (soma-bridge-bench--read-file renamed-file)
                                "source\n"))
                  (format "%S" dired-rename-result))
                 (soma-bridge-bench--check
                  "Dired mutators publish typed filesystem effects"
                  (let* ((rename-effect
                          (soma-bridge-bench--get
                           (soma-bridge-bench--get
                            dired-rename-response "effects")
                           "filesystem_rename"))
                         (delete-effect
                          (soma-bridge-bench--get
                           (soma-bridge-bench--get
                            dired-delete-response "effects")
                           "filesystem_delete"))
                         (post-delete-state
                          (soma-bridge-bench--get
                           delete-effect "post_filesystem_state")))
                    (and
                     (soma-bridge-bench--object-p rename-effect)
                     (string= (soma-bridge-bench--get
                               rename-effect "route")
                              "dired-rename-file")
                     (string= (soma-bridge-bench--get
                               rename-effect "source")
                              "source.txt")
                     (string= (soma-bridge-bench--get
                               rename-effect "target")
                              "renamed.txt")
                     (eq (soma-bridge-bench--get
                          (soma-bridge-bench--get
                           rename-effect "source_after")
                          "exists")
                         :json-false)
                     (eq (soma-bridge-bench--get
                          (soma-bridge-bench--get
                           rename-effect "target_after")
                          "exists")
                         t)
                     (soma-bridge-bench--object-p delete-effect)
                     (string= (soma-bridge-bench--get
                               delete-effect "route")
                              "dired-do-delete")
                     (equal (soma-bridge-bench--get
                             delete-effect "marked_files")
                            '("delete-a.txt" "delete-b.txt"))
                     (= (length post-delete-state) 2)
                     (cl-every
                      (lambda (entry)
                        (eq (soma-bridge-bench--get
                             (soma-bridge-bench--get
                              entry "signature")
                             "exists")
                            :json-false))
                      post-delete-state)))
                  (format "%S"
                          (list dired-rename-response
                                dired-delete-response)))
                 (soma-bridge-bench--check
                  "dired.rename_no_overwrite rejects collisions without overwrite"
                  (and (string= (soma-bridge-bench--get collision-response
                                                        "state")
                                "failed")
                       (file-exists-p collision-source-file)
                       (string= (soma-bridge-bench--read-file collision-file)
                                "collision-original\n"))
                  (format "%S" collision-result))
                 (soma-bridge-bench--check
                  "dired.delete_marked deletes only checked ordinary marks"
                  (and (string= (soma-bridge-bench--get dired-delete-response
                                                        "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get dired-delete-result
                                                        "route")
                                "dired-do-delete")
                       (not (file-exists-p delete-a-file))
                       (not (file-exists-p delete-b-file))
                       (file-exists-p flagged-decoy-file)
                       (string= (soma-bridge-bench--read-file
                                 flagged-decoy-file)
                                "flagged-decoy\n"))
                  (format "%S" dired-delete-result))
                 (soma-bridge-bench--check
                  "dired.delete_marked rejects stale same-path replacements"
                  (and (string= (soma-bridge-bench--get
                                 stale-delete-response "state")
                                "failed")
                       (string= (soma-bridge-bench--get
                                 stale-delete-response "error_code")
                                "conflict")
                       (file-exists-p stale-file)
                       (string= (soma-bridge-bench--read-file stale-file)
                                "external-replacement\n"))
                  (format "%S" stale-delete-result))))
          (soma-bridge-bench--alist
           (cons "org_observe_request" org-observe-request)
           (cons "org_observe_response" org-observe-response)
           (cons "org_update_request" org-update-request)
           (cons "org_update_response" org-update-response)
           (cons "org_after_update_request" org-after-update-request)
           (cons "org_after_update_response" org-after-update-response)
           (cons "org_stale_update_request" org-stale-update-request)
           (cons "org_stale_update_response" org-stale-update-response)
           (cons "org_edit_request" org-edit-request)
           (cons "org_edit_response" org-edit-response)
           (cons "org_edit_buffer_observe_request"
                 org-edit-buffer-observe-request)
           (cons "org_edit_buffer_observe_response"
                 org-edit-buffer-observe-response)
           (cons "org_edit_replace_request" org-edit-replace-request)
           (cons "org_edit_replace_response" org-edit-replace-response)
           (cons "org_exit_request" org-exit-request)
           (cons "org_exit_response" org-exit-response)
           (cons "org_after_exit_request" org-after-exit-request)
           (cons "org_after_exit_response" org-after-exit-response)
           (cons "org_stale_edit_request" org-stale-edit-request)
           (cons "org_stale_edit_response" org-stale-edit-response)
           (cons "org_stale_edit_buffer_observe_request"
                 org-stale-edit-buffer-observe-request)
           (cons "org_stale_edit_buffer_observe_response"
                 org-stale-edit-buffer-observe-response)
           (cons "org_stale_replace_request" org-stale-replace-request)
           (cons "org_stale_replace_response" org-stale-replace-response)
           (cons "org_stale_exit_request" org-stale-exit-request)
           (cons "org_stale_exit_response" org-stale-exit-response)
           (cons "org_before_babel_request" org-before-babel-request)
           (cons "org_before_babel_response" org-before-babel-response)
           (cons "org_babel_request" org-babel-request)
           (cons "org_babel_response" org-babel-response)
           (cons "org_after_babel_request" org-after-babel-request)
           (cons "org_after_babel_response" org-after-babel-response)
           (cons "org_stale_babel_request" org-stale-babel-request)
           (cons "org_stale_babel_response" org-stale-babel-response)
           (cons "org_export_request" org-export-request)
           (cons "org_export_response" org-export-response)
           (cons "dired_observe_request" dired-observe-request)
           (cons "dired_observe_response" dired-observe-response)
           (cons "dired_rename_request" dired-rename-request)
           (cons "dired_rename_response" dired-rename-response)
           (cons "collision_request" collision-request)
           (cons "collision_response" collision-response)
           (cons "dired_delete_request" dired-delete-request)
           (cons "dired_delete_response" dired-delete-response)
           (cons "stale_delete_request" stale-delete-request)
           (cons "stale_delete_response" stale-delete-response)
           (cons "export_file" export-file)
           (cons "checks" checks)))
      (when (buffer-live-p report-buffer)
        (kill-buffer report-buffer))
      (when (buffer-live-p dired-buffer)
        (kill-buffer dired-buffer))
      (when (buffer-live-p stale-dired-buffer)
        (kill-buffer stale-dired-buffer))
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-bridge-bench--scenario-command-dispatch ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-command" "alpha"))
         (request (soma-bridge-bench--command-request
                   "command-1" buffer 'soma-bridge-bench-append-token))
         response status result checks)
    (let ((soma-gym-bridge-command-allowlist
           (cons 'soma-bridge-bench-append-token
                 soma-gym-bridge-command-allowlist)))
      (setq response (soma-gym-bridge-submit-request request))
      (setq status (soma-gym-bridge-request-status "command-1")))
    (setq result (soma-bridge-bench--get response "result"))
    (setq checks
          (list
           (soma-bridge-bench--check
            "command request succeeded"
            (string= (soma-bridge-bench--get response "state") "succeeded")
            (format "%S" response))
           (soma-bridge-bench--check
            "command dispatch used checked bridge route"
            (and (string= (soma-bridge-bench--get result "operation")
                          "command.dispatch")
                 (string= (soma-bridge-bench--get result "route")
                          "command-execute")
                 (string= (soma-bridge-bench--get result "this_command")
                          "soma-bridge-bench-append-token"))
            (format "%S" result))
           (soma-bridge-bench--check
            "command mutation happened in target buffer"
            (with-current-buffer buffer
              (string= (buffer-substring-no-properties (point-min) (point-max))
                       "alpha done"))
            (with-current-buffer buffer
              (buffer-substring-no-properties (point-min) (point-max))))
           (soma-bridge-bench--check
            "status records completed command request"
            (and (string= (soma-bridge-bench--get status "request_id")
                          "command-1")
                 (string= (soma-bridge-bench--get status "state")
                          "succeeded"))
            (format "%S" status))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "status" status)
     (cons "buffer_text"
           (with-current-buffer buffer
             (buffer-substring-no-properties (point-min) (point-max))))
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-selected-window-command-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((target (soma-bridge-bench--make-buffer
                  "soma-bridge-selected-window-target" "alpha"))
         (other (soma-bridge-bench--make-buffer
                 "soma-bridge-selected-window-other" "other"))
         (preconditions nil)
         request response result target-text selected-check checks)
    (set-window-buffer (selected-window) target)
    (setq preconditions (soma-bridge-bench--preconditions target))
    (set-window-buffer (selected-window) other)
    (setq request
          (soma-bridge-bench--command-request
           "selected-window-command-1"
           target
           'soma-bridge-bench-append-token))
    (setcdr (assoc "preconditions" (soma-bridge-bench--get request "params"))
            preconditions)
    (let ((soma-gym-bridge-command-allowlist
           (cons 'soma-bridge-bench-append-token
                 soma-gym-bridge-command-allowlist)))
      (setq response (soma-gym-bridge-submit-request request)))
    (setq result (soma-bridge-bench--get response "result"))
    (setq selected-check
          (soma-bridge-bench--precondition-check-named
           response "selected window displays target buffer"))
    (setq target-text (soma-bridge-bench--buffer-full-text target))
    (setq checks
          (list
           (soma-bridge-bench--check
            "selected window drift is rejected"
            (and (string= (soma-bridge-bench--get response "state")
                          "failed")
                 (string= (soma-bridge-bench--get response "error_code")
                          "operation_failed")
                 (string= (soma-bridge-bench--get result "state")
                          "failed"))
            (format "%S" response))
           (soma-bridge-bench--check
            "selected-window precondition is the rejecting check"
            (and (listp selected-check)
                 (eq (soma-bridge-bench--get selected-check "passed")
                     :json-false))
            (format "%S" selected-check))
           (soma-bridge-bench--check
            "command did not mutate target buffer after selected-window drift"
            (string= target-text "alpha")
            target-text)
           (soma-bridge-bench--check
            "selected window now shows the other buffer"
            (eq (window-buffer (selected-window)) other)
            (buffer-name (window-buffer (selected-window))))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "target_text" target-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-buffer-read-purity ()
  (soma-gym-bridge-reset-transport)
  (let* ((left (soma-bridge-bench--make-buffer
                "soma-bridge-read-left" "left\n"))
         (buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-read-target" "alpha\nbeta\n"))
         before request response result after checks)
    (set-window-buffer (selected-window) left)
    (with-current-buffer buffer
      (goto-char 3)
      (set-mark 8)
      (setq mark-active t)
      (set-buffer-modified-p nil))
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (setq request (soma-bridge-bench--buffer-read-request
                   "read-1" buffer 1 7))
    (setq response (soma-gym-bridge-submit-request request))
    (setq result (soma-bridge-bench--get response "result"))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq checks
          (list
           (soma-bridge-bench--check
            "buffer read request succeeded"
            (string= (soma-bridge-bench--get response "state") "succeeded")
            (format "%S" response))
           (soma-bridge-bench--check
            "buffer read returns exact half-open range"
            (and (listp result)
                 (string= (soma-bridge-bench--get result "text") "alpha\n"))
            (format "%S" result))
	           (soma-bridge-bench--check
	            "buffer.read publishes typed range and text snapshot sections"
	            (let* ((evidence (soma-bridge-bench--get response "evidence"))
	                   (observations (soma-bridge-bench--get
	                                  response "observations"))
	                   (effects (soma-bridge-bench--get response "effects"))
	                   (buffer-identity (soma-bridge-bench--get
	                                     evidence "buffer_identity"))
	                   (text-range (soma-bridge-bench--get
	                                evidence "text_range"))
	                   (text-snapshot (soma-bridge-bench--get
	                                   observations "text_snapshot")))
	              (and (string= (soma-bridge-bench--get
	                              buffer-identity "buffer_name")
	                             (buffer-name buffer))
	                   (string= (soma-bridge-bench--get
	                             buffer-identity "text_owner_name")
	                            (buffer-name buffer))
	                   (= (or (soma-bridge-bench--get text-range "start")
	                          0)
	                      1)
	                   (= (or (soma-bridge-bench--get text-range "end")
	                          0)
	                      7)
	                   (string= (soma-bridge-bench--get
	                             text-snapshot "text")
	                            "alpha\n")
	                   (= (or (soma-bridge-bench--get
	                           text-snapshot "chars_tick")
	                          0)
	                      (soma-bridge-bench--get result "chars_tick"))
	                   (and (hash-table-p effects)
	                        (= (hash-table-count effects) 0))))
	            (format "%S" response))
           (soma-bridge-bench--check
            "read preserves point"
            (= (soma-bridge-bench--get before "point")
               (soma-bridge-bench--get after "point"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "point")
                    (soma-bridge-bench--get after "point")))
           (soma-bridge-bench--check
            "read preserves mark-active"
            (eq (soma-bridge-bench--get before "mark_active")
                (soma-bridge-bench--get after "mark_active"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "mark_active")
                    (soma-bridge-bench--get after "mark_active")))
           (soma-bridge-bench--check
            "read preserves chars tick"
            (= (soma-bridge-bench--get before "chars_tick")
               (soma-bridge-bench--get after "chars_tick"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))
           (soma-bridge-bench--check
            "read preserves selected window buffer"
            (string= (soma-bridge-bench--get before "selected_window_buffer")
                     (soma-bridge-bench--get after "selected_window_buffer"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "selected_window_buffer")
                    (soma-bridge-bench--get after "selected_window_buffer")))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "before" before)
     (cons "after" after)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-checked-edit-save ()
  (soma-gym-bridge-reset-transport)
  (let* ((file (concat
                (make-temp-name
                 (expand-file-name "soma-bridge-checked-edit-save-"
                                   temporary-file-directory))
                ".txt"))
         (buffer (find-file-noselect file))
         before edit-preconditions edit-request edit-response edit-result
         save-request save-response save-result after buffer-text file-text
         checks)
    (with-current-buffer buffer
      (setq-local make-backup-files nil)
      (erase-buffer)
      (insert "alpha\nbeta\n")
      (save-buffer)
      (goto-char 1))
    (set-window-buffer (selected-window) buffer)
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (setq edit-preconditions
          (soma-bridge-bench--alist
           (cons "buffer_name" (buffer-name buffer))
           (cons "buffer_file_name" file)
           (cons "text_owner_name" (buffer-name buffer))
           (cons "chars_tick" (soma-bridge-bench--get before "chars_tick"))
           (cons "expected_text" "beta")))
    (setq edit-request
          (soma-bridge-bench--buffer-replace-request
           "edit-1" buffer 7 11 "gamma" edit-preconditions))
    (setq edit-response (soma-gym-bridge-submit-request edit-request))
    (setq edit-result (soma-bridge-bench--get edit-response "result"))
    (setq save-request
          (soma-bridge-bench--buffer-save-request
           "save-1" buffer
           (soma-bridge-bench--alist
            (cons "buffer_name" (buffer-name buffer))
            (cons "buffer_file_name" file)
            (cons "chars_tick"
                  (if (listp edit-result)
                      (soma-bridge-bench--get
                       (soma-bridge-bench--get edit-result "after")
                       "chars_tick")
                    (with-current-buffer buffer
                      (buffer-chars-modified-tick)))))))
    (setq save-response (soma-gym-bridge-submit-request save-request))
    (setq save-result (soma-bridge-bench--get save-response "result"))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq buffer-text
          (with-current-buffer buffer
            (buffer-substring-no-properties (point-min) (point-max))))
    (setq file-text
          (with-temp-buffer
            (insert-file-contents file)
            (buffer-substring-no-properties (point-min) (point-max))))
    (setq checks
          (list
           (soma-bridge-bench--check
            "checked edit request succeeded"
            (string= (soma-bridge-bench--get edit-response "state")
                     "succeeded")
            (format "%S" edit-response))
           (soma-bridge-bench--check
            "checked edit changed the requested range"
            (string= buffer-text "alpha\ngamma\n")
            buffer-text)
           (soma-bridge-bench--check
            "checked edit advanced character revision"
            (< (soma-bridge-bench--get before "chars_tick")
               (soma-bridge-bench--get after "chars_tick"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))
           (soma-bridge-bench--check
            "checked edit publishes typed evidence observations and effects"
            (let* ((evidence (soma-bridge-bench--get edit-response "evidence"))
                   (observations
                    (soma-bridge-bench--get edit-response "observations"))
                   (effects (soma-bridge-bench--get edit-response "effects"))
                   (before-observation
                    (soma-bridge-bench--get observations "before"))
                   (after-observation
                    (soma-bridge-bench--get observations "after"))
                   (changed-range
                    (soma-bridge-bench--get effects "changed_range")))
              (and (soma-bridge-bench--object-p evidence)
                   (soma-bridge-bench--object-p observations)
                   (soma-bridge-bench--object-p effects)
                   (soma-bridge-bench--get evidence "precondition_checks")
                   (soma-bridge-bench--get
                    evidence "before_after_observations")
                   (soma-bridge-bench--object-p before-observation)
                   (soma-bridge-bench--object-p after-observation)
                   (= (soma-bridge-bench--get
                       before-observation "chars_tick")
                      (soma-bridge-bench--get before "chars_tick"))
                   (> (soma-bridge-bench--get
                       after-observation "chars_tick")
                      (soma-bridge-bench--get before "chars_tick"))
                   (soma-bridge-bench--object-p changed-range)
                   (= (soma-bridge-bench--get changed-range "start") 7)
                   (= (soma-bridge-bench--get changed-range "end") 11)
                   (= (soma-bridge-bench--get changed-range
                                               "replacement_chars")
                      5)))
            (format "%S" edit-response))
           (soma-bridge-bench--check
            "save request succeeded"
            (string= (soma-bridge-bench--get save-response "state")
                     "succeeded")
            (format "%S" save-response))
           (soma-bridge-bench--check
            "save wrote the intended file"
            (string= file-text "alpha\ngamma\n")
            file-text)
           (soma-bridge-bench--check
            "save cleared modified state"
            (and (listp save-result)
                 (eq (soma-bridge-bench--get
                      (soma-bridge-bench--get save-result "after")
                      "modified")
                     :json-false))
            (format "%S" save-result))))
    (soma-bridge-bench--alist
     (cons "file" file)
     (cons "edit_request" edit-request)
     (cons "edit_response" edit-response)
     (cons "save_request" save-request)
     (cons "save_response" save-response)
     (cons "before" before)
     (cons "after" after)
     (cons "buffer_text" buffer-text)
     (cons "file_text" file-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-stale-edit-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-stale-edit" "alpha beta\n"))
         before after-human stale-preconditions request response after
         buffer-text checks)
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (with-current-buffer buffer
      (goto-char 7)
      (insert "human-"))
    (setq after-human (soma-gym-bridge-observe-buffer buffer))
    (setq stale-preconditions
          (soma-bridge-bench--alist
           (cons "buffer_name" (buffer-name buffer))
           (cons "chars_tick" (soma-bridge-bench--get before "chars_tick"))
           (cons "expected_text" "beta")))
    (setq request
          (soma-bridge-bench--buffer-replace-request
           "stale-edit-1" buffer 7 11 "gamma" stale-preconditions))
    (setq response (soma-gym-bridge-submit-request request))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq buffer-text
          (with-current-buffer buffer
            (buffer-substring-no-properties (point-min) (point-max))))
    (setq checks
          (list
           (soma-bridge-bench--check
            "stale edit is rejected as conflict"
            (and (string= (soma-bridge-bench--get response "state")
                          "failed")
                 (string= (soma-bridge-bench--get response "error_code")
                          "conflict"))
            (format "%S" response))
           (soma-bridge-bench--check
            "stale edit did not apply replacement"
            (string= buffer-text "alpha human-beta\n")
            buffer-text)
           (soma-bridge-bench--check
            "failed stale edit did not advance revision further"
            (= (soma-bridge-bench--get after "chars_tick")
               (soma-bridge-bench--get after-human "chars_tick"))
            (format "after-human=%S after-request=%S"
                    (soma-bridge-bench--get after-human "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "before" before)
     (cons "after_human_edit" after-human)
     (cons "after" after)
     (cons "buffer_text" buffer-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-checked-apply-patch ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-apply-patch"
                  (concat
                   "fn alpha() {\n"
                   "  return 1;\n"
                   "}\n\n"
                   "fn beta() {\n"
                   "  return 2;\n"
                   "}\n")))
         (manifest (soma-gym-bridge-list-capabilities))
         (patch-entry
          (soma-bridge-bench--find-operation manifest "buffer.apply_patch"))
         before patch preconditions request response result after buffer-text
         stale-buffer stale-before stale-patch stale-preconditions
         stale-request stale-response stale-after stale-buffer-text
         checks)
    (set-window-buffer (selected-window) buffer)
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (setq patch
          (soma-bridge-bench--alist
           (cons "format" "soma-context-patch-v0")
           (cons "hunks"
                 (list
                  (soma-bridge-bench--alist
                   (cons "old_text" "  return 1;\n")
                   (cons "new_text" "  return 10;\n"))
                  (soma-bridge-bench--alist
                   (cons "old_text" "fn beta() {\n")
                   (cons "new_text" "fn beta_renamed() {\n"))))))
    (setq preconditions
          (soma-bridge-bench--buffer-edit-preconditions before nil))
    (setq request
          (soma-bridge-bench--buffer-apply-patch-request
           "apply-patch-1" buffer patch preconditions))
    (setq response (soma-gym-bridge-submit-request request))
    (setq result (soma-bridge-bench--get response "result"))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq buffer-text (soma-bridge-bench--buffer-full-text buffer))
    (setq stale-buffer
          (soma-bridge-bench--make-buffer
           "soma-bridge-apply-patch-stale"
           "alpha\nbeta\ngamma\n"))
    (set-window-buffer (selected-window) stale-buffer)
    (setq stale-before (soma-gym-bridge-observe-buffer stale-buffer))
    (setq stale-patch
          (soma-bridge-bench--alist
           (cons "format" "soma-context-patch-v0")
           (cons "hunks"
                 (list
                  (soma-bridge-bench--alist
                   (cons "old_text" "beta\n")
                   (cons "new_text" "BETA\n"))))))
    (setq stale-preconditions
          (soma-bridge-bench--buffer-edit-preconditions stale-before nil))
    (with-current-buffer stale-buffer
      (goto-char (point-min))
      (insert "human\n"))
    (setq stale-request
          (soma-bridge-bench--buffer-apply-patch-request
           "apply-patch-stale-1" stale-buffer stale-patch
           stale-preconditions))
    (setq stale-response (soma-gym-bridge-submit-request stale-request))
    (setq stale-after (soma-gym-bridge-observe-buffer stale-buffer))
    (setq stale-buffer-text
          (soma-bridge-bench--buffer-full-text stale-buffer))
    (setq checks
          (list
           (soma-bridge-bench--check
            "buffer.apply_patch is a stable enabled coding operation"
            (and patch-entry
                 (string= (soma-bridge-bench--get patch-entry "profile")
                          "coding-v0")
                 (string= (soma-bridge-bench--get patch-entry "maturity")
                          "stable")
                 (eq (soma-bridge-bench--get patch-entry "implemented") t)
                 (eq (soma-bridge-bench--get patch-entry "enabled") t)
                 (string= (soma-bridge-bench--get patch-entry "route")
                          "checked-buffer-patch"))
            (format "%S" patch-entry))
           (soma-bridge-bench--check
            "buffer.apply_patch applies multiple checked hunks"
            (and (string= (soma-bridge-bench--get response "state")
                          "succeeded")
                 (string= (soma-bridge-bench--get result "route")
                          "checked-buffer-patch")
                 (string= buffer-text
                          (concat
                           "fn alpha() {\n"
                           "  return 10;\n"
                           "}\n\n"
                           "fn beta_renamed() {\n"
                           "  return 2;\n"
                           "}\n")))
            (format "%S" result))
           (soma-bridge-bench--check
            "buffer.apply_patch advances buffer revision once it mutates text"
            (< (soma-bridge-bench--get before "chars_tick")
               (soma-bridge-bench--get after "chars_tick"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))
           (soma-bridge-bench--check
            "buffer.apply_patch publishes typed evidence observations and effects"
            (let* ((evidence (soma-bridge-bench--get response "evidence"))
                   (observations
                    (soma-bridge-bench--get response "observations"))
                   (effects (soma-bridge-bench--get response "effects"))
                   (before-observation
                    (soma-bridge-bench--get observations "before"))
                   (after-observation
                    (soma-bridge-bench--get observations "after"))
                   (changed-ranges
                    (soma-bridge-bench--get effects "changed_ranges")))
              (and (soma-bridge-bench--object-p evidence)
                   (soma-bridge-bench--object-p observations)
                   (soma-bridge-bench--object-p effects)
                   (soma-bridge-bench--get evidence "precondition_checks")
                   (soma-bridge-bench--get evidence "conflict_preview")
                   (soma-bridge-bench--get
                    evidence "before_after_observations")
                   (soma-bridge-bench--object-p before-observation)
                   (soma-bridge-bench--object-p after-observation)
                   (= (soma-bridge-bench--get
                       before-observation "chars_tick")
                      (soma-bridge-bench--get before "chars_tick"))
                   (> (soma-bridge-bench--get
                       after-observation "chars_tick")
                      (soma-bridge-bench--get before "chars_tick"))
                   (= (length changed-ranges) 2)))
            (format "%S" response))
           (soma-bridge-bench--check
            "buffer.apply_patch rejects stale buffer state before mutation"
            (and (string= (soma-bridge-bench--get stale-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get stale-response
                                                   "error_code")
                          "conflict")
                 (string= stale-buffer-text "human\nalpha\nbeta\ngamma\n")
                 (= (soma-bridge-bench--get stale-after "chars_tick")
                    (with-current-buffer stale-buffer
                      (buffer-chars-modified-tick))))
            (format "%S" stale-response))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "before" before)
     (cons "after" after)
     (cons "buffer_text" buffer-text)
     (cons "stale_request" stale-request)
     (cons "stale_response" stale-response)
     (cons "stale_buffer_text" stale-buffer-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-point-edit-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-point-edit" "alpha\nbeta\n"))
         before after-point request response after buffer-text
         point-check checks)
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (with-current-buffer buffer
      (goto-char 7))
    (setq after-point (soma-gym-bridge-observe-buffer buffer))
    (setq request
          (soma-bridge-bench--buffer-replace-request
           "point-edit-1" buffer 7 11 "gamma"
           (soma-bridge-bench--alist
            (cons "buffer_name" (buffer-name buffer))
            (cons "chars_tick" (soma-bridge-bench--get before "chars_tick"))
            (cons "point" (soma-bridge-bench--get before "point"))
            (cons "expected_text" "beta"))))
    (setq response (soma-gym-bridge-submit-request request))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq buffer-text (soma-bridge-bench--buffer-full-text buffer))
    (setq point-check
          (soma-bridge-bench--precondition-check-named
           response "point matches precondition"))
    (setq checks
          (list
           (soma-bridge-bench--check
            "point changed without text revision"
            (and (= (soma-bridge-bench--get after-point "point") 7)
                 (= (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after-point "chars_tick")))
            (format "before=%S after-point=%S" before after-point))
           (soma-bridge-bench--check
            "point-drift edit is rejected as conflict"
            (and (string= (soma-bridge-bench--get response "state")
                          "failed")
                 (string= (soma-bridge-bench--get response "error_code")
                          "conflict"))
            (format "%S" response))
           (soma-bridge-bench--check
            "point precondition is the rejecting check"
            (and (listp point-check)
                 (eq (soma-bridge-bench--get point-check "passed")
                     :json-false))
            (format "%S" point-check))
           (soma-bridge-bench--check
            "point-drift edit did not mutate text"
            (string= buffer-text "alpha\nbeta\n")
            buffer-text)
           (soma-bridge-bench--check
            "failed point-drift edit did not advance revision"
            (= (soma-bridge-bench--get before "chars_tick")
               (soma-bridge-bench--get after "chars_tick"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "before" before)
     (cons "after_point" after-point)
     (cons "after" after)
     (cons "buffer_text" buffer-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-narrowed-edit-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-narrowed-edit" "alpha\nbeta\nomega\n"))
         before after-narrow request response after buffer-text
         narrowing-check checks)
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (with-current-buffer buffer
      (narrow-to-region 1 12))
    (setq after-narrow (soma-gym-bridge-observe-buffer buffer))
    (setq request
          (soma-bridge-bench--buffer-replace-request
           "narrowed-edit-1" buffer 7 11 "gamma"
           (soma-bridge-bench--alist
            (cons "buffer_name" (buffer-name buffer))
            (cons "chars_tick" (soma-bridge-bench--get before "chars_tick"))
            (cons "narrowed" :json-false)
            (cons "expected_text" "beta"))))
    (setq response (soma-gym-bridge-submit-request request))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq buffer-text (soma-bridge-bench--buffer-full-text buffer))
    (setq narrowing-check
          (soma-bridge-bench--precondition-check-named
           response "narrowing state matches precondition"))
    (setq checks
          (list
           (soma-bridge-bench--check
            "narrowing changed without text revision"
            (and (eq (soma-bridge-bench--get after-narrow "narrowed") t)
                 (= (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after-narrow "chars_tick")))
            (format "before=%S after-narrow=%S" before after-narrow))
           (soma-bridge-bench--check
            "narrowed edit is rejected as conflict"
            (and (string= (soma-bridge-bench--get response "state")
                          "failed")
                 (string= (soma-bridge-bench--get response "error_code")
                          "conflict"))
            (format "%S" response))
           (soma-bridge-bench--check
            "narrowing precondition is the rejecting check"
            (and (listp narrowing-check)
                 (eq (soma-bridge-bench--get narrowing-check "passed")
                     :json-false))
            (format "%S" narrowing-check))
           (soma-bridge-bench--check
            "narrowed edit did not mutate underlying text"
            (string= buffer-text "alpha\nbeta\nomega\n")
            buffer-text)
           (soma-bridge-bench--check
            "failed narrowed edit did not advance revision"
            (= (soma-bridge-bench--get before "chars_tick")
               (soma-bridge-bench--get after "chars_tick"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "before" before)
     (cons "after_narrow" after-narrow)
     (cons "after" after)
     (cons "buffer_text" buffer-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-mark-active-edit-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-mark-active-edit" "alpha\nbeta\n"))
         before after-region request response after buffer-text
         mark-check checks)
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (with-current-buffer buffer
      (goto-char 7)
      (set-mark 11)
      (setq mark-active t))
    (setq after-region (soma-gym-bridge-observe-buffer buffer))
    (setq request
          (soma-bridge-bench--buffer-replace-request
           "mark-active-edit-1" buffer 7 11 "gamma"
           (soma-bridge-bench--alist
            (cons "buffer_name" (buffer-name buffer))
            (cons "chars_tick" (soma-bridge-bench--get before "chars_tick"))
            (cons "mark_active" :json-false)
            (cons "expected_text" "beta"))))
    (setq response (soma-gym-bridge-submit-request request))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq buffer-text (soma-bridge-bench--buffer-full-text buffer))
    (setq mark-check
          (soma-bridge-bench--precondition-check-named
           response "region active state matches precondition"))
    (setq checks
          (list
           (soma-bridge-bench--check
            "active region changed without text revision"
            (and (eq (soma-bridge-bench--get after-region "mark_active") t)
                 (= (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after-region "chars_tick")))
            (format "before=%S after-region=%S" before after-region))
           (soma-bridge-bench--check
            "mark-active edit is rejected as conflict"
            (and (string= (soma-bridge-bench--get response "state")
                          "failed")
                 (string= (soma-bridge-bench--get response "error_code")
                          "conflict"))
            (format "%S" response))
           (soma-bridge-bench--check
            "mark-active precondition is the rejecting check"
            (and (listp mark-check)
                 (eq (soma-bridge-bench--get mark-check "passed")
                     :json-false))
            (format "%S" mark-check))
           (soma-bridge-bench--check
            "mark-active edit did not mutate text"
            (string= buffer-text "alpha\nbeta\n")
            buffer-text)
           (soma-bridge-bench--check
            "failed mark-active edit did not advance revision"
            (= (soma-bridge-bench--get before "chars_tick")
               (soma-bridge-bench--get after "chars_tick"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "before" before)
     (cons "after_region" after-region)
     (cons "after" after)
     (cons "buffer_text" buffer-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-read-only-edit-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-read-only-edit" "alpha\nbeta\n"))
         before after-read-only request response after buffer-text
         read-only-check checks)
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (with-current-buffer buffer
      (setq buffer-read-only t))
    (setq after-read-only (soma-gym-bridge-observe-buffer buffer))
    (setq request
          (soma-bridge-bench--buffer-replace-request
           "read-only-edit-1" buffer 7 11 "gamma"
           (soma-bridge-bench--alist
            (cons "buffer_name" (buffer-name buffer))
            (cons "chars_tick" (soma-bridge-bench--get before "chars_tick"))
            (cons "read_only" :json-false)
            (cons "expected_text" "beta"))))
    (setq response (soma-gym-bridge-submit-request request))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq buffer-text (soma-bridge-bench--buffer-full-text buffer))
    (setq read-only-check
          (soma-bridge-bench--precondition-check-named
           response "read-only state matches precondition"))
    (setq checks
          (list
           (soma-bridge-bench--check
            "read-only changed without text revision"
            (and (eq (soma-bridge-bench--get after-read-only "read_only") t)
                 (= (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after-read-only "chars_tick")))
            (format "before=%S after-read-only=%S" before after-read-only))
           (soma-bridge-bench--check
            "read-only edit is rejected as conflict"
            (and (string= (soma-bridge-bench--get response "state")
                          "failed")
                 (string= (soma-bridge-bench--get response "error_code")
                          "conflict"))
            (format "%S" response))
           (soma-bridge-bench--check
            "read-only precondition is the rejecting check"
            (and (listp read-only-check)
                 (eq (soma-bridge-bench--get read-only-check "passed")
                     :json-false))
            (format "%S" read-only-check))
           (soma-bridge-bench--check
            "read-only edit did not mutate text"
            (string= buffer-text "alpha\nbeta\n")
            buffer-text)
           (soma-bridge-bench--check
            "failed read-only edit did not advance revision"
            (= (soma-bridge-bench--get before "chars_tick")
               (soma-bridge-bench--get after "chars_tick"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "before" before)
     (cons "after_read_only" after-read-only)
     (cons "after" after)
     (cons "buffer_text" buffer-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-completion-accept ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-completion-accept" "alpha \nomega\n"))
         observe-request observe-response observed-completion
         replacement-range candidates acceptance-preconditions
         before request response result after buffer-text checks)
    (with-current-buffer buffer
      (goto-char 7)
      (setq-local completion-at-point-functions
                  (list
                   (lambda ()
                     (list 7 7 '("beta" "better" "gamma")
                           :exclusive 'no)))))
    (setq observe-request
          (soma-bridge-bench--completion-observe-request
           "completion-observe-1" buffer 8))
    (setq observe-response (soma-gym-bridge-submit-request observe-request))
    (setq observed-completion
          (soma-bridge-bench--get observe-response "result"))
    (setq replacement-range
          (soma-bridge-bench--get observed-completion "replacement_range"))
    (setq candidates
          (soma-bridge-bench--sequence-values
           (soma-bridge-bench--get observed-completion "candidates")))
    (setq acceptance-preconditions
          (soma-bridge-bench--get observed-completion
                                  "acceptance_preconditions"))
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (setq request
          (soma-bridge-bench--completion-accept-request
           "completion-accept-1" buffer
           (soma-bridge-bench--get replacement-range "start")
           (soma-bridge-bench--get replacement-range "end")
           "beta"
           acceptance-preconditions))
    (setq response (soma-gym-bridge-submit-request request))
    (setq result (soma-bridge-bench--get response "result"))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq buffer-text (soma-bridge-bench--buffer-full-text buffer))
    (setq checks
          (list
           (soma-bridge-bench--check
            "completion observe request succeeded"
            (and (string= (soma-bridge-bench--get observe-response "state")
                          "succeeded")
                 (string= (soma-bridge-bench--get
                           observed-completion "operation")
                          "completion.observe"))
            (format "%S" observe-response))
           (soma-bridge-bench--check
            "completion observe reports provider range and candidates"
            (and (equal (soma-bridge-bench--get replacement-range "start") 7)
                 (equal (soma-bridge-bench--get replacement-range "end") 7)
                 (member "beta" candidates))
            (format "%S" observed-completion))
           (soma-bridge-bench--check
            "completion observe returns accept-ready context preconditions"
            (and (listp acceptance-preconditions)
                 (equal (soma-bridge-bench--get
                         acceptance-preconditions "chars_tick")
                        (soma-bridge-bench--get before "chars_tick"))
                 (string= (soma-bridge-bench--get
                           acceptance-preconditions "expected_prefix")
                          "alpha ")
                 (string= (soma-bridge-bench--get
                           acceptance-preconditions "expected_suffix")
                          "\nomega\n"))
            (format "%S" acceptance-preconditions))
           (soma-bridge-bench--check
            "completion accept request succeeded"
            (string= (soma-bridge-bench--get response "state")
                     "succeeded")
            (format "%S" response))
           (soma-bridge-bench--check
            "completion accept used checked operation"
            (and (listp result)
                 (string= (soma-bridge-bench--get result "operation")
                          "completion.accept"))
            (format "%S" result))
           (soma-bridge-bench--check
            "completion inserted only the proposed text"
            (string= buffer-text "alpha beta\nomega\n")
            buffer-text)
           (soma-bridge-bench--check
            "completion accept advanced character revision"
            (< (soma-bridge-bench--get before "chars_tick")
               (soma-bridge-bench--get after "chars_tick"))
            (format "before=%S after=%S"
                    (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))
           (soma-bridge-bench--check
            "completion accept records terminal success"
            (equal (soma-bridge-bench--get response "state_sequence")
                   '("accepted" "running" "succeeded"))
            (format "%S" response))))
    (soma-bridge-bench--alist
     (cons "observe_request" observe-request)
     (cons "observe_response" observe-response)
     (cons "request" request)
     (cons "response" response)
     (cons "before" before)
     (cons "after" after)
     (cons "buffer_text" buffer-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-stale-completion-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-stale-completion" "alpha \nomega\n"))
         observe-request observe-response observed-completion replacement-range
         acceptance-preconditions before after-human after-human-text request
         response after buffer-text suffix-check checks)
    (with-current-buffer buffer
      (goto-char 7)
      (setq-local completion-at-point-functions
                  (list
                   (lambda ()
                     (list 7 7 '("beta" "better" "gamma")
                           :exclusive 'no)))))
    (setq observe-request
          (soma-bridge-bench--completion-observe-request
           "stale-completion-observe-1" buffer 8))
    (setq observe-response (soma-gym-bridge-submit-request observe-request))
    (setq observed-completion
          (soma-bridge-bench--get observe-response "result"))
    (setq replacement-range
          (soma-bridge-bench--get observed-completion "replacement_range"))
    (setq acceptance-preconditions
          (soma-bridge-bench--get observed-completion
                                  "acceptance_preconditions"))
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (with-current-buffer buffer
      (insert "human-"))
    (setq after-human (soma-gym-bridge-observe-buffer buffer))
    (setq after-human-text (soma-bridge-bench--buffer-full-text buffer))
    (setq request
          (soma-bridge-bench--completion-accept-request
           "stale-completion-1" buffer
           (soma-bridge-bench--get replacement-range "start")
           (soma-bridge-bench--get replacement-range "end")
           "beta"
           acceptance-preconditions))
    (setq response (soma-gym-bridge-submit-request request))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq buffer-text (soma-bridge-bench--buffer-full-text buffer))
    (setq suffix-check
          (soma-bridge-bench--precondition-check-named
           response "completion suffix matches precondition"))
    (setq checks
          (list
           (soma-bridge-bench--check
            "completion observe request succeeded before human edit"
            (string= (soma-bridge-bench--get observe-response "state")
                     "succeeded")
            (format "%S" observe-response))
           (soma-bridge-bench--check
            "human edit changed completion context"
            (and (< (soma-bridge-bench--get before "chars_tick")
                    (soma-bridge-bench--get after-human "chars_tick"))
                 (string= after-human-text "alpha human-\nomega\n"))
            (format "before=%S after-human=%S after-human-text=%S"
                    before after-human after-human-text))
           (soma-bridge-bench--check
            "stale completion accept is rejected as conflict"
            (and (string= (soma-bridge-bench--get response "state")
                          "failed")
                 (string= (soma-bridge-bench--get response "error_code")
                          "conflict"))
            (format "%S" response))
           (soma-bridge-bench--check
            "suffix precondition is the rejecting context check"
            (and (listp suffix-check)
                 (eq (soma-bridge-bench--get suffix-check "passed")
                     :json-false))
            (format "%S" suffix-check))
           (soma-bridge-bench--check
            "stale completion did not insert proposed text"
            (string= buffer-text "alpha human-\nomega\n")
            buffer-text)
           (soma-bridge-bench--check
            "failed stale completion did not advance revision further"
            (= (soma-bridge-bench--get after-human "chars_tick")
               (soma-bridge-bench--get after "chars_tick"))
            (format "after-human=%S after=%S"
                    (soma-bridge-bench--get after-human "chars_tick")
                    (soma-bridge-bench--get after "chars_tick")))))
    (soma-bridge-bench--alist
     (cons "observe_request" observe-request)
     (cons "observe_response" observe-response)
     (cons "request" request)
     (cons "response" response)
     (cons "before" before)
     (cons "after_human_edit" after-human)
     (cons "after_human_text" after-human-text)
     (cons "after" after)
     (cons "buffer_text" buffer-text)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-episode-finish ()
  (soma-gym-bridge-reset-transport)
  (let* ((file (concat
                (make-temp-name
                 (expand-file-name "soma-bridge-episode-finish-"
                                   temporary-file-directory))
                ".txt"))
         (buffer (find-file-noselect file))
         before edit-preconditions edit-request edit-response edit-result
         save-request save-response after finish-request
         finish-response finish-result buffer-checks request-checks checks)
    (with-current-buffer buffer
      (setq-local make-backup-files nil)
      (erase-buffer)
      (insert "alpha\nbeta\n")
      (save-buffer)
      (goto-char 1))
    (set-window-buffer (selected-window) buffer)
    (setq before (soma-gym-bridge-observe-buffer buffer))
    (setq edit-preconditions
          (soma-bridge-bench--alist
           (cons "buffer_name" (buffer-name buffer))
           (cons "buffer_file_name" file)
           (cons "text_owner_name" (buffer-name buffer))
           (cons "chars_tick" (soma-bridge-bench--get before "chars_tick"))
           (cons "expected_text" "beta")))
    (setq edit-request
          (soma-bridge-bench--buffer-replace-request
           "finish-edit-1" buffer 7 11 "gamma" edit-preconditions))
    (setq edit-response (soma-gym-bridge-submit-request edit-request))
    (setq edit-result (soma-bridge-bench--get edit-response "result"))
    (setq save-request
          (soma-bridge-bench--buffer-save-request
           "finish-save-1" buffer
           (soma-bridge-bench--alist
            (cons "buffer_name" (buffer-name buffer))
            (cons "buffer_file_name" file)
            (cons "chars_tick"
                  (if (listp edit-result)
                      (soma-bridge-bench--get
                       (soma-bridge-bench--get edit-result "after")
                       "chars_tick")
                    (with-current-buffer buffer
                      (buffer-chars-modified-tick)))))))
    (setq save-response (soma-gym-bridge-submit-request save-request))
    (setq after (soma-gym-bridge-observe-buffer buffer))
    (setq finish-request
          (soma-bridge-bench--episode-finish-request
           "finish-1"
           (list
            (soma-bridge-bench--alist
             (cons "request_id" "finish-edit-1")
             (cons "state" "succeeded"))
            (soma-bridge-bench--alist
             (cons "request_id" "finish-save-1")
             (cons "state" "succeeded")))
           (list
            (soma-bridge-bench--alist
             (cons "buffer_name" (buffer-name buffer))
             (cons "start" 1)
             (cons "end" (with-current-buffer buffer (point-max)))
             (cons "preconditions"
                   (soma-bridge-bench--alist
                    (cons "buffer_name" (buffer-name buffer))
                    (cons "buffer_file_name" file)
                    (cons "chars_tick"
                          (soma-bridge-bench--get after "chars_tick"))
                    (cons "modified" :json-false)
                    (cons "expected_text" "alpha\ngamma\n")))))))
    (setq finish-response (soma-gym-bridge-submit-request finish-request))
    (setq finish-result (soma-bridge-bench--get finish-response "result"))
    (setq request-checks
          (and (listp finish-result)
               (soma-bridge-bench--get finish-result "request_checks")))
    (setq buffer-checks
          (and (listp finish-result)
               (soma-bridge-bench--get finish-result "buffer_checks")))
    (setq checks
          (list
           (soma-bridge-bench--check
            "finish request succeeded"
            (string= (soma-bridge-bench--get finish-response "state")
                     "succeeded")
            (format "%S" finish-response))
           (soma-bridge-bench--check
            "finish result records episode completion"
            (and (listp finish-result)
                 (string= (soma-bridge-bench--get finish-result "state")
                          "succeeded"))
            (format "%S" finish-result))
           (soma-bridge-bench--check
            "finish validated required edit and save requests"
            (and (listp request-checks)
                 (= (length request-checks) 2)
                 (member "finish-edit-1"
                         (mapcar
                          (lambda (check)
                            (soma-bridge-bench--get check "request_id"))
                          request-checks))
                 (member "finish-save-1"
                         (mapcar
                          (lambda (check)
                            (soma-bridge-bench--get check "request_id"))
                          request-checks))
                 (soma-bridge-bench--all-checks-pass-p request-checks))
            (format "%S" request-checks))
           (soma-bridge-bench--check
            "finish validated final buffer state"
            (and (listp buffer-checks)
                 (= (length buffer-checks) 1)
                 (soma-bridge-bench--all-checks-pass-p
                  (soma-bridge-bench--get (car buffer-checks) "checks")))
            (format "%S" buffer-checks))))
    (soma-bridge-bench--alist
     (cons "file" file)
     (cons "edit_response" edit-response)
     (cons "save_response" save-response)
     (cons "finish_request" finish-request)
     (cons "finish_response" finish-response)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-duplicate-request ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-duplicate" "alpha\n"))
         (request (soma-bridge-bench--observe-request "duplicate-1" buffer))
         (first (soma-gym-bridge-submit-request request))
         (second (soma-gym-bridge-submit-request request))
         (status (soma-gym-bridge-request-status "duplicate-1"))
         (checks
          (list
           (soma-bridge-bench--check
            "first request succeeds"
            (string= (soma-bridge-bench--get first "state") "succeeded")
            (format "%S" first))
           (soma-bridge-bench--check
            "duplicate request id is rejected"
            (and (string= (soma-bridge-bench--get second "state") "failed")
                 (string= (soma-bridge-bench--get second "error_code")
                          "duplicate_request_id"))
            (format "%S" second))
           (soma-bridge-bench--check
            "duplicate does not overwrite stored request"
            (and (string= (soma-bridge-bench--get status "request_id")
                          "duplicate-1")
                 (string= (soma-bridge-bench--get status "state")
                          "succeeded"))
            (format "%S" status)))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "first_response" first)
     (cons "second_response" second)
     (cons "status" status)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-async-terminal-publication-preserved ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-async-publication" "def old_name():\n    pass\n"))
         (request
          (soma-bridge-bench--alist
           (cons "request_id" "async-terminal-publication-1")
           (cons "operation" "lsp.rename_symbol")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer))
                  (cons "new_name" "new_name")
                  (cons "timeout_seconds" 1.0)))))
         response status checks)
    (let ((orig-run-at-time (symbol-function 'run-at-time))
          (orig-lsp-rename
           (symbol-function 'soma-gym-bridge-lsp-rename-symbol)))
      (unwind-protect
          (progn
            (fset 'run-at-time
                  (lambda (_delay _repeat function &rest args)
                    (apply function args)
                    nil))
            (fset 'soma-gym-bridge-lsp-rename-symbol
                  (lambda (_buffer new-name _preconditions timeout)
                    (soma-bridge-bench--alist
                     (cons "operation" "lsp.rename_symbol")
                     (cons "state" "succeeded")
                     (cons "state_sequence"
                           '("accepted" "running" "succeeded"))
                     (cons "route" "lsp-rename")
                     (cons "new_name" new-name)
                     (cons "wait_timeout_seconds" timeout)
                     (cons "request_methods" '("textDocument/rename"))
                     (cons "requests" nil)
                     (cons "workspace_edit_applied" t)
                     (cons "workspace_edit_sha256" "stubbed-workspace-edit")
                     (cons "workspace_edit_excerpt" "stubbed workspace edit")
                     (cons "saved_buffers" nil)
                     (cons "precondition_checks" nil)
                     (cons "before" (soma-gym-bridge-observe-buffer buffer))
                     (cons "after" (soma-gym-bridge-observe-buffer buffer))
                     (cons "error" :json-false))))
            (setq response (soma-gym-bridge-submit-request request)))
        (fset 'run-at-time orig-run-at-time)
        (fset 'soma-gym-bridge-lsp-rename-symbol orig-lsp-rename)))
    (setq status
          (soma-gym-bridge-request-status "async-terminal-publication-1"))
    (setq checks
          (list
           (soma-bridge-bench--check
            "initial async response remains running"
            (and (string= (soma-bridge-bench--get response "state")
                          "running")
                 (string= (soma-bridge-bench--get
                           (soma-bridge-bench--get response "result")
                           "route")
                          "queued-lsp-rename"))
            (format "%S" response))
           (soma-bridge-bench--check
            "terminal status published by the queued action is preserved"
            (and (string= (soma-bridge-bench--get status "state")
                          "succeeded")
                 (string= (soma-bridge-bench--get
                           (soma-bridge-bench--get status "result")
                           "route")
                          "lsp-rename"))
            (format "%S" status))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "status" status)
     (cons "checks" checks))))

(defun soma-bridge-bench--scenario-direct-call-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((request
          (soma-bridge-bench--alist
           (cons "request_id" "direct-1")
           (cons "operation" "direct-call")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "requested_operation" "magit-stage-1")))))
         (eval-request
          (soma-bridge-bench--alist
           (cons "request_id" "elisp-eval-1")
           (cons "operation" "elisp.eval")
           (cons "params"
                 (soma-bridge-bench--alist
                  (cons "form"
                        "(setq soma-bridge-bench--elisp-eval-ran t)")))))
         (response (soma-gym-bridge-submit-request request))
         (eval-response (soma-gym-bridge-submit-request eval-request))
         (status (soma-gym-bridge-request-status "direct-1"))
         (eval-status (soma-gym-bridge-request-status "elisp-eval-1"))
         (result (soma-bridge-bench--get response "result"))
         (eval-result (soma-bridge-bench--get eval-response "result"))
         (checks
          (list
           (soma-bridge-bench--check
            "direct operation is rejected"
            (and (string= (soma-bridge-bench--get response "state") "failed")
                 (string= (soma-bridge-bench--get response "error_code")
                          "unauthorized_operation"))
            (format "%S" response))
           (soma-bridge-bench--check
            "failed response carries structured envelope diagnostics"
            (let ((diagnostics
                   (soma-bridge-bench--get response "diagnostics")))
              (and (vectorp diagnostics)
                   (= (length diagnostics) 1)
                   (string= (soma-bridge-bench--get
                             (aref diagnostics 0) "code")
                            "unauthorized_operation")))
            (format "%S" response))
           (soma-bridge-bench--check
            "direct-call rejection is stored"
            (and (string= (soma-bridge-bench--get status "request_id")
                          "direct-1")
                 (string= (soma-bridge-bench--get status "state")
                          "failed"))
            (format "%S" status))
           (soma-bridge-bench--check
            "direct-call result reports checked rejection"
            (and (string= (soma-bridge-bench--get result "operation")
                          "direct-call")
                 (string= (soma-bridge-bench--get result "state")
                          "failed"))
            (format "%S" result))
           (soma-bridge-bench--check
            "arbitrary Elisp eval operation is rejected"
            (and (string= (soma-bridge-bench--get eval-response "state")
                          "failed")
                 (string= (soma-bridge-bench--get eval-response "error_code")
                          "unauthorized_operation"))
            (format "%S" eval-response))
           (soma-bridge-bench--check
            "arbitrary Elisp eval result reports no evaluation"
            (and (string= (soma-bridge-bench--get eval-result "operation")
                          "elisp.eval")
                 (eq (soma-bridge-bench--get eval-result "evaluated")
                     :json-false))
            (format "%S" eval-result))
           (soma-bridge-bench--check
            "arbitrary Elisp eval rejection is stored"
            (and (string= (soma-bridge-bench--get eval-status "request_id")
                          "elisp-eval-1")
                 (string= (soma-bridge-bench--get eval-status "state")
                          "failed"))
            (format "%S" eval-status)))))
    (soma-bridge-bench--alist
     (cons "request" request)
     (cons "response" response)
     (cons "status" status)
     (cons "eval_request" eval-request)
     (cons "eval_response" eval-response)
     (cons "eval_status" eval-status)
     (cons "checks" checks))))

(defun soma-bridge-bench--json-read (text)
  (let ((json-object-type 'hash-table)
        (json-array-type 'vector)
        (json-key-type 'string)
        (json-false :json-false))
    (json-read-from-string text)))

(defun soma-bridge-bench--socket-roundtrip (socket request)
  (let ((response-text "")
        response
        client)
    (unwind-protect
        (progn
          (setq client
                (make-network-process
                 :name "soma-bridge-bench-client"
                 :family 'local
                 :service socket
                 :filter
                 (lambda (_process chunk)
                   (setq response-text (concat response-text chunk)))))
          (process-send-string client (concat (json-encode request) "\n"))
          (let ((deadline (+ (float-time) 5.0)))
            (while (and (not (string-match-p "\n" response-text))
                        (< (float-time) deadline))
              (accept-process-output nil 0.05)))
          (unless (string-match "\\`\\([^\n]+\\)\n" response-text)
            (error "timed out waiting for bridge socket response: %S"
                   response-text))
          (setq response
                (soma-bridge-bench--json-read (match-string 1 response-text))))
      (when (process-live-p client)
        (delete-process client)))
    response))

(defun soma-bridge-bench--scenario-socket-jsonl-roundtrip ()
  (soma-gym-bridge-reset-transport)
  (let* ((buffer (soma-bridge-bench--make-buffer
                  "soma-bridge-socket" "alpha\n"))
         (socket (expand-file-name
                  (format "soma-bridge-%s.sock" (or (getenv "SOMA_INSULA_RUN_ID") "run"))
                  (or (getenv "XDG_RUNTIME_DIR") temporary-file-directory)))
         (request (soma-bridge-bench--observe-request "socket-1" buffer))
         response result checks)
    (unwind-protect
        (progn
          (soma-gym-bridge-start-jsonl-server socket)
          (setq response (soma-bridge-bench--socket-roundtrip socket request))
          (setq result (soma-bridge-bench--get response "result"))
          (setq checks
                (list
                 (soma-bridge-bench--check
                  "socket request id is echoed"
                  (string= (soma-bridge-bench--get response "request_id")
                           "socket-1")
                  (format "%S" response))
                 (soma-bridge-bench--check
                  "socket request succeeded"
                  (string= (soma-bridge-bench--get response "state")
                           "succeeded")
                  (format "%S" response))
                 (soma-bridge-bench--check
                  "socket observe result names target buffer"
                  (and (string= (soma-bridge-bench--get result "operation")
                                "emacs.observe_buffer")
                       (string= (soma-bridge-bench--get result "buffer_name")
                                (buffer-name buffer)))
                  (format "%S" result))))
          (soma-bridge-bench--alist
           (cons "request" request)
           (cons "response" response)
           (cons "socket" socket)
           (cons "checks" checks)))
      (soma-gym-bridge-stop-jsonl-server)
      (when (file-exists-p socket)
        (delete-file socket)))))

(defun soma-bridge-bench--scenario-comint-semantic-api ()
  (soma-gym-bridge-reset-transport)
  (let* ((tmp (make-temp-file "soma-bridge-comint-api-" t))
         (counter (expand-file-name "counter.txt" tmp))
         (cell (soma-bridge-bench--make-comint-shell
                "soma-bridge-comint-api"))
         (buffer (car cell))
         (semantic-label "semantic")
         (stale-label "stale")
         (done-needle
          (format "SOMA_BRIDGE_COMINT_DONE:%s" semantic-label))
         (stale-done-needle
          (format "SOMA_BRIDGE_COMINT_DONE:%s" stale-label))
         initial-request initial-response initial-result submit-request
         submit-response submit-result submit-after await-request
         await-response await-result stale-request stale-response
         stale-result final-observe counter-lines checks)
    (unwind-protect
        (progn
          (setq initial-request
                (soma-bridge-bench--api-request
                 "comint-api-observe-1" "comint.observe"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer)))))
          (setq initial-response
                (soma-gym-bridge-submit-request initial-request))
          (setq initial-result
                (soma-bridge-bench--get initial-response "result"))
          (setq submit-request
                (soma-bridge-bench--comint-submit-request
                 "comint-api-submit-1" buffer
                 (soma-bridge-bench--comint-command
                  semantic-label counter)
                 initial-result))
          (setq submit-response
                (soma-gym-bridge-submit-request submit-request))
          (setq submit-result
                (soma-bridge-bench--get submit-response "result"))
          (setq submit-after
                (or (and (listp submit-result)
                         (soma-bridge-bench--get submit-result "after"))
                    initial-result))
          (setq await-request
                (soma-bridge-bench--comint-await-request
                 "comint-api-await-1" buffer done-needle submit-after 3.0))
          (setq await-response
                (soma-gym-bridge-submit-request await-request))
          (setq await-result
                (soma-bridge-bench--get await-response "result"))
          (setq stale-request
                (soma-bridge-bench--comint-submit-request
                 "comint-api-stale-submit-1" buffer
                 (soma-bridge-bench--comint-command
                  stale-label counter)
                 initial-result))
          (setq stale-response
                (soma-gym-bridge-submit-request stale-request))
          (setq stale-result
                (soma-bridge-bench--get stale-response "result"))
          (setq final-observe
                (soma-gym-bridge-observe-comint buffer))
          (setq counter-lines
                (soma-bridge-bench--file-lines counter))
          (setq checks
                (list
                 (soma-bridge-bench--check
                  "comint.submit_input is a stable enabled terminal operation"
                  (let* ((manifest (soma-gym-bridge-list-capabilities))
                         (entry (soma-bridge-bench--find-operation
                                 manifest "comint.submit_input")))
                    (and entry
                         (string= (soma-bridge-bench--get entry "profile")
                                  "terminal-v0")
                         (string= (soma-bridge-bench--get entry "maturity")
                                  "stable")
                         (eq (soma-bridge-bench--get entry "implemented")
                             t)
                         (eq (soma-bridge-bench--get entry "enabled") t)
                         (string= (soma-bridge-bench--get entry "route")
                                  "comint-send-input")))
                  (format "%S"
                          (soma-bridge-bench--find-operation
                           (soma-gym-bridge-list-capabilities)
                           "comint.submit_input")))
                 (soma-bridge-bench--check
                  "comint.await_output is a stable enabled terminal operation"
                  (let* ((manifest (soma-gym-bridge-list-capabilities))
                         (entry (soma-bridge-bench--find-operation
                                 manifest "comint.await_output")))
                    (and entry
                         (string= (soma-bridge-bench--get entry "profile")
                                  "terminal-v0")
                         (string= (soma-bridge-bench--get entry "maturity")
                                  "stable")
                         (eq (soma-bridge-bench--get entry "implemented")
                             t)
                         (eq (soma-bridge-bench--get entry "enabled") t)))
                  (format "%S"
                          (soma-bridge-bench--find-operation
                           (soma-gym-bridge-list-capabilities)
                           "comint.await_output")))
                 (soma-bridge-bench--check
                  "initial Comint observation targets a live process"
                  (and (string= (soma-bridge-bench--get initial-response
                                                        "state")
                                "succeeded")
                       (eq (soma-bridge-bench--get initial-result
                                                   "process_live")
                           t)
                       (integerp (soma-bridge-bench--get initial-result
                                                          "process_id"))
                       (integerp (soma-bridge-bench--get initial-result
                                                          "process_mark")))
                  (format "%S" initial-response))
	                 (soma-bridge-bench--check
	                  "comint.observe publishes typed process and input-boundary sections"
	                  (let* ((evidence (soma-bridge-bench--get
	                                    initial-response "evidence"))
	                         (observations (soma-bridge-bench--get
	                                        initial-response
	                                        "observations"))
	                         (effects (soma-bridge-bench--get
	                                   initial-response "effects"))
	                         (process-identity (soma-bridge-bench--get
	                                            evidence "process_identity"))
	                         (input-boundary (soma-bridge-bench--get
	                                          evidence "input_boundary"))
	                         (runtime (soma-bridge-bench--get
	                                   evidence "rootfs_runtime"))
	                         (state (soma-bridge-bench--get
	                                 observations "comint_state")))
	                    (and (string= (soma-bridge-bench--get
	                                    process-identity "process_name")
	                                   "soma-bridge-comint-api")
	                         (eq (soma-bridge-bench--get
	                              process-identity "process_live")
	                             t)
	                         (integerp (soma-bridge-bench--get
	                                    input-boundary "process_mark"))
	                         (string= (soma-bridge-bench--get
	                                   runtime "run_id")
	                                  (or (getenv "SOMA_INSULA_RUN_ID") ""))
	                         (string= (soma-bridge-bench--get
	                                   state "buffer_tail")
	                                  "$ ")
	                         (and (hash-table-p effects)
	                              (= (hash-table-count effects) 0))))
	                  (format "%S" initial-response))
                 (soma-bridge-bench--check
                  "comint.submit_input succeeds through the v0 envelope"
                  (and (string= (soma-bridge-bench--get submit-response
                                                        "api_version")
                                "soma-emacs-agent-v0")
                       (string= (soma-bridge-bench--get submit-response
                                                        "state")
                                "succeeded")
                       (equal (soma-bridge-bench--get submit-response
                                                      "state_sequence")
                              '("accepted" "running" "succeeded")))
                  (format "%S" submit-response))
                 (soma-bridge-bench--check
                  "submit result uses comint-send-input and records before/after state"
                  (and (listp submit-result)
                       (string= (soma-bridge-bench--get submit-result
                                                        "route")
                                "comint-send-input")
                       (listp (soma-bridge-bench--get submit-result
                                                      "before"))
                       (listp (soma-bridge-bench--get submit-result
                                                      "after"))
                       (eq (soma-bridge-bench--get submit-result
                                                   "used_raw_process_write")
                           :json-false))
                  (format "%S" submit-result))
                 (soma-bridge-bench--check
                  "comint.submit_input publishes typed terminal evidence and effects"
                  (let* ((evidence (soma-bridge-bench--get
                                    submit-response "evidence"))
                         (observations (soma-bridge-bench--get
                                        submit-response "observations"))
                         (effects (soma-bridge-bench--get
                                   submit-response "effects"))
                         (process-input
                          (and (listp effects)
                               (soma-bridge-bench--get
                                effects "process_input"))))
                    (and (listp evidence)
                         (equal (soma-bridge-bench--get
                                 evidence "submitted_bytes")
                                (soma-bridge-bench--get
                                 submit-result "submitted_bytes"))
                         (listp (soma-bridge-bench--get
                                 evidence "comint_input_markers"))
                         (stringp (soma-bridge-bench--get
                                   evidence "process_output"))
                         (listp observations)
                         (listp (soma-bridge-bench--get
                                 observations "before"))
                         (listp (soma-bridge-bench--get
                                 observations "after"))
                         (listp process-input)
                         (equal (soma-bridge-bench--get
                                 process-input "submitted_bytes")
                                (soma-bridge-bench--get
                                 submit-result "submitted_bytes"))
                         (string= (soma-bridge-bench--get
                                   process-input "route")
                                  "comint-send-input")
                         (eq (soma-bridge-bench--get
                              process-input "used_raw_process_write")
                             :json-false)))
                  (soma-bridge-bench--alist
                   (cons "evidence"
                         (soma-bridge-bench--get
                          submit-response "evidence"))
                   (cons "observations"
                         (soma-bridge-bench--get
                          submit-response "observations"))
                   (cons "effects"
                         (soma-bridge-bench--get
                          submit-response "effects"))))
                 (soma-bridge-bench--check
                  "comint.await_output succeeds after matching the intended output"
                  (and (string= (soma-bridge-bench--get await-response
                                                        "state")
                                "succeeded")
                       (equal (soma-bridge-bench--get await-response
                                                      "state_sequence")
                              '("accepted" "running" "succeeded"))
                       (eq (soma-bridge-bench--get await-result "matched")
                           t))
                  (format "%S" await-response))
                 (soma-bridge-bench--check
                  "comint.await_output publishes typed output evidence without process effects"
                  (let* ((evidence (soma-bridge-bench--get
                                    await-response "evidence"))
                         (effects (soma-bridge-bench--get
                                   await-response "effects")))
                    (and (listp evidence)
                         (eq (soma-bridge-bench--get evidence "matched")
                             t)
                         (string= (soma-bridge-bench--get
                                   evidence "needle")
                                  done-needle)
                         (stringp (soma-bridge-bench--get
                                   evidence "matched_output"))
                         (stringp (soma-bridge-bench--get
                                   evidence "partial_output"))
                         (numberp (soma-bridge-bench--get
                                   evidence "timeout_seconds"))
                         (not (soma-bridge-bench--get
                               effects "process_input"))
                         (not (soma-bridge-bench--get
                               effects "process_interrupt"))))
                  (soma-bridge-bench--alist
                   (cons "evidence"
                         (soma-bridge-bench--get
                          await-response "evidence"))
                   (cons "effects"
                         (soma-bridge-bench--get
                          await-response "effects"))))
                 (soma-bridge-bench--check
                  "Comint semantic submit produced the expected side effect exactly once"
                  (equal counter-lines (list semantic-label))
                  counter-lines)
                 (soma-bridge-bench--check
                  "Comint process identity is stable across submit and await"
                  (and (equal (soma-bridge-bench--get initial-result
                                                      "process_id")
                              (soma-bridge-bench--get final-observe
                                                     "process_id"))
                       (eq (soma-bridge-bench--get final-observe
                                                   "process_live")
                           t))
                  (soma-bridge-bench--alist
                   (cons "initial" initial-result)
                   (cons "final" final-observe)))
                 (soma-bridge-bench--check
                  "stale process-mark submit is rejected before side effects"
                  (and (string= (soma-bridge-bench--get stale-response
                                                        "state")
                                "failed")
                       (not (member stale-label counter-lines))
                       (not (string-match-p
                             (regexp-quote stale-done-needle)
                             (or (soma-bridge-bench--get final-observe
                                                        "buffer_tail")
                                 ""))))
                  (soma-bridge-bench--alist
                   (cons "stale_response" stale-response)
                   (cons "stale_result" stale-result)
                   (cons "counter_lines" counter-lines)))))
          (soma-bridge-bench--alist
           (cons "initial_request" initial-request)
           (cons "initial_response" initial-response)
           (cons "submit_request" submit-request)
           (cons "submit_response" submit-response)
           (cons "await_request" await-request)
           (cons "await_response" await-response)
           (cons "stale_request" stale-request)
           (cons "stale_response" stale-response)
           (cons "final_observe" final-observe)
           (cons "counter" counter)
           (cons "counter_lines" counter-lines)
           (cons "checks" checks)))
      (soma-bridge-bench--cleanup-comint-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-bridge-bench--scenario-comint-process-ownership-rejected ()
  (soma-gym-bridge-reset-transport)
  (let* ((tmp (make-temp-file "soma-bridge-comint-host-owner-" t))
         (counter (expand-file-name "counter.txt" tmp))
         (host-parent
          (expand-file-name
           ".scratch/emacs-gym/host-shadow-tools/"
           (file-name-as-directory
            (or (locate-dominating-file default-directory ".git")
                default-directory))))
         (host-dir (make-temp-file
                    (expand-file-name "soma-bridge-host-comint-" host-parent)
                    t))
         (host-sh-path (expand-file-name "sh" host-dir))
         (source-sh (or (executable-find "sh")
                        (error "sh executable is required")))
         (label "host-comint")
         (done-needle (format "SOMA_BRIDGE_COMINT_DONE:%s" label))
         cell buffer process initial-request initial-response initial-result
         submit-request submit-response await-request await-response
         before-interrupt interrupt-request interrupt-response final-observe
         final-tail counter-lines checks)
    (unwind-protect
        (progn
          (make-directory host-parent t)
          (copy-file source-sh host-sh-path t)
          (set-file-modes host-sh-path #o755)
          (setq cell
                (soma-bridge-bench--make-comint-program
                 "soma-bridge-comint-host-owner" host-sh-path))
          (setq buffer (car cell))
          (setq process (cdr cell))
          (setq initial-request
                (soma-bridge-bench--api-request
                 "comint-host-owner-observe-1" "comint.observe"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer)))))
          (setq initial-response
                (soma-gym-bridge-submit-request initial-request))
          (setq initial-result
                (soma-bridge-bench--get initial-response "result"))
          (setq submit-request
                (soma-bridge-bench--comint-submit-request
                 "comint-host-owner-submit-1" buffer
                 (soma-bridge-bench--comint-command label counter)
                 initial-result))
          (setq submit-response
                (soma-gym-bridge-submit-request submit-request))
          (setq await-request
                (soma-bridge-bench--comint-await-request
                 "comint-host-owner-await-1" buffer done-needle
                 initial-result 0.2))
          (setq await-response
                (soma-gym-bridge-submit-request await-request))
          (setq before-interrupt
                (soma-gym-bridge-observe-comint buffer))
          (setq interrupt-request
                (soma-bridge-bench--comint-interrupt-request
                 "comint-host-owner-interrupt-1" buffer before-interrupt))
          (setq interrupt-response
                (soma-gym-bridge-submit-request interrupt-request))
          (accept-process-output process 0.2)
          (setq final-observe
                (soma-gym-bridge-observe-comint buffer))
          (setq final-tail
                (or (soma-bridge-bench--get final-observe "buffer_tail") ""))
          (setq counter-lines
                (soma-bridge-bench--file-lines counter))
          (setq checks
                (list
                 (soma-bridge-bench--check
                  "host-shadow Comint fixture starts a live process"
                  (and (string= (soma-bridge-bench--get initial-response
                                                        "state")
                                "succeeded")
                       (eq (soma-bridge-bench--get initial-result
                                                   "process_live")
                           t)
                       (integerp (soma-bridge-bench--get initial-result
                                                          "process_id")))
                  initial-response)
                 (soma-bridge-bench--check
                  "host-shadow Comint executable is outside selected rootfs"
                  (let ((process-exe
                         (soma-bridge-bench--get initial-result
                                                 "process_exe")))
                    (and (stringp process-exe)
                         (string-prefix-p host-dir process-exe)
                         (not (soma-gym-bridge--rootfs-owned-tool-path-p
                               process-exe))))
                  initial-result)
                 (soma-bridge-bench--check
                  "comint.submit_input rejects non-rootfs-owned process before input"
                  (and (string= (soma-bridge-bench--get submit-response
                                                        "state")
                                "failed")
                       (soma-bridge-bench--precondition-check-named
                        submit-response
                        "Comint process executable is rootfs-owned"))
                  submit-response)
                 (soma-bridge-bench--check
                  "comint.await_output rejects non-rootfs-owned process before waiting"
                  (and (string= (soma-bridge-bench--get await-response
                                                        "state")
                                "failed")
                       (soma-bridge-bench--precondition-check-named
                        await-response
                        "Comint process executable is rootfs-owned"))
                  await-response)
                 (soma-bridge-bench--check
                  "comint.interrupt rejects non-rootfs-owned process before control"
                  (and (string= (soma-bridge-bench--get interrupt-response
                                                        "state")
                                "failed")
                       (soma-bridge-bench--precondition-check-named
                        interrupt-response
                        "Comint process executable is rootfs-owned"))
                  interrupt-response)
                 (soma-bridge-bench--check
                  "host-shadow Comint rejection produces no submitted side effect"
                  (and (not counter-lines)
                       (not (string-match-p
                             (regexp-quote done-needle)
                             final-tail)))
                  (soma-bridge-bench--alist
                   (cons "counter_lines" counter-lines)
                   (cons "tail" final-tail)))))
          (soma-bridge-bench--alist
           (cons "host_sh_path" host-sh-path)
           (cons "initial_request" initial-request)
           (cons "initial_response" initial-response)
           (cons "submit_request" submit-request)
           (cons "submit_response" submit-response)
           (cons "await_request" await-request)
           (cons "await_response" await-response)
           (cons "interrupt_request" interrupt-request)
           (cons "interrupt_response" interrupt-response)
           (cons "final_observe" final-observe)
           (cons "counter_lines" counter-lines)
           (cons "checks" checks)))
      (when cell
        (soma-bridge-bench--cleanup-comint-shell cell))
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-bridge-bench--scenario-comint-interrupt-api ()
  (soma-gym-bridge-reset-transport)
  (let* ((tmp (make-temp-file "soma-bridge-comint-interrupt-api-" t))
         (counter (expand-file-name "counter.txt" tmp))
         (cell (soma-bridge-bench--make-comint-shell
                "soma-bridge-comint-interrupt-api"))
         (buffer (car cell))
         (process (cdr cell))
         (start-needle "SOMA_BRIDGE_COMINT_INTERRUPT_START")
         (done-needle "SOMA_BRIDGE_COMINT_INTERRUPT_DONE")
         initial-request initial-response initial-result submit-request
         submit-response submit-result submit-after await-request
         await-response await-result stale-interrupt-request
         stale-interrupt-response before-interrupt fresh-interrupt-request
         fresh-interrupt-response fresh-interrupt-result final-observe
         final-tail counter-lines checks)
    (unwind-protect
        (progn
          (setq initial-request
                (soma-bridge-bench--api-request
                 "comint-interrupt-api-observe-1" "comint.observe"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name buffer)))))
          (setq initial-response
                (soma-gym-bridge-submit-request initial-request))
          (setq initial-result
                (soma-bridge-bench--get initial-response "result"))
          (setq submit-request
                (soma-bridge-bench--comint-submit-request
                 "comint-interrupt-api-submit-1" buffer
                 (soma-bridge-bench--comint-interrupt-command counter)
                 initial-result))
          (setq submit-response
                (soma-gym-bridge-submit-request submit-request))
          (setq submit-result
                (soma-bridge-bench--get submit-response "result"))
          (setq submit-after
                (or (and (listp submit-result)
                         (soma-bridge-bench--get submit-result "after"))
                    initial-result))
          (setq await-request
                (soma-bridge-bench--comint-await-request
                 "comint-interrupt-api-await-start-1" buffer start-needle
                 submit-after 2.0))
          (setq await-response
                (soma-gym-bridge-submit-request await-request))
          (setq await-result
                (soma-bridge-bench--get await-response "result"))
          (setq stale-interrupt-request
                (soma-bridge-bench--comint-interrupt-request
                 "comint-interrupt-api-stale-1" buffer initial-result))
          (setq stale-interrupt-response
                (soma-gym-bridge-submit-request stale-interrupt-request))
          (setq before-interrupt (soma-gym-bridge-observe-comint buffer))
          (setq fresh-interrupt-request
                (soma-bridge-bench--comint-interrupt-request
                 "comint-interrupt-api-fresh-1" buffer before-interrupt))
          (setq fresh-interrupt-response
                (soma-gym-bridge-submit-request fresh-interrupt-request))
          (setq fresh-interrupt-result
                (soma-bridge-bench--get fresh-interrupt-response "result"))
          (let ((deadline (+ (float-time) 2.5)))
            (while (< (float-time) deadline)
              (accept-process-output process 0.05)))
          (setq final-observe
                (soma-gym-bridge-observe-comint buffer))
          (setq final-tail
                (or (soma-bridge-bench--get final-observe "buffer_tail") ""))
          (setq counter-lines
                (soma-bridge-bench--file-lines counter))
          (setq checks
                (list
                 (soma-bridge-bench--check
                  "comint.interrupt is a stable enabled terminal operation"
                  (let* ((manifest (soma-gym-bridge-list-capabilities))
                         (entry (soma-bridge-bench--find-operation
                                 manifest "comint.interrupt")))
                    (and entry
                         (string= (soma-bridge-bench--get entry "profile")
                                  "terminal-v0")
                         (string= (soma-bridge-bench--get entry "maturity")
                                  "stable")
                         (eq (soma-bridge-bench--get entry "implemented")
                             t)
                         (eq (soma-bridge-bench--get entry "enabled") t)
                         (string= (soma-bridge-bench--get entry "route")
                                  "comint-interrupt-subjob")))
                  (format "%S"
                          (soma-bridge-bench--find-operation
                           (soma-gym-bridge-list-capabilities)
                           "comint.interrupt")))
                 (soma-bridge-bench--check
                  "interrupt fixture targets a live Comint process"
                  (and (string= (soma-bridge-bench--get initial-response
                                                        "state")
                                "succeeded")
                       (eq (soma-bridge-bench--get initial-result
                                                   "process_live")
                           t)
                       (integerp (soma-bridge-bench--get initial-result
                                                          "process_id"))
                       (integerp (soma-bridge-bench--get initial-result
                                                          "process_mark")))
                  (format "%S" initial-response))
                 (soma-bridge-bench--check
                  "interrupt fixture command was submitted through Comint"
                  (and (string= (soma-bridge-bench--get submit-response
                                                        "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get submit-result "route")
                                "comint-send-input"))
                  (format "%S" submit-response))
                 (soma-bridge-bench--check
                  "interrupt fixture reached the delayed command before interrupt"
                  (and (string= (soma-bridge-bench--get await-response
                                                        "state")
                                "succeeded")
                       (eq (soma-bridge-bench--get await-result "matched")
                           t)
                       (string-match-p
                        (regexp-quote start-needle)
                        (or (soma-bridge-bench--get await-result
                                                    "matched_output")
                            "")))
                  (format "%S" await-response))
                 (soma-bridge-bench--check
                  "stale process-mark interrupt is rejected before side effects"
                  (string= (soma-bridge-bench--get stale-interrupt-response
                                                    "state")
                           "failed")
                  (format "%S" stale-interrupt-response))
                 (soma-bridge-bench--check
                  "fresh interrupt request succeeds through the v0 envelope"
                  (and (string= (soma-bridge-bench--get fresh-interrupt-response
                                                        "api_version")
                                "soma-emacs-agent-v0")
                       (string= (soma-bridge-bench--get fresh-interrupt-response
                                                        "state")
                                "succeeded")
                       (equal (soma-bridge-bench--get fresh-interrupt-response
                                                      "state_sequence")
                              '("accepted" "running" "succeeded")))
                  (format "%S" fresh-interrupt-response))
                 (soma-bridge-bench--check
                  "interrupt result uses comint-interrupt-subjob and records before/after state"
                  (and (listp fresh-interrupt-result)
                       (string= (soma-bridge-bench--get fresh-interrupt-result
                                                        "route")
                                "comint-interrupt-subjob")
                       (listp (soma-bridge-bench--get fresh-interrupt-result
                                                      "before"))
                       (listp (soma-bridge-bench--get fresh-interrupt-result
                                                      "after"))
                       (eq (soma-bridge-bench--get fresh-interrupt-result
                                                   "used_raw_process_write")
                           :json-false)
                       (eq (soma-bridge-bench--get fresh-interrupt-result
                                                   "bridge_used_raw_process_control")
                           :json-false))
                  (format "%S" fresh-interrupt-result))
                 (soma-bridge-bench--check
                  "comint.interrupt publishes typed terminal evidence and effects"
                  (let* ((evidence (soma-bridge-bench--get
                                    fresh-interrupt-response "evidence"))
                         (effects (soma-bridge-bench--get
                                   fresh-interrupt-response "effects"))
                         (process-identity
                          (and (listp evidence)
                               (soma-bridge-bench--get
                                evidence "process_identity")))
                         (process-survival
                          (and (listp evidence)
                               (soma-bridge-bench--get
                                evidence "process_survival")))
                         (process-interrupt
                          (and (listp effects)
                               (soma-bridge-bench--get
                                effects "process_interrupt"))))
                    (and (listp evidence)
                         (string= (soma-bridge-bench--get
                                   evidence "interrupt_command")
                                  "comint-interrupt-subjob")
                         (listp process-identity)
                         (integerp (soma-bridge-bench--get
                                    process-identity "process_id"))
                         (string= (soma-bridge-bench--get
                                   process-identity "status_before")
                                  "run")
                         (listp process-survival)
                         (eq (soma-bridge-bench--get
                              process-survival "process_live_after")
                             t)
                         (string= (soma-bridge-bench--get
                                   evidence "side_effect_reconciliation")
                                  "external_verifier_required")
                         (listp process-interrupt)
                         (string= (soma-bridge-bench--get
                                   process-interrupt "command")
                                  "comint-interrupt-subjob")
                         (eq (soma-bridge-bench--get
                              process-interrupt "dispatched")
                             t)
                         (eq (soma-bridge-bench--get
                              process-interrupt "used_raw_process_write")
                             :json-false)
                         (eq (soma-bridge-bench--get
                              process-interrupt
                              "bridge_used_raw_process_control")
                             :json-false)))
                  (soma-bridge-bench--alist
                   (cons "evidence"
                         (soma-bridge-bench--get
                          fresh-interrupt-response "evidence"))
                   (cons "effects"
                         (soma-bridge-bench--get
                          fresh-interrupt-response "effects"))))
                 (soma-bridge-bench--check
                  "interrupted Comint shell process survives"
                  (and (equal (soma-bridge-bench--get initial-result
                                                      "process_id")
                              (soma-bridge-bench--get final-observe
                                                     "process_id"))
                       (eq (soma-bridge-bench--get final-observe
                                                   "process_live")
                           t))
                  (soma-bridge-bench--alist
                   (cons "initial" initial-result)
                   (cons "final" final-observe)))
                 (soma-bridge-bench--check
                  "interrupt command did not reach completion marker"
                  (not (soma-bridge-bench--has-exact-line-p
                        final-tail done-needle))
                  final-tail)
                 (soma-bridge-bench--check
                  "interrupt side effect did not occur"
                  (not counter-lines)
                  counter-lines)))
          (soma-bridge-bench--alist
           (cons "initial_request" initial-request)
           (cons "initial_response" initial-response)
           (cons "submit_request" submit-request)
           (cons "submit_response" submit-response)
           (cons "await_request" await-request)
           (cons "await_response" await-response)
           (cons "stale_interrupt_request" stale-interrupt-request)
           (cons "stale_interrupt_response" stale-interrupt-response)
           (cons "fresh_interrupt_request" fresh-interrupt-request)
           (cons "fresh_interrupt_response" fresh-interrupt-response)
           (cons "final_observe" final-observe)
           (cons "counter" counter)
           (cons "counter_lines" counter-lines)
           (cons "checks" checks)))
      (soma-bridge-bench--cleanup-comint-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-bridge-bench--scenario-magit-semantic-api ()
  (soma-gym-bridge-reset-transport)
  (soma-bridge-bench--load-magit)
  (let ((hunk-repo (soma-bridge-bench--make-magit-repo "hunk-api"))
        (file-repo (soma-bridge-bench--make-magit-repo "file-api"))
        (stale-repo (soma-bridge-bench--make-magit-repo "stale-api")))
    (unwind-protect
        (let (manifest hunk-entry file-entry hunk-buffer hunk-object
                       hunk-observe-request hunk-observe-response
                       hunk-observe hunk-section hunk-preconditions
                       hunk-stage-request hunk-stage-response
                       hunk-stage-result hunk-capture hunk-state
                       file-buffer file-object file-observe-request
                       file-observe-response file-observe file-section
                       file-preconditions file-stage-request
                       file-stage-response file-stage-result file-capture
                       file-state stale-buffer stale-object
                       stale-observe-request stale-observe-response
                       stale-observe stale-section stale-preconditions
                       stale-stage-request stale-stage-response
                       stale-stage-result stale-state checks)
          (setq manifest (soma-gym-bridge-list-capabilities))

          (soma-bridge-bench--setup-magit-target-two-hunks hunk-repo)
          (setq hunk-buffer
                (soma-bridge-bench--magit-status-buffer hunk-repo))
          (with-current-buffer hunk-buffer
            (setq hunk-object
                  (soma-bridge-bench--magit-find-hunk-section
                   hunk-buffer "target.txt" "target-05-selected"))
            (unless hunk-object
              (error "could not locate semantic Magit target hunk"))
            (goto-char
             (soma-bridge-bench--magit-section-target-point
              hunk-object "target-05-selected"))
            (set-window-point (selected-window) (point)))
          (setq hunk-observe-request
                (soma-bridge-bench--api-request
                 "magit-api-hunk-observe-1" "magit.observe_sections"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name hunk-buffer)))))
          (setq hunk-observe-response
                (soma-gym-bridge-submit-request hunk-observe-request))
          (setq hunk-observe
                (soma-bridge-bench--get hunk-observe-response "result"))
          (setq hunk-section
                (soma-bridge-bench--find-magit-observed-section
                 hunk-observe "hunk" "target.txt" "target-05-selected"))
          (setq hunk-preconditions
                (soma-bridge-bench--magit-stage-preconditions
                 hunk-observe hunk-section))
          (setq hunk-stage-request
                (soma-bridge-bench--magit-stage-request
                 "magit-api-stage-hunk-1" "magit.stage_hunk"
                 hunk-buffer hunk-section hunk-preconditions))
          (setq hunk-capture
                (soma-bridge-bench--with-magit-capture
                 (lambda ()
                   (setq hunk-stage-response
                         (soma-gym-bridge-submit-request
                          hunk-stage-request)))))
          (setq hunk-stage-result
                (soma-bridge-bench--get hunk-stage-response "result"))
          (setq hunk-state
                (soma-bridge-bench--magit-repo-state hunk-repo))

          (soma-bridge-bench--setup-magit-target-one-hunk file-repo)
          (setq file-buffer
                (soma-bridge-bench--magit-status-buffer file-repo))
          (with-current-buffer file-buffer
            (setq file-object
                  (soma-bridge-bench--magit-find-file-section
                   file-buffer "target.txt"))
            (unless file-object
              (error "could not locate semantic Magit target file section"))
            (goto-char
             (soma-bridge-bench--magit-section-slot file-object 'start))
            (set-window-point (selected-window) (point)))
          (setq file-observe-request
                (soma-bridge-bench--api-request
                 "magit-api-file-observe-1" "magit.observe_sections"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name file-buffer)))))
          (setq file-observe-response
                (soma-gym-bridge-submit-request file-observe-request))
          (setq file-observe
                (soma-bridge-bench--get file-observe-response "result"))
          (setq file-section
                (soma-bridge-bench--find-magit-observed-section
                 file-observe "file" "target.txt" nil))
          (setq file-preconditions
                (soma-bridge-bench--magit-stage-preconditions
                 file-observe file-section))
          (setq file-stage-request
                (soma-bridge-bench--magit-stage-request
                 "magit-api-stage-file-1" "magit.stage_file"
                 file-buffer file-section file-preconditions))
          (setq file-capture
                (soma-bridge-bench--with-magit-capture
                 (lambda ()
                   (setq file-stage-response
                         (soma-gym-bridge-submit-request
                          file-stage-request)))))
          (setq file-stage-result
                (soma-bridge-bench--get file-stage-response "result"))
          (setq file-state
                (soma-bridge-bench--magit-repo-state file-repo))

          (soma-bridge-bench--setup-magit-target-one-hunk stale-repo)
          (setq stale-buffer
                (soma-bridge-bench--magit-status-buffer stale-repo))
          (with-current-buffer stale-buffer
            (setq stale-object
                  (soma-bridge-bench--magit-find-hunk-section
                   stale-buffer "target.txt" "target-05-selected"))
            (unless stale-object
              (error "could not locate stale semantic Magit hunk"))
            (goto-char
             (soma-bridge-bench--magit-section-target-point
              stale-object "target-05-selected"))
            (set-window-point (selected-window) (point)))
          (setq stale-observe-request
                (soma-bridge-bench--api-request
                 "magit-api-stale-observe-1" "magit.observe_sections"
                 (soma-bridge-bench--alist
                  (cons "buffer_name" (buffer-name stale-buffer)))))
          (setq stale-observe-response
                (soma-gym-bridge-submit-request stale-observe-request))
          (setq stale-observe
                (soma-bridge-bench--get stale-observe-response "result"))
          (setq stale-section
                (soma-bridge-bench--find-magit-observed-section
                 stale-observe "hunk" "target.txt" "target-05-selected"))
          (setq stale-preconditions
                (soma-bridge-bench--magit-stage-preconditions
                 stale-observe stale-section))
          (soma-bridge-bench--magit-stale-external-edit stale-repo)
          (setq stale-stage-request
                (soma-bridge-bench--magit-stage-request
                 "magit-api-stage-stale-hunk-1" "magit.stage_hunk"
                 stale-buffer stale-section stale-preconditions))
          (setq stale-stage-response
                (soma-gym-bridge-submit-request stale-stage-request))
          (setq stale-stage-result
                (soma-bridge-bench--get stale-stage-response "result"))
          (setq stale-state
                (soma-bridge-bench--magit-repo-state stale-repo))

          (setq hunk-entry
                (soma-bridge-bench--find-operation manifest
                                                   "magit.stage_hunk"))
          (setq file-entry
                (soma-bridge-bench--find-operation manifest
                                                   "magit.stage_file"))
          (setq checks
                (list
                 (soma-bridge-bench--check
                  "magit.stage_hunk is a stable enabled vcs-files operation"
                  (and hunk-entry
                       (string= (soma-bridge-bench--get hunk-entry "profile")
                                "vcs-files-v0")
                       (string= (soma-bridge-bench--get hunk-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get hunk-entry "implemented")
                           t)
                       (eq (soma-bridge-bench--get hunk-entry "enabled") t)
                       (string= (soma-bridge-bench--get hunk-entry "route")
                                "magit-stage"))
                  (format "%S" hunk-entry))
                 (soma-bridge-bench--check
                  "magit.stage_file is a stable enabled vcs-files operation"
                  (and file-entry
                       (string= (soma-bridge-bench--get file-entry "profile")
                                "vcs-files-v0")
                       (string= (soma-bridge-bench--get file-entry "maturity")
                                "stable")
                       (eq (soma-bridge-bench--get file-entry "implemented")
                           t)
                       (eq (soma-bridge-bench--get file-entry "enabled") t)
                       (string= (soma-bridge-bench--get file-entry "route")
                                "magit-stage"))
                  (format "%S" file-entry))
	                 (soma-bridge-bench--check
	                  "magit.observe_sections publishes typed repo, generation, selected-section, and section-list state"
	                  (let* ((evidence (soma-bridge-bench--get
	                                    hunk-observe-response "evidence"))
	                         (observations (soma-bridge-bench--get
	                                        hunk-observe-response
	                                        "observations"))
	                         (effects (soma-bridge-bench--get
	                                   hunk-observe-response "effects"))
	                         (repo (soma-bridge-bench--get
	                                evidence "repo_fingerprint"))
	                         (generation (soma-bridge-bench--get
	                                      evidence "status_generation"))
	                         (selected (soma-bridge-bench--get
	                                    evidence "selected_section"))
	                         (sections (soma-bridge-bench--get
	                                    observations "sections")))
	                    (and (string= (soma-bridge-bench--get
	                                    repo "magit_repo")
	                                   (file-name-as-directory
	                                    (file-truename hunk-repo)))
	                         (string= (soma-bridge-bench--get
	                                   generation "magit_status_hash")
	                                  (soma-bridge-bench--get
	                                   hunk-observe
	                                   "magit_status_hash"))
	                         (string= (soma-bridge-bench--get
	                                   selected "type")
	                                  "hunk")
	                         (= (or (soma-bridge-bench--get
	                                 sections "sections_returned")
	                                0)
	                            (length (soma-bridge-bench--get
	                                     hunk-observe "sections")))
	                         (and (hash-table-p effects)
	                              (= (hash-table-count effects) 0))))
	                  (format "%S" hunk-observe-response))
                 (soma-bridge-bench--check
                  "magit.stage_hunk and magit.stage_file publish typed Git index effects"
                  (let* ((hunk-evidence
                          (soma-bridge-bench--get
                           hunk-stage-response "evidence"))
                         (hunk-effects
                          (soma-bridge-bench--get
                           hunk-stage-response "effects"))
                         (hunk-dispatch
                          (soma-bridge-bench--get
                           hunk-evidence "command_dispatch"))
                         (hunk-index-effect
                          (soma-bridge-bench--get
                           hunk-effects "git_index_update"))
                         (file-evidence
                          (soma-bridge-bench--get
                           file-stage-response "evidence"))
                         (file-effects
                          (soma-bridge-bench--get
                           file-stage-response "effects"))
                         (file-dispatch
                          (soma-bridge-bench--get
                           file-evidence "command_dispatch"))
                         (file-index-effect
                          (soma-bridge-bench--get
                           file-effects "git_index_update")))
                    (and
                     (soma-bridge-bench--object-p hunk-dispatch)
                     (string= (soma-bridge-bench--get
                               hunk-dispatch "this_command")
                              "magit-stage")
                     (soma-bridge-bench--object-p hunk-index-effect)
                     (string= (soma-bridge-bench--get
                               hunk-index-effect "route")
                              "magit-stage")
                     (string= (soma-bridge-bench--get
                               hunk-index-effect "section_type")
                              "hunk")
                     (soma-bridge-bench--object-p
                      (soma-bridge-bench--get
                       hunk-index-effect "after_repo_fingerprint"))
                     (soma-bridge-bench--object-p file-dispatch)
                     (string= (soma-bridge-bench--get
                               file-dispatch "this_command")
                              "magit-stage")
                     (soma-bridge-bench--object-p file-index-effect)
                     (string= (soma-bridge-bench--get
                               file-index-effect "route")
                              "magit-stage")
                     (string= (soma-bridge-bench--get
                               file-index-effect "section_type")
                              "file")
                     (soma-bridge-bench--object-p
                      (soma-bridge-bench--get
                       file-index-effect "after_repo_fingerprint"))))
                  (format "%S"
                          (list hunk-stage-response
                                file-stage-response)))
                 (soma-bridge-bench--check
                  "magit.stage_hunk succeeds through the v0 envelope"
                  (and (string= (soma-bridge-bench--get hunk-stage-response
                                                        "api_version")
                                "soma-emacs-agent-v0")
                       (string= (soma-bridge-bench--get hunk-stage-response
                                                        "state")
                                "succeeded")
                       (equal (soma-bridge-bench--get hunk-stage-response
                                                      "state_sequence")
                              '("accepted" "running" "succeeded")))
                  (format "%S" hunk-stage-response))
                 (soma-bridge-bench--check
                  "magit.stage_hunk uses Magit command context"
                  (and (listp hunk-stage-result)
                       (string= (soma-bridge-bench--get hunk-stage-result
                                                        "route")
                                "magit-stage")
                       (string= (soma-bridge-bench--get hunk-stage-result
                                                        "this_command")
                                "magit-stage")
                       (member "magit-stage"
                               (soma-bridge-bench--get
                                hunk-capture "post_stage_hook_runs")))
                  (soma-bridge-bench--alist
                   (cons "result" hunk-stage-result)
                   (cons "capture" hunk-capture)))
                 (soma-bridge-bench--check
                  "magit.stage_hunk stages only the intended target hunk"
                  (and (equal (soma-bridge-bench--get hunk-state
                                                      "cached_names")
                              '("staged.txt" "target.txt"))
                       (equal (soma-bridge-bench--get hunk-state
                                                      "unstaged_names")
                              '("debug.txt" "target.txt"))
                       (soma-bridge-bench--string-contains-p
                        (soma-bridge-bench--get hunk-state "target_index")
                        "target-05-selected")
                       (soma-bridge-bench--string-contains-p
                        (soma-bridge-bench--get hunk-state "target_index")
                        "target-25-base")
                       (soma-bridge-bench--string-contains-p
                        (soma-bridge-bench--get hunk-state "target_worktree")
                        "target-25-other")
                       (string= (soma-bridge-bench--get hunk-state
                                                        "debug_index")
                                "debug-base\n")
                       (string= (soma-bridge-bench--get hunk-state
                                                        "staged_index")
                                "staged-expected\n"))
                  hunk-state)
                 (soma-bridge-bench--check
                  "magit.stage_file stages the selected file section"
                  (and (string= (soma-bridge-bench--get file-stage-response
                                                        "state")
                                "succeeded")
                       (string= (soma-bridge-bench--get file-stage-result
                                                        "this_command")
                                "magit-stage")
                       (member "magit-stage"
                               (soma-bridge-bench--get
                                file-capture "post_stage_hook_runs"))
                       (equal (soma-bridge-bench--get file-state
                                                      "cached_names")
                              '("staged.txt" "target.txt"))
                       (equal (soma-bridge-bench--get file-state
                                                      "unstaged_names")
                              '("debug.txt"))
                       (soma-bridge-bench--string-contains-p
                        (soma-bridge-bench--get file-state "target_index")
                        "target-05-selected")
                       (not (member "target.txt"
                                    (soma-bridge-bench--get
                                     file-state "unstaged_names"))))
                  (soma-bridge-bench--alist
                   (cons "response" file-stage-response)
                   (cons "result" file-stage-result)
                   (cons "state" file-state)
                   (cons "capture" file-capture)))
                 (soma-bridge-bench--check
                  "stale Magit hunk request is rejected before staging"
                  (and (string= (soma-bridge-bench--get stale-stage-response
                                                        "state")
                                "failed")
                       (equal (soma-bridge-bench--get stale-state
                                                      "cached_names")
                              '("staged.txt"))
                       (not (soma-bridge-bench--string-contains-p
                             (soma-bridge-bench--get stale-state
                                                      "target_index")
                             "target-05-selected"))
                       (soma-bridge-bench--string-contains-p
                        (soma-bridge-bench--get stale-state
                                                 "target_worktree")
                        "target-25-external"))
                  (soma-bridge-bench--alist
                   (cons "response" stale-stage-response)
                   (cons "result" stale-stage-result)
                   (cons "state" stale-state)))))
          (soma-bridge-bench--alist
           (cons "hunk_observe_request" hunk-observe-request)
           (cons "hunk_observe_response" hunk-observe-response)
           (cons "hunk_stage_request" hunk-stage-request)
           (cons "hunk_stage_response" hunk-stage-response)
           (cons "hunk_capture" hunk-capture)
           (cons "hunk_state" hunk-state)
           (cons "file_observe_request" file-observe-request)
           (cons "file_observe_response" file-observe-response)
           (cons "file_stage_request" file-stage-request)
           (cons "file_stage_response" file-stage-response)
           (cons "file_capture" file-capture)
           (cons "file_state" file-state)
           (cons "stale_observe_request" stale-observe-request)
           (cons "stale_observe_response" stale-observe-response)
           (cons "stale_stage_request" stale-stage-request)
           (cons "stale_stage_response" stale-stage-response)
           (cons "stale_state" stale-state)
           (cons "checks" checks)))
      (soma-bridge-bench--cleanup-magit-repo hunk-repo)
      (soma-bridge-bench--cleanup-magit-repo file-repo)
      (soma-bridge-bench--cleanup-magit-repo stale-repo))))

(defun soma-bridge-bench--record-scenario (index id title expected-pass thunk)
  (let* ((result (funcall thunk))
         (checks (soma-bridge-bench--get result "checks"))
         (policy-pass (soma-bridge-bench--all-checks-pass-p checks))
         (as-expected (if expected-pass policy-pass (not policy-pass)))
         (record
          (append
           (soma-bridge-bench--alist
            (cons "id" id)
            (cons "title" title)
            (cons "expected_policy_pass" (soma-bridge-bench--bool expected-pass))
            (cons "policy_passed_grader" (soma-bridge-bench--bool policy-pass))
            (cons "as_expected" (soma-bridge-bench--bool as-expected)))
           result)))
    (push record soma-bridge-benchmark-results)
    (soma-bridge-bench--write-json
     (expand-file-name (format "%02d-%s.json" index id)
                       soma-bridge-benchmark-output-dir)
     record)
    record))

(defun soma-bridge-bench--insula-checks ()
  (let* ((plan (soma-bridge-bench--plan))
         (mode (or (getenv "SOMA_INSULA_MODE") ""))
         (plan-mode (or (soma-bridge-bench--get plan "mode") ""))
         (closure (or (soma-bridge-bench--get plan "dependency_closure") ""))
         (glibc-owner (or (soma-bridge-bench--get plan "glibc_owner") ""))
         (benchmark (or (soma-bridge-bench--get plan "benchmark") "")))
    (list
     (soma-bridge-bench--check
      "SOMA_INSULA_ACTIVE is set"
      (string= (or (getenv "SOMA_INSULA_ACTIVE") "") "1")
      (or (getenv "SOMA_INSULA_ACTIVE") ""))
     (soma-bridge-bench--check
      "runtime mode is selected rootfs"
      (and (string= mode "selected-container-rootfs")
           (string= plan-mode "selected-container-rootfs-insula"))
      (format "env=%S plan=%S" mode plan-mode))
     (soma-bridge-bench--check
      "benchmark plan is for bridge transport"
      (string= benchmark "bridge-transport")
      benchmark)
     (soma-bridge-bench--check
      "dependency closure is rootfs-owned"
      (string= closure "rootfs_owned")
      closure)
     (soma-bridge-bench--check
      "glibc owner is container image rootfs"
      (string= glibc-owner "container-image-rootfs")
      glibc-owner)
     (soma-bridge-bench--check
      "Emacs executable is source-built rootfs path"
      (string= (soma-bridge-bench--get plan "emacs")
               "/opt/soma/emacs/bin/emacs")
      (or (soma-bridge-bench--get plan "emacs") "")))))

(defun soma-bridge-bench--write-summary ()
  (let* ((records (nreverse soma-bridge-benchmark-results))
         (total (length records))
         (as-expected
          (seq-count
           (lambda (record)
             (eq (soma-bridge-bench--get record "as_expected") t))
           records))
         (insula-checks (soma-bridge-bench--insula-checks))
         (insula-passed (soma-bridge-bench--all-checks-pass-p insula-checks))
         (passed (and (= total as-expected) insula-passed))
         (summary
          (soma-bridge-bench--alist
           (cons "generated_at" (format-time-string "%FT%TZ" nil t))
           (cons "benchmark" "bridge-transport")
           (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
           (cons "total_scenarios" total)
           (cons "scenarios_as_expected" as-expected)
           (cons "insula_checks_passed" (soma-bridge-bench--bool insula-passed))
           (cons "passed" (soma-bridge-bench--bool passed))
           (cons "insula_checks" insula-checks)
           (cons "records" records))))
    (soma-bridge-bench--write-json
     (expand-file-name "summary.json" soma-bridge-benchmark-output-dir)
     summary)
    (with-temp-file (expand-file-name "summary.md"
                                      soma-bridge-benchmark-output-dir)
      (insert "# Bridge Transport Benchmark Summary\n\n")
      (insert (format "- Generated: %s\n"
                      (soma-bridge-bench--get summary "generated_at")))
      (insert "- Benchmark: `bridge-transport`\n")
      (insert (format "- Run id: `%s`\n"
                      (soma-bridge-bench--get summary "run_id")))
      (insert (format "- Total scenarios: %d\n" total))
      (insert (format "- Scenarios as expected: %d\n" as-expected))
      (insert (format "- Insula checks passed: `%S`\n" insula-passed))
      (insert (format "- Passed: `%S`\n\n" passed))
      (cl-loop for record in records
               for first = t then nil
               do
        (unless first
          (insert "\n"))
        (insert (format "## %s\n\n"
                        (soma-bridge-bench--get record "id")))
        (insert (format "- Title: %s\n"
                        (soma-bridge-bench--get record "title")))
        (insert (format "- Expected policy pass: `%S`\n"
                        (soma-bridge-bench--get
                         record "expected_policy_pass")))
        (insert (format "- Policy passed grader: `%S`\n"
                        (soma-bridge-bench--get
                         record "policy_passed_grader")))
        (insert (format "- As expected: `%S`\n"
                        (soma-bridge-bench--get record "as_expected")))))
    (unless passed
      (error "bridge benchmark failed: %S" summary))
    summary))

(defun soma-bridge-bench-run ()
  (let ((args (seq-remove (lambda (arg) (string= arg "--"))
                          command-line-args-left)))
    (setq soma-bridge-benchmark-output-dir
          (file-name-as-directory
           (expand-file-name
            (or (car args)
                "benchmarks/emacs-gym/bridge/traces/latest"))))
    (setq soma-bridge-benchmark-results nil))
  (make-directory soma-bridge-benchmark-output-dir t)
  (soma-bridge-bench--record-scenario
   1
   "envelope-observe"
   "Request envelope observes a buffer and stores terminal status"
   t
   #'soma-bridge-bench--scenario-envelope-observe)
  (soma-bridge-bench--record-scenario
   2
   "api-manifest-introspection"
   "The bridge exposes a machine-readable stable API manifest and operation descriptions"
   t
   #'soma-bridge-bench--scenario-api-manifest-introspection)
  (soma-bridge-bench--record-scenario
   3
   "observe-session"
   "Stable core-v0 session observation anchors Emacs, Doom, selected rootfs, and frame identity"
   t
   #'soma-bridge-bench--scenario-observe-session)
  (soma-bridge-bench--record-scenario
   4
   "command-dispatch"
   "Request envelope executes an allowlisted interactive command through checked command dispatch"
   t
   #'soma-bridge-bench--scenario-command-dispatch)
  (soma-bridge-bench--record-scenario
   5
   "selected-window-command-rejected"
   "A command.dispatch request is rejected when the selected window no longer displays the target buffer"
   t
   #'soma-bridge-bench--scenario-selected-window-command-rejected)
  (soma-bridge-bench--record-scenario
   6
   "buffer-read-purity"
   "A buffer.read request returns text without moving point, mark, revision, or selected window state"
   t
   #'soma-bridge-bench--scenario-buffer-read-purity)
  (soma-bridge-bench--record-scenario
   7
   "checked-edit-save"
   "A checked buffer edit mutates only the expected range and save writes the visited file"
   t
   #'soma-bridge-bench--scenario-checked-edit-save)
  (soma-bridge-bench--record-scenario
   8
   "checked-apply-patch"
   "A buffer.apply_patch request applies multiple exact hunks and rejects stale context"
   t
   #'soma-bridge-bench--scenario-checked-apply-patch)
  (soma-bridge-bench--record-scenario
   9
   "stale-edit-rejected"
   "A buffer.replace_range request with a stale chars_tick is rejected without mutation"
   t
   #'soma-bridge-bench--scenario-stale-edit-rejected)
  (soma-bridge-bench--record-scenario
   10
   "point-edit-rejected"
   "A buffer.replace_range request is rejected when point changed without a text revision"
   t
   #'soma-bridge-bench--scenario-point-edit-rejected)
  (soma-bridge-bench--record-scenario
   11
   "narrowed-edit-rejected"
   "A buffer.replace_range request is rejected when narrowing changed without a text revision"
   t
   #'soma-bridge-bench--scenario-narrowed-edit-rejected)
  (soma-bridge-bench--record-scenario
   12
   "mark-active-edit-rejected"
   "A buffer.replace_range request is rejected when active-region state changed without a text revision"
   t
   #'soma-bridge-bench--scenario-mark-active-edit-rejected)
  (soma-bridge-bench--record-scenario
   13
   "read-only-edit-rejected"
   "A buffer.replace_range request is rejected when read-only state changed without a text revision"
   t
   #'soma-bridge-bench--scenario-read-only-edit-rejected)
  (soma-bridge-bench--record-scenario
   14
   "completion-accept"
   "A completion.accept request inserts a suggestion only when range, revision, point, prefix, and suffix match"
   t
   #'soma-bridge-bench--scenario-completion-accept)
  (soma-bridge-bench--record-scenario
   15
   "stale-completion-rejected"
   "A stale completion.accept request is rejected after intervening human editing"
   t
   #'soma-bridge-bench--scenario-stale-completion-rejected)
  (soma-bridge-bench--record-scenario
   16
   "episode-finish"
   "An episode.finish request succeeds only after required requests and final buffer evidence pass"
   t
   #'soma-bridge-bench--scenario-episode-finish)
  (soma-bridge-bench--record-scenario
   17
   "duplicate-request"
   "A duplicate request id is rejected without replacing the stored request"
   t
   #'soma-bridge-bench--scenario-duplicate-request)
  (soma-bridge-bench--record-scenario
   18
   "direct-call-rejected"
   "Model-facing requests cannot invoke unchecked direct-call operations"
   t
   #'soma-bridge-bench--scenario-direct-call-rejected)
  (soma-bridge-bench--record-scenario
   19
   "socket-jsonl-roundtrip"
   "A JSONL Unix-domain socket request returns a structured bridge response"
   t
   #'soma-bridge-bench--scenario-socket-jsonl-roundtrip)
  (soma-bridge-bench--record-scenario
   20
   "project-api"
   "Stable coding-v0 project operations observe Git project identity and search through rootfs-owned ripgrep"
   t
   #'soma-bridge-bench--scenario-project-api)
  (soma-bridge-bench--record-scenario
   21
   "reports-vcs-files-api"
   "Stable reports-v0 and vcs-files-v0 operations observe Org/Dired state, export HTML, and reject unsafe file collisions"
   t
   #'soma-bridge-bench--scenario-reports-vcs-files-api)
  (soma-bridge-bench--record-scenario
   22
   "comint-semantic-api"
   "Stable terminal-v0 Comint APIs submit input through Comint, await output, and reject stale process-mark ownership"
   t
   #'soma-bridge-bench--scenario-comint-semantic-api)
  (soma-bridge-bench--record-scenario
   23
   "comint-process-ownership-rejected"
   "Stable terminal-v0 Comint operations reject live processes outside the selected rootfs"
   t
   #'soma-bridge-bench--scenario-comint-process-ownership-rejected)
  (soma-bridge-bench--record-scenario
   24
   "comint-interrupt-api"
   "Stable terminal-v0 Comint API interrupts the intended subprocess and reconciles side effects"
   t
   #'soma-bridge-bench--scenario-comint-interrupt-api)
  (soma-bridge-bench--record-scenario
   25
   "magit-semantic-api"
   "Stable vcs-files-v0 Magit APIs stage intended hunks/files through Magit and reject stale sections"
   t
   #'soma-bridge-bench--scenario-magit-semantic-api)
  (soma-bridge-bench--record-scenario
   26
   "lsp-envelope-projection"
   "Stable coding-v0 LSP operations publish typed evidence, observations, and workspace-edit effects"
   t
   #'soma-bridge-bench--scenario-lsp-envelope-projection)
  (soma-bridge-bench--record-scenario
   27
   "lsp-server-ownership-rejected"
   "Stable coding-v0 LSP operations reject live server processes outside the selected rootfs"
   t
   #'soma-bridge-bench--scenario-lsp-server-ownership-rejected)
  (soma-bridge-bench--record-scenario
   28
   "request-envelope-validation"
   "The bridge validates api_version, capability, preconditions, and timeout_ms in the v0 request envelope"
   t
   #'soma-bridge-bench--scenario-request-envelope-validation)
  (soma-bridge-bench--record-scenario
   29
   "async-terminal-publication-preserved"
   "A request that reaches a terminal state during submission is not overwritten by its initial running response"
   t
   #'soma-bridge-bench--scenario-async-terminal-publication-preserved)
  (soma-bridge-bench--record-scenario
   30
   "response-envelope-validation"
   "Captured bridge responses recursively satisfy the v0 result envelope published by the operation manifest"
   t
   #'soma-bridge-bench--scenario-response-envelope-validation)
  (soma-bridge-bench--write-summary)
  (message "bridge-benchmark: wrote results to %s"
           soma-bridge-benchmark-output-dir))

(soma-bridge-bench-run)

;;; run-benchmark.el ends here
