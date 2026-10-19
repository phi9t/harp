;;; ai-lsp-verifier.el --- AI/LSP verification entrypoints -*- lexical-binding: t; -*-

(defconst devx-ai-test-directory
  (file-name-directory (or load-file-name buffer-file-name)))

(dolist (file '("ai-lsp-support.el" "ai-lsp-adapter.el"
                "ai-lsp-unit.el" "ai-lsp-integration.el"))
  (load (expand-file-name file devx-ai-test-directory) nil 'nomessage))

(defun devx-ai-test-run-unit ()
  "Run unit verification, optionally write its result, and return exactly t."
  (devx-ai-test-publish-result (devx-ai-test-run-unit-result)))

(defun devx-ai-test-run-integration ()
  "Run live verification, publish its result, and return exactly t."
  (let ((result (devx-ai-test-run-integration-result)))
    (devx-ai-test-populate-legacy-results (plist-get result :records))
    (devx-ai-test-publish-result result)))

(defun devx-ai-test-run-all ()
  "Require passing unit checks, then run and publish combined verification."
  (let* ((unit-result (devx-ai-test-run-unit-result))
         (integration-result
          (and (equal "PASS" (plist-get unit-result :status))
               (devx-ai-test-run-integration-result)))
         (result
          (devx-ai-test-combine-results unit-result integration-result)))
    (devx-ai-test-populate-legacy-results
     (and integration-result (plist-get integration-result :records)))
    (devx-ai-test-publish-result result)))

(provide 'ai-lsp-verifier)
