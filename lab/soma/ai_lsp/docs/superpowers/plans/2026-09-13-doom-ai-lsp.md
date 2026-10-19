# Doom AI Completion and LSP Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the Doom Copilot/Tabbymacs setup into `~/workspace/devx`, correct Copilot language IDs, and provide a repeatable real-server verifier for Rust, Python, C++, and CUDA.

**Architecture:** Doom retains two small loader blocks while this repository owns package declarations, runtime configuration, tests, and the verifier launcher. The launcher creates a uniquely named Doom daemon; the Emacs Lisp verifier creates disposable language fixtures and checks the language-specific LSP workspace and Copilot connection concurrently in each buffer.

**Tech Stack:** Doom Emacs Lisp, ERT, `lsp-mode`, `copilot.el`, Tabbymacs, Bash, Rust Analyzer, Pyright, Clangd.

**Spec:** `docs/superpowers/specs/2026-09-13-doom-ai-lsp-design.md`

## Global Constraints

- Only AI completion package declarations and configuration move out of `~/.doom.d`.
- Copilot remains the automatic default; Tabbymacs remains manually selectable.
- Automatic activation is limited to writable, local, file-visiting programming buffers.
- `lsp-copilot-enabled` remains disabled so `lsp-mode` does not start a second Copilot client.
- Rust, Python, C++, and CUDA use Copilot IDs `rust`, `python`, `cpp`, and `cuda-cpp` respectively.
- The integration verifier uses temporary source roots, a temporary LSP session, bounded waits, and a uniquely named Doom daemon.
- Empty Copilot completion results are valid; authentication, transport, protocol, and timeout errors fail verification.
- CUDA verification uses Clangd and does not require `nvcc`.
- Authentication tokens and the Copilot auth database must never be printed.

---

## File Map

- `doom/packages.el`: Doom package declarations only.
- `doom/ai-completion.el`: Runtime configuration, language compatibility aliases, selectors, keymap, and guarded automatic activation.
- `test/ai-lsp-verifier.el`: ERT unit tests and the real-server integration verifier.
- `bin/verify-ai-lsp`: Isolated-daemon launcher and cleanup boundary.
- `README.md`: Installation, usage, expected matrix, and troubleshooting.
- `~/.doom.d/packages.el`: Replace the current Copilot/Tabbymacs declarations with a checked loader.
- `~/.doom.d/config.el`: Replace the current AI completion block with a checked loader.

### Task 1: Add the Failing Configuration Contract Tests

**Files:**
- Create: `test/ai-lsp-verifier.el`
- Create: `bin/verify-ai-lsp`

**Interfaces:**
- Consumes: the Doom configuration loaded by a fresh daemon.
- Produces: `devx-ai-test-run-unit`, which returns `t` only when all unit contracts pass; `bin/verify-ai-lsp unit`, which propagates that result as its process exit code.

- [ ] **Step 1: Add the unit tests before moving or fixing configuration**

Start `test/ai-lsp-verifier.el` with these contracts. The production changes that make these tests pass are the new guarded activation function and symbol-key Copilot mappings.

