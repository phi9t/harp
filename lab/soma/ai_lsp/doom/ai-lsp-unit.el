;;; ai-lsp-unit.el --- AI/LSP unit contracts -*- lexical-binding: t; -*-

(require 'ert)
(require 'cl-lib)
(require 'seq)
(require 'subr-x)
(require 'ai-lsp-support)
(require 'ai-lsp-adapter)

(defconst devx-ai-test-production-language-ids
  '((rustic-mode . "rust") (rust-mode . "rust") (rust-ts-mode . "rust")
    (python-mode . "python") (python-ts-mode . "python")
    (c++-mode . "cpp") (c++-ts-mode . "cpp")
    (cuda-mode . "cuda-cpp") (cuda-ts-mode . "cuda-cpp")))

(ert-deftest devx-ai-unit-package-pins ()
  "Catch either AI completion package moving off its certified revision."
  (let ((contents
         (with-temp-buffer
           (insert-file-contents
            (expand-file-name "doom/packages.el" devx-ai-test-repo-root))
           (buffer-string))))
    (should (string-match-p
             (regexp-quote
              ":pin \"90f429b100418d897b673d6c36c2817364b28e3e\"")
             contents))
    (should (string-match-p
             (regexp-quote
              ":pin \"32b8c40774515c940a02c78874cf928b356478de\"")
             contents))))

(ert-deftest devx-ai-unit-language-ids ()
  "Catch a missing or incorrect Copilot language compatibility mapping."
  (dolist (case devx-ai-test-production-language-ids)
    (with-temp-buffer
      (setq major-mode (car case))
      (should (equal (cdr case)
                     (devx-ai-adapter-copilot-language-id))))))

(ert-deftest devx-ai-unit-lsp-copilot-client-disabled ()
  "Catch lsp-mode starting a second Copilot language-server client."
  (should (boundp 'lsp-copilot-enabled))
  (should-not lsp-copilot-enabled))

(ert-deftest devx-ai-unit-copilot-tab-accepts-completion ()
  "Catch TAB drifting away from accepting visible Copilot ghost text."
  (should (eq 'copilot-accept-completion
              (lookup-key copilot-completion-map (kbd "<tab>"))))
  (should (eq 'copilot-accept-completion
              (lookup-key copilot-completion-map (kbd "TAB"))))
  (should (eq 'copilot-accept-completion
              (lookup-key copilot-completion-map (kbd "C-f")))))

(ert-deftest devx-ai-unit-automatic-activation-policy ()
  "Catch automatic Copilot activation in ineligible programming buffers."
  (should-not (with-temp-buffer
                (prog-mode)
                (devx-ai-copilot-eligible-p)))
  (should-not (with-temp-buffer
                (prog-mode)
                (setq buffer-file-name "/tmp/read-only.rs"
                      buffer-read-only t)
                (devx-ai-copilot-eligible-p)))
  (should-not (with-temp-buffer
                (prog-mode)
                (setq buffer-file-name "/ssh:example:/tmp/main.rs")
                (devx-ai-copilot-eligible-p)))
  (should-not (with-temp-buffer
                (rename-buffer " devx-ai-hidden-file" t)
                (prog-mode)
                (setq buffer-file-name "/tmp/hidden.rs")
                (devx-ai-copilot-eligible-p)))
  (should (with-temp-buffer
            (rename-buffer "devx-ai-local-file" t)
            (prog-mode)
            (setq buffer-file-name "/tmp/main.rs")
            (devx-ai-copilot-eligible-p))))

(ert-deftest devx-ai-unit-adapter-compatible ()
  "Catch drift in the callable shape of every pinned private API."
  (should (equal t (devx-ai-adapter-check-compatibility))))

(ert-deftest devx-ai-unit-private-api-isolation ()
  "Catch private API symbol usage outside the single adapter boundary."
  (let ((prefixes (list (concat "copilot" "--") (concat "lsp" "--")))
        (adapter (expand-file-name "doom/ai-lsp-adapter.el"
                                   devx-ai-test-repo-root)))
    (dolist (path (directory-files-recursively
                   devx-ai-test-repo-root "\\.el\\'"))
      (unless (file-equal-p path adapter)
        (with-temp-buffer
          (insert-file-contents path)
          (should-not
           (devx-ai-test-prefixed-symbols-in-source
            (buffer-string) prefixes)))))))

(ert-deftest devx-ai-unit-private-symbol-scanner-covers-all-form-positions ()
  "Catch private symbols the former call-only regexp could not see."
  (let* ((prefixes (list (concat "copilot" "--") (concat "lsp" "--")))
         (names
          (list (concat (cadr prefixes) "session")
                (concat (cadr prefixes) "state")
                (concat (car prefixes) "declared")
                (concat (car prefixes) "quoted")
                (concat (cadr prefixes) "function-reference")
                (concat (cadr prefixes) "indirect")
                (concat (car prefixes) "spaced")))
         (source
          (string-join
           (list "(progn"
                 (concat "  " (nth 0 names))
                 (format "  (defvar %s)" (nth 1 names))
                 (format "  (declare-function %s \"dependency\")"
                         (nth 2 names))
                 (format "  '%s" (nth 3 names))
                 (format "  #'%s" (nth 4 names))
                 (format "  (funcall '%s)" (nth 5 names))
                 (format "  (  %s)" (nth 6 names))
                 (format "  \"%signored-string\"" (car prefixes))
                 (format "  ;; %signored-comment" (cadr prefixes))
                 ")")
           "\n"))
         (old-call-regexp (concat "(" (regexp-opt prefixes t)))
         (observed
          (mapcar #'symbol-name
                  (devx-ai-test-prefixed-symbols-in-source source prefixes))))
    (should-not (string-match-p old-call-regexp source))
    (should (equal (sort (copy-sequence names) #'string<)
                   (sort (delete-dups observed) #'string<)))))

(ert-deftest devx-ai-unit-workspace-contract-is-exact ()
  "Reject any workspace set not owned exactly by the expected live server."
  (should
   (devx-ai-test-workspaces-valid-p
    '((:id rust-analyzer :status initialized
       :command "/real/rust-analyzer" :alive t))
    'rust-analyzer "/real/rust-analyzer"))
  (dolist
      (records
       '(()
         ((:id rust-analyzer :status initialized
           :command "/real/rust-analyzer" :alive t)
          (:id copilot :status initialized
           :command "/tmp/copilot" :alive t))
         ((:id rls :status initialized
           :command "/real/rust-analyzer" :alive t))
         ((:id rust-analyzer :status starting
           :command "/real/rust-analyzer" :alive t))
         ((:id rust-analyzer :status initialized
           :command "/real/rust-analyzer" :alive nil))
         ((:id rust-analyzer :status initialized
           :command "/other/rust-analyzer" :alive t))))
    (should-not
     (devx-ai-test-workspaces-valid-p
      records 'rust-analyzer "/real/rust-analyzer"))))

(ert-deftest devx-ai-unit-workspace-contract-canonicalizes-executable ()
  "Accept a symlink alias only when it resolves to the expected executable."
  (let* ((root (make-temp-file "devx-ai-workspace-canonical." t))
         (expected (expand-file-name "rust-analyzer-real" root))
         (alias (expand-file-name "rust-analyzer-alias" root))
         (distinct (expand-file-name "rust-analyzer-distinct" root)))
    (unwind-protect
        (progn
          (with-temp-file expected)
          (with-temp-file distinct)
          (make-symbolic-link expected alias)
          (should
           (devx-ai-test-workspaces-valid-p
            `((:id rust-analyzer :status initialized
               :command ,alias :alive t))
            'rust-analyzer expected))
          (should-not
           (devx-ai-test-workspaces-valid-p
            `((:id rust-analyzer :status initialized
               :command ,distinct :alive t))
            'rust-analyzer expected)))
      (delete-directory root t))))

(ert-deftest devx-ai-unit-completion-response-schema ()
  "Reject malformed JSON-RPC inline-completion responses."
  (dolist (value
           (list nil [] [(:insertText "x")] '(:items [])
                 '(:items [(:insertText "x" :filterText "x")])
                 '(:items [(:insertText "x"
                            :range (:start (:line 0 :character 0)
                                    :end (:line 0 :character 1))
                            :command (:title "Apply" :command "apply"
                                      :arguments []))])))
    (should (devx-ai-test-valid-completion-response-p value)))
  (dolist (value
           (list '(:items nil) '(:items ()) '((:insertText "x"))
                 [(:insertText 7)]
                 [(:insertText "x" :filterText 7)]
                 [(:insertText "x" :range (:start (:line -1 :character 0)
                                                  :end (:line 0 :character 1)))]
                 [(:insertText "x" :command nil)]
                 [(:insertText "x" :command (:title "Apply" :command 7))]
                 [(:insertText "x" :command (:title "Apply" :command "apply"
                                             :arguments 7))]))
    (should-not (devx-ai-test-valid-completion-response-p value))))

(ert-deftest devx-ai-unit-test-seam-can-admit-list-arrays ()
  "Allow Lisp lists only through the explicit test seam."
  (should-not
   (devx-ai-test-valid-completion-response-p '((:insertText "x"))))
  (should
   (devx-ai-test-valid-completion-response-p '((:insertText "x")) t)))

(ert-deftest devx-ai-unit-diagnostic-tail-sanitizes-before-bounding ()
  "Prevent a credential suffix from surviving a tail boundary."
  (let ((tail (devx-ai-test-complete-line-tail
               "access_token=marker-outside-cutoff-visible" 8)))
    (should-not (string-search "visible" tail))
    (should (<= (length tail) 8))))

(ert-deftest devx-ai-unit-diagnostic-tail-bounds-redaction-expansion ()
  "Bound a diagnostic tail after redaction expands its contents."
  (should (<= (length (devx-ai-test-complete-line-tail "token=x" 8)) 8)))

(ert-deftest devx-ai-unit-diagnostic-redaction-and-boundaries ()
  "Redact credential families and return only complete bounded lines."
  (let* ((secrets
          '("Bearer bearer_secret_012345678901234567890123"
            "access_token=access_secret_01234567890123456789"
            "refresh-token: refresh_secret_012345678901234567"
            "oauthToken=oauth_secret_012345678901234567890"
            "\"token\":\"json_secret_01234567890123456789012\""
            "ghp_0123456789abcdefghijklmnopqrstuvwxyz"
            "github_pat_0123456789_abcdefghijklmnopqrstuvwxyz"
            "opaqueSecret0123456789_abcdefghijklmnopqrstuvwxyz"))
         (text (concat "safe-before\n"
                       (string-join secrets "\n")
                       "\nsafe-after\n"))
         (safe (devx-ai-test-sanitize-diagnostic text)))
    (dolist (secret secrets)
      (should-not (string-search secret safe)))
    (should (string-search "safe-before" safe))
    (should (string-search "safe-after" safe)))
  (let* ((partial (concat (make-string 5000 ?x)
                          "ghp_boundary_secret_012345678901234567890\n"
                          "complete-safe-line\n"))
         (tail (devx-ai-test-complete-line-tail partial 4000)))
    (should (string-prefix-p "complete-safe-line" tail))
    (should-not (string-search "boundary_secret" tail))
    (should (<= (length tail) 4000))))

(ert-deftest devx-ai-unit-result-json-schema-and-permissions ()
  "Write the versioned result schema as private, parseable JSON."
  (let ((path (make-temp-file "devx-ai-lsp-result.")))
    (unwind-protect
        (progn
          (devx-ai-test-write-json
           path
           (devx-ai-test-make-result
            "unit" "PASS"
            (list '(:name "Rust" :status "PASS"))
            (list '(:name "CUDA" :message "missing clangd"))
            '(:status "PASS")))
          (should (= #o600 (logand #o777 (file-modes path))))
          (with-temp-buffer
            (insert-file-contents path)
            (let ((value (json-parse-buffer :object-type 'plist
                                            :array-type 'array
                                            :null-object nil)))
              (should (equal '(:schema :kind :status :entrypoint :records :failures
                                :teardown :unit)
                             (cl-loop for (key _field) on value by #'cddr
                                      collect key)))
              (should (equal devx-ai-test-result-schema (plist-get value :schema)))
              (should (equal "unit" (plist-get value :kind)))
              (should (equal "PASS" (plist-get value :status)))
              (should (eq t (plist-get value :entrypoint)))
              (should (equal [(:name "Rust" :status "PASS")]
                             (plist-get value :records)))
              (should (equal [(:name "CUDA" :message "missing clangd")]
                             (plist-get value :failures)))
              (should (equal '(:status "PASS")
                             (plist-get value :teardown)))
              (should-not (plist-get value :unit)))))
      (delete-file path))))

(ert-deftest devx-ai-unit-pass-results-are-verifier-owned ()
  "Catch failed or unrelated records entering the transitional PASS channel."
  (let ((devx-ai-test-results-buffer-name
         (generate-new-buffer-name " *devx-ai-test-results*")))
    (unwind-protect
        (progn
          (message "PASS unrelated-launcher-canary")
          (devx-ai-test-populate-legacy-results
           '((:language "Rust" :expected-server "rust-analyzer"
              :expected-language-id "rust" :status "PASS")
             (:language "CUDA" :expected-server "clangd"
              :expected-language-id "cuda-cpp" :status "FAIL")))
          (with-current-buffer devx-ai-test-results-buffer-name
            (should (equal "PASS Rust   rust-analyzer + Copilot rust\n"
                           (buffer-string)))))
      (when-let ((buffer (get-buffer devx-ai-test-results-buffer-name)))
        (kill-buffer buffer)))))

(ert-deftest devx-ai-unit-lsp-state-summary-is-bounded-and-specific ()
  "Catch LSP failures omitting executable and observed workspace state."
  (cl-letf (((symbol-function 'devx-ai-adapter-workspaces)
             (lambda ()
               '((:id rust-analyzer :status initialized
                  :command "/servers/rust-analyzer" :alive t)
                 (:id rls :status starting
                  :command "/servers/rls" :alive nil)))))
    (should
     (equal
      "resolved executable: /servers/rust-analyzer; observed workspaces: rust-analyzer/initialized/alive=t/executable-match=t, rls/starting/alive=nil/executable-match=nil"
      (devx-ai-test--lsp-state-summary "/servers/rust-analyzer")))))

(ert-deftest devx-ai-unit-copilot-log-tail-redacts-credentials ()
  "Catch Copilot diagnostics exposing token-shaped values."
  (let ((buffer (generate-new-buffer " *devx-ai-copilot-log*"))
        (secret "ghp_0123456789abcdefghijklmnopqrstuvwxyz"))
    (unwind-protect
        (progn
          (with-current-buffer buffer
            (insert "Copilot request failed Authorization: Bearer " secret "\n")
            (insert "Copilot transport closed safely\n"))
          (let ((tail
                 (devx-ai-test--copilot-log-tail (list (buffer-name buffer)))))
            (should (string-match-p "Copilot transport closed safely" tail))
            (should (string-match-p "<redacted>" tail))
            (should-not (string-match-p (regexp-quote secret) tail))
            (should (<= (length tail) 4000))))
      (kill-buffer buffer))))

(ert-deftest devx-ai-unit-backend-selectors-switch-in-both-directions ()
  "Catch either selector leaving the previously active backend enabled."
  (with-temp-buffer
    (let ((copilot-mode nil)
          (tabbymacs-mode t)
          calls)
      (cl-letf (((symbol-function 'copilot-mode)
                 (lambda (argument)
                   (push (list 'copilot argument) calls)
                   (setq copilot-mode (> argument 0))))
                ((symbol-function 'tabbymacs-mode)
                 (lambda (argument)
                   (push (list 'tabby argument) calls)
                   (setq tabbymacs-mode (> argument 0))))
                ((symbol-function 'require) (lambda (&rest _) t)))
        (devx-ai-use-copilot)
        (should copilot-mode)
        (should-not tabbymacs-mode)
        (should (equal '((tabby -1) (copilot 1)) (nreverse calls)))
        (setq calls nil)
        (devx-ai-use-tabby)
        (should-not copilot-mode)
        (should tabbymacs-mode)
        (should (equal '((copilot -1) (tabby 1)) (nreverse calls)))))))

(ert-deftest devx-ai-unit-backend-disable-stops-both-backends ()
  "Catch the disable command leaving either inline backend active."
  (with-temp-buffer
    (let ((copilot-mode t)
          (tabbymacs-mode t)
          calls)
      (cl-letf (((symbol-function 'copilot-mode)
                 (lambda (argument)
                   (push (list 'copilot argument) calls)
                   (setq copilot-mode nil)))
                ((symbol-function 'tabbymacs-mode)
                 (lambda (argument)
                   (push (list 'tabby argument) calls)
                   (setq tabbymacs-mode nil))))
        (devx-ai-disable)
        (should-not copilot-mode)
        (should-not tabbymacs-mode)
        (should (equal '((copilot -1) (tabby -1)) (nreverse calls)))))))

(ert-deftest devx-ai-unit-minibuffer-is-never-eligible ()
  "Catch automatic Copilot activation leaking into a file-like minibuffer."
  (with-current-buffer (window-buffer (minibuffer-window))
    (let ((major-mode 'prog-mode)
          (buffer-file-name "/tmp/minibuffer.rs")
          (buffer-read-only nil))
      (should (minibufferp))
      (should-not (devx-ai-copilot-eligible-p)))))

(ert-deftest devx-ai-unit-aggregation-does-not-short-circuit ()
  "Catch one dependency failure suppressing other language records."
  (let (visited)
    (cl-letf (((symbol-function 'devx-ai-test-verify-case)
               (lambda (case _root _dependency)
                 (push (plist-get case :name) visited)
                 (if (member (plist-get case :name) '("C++" "CUDA"))
                     (list :status "FAIL" :boundary "dependency"
                           :language (plist-get case :name))
                   (list :status "PASS"
                         :language (plist-get case :name))))))
      (let ((result
             (devx-ai-test-run-cases
              devx-ai-test-language-cases "/tmp/root"
              '((rust-analyzer :path "/bin/rust-analyzer")
                (pyright :path "/bin/pyright-langserver")
                (clangd :error "Missing clangd")))))
        (should (equal '("Rust" "Python" "C++" "CUDA")
                       (nreverse visited)))
        (should (= 4 (length (plist-get result :records))))
        (should (= 2 (length (plist-get result :failures))))
        (should (equal "FAIL" (plist-get result :status)))
        (should (equal '("Rust" "Python" "C++" "CUDA")
                       (mapcar (lambda (record)
                                 (plist-get record :language))
                               (plist-get result :records))))))))

(ert-deftest devx-ai-unit-local-failure-precedes-external-blocker ()
  "Catch an external blocker masking a local verification defect."
  (cl-letf (((symbol-function 'devx-ai-test-verify-case)
             (lambda (case _root _dependency)
               (pcase (plist-get case :name)
                 ("Rust" '(:language "Rust" :status "BLOCKED_EXTERNAL"))
                 ("Python" '(:language "Python" :status "FAIL"))
                 (_ (list :language (plist-get case :name)
                          :status "PASS"))))))
    (let ((result
           (devx-ai-test-run-cases
            devx-ai-test-language-cases "/tmp/root"
            '((rust-analyzer :path "/bin/rust-analyzer")
              (pyright :path "/bin/pyright-langserver")
              (clangd :path "/bin/clangd")))))
      (should (equal "FAIL" (plist-get result :status)))
      (should (equal '("BLOCKED_EXTERNAL" "FAIL")
                     (mapcar (lambda (failure)
                               (plist-get failure :classification))
                             (plist-get result :failures)))))))

(ert-deftest devx-ai-unit-failures-are-allowlisted-and-sanitized ()
  "Catch missing failure fields or credential-bearing diagnostics."
  (let ((secret "ghp_0123456789abcdefghijklmnopqrstuvwxyz"))
    (cl-letf (((symbol-function 'devx-ai-test-verify-case)
               (lambda (_case _root _dependency)
                 (error "Authorization: Bearer %s" secret))))
      (let* ((result
              (devx-ai-test-run-cases
               (list (car devx-ai-test-language-cases)) "/tmp/root"
               '((rust-analyzer :path "/bin/rust-analyzer"))))
             (failure (car (plist-get result :failures))))
        (should
         (equal '(:language :boundary :expected :observed
                   :resolved-executable :workspaces :diagnostic :classification)
                (cl-loop for (key _value) on failure by #'cddr collect key)))
        (should (member (plist-get failure :classification)
                        '("FAIL" "BLOCKED_EXTERNAL")))
        (should-not (string-search secret (plist-get failure :diagnostic)))
        (should (string-search "<redacted>"
                               (plist-get failure :diagnostic)))))))

(ert-deftest devx-ai-unit-copilot-transport-errors-are-external-only ()
  "Catch network blockers being local, or arbitrary Lisp errors being external."
  (should-error
   (devx-ai-test--resignal-copilot-error
    '(jsonrpc-error "request failed"
                    (jsonrpc-error-message . "Timed out")))
   :type 'devx-ai-test-external-blocker)
  (should-error
   (devx-ai-test--resignal-copilot-error
    '(wrong-type-argument stringp "service unavailable"))
   :type 'wrong-type-argument)
  (let ((classification
         (condition-case nil
             (devx-ai-test--resignal-copilot-error
              '(wrong-type-argument stringp 7))
           (devx-ai-test-external-blocker "BLOCKED_EXTERNAL")
           (error "FAIL"))))
    (should (equal "FAIL" classification)))
  (let ((classification
         (condition-case nil
             (devx-ai-test--resignal-copilot-error
              '(wrong-type-argument stringp "service unavailable"))
           (devx-ai-test-external-blocker "BLOCKED_EXTERNAL")
           (error "FAIL"))))
    (should (equal "FAIL" classification))))

(ert-deftest devx-ai-unit-copilot-auth-classifies-only-valid-unavailability ()
  "Catch unavailable auth being local or malformed status being external."
  (should (eq t (devx-ai-test--require-copilot-auth '(:status "OK"))))
  (dolist (status '("NotSignedIn" "Unauthorized" "ServiceUnavailable"))
    (should-error
     (devx-ai-test--require-copilot-auth (list :status status))
     :type 'devx-ai-test-external-blocker))
  (let ((classification
         (condition-case nil
             (devx-ai-test--require-copilot-auth '(:status 503))
           (devx-ai-test-external-blocker "BLOCKED_EXTERNAL")
           (error "FAIL"))))
    (should (equal "FAIL" classification)))
  (let ((classification
         (condition-case nil
             (devx-ai-test--require-copilot-auth '(:status "Unexpected"))
           (devx-ai-test-external-blocker "BLOCKED_EXTERNAL")
           (error "FAIL"))))
    (should (equal "FAIL" classification))))

(ert-deftest devx-ai-unit-malformed-completion-records-only-its-type ()
  "Catch malformed completion diagnostics retaining response contents."
  (let ((buffer (generate-new-buffer " *devx-ai-malformed-response*"))
        (secret "ghp_0123456789abcdefghijklmnopqrstuvwxyz"))
    (unwind-protect
        (progn
          (with-current-buffer buffer
            (setq major-mode 'rustic-mode)
            (setq-local copilot-mode t)
            (setq-local tabbymacs-mode nil))
          (cl-letf (((symbol-function 'devx-ai-test--write-fixture)
                     (lambda (&rest _) "/tmp/root/src/lib.rs"))
                    ((symbol-function 'find-file-noselect)
                     (lambda (&rest _) buffer))
                    ((symbol-function 'devx-ai-adapter-copilot-language-id)
                     (lambda () "rust"))
                    ((symbol-function 'devx-ai-adapter-start-lsp)
                     (lambda () t))
                    ((symbol-function 'devx-ai-adapter-workspaces)
                     (lambda ()
                       '((:id rust-analyzer :status initialized
                          :command "/bin/rust-analyzer" :alive t
                          :process nil))))
                    ((symbol-function 'devx-ai-adapter-copilot-alive-p)
                     (lambda () t))
                    ((symbol-function 'devx-ai-adapter-copilot-status)
                     (lambda (_seconds) '(:status "OK")))
                    ((symbol-function 'devx-ai-adapter-inline-completion)
                     (lambda (_seconds) (list :secret secret))))
            (let* ((result
                    (devx-ai-test-run-cases
                     (list (car devx-ai-test-language-cases)) "/tmp/root"
                     '((rust-analyzer :path "/bin/rust-analyzer"))))
                   (failure (car (plist-get result :failures)))
                   (record (car (plist-get result :records))))
              (should (equal "FAIL" (plist-get result :status)))
              (should (equal "inline-completion-schema"
                             (plist-get failure :boundary)))
              (should (equal '(:observed-type "cons")
                             (plist-get failure :observed)))
              (should-not
               (string-search
                secret
                (json-serialize (list :record record :failure failure)))))))
      (kill-buffer buffer))))

(ert-deftest devx-ai-unit-certification-wrong-rust-server-is-scoped ()
  "Catch the Rust mutation changing canonical production configuration."
  (let ((process-environment (copy-sequence process-environment)))
    (setenv "DEVX_AI_LSP_CERTIFY" "1")
    (setenv "DEVX_AI_LSP_MUTATION" "wrong-rust-server")
    (let ((cases (devx-ai-test-effective-language-cases)))
      (should (eq 'rls (plist-get (car cases) :server)))
      (should (eq 'rust-analyzer
                  (plist-get (car devx-ai-test-language-cases) :server)))
      (cl-letf (((symbol-function 'devx-ai-test-verify-case)
                 (lambda (case _root _dependency)
                   (list :language (plist-get case :name)
                         :status (if (eq 'rls (plist-get case :server))
                                     "FAIL"
                                   "PASS")))))
        (let ((result
               (devx-ai-test-run-cases
                cases "/tmp/root"
                '((rust-analyzer :path "/bin/rust-analyzer")
                  (pyright :path "/bin/pyright-langserver")
                  (clangd :path "/bin/clangd")))))
          (should (equal '("Rust" "Python" "C++" "CUDA")
                         (mapcar (lambda (record)
                                   (plist-get record :language))
                                 (plist-get result :records))))
          (should (equal '("Rust")
                         (mapcar (lambda (failure)
                                   (plist-get failure :language))
                                 (plist-get result :failures)))))))))

(ert-deftest devx-ai-unit-certification-missing-clangd-keeps-runnable-cases ()
  "Catch a shared clangd mutation suppressing Rust or Python records."
  (let ((process-environment (copy-sequence process-environment))
        resolved
        visited)
    (setenv "DEVX_AI_LSP_CERTIFY" "1")
    (setenv "DEVX_AI_LSP_MUTATION" "missing-clangd")
    (cl-letf (((symbol-function 'devx-ai-adapter-check-compatibility)
               (lambda () t))
              ((symbol-function 'devx-ai-adapter-resolve-server)
               (lambda (server)
                 (push server resolved)
                 (format "/bin/%s" server)))
              ((symbol-function 'devx-ai-test-verify-case)
               (lambda (case _root dependency)
                 (if (plist-get dependency :error)
                     (list :language (plist-get case :name) :status "FAIL")
                   (push (plist-get case :name) visited)
                   (list :language (plist-get case :name) :status "PASS")))))
      (let* ((dependencies (devx-ai-test--preflight))
             (result (devx-ai-test-run-cases
                      devx-ai-test-language-cases "/tmp/root" dependencies)))
        (should (equal '(rust-analyzer pyright) (nreverse resolved)))
        (should (equal '("Rust" "Python") (nreverse visited)))
        (should (equal '("C++" "CUDA")
                       (mapcar (lambda (failure)
                                 (plist-get failure :language))
                               (plist-get result :failures))))
        (should (equal '("Rust" "Python" "C++" "CUDA")
                       (mapcar (lambda (record)
                                 (plist-get record :language))
                               (plist-get result :records))))))))

(ert-deftest devx-ai-unit-certification-mutations-require-release-guard ()
  "Catch mutation values affecting ordinary verifier runs."
  (let ((process-environment (copy-sequence process-environment)))
    (setenv "DEVX_AI_LSP_CERTIFY" nil)
    (setenv "DEVX_AI_LSP_MUTATION" "wrong-rust-server")
    (should (eq 'rust-analyzer
                (plist-get (car (devx-ai-test-effective-language-cases))
                           :server)))))

(ert-deftest devx-ai-unit-certification-unknown-mutation-is-local ()
  "Catch an unknown guarded mutation being ignored or externally blocked."
  (let ((process-environment (copy-sequence process-environment)))
    (setenv "DEVX_AI_LSP_CERTIFY" "1")
    (setenv "DEVX_AI_LSP_MUTATION" "unknown-mutation")
    (cl-letf (((symbol-function 'devx-ai-adapter-check-compatibility)
               (lambda () t))
              ((symbol-function 'devx-ai-adapter-resolve-server)
               (lambda (_server) (ert-fail "resolver must not run"))))
      (let ((result
             (devx-ai-test-run-cases
              devx-ai-test-language-cases "/tmp/root"
              (devx-ai-test--preflight))))
        (should (= 4 (length (plist-get result :records))))
        (should (= 4 (length (plist-get result :failures))))
        (should (equal "FAIL" (plist-get result :status)))
        (should (seq-every-p
                 (lambda (failure)
                   (equal "FAIL" (plist-get failure :classification)))
                 (plist-get result :failures)))))))

(ert-deftest devx-ai-unit-cleanup-records-stopped-child ()
  "Catch teardown losing the captured pid or running before disconnect."
  (let* ((root (make-temp-file "devx-ai-cleanup." t))
         (path (expand-file-name "main.py" root))
         (buffer nil)
         (process
          (make-process :name "devx-ai-cleanup-stopped"
                        :command '("sh" "-c" "sleep 30")
                        :noquery t))
         (pid (process-id process))
         disconnected
         shutdown-requested)
    (unwind-protect
        (progn
          (with-temp-file path (insert "pass\n"))
          (setq buffer (find-file-noselect path))
          (with-current-buffer buffer (setq-local lsp-mode t))
          (cl-letf (((symbol-function 'lsp-disconnect)
                     (lambda ()
                       (setq disconnected t)
                       (delete-process process)))
                    ((symbol-function
                      'devx-ai-adapter-shutdown-buffer-workspaces)
                     (lambda () (setq shutdown-requested t))))
            (let* ((result
                    (devx-ai-test--cleanup
                     root
                     (list (list :server 'pyright :pid pid
                                 :process process))))
                   (record (car (plist-get result :teardown))))
              (should disconnected)
              (should shutdown-requested)
              (should-not (buffer-live-p buffer))
              (should (equal "pyright" (plist-get record :server)))
              (should (= pid (plist-get record :pid)))
              (should (eq :json-false
                          (plist-get record :alive-after-cleanup)))
              (let ((json-path (make-temp-file "devx-ai-teardown-json.")))
                (unwind-protect
                    (progn
                      (devx-ai-test-write-json
                       json-path
                       (devx-ai-test-make-result
                        "integration" "PASS" nil nil (vector record)))
                      (with-temp-buffer
                        (insert-file-contents json-path)
                        (let* ((value
                                (json-parse-buffer :object-type 'plist
                                                   :array-type 'list
                                                   :null-object :null
                                                   :false-object :false))
                               (teardown (car (plist-get value :teardown))))
                          (should (eq :false
                                      (plist-get teardown
                                                 :alive-after-cleanup))))))
                  (delete-file json-path)))
              (should-not (plist-get result :failures)))))
      (when (process-live-p process) (delete-process process))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p root) (delete-directory root t)))))

(ert-deftest devx-ai-unit-cleanup-reports-vetoed-buffer-kill ()
  "Catch a fixture buffer that vetoes `kill-buffer' without signaling."
  (let* ((root (make-temp-file "devx-ai-cleanup-veto." t))
         (path (expand-file-name "main.py" root))
         buffer)
    (unwind-protect
        (progn
          (with-temp-file path (insert "pass\n"))
          (setq buffer (find-file-noselect path))
          (cl-letf (((symbol-function 'devx-ai-test--kill-fixture-buffer)
                     (lambda (&rest _) nil)))
            (let* ((result (devx-ai-test--cleanup root nil))
                   (failure (car (plist-get result :failures))))
              (should (buffer-live-p buffer))
              (should (equal "cleanup" (plist-get failure :boundary)))
              (should (equal "fixture buffer survived" (plist-get failure :observed)))
              (should (equal "FAIL" (plist-get failure :classification))))))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p root) (delete-directory root t)))))

(ert-deftest devx-ai-unit-cleanup-survivor-is-local-failure ()
  "Catch a surviving observed child being reported as successful teardown."
  (let* ((root (make-temp-file "devx-ai-cleanup-survivor." t))
         (process
          (make-process :name "devx-ai-cleanup-survivor"
                        :command '("sh" "-c" "sleep 30")
                        :noquery t))
         (pid (process-id process)))
    (unwind-protect
        (cl-letf (((symbol-function 'devx-ai-test--wait-until)
                   (lambda (&rest _) nil)))
          (let* ((result
                  (devx-ai-test--cleanup
                   root
                   (list (list :server 'rust-analyzer :pid pid
                               :process process))))
                 (record (car (plist-get result :teardown)))
                 (failure (car (plist-get result :failures))))
            (should (plist-get record :alive-after-cleanup))
            (should (equal "cleanup" (plist-get failure :boundary)))
            (should (equal "FAIL" (plist-get failure :classification)))))
      (when (process-live-p process) (delete-process process))
      (when (file-directory-p root) (delete-directory root t)))))

(ert-deftest devx-ai-unit-unit-result-reports-counts-without-writing ()
  "Catch the internal unit result signaling or omitting ERT counts."
  (let ((process-environment (copy-sequence process-environment))
        wrote)
    (setenv "DEVX_AI_LSP_RESULT_FILE" "/must/not/be/written")
    (cl-letf (((symbol-function 'ert-run-tests-batch)
               (lambda (_selector) 'fake-stats))
              ((symbol-function 'ert-stats-completed)
               (lambda (_stats) 23))
              ((symbol-function 'ert-stats-completed-unexpected)
               (lambda (_stats) 2))
              ((symbol-function 'devx-ai-test-write-json)
               (lambda (&rest _) (setq wrote t))))
      (let ((result (devx-ai-test-run-unit-result)))
        (should (equal "unit" (plist-get result :kind)))
        (should (equal "FAIL" (plist-get result :status)))
        (should (equal '(:completed 23 :unexpected 2)
                       (plist-get result :unit)))
        (should-not wrote)))))

(ert-deftest devx-ai-unit-public-entrypoints-write-results-and-return-t ()
  "Catch any public entrypoint leaking its result or skipping requested JSON."
  (should (fboundp 'devx-ai-test-run-unit-result))
  (should (fboundp 'devx-ai-test-run-integration-result))
  (let* ((process-environment (copy-sequence process-environment))
         (path (make-temp-file "devx-ai-entrypoint."))
         (unit-result
          (devx-ai-test-make-result "unit" "PASS" nil nil []))
         (integration-result
          (devx-ai-test-make-result
           "integration" "PASS"
           (mapcar
            (lambda (case)
              (list :language (plist-get case :name)
                    :mode (symbol-name (plist-get case :mode))
                    :expected-server (symbol-name (plist-get case :server))
                    :observed-server (symbol-name (plist-get case :server))
                    :expected-executable "/bin/server"
                    :observed-executable "/bin/server"
                    :expected-language-id (plist-get case :language-id)
                    :observed-language-id (plist-get case :language-id)
                    :gates [] :status "PASS"))
            devx-ai-test-language-cases)
           nil [])))
    (unwind-protect
        (progn
          (plist-put unit-result :unit '(:completed 23 :unexpected 0))
          (setenv "DEVX_AI_LSP_RESULT_FILE" path)
          (cl-letf (((symbol-function 'devx-ai-test-run-unit-result)
                     (lambda () unit-result))
                    ((symbol-function 'devx-ai-test-run-integration-result)
                     (lambda () integration-result)))
            (should (eq t (devx-ai-test-run-unit)))
            (with-temp-buffer
              (insert-file-contents path)
              (should (equal "unit"
                             (plist-get
                              (json-parse-buffer :object-type 'plist)
                              :kind))))
            (should (eq t (devx-ai-test-run-integration)))
            (with-temp-buffer
              (insert-file-contents path)
              (should (equal "integration"
                             (plist-get
                              (json-parse-buffer :object-type 'plist)
                              :kind))))
            (should (eq t (devx-ai-test-run-all)))
            (with-temp-buffer
              (insert-file-contents path)
              (let ((result (json-parse-buffer :object-type 'plist)))
                (should (equal "all" (plist-get result :kind)))
                (should (equal '(:completed 23 :unexpected 0)
                               (plist-get result :unit)))))))
      (delete-file path))))

(ert-deftest devx-ai-unit-all-stops-before-live-when-unit-fails ()
  "Catch all-mode running live checks after a failed unit precondition."
  (should (fboundp 'devx-ai-test-run-unit-result))
  (should (fboundp 'devx-ai-test-run-integration-result))
  (let ((process-environment (copy-sequence process-environment))
        live-called
        published)
    (setenv "DEVX_AI_LSP_RESULT_FILE" nil)
    (cl-letf (((symbol-function 'devx-ai-test-run-unit-result)
               (lambda ()
                 (let ((result
                        (devx-ai-test-make-result "unit" "FAIL" nil nil [])))
                   (plist-put result :unit '(:completed 23 :unexpected 1)))))
              ((symbol-function 'devx-ai-test-run-integration-result)
               (lambda () (setq live-called t)))
              ((symbol-function 'devx-ai-test-publish-result)
               (lambda (result) (setq published result) t)))
      (should (eq t (devx-ai-test-run-all)))
      (should-not live-called)
      (should (equal "all" (plist-get published :kind)))
      (should (equal "FAIL" (plist-get published :status)))
      (should (equal '(:completed 23 :unexpected 1)
                     (plist-get published :unit))))))

(defun devx-ai-test-run-unit-result ()
  "Run unit contracts and return their structured result without writing."
  (let* ((stats (ert-run-tests-batch "^devx-ai-unit-"))
         (completed (ert-stats-completed stats))
         (unexpected (ert-stats-completed-unexpected stats))
         (result
          (devx-ai-test-make-result
           "unit" (if (zerop unexpected) "PASS" "FAIL") nil nil [])))
    (plist-put result :unit
               (list :completed completed :unexpected unexpected))
    result))

(provide 'ai-lsp-unit)
