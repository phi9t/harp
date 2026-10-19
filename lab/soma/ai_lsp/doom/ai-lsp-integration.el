;;; ai-lsp-integration.el --- Structured live AI/LSP verification -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'seq)
(require 'subr-x)
(require 'ai-lsp-support)
(require 'ai-lsp-adapter)

(defvar copilot-mode)
(defvar tabbymacs-mode)
(defvar lsp-auto-guess-root)
(defvar lsp-keep-workspace-alive)
(defvar lsp-mode)

(declare-function lsp-disconnect "lsp-mode" ())

(defconst devx-ai-test-copilot-log-buffers
  '("*copilot stderr*" "*copilot-language-server-log*" "*Messages*"))

(defvar devx-ai-test--current-failures nil)
(defvar devx-ai-test--observed-processes nil)

(defun devx-ai-test--wait-until (predicate seconds)
  "Wait at most SECONDS for PREDICATE while servicing timers and processes."
  (let ((deadline (+ (float-time) seconds)))
    (while (and (< (float-time) deadline)
                (not (funcall predicate)))
      (accept-process-output nil 0.05))
    (funcall predicate)))

(defun devx-ai-test--write-fixture (root mode)
  "Write the fixture for MODE below ROOT and return its source path."
  (let ((files (alist-get mode devx-ai-test-fixtures)) source)
    (make-directory (expand-file-name ".git" root) t)
    (dolist (entry files)
      (let ((path (expand-file-name (car entry) root)))
        (make-directory (file-name-directory path) t)
        (with-temp-file path (insert (cdr entry)))
        (when (string-match-p "\\.\\(rs\\|py\\|cpp\\|cu\\)\\'" path)
          (setq source path))))
    source))

(defun devx-ai-test--log-tail (buffer-name)
  "Return a bounded, credential-safe diagnostic tail from BUFFER-NAME."
  (when-let ((buffer (get-buffer buffer-name)))
    (with-current-buffer buffer
      (devx-ai-test-complete-line-tail
       (buffer-substring-no-properties (point-min) (point-max)) 4000))))

(defun devx-ai-test--copilot-log-tail (&optional buffer-names)
  "Return a bounded, credential-safe tail from Copilot BUFFER-NAMES."
  (let (tails)
    (dolist (buffer-name (or buffer-names devx-ai-test-copilot-log-buffers))
      (when-let ((buffer (get-buffer buffer-name)))
        (with-current-buffer buffer
          (let ((tail
                 (buffer-substring-no-properties (point-min) (point-max))))
            (when (equal buffer-name "*Messages*")
              (setq tail
                    (string-join
                     (seq-filter
                      (lambda (line)
                        (let ((case-fold-search t))
                          (string-match-p "copilot" line)))
                      (split-string tail "\n" t))
                     "\n")))
            (unless (string-empty-p tail)
              (push (format "%s:\n%s" buffer-name tail) tails))))))
    (devx-ai-test-complete-line-tail
     (if tails (string-join (nreverse tails) "\n") "No Copilot log")
     4000)))

(defun devx-ai-test--workspace-executable-match-p (record executable)
  "Return non-nil when RECORD names the canonical EXECUTABLE."
  (let ((observed (plist-get record :command)))
    (and (stringp observed)
         (stringp executable)
         (equal (file-truename observed) (file-truename executable)))))

(defun devx-ai-test--workspace-core-valid-p (records expected-server)
  "Return non-nil for one live initialized EXPECTED-SERVER workspace."
  (and (= 1 (length records))
       (let ((record (car records)))
         (and (eq expected-server (plist-get record :id))
              (eq 'initialized (plist-get record :status))
              (eq t (plist-get record :alive))))))

(defun devx-ai-test--safe-workspace-summaries (records executable)
  "Return JSON-safe summaries for adapter workspace RECORDS."
  (mapcar
   (lambda (record)
     (list :server (and (plist-get record :id)
                        (symbol-name (plist-get record :id)))
           :status (and (plist-get record :status)
                        (symbol-name (plist-get record :status)))
           :executable (plist-get record :command)
           :alive (if (eq t (plist-get record :alive)) t :json-false)
           :executable-match
           (if (devx-ai-test--workspace-executable-match-p record executable)
               t
             :json-false)))
   records))

(defun devx-ai-test--lsp-state-summary (executable)
  "Return bounded LSP executable and workspace state diagnostics."
  (let ((workspaces
         (mapcar
          (lambda (record)
            (format "%S/%S/alive=%S/executable-match=%S"
                    (plist-get record :id)
                    (plist-get record :status)
                    (plist-get record :alive)
                    (devx-ai-test--workspace-executable-match-p
                     record executable)))
          (devx-ai-adapter-workspaces))))
    (let ((summary
           (format "resolved executable: %s; observed workspaces: %s"
                   (or executable "unavailable")
                   (if workspaces (string-join workspaces ", ") "none"))))
      (if (> (length summary) 4000) (substring summary 0 4000) summary))))

(defun devx-ai-test--capture-workspace-processes (records)
  "Capture each process in adapter workspace RECORDS before disconnect."
  (dolist (record records)
    (let ((process (plist-get record :process)))
      (when (and (processp process)
                 (not (seq-some
                       (lambda (observed)
                         (eq process (plist-get observed :process)))
                       devx-ai-test--observed-processes)))
        (push (list :server (plist-get record :id)
                    :pid (ignore-errors (process-id process))
                    :process process)
              devx-ai-test--observed-processes)))))

(defun devx-ai-test--certification-mutation ()
  "Return the guarded certification mutation, if enabled."
  (and (equal "1" (getenv "DEVX_AI_LSP_CERTIFY"))
       (getenv "DEVX_AI_LSP_MUTATION")))

(defun devx-ai-test-effective-language-cases ()
  "Return language cases with any guarded certification mutation applied."
  (let ((cases (copy-tree devx-ai-test-language-cases)))
    (when (equal "wrong-rust-server" (devx-ai-test--certification-mutation))
      (setf (plist-get (car cases) :server) 'rls))
    cases))

(defun devx-ai-test--preflight-error (servers boundary message)
  "Return dependency errors for SERVERS with BOUNDARY and safe MESSAGE."
  (mapcar (lambda (server)
            (list server :error (devx-ai-test-sanitize-diagnostic message)
                  :boundary boundary))
          servers))

(defun devx-ai-test--preflight ()
  "Return process-wide compatibility and per-server dependency resolution."
  (let ((servers '(rust-analyzer pyright clangd))
        (mutation (devx-ai-test--certification-mutation)))
    (cond
     ((and mutation
           (not (member mutation '("wrong-rust-server" "missing-clangd"))))
      (devx-ai-test--preflight-error
       servers "mutation" (format "Unknown certification mutation: %s" mutation)))
     (t
      (let ((shared-error
             (condition-case err
                 (progn (devx-ai-adapter-check-compatibility) nil)
               (error
                (devx-ai-test-sanitize-diagnostic
                 (error-message-string err))))))
        (if shared-error
            (devx-ai-test--preflight-error
             servers "compatibility"
             (format "Shared verifier dependency failed: %s" shared-error))
          (mapcar
           (lambda (dependency)
             (pcase-let ((`(,server ,feature ,executable) dependency))
               (if (and (eq server 'clangd)
                        (equal mutation "missing-clangd"))
                   (list server :error
                         "Missing clangd (certification mutation)"
                         :boundary "dependency")
                 (condition-case err
                     (let ((path (devx-ai-adapter-resolve-server server)))
                       (if path
                           (list server :path path)
                         (list server :error
                               (format
                                "Missing %s; install it before verification"
                                executable)
                               :boundary "dependency")))
                   (error
                    (list server :error
                          (format "Unable to load %S for %s: %s"
                                  feature executable
                                  (devx-ai-test-sanitize-diagnostic
                                   (error-message-string err)))
                          :boundary "dependency"))))))
           '((rust-analyzer lsp-rust "rust-analyzer")
             (pyright lsp-pyright "pyright-langserver")
             (clangd lsp-clangd "clangd")))))))))

(defun devx-ai-test--resignal-copilot-error (error-data)
  "Classify ERROR-DATA as external transport failure or local Lisp error."
  (let ((message
         (alist-get
          'jsonrpc-error-message
          (seq-filter #'consp (cdr error-data)))))
    (if (and (eq (car error-data) 'jsonrpc-error)
             (member message '("Timed out")))
        (signal 'devx-ai-test-external-blocker
                (list (devx-ai-test-sanitize-diagnostic message)))
      (signal (car error-data) (cdr error-data)))))

(defun devx-ai-test--copilot-call (function)
  "Call FUNCTION, classifying only Copilot transport unavailability."
  (condition-case err
      (funcall function)
    (error (devx-ai-test--resignal-copilot-error err))))

(defun devx-ai-test--require-copilot-auth (status)
  "Accept a valid OK STATUS or signal its precise failure class."
  (let ((value (and (devx-ai-test-plist-p status)
                    (plist-member status :status)
                    (plist-get status :status))))
    (unless (stringp value)
      (error "Malformed Copilot checkStatus response type: %s"
             (symbol-name (type-of status))))
    (cond
     ((equal value "OK") t)
     ((member value '("NotSignedIn" "Unauthorized" "ServiceUnavailable"))
      (signal 'devx-ai-test-external-blocker
              (list "Copilot authentication unavailable")))
     (t
      (error "Unexpected Copilot checkStatus value: %s" value)))))

(defun devx-ai-test--record-failure
    (language boundary expected observed executable workspaces diagnostic
              classification)
  "Append one failure to the dynamically scoped current case."
  (push (devx-ai-test-make-failure
         language boundary expected observed executable workspaces diagnostic
         classification)
        devx-ai-test--current-failures))

(defun devx-ai-test-verify-case (case fixture-root dependency)
  "Verify one language CASE using its preflight DEPENDENCY and return a record."
  (let* ((name (plist-get case :name))
         (expected-mode (plist-get case :mode))
         (expected-language-id (plist-get case :language-id))
         (expected-server (plist-get case :server))
         (executable (plist-get dependency :path))
         (dependency-error (plist-get dependency :error))
         (gate-statuses nil)
         observed-server observed-executable observed-language-id workspaces)
    (cl-labels
        ((set-gate (gate status)
           (setf (alist-get gate gate-statuses nil nil #'equal) status))
         (record (status)
           (devx-ai-test-make-language-record
            case executable observed-server observed-executable
            observed-language-id gate-statuses status))
         (fail (gate boundary expected observed classification diagnostic)
           (set-gate gate classification)
           (devx-ai-test--record-failure
            name boundary expected observed executable
            (devx-ai-test--safe-workspace-summaries workspaces executable)
            diagnostic classification)
           (throw 'devx-ai-case-result (record classification))))
      (catch 'devx-ai-case-result
        (when dependency-error
          (fail "lsp-executable"
                (or (plist-get dependency :boundary) "dependency")
                (format "executable for %s"
                        (symbol-name (plist-get case :dependency)))
                nil "FAIL" dependency-error))
        (let* ((project-root (expand-file-name (downcase name) fixture-root))
               (source (devx-ai-test--write-fixture project-root expected-mode))
               (buffer (find-file-noselect source)))
          (with-current-buffer buffer
            (if (eq major-mode expected-mode)
                (set-gate "mode" "PASS")
              (fail "mode" "mode" (symbol-name expected-mode)
                    (symbol-name major-mode) "FAIL" "Major mode mismatch"))
            (let ((copilot-enabled (and (bound-and-true-p copilot-mode) t))
                  (tabby-enabled (and (bound-and-true-p tabbymacs-mode) t)))
              (if (and copilot-enabled (not tabby-enabled))
                  (set-gate "backend-exclusivity" "PASS")
                (fail "backend-exclusivity" "backend-exclusivity"
                      "copilot enabled; tabbymacs disabled"
                      (list :copilot (if copilot-enabled t :json-false)
                            :tabbymacs (if tabby-enabled t :json-false))
                      "FAIL" "Inline completion backends are not exclusive")))
            (condition-case err
                (setq observed-language-id
                      (devx-ai-adapter-copilot-language-id))
              (error
               (fail "copilot-language-id" "copilot-language-id"
                     expected-language-id nil "FAIL"
                     (error-message-string err))))
            (if (equal expected-language-id observed-language-id)
                (set-gate "copilot-language-id" "PASS")
              (fail "copilot-language-id" "copilot-language-id"
                    expected-language-id observed-language-id "FAIL"
                    "Copilot language identifier mismatch"))
            (condition-case err
                (devx-ai-adapter-start-lsp)
              (error
               (fail "lsp-workspace" "lsp-workspace"
                     (symbol-name expected-server) nil "FAIL"
                     (error-message-string err))))
            (let ((ready
                   (devx-ai-test--wait-until
                    (lambda ()
                      (setq workspaces (devx-ai-adapter-workspaces))
                      (devx-ai-test--capture-workspace-processes workspaces)
                      (and (devx-ai-test--workspace-core-valid-p
                            workspaces expected-server)
                           (devx-ai-test--workspace-executable-match-p
                            (car workspaces) executable)))
                    30)))
              (unless workspaces
                (setq workspaces (devx-ai-adapter-workspaces))
                (devx-ai-test--capture-workspace-processes workspaces))
              (when workspaces
                (setq observed-server (plist-get (car workspaces) :id)
                      observed-executable (plist-get (car workspaces) :command)))
              (if (devx-ai-test--workspace-core-valid-p
                   workspaces expected-server)
                  (set-gate "lsp-workspace" "PASS")
                (set-gate "lsp-workspace" "FAIL"))
              (if (and (= 1 (length workspaces))
                       (devx-ai-test--workspace-executable-match-p
                        (car workspaces) executable))
                  (set-gate "lsp-executable" "PASS")
                (set-gate "lsp-executable" "FAIL"))
              (unless ready
                (let ((boundary
                       (if (equal "PASS"
                                  (cdr (assoc "lsp-workspace" gate-statuses)))
                           "lsp-executable"
                         "lsp-workspace")))
                  (fail boundary boundary
                        (if (equal boundary "lsp-executable")
                            executable
                          (symbol-name expected-server))
                        (if (equal boundary "lsp-executable")
                            observed-executable
                          (vconcat
                           (devx-ai-test--safe-workspace-summaries
                            workspaces executable)))
                        "FAIL"
                        (concat (devx-ai-test--lsp-state-summary executable)
                                "\n"
                                (or (devx-ai-test--log-tail "*lsp-log*")
                                    "No LSP log"))))))
            (if (devx-ai-adapter-copilot-alive-p)
                (set-gate "copilot-connection" "PASS")
              (fail "copilot-connection" "copilot-connection" "alive" "dead"
                    "BLOCKED_EXTERNAL" (devx-ai-test--copilot-log-tail)))
            (condition-case err
                (progn
                  (devx-ai-test--require-copilot-auth
                   (devx-ai-test--copilot-call
                    (lambda () (devx-ai-adapter-copilot-status 15))))
                  (set-gate "copilot-auth" "PASS"))
              (devx-ai-test-external-blocker
               (fail "copilot-auth" "copilot-auth" "OK" "unavailable"
                     "BLOCKED_EXTERNAL"
                     (concat (error-message-string err) "\n"
                             (devx-ai-test--copilot-log-tail))))
              (error
               (fail "copilot-auth" "copilot-auth" "valid checkStatus"
                     (list :observed-type (symbol-name (type-of err)))
                     "FAIL" (error-message-string err))))
            (condition-case err
                (let ((response
                       (devx-ai-test--copilot-call
                        (lambda ()
                          (devx-ai-adapter-inline-completion 30)))))
                  (if (devx-ai-test-valid-completion-response-p response)
                      (set-gate "inline-completion" "PASS")
                    (fail "inline-completion" "inline-completion-schema"
                          "valid inline completion response"
                          (list :observed-type
                                (symbol-name (type-of response)))
                          "FAIL" "Malformed inline completion response")))
              (devx-ai-test-external-blocker
               (fail "inline-completion" "inline-completion-transport"
                     "available Copilot transport" "unavailable"
                     "BLOCKED_EXTERNAL"
                     (concat (error-message-string err) "\n"
                             (devx-ai-test--copilot-log-tail))))
              (error
               (fail "inline-completion" "inline-completion"
                     "valid inline completion response"
                     (list :observed-type (symbol-name (type-of err)))
                     "FAIL" (error-message-string err))))
            (record "PASS")))))))

(defun devx-ai-test--fallback-failure (case dependency record diagnostic)
  "Return a complete failure when a case seam omitted structured details."
  (let* ((status (plist-get record :status))
         (classification
          (if (equal status "BLOCKED_EXTERNAL") "BLOCKED_EXTERNAL" "FAIL")))
    (devx-ai-test-make-failure
     (plist-get case :name)
     (or (plist-get record :boundary) "verification")
     nil (list :status status) (plist-get dependency :path) nil diagnostic
     classification)))

(defun devx-ai-test-run-cases (cases fixture-root dependencies)
  "Run every CASE and return ordered records, failures, and aggregate status."
  (let (records failures)
    (dolist (case cases)
      (let* ((dependency-key (plist-get case :dependency))
             (dependency
              (or (cdr (assq dependency-key dependencies))
                  (list :error
                        (format "Missing preflight result for %s"
                                (symbol-name dependency-key))
                        :boundary "preflight")))
             (devx-ai-test--current-failures nil)
             record)
        (condition-case err
            (setq record
                  (devx-ai-test-verify-case case fixture-root dependency))
          (error
           (setq record
                 (devx-ai-test-make-language-record
                  case (plist-get dependency :path) nil nil nil nil "FAIL"))
           (devx-ai-test--record-failure
            (plist-get case :name) "lisp"
            nil (list :observed-type (symbol-name (type-of err)))
            (plist-get dependency :path) nil (error-message-string err)
            "FAIL")))
        (when (and (not (equal "PASS" (plist-get record :status)))
                   (null devx-ai-test--current-failures))
          (push (devx-ai-test--fallback-failure case dependency record "")
                devx-ai-test--current-failures))
        (push record records)
        (setq failures
              (nconc failures (nreverse devx-ai-test--current-failures)))))
    (setq records (nreverse records))
    (list :records records
          :failures failures
          :status (devx-ai-test-result-status failures nil))))

(defun devx-ai-test--cleanup-failure (observed diagnostic)
  "Return a local cleanup failure describing only safe OBSERVED state."
  (devx-ai-test-make-failure
   nil "cleanup" "stopped LSP child processes" observed nil nil diagnostic
   "FAIL"))

(defun devx-ai-test--kill-fixture-buffer (buffer)
  "Kill fixture BUFFER and return the confirmation from `kill-buffer'."
  (kill-buffer buffer))

(defun devx-ai-test--cleanup (fixture-root observations)
  "Disconnect fixture buffers, wait for OBSERVATIONS, and return evidence."
  (let ((devx-ai-test--observed-processes (copy-sequence observations))
        failures teardown)
    (dolist (buffer (buffer-list))
      (when-let ((file (buffer-file-name buffer)))
        (when (and fixture-root (file-in-directory-p file fixture-root))
          (with-current-buffer buffer
            (condition-case err
                (devx-ai-test--capture-workspace-processes
                 (if (bound-and-true-p lsp-mode)
                     (devx-ai-adapter-workspaces)
                   nil))
              (error
               (push (devx-ai-test--cleanup-failure
                      (list :observed-type (symbol-name (type-of err)))
                      (error-message-string err))
                     failures)))
            (when (bound-and-true-p lsp-mode)
              (condition-case err
                  (progn
                    (devx-ai-adapter-shutdown-buffer-workspaces)
                    (lsp-disconnect))
                (error
                 (push (devx-ai-test--cleanup-failure
                        "lsp-disconnect failed" (error-message-string err))
                       failures))))
            (set-buffer-modified-p nil))
          (condition-case err
              (let ((killed (devx-ai-test--kill-fixture-buffer buffer)))
                (unless (and killed (not (buffer-live-p buffer)))
                  (push (devx-ai-test--cleanup-failure
                         (if (buffer-live-p buffer)
                             "fixture buffer survived"
                           "kill-buffer returned nil")
                         "Fixture buffer cleanup did not confirm termination")
                        failures)))
            (error
             (push (devx-ai-test--cleanup-failure
                    "fixture buffer survived" (error-message-string err))
                   failures))))))
    (when fixture-root
      (condition-case err
          (when (file-directory-p fixture-root)
            (delete-directory fixture-root t))
        (error
         (push (devx-ai-test--cleanup-failure
                "fixture directory survived" (error-message-string err))
               failures))))
    (dolist (observation (nreverse devx-ai-test--observed-processes))
      (let* ((process (plist-get observation :process))
             (_stopped
              (devx-ai-test--wait-until
               (lambda ()
                 (not (and (processp process) (process-live-p process))))
               5))
             (alive (if (and (processp process) (process-live-p process))
                        t
                      :json-false))
             (record
              (list :server
                    (and (plist-get observation :server)
                         (symbol-name (plist-get observation :server)))
                    :pid (plist-get observation :pid)
                    :alive-after-cleanup alive)))
        (push record teardown)
        (when (eq alive t)
          (push (devx-ai-test--cleanup-failure
                 (list :server (plist-get record :server)
                       :pid (plist-get record :pid)
                       :alive-after-cleanup t)
                 "LSP child survived bounded cleanup wait")
                failures))))
    (list :teardown (nreverse teardown) :failures (nreverse failures))))

(defun devx-ai-test--process-wide-result (cases boundary diagnostic)
  "Return per-language records and failures for one process-wide defect."
  (let (records failures)
    (dolist (case cases)
      (push (devx-ai-test-make-language-record
             case nil nil nil nil nil "FAIL")
            records)
      (push (devx-ai-test-make-failure
             (plist-get case :name) boundary nil nil nil nil diagnostic "FAIL")
            failures))
    (list :records (nreverse records) :failures (nreverse failures))))

(defun devx-ai-test-run-integration-result ()
  "Run all live cases and return records, failures, and teardown evidence."
  (let* ((cases (devx-ai-test-effective-language-cases))
         (lsp-auto-guess-root t)
         (lsp-keep-workspace-alive nil)
         (devx-ai-test--observed-processes nil)
         fixture-root case-result cleanup-result)
    (condition-case err
        (progn
          (setq fixture-root (make-temp-file "devx-ai-lsp-fixtures." t))
          (devx-ai-adapter-reset-session
           (expand-file-name "lsp-session-v1" fixture-root))
          (setq case-result
                (devx-ai-test-run-cases
                 cases fixture-root (devx-ai-test--preflight))))
      (error
       (setq case-result
             (devx-ai-test--process-wide-result
              cases "integration-setup" (error-message-string err)))))
    (setq cleanup-result
          (devx-ai-test--cleanup fixture-root devx-ai-test--observed-processes))
    (let* ((records (plist-get case-result :records))
           (failures
            (append (plist-get case-result :failures)
                    (plist-get cleanup-result :failures)))
           (teardown (plist-get cleanup-result :teardown)))
      (devx-ai-test-make-result
       "integration" (devx-ai-test-result-status failures teardown)
       records failures (vconcat teardown)))))

(provide 'ai-lsp-integration)