```elisp
;;; ai-lsp-verifier.el --- Verify DevX AI/LSP integration -*- lexical-binding: t; -*-

(require 'ert)
(require 'cl-lib)
(require 'subr-x)

(defconst devx-ai-test-language-cases
  '((:name "Rust"   :mode rustic-mode   :language-id "rust"     :server rust-analyzer)
    (:name "Python" :mode python-mode   :language-id "python"   :server pyright)
    (:name "C++"    :mode c++-mode      :language-id "cpp"      :server clangd)
    (:name "CUDA"   :mode cuda-mode     :language-id "cuda-cpp" :server clangd)))

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

(ert-deftest devx-ai-unit-language-ids ()
  "Catch a missing or incorrect Copilot language compatibility mapping."
  (require 'copilot)
  (dolist (case devx-ai-test-language-id-cases)
    (with-temp-buffer
      (setq major-mode (car case))
      (should (equal (cdr case)
                     (copilot--get-language-id))))))

(ert-deftest devx-ai-unit-lsp-copilot-client-disabled ()
  "Catch lsp-mode starting a second Copilot language-server client."
  (should (boundp 'lsp-copilot-enabled))
  (should-not lsp-copilot-enabled))

(ert-deftest devx-ai-unit-automatic-activation-policy ()
  "Catch automatic Copilot activation in non-file, read-only, or remote buffers."
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
  (should (with-temp-buffer
            (prog-mode)
            (setq buffer-file-name "/tmp/main.rs")
            (devx-ai-copilot-eligible-p))))

(ert-deftest devx-ai-unit-backends-remain-exclusive ()
  "Catch enabling Tabbymacs without disabling an active Copilot mode."
  (with-temp-buffer
    (let ((copilot-mode t)
          (tabbymacs-mode nil)
          copilot-disabled
          tabby-enabled)
      (cl-letf (((symbol-function 'copilot-mode)
                 (lambda (arg) (setq copilot-disabled (= arg -1)
                                    copilot-mode nil)))
                ((symbol-function 'tabbymacs-mode)
                 (lambda (arg) (setq tabby-enabled (= arg 1)
                                    tabbymacs-mode t)))
                ((symbol-function 'require) (lambda (&rest _) t)))
        (devx-ai-use-tabby)
        (should copilot-disabled)
        (should tabby-enabled)
        (should-not copilot-mode)))))

(defun devx-ai-test-run-unit ()
  "Run DevX unit contracts in the current Doom process."
  (let ((stats (ert-run-tests-batch "^devx-ai-unit-")))
    (if (zerop (ert-stats-completed-unexpected stats))
        t
      (error "DevX AI unit verification failed: %d unexpected result(s)"
             (ert-stats-completed-unexpected stats)))))

(provide 'ai-lsp-verifier)
```

- [ ] **Step 2: Add the minimum isolated-daemon launcher needed to run unit tests**

Create `bin/verify-ai-lsp` with strict shell handling, safe Emacs discovery, an isolated server name, and a cleanup trap:

```bash
#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
mode="${1:-all}"
case "$mode" in
  unit|integration|all) ;;
  *) printf 'usage: %s [unit|integration|all]\n' "$0" >&2; exit 64 ;;
esac

find_emacs() {
  if [[ -n "${DEVX_EMACS:-}" && -x "$DEVX_EMACS" ]]; then
    printf '%s\n' "$DEVX_EMACS"
    return
  fi
  if command -v emacs >/dev/null 2>&1; then
    command -v emacs
    return
  fi
  local doom_env="$HOME/.emacs.d/.local/env" saved_path directory
  [[ -r "$doom_env" ]] || {
    printf 'Missing Doom environment file: %s\n' "$doom_env" >&2
    return 1
  }
  saved_path="$(sed -n 's/^ "PATH=\(.*\)"$/\1/p' "$doom_env" | head -n 1)"
  IFS=: read -r -a directories <<<"$saved_path"
  for directory in "${directories[@]}"; do
    if [[ -x "$directory/emacs" ]]; then
      printf '%s\n' "$directory/emacs"
      return
    fi
  done
  printf 'Unable to find Emacs; set DEVX_EMACS to its absolute path.\n' >&2
  return 1
}

emacs_bin="$(find_emacs)"
emacsclient_bin="$(dirname -- "$emacs_bin")/emacsclient"
[[ -x "$emacsclient_bin" ]] || {
  printf 'Missing emacsclient beside %s\n' "$emacs_bin" >&2
  exit 69
}

server_name="devx-ai-lsp-${BASHPID}"
run_root="$(mktemp -d "${TMPDIR:-/tmp}/devx-ai-lsp.XXXXXXXX")"
daemon_log="$run_root/doom-daemon.log"

cleanup() {
  "$emacsclient_bin" -s "$server_name" -e '(kill-emacs)' >/dev/null 2>&1 || true
  if [[ "$run_root" == "${TMPDIR:-/tmp}"/devx-ai-lsp.* && -d "$run_root" ]]; then
    rm -rf -- "$run_root"
  fi
}
trap cleanup EXIT INT TERM

"$emacs_bin" --init-directory "$HOME/.emacs.d" "--daemon=$server_name" \
  >"$daemon_log" 2>&1

test_file="$repo_root/doom/ai-lsp-verifier.el"
case "$mode" in
  unit) entrypoint=devx-ai-test-run-unit ;;
  integration) entrypoint=devx-ai-test-run-integration ;;
  all) entrypoint=devx-ai-test-run-all ;;
esac

if ! "$emacsclient_bin" -s "$server_name" -e \
  "(progn (load-file \"$test_file\") ($entrypoint))"; then
  printf '\nDoom daemon log:\n' >&2
  tail -n 120 "$daemon_log" >&2
  exit 1
fi
```

