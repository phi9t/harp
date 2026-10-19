# Fail-Closed AI/LSP Certification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the existing Rust/Python/C++/CUDA Doom AI/LSP smoke test into a fail-closed, credential-safe, reproducible certification pipeline without changing normal editor behavior.

**Architecture:** Keep the public `bin/verify-ai-lsp unit|integration|all` interface, but split the Emacs verifier into a pure support module, a single private-API adapter, focused unit contracts, and live orchestration. Replace the background-daemon launcher with an owned foreground process group, validate JSON results rather than trusting printed PASS text, and add `bin/certify-ai-lsp` as the release gate that exercises static checks, Doom health, fresh startup, interpreted and compiled live runs, mutations, teardown, and attestation.

**Tech Stack:** Bash 5, GNU coreutils (`setsid`, `timeout`, `stat`, `sha256sum`), Python 3 standard library for strict JSON validation/serialization, Emacs 30.2 Lisp, ERT, Doom Emacs, `lsp-mode`, `copilot.el`, Tabbymacs, Rust Analyzer, Pyright, and Clangd.

**Spec:** `docs/superpowers/specs/2026-09-14-ai-lsp-certification-design.md`

## Global Constraints

- Normal Doom behavior stays unchanged: Copilot is the automatic inline backend for eligible buffers, Tabbymacs is manually selected, and the two modes remain mutually exclusive.
- `lsp-copilot-enabled` remains `nil`; only the language-specific analysis server may appear in `lsp-mode` workspaces.
- The certified matrix is exactly Rust/`rustic-mode`/`rust-analyzer`/`rust`, Python/`python-mode`/`pyright`/`python`, C++/`c++-mode`/`clangd`/`cpp`, and CUDA/`cuda-mode`/`clangd`/`cuda-cpp`.
- CUDA certification uses Clangd and does not require `nvcc` or a CUDA toolkit.
- Pin `copilot.el` to `90f429b100418d897b673d6c36c2817364b28e3e` and Tabbymacs to `32b8c40774515c940a02c78874cf928b356478de`.
- `test/ai-lsp-adapter.el` is the only repository file allowed to invoke a symbol whose name starts with `copilot--` or `lsp--`.
- Raw JSON-RPC responses, Copilot authentication data, environment variables, and unsanitized daemon output must never be printed or written to a regular file.
- A top-level completion response may be JSON null, a vector of valid items, or a plist with a present, vector-valued `:items`; `(:items nil)` is invalid.
- Each live language runs even when another language fails, unless a process-wide prerequisite makes all live checks meaningless.
- A missing shared Clangd dependency must report failures for both C++ and CUDA while Rust and Python still run.
- The live client timeout defaults to 360 seconds and must be at least `4 * (30 + 15 + 30) + 60 = 360` seconds.
- Every external wait has a TERM deadline and a KILL grace period; a timed-out client never implies cancellation of server-side evaluation.
- Direct signals target only the originally captured, validated process group. A daemon-name collision or identity mismatch never authorizes signaling the other daemon.
- Public verifier modes remain exactly `unit`, `integration`, and `all`; release-only seams are enabled only by `DEVX_AI_LSP_CERTIFY=1` from `bin/certify-ai-lsp`.
- `all` remains unit plus live integration. Doom sync/doctor, fresh-startup checks, mutations, and attestation belong only to `bin/certify-ai-lsp`.
- `BLOCKED_EXTERNAL` exits 75. Contract, schema, ownership, cleanup, and credential-safety failures exit 1. CLI usage errors exit 64 and missing local executables exit 69.
- Successful console output is allowlisted. Failure output includes only structured values and sanitized, complete-line diagnostic tails bounded to 4,000 characters.
- Certificates are persistent ignored files at `certificates/<UTC>-<SHA>.json`; launcher temporary roots are deleted on every exit.
- No implementation task is complete until its new test first fails for the intended reason and then passes.

---

## File Map

- `doom/packages.el`: repository declarations and immutable Copilot/Tabbymacs recipe pins.
- `doom/ai-completion.el`: production backend policy, selectors, aliases, keys, and automatic activation; no verifier logic.
- `test/ai-lsp-support.el`: language catalog, fixtures, pure schema predicates, diagnostic sanitization, result schema, JSON writing, failure records, and aggregation.
- `test/ai-lsp-adapter.el`: all private Copilot/lsp-mode calls and compatibility probes.
- `test/ai-lsp-unit.el`: ERT contracts for production configuration, support logic, adapter shape, aggregation, and result serialization.
- `test/ai-lsp-integration.el`: live fixture orchestration, per-language gates, external-blocker classification, and LSP teardown evidence.
- `test/ai-lsp-verifier.el`: compatibility loader and public entry points only.
- `bin/verify-ai-lsp`: input validation, sanitized logging, foreground daemon lifecycle, exact return/result validation, and PASS rendering.
- `test/verify-ai-lsp-launcher`: disposable fake-process contract suite for `bin/verify-ai-lsp`; never connects to a real user daemon.
- `bin/certify-ai-lsp`: ordered release gates, interpreted/compiled and mutation invocations, status classification, and certificate assembly.
- `certificates/.gitignore`: ignore transient certification work directories and persistent local JSON attestations while retaining the directory.
- `README.md`: verifier/certifier contract, prerequisites, result meanings, certificate fields, and troubleshooting.

### Task 1: Freeze Dependency Revisions and Static Pin Contracts

**Files:**
- Modify: `test/ai-lsp-verifier.el`
- Modify: `doom/packages.el`

**Interfaces:**
- Consumes: Doom's `package!` recipe syntax and the two installed repository revisions recorded in Global Constraints.
- Produces: immutable package recipes and ERT test `devx-ai-unit-package-pins` that fails on a missing or changed pin.

- [ ] **Step 1: Add a failing literal pin test to the current unit suite**

Add the repository root constant and test before changing the declarations:

```elisp
(defconst devx-ai-test-repo-root
  (file-name-directory
   (directory-file-name
    (file-name-directory (or load-file-name buffer-file-name)))))

(ert-deftest devx-ai-unit-package-pins ()
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
```

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```bash
bin/verify-ai-lsp unit
```

Expected: nonzero; `devx-ai-unit-package-pins` fails because neither recipe contains `:pin`.

- [ ] **Step 3: Add exact repository pins**

Change `doom/packages.el` to:

```elisp
;; -*- no-byte-compile: t; -*-
;;; packages.el --- DevX AI completion packages

(package! copilot
  :recipe (:host github
           :repo "copilot-emacs/copilot.el"
           :files ("*.el"))
  :pin "90f429b100418d897b673d6c36c2817364b28e3e")
(package! tabbymacs
  :recipe (:host github
           :repo "Bastillan/tabbymacs"
           :files ("*.el"))
  :pin "32b8c40774515c940a02c78874cf928b356478de")
```

- [ ] **Step 4: Run the unit verifier and verify GREEN**

Run:

```bash
bin/verify-ai-lsp unit
```

Expected: exit 0, exact final evaluator value `t`, and no failed ERT result.

- [ ] **Step 5: Commit the pin contract**

