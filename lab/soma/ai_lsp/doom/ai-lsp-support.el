;;; ai-lsp-support.el --- Pure AI/LSP verifier support -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'json)
(require 'seq)
(require 'subr-x)

(defconst devx-ai-test-language-cases
  '((:name "Rust" :mode rustic-mode :language-id "rust"
     :server rust-analyzer :dependency rust-analyzer :source "src/lib.rs")
    (:name "Python" :mode python-mode :language-id "python"
     :server pyright :dependency pyright :source "main.py")
    (:name "C++" :mode c++-mode :language-id "cpp"
     :server clangd :dependency clangd :source "main.cpp")
    (:name "CUDA" :mode cuda-mode :language-id "cuda-cpp"
     :server clangd :dependency clangd :source "kernel.cu")))

(defconst devx-ai-test-mode-dependencies
  '((rustic-mode . rust-analyzer)
    (python-mode . pyright)
    (c++-mode . clangd)
    (cuda-mode . clangd)))

(defconst devx-ai-test-language-id-cases
  '((rustic-mode . "rust")
    (rust-mode . "rust")
    (rust-ts-mode . "rust")
    (python-mode . "python")
    (python-ts-mode . "python")
    (c++-mode . "cpp")
    (c++-ts-mode . "cpp")
    (cuda-mode . "cuda-cpp")
    (cuda-ts-mode . "cuda-cpp")))

(defconst devx-ai-test-fixtures
  '((rustic-mode
     ("Cargo.toml" . "[package]\nname = \"devx_ai_rust\"\nversion = \"0.1.0\"\nedition = \"2024\"\n")
     ("src/lib.rs" . "pub fn add(left: u64, right: u64) -> u64 { left + right }\n"))
    (python-mode
     ("pyproject.toml" . "[project]\nname = \"devx-ai-python\"\nversion = \"0.1.0\"\n")
     ("main.py" . "def add(left: int, right: int) -> int:\n    return left + right\n"))
    (c++-mode
     ("compile_commands.json" . "[]\n")
     ("main.cpp" . "int add(int left, int right) { return left + right; }\n"))
    (cuda-mode
     ("compile_commands.json" . "[]\n")
     ("kernel.cu" . "__global__ void copy_kernel(float *out, const float *in) { out[threadIdx.x] = in[threadIdx.x]; }\n"))))

(defconst devx-ai-test-result-schema "devx.ai-lsp.result/v1")

(defconst devx-ai-test-repo-root
  (file-name-directory
   (directory-file-name
    (file-name-directory (or load-file-name buffer-file-name)))))

(defconst devx-ai-test-results-buffer-name " *devx-ai-lsp-results*")

(defconst devx-ai-test-gate-names
  '("mode" "backend-exclusivity" "copilot-language-id"
    "lsp-workspace" "lsp-executable" "copilot-connection"
    "copilot-auth" "inline-completion"))

(define-error 'devx-ai-test-external-blocker
  "External Copilot service unavailable")

(defun devx-ai-test-make-result (kind status records failures teardown)
  (list :schema devx-ai-test-result-schema
        :kind kind
        :status status
        :entrypoint t
        :records (vconcat records)
        :failures (vconcat failures)
        :teardown teardown
        :unit nil))

(defun devx-ai-test-make-gates (&optional statuses)
  "Return the fixed gate vector, taking gate values from STATUSES."
  (vconcat
   (mapcar (lambda (name)
             (list :name name
                   :status (or (cdr (assoc name statuses)) "NOT_RUN")))
           devx-ai-test-gate-names)))

(defun devx-ai-test-make-language-record
    (case expected-executable observed-server observed-executable
          observed-language-id gate-statuses status)
  "Return the exact structured result record for CASE."
  (list :language (plist-get case :name)
        :mode (symbol-name (plist-get case :mode))
        :expected-server (symbol-name (plist-get case :server))
        :observed-server (and observed-server (symbol-name observed-server))
        :expected-executable expected-executable
        :observed-executable observed-executable
        :expected-language-id (plist-get case :language-id)
        :observed-language-id observed-language-id
        :gates (devx-ai-test-make-gates gate-statuses)
        :status status))

(defun devx-ai-test-make-failure
    (language boundary expected observed resolved-executable workspaces
              diagnostic classification)
  "Return an allowlisted, credential-safe structured failure."
  (list :language language
        :boundary boundary
        :expected expected
        :observed observed
        :resolved-executable resolved-executable
        :workspaces (vconcat (or workspaces nil))
        :diagnostic (devx-ai-test-complete-line-tail diagnostic 4000)
        :classification classification))