- [ ] **Step 3: Make the launcher executable and run the unit suite to verify RED**

Run:

```bash
chmod +x bin/verify-ai-lsp
bin/verify-ai-lsp unit
```

Expected: non-zero exit. `devx-ai-unit-language-ids` reports `rustic`, `c++`, or `cuda` instead of the required IDs, and `devx-ai-unit-automatic-activation-policy` reports `devx-ai-copilot-eligible-p` as undefined.

- [ ] **Step 4: Commit the proven failing contracts**

```bash
git add bin/verify-ai-lsp test/ai-lsp-verifier.el
git commit -m "test: define AI completion contracts"
```

### Task 2: Move and Correct the Doom AI Configuration

**Files:**
- Create: `doom/packages.el`
- Create: `doom/ai-completion.el`
- Modify: `~/.doom.d/packages.el`
- Modify: `~/.doom.d/config.el`

**Interfaces:**
- Consumes: Doom's `package!` and `use-package!` macros plus `copilot.el` and Tabbymacs.
- Produces: `devx-ai-use-copilot`, `devx-ai-use-tabby`, `devx-ai-disable`, `devx-ai-copilot-eligible-p`, and `devx-ai-enable-copilot-maybe`.

- [ ] **Step 1: Add repository-owned package declarations**

Create `doom/packages.el`:

```elisp
;; -*- no-byte-compile: t; -*-
;;; packages.el --- DevX AI completion packages

(package! copilot
  :recipe (:host github :repo "copilot-emacs/copilot.el" :files ("*.el")))
(package! tabbymacs
  :recipe (:host github :repo "Bastillan/tabbymacs" :files ("*.el")))
```

- [ ] **Step 2: Add repository-owned runtime configuration**

Create `doom/ai-completion.el` with the existing keys and selectors, the guarded activation policy, and symbol-key aliases that coexist with upstream string entries:

```elisp
;;; ai-completion.el --- DevX inline AI completion -*- lexical-binding: t; -*-

;; `copilot.el' owns inline completions. Keep lsp-mode's add-on Copilot client
;; disabled so language-specific LSP servers remain the only lsp-mode clients.
(setq lsp-copilot-enabled nil)