```bash
git add doom/packages.el test/ai-lsp-verifier.el
git commit -m "build: pin AI completion packages"
```

### Task 2: Extract Pure Support Contracts and Enforce the Completion Schema

**Files:**
- Create: `test/ai-lsp-support.el`
- Create: `test/ai-lsp-unit.el`
- Modify: `test/ai-lsp-verifier.el`

**Interfaces:**
- Consumes: raw Lisp values decoded by `jsonrpc-request` and the fixed language matrix.
- Produces: `devx-ai-test-language-cases`, `devx-ai-test-valid-completion-response-p (response &optional allow-list-array)`, `devx-ai-test-sanitize-diagnostic (text)`, `devx-ai-test-complete-line-tail (text limit)`, `devx-ai-test-make-result (kind status records failures teardown)`, and `devx-ai-test-write-json (path value)`.

- [ ] **Step 1: Create the support module with the canonical catalog and result constructors**

Create `test/ai-lsp-support.el` with these public data shapes:

```elisp
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

(defconst devx-ai-test-result-schema "devx.ai-lsp.result/v1")

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

(provide 'ai-lsp-support)
```

Move fixture/catalog constants and pure helpers out of `test/ai-lsp-verifier.el`; load files relative to the loader so byte-compiled copies work:

```elisp
(defconst devx-ai-test-directory
  (file-name-directory (or load-file-name buffer-file-name)))

(dolist (file '("ai-lsp-support.el" "ai-lsp-unit.el"))
  (load (expand-file-name file devx-ai-test-directory) nil 'nomessage))
```

At this stage, leave the existing live integration definitions below these loads in the compatibility file. Tasks 4 and 5 move them only after the new adapter and integration modules exist and their contracts are green.

- [ ] **Step 2: Write the strict response tests in `test/ai-lsp-unit.el`**

Use table tests that distinguish JSON vectors from Lisp lists and reject every previously accepted malformed shape:

```elisp
;;; ai-lsp-unit.el --- AI/LSP unit contracts -*- lexical-binding: t; -*-

(require 'ert)
(require 'ai-lsp-support)

(ert-deftest devx-ai-unit-completion-response-schema ()
  (dolist (value
           (list nil
                 []
                 [(:insertText "x")]
                 '(:items [])
                 '(:items [(:insertText "x" :filterText "x")])
                 '(:items [(:insertText "x"
                            :range (:start (:line 0 :character 0)
                                    :end (:line 0 :character 1))
                            :command (:title "Apply" :command "apply"
                                      :arguments []))])))
    (should (devx-ai-test-valid-completion-response-p value)))
  (dolist (value
           (list '(:items nil)
                 '(:items ())
                 '((:insertText "x"))
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
  (should-not
   (devx-ai-test-valid-completion-response-p '((:insertText "x"))))
  (should
   (devx-ai-test-valid-completion-response-p '((:insertText "x")) t)))
```

- [ ] **Step 3: Run the new unit module and verify RED**

Run:

```bash
bin/verify-ai-lsp unit
```

Expected: nonzero because `(:items nil)`, list-valued raw results, nullable commands, and scalar `:arguments` still pass the old predicate.

- [ ] **Step 4: Implement exact predicates in the support module**

Use presence checks for optional fields and vector-only raw arrays:

```elisp
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
```

Unknown item and command extension keys remain valid because the predicates inspect required and known optional keys only.

- [ ] **Step 5: Add JSON serialization tests and make them pass**

Add a test that writes to a temporary file, asserts mode `0600`, parses with `json-parse-buffer`, and checks the exact schema/kind/status/records/failures/teardown keys. Run:

```bash
bin/verify-ai-lsp unit
```

Expected: exit 0 and all completion/result tests pass.

- [ ] **Step 6: Commit the support boundary**

```bash
git add test/ai-lsp-support.el test/ai-lsp-unit.el test/ai-lsp-verifier.el
git commit -m "test: enforce strict AI result schemas"
```

### Task 3: Make Diagnostics Credential-Safe Before Storage or Truncation

**Files:**
- Modify: `test/ai-lsp-support.el`
- Modify: `test/ai-lsp-unit.el`
- Modify: `bin/verify-ai-lsp`
- Test: `test/verify-ai-lsp-launcher`

**Interfaces:**
- Consumes: arbitrary diagnostic strings and daemon stdout/stderr streams.
- Produces: `devx-ai-test-sanitize-diagnostic`, `devx-ai-test-complete-line-tail`, shell `sanitize_stream`, and only sanitized `0600` log files below a `0700` run root.

- [ ] **Step 1: Add ERT redaction and boundary tests**

Generate only synthetic secrets and assert each original byte sequence is absent:

```elisp
(ert-deftest devx-ai-unit-diagnostic-redaction-and-boundaries ()
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
```

- [ ] **Step 2: Run unit tests and verify RED**

Run `bin/verify-ai-lsp unit`.

Expected: nonzero because current code truncates before sanitizing and can expose a suffix of an oversized token-bearing line.

- [ ] **Step 3: Implement complete-line selection followed by sanitization and final tailing**

In `test/ai-lsp-support.el`, make the order explicit:

```elisp
(defconst devx-ai-test-secret-regexps
  '("\\b\\(?:authorization\\|access[-_]?token\\|refresh[-_]?token\\|oauth[-_]?token\\|token\\)\\b[ \t]*[:=][ \t]*\\(?:Bearer[ \t]+\\)?[^][{}(),; \t\n]+"
    "\\\"\\(?:access_token\\|refresh_token\\|oauth_token\\|token\\)\\\"[ \t]*:[ \t]*\\\"[^\\\"\n]+\\\""
    "\\bBearer[ \t]+[^][{}(),; \t\n]+"
    "\\b\\(?:gh[opsu]_[[:alnum:]_]+\\|github_pat_[[:alnum:]_]+\\)\\b"
    "[[:alnum:]_+/.=-]\\{32,\\}"))

(defun devx-ai-test-sanitize-diagnostic (text)
  (let ((case-fold-search t)
        (safe (or text "")))
    (dolist (regexp devx-ai-test-secret-regexps safe)
      (setq safe (replace-regexp-in-string regexp "<redacted>" safe t t)))))

(defun devx-ai-test-complete-line-tail (text limit)
  (let* ((source (or text ""))
         (start (max 0 (- (length source) (* 2 limit))))
         (candidate (substring source start))
         (complete (if (zerop start)
                       candidate
                     (let ((newline (string-match "\n" candidate)))
                       (if newline (substring candidate (1+ newline)) ""))))
         (safe (devx-ai-test-sanitize-diagnostic complete)))
    (if (> (length safe) limit)
        (let* ((raw-start (- (length safe) limit))
               (newline (string-match "\n" safe raw-start)))
          (if newline (substring safe (1+ newline)) ""))
      safe)))
```

- [ ] **Step 4: Create the launcher contract test harness and prove raw-log leakage**

Create executable `test/verify-ai-lsp-launcher` with a private temporary fixture, assertion helpers, and fake sibling `emacs`/`emacsclient` programs. The first fixture writes a fake authorization header and GitHub token to daemon stdout, then makes startup fail. Assert recursively that neither secret occurs in any regular file:

```bash
#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
fixture_root="$(mktemp -d "${TMPDIR:-/tmp}/devx-ai-launcher-test.XXXXXXXX")"
trap 'rm -rf -- "$fixture_root"' EXIT
chmod 700 "$fixture_root"

assert_not_contains_tree() {
  local needle=$1 root=$2
  if grep -R -F -l -- "$needle" "$root" >/dev/null 2>&1; then
    printf 'secret leaked to a regular file: %s\n' "$needle" >&2
    return 1
  fi
}

run_case() {
  local name=$1
  shift
  printf 'CASE %s\n' "$name"
  "$@"
}
```

Use generated fake executables with modes controlled by `DEVX_FAKE_SCENARIO`; never place real credential material in fixtures.

- [ ] **Step 5: Stream daemon output through a sanitizer before the private log**

In `bin/verify-ai-lsp`, create a FIFO rather than a raw regular file. Start the sanitizer before Emacs and restrict permissions:

```bash
umask 077
run_root="$(mktemp -d "$run_parent/devx-ai-lsp.XXXXXXXX")"
chmod 700 "$run_root"
daemon_fifo="$run_root/daemon.pipe"
daemon_log="$run_root/daemon.sanitized.log"
mkfifo -m 600 "$daemon_fifo"
: >"$daemon_log"
chmod 600 "$daemon_log"

sanitize_stream() {
  sed -u -E \
    -e 's/([Aa]uthorization|[Aa]ccess[-_ ]?[Tt]oken|[Rr]efresh[-_ ]?[Tt]oken|[Oo][Aa]uth[-_ ]?[Tt]oken|[Tt]oken)[[:space:]]*[:=][[:space:]]*(Bearer[[:space:]]+)?[^[:space:],;}]+/\1=<redacted>/g' \
    -e 's/"(access_token|refresh_token|oauth_token|token)"[[:space:]]*:[[:space:]]*"[^"]+"/"\1":"<redacted>"/gI' \
    -e 's/(Bearer[[:space:]]+)[^[:space:],;}]+/\1<redacted>/g' \
    -e 's/(gh[opsu]_[[:alnum:]_]+|github_pat_[[:alnum:]_]+)/<redacted>/g' \
    -e 's/[[:alnum:]_+\/.=-]{32,}/<redacted>/g'
}

sanitize_stream <"$daemon_fifo" >"$daemon_log" &
sanitizer_pid=$!
```

Do not redirect any `emacsclient` output to a different file; pipe it through the same sanitizer or capture only exact allowlisted protocol values in shell variables.

- [ ] **Step 6: Make bounded diagnostics start at a complete line**

Implement `safe_log_tail` with `tail -c 8000`, discard the first partial line with `sed '1d'` when the source exceeded the window, sanitize once more defensively, and only then apply `tail -c 4000`. Add fake secrets immediately before, inside, and across both the 8,000-byte selection edge and 4,000-byte final edge.

- [ ] **Step 7: Run both suites and verify GREEN**

Run:

```bash
bin/verify-ai-lsp unit
test/verify-ai-lsp-launcher diagnostics
```

Expected: both exit 0; the launcher test reports its case count, every synthetic secret is absent from captured stderr and all regular files, and the diagnostic tail is at most 4,000 characters.

- [ ] **Step 8: Commit safe diagnostics**

```bash
git add bin/verify-ai-lsp test/ai-lsp-support.el test/ai-lsp-unit.el test/verify-ai-lsp-launcher
git commit -m "fix: sanitize verifier diagnostics before storage"
```

### Task 4: Isolate Private APIs and Prove Exact Workspace Ownership

**Files:**
- Create: `test/ai-lsp-adapter.el`
- Modify: `test/ai-lsp-unit.el`
- Modify: `test/ai-lsp-verifier.el`

**Interfaces:**
- Consumes: pinned `copilot.el`, Doom-pinned `lsp-mode`, and live buffer state.
- Produces: `devx-ai-adapter-check-compatibility ()`, `devx-ai-adapter-copilot-language-id ()`, `devx-ai-adapter-copilot-alive-p ()`, `devx-ai-adapter-copilot-status (seconds)`, `devx-ai-adapter-inline-completion (seconds)`, `devx-ai-adapter-start-lsp ()`, `devx-ai-adapter-workspaces ()`, `devx-ai-adapter-reset-session ()`, `devx-ai-adapter-resolve-server (server)`, and pure `devx-ai-test-workspaces-valid-p (records expected-server expected-executable)`.

- [ ] **Step 1: Add adapter compatibility and source-isolation tests**

Add literal expected language IDs independent from the operational catalog and check every private symbol's callable shape:

```elisp
(defconst devx-ai-test-production-language-ids
  '((rustic-mode . "rust") (rust-mode . "rust") (rust-ts-mode . "rust")
    (python-mode . "python") (python-ts-mode . "python")
    (c++-mode . "cpp") (c++-ts-mode . "cpp")
    (cuda-mode . "cuda-cpp") (cuda-ts-mode . "cuda-cpp")))

(ert-deftest devx-ai-unit-adapter-compatible ()
  (should (equal t (devx-ai-adapter-check-compatibility))))

(ert-deftest devx-ai-unit-private-api-isolation ()
  (dolist (path (directory-files devx-ai-test-directory t "\\.el\\'"))
    (unless (string= (file-name-nondirectory path) "ai-lsp-adapter.el")
      (with-temp-buffer
        (insert-file-contents path)
        (should-not (re-search-forward
                     "(\\(?:copilot--\\|lsp--\\)" nil t))))))
```

- [ ] **Step 2: Run unit tests and verify RED**

Run `bin/verify-ai-lsp unit`.

Expected: nonzero because private calls still exist in the monolith/integration file and the adapter functions are undefined.

- [ ] **Step 3: Implement the single compatibility boundary**

Create `test/ai-lsp-adapter.el`. Probe all required symbols before use and expose normalized workspace records:

```elisp
;;; ai-lsp-adapter.el --- Pinned private API boundary -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'seq)

(defconst devx-ai-adapter-required-functions
  '((copilot--connection-alivep . 0)
    (copilot--ensure-doc-open . 0)
    (copilot--get-language-id . 0)
    (copilot--inline-completion-params . 1)
    (lsp--client-server-id . 1)
    (lsp--workspace-client . 1)
    (lsp--workspace-status . 1)
    (lsp--workspace-proc . 1)
    (lsp-package-path . 1)))

(defconst devx-ai-adapter-required-macros
  '(copilot--request))

(defun devx-ai-adapter-accepts-arity-p (function arity)
  (pcase-let ((`(,minimum . ,maximum) (func-arity function)))
    (and (<= minimum arity)
         (or (eq maximum 'many) (<= arity maximum)))))

(defun devx-ai-adapter-check-compatibility ()
  (require 'copilot)
  (require 'lsp-mode)
  (let ((bad-functions
         (seq-filter
          (lambda (entry)
            (or (not (functionp (symbol-function (car entry))))
                (not (devx-ai-adapter-accepts-arity-p
                      (car entry) (cdr entry)))))
          devx-ai-adapter-required-functions))
        (bad-macros (seq-remove #'macrop devx-ai-adapter-required-macros)))
    (if (or bad-functions bad-macros)
        (error "Incompatible pinned dependency API: functions=%S macros=%S"
               (mapcar #'car bad-functions) bad-macros)
      t)))

(defun devx-ai-adapter-copilot-language-id ()
  (copilot--get-language-id))

(defun devx-ai-adapter-copilot-alive-p ()
  (copilot--connection-alivep))

(defun devx-ai-adapter-copilot-status (seconds)
  (copilot--request 'checkStatus nil :timeout seconds))

(defun devx-ai-adapter-inline-completion (seconds)
  (copilot--ensure-doc-open)
  (copilot--request 'textDocument/inlineCompletion
                    (copilot--inline-completion-params 1)
                    :timeout seconds))

(defun devx-ai-adapter-start-lsp () (lsp))

(defun devx-ai-adapter-workspaces ()
  (mapcar
   (lambda (workspace)
     (let* ((client (lsp--workspace-client workspace))
            (process (ignore-errors (lsp--workspace-proc workspace)))
            (command (and (processp process) (process-command process))))
       (list :id (lsp--client-server-id client)
             :status (lsp--workspace-status workspace)
             :command (and command (car command))
             :process process
             :alive (and (processp process) (process-live-p process)))))
   lsp--buffer-workspaces))

(defun devx-ai-adapter-reset-session (session-file)
  (setq lsp-session-file session-file
        lsp--session nil))

(defun devx-ai-adapter-resolve-server (server)
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
```

Keep all `declare-function` and private-variable declarations in this file, so callers never read `lsp--session`, `lsp--buffer-workspaces`, or call `lsp-package-path` directly.

- [ ] **Step 4: Add exact workspace-set and executable-identity unit tests**

Given normalized records, test all rejection cases:

```elisp
(ert-deftest devx-ai-unit-workspace-contract-is-exact ()
  (should
   (devx-ai-test-workspaces-valid-p
    '((:id rust-analyzer :status initialized
       :command "/real/rust-analyzer" :alive t))
    'rust-analyzer "/real/rust-analyzer"))
  (should-not
   (devx-ai-test-workspaces-valid-p
    '((:id rust-analyzer :status initialized
       :command "/real/rust-analyzer" :alive t)
      (:id copilot :status initialized :command "/tmp/copilot" :alive t))
    'rust-analyzer "/real/rust-analyzer"))
  (should-not
   (devx-ai-test-workspaces-valid-p
    '((:id rust-analyzer :status initialized
       :command "/other/rust-analyzer" :alive t))
    'rust-analyzer "/real/rust-analyzer")))
```

`devx-ai-test-workspaces-valid-p` must require exactly one record, exact server ID, `initialized`, a live process, and equality of `file-truename` for the observed command and preflight executable.

Implement it in `test/ai-lsp-support.el` so the adapter only normalizes dependency state:

```elisp
(defun devx-ai-test-workspaces-valid-p
    (records expected-server expected-executable)
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
```

- [ ] **Step 5: Replace every private call outside the adapter**

Use the adapter APIs in production-ID tests and in the still-inline integration orchestration. Verify isolation mechanically:

```bash
if rg -n '\((copilot--|lsp--)' test --glob '*.el' --glob '!ai-lsp-adapter.el'; then
  exit 1
fi
```

Expected: no matches and exit 0.

- [ ] **Step 6: Run unit and live regression tests**

Run:

```bash
bin/verify-ai-lsp unit
bin/verify-ai-lsp integration
```

Expected: unit exits 0; live integration still produces the four matrix PASS rows on a healthy external service.

- [ ] **Step 7: Commit the adapter boundary**

```bash
git add test/ai-lsp-adapter.el test/ai-lsp-unit.el test/ai-lsp-verifier.el
git commit -m "refactor: isolate AI LSP private APIs"
```

### Task 5: Refactor Live Orchestration, Aggregate Failures, and Prove Teardown

**Files:**
- Modify: `test/ai-lsp-support.el`
- Modify: `test/ai-lsp-unit.el`
- Create: `test/ai-lsp-integration.el`
- Modify: `test/ai-lsp-verifier.el`

**Interfaces:**
- Consumes: canonical language cases, adapter records, and resolved executable paths. Until Task 7, it also maintains the legacy verifier-owned PASS buffer solely so the existing launcher stays green.
- Produces: `devx-ai-test-run-cases (cases fixture-root dependencies)`, `devx-ai-test-run-unit-result ()`, `devx-ai-test-run-integration-result ()`, `devx-ai-test-run-unit ()`, `devx-ai-test-run-integration ()`, `devx-ai-test-run-all ()`, JSON result records for every language, and teardown records for every observed LSP child process.

- [ ] **Step 1: Add unit contracts for selectors, minibuffers, aggregation, and shared dependencies**

Cover both backend-switch directions, disabling both backends, and minibuffer exclusion. Add an aggregation seam whose case runner returns records rather than signaling immediately:

```elisp
(ert-deftest devx-ai-unit-aggregation-does-not-short-circuit ()
  (let (visited)
    (cl-letf (((symbol-function 'devx-ai-test-verify-case)
               (lambda (case _root _dependency)
                 (push (plist-get case :name) visited)
                 (if (member (plist-get case :name) '("C++" "CUDA"))
                     (list :status "FAIL" :boundary "dependency"
                           :language (plist-get case :name))
                   (list :status "PASS"
                         :language (plist-get case :name))))))
      (let ((result (devx-ai-test-run-cases
                     devx-ai-test-language-cases "/tmp/root"
                     '((rust-analyzer :path "/bin/rust-analyzer")
                       (pyright :path "/bin/pyright-langserver")
                       (clangd :error "Missing clangd")))))
        (should (equal '("Rust" "Python" "C++" "CUDA")
                       (nreverse visited)))
        (should (= 4 (length (plist-get result :records))))
        (should (= 2 (length (plist-get result :failures))))))))
```

- [ ] **Step 2: Run unit tests and verify RED**

Run `bin/verify-ai-lsp unit`.

Expected: nonzero because the current integration signals after collecting strings and does not expose record/failure values or teardown evidence.

- [ ] **Step 3: Create the integration module and implement one structured record per language**

Move fixture creation, preflight, per-case execution, aggregation, and cleanup from the compatibility loader into `test/ai-lsp-integration.el`. Add `(require 'ai-lsp-support)` and `(require 'ai-lsp-adapter)`, and leave no ERT definitions or private dependency calls in this module.

Each record must contain these exact keys and safe value types:

```elisp
(list :language name
      :mode (symbol-name expected-mode)
      :expected-server (symbol-name expected-server)
      :observed-server (and observed-server (symbol-name observed-server))
      :expected-executable expected-executable
      :observed-executable observed-executable
      :expected-language-id expected-language-id
      :observed-language-id observed-language-id
      :gates (vector
              (list :name "mode" :status mode-status)
              (list :name "backend-exclusivity" :status backend-status)
              (list :name "copilot-language-id" :status language-id-status)
              (list :name "lsp-workspace" :status workspace-status)
              (list :name "lsp-executable" :status executable-status)
              (list :name "copilot-connection" :status connection-status)
              (list :name "copilot-auth" :status auth-status)
              (list :name "inline-completion" :status completion-status))
      :status overall-status)
```

On malformed completion, record only `:observed-type (symbol-name (type-of response))` and `:boundary "inline-completion-schema"`; never format the response itself. Every failure record uses the allowlisted shape below, with unavailable fields represented by JSON null:

```elisp
(list :language name
      :boundary boundary
      :expected expected-summary
      :observed observed-summary
      :resolved-executable resolved-executable
      :workspaces (vconcat safe-workspace-summaries)
      :diagnostic (devx-ai-test-complete-line-tail safe-log 4000)
      :classification classification)
```

Unit tests assert all eight keys are present, `:classification` is exactly `FAIL` or `BLOCKED_EXTERNAL`, and no diagnostic contains any synthetic secret.

- [ ] **Step 4: Classify only external Copilot availability as blocked**

Map transport timeout, dead Copilot connection, authentication unavailable, and service-unavailable status to `devx-ai-test-external-blocker`. Malformed protocol, wrong IDs, unexpected workspaces, executable mismatch, Lisp errors, and cleanup failures stay `FAIL`. The final result status is `FAIL` if any local failure exists, otherwise `BLOCKED_EXTERNAL` if an external blocker exists, otherwise `PASS`.

- [ ] **Step 5: Wait for workspace shutdown and record child state**

Before killing fixture buffers, capture adapter process objects. After `lsp-disconnect`/buffer cleanup, wait up to five seconds for each process to stop, then write:

```elisp
(list :server (symbol-name server-id)
      :pid (and (processp process) (process-id process))
      :alive-after-cleanup (and (processp process) (process-live-p process)))
```

Any `:alive-after-cleanup t` makes the integration result `FAIL`; the shell process-group cleanup remains the final containment layer.

- [ ] **Step 6: Make entry points write results and return exactly `t`**

`devx-ai-test-run-unit-result` returns `kind=unit`, completed test count, unexpected count, and status without writing. `devx-ai-test-run-integration-result` returns all records/failures/teardown even on FAIL or BLOCKED_EXTERNAL. The three public entry points select the appropriate internal result, write it when `DEVX_AI_LSP_RESULT_FILE` is present, and return exactly `t`; status is carried by the trusted file after Task 7. `devx-ai-test-run-all` first requires a passing unit result and then produces one combined result containing the unit counts and live records. Through Task 6, retain and populate `devx-ai-test-results-buffer-name` from PASS records so the existing launcher remains green; Task 7 removes that transitional text path after it sets `DEVX_AI_LSP_RESULT_FILE` and validates JSON. If requested result writing fails, signal a local error.

Finish the compatibility loader with only relative loads, entrypoint definitions, `provide`, and no verifier implementation body:

```elisp
(dolist (file '("ai-lsp-support.el" "ai-lsp-adapter.el"
                "ai-lsp-unit.el" "ai-lsp-integration.el"))
  (load (expand-file-name file devx-ai-test-directory) nil 'nomessage))

(provide 'ai-lsp-verifier)
```

- [ ] **Step 7: Add release-only mutation seams**

When and only when `DEVX_AI_LSP_CERTIFY` equals `1`, accept:

```text
DEVX_AI_LSP_MUTATION=wrong-rust-server
DEVX_AI_LSP_MUTATION=missing-clangd
```

The first substitutes `rls` for Rust's expected server without changing production configuration. The second makes the preflight resolver return `(:error "Missing clangd (certification mutation)")`. Unknown mutation values are local failures. Unit tests bind the environment and assert that Rust fails in the first case, C++ and CUDA fail in the second, and all later runnable language names are present.

- [ ] **Step 8: Run unit and integration tests**

Run:

```bash
bin/verify-ai-lsp unit
bin/verify-ai-lsp integration
```

Expected: both exit 0 on a healthy external service; integration result contains exactly four records and teardown says every LSP child is stopped.

- [ ] **Step 9: Commit structured live verification**

```bash
git add test/ai-lsp-support.el test/ai-lsp-unit.el test/ai-lsp-integration.el test/ai-lsp-verifier.el
git commit -m "feat: aggregate exact AI LSP verification results"
```

### Task 6: Replace Client-Only Timeout with Owned Process-Group Lifecycle

**Files:**
- Modify: `bin/verify-ai-lsp`
- Modify: `test/verify-ai-lsp-launcher`

**Interfaces:**
- Consumes: GNU `setsid`, `timeout`, fake or real sibling Emacs executables, and exact identity response `(OWNER PID)`.
- Produces: a captured `daemon_pid`/`daemon_pgid`, validated ownership state, bounded `run_client`, idempotent `cleanup`, and conventional 130/143 signal exits.

- [ ] **Step 1: Add launcher failure-path cases before changing lifecycle code**

The fake executable suite must exercise these named cases independently:

```text
exact-t-success
nil-return-rejected
invalid-mode-rejected
invalid-timeout-rejected
client-timeout-kills-group
startup-failure-kills-group
term-ignored-forces-kill
name-collision-preserves-sentinel
pid-mismatch-rejected
marker-mismatch-rejected
sigint-exits-130
sigterm-exits-143
temp-root-removed
```

Each fake foreground daemon forks a child and records PID/PGID under the fixture root. Collision tests start a sentinel in a different group and assert `kill -0` still succeeds after the verifier exits. Force-kill tests trap TERM in both fake parent and child and assert neither PID exists after the KILL grace.

- [ ] **Step 2: Run lifecycle tests and verify RED**

Run:

```bash
test/verify-ai-lsp-launcher lifecycle
```

Expected: nonzero; current `--daemon` startup returns before ownership capture and a timed-out `emacsclient` can leave the real daemon and its children alive.

- [ ] **Step 3: Validate timeouts including the computed live floor**

Use these exact defaults and constraints:

```bash
lsp_timeout=30
auth_timeout=15
completion_timeout=30
language_count=4
orchestration_margin=60
minimum_client_timeout=$((language_count * (lsp_timeout + auth_timeout + completion_timeout) + orchestration_margin))
startup_timeout="${DEVX_AI_LSP_STARTUP_TIMEOUT:-30}"
client_timeout="${DEVX_AI_LSP_CLIENT_TIMEOUT:-$minimum_client_timeout}"
cleanup_timeout="${DEVX_AI_LSP_CLEANUP_TIMEOUT:-5}"
kill_grace="${DEVX_AI_LSP_KILL_GRACE:-3}"
```

All values must match `^[1-9][0-9]*$`; integration/all client overrides below 360 exit 64. Unit may use a smaller positive client bound because it has no four-language live aggregation.

- [ ] **Step 4: Start a foreground daemon in its own session and capture identity**

Launch through the sanitized FIFO:

```bash
DEVX_AI_LSP_OWNER="$owner_token" \
  setsid "$emacs_bin" --init-directory "$doom_root" \
  "--fg-daemon=$server_name" >"$daemon_fifo" 2>&1 &
daemon_pid=$!

daemon_pgid="$(ps -o pgid= -p "$daemon_pid" | tr -d ' ')"
[[ "$daemon_pgid" == "$daemon_pid" ]] ||
  fail "Foreground daemon did not own its process group."
```

Poll identity using a bounded client call with `timeout --foreground --kill-after="${kill_grace}s"`. Accept only an exact Emacs list containing the private marker and the captured decimal PID. A collision or mismatch fails without calling `kill-emacs` on that server name.

- [ ] **Step 5: Make all client calls TERM/KILL bounded and capture exact output**

Implement:

```bash
run_client() {
  local seconds=$1 expression=$2 output
  if ! output="$(timeout --foreground --kill-after="${kill_grace}s" \
      "${seconds}s" "$emacsclient_bin" -s "$server_name" -e "$expression" \
      2> >(sanitize_stream >&2))"; then
    return 124
  fi
  printf '%s' "$output"
}
```

For the verifier entrypoint, compare the captured output exactly to `t`. Empty output, whitespace variants, `nil`, quoted `"t"`, or additional forms fail even if emacsclient exits 0.

- [ ] **Step 6: Implement idempotent process-group cleanup**

Cleanup must preserve the triggering status and execute this order:

```bash
if (( identity_validated )); then
  run_client "$cleanup_timeout" '(kill-emacs)' >/dev/null 2>&1 || true
fi

deadline=$((SECONDS + cleanup_timeout))
while kill -0 -- "-$daemon_pgid" 2>/dev/null && (( SECONDS < deadline )); do
  read -r -t 0.05 _ || true
done
if kill -0 -- "-$daemon_pgid" 2>/dev/null; then
  kill -TERM -- "-$daemon_pgid" 2>/dev/null || true
fi
deadline=$((SECONDS + kill_grace))
while kill -0 -- "-$daemon_pgid" 2>/dev/null && (( SECONDS < deadline )); do
  read -r -t 0.05 _ || true
done
if kill -0 -- "-$daemon_pgid" 2>/dev/null; then
  kill -KILL -- "-$daemon_pgid" 2>/dev/null || true
fi
wait "$daemon_pid" 2>/dev/null || true
```

Signal the group only if it is the group created by this invocation and `daemon_pgid == daemon_pid`. Ownership mismatch suppresses graceful client shutdown but does not prevent cleanup of the launcher's own captured child group. Verify `kill -0 -- -PGID` fails before deleting the exact validated `run_root`.

- [ ] **Step 7: Preserve signal semantics**

Use `trap 'exit 130' INT`, `trap 'exit 143' TERM`, and a separate EXIT trap that saves `$?`, disables traps, runs cleanup, and exits with the saved status.

- [ ] **Step 8: Run launcher and real smoke suites**

Run:

```bash
test/verify-ai-lsp-launcher lifecycle
bin/verify-ai-lsp unit
```

Expected: all fake lifecycle cases pass, no recorded fake PID survives, sentinel survives collision, temporary roots are gone, and real unit mode exits 0.

- [ ] **Step 9: Commit owned lifecycle handling**

```bash
git add bin/verify-ai-lsp test/verify-ai-lsp-launcher
git commit -m "fix: own and bound verifier process groups"
```

### Task 7: Validate Structured Results Instead of Trusting PASS Text

**Files:**
- Modify: `bin/verify-ai-lsp`
- Modify: `test/verify-ai-lsp-launcher`
- Modify: `test/ai-lsp-support.el`
- Modify: `test/ai-lsp-verifier.el`

**Interfaces:**
- Consumes: private `0600` JSON written by the Emacs entrypoint.
- Produces: shell-validated unit/integration/all contracts, allowlisted human output, and optional certification export below `certificates/.cert-run.*`.

- [ ] **Step 1: Add result-contract launcher cases**

Fake emacsclient responses cover exact `t` plus result files with wrong schema, missing field, duplicate language, extra language, wrong mode/server/ID, failed gate under PASS, unexpected unit count, and invalid teardown. Every malformed result must make the launcher exit 1 without printing a PASS line.

- [ ] **Step 2: Run the result cases and verify RED**

Run:

```bash
test/verify-ai-lsp-launcher results
```

Expected: nonzero because the existing launcher trusts a hand-formatted text buffer and does not validate JSON structure.

- [ ] **Step 3: Validate JSON with an inline Python standard-library checker**

After exact `t`, run `python3` with the result path and mode. The checker rejects unknown top-level keys and verifies:

```python
EXPECTED = [
    ("Rust", "rustic-mode", "rust-analyzer", "rust"),
    ("Python", "python-mode", "pyright", "python"),
    ("C++", "c++-mode", "clangd", "cpp"),
    ("CUDA", "cuda-mode", "clangd", "cuda-cpp"),
]
TOP_KEYS = {"schema", "kind", "status", "entrypoint", "records",
            "failures", "teardown", "unit"}

assert type(doc) is dict and set(doc) == TOP_KEYS
assert doc["schema"] == "devx.ai-lsp.result/v1"
assert doc["entrypoint"] is True
assert doc["status"] in {"PASS", "FAIL", "BLOCKED_EXTERNAL"}
```

For PASS integration/all, compare the ordered tuple projection of all four records to `EXPECTED`, require every gate status to equal `PASS`, require no failures, and require every teardown `alive-after-cleanup` to be false. Unit requires integer `completed > 0` and `unexpected == 0`.

- [ ] **Step 4: Render output only after validation**

Print unit summary as `PASS Unit <completed> tests, 0 unexpected`. Print the four integration rows from validated fields, not from Emacs messages. Emit only:

```text
PASS Rust   rust-analyzer + Copilot rust
PASS Python pyright + Copilot python
PASS C++    clangd + Copilot cpp
PASS CUDA   clangd + Copilot cuda-cpp
```

Status `BLOCKED_EXTERNAL` exits 75 after a sanitized boundary summary; `FAIL` exits 1.

- [ ] **Step 5: Add guarded certification export**

If `DEVX_AI_LSP_CERTIFY=1` and `DEVX_AI_LSP_RESULT_EXPORT` is set, require its canonical parent to match `$repo_root/certificates/.cert-run.*`, require that parent to be owned by the current UID with mode `0700`, reject symlinks, and install the already-validated JSON with mode `0600`. Ignore no validation failure.

- [ ] **Step 6: Run all launcher contracts and public verifier modes**

Run:

```bash
test/verify-ai-lsp-launcher
bin/verify-ai-lsp unit
bin/verify-ai-lsp integration
bin/verify-ai-lsp all
```

Expected: launcher suite exits 0 and reports all case counts; all three public modes exit 0 on healthy dependencies; output contains no raw evaluator or JSON payload.

- [ ] **Step 7: Commit the trusted result boundary**

```bash
git add bin/verify-ai-lsp test/verify-ai-lsp-launcher test/ai-lsp-support.el test/ai-lsp-verifier.el
git commit -m "feat: validate structured verifier results"
```

### Task 8: Build the Complete Release Certification Pipeline

**Files:**
- Create: `bin/certify-ai-lsp`
- Create: `certificates/.gitignore`
- Modify: `bin/verify-ai-lsp`
- Modify: `test/ai-lsp-integration.el`
- Modify: `test/ai-lsp-unit.el`
- Modify: `test/verify-ai-lsp-launcher`

**Interfaces:**
- Consumes: a clean committed repository, checked external Doom loaders, pinned dependencies, public verifier, release-only environment seams, Doom CLI, and local language servers.
- Produces: `devx-ai-test-run-fresh-startup ()`, release-only compiled/mutation launcher paths, ordered gate records, exit 0/1/75 classification, and an allowlisted JSON certificate.

- [ ] **Step 1: Add a fake-gate certification harness and verify RED**

Extend `test/verify-ai-lsp-launcher` with a `certifier` group. Source `bin/certify-ai-lsp` under its `[[ ${BASH_SOURCE[0]} == "$0" ]]` main guard and override its `gate_static`, `gate_unit`, `gate_launcher`, `gate_doom`, `gate_fresh`, `gate_live`, `gate_mutations`, `gate_teardown`, and `gate_attestation` shell functions; assert gate order and short-circuit rules from a call log. Cases must cover all-pass, static failure, Doom failure, external block, local live failure, mutation unexpectedly passing, teardown failure, dirty repository, and certificate schema/permissions.

Run:

```bash
test/verify-ai-lsp-launcher certifier
```

Expected: nonzero because `bin/certify-ai-lsp` does not exist.

- [ ] **Step 2: Create the ignored persistent certificate directory contract**

Create `certificates/.gitignore`:

```gitignore
*
!.gitignore
```

Certification work roots use `certificates/.cert-run.XXXXXXXX`, mode `0700`, and are removed on exit. Final JSON artifacts remain until the user deletes them.

- [ ] **Step 3: Implement ordered gate execution and status precedence**

Create executable `bin/certify-ai-lsp` with a source-safe main guard and `run_gate NAME FUNCTION`. Record monotonic start/end seconds and one of `PASS`, `FAIL`, or `BLOCKED_EXTERNAL`. Stop immediately for static, unit, launcher, or Doom prerequisite failure. Continue through all language/mutation/teardown gates once live execution begins so failure aggregation remains observable. Overall `FAIL` overrides `BLOCKED_EXTERNAL`; otherwise any external blocker yields `BLOCKED_EXTERNAL`; otherwise `PASS`.

Use this explicit orchestration skeleton:

```bash
run_gate() {
  local name=$1 function=$2 started=$SECONDS status
  set +e
  "$function"
  status=$?
  set -e
  record_gate "$name" "$status" "$((SECONDS - started))"
  return "$status"
}

main() {
  initialize_private_cert_run
  run_gate static gate_static || abort_after_prerequisite
  run_gate unit gate_unit || abort_after_prerequisite
  run_gate launcher gate_launcher || abort_after_prerequisite
  run_gate doom gate_doom || abort_after_prerequisite
  run_gate fresh gate_fresh || remember_live_status "$?"
  run_gate live gate_live || remember_live_status "$?"
  run_gate mutations gate_mutations || remember_live_status "$?"
  run_gate teardown gate_teardown || remember_live_status "$?"
  run_gate attestation gate_attestation || remember_live_status "$?"
  exit_for_overall_status
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
```

`abort_after_prerequisite` runs the attestation gate to emit the smaller failure artifact and then calls `exit_for_overall_status`; it never returns to `main`. Unit tests assert that gate status and the final 1/75 exit are preserved.

- [ ] **Step 4: Implement the static gate exactly**

The static gate executes:

```bash
git diff --quiet
git diff --cached --quiet
test -z "$(git ls-files --others --exclude-standard)"
bash -n bin/verify-ai-lsp bin/certify-ai-lsp test/verify-ai-lsp-launcher
rg -n '\((copilot--|lsp--)' test --glob '*.el' --glob '!ai-lsp-adapter.el' && exit 1 || true
rg -F '90f429b100418d897b673d6c36c2817364b28e3e' doom/packages.el
rg -F '32b8c40774515c940a02c78874cf928b356478de' doom/packages.el
```

Validate exactly one checked loader in `~/.doom.d/config.el` and `~/.doom.d/packages.el`, each resolving to this repository. Byte-compile all five test modules into the certification work root with warnings treated as errors; load the compiled compatibility loader in a clean batch Emacs process.

- [ ] **Step 5: Add unit and launcher gates**

Run `bin/verify-ai-lsp unit`, then `test/verify-ai-lsp-launcher`. Capture only their allowlisted summaries and durations. The launcher suite must use fakes and must not discover or connect to a real Emacs daemon.

- [ ] **Step 6: Add Doom sync and doctor gates**

Run:

```bash
~/.emacs.d/bin/doom sync --force
~/.emacs.d/bin/doom doctor
```

Sanitize both streams before any file storage. A sync failure fails certification. Doctor lines containing case-insensitive `error` with `copilot`, `tabby`, `lsp`, `devx`, `config`, or `packages` fail; unrelated warnings are reduced to allowlisted category/count summaries for the certificate, not copied verbatim.

- [ ] **Step 7: Add the fresh-startup gate through the owned launcher**

With `DEVX_AI_LSP_CERTIFY=1 DEVX_AI_LSP_CERT_GATE=fresh`, the launcher calls `devx-ai-test-run-fresh-startup`. It verifies in `*scratch*` that `copilot-mode` and `tabbymacs-mode` are false, `(featurep 'copilot)` is false, and no live entry in `(process-list)` has a name or command containing `copilot`. This proves no Copilot document or server was opened by startup without loading the deferred package to inspect it. The owned daemon then exits and the launcher verifies its process group is empty.

Implement the Emacs-side assertion without touching a private Copilot symbol:

```elisp
(defun devx-ai-test-run-fresh-startup ()
  (let* ((scratch (get-buffer "*scratch*"))
         (copilot-processes
          (seq-filter
           (lambda (process)
             (string-match-p
              "copilot"
              (downcase
               (string-join
                (cons (process-name process)
                      (or (process-command process) '()))
                " "))))
           (process-list)))
         (clean
          (and scratch
               (with-current-buffer scratch
                 (and (not (bound-and-true-p copilot-mode))
                      (not (bound-and-true-p tabbymacs-mode))))
               (not (featurep 'copilot))
               (null copilot-processes)))
         (result
          (devx-ai-test-make-result
           "fresh" (if clean "PASS" "FAIL") nil
           (unless clean
             (list (list :language nil :boundary "fresh-startup"
                         :expected "deferred AI backends"
                         :observed "AI backend loaded or running"
                         :resolved-executable nil :workspaces []
                         :diagnostic "" :classification "FAIL")))
           [])))
    (devx-ai-test-write-json (getenv "DEVX_AI_LSP_RESULT_FILE") result)
    t))
```

Add ERT coverage that stubs `process-list` for clean and Copilot-process cases and separately binds `features` to include `copilot`; both dirty cases must return a result with status `FAIL`.

- [ ] **Step 8: Add interpreted and compiled live gates**

Run normal integration once. Then set `DEVX_AI_LSP_COMPILED=1`; the launcher byte-compiles a copy of the five test files below its private run root, loads the `.elc` compatibility loader, and runs integration again. Export each validated result into the certificate work root. Both must contain all four exact rows and clean teardown.

- [ ] **Step 9: Add both negative mutations**

Run integration with each release-only mutation. `wrong-rust-server` must exit 1, identify Rust's `lsp-workspace` boundary, and still contain Python/C++/CUDA records. `missing-clangd` must exit 1, contain PASS records for Rust and Python, and dependency failures for both C++ and CUDA. A mutation run that exits 0 or 75 fails certification.

- [ ] **Step 10: Attest process teardown**

For every launcher invocation, require its validated result teardown records to be false and require the launcher-owned process group to be absent. The fake suite additionally scans recorded child PIDs. Any surviving daemon, language server, sanitizer, or fake child is a local FAIL.

- [ ] **Step 11: Write the final certificate with an allowlist serializer**

Use Python's `json` module to build a new object from validated inputs; never merge arbitrary result dictionaries. The exact top-level schema is:

```python
certificate = {
    "schema": "devx.ai-lsp.certificate/v1",
    "timestamp_utc": timestamp,
    "result": overall_status,
    "repository": {"commit": git_sha, "dirty": dirty},
    "hashes": {
        "doom/packages.el": packages_hash,
        "doom/ai-completion.el": config_hash,
        "loader/packages": package_loader_hash,
        "loader/config": config_loader_hash,
    },
    "versions": versions,
    "gates": gates,
    "languages": languages,
    "teardown": teardown,
    "doom_warnings": doom_warning_summaries,
}
```

`versions` contains only name/version/commit strings for Emacs, Doom, copilot.el, Copilot server, Tabbymacs, lsp-mode, rust-analyzer, Pyright, and Clangd. Read repository commits with `git -C PATH rev-parse HEAD`, Emacs from `emacs --version`, Rust Analyzer and Clangd from their first `--version` line, and Pyright/Copilot server versions from their installed `package.json` `version` field; reject values outside conservative version/hex patterns. `gates` contains name/status/duration seconds. `languages` is projected from the validated interpreted run and contains expected/observed mode, server, executable, language ID, and request status. Failed certificates omit language detail not already validated and include only sanitized boundary names. Never include raw logs, exception text, environment values, or auth status payloads.

Hash repository files with `sha256sum`. For each external loader hash, locate exactly one occurrence of the absolute repository file path, scan backward to its containing `(let` form, scan forward with balanced parentheses while respecting Emacs Lisp strings, escapes, and `;` comments, and SHA-256 the exact UTF-8 source slice. Zero or multiple matching forms fail the static gate.

Write atomically to `certificates/<YYYYMMDDTHHMMSSZ>-<git-sha>.json` with mode `0600`, validate it by reading it back, then print its relative path. Emit a smaller certificate on FAIL or BLOCKED_EXTERNAL before exiting 1 or 75.

- [ ] **Step 12: Run fake certifier contracts and shell syntax checks**

Run:

```bash
test/verify-ai-lsp-launcher certifier
bash -n bin/certify-ai-lsp
```

Expected: fake gate-order, failure-precedence, artifact, and cleanup contracts pass; syntax validation exits 0. The first real certifier invocation occurs only after the implementation and documentation are committed, because repository cleanliness is a mandatory first gate.

- [ ] **Step 13: Commit the certifier**

```bash
git add bin/certify-ai-lsp certificates/.gitignore test/verify-ai-lsp-launcher
git commit -m "feat: add complete AI LSP certification pipeline"
```

### Task 9: Document, Certify, Review, and Attest the Release

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-09-14-ai-lsp-certification-design.md` only if implementation revealed an approved design correction
- Create: ignored `certificates/<UTC>-<SHA>.json` through the certifier

**Interfaces:**
- Consumes: completed verifier/certifier and a healthy or explicitly blocked external Copilot service.
- Produces: operator documentation, a clean committed tree, final certificate, and a review with no Critical or Important findings.

- [ ] **Step 1: Update the README file map and command contract**

Document every file in the new layout, the immutable pins, GNU/Linux requirements, Python 3 requirement, the 360-second floor, public verifier modes, `bin/certify-ai-lsp`, exit 0/1/75 meanings, certificate path/permissions/schema, interpreted and compiled runs, mutation gates, and the fact that `all` does not run Doom sync.

- [ ] **Step 2: Add failure-oriented troubleshooting**

Document how to distinguish local `FAIL` from `BLOCKED_EXTERNAL`, how missing Clangd maps to both C++ and CUDA, how to rerun after Copilot authentication/network recovery, and that diagnostic artifacts are sanitized and bounded. State that users must never attach the Copilot auth database or unsanitized daemon output.

- [ ] **Step 3: Run syntax and focused regression checks**

Run:

```bash
bash -n bin/verify-ai-lsp bin/certify-ai-lsp test/verify-ai-lsp-launcher
test/verify-ai-lsp-launcher
bin/verify-ai-lsp unit
```

Expected: all exit 0; no fake PID remains; unit summary includes a positive completed count and zero unexpected results.

- [ ] **Step 4: Commit documentation before certification**

```bash
git add README.md docs/superpowers/specs/2026-09-14-ai-lsp-certification-design.md
git commit -m "docs: explain AI LSP certification"
```

If the design document did not require a correction, add only `README.md`.

- [ ] **Step 5: Run the complete certification on a clean commit**

Run:

```bash
bin/certify-ai-lsp
```

Expected on a healthy Copilot service: exit 0, all nine ordered gates PASS, four exact language records in both interpreted and compiled live runs, both mutations fail in their intended boundaries while later languages run, teardown is clean, and one ignored mode-`0600` certificate is printed.

Expected when Copilot alone is unavailable: exit 75 with a `BLOCKED_EXTERNAL` certificate; do not describe the system as certified until a later exit-0 run.

- [ ] **Step 6: Inspect the certificate for secret-shaped material and schema drift**

Run:

```bash
certificate="$(find certificates -maxdepth 1 -type f -name '*.json' -printf '%T@ %p\n' | sort -n | tail -1 | cut -d' ' -f2-)"
python3 -m json.tool "$certificate" >/dev/null
test "$(stat -c '%a' "$certificate")" = 600
if rg -i '(authorization|bearer |access[_-]?token|refresh[_-]?token|oauth[_-]?token|gh[opsu]_|github_pat_)' "$certificate"; then
  exit 1
fi
```

Expected: valid JSON, mode 600, and no secret-field or token-pattern match.

- [ ] **Step 7: Run the repository code-review gate**

Invoke the `code-review` skill against the commit before Task 1. Review both correctness-to-spec and engineering quality. Resolve every Critical and Important finding with a new red/green regression test, rerun the affected gates, and commit each coherent fix.

- [ ] **Step 8: Rerun the complete certificate after review fixes**

Run `bin/certify-ai-lsp` from the new clean commit.

Expected: exit 0 and the certificate's repository commit equals `git rev-parse HEAD`; the final review reports no Critical or Important findings.

- [ ] **Step 9: Record final evidence without committing the private certificate**

Run:

```bash
git status --short --branch
git check-ignore certificates/*.json
```

Expected: clean tracked tree, certificate reported as ignored, and no generated `.elc`, result, log, fixture, or temporary-root file outside `certificates/`.