(defun devx-ai-test-result-status (failures teardown)
  "Return FAIL, BLOCKED_EXTERNAL, or PASS for FAILURES and TEARDOWN."
  (let ((classifications
         (mapcar (lambda (failure)
                   (plist-get failure :classification))
                 failures)))
    (cond
     ((or (member "FAIL" classifications)
          (seq-some (lambda (record)
                      (eq t (plist-get record :alive-after-cleanup)))
                    teardown))
      "FAIL")
     ((member "BLOCKED_EXTERNAL" classifications) "BLOCKED_EXTERNAL")
     (t "PASS"))))

(defun devx-ai-test--reset-results ()
  "Reset the verifier-owned transitional PASS buffer."
  (with-current-buffer (get-buffer-create devx-ai-test-results-buffer-name)
    (erase-buffer)))

(defun devx-ai-test-populate-legacy-results (records)
  "Populate the transitional launcher buffer from passing RECORDS."
  (devx-ai-test--reset-results)
  (dolist (record (append records nil))
    (when (equal "PASS" (plist-get record :status))
      (let ((line
             (format "PASS %-6s %s + Copilot %s"
                     (plist-get record :language)
                     (plist-get record :expected-server)
                     (plist-get record :expected-language-id))))
        (with-current-buffer
            (get-buffer-create devx-ai-test-results-buffer-name)
          (goto-char (point-max))
          (insert line "\n"))
        (message "%s" line)))))

(defun devx-ai-test-publish-result (result)
  "Write RESULT when requested and return exactly t."
  (when-let ((path (getenv "DEVX_AI_LSP_RESULT_FILE")))
    (devx-ai-test-write-json path result))
  t)

(defun devx-ai-test-combine-results (unit-result integration-result)
  "Return one all-mode RESULT from UNIT-RESULT and INTEGRATION-RESULT."
  (let ((result
         (if integration-result
             (copy-sequence integration-result)
           (devx-ai-test-make-result "all" "FAIL" nil nil []))))
    (plist-put result :kind "all")
    (plist-put result :status
               (if (equal "PASS" (plist-get unit-result :status))
                   (plist-get result :status)
                 "FAIL"))
    (plist-put result :unit (plist-get unit-result :unit))
    result))