(use-package! copilot
  :defer t
  :config
  (dolist (key '("<tab>" "TAB" "C-<tab>" "C-TAB"))
    (keymap-unset copilot-completion-map key))
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
       (derived-mode-p 'prog-mode)
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
```

- [ ] **Step 3: Replace inline package declarations with a checked loader**

In `~/.doom.d/packages.el`, remove only the existing Copilot and Tabbymacs block and insert:

```elisp
(let ((devx-ai-packages
       (expand-file-name "~/workspace/devx/doom/packages.el")))
  (unless (file-readable-p devx-ai-packages)
    (error "DevX AI package declarations are missing: %s" devx-ai-packages))
  (load devx-ai-packages nil 'nomessage))
```

- [ ] **Step 4: Replace inline runtime configuration with a checked loader**

In `~/.doom.d/config.el`, remove only lines belonging to the existing AI code completion section and insert at the same location:

```elisp
(let ((devx-ai-config
       (expand-file-name "~/workspace/devx/doom/ai-completion.el")))
  (unless (file-readable-p devx-ai-config)
    (error "DevX AI completion configuration is missing: %s" devx-ai-config))
  (load devx-ai-config nil 'nomessage))
```

- [ ] **Step 5: Run the unit suite to verify GREEN**

```bash
bin/verify-ai-lsp unit
```

Expected: exit 0 with four expected ERT results and no unexpected results.

- [ ] **Step 6: Verify the migration did not duplicate AI declarations or hooks**

```bash
rg -n "package! (copilot|tabbymacs)|devx-ai-packages" \
  doom/packages.el ~/.doom.d/packages.el
rg -n "prog-mode-hook|devx-ai-config|devx-ai-enable-copilot-maybe" \
  doom/ai-completion.el ~/.doom.d/config.el
```

Expected: package declarations exist only in `doom/packages.el`; the automatic AI hook exists only in `doom/ai-completion.el`; each Doom file contains one loader.

- [ ] **Step 7: Commit the repository-owned configuration**

```bash
git add doom/packages.el doom/ai-completion.el
git commit -m "fix: integrate Copilot with language modes"
```

### Task 3: Add Real LSP and Copilot Integration Verification

**Files:**
- Modify: `test/ai-lsp-verifier.el`
- Modify: `bin/verify-ai-lsp`

**Interfaces:**
- Consumes: `devx-ai-test-language-cases`, Doom's mode hooks, `lsp-mode`, language client packages, and the authenticated `copilot.el` server.
- Produces: `devx-ai-test-run-integration` and `devx-ai-test-run-all`; both return `t` on success and signal one aggregated error after running every selected language on failure.

- [ ] **Step 1: Add deterministic fixture definitions and bounded wait helpers**

Extend `test/ai-lsp-verifier.el` with fixture content and helpers whose expected values are literal rather than derived from production configuration:

```elisp
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
  "Return a bounded, token-free diagnostic tail from BUFFER-NAME."
  (when-let* ((buffer (get-buffer buffer-name)))
    (with-current-buffer buffer
      (buffer-substring-no-properties
       (max (point-min) (- (point-max) 4000))
       (point-max)))))
```

- [ ] **Step 2: Add the real per-language verification path**

Implement `devx-ai-test--verify-case` so it performs these exact operations:

```elisp
(defun devx-ai-test--verify-case (case fixture-root)
  "Verify one language CASE below FIXTURE-ROOT and return its name."
  (let* ((name (plist-get case :name))
         (expected-mode (plist-get case :mode))
         (expected-language (plist-get case :language-id))
         (expected-server (plist-get case :server))
         (project-root (expand-file-name (downcase name) fixture-root))
         (source (devx-ai-test--write-fixture project-root expected-mode))
         (buffer (find-file-noselect source)))
    (with-current-buffer buffer
      (unless (eq major-mode expected-mode)
        (error "%s: expected mode %S, observed %S" name expected-mode major-mode))
      (unless (bound-and-true-p copilot-mode)
        (error "%s: Copilot mode is disabled" name))
      (when (bound-and-true-p tabbymacs-mode)
        (error "%s: Tabbymacs and Copilot are active together" name))
      (unless (equal expected-language (copilot--get-language-id))
        (error "%s: expected Copilot language %S, observed %S"
               name expected-language (copilot--get-language-id)))
      (lsp)
      (unless (devx-ai-test--wait-until
               (lambda ()
                 (cl-some
                  (lambda (workspace)
                    (and (eq expected-server
                             (lsp--client-server-id
                              (lsp--workspace-client workspace)))
                         (eq 'initialized (lsp--workspace-status workspace))))
                  lsp--buffer-workspaces))
               30)
        (error "%s: %S did not initialize within 30 seconds\n%s"
               name expected-server
               (or (devx-ai-test--log-tail "*lsp-log*") "No LSP log")))
      (unless (copilot--connection-alivep)
        (error "%s: Copilot server connection is not alive" name))
      (let ((status (copilot--request 'checkStatus nil :timeout 15)))
        (unless (equal "OK" (plist-get status :status))
          (error "%s: Copilot authentication status is %S"
                 name (plist-get status :status))))
      (copilot--ensure-doc-open)
      (copilot--request 'textDocument/inlineCompletion
                        (copilot--inline-completion-params 1)
                        :timeout 30)
      (message "PASS %-6s %S + Copilot %s"
               name expected-server expected-language)
      name)))
```

- [ ] **Step 3: Add dependency preflight, failure aggregation, and cleanup**

Add these entry points. Preflight calls `lsp-package-path` after requiring each client, so Doom-local Clangd/Pyright installations are accepted even when their binaries are not globally visible:

```elisp
(defun devx-ai-test--preflight ()
  "Return non-nil when every required server dependency is available."
  (require 'lsp-mode)
  (require 'lsp-rust)
  (require 'lsp-pyright)
  (require 'lsp-clangd)
  (require 'copilot)
  (dolist (dependency '((rust-analyzer . "rust-analyzer")
                        (pyright . "pyright-langserver")
                        (clangd . "clangd")))
    (unless (pcase (car dependency)
              ('rust-analyzer (executable-find (cdr dependency)))
              (_ (ignore-errors (lsp-package-path (car dependency)))))
      (error "Missing %s; install it before verification" (cdr dependency))))
  t)

(defun devx-ai-test-run-integration ()
  "Run all real language-server plus Copilot checks in this isolated daemon."
  (devx-ai-test--preflight)
  (let* ((fixture-root (make-temp-file "devx-ai-lsp-fixtures." t))
         (lsp-session-file (expand-file-name "lsp-session-v1" fixture-root))
         (lsp-auto-guess-root t)
         (lsp-keep-workspace-alive nil)
         failures)
    (unwind-protect
        (progn
          (setq lsp--session nil)
          (dolist (case devx-ai-test-language-cases)
            (condition-case err
                (devx-ai-test--verify-case case fixture-root)
              (error
               (push (format "%s: %s"
                             (plist-get case :name)
                             (error-message-string err))
                     failures))))
          (if failures
              (error "DevX AI/LSP verification failed:\n%s"
                     (string-join (nreverse failures) "\n"))
            t))
      (dolist (buffer (buffer-list))
        (when-let* ((file (buffer-file-name buffer))
                    ((file-in-directory-p file fixture-root)))
          (kill-buffer buffer)))
      (delete-directory fixture-root t))))

(defun devx-ai-test-run-all ()
  "Run unit and integration verification."
  (and (devx-ai-test-run-unit)
       (devx-ai-test-run-integration)))
```

- [ ] **Step 4: Prove the integration assertion fails for a wrong server contract**

Temporarily evaluate this only in a disposable verifier daemon; do not edit the committed matrix:

```elisp
(setf (plist-get (car devx-ai-test-language-cases) :server) 'wrong-rust-server)
(devx-ai-test-run-integration)
```

Expected: non-zero result containing `wrong-rust-server did not initialize`, while Python, C++, and CUDA still produce results. Restart the disposable daemon afterward so the literal matrix is restored.

- [ ] **Step 5: Run Emacs Lisp syntax and byte-compilation checks**

Run through a clean Doom daemon so Doom macros and package load paths are present:

```bash
bin/verify-ai-lsp unit
```

Then evaluate byte compilation without writing a `.elc` into the repository:

```elisp
(let ((byte-compile-dest-file-function
       (lambda (_) (make-temp-file "devx-ai-byte-compile." nil ".elc"))))
  (byte-compile-file "~/workspace/devx/doom/ai-lsp-verifier.el"))
```

Expected: unit tests exit zero and byte compilation reports no errors.

- [ ] **Step 6: Commit the real-server verifier**

```bash
git add bin/verify-ai-lsp test/ai-lsp-verifier.el
git commit -m "test: verify Copilot with language servers"
```

### Task 4: Install Pyright and Document the Standalone Repository

**Files:**
- Create: `README.md`

**Interfaces:**
- Consumes: Doom's local LSP package installer and the repository launcher.
- Produces: an installed `pyright-langserver` discoverable through `lsp-package-path 'pyright`, plus human setup and troubleshooting instructions.

- [ ] **Step 1: Install Pyright with lsp-mode's own dependency provider**

In an isolated Doom daemon, require `lsp-pyright` and invoke:

```elisp
(let (failure done)
  (lsp-package-ensure
   'pyright
   (lambda () (setq done t))
   (lambda (message) (setq failure message done t)))
  (while (not done)
    (accept-process-output nil 0.1))
  (when failure (error "Pyright installation failed: %s" failure))
  (or (lsp-package-path 'pyright)
      (error "Pyright installed but is not discoverable")))
```

Expected: an executable path below the current value of
`lsp-server-install-dir` (on this installation,
`~/.emacs.d/.local/etc/lsp/npm/pyright/`).

- [ ] **Step 2: Write repository documentation**

Create `README.md` with:

- Purpose and repository layout.
- The two checked loader blocks required in `~/.doom.d`.
- `doom sync` and restart instructions.
- Backend selector and acceptance keys.
- `bin/verify-ai-lsp unit`, `integration`, and `all` commands.
- The exact Rust/Python/C++/CUDA server and language-ID matrix.
- A note that CUDA verification does not require `nvcc`.
- Troubleshooting for missing Emacs (`DEVX_EMACS`), missing server binaries,
  Copilot authentication failure, network timeout, and LSP initialization logs.
- A warning that verifier output must not include auth database contents or
  access tokens.

- [ ] **Step 3: Validate every documented command exists**

```bash
test -x bin/verify-ai-lsp
bin/verify-ai-lsp --help || test "$?" -eq 64
rg -n "verify-ai-lsp (unit|integration|all)|C-c a|cuda-cpp|rust-analyzer|pyright|clangd" README.md
```

Expected: launcher is executable, invalid usage returns 64 with usage text, and documentation contains all commands and matrix values.

- [ ] **Step 4: Commit documentation**

```bash
git add README.md
git commit -m "docs: explain Doom AI LSP verification"
```

### Task 5: Synchronize Doom and Run Final Verification

**Files:**
- Verify: all repository files and the two `~/.doom.d` loaders.

**Interfaces:**
- Consumes: the completed repository and installed language servers.
- Produces: fresh evidence for Doom package synchronization, configuration health, guarded startup, and all four integrations.

- [ ] **Step 1: Run Doom package synchronization**

Use the Emacs directory from Doom's saved environment:

```bash
export PATH="/private-source/home/operator/devx/envs/doom-emacs/.spack-env/view/bin:$PATH"
~/.emacs.d/bin/doom sync --force
```

Expected: exit 0 and both `copilot` and `tabbymacs` are present in the synchronized package set.

- [ ] **Step 2: Run Doom's diagnostic suite**

```bash
~/.emacs.d/bin/doom doctor
```

Expected: exit 0. Existing unrelated warnings may be reported separately; AI/LSP loader, package, or server errors are failures.

- [ ] **Step 3: Verify a fresh daemon does not start Copilot in scratch**

Start a uniquely named daemon and evaluate:

```elisp
(with-current-buffer "*scratch*"
  (list :copilot (bound-and-true-p copilot-mode)
        :opened (and (boundp 'copilot--opened-buffers)
                     (memq (current-buffer) copilot--opened-buffers))))
```

Expected: `(:copilot nil :opened nil)` and no Copilot language-server child exists before a file-backed programming buffer is opened.

- [ ] **Step 4: Run the complete isolated verifier**

```bash
bin/verify-ai-lsp all
```

Expected: exit 0 with four integration PASS lines:

```text
PASS Rust   rust-analyzer + Copilot rust
PASS Python pyright + Copilot python
PASS C++    clangd + Copilot cpp
PASS CUDA   clangd + Copilot cuda-cpp
```

- [ ] **Step 5: Run repository hygiene checks**

```bash
git diff --check
git status --short
```

Expected: no whitespace errors and a clean worktree.

- [ ] **Step 6: Record verification versions without credentials**

```bash
git log --oneline --decorate -5
rust-analyzer --version
~/.emacs.d/.local/etc/lsp/clangd/clangd_15.0.6/bin/clangd --version | head -n 1
```

Query Pyright through `lsp-package-path 'pyright` and invoke its `--version`; query Copilot with `copilot-installed-version`. Report only version strings and paths, never authentication records.
