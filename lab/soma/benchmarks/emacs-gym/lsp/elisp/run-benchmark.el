;;; run-benchmark.el --- Rootfs-owned LSP benchmark for the Emacs gym -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'json)
(require 'seq)
(require 'subr-x)

(defvar soma-lsp-benchmark-output-dir nil)
(defvar soma-lsp-benchmark-results nil)
(defvar soma-lsp-bench--jsonrpc-transport-suppressed nil)
(defvar soma-lsp-bench--incoming-messages nil)

(defun soma-lsp-bench--alist (&rest pairs)
  pairs)

(defun soma-lsp-bench--bool (value)
  (if value t :json-false))

(defun soma-lsp-bench--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-lsp-bench--first-user-arg ()
  (car (seq-drop-while
        (lambda (arg) (string= arg "--"))
        command-line-args-left)))

(defun soma-lsp-bench--read-json-file (path)
  (when (and path (not (string-empty-p path)) (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-lsp-bench--plan ()
  (soma-lsp-bench--read-json-file (or (getenv "SOMA_INSULA_PLAN") "")))

(defun soma-lsp-bench--get (alist key)
  (alist-get key alist nil nil #'string=))

(defun soma-lsp-bench--check (name passed detail)
  (soma-lsp-bench--alist
   (cons "name" name)
   (cons "passed" (soma-lsp-bench--bool passed))
   (cons "detail" detail)))

(defun soma-lsp-bench--all-checks-pass-p (checks)
  (cl-every
   (lambda (check)
     (eq (soma-lsp-bench--get check "passed") t))
   checks))

(defun soma-lsp-bench--rootfs-owned-path-p (path)
  (and (stringp path)
       (file-name-absolute-p path)
       (not (string-empty-p path))
       (not (string-prefix-p "/nix/" path))
       (not (string-prefix-p "/data02/home/" path))
       (not (string-prefix-p "/workspace/" path))
       (not (string-prefix-p "/project/" path))))

(defun soma-lsp-bench--inside-insula-path-p (path)
  (and (stringp path)
       (file-name-absolute-p path)
       (not (string-empty-p path))
       (not (string-prefix-p "/nix/" path))
       (not (string-prefix-p "/data02/home/" path))))

(defun soma-lsp-bench--doom-straight-build-dir ()
  (expand-file-name
   (format "~/.emacs.d/.local/straight/build-%d.%d"
           emacs-major-version emacs-minor-version)))

(defun soma-lsp-bench--add-straight-build-paths ()
  (let ((build (soma-lsp-bench--doom-straight-build-dir)))
    (when (file-directory-p build)
      (dolist (dir (directory-files build t directory-files-no-dot-files-regexp))
        (when (file-directory-p dir)
          (add-to-list 'load-path dir))))))

(defun soma-lsp-bench--stdio-connection (command &optional test-command)
  "Create an lsp-mode stdio connection without Emacs's jsonrpc transport.

The production bridge still exercises lsp-mode's normal client registration,
request routing, process filters, capabilities, and workspace state. This
benchmark-specific constructor avoids the Emacs 30 built-in jsonrpc connection
path in noninteractive batch runs because its initialize callback did not settle
under explicit `accept-process-output' waits in this rootfs.
"
  (list
   :connect
   (lambda (filter sentinel name environment-fn _workspace)
     (let ((final-command (lsp-resolve-final-command command))
           (process-name (generate-new-buffer-name name))
           (process-environment
            (lsp--compute-process-environment environment-fn)))
       (let* ((stderr-buf (get-buffer-create (format "*%s::stderr*" process-name)))
              (default-directory (lsp--default-directory-for-connection))
              (proc (make-process
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

(defun soma-lsp-bench--load-lsp ()
  (soma-lsp-bench--add-straight-build-paths)
  (setenv "LSP_USE_PLISTS" "true")
  (setq lsp-rust-server 'rust-analyzer)
  (require 'python)
  (require 'cc-mode)
  (require 'rust-mode)
  (require 'lsp-mode)
  (setq soma-lsp-bench--jsonrpc-transport-suppressed t)
  (fset 'lsp-stdio-connection #'soma-lsp-bench--stdio-connection)
  (require 'lsp-pylsp)
  (require 'lsp-clangd)
  (require 'lsp-rust)
  (setq lsp-session-file nil
        lsp-auto-guess-root nil
        lsp-guess-root-without-session nil
        lsp-enable-snippet nil
        lsp-enable-file-watchers nil
        lsp-enable-symbol-highlighting nil
        lsp-lens-enable nil
        lsp-headerline-breadcrumb-enable nil
        lsp-modeline-workspace-status-enable nil
        lsp-modeline-diagnostics-enable nil
        lsp-modeline-code-actions-enable nil
        lsp-log-io nil
        lsp-response-timeout 20
        lsp-restart 'ignore)
  (setq lsp-pylsp-server-command '("/usr/bin/pylsp")
        lsp-clients-clangd-executable "/usr/bin/clangd"
        lsp-clients-clangd-args '("--header-insertion-decorators=0"
                                  "--query-driver=/usr/bin/g++")
        lsp-rust-analyzer-server-command '("/opt/soma/cargo/bin/rust-analyzer")))

(defun soma-lsp-bench--transport-evidence ()
  (soma-lsp-bench--alist
   (cons "transport"
         (if soma-lsp-bench--jsonrpc-transport-suppressed
             "lsp-mode-stdio-process-filter"
           "emacs-json-rpc-connection"))
   (cons "json_rpc_connection_suppressed"
         (soma-lsp-bench--bool soma-lsp-bench--jsonrpc-transport-suppressed))
   (cons "reason"
         (if soma-lsp-bench--jsonrpc-transport-suppressed
             "noninteractive batch benchmark uses lsp-mode process filters so initialize replies and capabilities settle under explicit accept-process-output waits"
           "Emacs json-rpc-connection remained active"))))

(defun soma-lsp-bench--run-process (program &rest args)
  (with-temp-buffer
    (let ((status (apply #'process-file program nil t nil args))
          (output (buffer-substring-no-properties (point-min) (point-max))))
      (soma-lsp-bench--alist
       (cons "program" program)
       (cons "args" args)
       (cons "status" status)
       (cons "output" output)))))

(defun soma-lsp-bench--run-process-in (cwd program &rest args)
  (let ((default-directory (file-name-as-directory cwd)))
    (apply #'soma-lsp-bench--run-process program args)))

(defun soma-lsp-bench--command-ok-p (result)
  (equal (soma-lsp-bench--get result "status") 0))

(defun soma-lsp-bench--write-file (path text)
  (make-directory (file-name-directory path) t)
  (with-temp-file path
    (insert text)))

(defun soma-lsp-bench--read-file (path)
  (with-temp-buffer
    (insert-file-contents path)
    (buffer-substring-no-properties (point-min) (point-max))))

(defun soma-lsp-bench--tool-evidence (name &optional version-args)
  (let* ((path (executable-find name))
         (realpath (and path (file-truename path)))
         (version
          (when path
            (let ((result (apply #'soma-lsp-bench--run-process
                                 path
                                 (or version-args '("--version")))))
              (car (split-string (or (soma-lsp-bench--get result "output") "")
                                 "\n" t))))))
    (soma-lsp-bench--alist
     (cons "name" name)
     (cons "path" (or path ""))
     (cons "realpath" (or realpath ""))
     (cons "rootfs_owned"
           (soma-lsp-bench--bool
            (soma-lsp-bench--rootfs-owned-path-p (or realpath path ""))))
     (cons "version" (or version "")))))

(defun soma-lsp-bench--python-env-evidence (root package-path)
  (let* ((python (executable-find "python3"))
         (probe
          (soma-lsp-bench--run-process-in
           root python "-c"
           "import json, os, pathlib, sys, unittest; print(json.dumps({'executable': sys.executable, 'prefix': sys.prefix, 'path0': sys.path[0], 'package_path': str(pathlib.Path('soma_pkg').resolve()), 'unittest_module': unittest.__file__}, sort_keys=True))"))
         (payload
          (when (soma-lsp-bench--command-ok-p probe)
            (let ((json-object-type 'alist)
                  (json-array-type 'list)
                  (json-key-type 'string)
                  (json-false :json-false))
              (json-read-from-string
               (string-trim (soma-lsp-bench--get probe "output")))))))
    (soma-lsp-bench--alist
     (cons "python3" (soma-lsp-bench--tool-evidence "python3" '("-c" "import sys; print(sys.version.split()[0])")))
     (cons "pylsp" (soma-lsp-bench--tool-evidence "pylsp"))
     (cons "pyright-langserver" (soma-lsp-bench--tool-evidence "pyright-langserver" '("--version")))
     (cons "package_path" package-path)
     (cons "probe" probe)
     (cons "payload" (or payload :json-false)))))

(defun soma-lsp-bench--proc-link (pid name)
  (let ((path (format "/proc/%s/%s" pid name)))
    (if (file-exists-p path)
        (file-truename path)
      "")))

(defun soma-lsp-bench--proc-environment (pid)
  (let ((path (format "/proc/%s/environ" pid))
        entries selected)
    (when (file-readable-p path)
      (with-temp-buffer
        (insert-file-contents-literally path)
        (setq entries (split-string (buffer-string) "\0" t))))
    (dolist (entry entries)
      (when (string-match "\\`\\([^=]+\\)=\\(.*\\)\\'" entry)
        (let ((key (match-string 1 entry))
              (value (match-string 2 entry)))
          (when (member key '("PATH" "HOME" "TMPDIR" "XDG_RUNTIME_DIR"
                              "XDG_CACHE_HOME" "RUSTUP_HOME" "CARGO_HOME"
                              "SOMA_INSULA_ACTIVE" "SOMA_INSULA_RUN_ID"))
            (push (cons key value) selected)))))
    (nreverse selected)))

(defun soma-lsp-bench--buffer-tail (buffer-name max-chars)
  (if-let* ((buffer (get-buffer buffer-name)))
      (with-current-buffer buffer
        (buffer-substring-no-properties
         (max (point-min) (- (point-max) max-chars))
         (point-max)))
    ""))

(defun soma-lsp-bench--workspace-process-evidence (workspace)
  (let* ((proc (ignore-errors (lsp--workspace-cmd-proc workspace)))
         (wire-proc (ignore-errors (lsp--workspace-proc workspace)))
         (pid (and proc (ignore-errors (lsp-process-id proc)))))
    (soma-lsp-bench--alist
     (cons "server_id"
           (if workspace
               (symbol-name
                (lsp--client-server-id (lsp--workspace-client workspace)))
             ""))
     (cons "workspace_status"
           (if workspace (symbol-name (lsp--workspace-status workspace)) ""))
     (cons "workspace_proc_type" (format "%S" (type-of wire-proc)))
     (cons "cmd_proc_type" (format "%S" (type-of proc)))
     (cons "cmd_proc_status"
           (if (and proc (processp proc))
               (symbol-name (process-status proc))
             ""))
     (cons "workspace_root" (if workspace (lsp--workspace-root workspace) ""))
     (cons "server_pid" (or pid :json-false))
     (cons "server_executable"
           (if pid (soma-lsp-bench--proc-link pid "exe") ""))
     (cons "server_cwd"
           (if pid (soma-lsp-bench--proc-link pid "cwd") ""))
     (cons "server_environment"
           (if pid (soma-lsp-bench--proc-environment pid) nil))
     (cons "server_command"
           (if (and proc (processp proc))
               (process-command proc)
             nil))
     (cons "server_buffer_tail"
           (if (and proc (processp proc))
               (soma-lsp-bench--buffer-tail (process-buffer proc) 2000)
             ""))
     (cons "server_stderr_tail"
           (if (and proc (processp proc))
               (soma-lsp-bench--buffer-tail
                (format "*%s::stderr*" (process-name proc))
                4000)
             ""))
     (cons "messages_tail" (soma-lsp-bench--buffer-tail "*Messages*" 4000))
     (cons "transport" (soma-lsp-bench--transport-evidence)))))

(defun soma-lsp-bench--position-encoding (workspace)
  (or (ignore-errors
        (lsp:server-capabilities-position-encoding?
         (lsp--workspace-server-capabilities workspace)))
      (cond
       ((eq lsp--position-column-function #'lsp--utf-16-column) "utf-16")
       ((eq lsp--position-column-function #'lsp--utf-8-column) "utf-8")
       ((eq lsp--position-column-function #'lsp--utf-32-column) "utf-32")
       (t "unknown"))))

(defun soma-lsp-bench--diagnostic-count (diagnostics)
  (cond
   ((vectorp diagnostics) (length diagnostics))
   ((listp diagnostics) (length diagnostics))
   (diagnostics 1)
   (t 0)))

(defun soma-lsp-bench--diagnostic-evidence ()
  (let ((diagnostics (ignore-errors (lsp--get-buffer-diagnostics))))
    (soma-lsp-bench--alist
     (cons "count" (soma-lsp-bench--diagnostic-count diagnostics))
     (cons "raw" (if diagnostics (format "%S" diagnostics) "")))))

(defun soma-lsp-bench--request-ids (requests)
  (delq nil
        (mapcar (lambda (entry) (soma-lsp-bench--get entry "id"))
                requests)))

(defun soma-lsp-bench--request-method-present-p (requests method)
  (seq-some
   (lambda (entry)
     (string= (soma-lsp-bench--get entry "method") method))
   requests))

(defun soma-lsp-bench--method-supported-p (method)
  (soma-lsp-bench--bool (ignore-errors (lsp-feature? method))))

(defun soma-lsp-bench--capability-snapshot ()
  (mapcar
   (lambda (method)
     (cons method (soma-lsp-bench--method-supported-p method)))
   '("textDocument/hover"
     "textDocument/references"
     "textDocument/definition"
     "textDocument/documentSymbol"
     "textDocument/rename")))

(defun soma-lsp-bench--with-request-trace (thunk)
  (let ((orig-request (symbol-function 'lsp--send-request-async))
        (orig-parser (symbol-function 'lsp--parser-on-message))
        requests
        incoming)
    (cl-letf (((symbol-function 'lsp--send-request-async)
               (lambda (body callback &rest args)
                 (let ((returned (apply orig-request body callback args)))
                   (push
                    (soma-lsp-bench--alist
                     (cons "method" (or (plist-get body :method) ""))
                     (cons "id" (or (and (listp returned)
                                         (plist-get returned :id))
                                    :json-false))
                     (cons "buffer" (buffer-name (current-buffer)))
                     (cons "document_version"
                           (if (boundp 'lsp--cur-version)
                               lsp--cur-version
                             :json-false)))
                    requests)
                   returned)))
              ((symbol-function 'lsp--parser-on-message)
               (lambda (json-data workspace)
                 (push
                  (soma-lsp-bench--alist
                   (cons "message_type"
                         (format "%S"
                                 (ignore-errors
                                   (lsp--get-message-type json-data))))
                   (cons "id"
                         (or (ignore-errors
                               (let ((id (lsp:json-response-id json-data)))
                                 (if (stringp id) (string-to-number id) id)))
                             :json-false))
                   (cons "method"
                         (or (ignore-errors
                               (lsp:json-notification-method json-data))
                             (ignore-errors
                               (lsp:json-request-method json-data))
                             ""))
                   (cons "workspace_status_before"
                         (if workspace
                             (symbol-name (lsp--workspace-status workspace))
                           ""))
                   (cons "server_id"
                         (if workspace
                             (symbol-name
                              (lsp--client-server-id
                               (lsp--workspace-client workspace)))
                           "")))
                  incoming)
                 (let ((result (funcall orig-parser json-data workspace)))
                   (push
                    (soma-lsp-bench--alist
                     (cons "message_type" "post-parser-state")
                     (cons "id"
                           (or (ignore-errors
                                 (let ((id (lsp:json-response-id json-data)))
                                   (if (stringp id) (string-to-number id) id)))
                               :json-false))
                     (cons "workspace_status_after"
                           (if workspace
                               (symbol-name (lsp--workspace-status workspace))
                             "")))
                    incoming)
                   result))))
      (let ((result (funcall thunk)))
        (soma-lsp-bench--alist
         (cons "result" result)
         (cons "requests" (nreverse requests))
         (cons "incoming_messages" (nreverse incoming)))))))

(defun soma-lsp-bench--wait-until (predicate timeout)
  (let ((deadline (+ (float-time) timeout))
        done)
    (while (and (not done) (< (float-time) deadline))
      (accept-process-output nil 0.05)
      (sit-for 0.01)
      (setq done (funcall predicate)))
    done))

(defun soma-lsp-bench--drain (seconds)
  (let ((deadline (+ (float-time) seconds)))
    (while (< (float-time) deadline)
      (accept-process-output nil 0.05)
      (sit-for 0.01))))

(defun soma-lsp-bench--benchmark-server-process-p (process)
  (let ((command (ignore-errors (process-command process))))
    (and command
         (seq-some
          (lambda (part)
            (and (stringp part)
                 (string-match-p
                  "\\(pylsp\\|clangd\\|rust-analyzer\\)" part)))
          command))))

(defun soma-lsp-bench--live-benchmark-server-processes ()
  (seq-filter
   (lambda (process)
     (and (process-live-p process)
          (soma-lsp-bench--benchmark-server-process-p process)))
   (process-list)))

(defun soma-lsp-bench--shutdown-lsp ()
  (when (fboundp 'lsp--global-teardown)
    (ignore-errors (lsp--global-teardown)))
  (soma-lsp-bench--drain 0.5)
  (dolist (process (soma-lsp-bench--live-benchmark-server-processes))
    (ignore-errors (delete-process process)))
  (soma-lsp-bench--wait-until
   (lambda ()
     (null (soma-lsp-bench--live-benchmark-server-processes)))
   5)
  (setq lsp--session (make-lsp-session)))

(defun soma-lsp-bench--open-workspace (fixture)
  (let* ((root (soma-lsp-bench--get fixture "root"))
         (file (soma-lsp-bench--get fixture "file"))
         (mode (soma-lsp-bench--get fixture "mode"))
         (canonical-root (file-name-as-directory (file-truename root)))
         buffer workspace initialized)
    (soma-lsp-bench--shutdown-lsp)
    (setq lsp--session
          (make-lsp-session
           :folders (list canonical-root)
           :folders-blocklist nil))
    (setq buffer (find-file-noselect file))
    (with-current-buffer buffer
      (setq default-directory canonical-root)
      (funcall mode)
      (lsp)
      (setq initialized
            (soma-lsp-bench--wait-until
             (lambda ()
               (and lsp--buffer-workspaces
                    (setq workspace (car lsp--buffer-workspaces))
                    (eq (lsp--workspace-status workspace) 'initialized)))
             30))
      (soma-lsp-bench--drain 0.5)
      (list buffer workspace initialized))))

(defun soma-lsp-bench--goto-symbol (symbol)
  (goto-char (point-min))
  (unless (search-forward symbol nil t)
    (error "symbol not found: %s" symbol))
  (goto-char (match-beginning 0)))

(defun soma-lsp-bench--safe-lsp-request (method params)
  (if (not (eq (soma-lsp-bench--method-supported-p method) t))
      (soma-lsp-bench--alist
       (cons "method" method)
       (cons "supported" :json-false)
       (cons "status" "unsupported")
       (cons "raw" "")
       (cons "count" 0)
       (cons "error" "server capability not advertised"))
    (condition-case err
        (let ((response (lsp-request method params)))
          (soma-lsp-bench--alist
           (cons "method" method)
           (cons "supported" t)
           (cons "status" "ok")
           (cons "raw" (format "%S" response))
           (cons "count" (soma-lsp-bench--diagnostic-count response))
           (cons "error" "")))
      (error
       (soma-lsp-bench--alist
        (cons "method" method)
        (cons "supported" t)
        (cons "status" "error")
        (cons "raw" "")
        (cons "count" 0)
        (cons "error" (error-message-string err)))))))

(defun soma-lsp-bench--response-names-path-p (evidence expected-path-fragment)
  (let ((definition-raw (or (soma-lsp-bench--get evidence "definition_raw") ""))
        (references-raw (or (soma-lsp-bench--get evidence "references_raw") "")))
    (and (stringp expected-path-fragment)
         (not (string-empty-p expected-path-fragment))
         (or (string-match-p (regexp-quote expected-path-fragment) definition-raw)
             (string-match-p (regexp-quote expected-path-fragment) references-raw)))))

(defun soma-lsp-bench--request-evidence-once (symbol)
  (soma-lsp-bench--goto-symbol symbol)
  (when (fboundp 'lsp--flush-delayed-changes)
    (ignore-errors (lsp--flush-delayed-changes)))
  (let* ((position-params (lsp--text-document-position-params))
         (reference-params (list :textDocument
                                 (lsp--text-document-identifier)
                                 :position (lsp--cur-position)
                                 :context (list :includeDeclaration t)))
         (hover (soma-lsp-bench--safe-lsp-request
                 "textDocument/hover" position-params))
         (references (soma-lsp-bench--safe-lsp-request
                      "textDocument/references" reference-params))
         (definition (soma-lsp-bench--safe-lsp-request
                      "textDocument/definition" position-params)))
    (soma-lsp-bench--alist
     (cons "target_symbol" symbol)
     (cons "capabilities" (soma-lsp-bench--capability-snapshot))
     (cons "hover" hover)
     (cons "references" references)
     (cons "definition" definition)
     (cons "hover_raw" (soma-lsp-bench--get hover "raw"))
     (cons "references_raw" (soma-lsp-bench--get references "raw"))
     (cons "definition_raw" (soma-lsp-bench--get definition "raw"))
     (cons "reference_count"
           (soma-lsp-bench--get references "count")))))

(defun soma-lsp-bench--request-evidence (symbol &optional expected-path-fragment timeout)
  (let ((deadline (+ (float-time) (or timeout 10)))
        (attempt 0)
        evidence)
    (while (and (< (float-time) deadline)
                (not (and evidence
                          (soma-lsp-bench--response-names-path-p
                           evidence expected-path-fragment))))
      (setq attempt (1+ attempt))
      (setq evidence (soma-lsp-bench--request-evidence-once symbol))
      (unless (soma-lsp-bench--response-names-path-p
               evidence expected-path-fragment)
        (soma-lsp-bench--drain 0.5)))
    (append
     evidence
     (list
      (cons "semantic_attempts" attempt)
      (cons "semantic_ready"
            (soma-lsp-bench--bool
             (soma-lsp-bench--response-names-path-p
              evidence expected-path-fragment)))))))

(defun soma-lsp-bench--buffer-evidence (buffer workspace)
  (with-current-buffer buffer
    (soma-lsp-bench--alist
     (cons "buffer" (buffer-name))
     (cons "file" (or buffer-file-name ""))
     (cons "major_mode" (symbol-name major-mode))
     (cons "workspace_root" (or (ignore-errors (lsp-workspace-root)) ""))
     (cons "document_uri" (or (ignore-errors (lsp--buffer-uri)) ""))
     (cons "document_version"
           (if (boundp 'lsp--cur-version) lsp--cur-version :json-false))
     (cons "position_encoding"
           (if workspace
               (soma-lsp-bench--position-encoding workspace)
             ""))
     (cons "capabilities" (soma-lsp-bench--capability-snapshot))
     (cons "diagnostics" (soma-lsp-bench--diagnostic-evidence)))))

(defun soma-lsp-bench--make-python-fixture (name)
  (let* ((root (make-temp-file (format "soma-lsp-python-%s-" name) t))
         (pkg (expand-file-name "soma_pkg" root))
         (tests (expand-file-name "tests" root))
         (file (expand-file-name "main.py" root)))
    (soma-lsp-bench--write-file (expand-file-name "pyproject.toml" root)
                                "[project]\nname = \"soma-lsp-fixture\"\nversion = \"0.0.0\"\n")
    (soma-lsp-bench--write-file (expand-file-name "__init__.py" pkg) "")
    (soma-lsp-bench--write-file (expand-file-name "core.py" pkg)
                                "def compute_value(x: int) -> int:\n    return x + 22\n")
    (soma-lsp-bench--write-file
     file
     "from soma_pkg.core import compute_value\n\nRESULT = compute_value(20)\n")
    (soma-lsp-bench--write-file (expand-file-name "__init__.py" tests) "")
    (soma-lsp-bench--write-file
     (expand-file-name "test_core.py" tests)
     "import unittest\nfrom soma_pkg.core import compute_value\n\nclass CoreTest(unittest.TestCase):\n    def test_compute_value(self):\n        self.assertEqual(compute_value(20), 42)\n\nif __name__ == '__main__':\n    unittest.main()\n")
    (soma-lsp-bench--alist
     (cons "language" "python")
     (cons "server_id" "pylsp")
     (cons "root" root)
     (cons "file" file)
     (cons "mode" #'python-mode)
     (cons "symbol" "compute_value")
     (cons "expected_path_fragment" "/soma_pkg/core.py")
     (cons "package_path" pkg)
     (cons "build_test"
           (lambda ()
             (soma-lsp-bench--run-process-in
              root "python3" "-m" "unittest" "discover" "-s" "tests"))))))

(defun soma-lsp-bench--make-cpp-fixture (name)
  (let* ((root (make-temp-file (format "soma-lsp-cpp-%s-" name) t))
         (include (expand-file-name "include" root))
         (src (expand-file-name "src" root))
         (file (expand-file-name "main.cpp" src)))
    (soma-lsp-bench--write-file
     (expand-file-name "soma_lib.hpp" include)
     "#pragma once\ninline int soma_metric(int x) { return x + 22; }\n")
    (soma-lsp-bench--write-file
     file
     "#include \"soma_lib.hpp\"\n#include <iostream>\n\nint main() {\n  std::cout << soma_metric(20) << \"\\n\";\n  return 0;\n}\n")
    (soma-lsp-bench--write-file
     (expand-file-name "compile_commands.json" root)
     (format "[{\"directory\":\"%s\",\"command\":\"/usr/bin/g++ -std=c++20 -Iinclude -c src/main.cpp\",\"file\":\"%s\"}]\n"
             root file))
    (soma-lsp-bench--alist
     (cons "language" "cpp")
     (cons "server_id" "clangd")
     (cons "root" root)
     (cons "file" file)
     (cons "mode" #'c++-mode)
     (cons "symbol" "soma_metric")
     (cons "expected_path_fragment" "/include/soma_lib.hpp")
     (cons "compile_database" (expand-file-name "compile_commands.json" root))
     (cons "build_test"
           (lambda ()
             (let ((build (soma-lsp-bench--run-process-in
                           root "g++" "-std=c++20" "-Iinclude"
                           "src/main.cpp" "-o" "soma_cpp_app")))
               (if (soma-lsp-bench--command-ok-p build)
                   (let ((run (soma-lsp-bench--run-process-in root
                                                               (expand-file-name "soma_cpp_app" root))))
                     (append build
                             (list (cons "run_status"
                                         (soma-lsp-bench--get run "status"))
                                   (cons "run_output"
                                         (soma-lsp-bench--get run "output"))
                                   (cons "linked_libraries"
                                         (soma-lsp-bench--get
                                          (soma-lsp-bench--run-process-in
                                           root "ldd"
                                           (expand-file-name "soma_cpp_app" root))
                                          "output")))))
                 build)))))))

(defun soma-lsp-bench--make-rust-fixture (name)
  (let* ((root (make-temp-file (format "soma-lsp-rust-%s-" name) t))
         (src (expand-file-name "src" root))
         (file (expand-file-name "main.rs" src)))
    (soma-lsp-bench--write-file
     (expand-file-name "Cargo.toml" root)
     "[package]\nname = \"soma_lsp_fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
    (soma-lsp-bench--write-file
     (expand-file-name "lib.rs" src)
     "pub fn soma_metric(x: i32) -> i32 {\n    x + 22\n}\n\n#[cfg(test)]\nmod tests {\n    use super::soma_metric;\n\n    #[test]\n    fn computes_answer() {\n        assert_eq!(soma_metric(20), 42);\n    }\n}\n")
    (soma-lsp-bench--write-file
     file
     "fn main() {\n    let answer = soma_lsp_fixture::soma_metric(20);\n    println!(\"{answer}\");\n}\n")
    (soma-lsp-bench--alist
     (cons "language" "rust")
     (cons "server_id" "rust-analyzer")
     (cons "root" root)
     (cons "file" file)
     (cons "mode" #'rust-mode)
     (cons "symbol" "soma_metric")
     (cons "expected_path_fragment" "/src/lib.rs")
     (cons "crate_manifest" (expand-file-name "Cargo.toml" root))
     (cons "build_test"
           (lambda ()
             (soma-lsp-bench--run-process-in
              root "cargo" "test" "--offline" "--quiet"))))))

(defun soma-lsp-bench--fixture-public (fixture)
  (soma-lsp-bench--alist
   (cons "language" (soma-lsp-bench--get fixture "language"))
   (cons "server_id" (soma-lsp-bench--get fixture "server_id"))
   (cons "root" (soma-lsp-bench--get fixture "root"))
   (cons "file" (soma-lsp-bench--get fixture "file"))
   (cons "symbol" (soma-lsp-bench--get fixture "symbol"))
   (cons "expected_path_fragment"
         (soma-lsp-bench--get fixture "expected_path_fragment"))
   (cons "fixture_origin" "generated-inside-bwrap-private-project-runtime")))

(defun soma-lsp-bench--toolchain-evidence (fixture build-test)
  (let ((language (soma-lsp-bench--get fixture "language"))
        (root (soma-lsp-bench--get fixture "root")))
    (pcase language
      ("python"
       (soma-lsp-bench--python-env-evidence
        root
        (soma-lsp-bench--get fixture "package_path")))
      ("cpp"
       (soma-lsp-bench--alist
        (cons "clangd" (soma-lsp-bench--tool-evidence "clangd"))
        (cons "clang++" (soma-lsp-bench--tool-evidence "clang++"))
        (cons "gcc" (soma-lsp-bench--tool-evidence "gcc"))
        (cons "g++" (soma-lsp-bench--tool-evidence "g++"))
        (cons "compile_database"
              (soma-lsp-bench--get fixture "compile_database"))
        (cons "compile_database_inside_insula"
              (soma-lsp-bench--bool
               (soma-lsp-bench--inside-insula-path-p
                (soma-lsp-bench--get fixture "compile_database"))))
        (cons "build_result" build-test)))
      ("rust"
       (soma-lsp-bench--alist
        (cons "rust-analyzer" (soma-lsp-bench--tool-evidence "rust-analyzer"))
        (cons "rustc" (soma-lsp-bench--tool-evidence "rustc"))
        (cons "cargo" (soma-lsp-bench--tool-evidence "cargo"))
        (cons "rustup_home" (or (getenv "RUSTUP_HOME") ""))
        (cons "cargo_home" (or (getenv "CARGO_HOME") ""))
        (cons "crate_manifest" (soma-lsp-bench--get fixture "crate_manifest"))
        (cons "build_result" build-test)))
      (_ nil))))

(defun soma-lsp-bench--exercise-fixture (fixture)
  (let* ((trace
          (soma-lsp-bench--with-request-trace
           (lambda ()
             (let* ((opened (soma-lsp-bench--open-workspace fixture))
                    (buffer (nth 0 opened))
                    (workspace (nth 1 opened))
                    (initialized (nth 2 opened))
                    (symbol (soma-lsp-bench--get fixture "symbol"))
                    (expected-path-fragment
                     (soma-lsp-bench--get fixture "expected_path_fragment"))
                    request-evidence
                    buffer-evidence
                    process-evidence
                    build-test)
               (with-current-buffer buffer
                 (setq request-evidence
                       (soma-lsp-bench--request-evidence
                        symbol expected-path-fragment 12))
                 (setq buffer-evidence
                       (soma-lsp-bench--buffer-evidence buffer workspace))
                 (setq process-evidence
                       (soma-lsp-bench--workspace-process-evidence workspace)))
               (setq build-test
                     (funcall (soma-lsp-bench--get fixture "build_test")))
               (soma-lsp-bench--alist
                (cons "state_sequence"
                      (list "fixture-created"
                            "buffer-opened"
                            "lsp-start-requested"
                            (if initialized
                                "workspace-initialized"
                              "workspace-timeout-outcome_unknown")
                            "diagnostics-observed"
                            "symbol-requests-completed"
                            "build-test-command-completed"))
                (cons "fixture" (soma-lsp-bench--fixture-public fixture))
                (cons "buffer" buffer-evidence)
                (cons "server" process-evidence)
                (cons "request_evidence" request-evidence)
                (cons "build_test" build-test)
                (cons "toolchain"
                      (soma-lsp-bench--toolchain-evidence fixture build-test)))))))
         (result (soma-lsp-bench--get trace "result"))
         (requests (soma-lsp-bench--get trace "requests")))
    (append
     result
     (list (cons "requests" requests)
           (cons "incoming_messages"
                 (soma-lsp-bench--get trace "incoming_messages"))
           (cons "request_ids" (soma-lsp-bench--request-ids requests))))))

(defun soma-lsp-bench--positive-checks (language result)
  (let* ((fixture (soma-lsp-bench--get result "fixture"))
         (expected (soma-lsp-bench--get fixture "expected_path_fragment"))
         (server (soma-lsp-bench--get result "server"))
         (buffer (soma-lsp-bench--get result "buffer"))
         (request-evidence (soma-lsp-bench--get result "request_evidence"))
         (toolchain (soma-lsp-bench--get result "toolchain"))
         (requests (soma-lsp-bench--get result "requests"))
         (build-test (soma-lsp-bench--get result "build_test"))
         (hover (soma-lsp-bench--get request-evidence "hover"))
         (references (soma-lsp-bench--get request-evidence "references"))
         (definition (soma-lsp-bench--get request-evidence "definition"))
         (definition-raw (or (soma-lsp-bench--get request-evidence "definition_raw") ""))
         (references-raw (or (soma-lsp-bench--get request-evidence "references_raw") ""))
         (server-exe (soma-lsp-bench--get server "server_executable"))
         (workspace-root (soma-lsp-bench--get server "workspace_root"))
         (document-uri (soma-lsp-bench--get buffer "document_uri")))
    (append
     (list
      (soma-lsp-bench--check
       "workspace initialized"
       (string= (soma-lsp-bench--get server "workspace_status") "initialized")
       (format "%S" server))
      (soma-lsp-bench--check
       "expected server id"
       (string= (soma-lsp-bench--get server "server_id")
                (soma-lsp-bench--get fixture "server_id"))
       (format "%S" server))
      (soma-lsp-bench--check
       "server executable is rootfs-owned"
       (soma-lsp-bench--rootfs-owned-path-p server-exe)
       server-exe)
      (soma-lsp-bench--check
       "workspace root is inside Insula runtime"
       (soma-lsp-bench--inside-insula-path-p workspace-root)
       workspace-root)
      (soma-lsp-bench--check
       "document uri is file URI"
       (and (stringp document-uri)
            (string-prefix-p "file://" document-uri))
       document-uri)
      (soma-lsp-bench--check
       "document version recorded"
       (integerp (soma-lsp-bench--get buffer "document_version"))
       (format "%S" (soma-lsp-bench--get buffer "document_version")))
      (soma-lsp-bench--check
       "position encoding recorded"
       (member (soma-lsp-bench--get buffer "position_encoding")
               '("utf-8" "utf-16" "utf-32" "unknown"))
       (soma-lsp-bench--get buffer "position_encoding"))
      (soma-lsp-bench--check
       "supported semantic LSP request observed"
       (seq-some
        (lambda (entry)
          (and (string= (soma-lsp-bench--get entry "status") "ok")
               (soma-lsp-bench--request-method-present-p
                requests
                (soma-lsp-bench--get entry "method"))))
        (list hover references definition))
       (format "requests=%S evidence=%S" requests request-evidence))
      (soma-lsp-bench--check
       "LSP response names expected definition path"
       (or (string-match-p (regexp-quote expected) definition-raw)
           (string-match-p (regexp-quote expected) references-raw))
       (format "expected=%s definition=%s references=%s"
               expected definition-raw references-raw))
      (soma-lsp-bench--check
       "build or test command succeeded"
       (or (soma-lsp-bench--command-ok-p build-test)
           (equal (soma-lsp-bench--get build-test "run_status") 0))
       (format "%S" build-test)))
     (pcase language
       ("python"
        (let* ((python (soma-lsp-bench--get toolchain "python3"))
               (pylsp (soma-lsp-bench--get toolchain "pylsp"))
               (payload (soma-lsp-bench--get toolchain "payload"))
               (executable (and (listp payload)
                                (soma-lsp-bench--get payload "executable")))
               (prefix (and (listp payload)
                            (soma-lsp-bench--get payload "prefix")))
               (package-path (soma-lsp-bench--get toolchain "package_path")))
          (list
           (soma-lsp-bench--check
            "python executable is rootfs-owned"
            (and (eq (soma-lsp-bench--get python "rootfs_owned") t)
                 (soma-lsp-bench--rootfs-owned-path-p executable))
            (format "tool=%S payload=%S" python payload))
           (soma-lsp-bench--check
            "python prefix is rootfs-owned"
            (soma-lsp-bench--rootfs-owned-path-p prefix)
            (format "%S" payload))
           (soma-lsp-bench--check
            "pylsp executable is rootfs-owned"
            (eq (soma-lsp-bench--get pylsp "rootfs_owned") t)
            (format "%S" pylsp))
           (soma-lsp-bench--check
            "package path is inside Insula runtime"
            (soma-lsp-bench--inside-insula-path-p package-path)
            package-path))))
       ("cpp"
        (let ((clangd (soma-lsp-bench--get toolchain "clangd"))
              (clang++ (soma-lsp-bench--get toolchain "clang++"))
              (gcc (soma-lsp-bench--get toolchain "gcc"))
              (g++ (soma-lsp-bench--get toolchain "g++"))
              (compile-database
               (soma-lsp-bench--get toolchain "compile_database"))
              (build-result (soma-lsp-bench--get toolchain "build_result")))
          (list
           (soma-lsp-bench--check
            "clangd executable is rootfs-owned"
            (eq (soma-lsp-bench--get clangd "rootfs_owned") t)
            (format "%S" clangd))
           (soma-lsp-bench--check
            "compiler and linker tools are rootfs-owned"
            (and (eq (soma-lsp-bench--get clang++ "rootfs_owned") t)
                 (eq (soma-lsp-bench--get gcc "rootfs_owned") t)
                 (eq (soma-lsp-bench--get g++ "rootfs_owned") t))
            (format "clang++=%S gcc=%S g++=%S" clang++ gcc g++))
           (soma-lsp-bench--check
            "compile database is inside Insula runtime"
            (and (soma-lsp-bench--inside-insula-path-p compile-database)
                 (file-readable-p compile-database))
            compile-database)
           (soma-lsp-bench--check
            "linked libc and libstdc++ resolve from rootfs"
            (let ((linked (or (soma-lsp-bench--get build-result
                                                   "linked_libraries")
                              "")))
              (and (string-match-p "/lib/.+libc\\.so" linked)
                   (string-match-p "/lib/.+libstdc\\+\\+\\.so" linked)
                   (not (string-match-p "/nix/\\|/data02/home/" linked))))
            (format "%S" build-result)))))
       ("rust"
        (let ((rust-analyzer (soma-lsp-bench--get toolchain "rust-analyzer"))
              (rustc (soma-lsp-bench--get toolchain "rustc"))
              (cargo (soma-lsp-bench--get toolchain "cargo"))
              (rustup-home (soma-lsp-bench--get toolchain "rustup_home"))
              (cargo-home (soma-lsp-bench--get toolchain "cargo_home"))
              (crate-manifest (soma-lsp-bench--get toolchain "crate_manifest")))
          (list
           (soma-lsp-bench--check
            "rust-analyzer executable is rootfs-owned"
            (eq (soma-lsp-bench--get rust-analyzer "rootfs_owned") t)
            (format "%S" rust-analyzer))
           (soma-lsp-bench--check
            "rustc and cargo executables are rootfs-owned"
            (and (eq (soma-lsp-bench--get rustc "rootfs_owned") t)
                 (eq (soma-lsp-bench--get cargo "rootfs_owned") t))
            (format "rustc=%S cargo=%S" rustc cargo))
           (soma-lsp-bench--check
            "rust tool cache roots are rootfs-owned"
            (and (soma-lsp-bench--rootfs-owned-path-p rustup-home)
                 (soma-lsp-bench--rootfs-owned-path-p cargo-home))
            (format "RUSTUP_HOME=%S CARGO_HOME=%S" rustup-home cargo-home))
           (soma-lsp-bench--check
            "crate manifest is inside Insula runtime"
            (and (soma-lsp-bench--inside-insula-path-p crate-manifest)
                 (file-readable-p crate-manifest))
            crate-manifest))))
       (_ nil)))))

(defun soma-lsp-bench--scenario-language-positive (fixture-fn id)
  (let ((fixture nil))
    (unwind-protect
        (progn
          (setq fixture (funcall fixture-fn id))
          (let* ((language (soma-lsp-bench--get fixture "language"))
                 (result (soma-lsp-bench--exercise-fixture fixture))
                 (checks (soma-lsp-bench--positive-checks language result)))
            (append
             (soma-lsp-bench--alist
              (cons "name" (format "correct-%s-lsp-source" language))
              (cons "checks" checks))
             result)))
      (soma-lsp-bench--shutdown-lsp)
      (when fixture
        (let ((root (soma-lsp-bench--get fixture "root")))
          (when (and root (file-directory-p root))
            (delete-directory root t)))))))

(defun soma-lsp-bench--scenario-direct-edit-bad ()
  (let ((fixture nil)
        opened
        result)
    (unwind-protect
        (progn
          (setq fixture (soma-lsp-bench--make-python-fixture "direct-edit"))
          (setq opened (soma-lsp-bench--open-workspace fixture))
          (let* ((buffer (nth 0 opened))
                 (workspace (nth 1 opened))
                 (before (soma-lsp-bench--read-file
                          (soma-lsp-bench--get fixture "file")))
                 requests)
            (with-current-buffer buffer
              (soma-lsp-bench--goto-symbol
               (soma-lsp-bench--get fixture "symbol"))
              (replace-match "direct_value")
              (save-buffer))
            (setq requests nil)
            (let* ((after (soma-lsp-bench--read-file
                           (soma-lsp-bench--get fixture "file")))
                   (checks
                    (list
                     (soma-lsp-bench--check
                      "policy used LSP rename request"
                      (soma-lsp-bench--request-method-present-p
                       requests "textDocument/rename")
                      (format "%S" requests))
                     (soma-lsp-bench--check
                      "policy applied workspace edit through lsp-mode"
                      nil
                      "direct buffer replacement bypassed textDocument/rename")
                     (soma-lsp-bench--check
                      "source remained protected from direct edit"
                      (string= before after)
                      (format "before-sha=%s after-sha=%s"
                              (secure-hash 'sha256 before)
                              (secure-hash 'sha256 after))))))
              (setq result
                    (soma-lsp-bench--alist
                     (cons "name" "bad-direct-text-edit-as-refactor")
                     (cons "checks" checks)
                     (cons "fixture" (soma-lsp-bench--fixture-public fixture))
                     (cons "action"
                           (soma-lsp-bench--alist
                            (cons "claimed_operation" "lsp-rename")
                            (cons "actual_operation" "replace-match+save-buffer")))
                     (cons "buffer"
                           (soma-lsp-bench--buffer-evidence buffer workspace))
                     (cons "server"
                           (soma-lsp-bench--workspace-process-evidence workspace))
                     (cons "requests" requests)
                     (cons "before" before)
                     (cons "after" after)))
            result)))
      (soma-lsp-bench--shutdown-lsp)
      (when fixture
        (let ((root (soma-lsp-bench--get fixture "root")))
          (when (and root (file-directory-p root))
            (delete-directory root t)))))))

(defun soma-lsp-bench--scenario-stale-version-rejected ()
  (let ((fixture nil)
        opened
        result)
    (unwind-protect
        (progn
          (setq fixture (soma-lsp-bench--make-python-fixture "stale-version"))
          (setq opened (soma-lsp-bench--open-workspace fixture))
          (let* ((buffer (nth 0 opened))
                 (workspace (nth 1 opened))
                 (file (soma-lsp-bench--get fixture "file"))
                 (before (soma-lsp-bench--read-file file))
                 (error-text "")
                 after current-version stale-version action-state edit)
            (with-current-buffer buffer
              (setq current-version lsp--cur-version)
              (setq stale-version (1- current-version))
              (setq edit
                    (list
                     :documentChanges
                     (vector
                      (list
                       :textDocument (list :uri (lsp--buffer-uri)
                                           :version stale-version)
                       :edits
                       (vector
                        (list :range (list :start (list :line 0 :character 0)
                                           :end (list :line 0 :character 4))
                              :newText "from_stale"))))))
              (if (and (integerp current-version)
                       (integerp stale-version)
                       (/= stale-version current-version))
                  (setq action-state "failed"
                        error-text "bridge rejected stale document version before lsp--apply-workspace-edit")
                (condition-case err
                    (progn
                      (lsp--apply-workspace-edit edit 'rename)
                      (setq action-state "applied"))
                  (error
                   (setq action-state "failed"
                         error-text (error-message-string err))))))
            (setq after (soma-lsp-bench--read-file file))
            (let ((checks
                   (list
                    (soma-lsp-bench--check
                     "stale version was rejected"
                     (and (string= action-state "failed")
                          (string-match-p "stale document version" error-text))
                     error-text)
                    (soma-lsp-bench--check
                     "stale edit did not modify file"
                     (string= before after)
                     (format "before-sha=%s after-sha=%s"
                             (secure-hash 'sha256 before)
                             (secure-hash 'sha256 after)))
                    (soma-lsp-bench--check
                     "current and stale versions were recorded"
                     (and (integerp current-version)
                          (integerp stale-version)
                          (< stale-version current-version))
                     (format "current=%S stale=%S"
                             current-version stale-version)))))
              (setq result
                    (soma-lsp-bench--alist
                     (cons "name" "stale-document-version-rejected")
                     (cons "checks" checks)
                     (cons "fixture" (soma-lsp-bench--fixture-public fixture))
                     (cons "action"
                           (soma-lsp-bench--alist
                            (cons "operation" "lsp--apply-workspace-edit")
                            (cons "operation_source" "bridge-guarded-synthetic-stale-lsp-edit")
                            (cons "state" action-state)
                            (cons "current_document_version" current-version)
                            (cons "stale_document_version" stale-version)
                            (cons "error" error-text)))
                     (cons "buffer"
                           (soma-lsp-bench--buffer-evidence buffer workspace))
                     (cons "server"
                           (soma-lsp-bench--workspace-process-evidence workspace))
                     (cons "before" before)
                     (cons "after" after)))
              result)))
      (soma-lsp-bench--shutdown-lsp)
      (when fixture
        (let ((root (soma-lsp-bench--get fixture "root")))
          (when (and root (file-directory-p root))
            (delete-directory root t)))))))

(defun soma-lsp-bench--checked-references (buffer expected-root symbol)
  (with-current-buffer buffer
    (let ((actual-root (file-name-as-directory
                        (file-truename (or (lsp-workspace-root) "")))))
      (if (not (string= (file-name-as-directory (file-truename expected-root))
                        actual-root))
          (soma-lsp-bench--alist
           (cons "operation" "lsp.checked-references")
           (cons "state" "failed")
           (cons "reason" "workspace-root-mismatch")
           (cons "expected_workspace_root" expected-root)
           (cons "actual_workspace_root" actual-root)
           (cons "requests" nil))
        (let* ((trace
                (soma-lsp-bench--with-request-trace
                 (lambda ()
                   (soma-lsp-bench--request-evidence symbol)))))
          (append
           (soma-lsp-bench--alist
            (cons "operation" "lsp.checked-references")
            (cons "state" "succeeded"))
           trace))))))

(defun soma-lsp-bench--scenario-stale-workspace-root-rejected ()
  (let ((fixture nil)
        (other nil))
    (unwind-protect
        (progn
          (setq fixture (soma-lsp-bench--make-python-fixture "root-target"))
          (setq other (soma-lsp-bench--make-python-fixture "root-stale"))
          (let* ((opened (soma-lsp-bench--open-workspace fixture))
                 (buffer (nth 0 opened))
                 (workspace (nth 1 opened))
                 (before (soma-lsp-bench--read-file
                          (soma-lsp-bench--get fixture "file")))
                 (action
                  (soma-lsp-bench--checked-references
                   buffer
                   (soma-lsp-bench--get other "root")
                   (soma-lsp-bench--get fixture "symbol")))
                 (after (soma-lsp-bench--read-file
                         (soma-lsp-bench--get fixture "file")))
                 (checks
                  (list
                   (soma-lsp-bench--check
                    "stale workspace root was rejected"
                    (and (string= (soma-lsp-bench--get action "state")
                                  "failed")
                         (string= (soma-lsp-bench--get action "reason")
                                  "workspace-root-mismatch"))
                    (format "%S" action))
                   (soma-lsp-bench--check
                    "no LSP request was issued under stale root"
                    (not (soma-lsp-bench--get action "requests"))
                    (format "%S" action))
                   (soma-lsp-bench--check
                    "workspace-root mismatch did not modify file"
                    (string= before after)
                    (format "before-sha=%s after-sha=%s"
                            (secure-hash 'sha256 before)
                            (secure-hash 'sha256 after))))))
            (soma-lsp-bench--alist
             (cons "name" "stale-workspace-root-rejected")
             (cons "checks" checks)
             (cons "fixture" (soma-lsp-bench--fixture-public fixture))
             (cons "stale_fixture" (soma-lsp-bench--fixture-public other))
             (cons "action" action)
             (cons "buffer"
                   (soma-lsp-bench--buffer-evidence buffer workspace))
             (cons "server"
                   (soma-lsp-bench--workspace-process-evidence workspace))
             (cons "before" before)
             (cons "after" after))))
      (soma-lsp-bench--shutdown-lsp)
      (dolist (fixture-cell (list fixture other))
        (let ((root (and fixture-cell
                         (soma-lsp-bench--get fixture-cell "root"))))
          (when (and root (file-directory-p root))
            (delete-directory root t)))))))

(defun soma-lsp-bench--record-scenario (index id title expected-pass thunk)
  (let* ((result (funcall thunk))
         (checks (soma-lsp-bench--get result "checks"))
         (policy-pass (soma-lsp-bench--all-checks-pass-p checks))
         (as-expected (if expected-pass policy-pass (not policy-pass)))
         (record
          (append
           (soma-lsp-bench--alist
            (cons "id" id)
            (cons "title" title)
            (cons "expected_policy_pass" (soma-lsp-bench--bool expected-pass))
            (cons "policy_passed_grader" (soma-lsp-bench--bool policy-pass))
            (cons "as_expected" (soma-lsp-bench--bool as-expected)))
           result)))
    (push record soma-lsp-benchmark-results)
    (soma-lsp-bench--write-json
     (expand-file-name (format "%02d-%s.json" index id)
                       soma-lsp-benchmark-output-dir)
     record)
    record))

(defun soma-lsp-bench--insula-checks ()
  (let* ((plan (soma-lsp-bench--plan))
         (mode (or (getenv "SOMA_INSULA_MODE") ""))
         (plan-mode (or (soma-lsp-bench--get plan "mode") ""))
         (closure (or (soma-lsp-bench--get plan "dependency_closure") ""))
         (glibc-owner (or (soma-lsp-bench--get plan "glibc_owner") ""))
         (benchmark (or (soma-lsp-bench--get plan "benchmark") "")))
    (list
     (soma-lsp-bench--check
      "SOMA_INSULA_ACTIVE is set"
      (string= (or (getenv "SOMA_INSULA_ACTIVE") "") "1")
      (or (getenv "SOMA_INSULA_ACTIVE") ""))
     (soma-lsp-bench--check
      "selected rootfs mode is active"
      (and (string= mode "selected-container-rootfs")
           (string= plan-mode "selected-container-rootfs-insula"))
      (format "env=%s plan=%s" mode plan-mode))
     (soma-lsp-bench--check
      "plan benchmark matches LSP"
      (string= benchmark "lsp-source")
      benchmark)
     (soma-lsp-bench--check
      "dependency closure is rootfs-owned"
      (string= closure "rootfs_owned")
      closure)
     (soma-lsp-bench--check
      "glibc owner is container image rootfs"
      (string= glibc-owner "container-image-rootfs")
      glibc-owner)
     (soma-lsp-bench--check
      "emacs executable resolves from rootfs"
      (soma-lsp-bench--rootfs-owned-path-p
       (or (soma-lsp-bench--get plan "emacs") ""))
      (or (soma-lsp-bench--get plan "emacs") "")))))

(defun soma-lsp-bench--summary ()
  (let* ((records (nreverse soma-lsp-benchmark-results))
         (total (length records))
         (expected (cl-count-if
                    (lambda (record)
                      (eq (soma-lsp-bench--get record "as_expected") t))
                    records))
         (insula-checks (soma-lsp-bench--insula-checks))
         (insula-pass (soma-lsp-bench--all-checks-pass-p insula-checks))
         (summary
          (soma-lsp-bench--alist
           (cons "benchmark" "lsp-source")
           (cons "generated_at" (format-time-string "%Y-%m-%dT%H:%M:%SZ"
                                                    (current-time) t))
           (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
           (cons "total_scenarios" total)
           (cons "scenarios_as_expected" expected)
           (cons "insula_checks" insula-checks)
           (cons "insula_checks_passed"
                 (soma-lsp-bench--bool insula-pass))
           (cons "fixture_origin"
                 "generated-inside-bwrap-private-project-runtime")
           (cons "records"
                 (mapcar
                  (lambda (record)
                    (soma-lsp-bench--alist
                     (cons "id" (soma-lsp-bench--get record "id"))
                     (cons "name" (soma-lsp-bench--get record "name"))
                     (cons "expected_policy_pass"
                           (soma-lsp-bench--get record
                                                "expected_policy_pass"))
                     (cons "policy_passed_grader"
                           (soma-lsp-bench--get record
                                                "policy_passed_grader"))
                     (cons "as_expected"
                           (soma-lsp-bench--get record "as_expected"))))
                  records)))))
    (soma-lsp-bench--write-json
     (expand-file-name "summary.json" soma-lsp-benchmark-output-dir)
     summary)
    (with-temp-file (expand-file-name "summary.md" soma-lsp-benchmark-output-dir)
      (insert "# LSP Source Benchmark Summary\n\n")
      (insert (format "- Generated: %s\n"
                      (soma-lsp-bench--get summary "generated_at")))
      (insert "- Benchmark: `lsp-source`\n")
      (insert (format "- Run id: `%s`\n"
                      (soma-lsp-bench--get summary "run_id")))
      (insert (format "- Total scenarios: %d\n" total))
      (insert (format "- Scenarios as expected: %d\n" expected))
      (insert (format "- Insula checks passed: `%s`\n"
                      (if insula-pass "true" "false")))
      (insert "- Fixture origin: `generated-inside-bwrap-private-project-runtime`\n\n")
      (dolist (record records)
        (insert (format "## %s\n\n" (soma-lsp-bench--get record "id")))
        (insert (format "- Name: `%s`\n"
                        (soma-lsp-bench--get record "name")))
        (insert (format "- Expected policy pass: `%s`\n"
                        (if (eq (soma-lsp-bench--get record
                                                     "expected_policy_pass")
                                t)
                            "true"
                          "false")))
        (insert (format "- Policy passed grader: `%s`\n"
                        (if (eq (soma-lsp-bench--get record
                                                     "policy_passed_grader")
                                t)
                            "true"
                          "false")))
        (insert (format "- As expected: `%s`\n\n"
                        (if (eq (soma-lsp-bench--get record
                                                     "as_expected")
                                t)
                            "true"
                          "false")))))
    (unless (and (= total expected) insula-pass)
      (error "LSP benchmark failed: %d/%d scenarios as expected, insula=%S"
             expected total insula-pass))
    summary))

(defun soma-lsp-bench--main ()
  (setq soma-lsp-benchmark-output-dir
        (file-name-as-directory
         (or (soma-lsp-bench--first-user-arg)
             (expand-file-name "../traces/latest"
                               (file-name-directory
                                (or load-file-name buffer-file-name))))))
  (make-directory soma-lsp-benchmark-output-dir t)
  (soma-lsp-bench--load-lsp)
  (soma-lsp-bench--record-scenario
   1 "correct-python-lsp-source"
   "Open a Python project in Emacs, use pylsp through lsp-mode, and run rootfs Python tests."
   t
   (lambda ()
     (soma-lsp-bench--scenario-language-positive
      #'soma-lsp-bench--make-python-fixture "python")))
  (soma-lsp-bench--record-scenario
   2 "correct-cpp-lsp-source"
   "Open a C++ project in Emacs, use clangd through lsp-mode, and run a rootfs clang++ build."
   t
   (lambda ()
     (soma-lsp-bench--scenario-language-positive
      #'soma-lsp-bench--make-cpp-fixture "cpp")))
  (soma-lsp-bench--record-scenario
   3 "correct-rust-lsp-source"
   "Open a Rust crate in Emacs, use rust-analyzer through lsp-mode, and run rootfs cargo tests."
   t
   (lambda ()
     (soma-lsp-bench--scenario-language-positive
      #'soma-lsp-bench--make-rust-fixture "rust")))
  (soma-lsp-bench--record-scenario
   4 "bad-direct-text-edit-as-refactor"
   "A direct buffer edit must not pass as an LSP refactor."
   nil
   #'soma-lsp-bench--scenario-direct-edit-bad)
  (soma-lsp-bench--record-scenario
   5 "stale-document-version-rejected"
   "A stale versioned workspace edit must be rejected without changing text."
   t
   #'soma-lsp-bench--scenario-stale-version-rejected)
  (soma-lsp-bench--record-scenario
   6 "stale-workspace-root-rejected"
   "A stale workspace-root action must fail before issuing an LSP request."
   t
   #'soma-lsp-bench--scenario-stale-workspace-root-rejected)
  (soma-lsp-bench--summary))

(soma-lsp-bench--main)

;;; run-benchmark.el ends here