(defun devx-ai-test-write-json (path value)
  (let ((coding-system-for-write 'utf-8-unix)
        (write-region-inhibit-fsync nil)
        (saved-modes (default-file-modes)))
    (unwind-protect
        (progn
          (set-default-file-modes #o600)
          (with-temp-file path
            (insert (json-serialize value
                                    :null-object nil
                                    :false-object :json-false))
            (insert "\n"))
          (set-file-modes path #o600))
      (set-default-file-modes saved-modes))))

(defconst devx-ai-test-secret-regexps
  '("\\b\\(?:authorization\\|access[-_]?token\\|refresh[-_]?token\\|oauth[-_]?token\\|token\\)\\b[ \t]*[:=][ \t]*\\(?:Bearer[ \t]+\\)?[^][{}(),; \t\n]+"
    "\\\"\\(?:access_token\\|refresh_token\\|oauth_token\\|token\\)\\\"[ \t]*:[ \t]*\\\"[^\\\"\n]+\\\""
    "\\bBearer[ \t]+[^][{}(),; \t\n]+"
    "\\b\\(?:gh[opsu]_[[:alnum:]_]+\\|github_pat_[[:alnum:]_]+\\)\\b"
    "[[:alnum:]_+/.=-]\\{32,\\}")
  "Credential-shaped patterns redacted from verifier diagnostics.")

(defun devx-ai-test-sanitize-diagnostic (text)
  "Return TEXT with credential-shaped values redacted."
  (let ((case-fold-search t)
        (safe (or text "")))
    (dolist (regexp devx-ai-test-secret-regexps safe)
      (setq safe (replace-regexp-in-string regexp "<redacted>" safe t t)))))

(defun devx-ai-test-complete-line-tail (text limit)
  "Return a sanitized tail of TEXT no longer than LIMIT characters."
  (let* ((source (or text ""))
         (start (max 0 (- (length source) (* 2 limit))))
         (candidate (substring source start))
         (complete (if (zerop start)
                       candidate
                     (let ((newline (string-match "\n" candidate)))
                       (if newline (substring candidate (1+ newline)) ""))))
         (bounded (if (> (length complete) limit)
                      (let* ((raw-start (- (length complete) limit))
                             (newline (string-match "\n" complete raw-start)))
                        (if newline (substring complete (1+ newline)) ""))
                    complete))
         (safe (devx-ai-test-sanitize-diagnostic bounded)))
    (if (> (length safe) limit)
        (let* ((raw-start (- (length safe) limit))
               (newline (string-match "\n" safe raw-start)))
          (if newline (substring safe (1+ newline)) ""))
      safe)))

(defun devx-ai-test-plist-p (value)
  (and (proper-list-p value)
       (cl-evenp (length value))
       (cl-loop for (key _value) on value by #'cddr always (keywordp key))))

(defun devx-ai-test-position-p (value)
  (and (devx-ai-test-plist-p value)
       (plist-member value :line)
       (plist-member value :character)
       (natnump (plist-get value :line))
       (natnump (plist-get value :character))))

(defun devx-ai-test-command-p (value)
  (and (devx-ai-test-plist-p value)
       (plist-member value :title)
       (stringp (plist-get value :title))
       (plist-member value :command)
       (stringp (plist-get value :command))
       (or (not (plist-member value :arguments))
           (vectorp (plist-get value :arguments)))))

(defun devx-ai-test-completion-item-p (item)
  (and (devx-ai-test-plist-p item)
       (plist-member item :insertText)
       (stringp (plist-get item :insertText))
       (or (not (plist-member item :filterText))
           (stringp (plist-get item :filterText)))
       (or (not (plist-member item :range))
           (let ((range (plist-get item :range)))
             (and (devx-ai-test-plist-p range)
                  (devx-ai-test-position-p (plist-get range :start))
                  (devx-ai-test-position-p (plist-get range :end)))))
       (or (not (plist-member item :command))
           (devx-ai-test-command-p (plist-get item :command)))))

(defun devx-ai-test-valid-completion-response-p
    (response &optional allow-list-array)
  "Return non-nil when RESPONSE has a supported inline completion schema."
  (let ((array-p (lambda (value)
                   (or (vectorp value)
                       (and allow-list-array (proper-list-p value))))))
    (cond
     ((null response) t)
     ((funcall array-p response)
      (cl-every #'devx-ai-test-completion-item-p response))
     ((and (devx-ai-test-plist-p response)
           (plist-member response :items)
           (vectorp (plist-get response :items)))
      (cl-every #'devx-ai-test-completion-item-p
                (plist-get response :items)))
     (t nil))))

(defun devx-ai-test-workspaces-valid-p
    (records expected-server expected-executable)
  "Return non-nil for the exact live expected workspace and executable."
  (and (= 1 (length records))
       (let* ((record (car records))
              (observed (plist-get record :command)))
         (and (eq expected-server (plist-get record :id))
              (eq 'initialized (plist-get record :status))
              (eq t (plist-get record :alive))
              (stringp observed)
              (stringp expected-executable)
              (equal (file-truename observed)
                     (file-truename expected-executable))))))

(defun devx-ai-test-prefixed-symbols-in-form (form prefixes)
  "Return symbols in FORM whose names begin with one of PREFIXES."
  (let (matches)
    (cl-labels
        ((visit
          (value)
          (cond
           ((symbolp value)
            (when (seq-some
                   (lambda (prefix)
                     (string-prefix-p prefix (symbol-name value)))
                   prefixes)
              (push value matches)))
           ((consp value)
            (visit (car value))
            (visit (cdr value)))
           ((vectorp value)
            (mapc #'visit value))
           ((hash-table-p value)
            (maphash (lambda (key item)
                       (visit key)
                       (visit item))
                     value)))))
      (visit form))
    (nreverse matches)))

(defun devx-ai-test-prefixed-symbols-in-source (source prefixes)
  "Read Lisp SOURCE and return symbol uses beginning with PREFIXES.

Comments and string contents are intentionally not symbol uses."
  (with-temp-buffer
    (insert source)
    (emacs-lisp-mode)
    (goto-char (point-min))
    (let (matches)
      (while (progn
               (forward-comment (point-max))
               (not (eobp)))
        (setq matches
              (nconc matches
                     (devx-ai-test-prefixed-symbols-in-form
                      (read (current-buffer)) prefixes))))
      matches)))

(provide 'ai-lsp-support)
