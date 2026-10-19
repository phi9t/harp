;;; ai-completion.el --- DevX inline AI completion -*- lexical-binding: t; -*-

;; `copilot.el' owns inline completions. Keep lsp-mode's add-on Copilot client
;; disabled so language-specific LSP servers remain the only lsp-mode clients.
(setq lsp-copilot-enabled nil)

;; The backends are deferred, but their buffer-local mode variables must be
;; dynamically bound before either package is loaded.
(defvar copilot-mode nil)
(defvar tabbymacs-mode nil)

(use-package! copilot
  :defer t
  :config
  (dolist (key '("C-<tab>" "C-TAB"))
    (keymap-unset copilot-completion-map key))
  (keymap-set copilot-completion-map "<tab>" #'copilot-accept-completion)
  (keymap-set copilot-completion-map "TAB" #'copilot-accept-completion)
  (keymap-set copilot-completion-map "C-f" #'copilot-accept-completion)
  (keymap-set copilot-completion-map "M-f" #'copilot-accept-completion-by-word)
  (keymap-set copilot-completion-map "C-e" #'copilot-accept-completion-by-line)
  ;; copilot.el 0.9.0 looks up a major-mode symbol in an alist whose upstream
  ;; keys are strings. Symbol aliases fix mapped modes without removing the
  ;; string entries that a future upstream correction will use.
  (dolist (entry '((rustic-mode . "rust")
                   (rust-mode . "rust")
                   (rust-ts-mode . "rust")
                   (python-mode . "python")
                   (python-ts-mode . "python")
                   (c++-mode . "cpp")
                   (c++-ts-mode . "cpp")
                   (cuda-mode . "cuda-cpp")
                   (cuda-ts-mode . "cuda-cpp")))
    (setf (alist-get (car entry) copilot-major-mode-alist nil nil #'eq)
          (cdr entry))))

(use-package! tabbymacs
  :defer t
  :custom
  (tabbymacs-inline-completion-idle-time 0.6)
  (tabbymacs-log-level :warning)
  :config
  (keymap-unset tabbymacs-overlay-map "<tab>")
  (keymap-set tabbymacs-overlay-map "C-f" #'tabbymacs-accept-ghost-text))

(defun devx-ai-use-copilot ()
  "Use Copilot for inline completion in the current buffer."
  (interactive)
  (when (bound-and-true-p tabbymacs-mode)
    (tabbymacs-mode -1))
  (require 'copilot)
  (copilot-mode 1)
  (when (called-interactively-p 'interactive)
    (message "AI completion: Copilot")))

(defun devx-ai-use-tabby ()
  "Use Tabby for inline completion in the current buffer."
  (interactive)
  (when (bound-and-true-p copilot-mode)
    (copilot-mode -1))
  (require 'tabbymacs)
  (tabbymacs-mode 1)
  (when (called-interactively-p 'interactive)
    (message "AI completion: Tabby")))

(defun devx-ai-disable ()
  "Disable inline AI completion in the current buffer."
  (interactive)
  (when (bound-and-true-p copilot-mode)
    (copilot-mode -1))
  (when (bound-and-true-p tabbymacs-mode)
    (tabbymacs-mode -1))
  (message "AI completion disabled"))

(defun devx-ai-copilot-eligible-p ()
  "Return non-nil when the current buffer should start Copilot automatically."
  (and buffer-file-name
       (or (eq major-mode 'prog-mode)
           (derived-mode-p 'prog-mode))
       (not buffer-read-only)
       (not (minibufferp))
       (not (string-prefix-p " " (buffer-name)))
       (not (file-remote-p buffer-file-name))))

(defun devx-ai-enable-copilot-maybe ()
  "Enable Copilot automatically when `devx-ai-copilot-eligible-p' allows it."
  (when (devx-ai-copilot-eligible-p)
    (devx-ai-use-copilot)))

(defvar devx-ai-map
  (let ((map (make-sparse-keymap)))
    (keymap-set map "c" #'devx-ai-use-copilot)
    (keymap-set map "t" #'devx-ai-use-tabby)
    (keymap-set map "0" #'devx-ai-disable)
    map)
  "Keymap for selecting an inline AI completion backend.")

(keymap-global-set "C-c a" devx-ai-map)
(add-hook 'prog-mode-hook #'devx-ai-enable-copilot-maybe)

(provide 'devx-ai-completion)
