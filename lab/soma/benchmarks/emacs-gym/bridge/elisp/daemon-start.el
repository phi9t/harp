;;; daemon-start.el --- Bootstrap live daemon bridge benchmark -*- lexical-binding: t; -*-

(require 'json)
(require 'cl-lib)
(require 'eieio)
(require 'seq)
(require 'server)
(require 'subr-x)

(add-to-list
 'load-path
 (expand-file-name "../../elisp"
                   (file-name-directory (or load-file-name buffer-file-name))))
(require 'soma-gym-bridge)

(defun soma-bridge-daemon--alist (&rest pairs)
  pairs)

(defun soma-bridge-daemon--bool (value)
  (if value t :json-false))

(defun soma-bridge-daemon--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (make-directory (file-name-directory path) t)
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-bridge-daemon--env-required (name)
  (let ((value (getenv name)))
    (unless (and value (not (string-empty-p value)))
      (error "missing required environment variable: %s" name))
    value))

(defvar magit-auto-revert-mode)
(defvar magit-display-buffer-function)
(defvar magit-display-buffer-noselect)
(defvar magit-post-stage-hook)
(defvar magit-refresh-buffer-hook)
(defvar lsp--buffer-workspaces)
(defvar lsp--session)
(defvar lsp-auto-guess-root)
(defvar lsp-enable-file-watchers)
(defvar lsp-enable-snippet)
(defvar lsp-enable-symbol-highlighting)
(defvar lsp-enabled-clients)
(defvar lsp-diagnostic-clean-after-change)
(defvar lsp-diagnostics-provider)
(defvar lsp-guess-root-without-session)
(defvar lsp-headerline-breadcrumb-enable)
(defvar lsp-lens-enable)
(defvar lsp-log-io)
(defvar lsp-modeline-code-actions-enable)
(defvar lsp-modeline-diagnostics-enable)
(defvar lsp-modeline-workspace-status-enable)
(defvar lsp-pylsp-server-command)
(defvar lsp-response-timeout)
(defvar lsp-restart)
(defvar lsp-session-file)

(defvar soma-bridge-daemon--magit-hook-log nil
  "Path where daemon Magit benchmark hooks append evidence.")

(defvar-local soma-bridge-daemon--magit-buffer-hook-log nil
  "Buffer-local path where a Magit fixture records hook evidence.")

(defvar-local soma-bridge-daemon--comint-command-log nil
  "Buffer-local path where a Comint fixture records command evidence.")

(defvar soma-bridge-daemon--jupyter-fixtures nil
  "Live Jupyter fixtures owned by the daemon benchmark.")

(defvar soma-bridge-daemon--repo-root
  (file-name-as-directory
   (or (locate-dominating-file default-directory ".git")
       default-directory))
  "Repository root used for daemon benchmark scratch fixtures.")

(defvar soma-bridge-daemon--lsp-fixtures nil
  "Live LSP fixtures owned by the daemon benchmark.")

(defvar soma-bridge-daemon--lsp-jsonrpc-transport-suppressed nil
  "Non-nil when daemon LSP fixture uses lsp-mode process-filter transport.")

(defvar soma-bridge-daemon--lsp-diagnostic-events nil
  "Bounded publishDiagnostics events observed by daemon LSP instrumentation.")

(defvar soma-bridge-daemon--lsp-notify-events nil
  "Bounded lsp-notify calls observed by daemon LSP instrumentation.")

(defvar soma-bridge-daemon--lsp-incoming-events nil
  "Bounded incoming LSP notifications observed by daemon LSP instrumentation.")

(defvar soma-bridge-daemon--lsp-diagnostic-advice-installed nil
  "Non-nil once daemon LSP diagnostic advice has been installed.")

(declare-function dired-flag-file-deletion "dired")
(declare-function dired-goto-file "dired")
(declare-function dired-mark "dired")
(declare-function dired-noselect "dired")
(declare-function org-mode "org")

(defun soma-bridge-daemon--lsp-diagnostic-count (diagnostics)
  (cond
   ((vectorp diagnostics) (length diagnostics))
   ((listp diagnostics) (length diagnostics))
   ((hash-table-p diagnostics) (hash-table-count diagnostics))
   (diagnostics 1)
   (t 0)))

(defun soma-bridge-daemon--record-lsp-diagnostics (workspace params)
  "Record a bounded diagnostic notification trace for WORKSPACE and PARAMS."
  (let* ((uri (soma-gym-bridge--value params "uri"))
         (diagnostics (soma-gym-bridge--value params "diagnostics"))
         (server-id
          (if workspace
              (soma-gym-bridge--json-symbol
               (ignore-errors
                 (lsp--client-server-id (lsp--workspace-client workspace))))
            ""))
         (path (or (ignore-errors (lsp--uri-to-path uri)) ""))
         (event
          (soma-bridge-daemon--alist
           (cons "server_id" server-id)
           (cons "uri" (or uri ""))
           (cons "path" path)
           (cons "count"
                 (soma-bridge-daemon--lsp-diagnostic-count diagnostics))
           (cons "raw_excerpt"
                 (let ((raw (if diagnostics (format "%S" diagnostics) "")))
                   (substring raw 0 (min 1200 (length raw))))))))
    (push event soma-bridge-daemon--lsp-diagnostic-events)
    (when (> (length soma-bridge-daemon--lsp-diagnostic-events) 20)
      (setcdr (nthcdr 19 soma-bridge-daemon--lsp-diagnostic-events) nil))))

(defun soma-bridge-daemon--record-lsp-notify (method params)
  "Record a bounded outgoing LSP notification trace."
  (when (member method
                '("textDocument/didOpen"
                  "textDocument/didChange"
                  "textDocument/didSave"
                  "workspace/didChangeConfiguration"))
    (let* ((text-document (soma-gym-bridge--value params "textDocument"))
           (uri (or (soma-gym-bridge--value text-document "uri") ""))
           (version (soma-gym-bridge--value text-document "version"))
           (content-changes (soma-gym-bridge--value params "contentChanges"))
           (change-count (soma-bridge-daemon--lsp-diagnostic-count
                          content-changes))
           (event
            (soma-bridge-daemon--alist
             (cons "method" method)
             (cons "uri" uri)
             (cons "version" (or version :json-false))
             (cons "content_change_count" change-count)
             (cons "raw_excerpt"
                   (let ((raw (format "%S" params)))
                     (substring raw 0 (min 1200 (length raw))))))))
      (push event soma-bridge-daemon--lsp-notify-events)
      (when (> (length soma-bridge-daemon--lsp-notify-events) 40)
        (setcdr (nthcdr 39 soma-bridge-daemon--lsp-notify-events) nil)))))

(defun soma-bridge-daemon--record-lsp-incoming-notification (workspace message)
  "Record a bounded incoming LSP notification trace for WORKSPACE and MESSAGE."
  (let* ((method (or (soma-gym-bridge--value message "method") ""))
         (params (soma-gym-bridge--value message "params"))
         (text-document (soma-gym-bridge--value params "textDocument"))
         (uri (or (soma-gym-bridge--value params "uri")
                  (soma-gym-bridge--value text-document "uri")
                  ""))
         (diagnostics (soma-gym-bridge--value params "diagnostics"))
         (server-id
          (if workspace
              (soma-gym-bridge--json-symbol
               (ignore-errors
                 (lsp--client-server-id (lsp--workspace-client workspace))))
            ""))
         (event
          (soma-bridge-daemon--alist
           (cons "server_id" server-id)
           (cons "method" method)
           (cons "uri" uri)
           (cons "diagnostic_count"
                 (soma-bridge-daemon--lsp-diagnostic-count diagnostics))
           (cons "raw_excerpt"
                 (let ((raw (format "%S" message)))
                   (substring raw 0 (min 1200 (length raw))))))))
    (push event soma-bridge-daemon--lsp-incoming-events)
    (when (> (length soma-bridge-daemon--lsp-incoming-events) 40)
      (setcdr (nthcdr 39 soma-bridge-daemon--lsp-incoming-events) nil))))

(defun soma-bridge-daemon--install-lsp-diagnostic-trace ()
  "Install daemon-local diagnostic notification instrumentation."
  (unless soma-bridge-daemon--lsp-diagnostic-advice-installed
    (advice-add 'lsp--on-diagnostics
                :before
                #'soma-bridge-daemon--record-lsp-diagnostics)
    (advice-add 'lsp-notify
                :before
                #'soma-bridge-daemon--record-lsp-notify)
    (advice-add 'lsp--on-notification
                :before
                #'soma-bridge-daemon--record-lsp-incoming-notification)
    (setq soma-bridge-daemon--lsp-diagnostic-advice-installed t)))

(defun soma-bridge-daemon--doom-straight-build-dir ()
  (expand-file-name
   (format "~/.emacs.d/.local/straight/build-%d.%d"
           emacs-major-version emacs-minor-version)))

(defun soma-bridge-daemon--add-straight-build-paths ()
  (let ((build (soma-bridge-daemon--doom-straight-build-dir)))
    (when (file-directory-p build)
      (dolist (dir (directory-files build t directory-files-no-dot-files-regexp))
        (when (file-directory-p dir)
          (add-to-list 'load-path dir))))))

(defun soma-bridge-daemon--load-magit ()
  (soma-bridge-daemon--add-straight-build-paths)
  (require 'transient)
  (require 'magit)
  (require 'magit-status)
  (require 'magit-apply)
  (require 'magit-section))

(defun soma-bridge-daemon--load-jupyter ()
  (soma-bridge-daemon--add-straight-build-paths)
  (require 'python)
  (require 'jupyter)
  (require 'jupyter-client)
  (require 'jupyter-messages)
  (require 'jupyter-repl))

(defun soma-bridge-daemon--lsp-stdio-connection (command &optional test-command)
  "Create an lsp-mode stdio connection backed by rootfs-owned COMMAND."
  (list
   :connect
   (lambda (filter sentinel name environment-fn _workspace)
     (let ((final-command (lsp-resolve-final-command command))
           (process-name (generate-new-buffer-name name))
           (process-environment
            (lsp--compute-process-environment environment-fn)))
       (let* ((stderr-buf
               (get-buffer-create (format "*%s::stderr*" process-name)))
              (default-directory (lsp--default-directory-for-connection))
              (proc
               (make-process
                :name process-name
                :connection-type 'pipe
                :buffer (format "*%s*" process-name)
                :coding 'no-conversion
                :command final-command
                :filter filter
                :sentinel sentinel
                :stderr stderr-buf
                :noquery t
                :file-handler t)))
         (set-process-query-on-exit-flag proc nil)
         (when-let* ((stderr-proc (get-buffer-process stderr-buf)))
           (set-process-query-on-exit-flag stderr-proc nil))
         (with-current-buffer stderr-buf
           (special-mode))
         (cons proc proc))))
   :test?
   (or test-command
       (lambda ()
         (lsp-server-present?
          (lsp-resolve-final-command command t))))))

(defun soma-bridge-daemon--load-lsp ()
  (soma-bridge-daemon--add-straight-build-paths)
  (setenv "LSP_USE_PLISTS" "true")
  (require 'python)
  (require 'lsp-mode)
  (soma-bridge-daemon--install-lsp-diagnostic-trace)
  (setq soma-bridge-daemon--lsp-jsonrpc-transport-suppressed t)
  (fset 'lsp-stdio-connection
        #'soma-bridge-daemon--lsp-stdio-connection)
  (require 'lsp-pylsp)
  (setq lsp-session-file nil
        lsp-auto-guess-root nil
        lsp-guess-root-without-session nil
        lsp-enable-snippet nil
        lsp-enable-file-watchers nil
        lsp-enable-symbol-highlighting nil
        ;; The bridge observes lsp-mode's protocol/cache state directly.  Keep
        ;; daemon fixtures independent of Flymake/Flycheck presentation
        ;; backends, which are UI concerns and may be absent in batch-like
        ;; daemon sessions.
        lsp-diagnostics-provider :none
        lsp-lens-enable nil
        lsp-headerline-breadcrumb-enable nil
        lsp-modeline-workspace-status-enable nil
        lsp-modeline-diagnostics-enable nil
        lsp-modeline-code-actions-enable nil
        lsp-log-io nil
        lsp-response-timeout 20
        lsp-restart 'ignore
        lsp-pylsp-server-command '("/usr/bin/pylsp")))

(defun soma-bridge-daemon--run-process (program &rest args)
  (with-temp-buffer
    (let ((status (apply #'process-file program nil t nil args))
          (output (buffer-substring-no-properties (point-min) (point-max))))
      (unless (equal status 0)
        (error "%s %S failed with %S: %s" program args status output))
      output)))

(defun soma-bridge-daemon--git (repo &rest args)
  (let ((default-directory (file-name-as-directory repo)))
    (apply #'soma-bridge-daemon--run-process "git" args)))

(defun soma-bridge-daemon--run-process-in (cwd program &rest args)
  (let ((default-directory (file-name-as-directory cwd)))
    (apply #'soma-bridge-daemon--run-process program args)))

(defun soma-bridge-daemon--write-file (path text)
  (make-directory (file-name-directory path) t)
  (with-temp-file path
    (insert text)))

(defun soma-bridge-daemon--process-output (process)
  (let ((buffer (and process (process-buffer process))))
    (if (buffer-live-p buffer)
        (with-current-buffer buffer
          (buffer-substring-no-properties (point-min) (point-max)))
      "")))

(defun soma-bridge-daemon--target-content (&optional selected other external)
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

(defun soma-bridge-daemon--make-magit-repo (&optional target-shape)
  (let ((repo (make-temp-file "soma-daemon-magit-" t)))
    (soma-bridge-daemon--git repo "init" "-q")
    (soma-bridge-daemon--git repo "config" "user.name" "Soma Daemon Benchmark")
    (soma-bridge-daemon--git repo "config" "user.email"
                              "soma-daemon-bench@example.invalid")
    (soma-bridge-daemon--write-file
     (expand-file-name "target.txt" repo)
     (soma-bridge-daemon--target-content nil nil))
    (soma-bridge-daemon--write-file
     (expand-file-name "debug.txt" repo)
     "debug-base\n")
    (soma-bridge-daemon--write-file
     (expand-file-name "staged.txt" repo)
     "staged-base\n")
    (soma-bridge-daemon--git repo "add" ".")
    (soma-bridge-daemon--git repo "commit" "-q" "-m" "initial")
    (soma-bridge-daemon--write-file
     (expand-file-name "staged.txt" repo)
     "staged-expected\n")
    (soma-bridge-daemon--git repo "add" "staged.txt")
    (soma-bridge-daemon--write-file
     (expand-file-name "debug.txt" repo)
     "debug-base\ndebug-worktree-only\n")
    (soma-bridge-daemon--write-file
     (expand-file-name "target.txt" repo)
     (pcase target-shape
       ('one-hunk (soma-bridge-daemon--target-content t nil nil))
       ('external-hunk (soma-bridge-daemon--target-content t nil t))
       (_ (soma-bridge-daemon--target-content t t nil))))
    repo))

(defun soma-bridge-daemon--display-buffer (buffer)
  (set-window-buffer (selected-window) buffer)
  (selected-window))

(defun soma-bridge-daemon--section-slot (section slot)
  (eieio-oref section slot))

(defun soma-bridge-daemon--position-value (position)
  (if (markerp position)
      (marker-position position)
    position))

(defun soma-bridge-daemon--section-ancestor-file-p (section file)
  (let ((cursor section)
        found)
    (while cursor
      (when (and (eq (soma-bridge-daemon--section-slot cursor 'type) 'file)
                 (equal (soma-bridge-daemon--section-slot cursor 'value)
                        file))
        (setq found t))
      (setq cursor (soma-bridge-daemon--section-slot cursor 'parent)))
    found))

(defun soma-bridge-daemon--section-text (section)
  (buffer-substring-no-properties
   (soma-bridge-daemon--section-slot section 'start)
   (soma-bridge-daemon--section-slot section 'end)))

(defun soma-bridge-daemon--section-target-point (section needle)
  (save-excursion
    (goto-char (soma-bridge-daemon--section-slot section 'content))
    (if (search-forward needle
                        (soma-bridge-daemon--section-slot section 'end)
                        t)
        (match-beginning 0)
      (soma-bridge-daemon--section-slot section 'content))))

(defun soma-bridge-daemon--section-observation (section &optional target-point)
  (when section
    (let ((text (soma-bridge-daemon--section-text section)))
      (soma-bridge-daemon--alist
       (cons "type"
             (format "%S" (soma-bridge-daemon--section-slot section 'type)))
       (cons "value"
             (format "%S" (soma-bridge-daemon--section-slot section 'value)))
       (cons "start"
             (soma-bridge-daemon--position-value
              (soma-bridge-daemon--section-slot section 'start)))
       (cons "content"
             (soma-bridge-daemon--position-value
              (soma-bridge-daemon--section-slot section 'content)))
       (cons "end"
             (soma-bridge-daemon--position-value
              (soma-bridge-daemon--section-slot section 'end)))
       (cons "target_point"
             (if target-point
                 (soma-bridge-daemon--position-value target-point)
               :json-false))
       (cons "text_excerpt" (substring text 0 (min 400 (length text))))))))

(defun soma-bridge-daemon--find-section (buffer predicate)
  (let (found)
    (with-current-buffer buffer
      (magit-map-sections
       (lambda (section)
         (when (and (not found) (funcall predicate section))
           (setq found section)))))
    found))

(defun soma-bridge-daemon--find-hunk-section (buffer file needle)
  (soma-bridge-daemon--find-section
   buffer
   (lambda (section)
     (and (eq (soma-bridge-daemon--section-slot section 'type) 'hunk)
          (soma-bridge-daemon--section-ancestor-file-p section file)
          (string-match-p (regexp-quote needle)
                          (with-current-buffer buffer
                            (soma-bridge-daemon--section-text section)))))))

(defun soma-bridge-daemon--append-hook-log (line)
  (when (or soma-bridge-daemon--magit-buffer-hook-log
            soma-bridge-daemon--magit-hook-log)
    (append-to-file
     line nil
     (or soma-bridge-daemon--magit-buffer-hook-log
         soma-bridge-daemon--magit-hook-log))))

(defun soma-bridge-daemon--capture-post-stage-hook ()
  (soma-bridge-daemon--append-hook-log
   (format "post-stage:%S\n" this-command)))

(defun soma-bridge-daemon--capture-refresh-hook ()
  (soma-bridge-daemon--append-hook-log
   (format "refresh:%s\n" (buffer-name (current-buffer)))))

(defun soma-bridge-daemon-terminal-prompt-insert (value)
  "Read VALUE through a real minibuffer prompt and append it to this buffer."
  (interactive (list (read-string "Soma terminal prompt: ")))
  (goto-char (point-max))
  (insert (format "terminal-prompt:%s\n" value)))

(defun soma-bridge-daemon--capture-comint-post-command ()
  (when (and soma-bridge-daemon--comint-command-log
             (memq this-command '(comint-send-input comint-interrupt-subjob)))
    (let ((inhibit-message t))
      (append-to-file
       (format "post-command:%S\n" this-command)
       nil
       soma-bridge-daemon--comint-command-log))))

(defun soma-bridge-daemon--comint-command (label counter)
  (format
   "printf 'SOMA_DAEMON_COMINT_SENT:%s\\n'; printf 'SOMA_DAEMON_COMINT_RUN:%%s\\n' \"$SOMA_INSULA_RUN_ID\"; printf 'SOMA_DAEMON_COMINT_CWD:%%s\\n' \"$PWD\"; printf '%s\\n' >> %s; printf 'SOMA_DAEMON_COMINT_DONE:%s\\n'"
   label
   label
   (shell-quote-argument counter)
   label))

(defun soma-bridge-daemon--comint-interrupt-command (counter)
  (format
   "printf 'SOMA_DAEMON_COMINT_INTERRUPT_START\\n'; sleep 5 && { printf 'finished\\n' >> %s; printf 'SOMA_DAEMON_COMINT_INTERRUPT_DONE\\n'; }"
   (shell-quote-argument counter)))

(defun soma-bridge-daemon--setup-comint-fixture ()
  (require 'comint)
  (let* ((tmp (make-temp-file "soma-daemon-comint-" t))
         (counter (expand-file-name "counter" tmp))
         (command-log (expand-file-name "commands.log" tmp))
         (buffer (generate-new-buffer "*soma-daemon-comint-live*"))
         (label "live")
         process
         command)
    (with-current-buffer buffer
      (setq default-directory (file-name-as-directory tmp))
      (make-comint-in-buffer "soma-daemon-comint" buffer "sh" nil)
      (setq process (get-buffer-process buffer))
      (set-process-query-on-exit-flag process nil)
      (setq-local soma-bridge-daemon--comint-command-log command-log)
      (add-hook 'post-command-hook
                #'soma-bridge-daemon--capture-comint-post-command
                nil
                t))
    (accept-process-output process 0.2)
    (with-current-buffer buffer
      (goto-char (process-mark process)))
    (setq command (soma-bridge-daemon--comint-command label counter))
    (soma-bridge-daemon--alist
     (cons "tmp" tmp)
     (cons "counter" counter)
     (cons "command_log" command-log)
     (cons "buffer_name" (buffer-name buffer))
     (cons "process_name" (process-name process))
     (cons "pid" (or (process-id process) :json-false))
     (cons "command" command)
     (cons "expected_label" label)
     (cons "observation" (soma-gym-bridge-observe-comint buffer)))))

(defun soma-bridge-daemon--setup-comint-interrupt-fixture ()
  (require 'comint)
  (let* ((tmp (make-temp-file "soma-daemon-comint-interrupt-" t))
         (counter (expand-file-name "counter" tmp))
         (command-log (expand-file-name "commands.log" tmp))
         (buffer (generate-new-buffer "*soma-daemon-comint-interrupt*"))
         process
         command)
    (with-current-buffer buffer
      (setq default-directory (file-name-as-directory tmp))
      (make-comint-in-buffer "soma-daemon-comint-interrupt" buffer "sh" nil)
      (setq process (get-buffer-process buffer))
      (set-process-query-on-exit-flag process nil)
      (setq-local soma-bridge-daemon--comint-command-log command-log)
      (add-hook 'post-command-hook
                #'soma-bridge-daemon--capture-comint-post-command
                nil
                t))
    (accept-process-output process 0.2)
    (with-current-buffer buffer
      (goto-char (process-mark process)))
    (setq command (soma-bridge-daemon--comint-interrupt-command counter))
    (soma-bridge-daemon--alist
     (cons "tmp" tmp)
     (cons "counter" counter)
     (cons "command_log" command-log)
     (cons "buffer_name" (buffer-name buffer))
     (cons "process_name" (process-name process))
     (cons "pid" (or (process-id process) :json-false))
     (cons "command" command)
     (cons "start_needle" "SOMA_DAEMON_COMINT_INTERRUPT_START")
     (cons "done_needle" "SOMA_DAEMON_COMINT_INTERRUPT_DONE")
     (cons "observation" (soma-gym-bridge-observe-comint buffer)))))

(defun soma-bridge-daemon--setup-completion-fixture ()
  (let* ((initial "alpha \nomega\n")
         (human "human-")
         (proposal "beta")
         (prefix "alpha ")
         (suffix "\nomega\n")
         (point (1+ (length prefix)))
         (after-human (concat prefix human suffix))
         (buffer (generate-new-buffer "soma-daemon-stale-completion.txt"))
         observation)
    (with-current-buffer buffer
      (erase-buffer)
      (insert initial)
      (goto-char point)
      (setq-local completion-at-point-functions
                  (list
                   (lambda ()
                     (list point point
                           (list proposal "better" "gamma")
                           :exclusive 'no))))
      (setq observation (soma-gym-bridge-observe-buffer buffer)))
    (soma-bridge-daemon--alist
     (cons "buffer_name" (buffer-name buffer))
     (cons "initial_text" initial)
     (cons "after_human_text" after-human)
     (cons "point" point)
     (cons "start" point)
     (cons "end" point)
     (cons "prefix" prefix)
     (cons "suffix" suffix)
     (cons "proposal" proposal)
     (cons "human_text" human)
     (cons "post_human_read_end" (1+ (length after-human)))
     (cons "observation" observation))))

(defun soma-bridge-daemon--write-file (path text)
  "Write TEXT to PATH, creating its parent directory."
  (make-directory (file-name-directory path) t)
  (with-temp-file path
    (insert text)))

(defun soma-bridge-daemon--setup-org-report-fixture (&optional name)
  "Create an Org report buffer for live daemon reports-v0 requests."
  (require 'org)
  (let* ((fixture-name (and name (format "%s" name)))
         (tmp (make-temp-file
               (if fixture-name
                   (format "soma-daemon-org-report-%s-" fixture-name)
                 "soma-daemon-org-report-")
               t))
         (report-basename
          (if fixture-name
              (format "report-%s.org" fixture-name)
            "report.org"))
         (export-basename
          (if fixture-name
              (format "report-%s.html" fixture-name)
            "report.html"))
         (report-file (expand-file-name report-basename tmp))
         (publish-dir (file-name-as-directory
                       (expand-file-name "published" tmp)))
         (export-file (expand-file-name export-basename publish-dir))
         (initial-text
          (concat
           "#+title: Soma Daemon API Report\n"
           "* TODO Summary\n"
           ":PROPERTIES:\n"
           ":ID: daemon-report-summary\n"
           ":END:\n"
           "The daemon bridge should preserve Org structure.\n\n"
           "#+begin_src emacs-lisp :results output replace\n"
           "(princ \"ok\\n\")\n"
           "#+end_src\n"))
         buffer observation)
    (soma-bridge-daemon--write-file report-file initial-text)
    (make-directory publish-dir t)
    (setq buffer (find-file-noselect report-file))
    (with-current-buffer buffer
      (setq default-directory (file-name-as-directory tmp))
      (org-mode)
      (goto-char (point-min))
      (set-buffer-modified-p nil))
    (set-window-buffer (selected-window) buffer)
    (setq observation (soma-gym-bridge-observe-org-tree buffer 20 20 400))
    (soma-bridge-daemon--alist
     (cons "tmp" tmp)
     (cons "report_file" report-file)
     (cons "publish_dir" publish-dir)
     (cons "export_file" export-file)
     (cons "buffer_name" (buffer-name buffer))
     (cons "initial_text_sha256" (secure-hash 'sha256 initial-text))
     (cons "observation" observation))))

(defun soma-bridge-daemon--setup-org-duplicate-id-fixture ()
  "Create an Org report buffer with duplicate IDs for negative tests."
  (require 'org)
  (let* ((tmp (make-temp-file "soma-daemon-org-duplicate-id-" t))
         (report-file (expand-file-name "report-duplicate-id.org" tmp))
         (initial-text
          (concat
           "#+title: Soma Duplicate ID Report\n"
           "* TODO First Summary\n"
           ":PROPERTIES:\n"
           ":ID: daemon-duplicate-heading\n"
           ":END:\n"
           "First body.\n\n"
           "* TODO Second Summary\n"
           ":PROPERTIES:\n"
           ":ID: daemon-duplicate-heading\n"
           ":END:\n"
           "Second body.\n"))
         buffer observation)
    (soma-bridge-daemon--write-file report-file initial-text)
    (setq buffer (find-file-noselect report-file))
    (with-current-buffer buffer
      (setq default-directory (file-name-as-directory tmp))
      (org-mode)
      (goto-char (point-min))
      (set-buffer-modified-p nil))
    (setq observation (soma-gym-bridge-observe-org-tree buffer 20 20 400))
    (soma-bridge-daemon--alist
     (cons "tmp" tmp)
     (cons "report_file" report-file)
     (cons "buffer_name" (buffer-name buffer))
     (cons "duplicate_id" "daemon-duplicate-heading")
     (cons "initial_text_sha256" (secure-hash 'sha256 initial-text))
     (cons "observation" observation))))

(defun soma-bridge-daemon--setup-org-subtree-bypass-fixture ()
  "Create an Org report buffer for direct-text subtree reparenting tests."
  (require 'org)
  (let* ((tmp (make-temp-file "soma-daemon-org-subtree-bypass-" t))
         (report-file (expand-file-name "report-subtree-bypass.org" tmp))
         (initial-text
          (concat
           "#+title: Soma Subtree Bypass Report\n"
           "* TODO Inbox\n"
           ":PROPERTIES:\n"
           ":ID: daemon-subtree-inbox\n"
           ":END:\n"
           "** TODO Draft Section\n"
           ":PROPERTIES:\n"
           ":ID: daemon-subtree-draft\n"
           ":END:\n"
           "Draft body.\n\n"
           "* TODO Published\n"
           ":PROPERTIES:\n"
           ":ID: daemon-subtree-published\n"
           ":END:\n"
           "Published body.\n"))
         buffer observation)
    (soma-bridge-daemon--write-file report-file initial-text)
    (setq buffer (find-file-noselect report-file))
    (with-current-buffer buffer
      (setq default-directory (file-name-as-directory tmp))
      (org-mode)
      (goto-char (point-min))
      (set-buffer-modified-p nil))
    (setq observation (soma-gym-bridge-observe-org-tree buffer 20 20 400))
    (soma-bridge-daemon--alist
     (cons "tmp" tmp)
     (cons "report_file" report-file)
     (cons "buffer_name" (buffer-name buffer))
     (cons "moved_subtree_id" "daemon-subtree-draft")
     (cons "target_parent_id" "daemon-subtree-published")
     (cons "initial_text_sha256" (secure-hash 'sha256 initial-text))
     (cons "observation" observation))))

(defun soma-bridge-daemon--setup-dired-rename-fixture ()
  "Create a Dired buffer for live daemon no-overwrite rename requests."
  (require 'dired)
  (let* ((tmp (make-temp-file "soma-daemon-dired-rename-" t))
         (source-file (expand-file-name "source.txt" tmp))
         (target-file (expand-file-name "renamed.txt" tmp))
         (decoy-file (expand-file-name "decoy.txt" tmp))
         buffer observation)
    (soma-bridge-daemon--write-file source-file "source\n")
    (soma-bridge-daemon--write-file decoy-file "decoy\n")
    (setq buffer (dired-noselect tmp))
    (with-current-buffer buffer
      (revert-buffer))
    (setq observation (soma-gym-bridge-observe-dired buffer 50))
    (soma-bridge-daemon--alist
     (cons "tmp" tmp)
     (cons "buffer_name" (buffer-name buffer))
     (cons "source" "source.txt")
     (cons "target" "renamed.txt")
     (cons "source_path" source-file)
     (cons "target_path" target-file)
     (cons "decoy_path" decoy-file)
     (cons "observation" observation))))

(defun soma-bridge-daemon--setup-dired-delete-fixture ()
  "Create a marked Dired buffer for live daemon delete requests."
  (require 'dired)
  (let* ((tmp (make-temp-file "soma-daemon-dired-delete-" t))
         (delete-a-file (expand-file-name "delete-a.txt" tmp))
         (delete-b-file (expand-file-name "delete-b.txt" tmp))
         (flagged-decoy-file (expand-file-name "flagged-decoy.txt" tmp))
         buffer observation)
    (soma-bridge-daemon--write-file delete-a-file "delete-a\n")
    (soma-bridge-daemon--write-file delete-b-file "delete-b\n")
    (soma-bridge-daemon--write-file flagged-decoy-file "flagged-decoy\n")
    (setq buffer (dired-noselect tmp))
    (with-current-buffer buffer
      (revert-buffer)
      (dired-goto-file delete-a-file)
      (dired-mark 1)
      (dired-goto-file delete-b-file)
      (dired-mark 1)
      (dired-goto-file flagged-decoy-file)
      (dired-flag-file-deletion 1))
    (setq observation (soma-gym-bridge-observe-dired buffer 50))
    (soma-bridge-daemon--alist
     (cons "tmp" tmp)
     (cons "buffer_name" (buffer-name buffer))
     (cons "marked_files" '("delete-a.txt" "delete-b.txt"))
     (cons "delete_a_path" delete-a-file)
     (cons "delete_b_path" delete-b-file)
     (cons "flagged_decoy_path" flagged-decoy-file)
     (cons "observation" observation))))

(defun soma-bridge-daemon--setup-project-host-shadow-fixture ()
  "Create project buffers whose tool lookup resolves to host-shadow tools."
  (let* ((root (make-temp-file "soma-daemon-project-host-shadow-" t))
         (src-dir (expand-file-name "src" root))
         (rg-file (expand-file-name "src/rg_subject.py" root))
         (git-file (expand-file-name "src/git_subject.py" root))
         (host-parent
          (expand-file-name
           ".scratch/emacs-gym/host-shadow-tools/"
           soma-bridge-daemon--repo-root))
         (host-rg-dir
          (make-temp-file
           (expand-file-name "soma-daemon-host-rg-" host-parent)
           t))
         (host-git-dir
          (make-temp-file
           (expand-file-name "soma-daemon-host-git-" host-parent)
           t))
         (host-rg-path (expand-file-name "rg" host-rg-dir))
         (host-git-path (expand-file-name "git" host-git-dir))
         (host-rg-marker (expand-file-name "called" host-rg-dir))
         (host-git-marker (expand-file-name "called" host-git-dir))
         rg-buffer
         git-buffer)
    (make-directory host-parent t)
    (make-directory src-dir t)
    (soma-bridge-daemon--write-file
     rg-file
     "def daemon_host_rg_probe():\n    return 'daemon-host-rg-needle'\n")
    (soma-bridge-daemon--write-file
     git-file
     "def daemon_host_git_probe():\n    return 'daemon-host-git-needle'\n")
    (soma-bridge-daemon--git root "init" "-q")
    (soma-bridge-daemon--git root "add" "src/rg_subject.py" "src/git_subject.py")
    (soma-bridge-daemon--write-file
     host-rg-path
     (format
      "#!/bin/sh\nprintf 'called\\n' >> %s\nprintf 'HOST_RG_SHOULD_NOT_RUN\\n'\nexit 0\n"
      (shell-quote-argument host-rg-marker)))
    (set-file-modes host-rg-path #o755)
    (soma-bridge-daemon--write-file
     host-git-path
     (format
      "#!/bin/sh\nprintf 'called %%s\\n' \"$*\" >> %s\nexit 0\n"
      (shell-quote-argument host-git-marker)))
    (set-file-modes host-git-path #o755)
    (setq rg-buffer (find-file-noselect rg-file))
    (setq git-buffer (find-file-noselect git-file))
    (with-current-buffer rg-buffer
      (rename-buffer "soma-project-host-rg.py" t)
      (python-mode)
      (setq default-directory (file-name-as-directory (file-truename root)))
      (setq-local exec-path (list host-rg-dir))
      (setq-local process-environment
                  (cons (concat "PATH=" host-rg-dir)
                        process-environment)))
    (with-current-buffer git-buffer
      (rename-buffer "soma-project-host-git.py" t)
      (python-mode)
      (setq default-directory (file-name-as-directory (file-truename root)))
      (setq-local exec-path (list host-git-dir))
      (setq-local process-environment
                  (cons (concat "PATH=" host-git-dir)
                        process-environment)))
    (soma-bridge-daemon--alist
     (cons "root" (file-name-as-directory (file-truename root)))
     (cons "rg_buffer_name" (buffer-name rg-buffer))
     (cons "git_buffer_name" (buffer-name git-buffer))
     (cons "host_rg_path" host-rg-path)
     (cons "host_rg_marker" host-rg-marker)
     (cons "host_git_path" host-git-path)
     (cons "host_git_marker" host-git-marker))))

(defun soma-bridge-daemon--setup-profile-smoke-code-fixture ()
  "Create a file-backed project buffer for the stable profile smoke path."
  (require 'python)
  (let* ((root (make-temp-file "soma-daemon-profile-smoke-project-" t))
         (src (expand-file-name "src" root))
         (source-file (expand-file-name "profile_task.py" src))
         (readme-file (expand-file-name "README.md" root))
         (initial-text
          (concat
           "def profile_entry():\n"
           "    note = \"TODO_NOTE\"\n"
           "    answer = \n"
           "    return note, answer\n"))
         (completion-text "42")
         buffer
         completion-point
         observation)
    (make-directory src t)
    (soma-bridge-daemon--write-file source-file initial-text)
    (soma-bridge-daemon--write-file
     readme-file
     "# Soma profile smoke project\n\nContains soma-profile-smoke-needle.\n")
    (soma-bridge-daemon--git root "init" "-q")
    (soma-bridge-daemon--git root "config" "user.name"
                              "Soma Profile Smoke")
    (soma-bridge-daemon--git root "config" "user.email"
                              "soma-profile-smoke@example.invalid")
    (soma-bridge-daemon--git root "add" ".")
    (soma-bridge-daemon--git root "commit" "-q" "-m" "initial")
    (setq buffer (find-file-noselect source-file))
    (with-current-buffer buffer
      (setq default-directory (file-name-as-directory root))
      (python-mode)
      (goto-char (point-min))
      (search-forward "    answer = ")
      (setq completion-point (point))
      (setq-local completion-at-point-functions
                  (list
                   (lambda ()
                     (list completion-point completion-point
                           (list completion-text "43")
                           :exclusive 'no))))
      (set-buffer-modified-p nil))
    (set-window-buffer (selected-window) buffer)
    (with-current-buffer buffer
      (goto-char completion-point)
      (set-window-point (selected-window) completion-point)
      (setq observation (soma-gym-bridge-observe-buffer buffer)))
    (soma-bridge-daemon--alist
     (cons "root" (file-name-as-directory (file-truename root)))
     (cons "source_file" source-file)
     (cons "readme_file" readme-file)
     (cons "buffer_name" (buffer-name buffer))
     (cons "initial_text" initial-text)
     (cons "completion_text" completion-text)
     (cons "completion_point" completion-point)
     (cons "edit_old" "TODO_NOTE")
     (cons "edit_new" "API_READY")
     (cons "search_needle" "soma-profile-smoke-needle")
     (cons "observation" observation))))

(defun soma-bridge-daemon--setup-profile-smoke-fixture ()
  "Create fresh fixtures for one stable API profile-level smoke workflow."
  (let* ((code-fixture
          (soma-bridge-daemon--setup-profile-smoke-code-fixture))
         (comint-fixture
          (soma-bridge-daemon--setup-comint-fixture))
         (comint-interrupt-fixture
          (soma-bridge-daemon--setup-comint-interrupt-fixture))
         (jupyter-fixture
          (soma-bridge-daemon--setup-jupyter-attach-fixture))
         (jupyter-display-fixture
          (soma-bridge-daemon--setup-jupyter-fixture
           "profile-display"
           #'soma-bridge-daemon--jupyter-display-update-code
           t))
         (org-report-fixture
          (soma-bridge-daemon--setup-org-report-fixture "profile-smoke"))
         (dired-rename-fixture
          (soma-bridge-daemon--setup-dired-rename-fixture))
         (dired-delete-fixture
          (soma-bridge-daemon--setup-dired-delete-fixture))
         (magit-fixture
          (soma-bridge-daemon--setup-magit-fixture))
         (magit-file-fixture
          (soma-bridge-daemon--setup-magit-fixture 'one-hunk))
         (lsp-fixture
          (soma-bridge-daemon--setup-lsp-fixture
           "profile-smoke"
           t))
         (code-buffer
          (get-buffer (alist-get "buffer_name" code-fixture nil nil #'string=)))
         (completion-point
          (alist-get "completion_point" code-fixture nil nil #'string=)))
    (when (buffer-live-p code-buffer)
      (set-window-buffer (selected-window) code-buffer)
      (with-current-buffer code-buffer
        (goto-char completion-point)
        (set-window-point (selected-window) completion-point)))
    (soma-bridge-daemon--alist
     (cons "code" code-fixture)
     (cons "comint" comint-fixture)
     (cons "comint_interrupt" comint-interrupt-fixture)
     (cons "jupyter" jupyter-fixture)
     (cons "jupyter_display" jupyter-display-fixture)
     (cons "org_report" org-report-fixture)
     (cons "dired_rename" dired-rename-fixture)
     (cons "dired_delete" dired-delete-fixture)
     (cons "magit" magit-fixture)
     (cons "magit_file" magit-file-fixture)
     (cons "lsp" lsp-fixture))))

(defun soma-bridge-daemon--wait-until (predicate timeout)
  (let ((deadline (+ (float-time) timeout))
        done)
    (while (and (not done) (< (float-time) deadline))
      (accept-process-output nil 0.05)
      (sit-for 0.01)
      (setq done (funcall predicate)))
    done))

(defun soma-bridge-daemon--drain (seconds)
  (let ((deadline (+ (float-time) seconds)))
    (while (< (float-time) deadline)
      (accept-process-output nil 0.05)
      (sit-for 0.01))))

(defun soma-bridge-daemon--lsp-server-process-p (process)
  (let ((command (ignore-errors (process-command process))))
    (and (listp command)
         (seq-some
          (lambda (part)
            (and (stringp part)
                 (string-match-p "pylsp\\|pyright-langserver" part)))
          command))))

(defun soma-bridge-daemon--live-lsp-server-processes ()
  (seq-filter
   (lambda (process)
     (and (process-live-p process)
          (soma-bridge-daemon--lsp-server-process-p process)))
   (process-list)))

(defun soma-bridge-daemon--shutdown-lsp ()
  (when (fboundp 'lsp--global-teardown)
    (ignore-errors (lsp--global-teardown)))
  (soma-bridge-daemon--drain 0.5)
  (dolist (process (soma-bridge-daemon--live-lsp-server-processes))
    (ignore-errors (delete-process process)))
  (soma-bridge-daemon--wait-until
   (lambda ()
     (null (soma-bridge-daemon--live-lsp-server-processes)))
   5)
  (when (fboundp 'make-lsp-session)
    (setq lsp--session (make-lsp-session))))

(defun soma-bridge-daemon--goto-lsp-symbol (symbol)
  (goto-char (point-min))
  (unless (search-forward symbol nil t)
    (error "symbol not found: %s" symbol))
  (goto-char (match-beginning 0)))

(defun soma-bridge-daemon--setup-lsp-fixture (&optional name preserve-existing)
  (soma-bridge-daemon--load-lsp)
  (let* ((fixture-name (or name "python"))
         (root (make-temp-file
                (format "soma-daemon-lsp-%s-" fixture-name)
                t))
         (pkg (expand-file-name "soma_pkg" root))
         (tests (expand-file-name "tests" root))
         (definition-file (expand-file-name "soma_pkg/core.py" root))
         (main-file (expand-file-name "main.py" root))
         (test-file (expand-file-name "tests/test_core.py" root))
         (symbol "compute_value")
         (new-name "compute_answer")
         (canonical-root (file-name-as-directory (file-truename root)))
         definition-buffer
         main-buffer
         test-buffer
         initialized
         workspace
         observation
         initial-test)
    (soma-bridge-daemon--write-file
     (expand-file-name "pyproject.toml" root)
     "[project]\nname = \"soma-daemon-lsp-fixture\"\nversion = \"0.0.0\"\n")
    (soma-bridge-daemon--write-file
     (expand-file-name "__init__.py" pkg)
     "")
    (soma-bridge-daemon--write-file
     definition-file
     "def compute_value(x: int) -> int:\n    return x + 22\n")
    (soma-bridge-daemon--write-file
     main-file
     "from soma_pkg.core import compute_value\n\nRESULT = compute_value(20)\n")
    (soma-bridge-daemon--write-file
     (expand-file-name "__init__.py" tests)
     "")
    (soma-bridge-daemon--write-file
     test-file
     "import unittest\nfrom soma_pkg.core import compute_value\n\nclass CoreTest(unittest.TestCase):\n    def test_answer(self):\n        self.assertEqual(compute_value(20), 42)\n\nif __name__ == '__main__':\n    unittest.main()\n")
    (setq initial-test
          (soma-bridge-daemon--run-process-in
           root "python3" "-m" "unittest" "discover" "-s" "tests"))
    (unless preserve-existing
      (soma-bridge-daemon--shutdown-lsp))
    (setq lsp--session
          (if (and preserve-existing
                   (boundp 'lsp--session)
                   lsp--session)
              (make-lsp-session
               :folders (cl-remove-duplicates
                         (cons canonical-root
                               (lsp-session-folders lsp--session))
                         :test #'equal)
               :folders-blocklist (lsp-session-folders-blocklist lsp--session)
               :server-id->folders
               (lsp-session-server-id->folders lsp--session)
               :folder->servers (lsp-session-folder->servers lsp--session)
               :metadata (lsp-session-metadata lsp--session))
            (make-lsp-session
             :folders (list canonical-root)
             :folders-blocklist nil)))
    (setq definition-buffer (find-file-noselect definition-file))
    (setq main-buffer (find-file-noselect main-file))
    (setq test-buffer (find-file-noselect test-file))
    (when name
      (with-current-buffer definition-buffer
        (rename-buffer (format "soma-lsp-%s-core.py" fixture-name) t))
      (with-current-buffer main-buffer
        (rename-buffer (format "soma-lsp-%s-main.py" fixture-name) t))
      (with-current-buffer test-buffer
        (rename-buffer (format "soma-lsp-%s-test_core.py" fixture-name) t)))
    (dolist (buffer (list definition-buffer main-buffer test-buffer))
      (with-current-buffer buffer
        (setq default-directory canonical-root)
        (python-mode)
        (setq-local lsp-enabled-clients '(pylsp))
        (lsp)))
    (with-current-buffer definition-buffer
      (setq initialized
            (soma-bridge-daemon--wait-until
             (lambda ()
               (and lsp--buffer-workspaces
                    (setq workspace (car lsp--buffer-workspaces))
                    (eq (lsp--workspace-status workspace) 'initialized)))
             30))
      (soma-bridge-daemon--drain 0.5)
      (soma-bridge-daemon--goto-lsp-symbol symbol)
      (setq observation
            (soma-gym-bridge-observe-lsp-source definition-buffer 1600)))
    (push
     (soma-bridge-daemon--alist
      (cons "root" root)
      (cons "buffers" (list definition-buffer main-buffer test-buffer)))
     soma-bridge-daemon--lsp-fixtures)
    (soma-bridge-daemon--alist
     (cons "name" fixture-name)
     (cons "language" "python")
     (cons "server_id" "pylsp")
     (cons "root" canonical-root)
     (cons "buffer_name" (buffer-name definition-buffer))
     (cons "main_buffer_name" (buffer-name main-buffer))
     (cons "test_buffer_name" (buffer-name test-buffer))
     (cons "symbol" symbol)
     (cons "new_name" new-name)
     (cons "definition_file" definition-file)
     (cons "main_file" main-file)
     (cons "test_file" test-file)
     (cons "test_dir" tests)
     (cons "definition_point"
           (with-current-buffer definition-buffer (point)))
     (cons "workspace_initialized" (soma-bridge-daemon--bool initialized))
     (cons "pylsp_path" (or (executable-find "pylsp") ""))
     (cons "python_path" (or (executable-find "python3") ""))
     (cons "transport"
           (if soma-bridge-daemon--lsp-jsonrpc-transport-suppressed
               "lsp-mode-stdio-process-filter"
             "emacs-json-rpc-connection"))
     (cons "initial_test" initial-test)
     (cons "observation" observation))))

(defun soma-bridge-daemon--register-pyright-diagnostics-settings ()
  "Register minimal pyright settings owned by the daemon diagnostic fixture."
  (lsp-register-custom-settings
   `(("python.analysis.diagnosticMode" "openFilesOnly")
     ("python.analysis.typeCheckingMode" "basic")
     ("python.analysis.autoSearchPaths" t t)
     ("python.analysis.logLevel" "info")
     ("python.pythonPath" ,(lambda () (or (executable-find "python3")
                                          "python3")))
     ;; Pyright expects this setting to be present; use an empty rootfs-owned
     ;; venv path rather than leaking host virtualenv state.
     ("python.venvPath" ""))))

(defun soma-bridge-daemon--register-pyright-diagnostics-client ()
  (soma-bridge-daemon--load-lsp)
  (soma-bridge-daemon--register-pyright-diagnostics-settings)
  (lsp-register-client
   (make-lsp-client
    :new-connection
    (lsp-stdio-connection '("pyright-langserver" "--stdio"))
    :activation-fn (lsp-activate-on "python")
    :major-modes '(python-mode python-ts-mode)
    :priority 10
    :server-id 'soma-pyright-diagnostics
    :initialized-fn
    (lambda (workspace)
      (with-lsp-workspace workspace
        ;; Match lsp-pyright's initialize path: notify pyright that client
        ;; settings are available, then answer its workspace configuration
        ;; requests through lsp-mode's registered custom settings.
        (lsp--set-configuration
         (make-hash-table :test 'equal)))))))

(defun soma-bridge-daemon--setup-lsp-diagnostics-fixture ()
  (soma-bridge-daemon--register-pyright-diagnostics-client)
  (let* ((root (make-temp-file "soma-daemon-lsp-pyright-" t))
         (tests (expand-file-name "tests" root))
         (source-file (expand-file-name "diagnostics_subject.py" root))
         (test-file (expand-file-name "tests/test_core.py" root))
         (symbol "compute_total")
         (good-return "return x + 22")
         (broken-return "return x +")
         (expected-diagnostic-needle "Expected expression")
         (canonical-root (file-name-as-directory (file-truename root)))
         buffer
         workspace
         initialized
         observation
         initial-test)
    (soma-bridge-daemon--write-file
     (expand-file-name "pyrightconfig.json" root)
     "{\n  \"include\": [\"diagnostics_subject.py\"],\n  \"typeCheckingMode\": \"basic\",\n  \"reportUndefinedVariable\": \"error\"\n}\n")
    (soma-bridge-daemon--write-file
     source-file
     (format "def %s(x: int) -> int:\n    %s\n" symbol good-return))
    (soma-bridge-daemon--write-file
     (expand-file-name "__init__.py" tests)
     "")
    (soma-bridge-daemon--write-file
     test-file
     (format
      "import unittest\nfrom diagnostics_subject import %s\n\nclass CoreTest(unittest.TestCase):\n    def test_answer(self):\n        self.assertEqual(%s(20), 42)\n\nif __name__ == '__main__':\n    unittest.main()\n"
      symbol
      symbol))
    (setq initial-test
          (soma-bridge-daemon--run-process-in
           root "python3" "-m" "unittest" "discover" "-s" "tests"))
    (setq lsp--session
          (if (and (boundp 'lsp--session) lsp--session)
              (make-lsp-session
               :folders (cl-remove-duplicates
                         (cons canonical-root
                               (lsp-session-folders lsp--session))
                         :test #'equal)
               :folders-blocklist (lsp-session-folders-blocklist lsp--session)
               :server-id->folders
               (lsp-session-server-id->folders lsp--session)
               :folder->servers (lsp-session-folder->servers lsp--session)
               :metadata (lsp-session-metadata lsp--session))
            (make-lsp-session
             :folders (list canonical-root)
             :folders-blocklist nil)))
    (setq buffer (find-file-noselect source-file))
    (with-current-buffer buffer
      (setq default-directory canonical-root)
      (python-mode)
      (setq-local lsp-enabled-clients '(soma-pyright-diagnostics))
      (setq-local lsp-document-sync-method lsp--sync-full)
      (setq-local lsp-debounce-full-sync-notifications nil)
      (setq-local lsp-diagnostic-clean-after-change t)
      (lsp)
      (setq initialized
            (soma-bridge-daemon--wait-until
             (lambda ()
               (setq workspace
                     (seq-find
                      (lambda (candidate)
                        (eq (lsp--client-server-id
                             (lsp--workspace-client candidate))
                            'soma-pyright-diagnostics))
                      lsp--buffer-workspaces))
               (and workspace
                    (eq (lsp--workspace-status workspace) 'initialized)))
             30))
      (soma-bridge-daemon--drain 1.0)
      (soma-bridge-daemon--goto-lsp-symbol symbol)
      (setq observation
            (soma-gym-bridge-observe-lsp-source buffer 1600)))
    (push
     (soma-bridge-daemon--alist
      (cons "root" root)
      (cons "buffers" (list buffer)))
     soma-bridge-daemon--lsp-fixtures)
    (soma-bridge-daemon--alist
     (cons "language" "python")
     (cons "server_id" "soma-pyright-diagnostics")
     (cons "root" canonical-root)
     (cons "buffer_name" (buffer-name buffer))
     (cons "symbol" symbol)
     (cons "definition_file" source-file)
     (cons "test_file" test-file)
     (cons "test_dir" tests)
     (cons "good_return" good-return)
     (cons "broken_return" broken-return)
     (cons "expected_diagnostic_needle" expected-diagnostic-needle)
     (cons "diagnostic_sync_method" "full")
     (cons "diagnostic_debounce_full_sync" :json-false)
     (cons "diagnostic_clean_after_change" t)
     (cons "diagnostics_provider"
           (if (keywordp lsp-diagnostics-provider)
               (substring (symbol-name lsp-diagnostics-provider) 1)
             (soma-gym-bridge--json-symbol lsp-diagnostics-provider)))
     (cons "pyright_settings_source" "daemon-inline-lsp-pyright-minimal")
     (cons "pyright_initialized_configuration" t)
     (cons "definition_point"
           (with-current-buffer buffer (point)))
     (cons "workspace_initialized" (soma-bridge-daemon--bool initialized))
     (cons "pyright_path" (or (executable-find "pyright-langserver") ""))
     (cons "node_path" (or (executable-find "node") ""))
     (cons "python_path" (or (executable-find "python3") ""))
     (cons "transport"
           (if soma-bridge-daemon--lsp-jsonrpc-transport-suppressed
               "lsp-mode-stdio-process-filter"
             "emacs-json-rpc-connection"))
     (cons "initial_test" initial-test)
     (cons "observation" observation))))

(defun soma-bridge-daemon--cleanup-lsp-fixtures ()
  (soma-bridge-daemon--shutdown-lsp)
  (dolist (fixture soma-bridge-daemon--lsp-fixtures)
    (dolist (buffer (alist-get "buffers" fixture nil nil #'string=))
      (when (buffer-live-p buffer)
        (kill-buffer buffer)))
    (let ((root (alist-get "root" fixture nil nil #'string=)))
      (when (and root (file-directory-p root))
        (ignore-errors
          (delete-directory root t)))))
  (setq soma-bridge-daemon--lsp-fixtures nil))

(defun soma-bridge-daemon--write-jupyter-fixture-helper (path)
  (soma-bridge-daemon--write-file
   path
   (concat
    "import gc, json, os, pathlib, sys, time, traceback\n"
    "from jupyter_client import KernelManager\n"
    "\n"
    "tmp = pathlib.Path(sys.argv[1])\n"
    "token = sys.argv[2]\n"
    "ready_path = tmp / 'ready.json'\n"
    "stop_path = tmp / 'stop'\n"
    "km = KernelManager(kernel_name='python3')\n"
    "kc = None\n"
    "\n"
    "def write_ready(payload):\n"
    "    ready_path.write_text(json.dumps(payload, sort_keys=True) + '\\n', encoding='utf-8')\n"
    "    sys.stdout.write('SOMA_DAEMON_JUPYTER_READY:' + str(ready_path) + '\\n')\n"
    "    sys.stdout.flush()\n"
    "\n"
    "def execute(code, timeout=30):\n"
    "    msg_id = kc.execute(code, store_history=False, allow_stdin=False)\n"
    "    streams = []\n"
    "    errors = []\n"
    "    idle = False\n"
    "    deadline = time.time() + timeout\n"
    "    while time.time() < deadline and not idle:\n"
    "        try:\n"
    "            msg = kc.get_iopub_msg(timeout=0.2)\n"
    "        except Exception:\n"
    "            continue\n"
    "        if msg.get('parent_header', {}).get('msg_id') != msg_id:\n"
    "            continue\n"
    "        msg_type = msg.get('header', {}).get('msg_type')\n"
    "        content = msg.get('content', {})\n"
    "        if msg_type == 'stream':\n"
    "            streams.append(content.get('text', ''))\n"
    "        elif msg_type == 'error':\n"
    "            errors.append(content)\n"
    "        elif msg_type == 'status' and content.get('execution_state') == 'idle':\n"
    "            idle = True\n"
    "    reply = None\n"
    "    deadline = time.time() + timeout\n"
    "    while time.time() < deadline:\n"
    "        try:\n"
    "            candidate = kc.get_shell_msg(timeout=0.2)\n"
    "        except Exception:\n"
    "            continue\n"
    "        if candidate.get('parent_header', {}).get('msg_id') == msg_id:\n"
    "            reply = candidate\n"
    "            break\n"
    "    return {\n"
    "        'msg_id': msg_id,\n"
    "        'idle': idle,\n"
    "        'reply_parent': reply.get('parent_header', {}).get('msg_id') if reply else '',\n"
    "        'reply_status': reply.get('content', {}).get('status') if reply else '',\n"
    "        'streams': streams,\n"
    "        'errors': errors,\n"
    "    }\n"
    "\n"
    "try:\n"
    "    km.start_kernel(cwd=os.getcwd(), env=os.environ.copy())\n"
    "    kc = km.client()\n"
    "    kc.start_channels()\n"
    "    kc.wait_for_ready(timeout=30)\n"
    "    prelude = (\n"
    "        \"import os, sys\\n\"\n"
    "        \"SOMA_BENCH_RUN_ID = os.environ.get('SOMA_INSULA_RUN_ID', '')\\n\"\n"
    "        f\"SOMA_BENCH_KERNEL_TOKEN = {token!r}\\n\"\n"
    "        \"SOMA_BENCH_COUNTER = 0\\n\"\n"
    "        \"SOMA_TRAINING_EXECUTIONS = 0\\n\"\n"
    "    )\n"
    "    prelude_result = execute(prelude)\n"
    "    provisioner = getattr(km, 'provisioner', None)\n"
    "    kernel_pid = getattr(provisioner, 'pid', None)\n"
    "    kernel_exe = ''\n"
    "    if kernel_pid:\n"
    "        proc_exe = pathlib.Path('/proc') / str(kernel_pid) / 'exe'\n"
    "        if proc_exe.exists():\n"
    "            kernel_exe = os.path.realpath(str(proc_exe))\n"
    "    connection_file = pathlib.Path(km.connection_file)\n"
    "    if not connection_file.is_absolute():\n"
    "        connection_file = connection_file.resolve()\n"
    "    write_ready({\n"
    "        'ok': bool(prelude_result['idle'] and prelude_result['reply_status'] == 'ok'),\n"
    "        'connection_file': str(connection_file),\n"
    "        'helper_executable': sys.executable,\n"
    "        'helper_prefix': sys.prefix,\n"
    "        'kernel_pid': kernel_pid,\n"
    "        'kernel_exe': kernel_exe,\n"
    "        'kernel_token': token,\n"
    "        'run_id': os.environ.get('SOMA_INSULA_RUN_ID', ''),\n"
    "        'plan': os.environ.get('SOMA_INSULA_PLAN', ''),\n"
    "        'cwd': os.getcwd(),\n"
    "        'jupyter_runtime_dir': os.environ.get('JUPYTER_RUNTIME_DIR', ''),\n"
    "        'prelude': prelude_result,\n"
    "    })\n"
    "    while not stop_path.exists():\n"
    "        time.sleep(0.1)\n"
    "except Exception as exc:\n"
    "    write_ready({'ok': False, 'error': repr(exc), 'traceback': traceback.format_exc()[-3000:]})\n"
    "finally:\n"
    "    try:\n"
    "        if kc is not None:\n"
    "            kc.stop_channels()\n"
    "    finally:\n"
    "        try:\n"
    "            km.shutdown_kernel(now=True)\n"
    "        except Exception:\n"
    "            pass\n"
    "    kc = None\n"
    "    km = None\n"
    "    gc.collect()\n")))

(defun soma-bridge-daemon--start-jupyter-kernel-fixture (name)
  (let* ((tmp (make-temp-file (format "soma-daemon-jupyter-%s-" name) t))
         (helper (expand-file-name "kernel_fixture.py" tmp))
         (ready-path (expand-file-name "ready.json" tmp))
         (stop-path (expand-file-name "stop" tmp))
         (python (or (executable-find "python3")
                     (error "python3 is required in the Insula rootfs")))
         (token (format "%s-%s-%06x"
                        (or (getenv "SOMA_INSULA_RUN_ID") "no-run")
                        name
                        (random #x1000000)))
         (process-buffer
          (generate-new-buffer
           (format " *soma-daemon-jupyter-%s-fixture*" name)))
         process
         ready)
    (soma-bridge-daemon--write-jupyter-fixture-helper helper)
    (let ((default-directory "/workspace/soma/"))
      (setq process
            (make-process
             :name (format "soma-daemon-jupyter-%s-fixture" name)
             :buffer process-buffer
             :command (list python helper tmp token)
             :connection-type 'pipe
             :noquery t)))
    (let ((deadline (+ (float-time) 45)))
      (while (and (not ready) (< (float-time) deadline))
        (accept-process-output process 0.1)
        (when (file-readable-p ready-path)
          (setq ready
                (condition-case nil
                    (soma-gym-bridge--json-read-file ready-path)
                  (error nil))))
        (when (and (not ready)
                   (not (process-live-p process))
                   (not (file-readable-p ready-path)))
          (error "daemon Jupyter fixture exited before ready: %s"
                 (soma-bridge-daemon--process-output process)))))
    (unless ready
      (error "timed out waiting for daemon Jupyter fixture: %s"
             (soma-bridge-daemon--process-output process)))
    (unless (eq (alist-get "ok" ready nil nil #'string=) t)
      (error "daemon Jupyter fixture failed to initialize: %S output=%s"
             ready
             (soma-bridge-daemon--process-output process)))
    (soma-bridge-daemon--alist
     (cons "tmp" tmp)
     (cons "ready_path" ready-path)
     (cons "stop_path" stop-path)
     (cons "process" process)
     (cons "process_buffer" process-buffer)
     (cons "ready" ready)
     (cons "token" token))))

(defun soma-bridge-daemon--jupyter-source-code ()
  (concat
   "import json, os, sys\n"
   "SOMA_BENCH_COUNTER += 1\n"
   "print('SOMA_SELECTED_JSON:' + json.dumps({\n"
   "    'counter': SOMA_BENCH_COUNTER,\n"
   "    'kernel_token': SOMA_BENCH_KERNEL_TOKEN,\n"
   "    'run_id': SOMA_BENCH_RUN_ID,\n"
   "    'training_executions': SOMA_TRAINING_EXECUTIONS,\n"
   "}, sort_keys=True))\n"))

(defun soma-bridge-daemon--jupyter-display-update-code (display-id)
  (concat
   "import json\n"
   "from IPython.display import display, update_display\n"
   "SOMA_BENCH_COUNTER += 1\n"
   "_soma_display_id = " (json-encode display-id) "\n"
   "_soma_initial = {\n"
   "    'stage': 'initial',\n"
   "    'kernel_token': SOMA_BENCH_KERNEL_TOKEN,\n"
   "    'display_id': _soma_display_id,\n"
   "}\n"
   "_soma_updated = {\n"
   "    'stage': 'updated',\n"
   "    'kernel_token': SOMA_BENCH_KERNEL_TOKEN,\n"
   "    'display_id': _soma_display_id,\n"
   "}\n"
   "display({\n"
   "    'application/json': _soma_initial,\n"
   "    'text/plain': 'SOMA_DISPLAY_STAGE:initial ' + json.dumps(_soma_initial, sort_keys=True),\n"
   "}, raw=True, display_id=_soma_display_id)\n"
   "update_display({\n"
   "    'application/json': _soma_updated,\n"
   "    'text/plain': 'SOMA_DISPLAY_STAGE:updated ' + json.dumps(_soma_updated, sort_keys=True),\n"
   "}, raw=True, display_id=_soma_display_id)\n"
   "print('SOMA_SELECTED_JSON:' + json.dumps({\n"
   "    'counter': SOMA_BENCH_COUNTER,\n"
   "    'kernel_token': SOMA_BENCH_KERNEL_TOKEN,\n"
   "    'run_id': SOMA_BENCH_RUN_ID,\n"
   "    'training_executions': SOMA_TRAINING_EXECUTIONS,\n"
   "}, sort_keys=True))\n"))

(defun soma-bridge-daemon--jupyter-async-package-code (_display-id)
  (concat
   "import json, os, sys, time\n"
   "time.sleep(0.4)\n"
   "SOMA_BENCH_COUNTER += 1\n"
   "print('SOMA_ASYNC_PACKAGE_JSON:' + json.dumps({\n"
   "    'counter': SOMA_BENCH_COUNTER,\n"
   "    'kernel_token': SOMA_BENCH_KERNEL_TOKEN,\n"
   "    'run_id': SOMA_BENCH_RUN_ID,\n"
   "    'training_executions': SOMA_TRAINING_EXECUTIONS,\n"
   "    'phase': 'async-package',\n"
   "}, sort_keys=True))\n"))

(defun soma-bridge-daemon--jupyter-training-code ()
  (concat
   "SOMA_TRAINING_EXECUTIONS += 1\n"
   "print('SOMA_TRAINING_EXECUTED')\n"))

(defun soma-bridge-daemon--jupyter-client-session-id (client)
  (or (ignore-errors
        (jupyter-session-id (eieio-oref client 'session)))
      ""))

(defun soma-bridge-daemon--connect-existing-jupyter (kernel-fixture name)
  (let* ((ready (alist-get "ready" kernel-fixture nil nil #'string=))
         (connection-file (alist-get "connection_file" ready nil nil #'string=)))
    (jupyter-connect-repl
     connection-file
     (format "soma-daemon-%s" name)
     nil
     'jupyter-repl-client
     nil)))

(defun soma-bridge-daemon--prepare-jupyter-display (source-buffer beg end timeout)
  "Execute SOURCE-BUFFER's selected region through its REPL for fixture setup."
  (set-window-buffer (selected-window) source-buffer)
  (with-current-buffer source-buffer
    (goto-char beg)
    (set-window-point (selected-window) beg))
  (let* ((selected
          (with-current-buffer source-buffer
            (buffer-substring-no-properties beg end)))
         (observation
          (soma-gym-bridge-observe-jupyter-source source-buffer beg end 800))
         (preconditions
          (soma-bridge-daemon--alist
           (cons "buffer_name"
                 (soma-gym-bridge--get observation "buffer_name"))
           (cons "text_owner_name"
                 (soma-gym-bridge--get observation "text_owner_name"))
           (cons "buffer_file_name"
                 (soma-gym-bridge--get observation "buffer_file_name"))
           (cons "chars_tick"
                 (soma-gym-bridge--get observation "chars_tick"))
           (cons "point"
                 (soma-gym-bridge--get observation "point"))
           (cons "selected_window_buffer"
                 (soma-gym-bridge--get observation
                                       "selected_window_buffer"))
           (cons "modified"
                 (soma-gym-bridge--get observation "modified"))
           (cons "mark_active"
                 (soma-gym-bridge--get observation "mark_active"))
           (cons "narrowed"
                 (soma-gym-bridge--get observation "narrowed"))
           (cons "read_only"
                 (soma-gym-bridge--get observation "read_only"))
           (cons "minibuffer_active"
                 (soma-gym-bridge--get observation
                                       "minibuffer_active"))
           (cons "expected_text" selected)
           (cons "jupyter_connection_file"
                 (soma-gym-bridge--get observation
                                       "jupyter_connection_file"))
           (cons "jupyter_connection_file_rootfs_owned"
                 (soma-gym-bridge--get observation
                                       "jupyter_connection_file_rootfs_owned"))
           (cons "jupyter_kernel_token"
                 (soma-gym-bridge--get observation
                                       "jupyter_kernel_token"))
           (cons "jupyter_kernel_exe"
                 (soma-gym-bridge--get observation
                                       "jupyter_kernel_exe"))
           (cons "jupyter_kernel_exe_rootfs_owned"
                 (soma-gym-bridge--get observation
                                       "jupyter_kernel_exe_rootfs_owned"))
           (cons "jupyter_client_session_id"
                 (soma-gym-bridge--get observation
                                       "jupyter_client_session_id"))
           (cons "jupyter_repl_buffer"
                 (soma-gym-bridge--get observation
                                       "jupyter_repl_buffer"))))
         (result
          (soma-gym-bridge-jupyter-eval-region-via-repl
           source-buffer beg end preconditions timeout))
         (repl-buffer
          (soma-gym-bridge--get result "jupyter_repl_buffer")))
    (soma-bridge-daemon--alist
     (cons "evidence"
           (soma-bridge-daemon--alist
            (cons "operation" "fixture.jupyter.prepare_display")
            (cons "route" (soma-gym-bridge--get result "route"))
            (cons "state" (soma-gym-bridge--get result "state"))
            (cons "request_id" (soma-gym-bridge--get result "request_id"))
            (cons "reply_status" (soma-gym-bridge--get result "reply_status"))
            (cons "message_types"
                  (soma-gym-bridge--get result "message_types"))
            (cons "display_events"
                  (soma-gym-bridge--get result "display_events"))
            (cons "display_update_events"
                  (soma-gym-bridge--get result "display_update_events"))
            (cons "precondition_checks"
                  (soma-gym-bridge--get result "precondition_checks"))))
     (cons "repl_buffer" (or repl-buffer :json-false)))))

(defun soma-bridge-daemon--setup-jupyter-fixture
    (&optional name selected-fn prepare-display)
  (soma-bridge-daemon--load-jupyter)
  (let* ((fixture-name (or name "existing"))
         (kernel-fixture
          (soma-bridge-daemon--start-jupyter-kernel-fixture fixture-name))
         (ready (alist-get "ready" kernel-fixture nil nil #'string=))
         (connection-file (alist-get "connection_file" ready nil nil #'string=))
         (token (alist-get "token" kernel-fixture nil nil #'string=))
         (client
          (soma-bridge-daemon--connect-existing-jupyter
           kernel-fixture fixture-name))
         (source-buffer
          (generate-new-buffer
           (format "soma-daemon-jupyter-%s.py" fixture-name)))
         (display-id
          (and selected-fn
               (format "soma-display-%s" token)))
         (selected
          (if selected-fn
              (funcall selected-fn display-id)
            (soma-bridge-daemon--jupyter-source-code)))
         (training (soma-bridge-daemon--jupyter-training-code))
         beg
         end
         assoc-buffer
         display-preparation
         observation)
    (with-current-buffer source-buffer
      (insert "# soma-selected-begin\n")
      (setq beg (point))
      (insert selected)
      (setq end (point))
      (insert "# soma-selected-end\n\n")
      (insert "# soma-training-cell\n")
      (insert training)
      (let ((mode (ignore-errors (jupyter-kernel-language-mode client))))
        (cond
         ((and (symbolp mode) (fboundp mode))
          (funcall mode))
         ((fboundp 'python-mode)
          (python-mode))))
      (setq-local soma-gym-bridge-jupyter-connection-file connection-file)
      (setq-local soma-gym-bridge-jupyter-kernel-token token)
      (setq-local soma-gym-bridge-jupyter-kernel-pid
                  (alist-get "kernel_pid" ready nil nil #'string=))
      (setq-local soma-gym-bridge-jupyter-kernel-exe
                  (alist-get "kernel_exe" ready nil nil #'string=))
      (jupyter-repl-associate-buffer client)
      (setq assoc-buffer (buffer-name))
      (goto-char beg)
      (setq observation
            (soma-gym-bridge-observe-jupyter-source
             source-buffer beg end 800)))
    (when prepare-display
      (setq display-preparation
            (soma-bridge-daemon--prepare-jupyter-display
             source-buffer beg end 30.0)))
    (push
     (soma-bridge-daemon--alist
      (cons "kernel_fixture" kernel-fixture)
      (cons "client" client)
      (cons "source_buffer" source-buffer))
     soma-bridge-daemon--jupyter-fixtures)
    (soma-bridge-daemon--alist
     (cons "name" fixture-name)
     (cons "source_buffer" (buffer-name source-buffer))
     (cons "beg" beg)
     (cons "end" end)
     (cons "display_id" (or display-id :json-false))
     (cons "selected_sha256" (secure-hash 'sha256 selected))
     (cons "selected_code" selected)
     (cons "training_code" training)
     (cons "connection_file" connection-file)
     (cons "kernel_token" token)
     (cons "kernel_pid" (alist-get "kernel_pid" ready nil nil #'string=))
     (cons "kernel_exe" (alist-get "kernel_exe" ready nil nil #'string=))
     (cons "helper_executable"
           (alist-get "helper_executable" ready nil nil #'string=))
     (cons "helper_prefix"
           (alist-get "helper_prefix" ready nil nil #'string=))
     (cons "jupyter_runtime_dir"
           (alist-get "jupyter_runtime_dir" ready nil nil #'string=))
     (cons "client_session_id"
           (soma-bridge-daemon--jupyter-client-session-id client))
     (cons "associated_buffer" assoc-buffer)
     (cons "repl_buffer"
           (let ((buffer (ignore-errors (eieio-oref client 'buffer))))
             (if (buffer-live-p buffer) (buffer-name buffer) :json-false)))
     (cons "display_preparation"
           (or display-preparation :json-false))
     (cons "observation" observation))))

(defun soma-bridge-daemon--setup-jupyter-attach-fixture ()
  (soma-bridge-daemon--load-jupyter)
  (let* ((fixture-name "attach-existing")
         (kernel-fixture
          (soma-bridge-daemon--start-jupyter-kernel-fixture fixture-name))
         (ready (alist-get "ready" kernel-fixture nil nil #'string=))
         (connection-file (alist-get "connection_file" ready nil nil #'string=))
         (token (alist-get "token" kernel-fixture nil nil #'string=))
         (client
          (soma-bridge-daemon--connect-existing-jupyter
           kernel-fixture fixture-name))
         (source-buffer
          (generate-new-buffer
           (format "soma-daemon-jupyter-%s.py" fixture-name)))
         (selected (soma-bridge-daemon--jupyter-source-code))
         (training (soma-bridge-daemon--jupyter-training-code))
         beg end observation)
    (with-current-buffer source-buffer
      (insert "# soma-selected-begin\n")
      (setq beg (point))
      (insert selected)
      (setq end (point))
      (insert "# soma-selected-end\n\n")
      (insert "# soma-training-cell\n")
      (insert training)
      (let ((mode (ignore-errors (jupyter-kernel-language-mode client))))
        (cond
         ((and (symbolp mode) (fboundp mode))
          (funcall mode))
         ((fboundp 'python-mode)
          (python-mode))))
      (setq-local soma-gym-bridge-jupyter-connection-file connection-file)
      (setq-local soma-gym-bridge-jupyter-kernel-token token)
      (setq-local soma-gym-bridge-jupyter-kernel-pid
                  (alist-get "kernel_pid" ready nil nil #'string=))
      (setq-local soma-gym-bridge-jupyter-kernel-exe
                  (alist-get "kernel_exe" ready nil nil #'string=))
      (goto-char beg)
      (setq observation
            (soma-gym-bridge-observe-jupyter-source
             source-buffer beg end 800)))
    (push
     (soma-bridge-daemon--alist
      (cons "kernel_fixture" kernel-fixture)
      (cons "client" client)
      (cons "source_buffer" source-buffer))
     soma-bridge-daemon--jupyter-fixtures)
    (soma-bridge-daemon--alist
     (cons "name" fixture-name)
     (cons "source_buffer" (buffer-name source-buffer))
     (cons "beg" beg)
     (cons "end" end)
     (cons "selected_sha256" (secure-hash 'sha256 selected))
     (cons "selected_code" selected)
     (cons "training_code" training)
     (cons "connection_file" connection-file)
     (cons "kernel_token" token)
     (cons "kernel_pid" (alist-get "kernel_pid" ready nil nil #'string=))
     (cons "kernel_exe" (alist-get "kernel_exe" ready nil nil #'string=))
     (cons "helper_executable"
           (alist-get "helper_executable" ready nil nil #'string=))
     (cons "helper_prefix"
           (alist-get "helper_prefix" ready nil nil #'string=))
     (cons "jupyter_runtime_dir"
           (alist-get "jupyter_runtime_dir" ready nil nil #'string=))
     (cons "client_session_id"
           (soma-bridge-daemon--jupyter-client-session-id client))
     (cons "repl_buffer"
           (let ((buffer (ignore-errors (eieio-oref client 'buffer))))
             (if (buffer-live-p buffer) (buffer-name buffer) :json-false)))
     (cons "initial_observation" observation))))

(defun soma-bridge-daemon--setup-jupyter-host-shadow-fixture ()
  "Create an associated Jupyter source buffer with host-shadow metadata."
  (soma-bridge-daemon--load-jupyter)
  (let* ((fixture-name "host-shadow")
         (kernel-fixture
          (soma-bridge-daemon--start-jupyter-kernel-fixture fixture-name))
         (ready (alist-get "ready" kernel-fixture nil nil #'string=))
         (connection-file (alist-get "connection_file" ready nil nil #'string=))
         (token (alist-get "token" kernel-fixture nil nil #'string=))
         (client
          (soma-bridge-daemon--connect-existing-jupyter
           kernel-fixture fixture-name))
         (host-parent
          (expand-file-name
           ".scratch/emacs-gym/host-shadow-tools/"
           soma-bridge-daemon--repo-root))
         (host-dir
          (make-temp-file
           (expand-file-name "soma-daemon-host-jupyter-" host-parent)
           t))
         (host-connection-file
          (expand-file-name "kernel-host-shadow.json" host-dir))
         (host-kernel-exe
          (expand-file-name "python-host-shadow" host-dir))
         (source-python
          (or (executable-find "python3")
              (error "python3 is required in the Insula rootfs")))
         (source-buffer
          (generate-new-buffer
           (format "soma-daemon-jupyter-%s.py" fixture-name)))
         (selected (soma-bridge-daemon--jupyter-source-code))
         (training (soma-bridge-daemon--jupyter-training-code))
         beg end assoc-buffer observation)
    (make-directory host-parent t)
    (soma-bridge-daemon--write-file
     host-connection-file
     (json-encode
      (soma-bridge-daemon--alist
       (cons "source" "host-shadow")
       (cons "real_connection_file" connection-file))))
    (copy-file source-python host-kernel-exe t)
    (set-file-modes host-kernel-exe #o755)
    (with-current-buffer source-buffer
      (insert "# soma-selected-begin\n")
      (setq beg (point))
      (insert selected)
      (setq end (point))
      (insert "# soma-selected-end\n\n")
      (insert "# soma-training-cell\n")
      (insert training)
      (let ((mode (ignore-errors (jupyter-kernel-language-mode client))))
        (cond
         ((and (symbolp mode) (fboundp mode))
          (funcall mode))
         ((fboundp 'python-mode)
          (python-mode))))
      (setq-local soma-gym-bridge-jupyter-connection-file
                  host-connection-file)
      (setq-local soma-gym-bridge-jupyter-kernel-token token)
      (setq-local soma-gym-bridge-jupyter-kernel-pid
                  (alist-get "kernel_pid" ready nil nil #'string=))
      (setq-local soma-gym-bridge-jupyter-kernel-exe host-kernel-exe)
      (jupyter-repl-associate-buffer client)
      (setq assoc-buffer (buffer-name))
      (goto-char beg)
      (setq observation
            (soma-gym-bridge-observe-jupyter-source
             source-buffer beg end 800)))
    (push
     (soma-bridge-daemon--alist
      (cons "kernel_fixture" kernel-fixture)
      (cons "client" client)
      (cons "source_buffer" source-buffer)
      (cons "host_dir" host-dir))
     soma-bridge-daemon--jupyter-fixtures)
    (soma-bridge-daemon--alist
     (cons "name" fixture-name)
     (cons "source_buffer" (buffer-name source-buffer))
     (cons "beg" beg)
     (cons "end" end)
     (cons "selected_sha256" (secure-hash 'sha256 selected))
     (cons "selected_code" selected)
     (cons "training_code" training)
     (cons "real_connection_file" connection-file)
     (cons "connection_file" host-connection-file)
     (cons "kernel_token" token)
     (cons "kernel_pid" (alist-get "kernel_pid" ready nil nil #'string=))
     (cons "real_kernel_exe"
           (alist-get "kernel_exe" ready nil nil #'string=))
     (cons "kernel_exe" host-kernel-exe)
     (cons "host_dir" host-dir)
     (cons "helper_executable"
           (alist-get "helper_executable" ready nil nil #'string=))
     (cons "helper_prefix"
           (alist-get "helper_prefix" ready nil nil #'string=))
     (cons "jupyter_runtime_dir"
           (alist-get "jupyter_runtime_dir" ready nil nil #'string=))
     (cons "client_session_id"
           (soma-bridge-daemon--jupyter-client-session-id client))
     (cons "associated_buffer" assoc-buffer)
     (cons "repl_buffer"
           (let ((buffer (ignore-errors (eieio-oref client 'buffer))))
             (if (buffer-live-p buffer) (buffer-name buffer) :json-false)))
     (cons "initial_observation" observation))))

(defun soma-bridge-daemon--cleanup-jupyter-fixtures ()
  (dolist (fixture soma-bridge-daemon--jupyter-fixtures)
    (let* ((kernel-fixture
            (alist-get "kernel_fixture" fixture nil nil #'string=))
           (client (alist-get "client" fixture nil nil #'string=))
           (source-buffer
            (alist-get "source_buffer" fixture nil nil #'string=))
	     (host-dir
	      (alist-get "host_dir" fixture nil nil #'string=))
           (process
            (and kernel-fixture
                 (alist-get "process" kernel-fixture nil nil #'string=)))
           (process-buffer
            (and kernel-fixture
                 (alist-get "process_buffer" kernel-fixture nil nil #'string=)))
           (stop-path
            (and kernel-fixture
                 (alist-get "stop_path" kernel-fixture nil nil #'string=)))
           (tmp
            (and kernel-fixture
                 (alist-get "tmp" kernel-fixture nil nil #'string=))))
      (when client
        (ignore-errors
          (jupyter-disconnect client)))
      (when stop-path
        (ignore-errors
          (soma-bridge-daemon--write-file stop-path "stop\n")))
      (when process
        (let ((deadline (+ (float-time) 8)))
          (while (and (process-live-p process)
                      (< (float-time) deadline))
            (accept-process-output process 0.1)))
        (when (process-live-p process)
          (delete-process process)))
      (when (buffer-live-p process-buffer)
        (kill-buffer process-buffer))
      (when (buffer-live-p source-buffer)
        (kill-buffer source-buffer))
      (when (and host-dir (file-directory-p host-dir))
	(ignore-errors
	  (delete-directory host-dir t)))
      (when (and tmp (file-directory-p tmp))
        (ignore-errors
          (delete-directory tmp t)))))
  (setq soma-bridge-daemon--jupyter-fixtures nil))

(add-hook 'kill-emacs-hook #'soma-bridge-daemon--cleanup-jupyter-fixtures)
(add-hook 'kill-emacs-hook #'soma-bridge-daemon--cleanup-lsp-fixtures)

(defun soma-bridge-daemon--magit-preconditions (buffer)
  (with-current-buffer buffer
    (soma-bridge-daemon--alist
     (cons "buffer_name" (buffer-name buffer))
     (cons "selected_window_buffer" (buffer-name buffer))
     (cons "chars_tick" (buffer-chars-modified-tick))
     (cons "point" (point))
     (cons "mark_active" (soma-bridge-daemon--bool mark-active))
     (cons "narrowed" (soma-bridge-daemon--bool (buffer-narrowed-p)))
     (cons "read_only" (soma-bridge-daemon--bool buffer-read-only))
     (cons "minibuffer_active" :json-false))))

(defun soma-bridge-daemon--setup-magit-fixture (&optional target-shape)
  (soma-bridge-daemon--load-magit)
  (let* ((repo (soma-bridge-daemon--make-magit-repo target-shape))
         (hook-log (expand-file-name "soma-daemon-magit-hooks.log" repo))
         (magit-display-buffer-function #'soma-bridge-daemon--display-buffer)
         (magit-display-buffer-noselect nil)
         (magit-auto-revert-mode nil)
         (buffer (let ((default-directory (file-name-as-directory repo)))
                   (magit-status-setup-buffer repo)))
         section
         target-section
         observation
         preconditions)
    (setq magit-post-stage-hook
          '(soma-bridge-daemon--capture-post-stage-hook))
    (setq magit-refresh-buffer-hook
          '(soma-bridge-daemon--capture-refresh-hook))
    (setq soma-bridge-daemon--magit-hook-log
          hook-log)
    (with-current-buffer buffer
      (setq-local soma-bridge-daemon--magit-buffer-hook-log hook-log)
      (setq section
            (soma-bridge-daemon--find-hunk-section
             buffer "target.txt" "target-05-selected"))
      (unless section
        (error "could not locate daemon Magit target hunk"))
      (goto-char
       (soma-bridge-daemon--section-target-point
        section "target-05-selected"))
      (when (eq (window-buffer (selected-window)) buffer)
        (set-window-point (selected-window) (point)))
      (setq observation (soma-gym-bridge-observe-buffer buffer))
      (setq preconditions
            (soma-bridge-daemon--magit-preconditions buffer))
      (setq target-section
            (soma-bridge-daemon--section-observation
             section
             (alist-get "point" preconditions nil nil #'string=))))
    (soma-bridge-daemon--alist
     (cons "repo" repo)
     (cons "status_buffer" (buffer-name buffer))
     (cons "hook_log" soma-bridge-daemon--magit-hook-log)
     (cons "target_file" "target.txt")
     (cons "target_section" target-section)
     (cons "preconditions" preconditions)
     (cons "observation" observation))))

(defun soma-bridge-daemon-start-from-env ()
  "Start the JSONL bridge in a live daemon using benchmark environment."
  (let* ((socket (soma-bridge-daemon--env-required
                  "SOMA_BRIDGE_DAEMON_SOCKET"))
         (ready-path (soma-bridge-daemon--env-required
                      "SOMA_BRIDGE_DAEMON_READY"))
         (buffer-name (or (getenv "SOMA_BRIDGE_DAEMON_BUFFER")
                          "soma-bridge-daemon-target"))
         (buffer-text (or (getenv "SOMA_BRIDGE_DAEMON_TEXT")
                          "daemon bridge buffer\n"))
         buffer
         magit-fixture
         terminal-magit-fixture
         transient-magit-fixture
         transient-stage-magit-fixture
         transient-diff-magit-fixture
         prompt-magit-fixture
         stale-magit-fixture
         async-magit-fixture
         semantic-magit-fixture
         semantic-file-magit-fixture
         semantic-stale-magit-fixture
         bypass-magit-fixture
         comint-fixture
         comint-interrupt-fixture
         comint-api-interrupt-fixture
         jupyter-fixture
         jupyter-attach-fixture
	 jupyter-host-shadow-fixture
         jupyter-display-fixture
         async-jupyter-fixture
         lsp-fixture
         lsp-diagnostics-fixture
         completion-fixture
         org-report-fixture
         org-heading-bypass-fixture
         org-bypass-fixture
         org-duplicate-id-fixture
         org-subtree-bypass-fixture
         dired-rename-fixture
         dired-delete-fixture
         project-host-shadow-fixture
         profile-smoke-fixture
         start-result
         observation)
    (soma-gym-bridge-reset-transport)
    (setq soma-gym-bridge-async-command-dispatch t)
    (setq soma-gym-bridge-operation-allowlist
          '("api.list_capabilities"
            "api.describe_operation"
            "emacs.observe_session"
            "emacs.observe_buffer"
            "emacs.observe_frames"
            "emacs.observe_interaction"
            "buffer.select"
            "project.observe"
            "project.search"
            "comint.observe"
            "comint.submit_input"
            "comint.await_output"
            "comint.interrupt"
            "magit.observe_sections"
            "magit.stage_hunk"
            "magit.stage_file"
            "jupyter.observe_source"
            "jupyter.attach_existing"
            "jupyter.observe_display"
            "jupyter.eval_region"
            "jupyter.await_request"
            "jupyter.eval_region_via_repl"
            "jupyter.query_state"
            "jupyter.restart_kernel"
            "lsp.observe_source"
            "lsp.observe_diagnostics"
            "lsp.rename_symbol"
            "emacs.goto_char"
            "buffer.read"
            "buffer.replace_range"
            "buffer.apply_patch"
            "buffer.save"
            "completion.observe"
            "completion.accept"
            "org.observe_tree"
            "org.update_heading"
            "org.edit_src_block"
            "org.exit_src_edit"
            "org.execute_babel_block"
            "org.export_html"
            "org.parent_text_write"
            "dired.observe"
            "dired.rename_no_overwrite"
            "dired.delete_marked"
            "filesystem.rename"
            "command.dispatch"
            "request.status"
            "request.await"
            "episode.finish"
            "bridge.defer_echo"
            "bridge.prompt_echo"
            "request.respond"
            "direct-call"
            "elisp.eval"
            "process.write"
            "git.index_write"))
    (add-to-list 'soma-gym-bridge-command-allowlist
                 'soma-bridge-daemon-terminal-prompt-insert)
    (add-to-list 'soma-gym-bridge-command-allowlist
                 'magit-dispatch)
    (add-to-list 'soma-gym-bridge-command-allowlist
                 'magit-diff)
    (add-to-list 'soma-gym-bridge-command-allowlist
                 'magit-stage-files)
    (add-to-list 'soma-gym-bridge-command-allowlist
                 'magit-refresh)
    (add-to-list 'soma-gym-bridge-command-allowlist
                 'lsp-rename)
    (setq buffer (get-buffer-create buffer-name))
    (with-current-buffer buffer
      (erase-buffer)
      (insert buffer-text)
      (goto-char (point-min))
      (set-buffer-modified-p nil))
    (setq magit-fixture (soma-bridge-daemon--setup-magit-fixture))
    (setq terminal-magit-fixture (soma-bridge-daemon--setup-magit-fixture))
    (setq transient-magit-fixture (soma-bridge-daemon--setup-magit-fixture))
    (setq transient-stage-magit-fixture
          (soma-bridge-daemon--setup-magit-fixture))
    (setq transient-diff-magit-fixture
          (soma-bridge-daemon--setup-magit-fixture))
    (setq prompt-magit-fixture (soma-bridge-daemon--setup-magit-fixture))
    (setq stale-magit-fixture
          (soma-bridge-daemon--setup-magit-fixture 'one-hunk))
    (setq async-magit-fixture (soma-bridge-daemon--setup-magit-fixture))
    (setq semantic-magit-fixture
          (soma-bridge-daemon--setup-magit-fixture))
    (setq semantic-file-magit-fixture
          (soma-bridge-daemon--setup-magit-fixture 'one-hunk))
    (setq semantic-stale-magit-fixture
          (soma-bridge-daemon--setup-magit-fixture 'one-hunk))
    (setq bypass-magit-fixture
          (soma-bridge-daemon--setup-magit-fixture 'one-hunk))
    (setq comint-fixture (soma-bridge-daemon--setup-comint-fixture))
    (setq comint-interrupt-fixture
          (soma-bridge-daemon--setup-comint-interrupt-fixture))
    (setq comint-api-interrupt-fixture
          (soma-bridge-daemon--setup-comint-interrupt-fixture))
    (setq jupyter-fixture (soma-bridge-daemon--setup-jupyter-fixture))
    (setq jupyter-attach-fixture
          (soma-bridge-daemon--setup-jupyter-attach-fixture))
    (setq jupyter-host-shadow-fixture
	  (soma-bridge-daemon--setup-jupyter-host-shadow-fixture))
    (setq jupyter-display-fixture
          (soma-bridge-daemon--setup-jupyter-fixture
           "display-update"
           #'soma-bridge-daemon--jupyter-display-update-code))
    (setq async-jupyter-fixture
          (soma-bridge-daemon--setup-jupyter-fixture
           "async-package"
           #'soma-bridge-daemon--jupyter-async-package-code))
    (setq lsp-fixture (soma-bridge-daemon--setup-lsp-fixture))
    (setq lsp-diagnostics-fixture
          (soma-bridge-daemon--setup-lsp-diagnostics-fixture))
    (setq completion-fixture (soma-bridge-daemon--setup-completion-fixture))
    (setq org-report-fixture (soma-bridge-daemon--setup-org-report-fixture))
    (setq org-heading-bypass-fixture
          (soma-bridge-daemon--setup-org-report-fixture "heading-bypass"))
    (setq org-bypass-fixture
          (soma-bridge-daemon--setup-org-report-fixture "bypass"))
    (setq org-duplicate-id-fixture
          (soma-bridge-daemon--setup-org-duplicate-id-fixture))
    (setq org-subtree-bypass-fixture
          (soma-bridge-daemon--setup-org-subtree-bypass-fixture))
    (setq dired-rename-fixture (soma-bridge-daemon--setup-dired-rename-fixture))
    (setq dired-delete-fixture (soma-bridge-daemon--setup-dired-delete-fixture))
    (setq project-host-shadow-fixture
          (soma-bridge-daemon--setup-project-host-shadow-fixture))
    (setq profile-smoke-fixture
          (soma-bridge-daemon--setup-profile-smoke-fixture))
    (delete-other-windows)
    (set-window-buffer (selected-window) buffer)
    (with-current-buffer buffer
      (goto-char (point-min))
      (set-window-point (selected-window) (point)))
    (setq start-result (soma-gym-bridge-start-jsonl-server socket))
    (setq observation (soma-gym-bridge-observe-buffer buffer))
    (soma-bridge-daemon--write-json
     ready-path
     (soma-bridge-daemon--alist
      (cons "ok" t)
      (cons "daemonp" (soma-bridge-daemon--bool (daemonp)))
      (cons "server_name" server-name)
      (cons "emacs_pid" (emacs-pid))
      (cons "socket" socket)
      (cons "buffer_name" buffer-name)
      (cons "buffer_text" buffer-text)
      (cons "magit_fixture" magit-fixture)
      (cons "terminal_magit_fixture" terminal-magit-fixture)
      (cons "transient_magit_fixture" transient-magit-fixture)
      (cons "transient_stage_magit_fixture" transient-stage-magit-fixture)
      (cons "transient_diff_magit_fixture" transient-diff-magit-fixture)
      (cons "prompt_magit_fixture" prompt-magit-fixture)
      (cons "stale_magit_fixture" stale-magit-fixture)
      (cons "async_magit_fixture" async-magit-fixture)
      (cons "semantic_magit_fixture" semantic-magit-fixture)
      (cons "semantic_file_magit_fixture" semantic-file-magit-fixture)
      (cons "semantic_stale_magit_fixture" semantic-stale-magit-fixture)
      (cons "bypass_magit_fixture" bypass-magit-fixture)
      (cons "comint_fixture" comint-fixture)
      (cons "comint_interrupt_fixture" comint-interrupt-fixture)
      (cons "comint_api_interrupt_fixture" comint-api-interrupt-fixture)
      (cons "jupyter_fixture" jupyter-fixture)
      (cons "jupyter_attach_fixture" jupyter-attach-fixture)
	      (cons "jupyter_host_shadow_fixture" jupyter-host-shadow-fixture)
      (cons "jupyter_display_fixture" jupyter-display-fixture)
      (cons "async_jupyter_fixture" async-jupyter-fixture)
      (cons "lsp_fixture" lsp-fixture)
      (cons "lsp_diagnostics_fixture" lsp-diagnostics-fixture)
      (cons "completion_fixture" completion-fixture)
      (cons "org_report_fixture" org-report-fixture)
      (cons "org_heading_bypass_fixture" org-heading-bypass-fixture)
      (cons "org_bypass_fixture" org-bypass-fixture)
      (cons "org_duplicate_id_fixture" org-duplicate-id-fixture)
      (cons "org_subtree_bypass_fixture" org-subtree-bypass-fixture)
      (cons "dired_rename_fixture" dired-rename-fixture)
      (cons "dired_delete_fixture" dired-delete-fixture)
      (cons "project_host_shadow_fixture" project-host-shadow-fixture)
      (cons "profile_smoke_fixture" profile-smoke-fixture)
      (cons "start_result" start-result)
      (cons "observation" observation)
      (cons "soma_insula_run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))))
    ready-path))

(soma-bridge-daemon-start-from-env)

;;; daemon-start.el ends here
