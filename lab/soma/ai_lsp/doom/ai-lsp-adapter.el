;;; ai-lsp-adapter.el --- Pinned private API boundary -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'seq)
(eval-when-compile (require 'copilot))

(defvar lsp--buffer-workspaces)
(defvar lsp--session)
(defvar lsp-session-file)

(declare-function copilot--connection-alivep "copilot" ())
(declare-function copilot--ensure-doc-open "copilot" ())
(declare-function copilot--get-language-id "copilot" ())
(declare-function copilot--inline-completion-params "copilot" (position))
(declare-function lsp "lsp-mode" ())
(declare-function lsp--client-server-id "lsp-mode" (client))
(declare-function lsp--workspace-client "lsp-mode" (workspace))
(declare-function lsp--workspace-proc "lsp-mode" (workspace))
(declare-function lsp--workspace-status "lsp-mode" (workspace))
(declare-function lsp-package-path "lsp-mode" (package))
(declare-function lsp-workspace-shutdown "lsp-mode" (workspace))

(defconst devx-ai-adapter-required-functions
  '((copilot--connection-alivep . 0)
    (copilot--ensure-doc-open . 0)
    (copilot--get-language-id . 0)
    (copilot--inline-completion-params . 1)
    (lsp--client-server-id . 1)
    (lsp--workspace-client . 1)
    (lsp--workspace-status . 1)
    (lsp--workspace-proc . 1)
    (lsp-package-path . 1)
    (lsp-workspace-shutdown . 1)))

(defconst devx-ai-adapter-required-macros
  '(copilot--request))

(defun devx-ai-adapter-accepts-arity-p (function arity)
  "Return non-nil when FUNCTION accepts ARITY arguments."
  (pcase-let ((`(,minimum . ,maximum) (func-arity function)))
    (and (<= minimum arity)
         (or (eq maximum 'many) (<= arity maximum)))))

(defun devx-ai-adapter-check-compatibility ()
  "Require and validate every pinned dependency API used by this adapter."
  (require 'copilot)
  (require 'lsp-mode)
  (let ((bad-functions
         (seq-filter
          (lambda (entry)
            (let ((function (car entry))
                  (arity (cdr entry)))
              (or (not (fboundp function))
                  (macrop function)
                  (not (condition-case nil
                           (devx-ai-adapter-accepts-arity-p function arity)
                         (error nil))))))
          devx-ai-adapter-required-functions))
        (bad-macros (seq-remove #'macrop devx-ai-adapter-required-macros)))
    (if (or bad-functions bad-macros)
        (error "Incompatible pinned dependency API: functions=%S macros=%S"
               (mapcar #'car bad-functions) bad-macros)
      t)))

(defun devx-ai-adapter-copilot-language-id ()
  "Return Copilot's language identifier for the current buffer."
  (copilot--get-language-id))

(defun devx-ai-adapter-copilot-alive-p ()
  "Return non-nil when Copilot's connection is alive."
  (copilot--connection-alivep))

(defun devx-ai-adapter-copilot-status (seconds)
  "Return Copilot's status, waiting at most SECONDS."
  (copilot--request 'checkStatus nil :timeout seconds))

(defun devx-ai-adapter-inline-completion (seconds)
  "Request an inline completion, waiting at most SECONDS."
  (copilot--ensure-doc-open)
  (copilot--request 'textDocument/inlineCompletion
                    (copilot--inline-completion-params 1)
                    :timeout seconds))

(defun devx-ai-adapter-start-lsp ()
  "Start lsp-mode for the current buffer."
  (lsp))

(defun devx-ai-adapter-workspaces ()
  "Return normalized lsp-mode workspace records for the current buffer."
  (mapcar
   (lambda (workspace)
     (let* ((client (lsp--workspace-client workspace))
            (process (ignore-errors (lsp--workspace-proc workspace)))
            (command (and (processp process) (process-command process))))
       (list :id (lsp--client-server-id client)
             :status (lsp--workspace-status workspace)
             :command (and command (car command))
             :process process
             :alive (and (processp process) (process-live-p process) t))))
   lsp--buffer-workspaces))

(defun devx-ai-adapter-shutdown-buffer-workspaces ()
  "Stop every LSP workspace attached to the current fixture buffer."
  (let ((+lsp-defer-shutdown 0))
    (dolist (workspace lsp--buffer-workspaces)
      (lsp-workspace-shutdown workspace))))

(defun devx-ai-adapter-reset-session (session-file)
  "Reset lsp-mode state to use SESSION-FILE."
  (setq lsp-session-file session-file
        lsp--session nil))

(defun devx-ai-adapter-resolve-server (server)
  "Resolve the executable path for supported SERVER."
  (pcase server
    ('rust-analyzer
     (require 'lsp-rust)
     (executable-find "rust-analyzer"))
    ('pyright
     (require 'lsp-pyright)
     (ignore-errors (lsp-package-path 'pyright)))
    ('clangd
     (require 'lsp-clangd)
     (ignore-errors (lsp-package-path 'clangd)))
    (_ (error "Unsupported verifier server: %S" server))))

(provide 'ai-lsp-adapter)
