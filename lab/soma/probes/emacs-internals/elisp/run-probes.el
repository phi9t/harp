;;; run-probes.el --- executable probes for the Emacs internals atlas -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'json)
(require 'subr-x)

(defvar soma-probe-output-dir nil)
(defvar soma-probe-results nil)

(defun soma-probe--alist (&rest pairs)
  pairs)

(defun soma-probe--source (name path lines &optional note)
  (soma-probe--alist
   (cons "name" name)
   (cons "path" path)
   (cons "lines" lines)
   (cons "note" (or note ""))))

(defun soma-probe--bool (value)
  (if value t :json-false))

(defun soma-probe--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-probe--read-json-file (path)
  (when (and path (not (string-empty-p path)) (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-probe--insula-plan ()
  (soma-probe--read-json-file (or (getenv "SOMA_INSULA_PLAN") "")))

(defun soma-probe--plan-get (plan key)
  (alist-get key plan nil nil #'string=))

(defun soma-probe--development-insula-p (&optional plan)
  (let ((mode (or (and plan (soma-probe--plan-get plan "mode"))
                  (getenv "SOMA_INSULA_MODE")
                  "")))
    (not (null (string-match-p "development" mode)))))

(defun soma-probe--production-rootfs-p (&optional plan)
  (let ((rootfs (or (and plan (soma-probe--plan-get plan "rootfs")) ""))
        (closure (or (and plan (soma-probe--plan-get plan "dependency_closure")) "")))
    (and plan
         (not (soma-probe--development-insula-p plan))
         (not (string-empty-p rootfs))
         (not (string= rootfs "/"))
         (not (string= closure "not_rootfs_owned")))))

(defun soma-probe--real-emacs-path ()
  (file-truename
   (or (and (boundp 'invocation-directory)
            invocation-directory
            (expand-file-name invocation-name invocation-directory))
       (executable-find invocation-name)
       invocation-name)))

(defun soma-probe--shell-output (&rest args)
  (with-temp-buffer
    (let ((status (apply #'call-process (car args) nil t nil (cdr args)))
          (output (buffer-substring-no-properties (point-min) (point-max))))
      (soma-probe--alist
       (cons "status" status)
       (cons "output" output)))))

(defun soma-probe--tool-inventory (tools)
  (mapcar (lambda (tool)
            (let ((path (executable-find tool)))
              (soma-probe--alist
               (cons "tool" tool)
               (cons "path" (or path ""))
               (cons "found" (soma-probe--bool path)))))
          tools))

(defun soma-probe--missing-tools (inventory)
  (delq nil
        (mapcar (lambda (entry)
                  (unless (eq (alist-get "found" entry nil nil #'string=) t)
                    (alist-get "tool" entry nil nil #'string=)))
                inventory)))

(defun soma-probe--shell-output-timeout (timeout program &rest args)
  (let ((buffer (generate-new-buffer " *soma-probe-process*"))
        (status nil)
        (timed-out nil)
        (error-text nil)
        (output "")
        process)
    (unwind-protect
        (progn
          (condition-case err
              (setq process
                    (make-process
                     :name "soma-probe-process"
                     :buffer buffer
                     :command (cons program args)
                     :connection-type 'pipe
                     :noquery t))
            (error
             (setq error-text (format "%S" err))))
          (when process
            (let ((deadline (+ (float-time) timeout)))
              (while (and (process-live-p process)
                          (< (float-time) deadline))
                (accept-process-output process 0.05)))
            (if (process-live-p process)
                (progn
                  (setq timed-out t)
                  (delete-process process)
                  (setq status 'timeout))
              (setq status (process-exit-status process))))
          (with-current-buffer buffer
            (setq output (buffer-substring-no-properties (point-min) (point-max))))
          (soma-probe--alist
           (cons "status" status)
           (cons "timed_out" (soma-probe--bool timed-out))
           (cons "error" (or error-text ""))
           (cons "output" output)))
      (when (buffer-live-p buffer)
        (kill-buffer buffer)))))

(defun soma-probe--shell-output-truncated (timeout limit program &rest args)
  (let* ((result (apply #'soma-probe--shell-output-timeout timeout program args))
         (output (or (alist-get "output" result nil nil #'string=) "")))
    (setf (alist-get "output" result nil nil #'string=)
          (substring output 0 (min limit (length output))))
    result))

(defun soma-probe--python-script-output (script timeout &rest args)
  (let ((script-path (make-temp-file "soma-probe-" nil ".py")))
    (unwind-protect
        (progn
          (with-temp-file script-path
            (insert script))
          (apply #'soma-probe--shell-output-timeout
                 timeout "python3" script-path args))
      (when (file-exists-p script-path)
        (delete-file script-path)))))

(defun soma-probe--read-json-string (text)
  (let ((json-object-type 'alist)
        (json-array-type 'list)
        (json-key-type 'string)
        (json-false :json-false))
    (with-temp-buffer
      (insert text)
      (goto-char (point-min))
      (json-read))))

(defun soma-probe--json-line-with-prefix (prefix text)
  (catch 'found
    (dolist (line (split-string text "\n" t))
      (when (string-prefix-p prefix line)
        (throw 'found (substring line (length prefix)))))
    nil))

(defun soma-probe--wait-buffer-regexp (buffer regexp timeout)
  (let ((deadline (+ (float-time) timeout))
        found)
    (while (and (not found) (< (float-time) deadline))
      (accept-process-output (get-buffer-process buffer) 0.05)
      (with-current-buffer buffer
        (setq found (save-excursion
                      (goto-char (point-min))
                      (re-search-forward regexp nil t)))))
    found))

(defun soma-probe--forbidden-rootfs-path-p (path)
  (and (stringp path)
       (or (string-prefix-p "/nix/" path)
           (string-prefix-p "/nix/store/" path)
           (string-prefix-p "/data02/home/" path)
           (string-prefix-p "/workspace/" path)
           (string-prefix-p "/project/" path))))

(defun soma-probe--rootfs-owned-executable-path-p (path)
  (and (stringp path)
       (not (string-empty-p path))
       (file-name-absolute-p path)
       (not (soma-probe--forbidden-rootfs-path-p path))))

(defun soma-probe--tool-provenance (tool)
  (let* ((path (executable-find tool))
         (realpath (and path (file-truename path)))
         (version (and path
                       (soma-probe--shell-output-truncated
                        3 1200 path "--version")))
         (sha (and path
                   (executable-find "sha256sum")
                   (soma-probe--shell-output-truncated
                    3 200 "sha256sum" path)))
         (ldd (and path
                   (executable-find "ldd")
                   (soma-probe--shell-output-truncated
                    3 2000 "ldd" path))))
    (soma-probe--alist
     (cons "tool" tool)
     (cons "path" (or path ""))
     (cons "realpath" (or realpath ""))
     (cons "found" (soma-probe--bool path))
     (cons "rootfs_owned_path"
           (soma-probe--bool (soma-probe--rootfs-owned-executable-path-p
                              (or realpath path ""))))
     (cons "version_status" (or (and version
                                     (alist-get "status" version nil nil #'string=))
                                ""))
     (cons "version_output" (or (and version
                                     (alist-get "output" version nil nil #'string=))
                                ""))
     (cons "sha256_output" (or (and sha
                                  (alist-get "output" sha nil nil #'string=))
                               ""))
     (cons "ldd_status" (or (and ldd
                                 (alist-get "status" ldd nil nil #'string=))
                            ""))
     (cons "ldd_output" (or (and ldd
                                 (alist-get "output" ldd nil nil #'string=))
                            "")))))

(defun soma-probe--env-names-from-lines (text)
  (sort
   (delq nil
         (mapcar (lambda (line)
                   (when (string-match "\\`\\([^=]+\\)=" line)
                     (match-string 1 line)))
                 (split-string text "\n" t)))
   #'string<))

(defun soma-probe--known-private-path-p (path)
  (and (stringp path)
       (or (string-prefix-p "/tmp/soma-insula-home" path)
           (string-prefix-p "/tmp/soma-insula-tmp" path)
           (string-prefix-p "/tmp/soma-insula-run" path)
           (string-prefix-p "/project/" path)
           (string-prefix-p "/workspace/soma" path))))

(defun soma-probe--doom-straight-build-dir ()
  (expand-file-name
   (format "~/.emacs.d/.local/straight/build-%d.%d"
           emacs-major-version emacs-minor-version)))

(defun soma-probe--emacs-identity ()
  (soma-probe--alist
   (cons "emacs_version" emacs-version)
   (cons "system_configuration" system-configuration)
   (cons "system_features" system-configuration-features)
   (cons "executable" invocation-name)
   (cons "exec_directory" exec-directory)
   (cons "source_directory_reported" source-directory)
   (cons "source_directory_exists" (soma-probe--bool (file-directory-p source-directory)))
   (cons "home" (or (getenv "HOME") ""))
   (cons "tmpdir" (or (getenv "TMPDIR") ""))
   (cons "xdg_runtime_dir" (or (getenv "XDG_RUNTIME_DIR") ""))
   (cons "xdg_cache_home" (or (getenv "XDG_CACHE_HOME") ""))
   (cons "xdg_data_home" (or (getenv "XDG_DATA_HOME") ""))
   (cons "user_emacs_directory" user-emacs-directory)
   (cons "doom_core" (expand-file-name "~/.emacs.d"))
   (cons "doom_user" (expand-file-name "~/.doom.d"))
   (cons "straight_build" (soma-probe--doom-straight-build-dir))
   (cons "straight_repos" (expand-file-name "~/.emacs.d/.local/straight/repos"))
   (cons "insula_active" (soma-probe--bool (getenv "SOMA_INSULA_ACTIVE")))
   (cons "insula_mode" (or (getenv "SOMA_INSULA_MODE") "none"))
   (cons "insula_plan" (or (getenv "SOMA_INSULA_PLAN") ""))))

(defun soma-probe--insula-runtime ()
  (let* ((env-names (sort (mapcar (lambda (item) (car (split-string item "=" t)))
                                  process-environment)
                          #'string<))
         (allowed '("HOME" "LANG" "LC_ALL" "PATH" "PWD" "SHELL"
                    "SHLVL" "_"
                    "CARGO_HOME" "RUSTUP_HOME"
                    "SOMA_INSULA_ACTIVE" "SOMA_INSULA_MODE"
                    "SOMA_INSULA_PLAN" "SOMA_INSULA_RUN_ID"
                    "TMPDIR" "XDG_CACHE_HOME" "XDG_CONFIG_HOME"
                    "XDG_DATA_HOME" "XDG_RUNTIME_DIR"))
         (unexpected (seq-remove (lambda (name) (member name allowed)) env-names))
         (home (or (getenv "HOME") ""))
         (tmpdir (or (getenv "TMPDIR") ""))
         (runtime (or (getenv "XDG_RUNTIME_DIR") ""))
         (plan (or (getenv "SOMA_INSULA_PLAN") "")))
    (soma-probe--alist
     (cons "active" (soma-probe--bool (string= (getenv "SOMA_INSULA_ACTIVE") "1")))
     (cons "mode" (or (getenv "SOMA_INSULA_MODE") "none"))
     (cons "home" home)
     (cons "tmpdir" tmpdir)
     (cons "xdg_runtime_dir" runtime)
     (cons "plan" plan)
     (cons "env_names" env-names)
     (cons "unexpected_env_names" unexpected)
     (cons "checks"
           (list
            (soma-probe--check "SOMA_INSULA_ACTIVE is set"
                               (string= (getenv "SOMA_INSULA_ACTIVE") "1")
                               (or (getenv "SOMA_INSULA_ACTIVE") ""))
            (soma-probe--check "HOME is private"
                               (soma-probe--known-private-path-p home)
                               home)
            (soma-probe--check "TMPDIR is private"
                               (soma-probe--known-private-path-p tmpdir)
                               tmpdir)
            (soma-probe--check "XDG_RUNTIME_DIR is private"
                               (soma-probe--known-private-path-p runtime)
                               runtime)
            (soma-probe--check "environment is allowlisted"
                               (null unexpected)
                               (format "%S" unexpected))
            (soma-probe--check "insula plan is visible"
                               (and (not (string-empty-p plan))
                                    (file-readable-p plan))
                               plan))))))

(defun soma-probe--add-straight-build-paths ()
  (let ((build (soma-probe--doom-straight-build-dir)))
    (when (file-directory-p build)
      (dolist (dir (directory-files build t directory-files-no-dot-files-regexp))
        (when (file-directory-p dir)
          (add-to-list 'load-path dir))))))

(defun soma-probe--check (name passed detail)
  (soma-probe--alist
   (cons "name" name)
   (cons "passed" (soma-probe--bool passed))
   (cons "detail" detail)))

(defun soma-probe--all-checks-pass-p (checks)
  (cl-every (lambda (check) (eq (alist-get "passed" check nil nil #'string=) t))
            checks))

(defun soma-probe--record (id title status mechanism hypothesis sources fixture
                              predicted observed checks counterexample requirement)
  (let ((record (soma-probe--alist
                 (cons "id" id)
                 (cons "title" title)
                 (cons "status" status)
                 (cons "execution_mode" "emacs --quick --batch")
                 (cons "environment" (soma-probe--emacs-identity))
                 (cons "mechanism" mechanism)
                 (cons "hypothesis" hypothesis)
                 (cons "sources" sources)
                 (cons "minimal_fixture" fixture)
                 (cons "predicted_behavior" predicted)
                 (cons "observed_behavior" observed)
                 (cons "independent_checks" checks)
                 (cons "counterexample" counterexample)
                 (cons "derived_gym_requirement" requirement))))
    (let ((index (1+ (length soma-probe-results))))
      (push record soma-probe-results)
      (soma-probe--write-json
       (expand-file-name (format "%02d-%s.json" index id)
                         soma-probe-output-dir)
       record))
    record))

(defmacro soma-probe--case (id title mechanism hypothesis sources fixture
                               predicted counterexample requirement &rest body)
  `(condition-case err
       (let* ((result (progn ,@body))
              (checks (alist-get "checks" result nil nil #'string=))
              (observed (alist-get "observed" result nil nil #'string=))
              (status (if (soma-probe--all-checks-pass-p checks) "pass" "fail")))
         (soma-probe--record ,id ,title status ,mechanism ,hypothesis ,sources
                             ,fixture ,predicted observed checks ,counterexample
                             ,requirement))
     (error
      (soma-probe--record
       ,id ,title "error" ,mechanism ,hypothesis ,sources ,fixture ,predicted
       (format "Probe raised %S" err)
       (list (soma-probe--check "probe completed" nil (format "%S" err)))
       ,counterexample ,requirement))))

(defun soma-probe-current-buffer-window ()
  (soma-probe--case
   "current-buffer-selected-window"
   "Current buffer versus selected window"
   "Current buffer, selected window, and displayed buffer are separate pieces of state."
   "`with-current-buffer' should change the current buffer for the dynamic extent without changing the selected window's displayed buffer; `switch-to-buffer' should display the target in the selected window."
   (list
    (soma-probe--source "with-current-buffer" "subr.el.gz" "4849")
    (soma-probe--source "selected-window helpers" "window.el.gz" "30-75")
    (soma-probe--source "switch-to-buffer references" "window.el.gz" "4037-4059, 5013, 6942"))
   "Two temporary buffers and the initial selected window in batch Emacs."
   "Reading or mutating a hidden buffer with `with-current-buffer' will not display it. Selecting/showing it requires a window operation."
   "Treating `with-current-buffer' as equivalent to user-visible selection loses window history and focus semantics."
   "Bridge operations must separate read/operate-in-buffer, display-buffer, and select-for-user actions."
   (let* ((buf-a (get-buffer-create "soma-probe-a"))
          (buf-b (get-buffer-create "soma-probe-b"))
          selected-before selected-during current-during selected-after)
     (unwind-protect
         (progn
           (switch-to-buffer buf-a)
           (setq selected-before (buffer-name (window-buffer (selected-window))))
           (with-current-buffer buf-b
             (setq current-during (buffer-name (current-buffer)))
             (setq selected-during (buffer-name (window-buffer (selected-window)))))
           (switch-to-buffer buf-b)
           (setq selected-after (buffer-name (window-buffer (selected-window))))
           (soma-probe--alist
            (cons "observed"
                  (format "selected-before=%s current-during=%s selected-during=%s selected-after=%s"
                          selected-before current-during selected-during selected-after))
            (cons "checks"
                  (list
                   (soma-probe--check "with-current-buffer changed current buffer"
                                      (string= current-during "soma-probe-b")
                                      current-during)
                   (soma-probe--check "with-current-buffer preserved selected window buffer"
                                      (string= selected-during "soma-probe-a")
                                      selected-during)
                   (soma-probe--check "switch-to-buffer displayed target"
                                      (string= selected-after "soma-probe-b")
                                      selected-after)))))
       (kill-buffer buf-a)
       (kill-buffer buf-b)))))

(defvar soma-probe-command-events nil)

(defun soma-probe-command (&optional arg)
  "Record command context for route-comparison probes."
  (interactive "P")
  (push (soma-probe--alist
         (cons "arg" (format "%S" arg))
         (cons "this_command" (format "%S" this-command))
         (cons "last_command" (format "%S" last-command))
         (cons "called_interactively" (soma-probe--bool (called-interactively-p 'interactive))))
        soma-probe-command-events))

(defun soma-probe-command-routes ()
  (soma-probe--case
   "command-routes"
   "Direct call versus interactive invocation versus command dispatch"
   "Command execution path changes implicit context such as interactive arguments, command recording, and keymap resolution."
   "In batch Emacs, `call-interactively' and `command-execute' acquire arguments but do not reproduce full command-loop state; keyboard-macro dispatch through a keymap changes `this-command'."
   (list
    (soma-probe--source "command-execute" "simple.el.gz" "2753-2815")
    (soma-probe--source "call-interactively caller sites" "simple.el.gz" "2204, 2797-2804")
    (soma-probe--source "Doom map!" "~/.emacs.d/lisp/doom-keybinds.el" "439-486"))
   "One temporary interactive command, one temporary key binding, and no real PTY."
   "Direct call, `call-interactively', `command-execute', and keymap dispatch are observably different in batch, but none prove terminal byte decoding through Ghostty/SSH/tmux."
   "A bridge that only invokes functions directly can miss interactive argument acquisition and command-loop bookkeeping."
   "Action profiles must classify direct, interactive, command-dispatch, and terminal-input paths; terminal fidelity remains a separate PTY probe gate."
   (let ((soma-probe-command-events nil))
     (soma-probe-command 'direct)
     (let ((current-prefix-arg '(4)))
       (call-interactively #'soma-probe-command))
     (let ((current-prefix-arg '(16)))
       (command-execute #'soma-probe-command 'record))
     (let ((map (make-sparse-keymap)))
       (define-key map (kbd "C-c p") #'soma-probe-command)
       (let ((overriding-terminal-local-map map)
             (current-prefix-arg nil))
         (execute-kbd-macro (kbd "C-c p"))))
     (let* ((events (nreverse soma-probe-command-events))
            (direct (nth 0 events))
            (call (nth 1 events))
            (command (nth 2 events))
            (key (nth 3 events)))
       (soma-probe--alist
        (cons "observed" (format "%S" events))
        (cons "checks"
              (list
               (soma-probe--check "four routes recorded" (= (length events) 4)
                                  (number-to-string (length events)))
               (soma-probe--check "direct call was not command-loop interactive"
                                  (eq (alist-get "called_interactively" direct nil nil #'string=)
                                      :json-false)
                                  (format "%S" direct))
               (soma-probe--check "call-interactively acquired prefix argument"
                                  (string= (alist-get "arg" call nil nil #'string=) "(4)")
                                  (format "%S" call))
               (soma-probe--check "command-execute route recorded separately"
                                  command
                                  (format "%S" command))
               (soma-probe--check "keymap dispatch set this-command"
                                  (string= (alist-get "this_command" key nil nil #'string=)
                                           "soma-probe-command")
                                  (format "%S" key)))))))))

(defun soma-probe-magit-wrapper ()
  (soma-probe--case
   "magit-wrapper-this-command"
   "Magit post-stage hook and bridge wrapper command identity"
   "`magit-run-post-stage-hook' uses command identity, so a direct function call inside a wrapper can have correct Git effects but different Magit semantics."
   "Calling `magit-run-post-stage-hook' while `this-command' is a bridge wrapper should not run `magit-post-stage-hook`; setting `this-command' to a recognized staging command should run it."
   (list
    (soma-probe--source "magit-stage dispatch" "~/.emacs.d/.local/straight/repos/magit/lisp/magit-apply.el" "282-306")
    (soma-probe--source "magit-stage-1 side effects" "~/.emacs.d/.local/straight/repos/magit/lisp/magit-apply.el" "340-345")
    (soma-probe--source "magit-run-post-stage-hook" "~/.emacs.d/.local/straight/repos/magit/lisp/magit-apply.el" "400-408")
    (soma-probe--source "magit post-refresh hook includes post-stage runner" "~/.emacs.d/.local/straight/repos/magit/lisp/magit-mode.el" "94-115"))
   "Magit loaded from the installed Straight build; no Git repository is mutated."
   "Hook execution depends on `this-command', independent of whether lower-level Git state was already changed."
   "A wrapper command that calls Magit lower-level functions can pass an index checker while skipping package hooks."
   "Faithful Magit actions must either enter through a recognized command path or explicitly model and test the command identity consequences."
   (soma-probe--add-straight-build-paths)
   (require 'magit-apply)
   (let ((runs nil))
     (cl-letf (((symbol-function 'soma-probe-post-stage-hook)
                (lambda () (push (format "%S" this-command) runs))))
       (let ((magit-post-stage-hook '(soma-probe-post-stage-hook)))
         (let ((this-command 'soma-bridge-stage))
           (magit-run-post-stage-hook))
         (let ((this-command 'magit-stage))
           (magit-run-post-stage-hook))))
     (soma-probe--alist
      (cons "observed" (format "post-stage-hook-runs=%S" (nreverse runs)))
      (cons "checks"
            (list
             (soma-probe--check "wrapper command did not trigger post-stage hook"
                                (= (length runs) 1)
                                (format "%S" runs))
             (soma-probe--check "recognized staging command triggered hook"
                                (member "magit-stage" runs)
                                (format "%S" runs))))))))

(defun soma-probe-indirect-buffer ()
  (soma-probe--case
   "base-indirect-buffers"
   "Base and indirect buffers share text"
   "Indirect buffers are distinct buffer objects whose text changes affect the base buffer."
   "Editing the indirect buffer should change the base buffer text while preserving separate buffer identities."
   (list
    (soma-probe--source "buffer identity runtime" "Emacs primitive introspection" "buffer-base-buffer, clone-indirect-buffer")
    (soma-probe--source "reported C source gap" "source-directory" "missing Spack source directory"))
   "One base buffer and one indirect buffer."
   "The base buffer text changes after insertion in the indirect buffer; buffer-local state such as point can still differ."
   "A checker that says only one buffer object changed misses shared text ownership."
   "Observations must include buffer identity, base-buffer/text-owner identity, and visited file identity separately."
   (let* ((base (get-buffer-create "soma-base"))
          (indirect nil))
     (unwind-protect
         (with-current-buffer base
           (erase-buffer)
           (insert "alpha")
           (setq indirect (clone-indirect-buffer "soma-indirect" nil))
           (with-current-buffer indirect
             (goto-char (point-max))
             (insert "-from-indirect"))
           (soma-probe--alist
            (cons "observed"
                  (format "base=%S indirect=%S indirect-base=%S base-text=%S"
                          (buffer-name base)
                          (buffer-name indirect)
                          (buffer-name (buffer-base-buffer indirect))
                          (buffer-string)))
            (cons "checks"
                  (list
                   (soma-probe--check "buffer objects differ" (not (eq base indirect))
                                      (format "base-eq-indirect=%S" (eq base indirect)))
                   (soma-probe--check "indirect records base buffer" (eq (buffer-base-buffer indirect) base)
                                      (format "%S" (buffer-base-buffer indirect)))
                   (soma-probe--check "base text reflects indirect edit" (string= (buffer-string) "alpha-from-indirect")
                                      (buffer-string))))))
       (when (buffer-live-p indirect) (kill-buffer indirect))
       (when (buffer-live-p base) (kill-buffer base))))))

(defun soma-probe-marker-movement ()
  (soma-probe--case
   "marker-movement"
   "Marker movement under edits"
   "Markers preserve a moving position, not a semantic target."
   "A marker should move when text is inserted before it, while text around the marker may no longer match the original target context."
   (list
    (soma-probe--source "marker runtime" "Emacs primitive introspection" "make-marker, set-marker, marker-position"))
   "One temporary buffer, one marker before a target character, and two edits before/around it."
   "The marker remains live and moves, but the original semantic context is stale."
   "Accepting a delayed edit because its marker is live can apply it to the wrong text."
   "Delayed edits and FIM suggestions must validate prefix, suffix, buffer incarnation, and target range text before mutation."
   (with-temp-buffer
     (insert "abcDEF")
     (let* ((marker (copy-marker 4))
            (before-pos (marker-position marker))
            (before-context (buffer-substring-no-properties (max 1 (- before-pos 1))
                                                            (min (point-max) (+ before-pos 2)))))
       (goto-char 1)
       (insert "xxx")
       (goto-char marker)
       (delete-char 1)
       (insert "Z")
       (let* ((after-pos (marker-position marker))
              (after-context (buffer-substring-no-properties (max 1 (- after-pos 1))
                                                             (min (point-max) (+ after-pos 2)))))
         (soma-probe--alist
          (cons "observed"
                (format "before-pos=%s before-context=%S after-pos=%s after-context=%S text=%S"
                        before-pos before-context after-pos after-context (buffer-string)))
          (cons "checks"
                (list
                 (soma-probe--check "marker moved after insertion before it"
                                    (> after-pos before-pos)
                                    (format "%s -> %s" before-pos after-pos))
                 (soma-probe--check "original context changed"
                                    (not (string= before-context after-context))
                                    (format "%S -> %S" before-context after-context))))))))))

(defun soma-probe-narrowing-region-ticks ()
  (soma-probe--case
   "narrowing-region-without-text-change"
   "Narrowing or region changes without character changes"
   "Character modification ticks do not cover interaction state such as narrowing, point, mark, and active region."
   "Changing narrowing and the active region should leave `buffer-chars-modified-tick' unchanged."
   (list
    (soma-probe--source "buffer revision runtime" "Emacs primitive introspection" "buffer-chars-modified-tick")
    (soma-probe--source "narrowing runtime" "Emacs primitive introspection" "narrow-to-region, widen"))
   "One temporary buffer with stable text."
   "Character tick remains stable while visible accessible range and region state change."
   "A stale-edit guard using only character ticks misses interaction state changes."
   "Every action needs operation-specific preconditions beyond a universal character revision guard."
   (with-temp-buffer
     (insert "one\ntwo\nthree\n")
     (let ((tick0 (buffer-chars-modified-tick))
           (min0 (point-min))
           (max0 (point-max)))
       (goto-char 5)
       (set-mark 8)
       (setq mark-active t)
       (narrow-to-region 5 9)
       (let ((tick1 (buffer-chars-modified-tick)))
         (soma-probe--alist
          (cons "observed"
                (format "tick=%s->%s range=%s-%s->%s-%s region-active=%S"
                        tick0 tick1 min0 max0 (point-min) (point-max) mark-active))
          (cons "checks"
                (list
                 (soma-probe--check "character tick unchanged" (= tick0 tick1)
                                    (format "%s -> %s" tick0 tick1))
                 (soma-probe--check "accessible range changed"
                                    (or (/= min0 (point-min)) (/= max0 (point-max)))
                                    (format "%s-%s -> %s-%s" min0 max0 (point-min) (point-max)))
                 (soma-probe--check "region active changed interaction state"
                                    mark-active
                                    (format "mark=%s point=%s" (mark) (point)))))))))))

(defun soma-probe-minibuffer-transient ()
  (soma-probe--case
   "minibuffer-transient-interruption"
   "Minibuffer cancellation and transient argument context"
   "Prompt cancellation and transient suffix execution are interaction states, not ordinary function calls."
   "A batch minibuffer can be canceled and observed as a quit; transient source can be loaded and its command-context requirement recorded, but real transient keymaps require a terminal-frame probe."
   (list
    (soma-probe--source "minibuffer selected window references" "minibuffer.el.gz" "212-258")
    (soma-probe--source "transient source" "~/.emacs.d/.local/straight/repos/transient/lisp/transient.el" "installed package"))
   "One minibuffer setup hook that aborts recursive edit; transient source availability check."
   "The minibuffer cancellation is observable as a quit. Transient terminal-key lifecycle is not fully exercised in batch."
   "Treating every prompt as a direct function call cannot model cancellation and continuation."
   "Bridge observations must expose active minibuffer/transient state, and terminal-path probes must cover transient prefix/infix/suffix lifetimes before those actions are benchmarked."
   (soma-probe--add-straight-build-paths)
   (let ((minibuffer-result nil)
         (transient-loadable (locate-library "transient")))
     (condition-case nil
         (minibuffer-with-setup-hook
             (lambda () (abort-recursive-edit))
           (read-from-minibuffer "soma-probe: "))
       (quit (setq minibuffer-result 'quit))
       (end-of-file (setq minibuffer-result 'end-of-file)))
     (soma-probe--alist
      (cons "observed"
            (format "minibuffer-result=%S transient-library=%S" minibuffer-result transient-loadable))
      (cons "checks"
            (list
             (soma-probe--check "minibuffer cancellation produced quit"
                                (memq minibuffer-result '(quit end-of-file))
                                (format "%S" minibuffer-result))
             (soma-probe--check "transient package source is available for terminal follow-up"
                                (and transient-loadable t)
                                (format "%S" transient-loadable))))))))

(defun soma-probe-reentrant-process-output ()
  (soma-probe--case
   "reentrant-process-output"
   "Reentrant process output while instrumentation is active"
   "Process filters can run during waits initiated from Lisp; instrumentation must tolerate callbacks during an action."
   "A filter that calls `accept-process-output' while the process continues producing output can observe nested or at least interleaved output processing before the outer action returns."
   (list
    (soma-probe--source "Comint process mark source" "comint.el.gz" "467, 889, 998")
    (soma-probe--source "process wait primitive" "Emacs primitive introspection" "accept-process-output"))
   "One shell process that emits two lines with a short delay and one filter that waits inside the first callback."
   "The trace records process output arriving while the filter/action is active; recursive depth may be runtime-dependent."
   "Assuming no asynchronous callbacks occur until an operation returns hides race points."
   "Actions that wait for external output must repeat relevant consistency checks after waits and represent outcome_unknown where completion cannot be proven."
   (let ((events nil)
         (max-depth 0)
         (depth 0)
         proc)
     (setq proc
           (make-process
            :name "soma-probe-output"
            :buffer (generate-new-buffer " *soma-probe-output*")
            :command '("sh" "-c" "printf first; sleep 0.1; printf second")
            :noquery t
            :filter (lambda (p text)
                      (setq depth (1+ depth))
                      (setq max-depth (max max-depth depth))
                      (push (format "depth=%d text=%S" depth text) events)
                      (when (string-match-p "first" text)
                        (accept-process-output p 0.3))
                      (setq depth (1- depth)))))
     (while (process-live-p proc)
       (accept-process-output proc 0.5))
     (when (buffer-live-p (process-buffer proc))
       (kill-buffer (process-buffer proc)))
     (soma-probe--alist
      (cons "observed" (format "max-depth=%s events=%S" max-depth (nreverse events)))
      (cons "checks"
            (list
             (soma-probe--check "process output captured"
                                (>= (length events) 1)
                                (format "%S" events))
             (soma-probe--check "action observed output before final completion"
                                (cl-some (lambda (event) (string-match-p "first\\|second" event)) events)
                                (format "%S" events))))))))

(defun soma-probe-doom-cold-warm ()
  (soma-probe--case
   "doom-cold-warmed-state"
   "Cold versus warmed Doom session"
   "Doom environment identity includes generated profile/module artifacts and lazy package activation state, not just user declarations."
   "The generated module file should exist and list declared modules; package autoload files should exist before the corresponding feature is loaded in a clean batch process."
   (list
    (soma-probe--source "doom! macro" "~/.emacs.d/lisp/doom-lib.el" "1528-1547")
    (soma-probe--source "generated module state" "~/.emacs.d/.local/etc/@/init.30.d/95-modules.auto.el" "1")
    (soma-probe--source "user module declarations" "~/.doom.d/init.el" "1-220"))
   "Filesystem-only inspection under `emacs --quick --batch`; no user Doom session is opened."
   "Generated modules and autoloads exist while features such as Magit are not loaded in the clean probe process."
   "Inferring readiness from `doom!' declarations confuses module declaration with installed package, autoload, loaded feature, active mode, or external service."
   "Gym environment identity must record Doom core/user config/generated profile/package build ids and current feature/mode/service readiness separately."
   (let* ((module-file (expand-file-name "~/.emacs.d/.local/etc/@/init.30.d/95-modules.auto.el"))
          (autoload-file (expand-file-name "~/.emacs.d/.local/straight/build-30.2/magit/magit-autoloads.el"))
          (module-text (and (file-readable-p module-file)
                            (with-temp-buffer
                              (insert-file-contents module-file nil 0 4000)
                              (buffer-string))))
          (has-magit-module (and module-text (string-match-p "(:tools \\. magit)" module-text)))
          (feature-before (featurep 'magit)))
     (soma-probe--alist
      (cons "observed"
            (format "module-file=%S has-magit-module=%S magit-autoload=%S feature-before=%S"
                    module-file has-magit-module (file-readable-p autoload-file) feature-before))
      (cons "checks"
            (list
             (soma-probe--check "generated module file exists" (file-readable-p module-file) module-file)
             (soma-probe--check "generated module file records Magit module" has-magit-module
                                (if module-text "found or not found in first 4000 chars" "module file unreadable"))
             (soma-probe--check "Magit autoload/build artifact exists" (file-readable-p autoload-file) autoload-file)
             (soma-probe--check "Magit feature is not loaded in clean quick batch"
                                (not feature-before)
                                (format "%S" feature-before))))))))

(defun soma-probe-org-src-propagation ()
  (soma-probe--case
   "org-source-edit-propagation"
   "Org source-edit propagation"
   "Org source block editing uses a separate edit buffer and writes back on the source-edit exit path."
   "Editing the source edit buffer should not immediately prove the parent Org block changed; `org-edit-src-exit' should propagate the edited contents."
   (list
    (soma-probe--source "org-edit-src-code" "~/.emacs.d/.local/straight/repos/org/lisp/org-src.el" "1321-1363")
    (soma-probe--source "org-edit-src-save/exit" "~/.emacs.d/.local/straight/repos/org/lisp/org-src.el" "1447-1494"))
   "One temporary Org buffer containing one emacs-lisp source block."
   "The edit buffer change becomes authoritative only after the exit/write-back path."
   "Checking only the edit buffer can report success when the parent Org document is unchanged."
   "Org gym actions must distinguish source-edit buffers from parent Org buffers and grade the parent document after write-back."
   (soma-probe--add-straight-build-paths)
   (require 'org)
   (require 'org-src)
   (let ((parent-text-before nil)
         (parent-text-after-edit nil)
         (parent-text-after-exit nil))
     (with-temp-buffer
       (org-mode)
       (insert "#+begin_src emacs-lisp\n(+ 1 2)\n#+end_src\n")
       (goto-char (point-min))
       (search-forward "(+ 1 2)")
       (setq parent-text-before (buffer-string))
       (org-edit-src-code)
       (with-current-buffer (current-buffer)
         (erase-buffer)
         (insert "(+ 3 4)\n")
         (setq parent-text-after-edit
               (with-current-buffer (org-src-source-buffer)
                 (buffer-string)))
         (org-edit-src-exit))
       (setq parent-text-after-exit (buffer-string)))
     (soma-probe--alist
      (cons "observed"
            (format "before=%S after-edit-parent=%S after-exit-parent=%S"
                    parent-text-before parent-text-after-edit parent-text-after-exit))
      (cons "checks"
            (list
             (soma-probe--check "parent unchanged before source-edit exit"
                                (string= parent-text-before parent-text-after-edit)
                                parent-text-after-edit)
             (soma-probe--check "parent updated after source-edit exit"
                                (string-match-p (regexp-quote "(+ 3 4)") parent-text-after-exit)
                                parent-text-after-exit)))))))

(defun soma-probe-jupyter-delayed-retry ()
  (soma-probe--case
   "jupyter-delayed-reply-duplicate-retry"
   "Jupyter delayed reply and duplicate retry"
   "A timeout after request submission means the outcome is unknown; retrying a non-idempotent execution can duplicate work."
   "The fake transport should mark a request as submitted, report outcome_unknown before the delayed reply, and demonstrate that a blind retry creates a second request."
   (list
    (soma-probe--source "jupyter current client association" "~/.emacs.d/.local/straight/repos/jupyter/jupyter-repl.el" "2006-2014")
    (soma-probe--source "jupyter completion busy guard" "~/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el" "1614-1638")
    (soma-probe--source "jupyter code context" "~/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el" "1283-1308"))
   "A deterministic fake request table and timer; no real kernel is launched."
   "Before the delayed reply arrives the only honest state is outcome_unknown. A retry increments submitted request count."
   "Treating timeout as not-run can submit a training cell twice."
   "Jupyter actions need request ids, parent headers or equivalent handles, kernel incarnation, and an outcome_unknown state that blocks blind retry of non-idempotent code."
   (let ((requests nil)
         (next-id 0)
         (reply nil))
     (cl-labels ((submit
                  (code)
                  (setq next-id (1+ next-id))
                  (push (list :id next-id :code code :state 'submitted) requests)
                  (run-at-time
                   0.15 nil
                   (lambda (id)
                     (setq reply id)
                     (setq requests
                           (mapcar (lambda (req)
                                     (if (= (plist-get req :id) id)
                                         (plist-put req :state 'replied)
                                       req))
                                   requests)))
                   next-id)
                  next-id))
       (let* ((id1 (submit "counter += 1"))
              (_unknown (unless reply 'outcome_unknown))
              (id2 (submit "counter += 1")))
         (sleep-for 0.25)
         (soma-probe--alist
          (cons "observed"
                (format "first=%s retry=%s reply=%S requests=%S" id1 id2 reply (reverse requests)))
          (cons "checks"
                (list
                 (soma-probe--check "timeout before reply is outcome_unknown"
                                    (not (null _unknown))
                                    (format "%S" _unknown))
                 (soma-probe--check "blind retry created a second request"
                                    (= (length requests) 2)
                                    (format "%S" requests))
                 (soma-probe--check "delayed replies eventually arrived"
                                    (cl-every (lambda (req)
                                                (eq (plist-get req :state) 'replied))
                                              requests)
                                    (format "%S" requests))))))))))

(defun soma-probe-fim-stale-acceptance ()
  (soma-probe--case
   "fim-stale-acceptance"
   "FIM acceptance after human editing"
   "A delayed completion suggestion must be rejected if the buffer context that authorized it is stale."
   "Changing the prefix before accepting a suggestion should change the character tick and prefix validation even if a marker still identifies a live position."
   (list
    (soma-probe--source "buffer revision runtime" "Emacs primitive introspection" "buffer-chars-modified-tick")
    (soma-probe--source "marker runtime" "Emacs primitive introspection" "copy-marker"))
   "One temporary code buffer, one suggestion snapshot, one simulated human edit."
   "The stale suggestion is rejected because tick and prefix changed."
   "Accepting ghost text solely by marker position can splice stale text into a human-edited buffer."
   "Completion acceptance must validate buffer identity, modification tick, target range, prefix, suffix, and intended replacement before editing."
   (with-temp-buffer
     (insert "def f(x):\n    return x\n")
     (goto-char (point-max))
     (let* ((marker (copy-marker (point)))
            (snapshot-tick (buffer-chars-modified-tick))
            (snapshot-prefix (buffer-substring-no-properties (point-min) (point)))
            (snapshot-suffix "")
            (suggestion " + 1")
            accepted-p current-prefix current-suffix)
       (goto-char (point-min))
       (search-forward "x")
       (replace-match "value")
       (goto-char marker)
       (setq current-prefix (buffer-substring-no-properties (point-min) marker))
       (setq current-suffix (buffer-substring-no-properties marker (point-max)))
       (setq accepted-p
             (and (= snapshot-tick (buffer-chars-modified-tick))
                  (string= snapshot-prefix current-prefix)
                  (string= snapshot-suffix current-suffix)))
       (when accepted-p
         (insert suggestion))
       (soma-probe--alist
        (cons "observed"
              (format "marker=%s tick=%s->%s accepted=%S text=%S"
                      (marker-position marker)
                      snapshot-tick (buffer-chars-modified-tick)
                      accepted-p (buffer-string)))
        (cons "checks"
              (list
               (soma-probe--check "marker remained live" (marker-buffer marker)
                                  (format "%S" (marker-position marker)))
               (soma-probe--check "buffer tick changed after human edit"
                                  (/= snapshot-tick (buffer-chars-modified-tick))
                                  (format "%s -> %s" snapshot-tick (buffer-chars-modified-tick)))
               (soma-probe--check "stale suggestion rejected"
                                  (not accepted-p)
                                  (format "accepted=%S suggestion=%S" accepted-p suggestion)))))))))

(defun soma-probe-observation-fontification ()
  (soma-probe--case
   "observation-fontification-side-effect"
   "Observation reads can trigger fontification hooks"
   "`buffer-substring' and `buffer-substring-no-properties' are different observation operations; the former can run `buffer-access-fontify-functions'."
   "Reading text with properties should invoke `buffer-access-fontify-functions', while `buffer-substring-no-properties' should not invoke the access fontification hook."
   (list
    (soma-probe--source "buffer-substring" "C-source" "Emacs primitive introspection")
    (soma-probe--source "buffer-substring-no-properties" "C-source" "Emacs primitive introspection")
    (soma-probe--source "buffer-access-fontify-functions" "Emacs runtime variable" "observed by local hook execution"))
   "One temporary buffer with a buffer-local `buffer-access-fontify-functions' hook."
   "`buffer-substring-no-properties' returns text without triggering the hook; `buffer-substring' triggers it for the accessed range."
   "A read-only observer that uses the wrong text API can load or compute display-derived state while claiming to be passive."
   "Observation APIs must be classified as passive state reads, derived inspections, or active diagnostics, and probe traces must record which class was used."
   (with-temp-buffer
     (insert "alpha beta")
     (let ((events nil))
       (setq-local buffer-access-fontify-functions
                   (list (lambda (start end)
                           (push (format "%s-%s" start end) events))))
       (buffer-substring-no-properties (point-min) (point-max))
       (let ((after-no-properties (copy-sequence events)))
         (buffer-substring (point-min) (point-max))
         (soma-probe--alist
          (cons "observed"
                (format "after-no-properties=%S after-buffer-substring=%S"
                        after-no-properties (nreverse events)))
          (cons "checks"
                (list
                 (soma-probe--check "no-properties read did not invoke fontification"
                                    (null after-no-properties)
                                    (format "%S" after-no-properties))
                 (soma-probe--check "property-preserving read invoked fontification"
                                    (= (length events) 1)
                                    (format "%S" events))))))))))

(defun soma-probe-capf-exclusivity ()
  (soma-probe--case
   "capf-provider-exclusivity"
   "Completion-at-point provider exclusivity"
   "`completion-at-point-functions' providers are queried through hook wrapping; `:exclusive no' changes whether later providers are tried after a failed prefix match."
   "An exclusive first CAPF with no matching candidate should prevent the second CAPF from being queried; marking it `:exclusive no' should allow the second CAPF to answer."
   (list
    (soma-probe--source "completion-at-point-functions" "minibuffer.el.gz" "2918-2941")
    (soma-probe--source "completion--capf-wrapper" "minibuffer.el.gz" "2950-2979")
    (soma-probe--source "completion-at-point" "minibuffer.el.gz" "2986-3009"))
   "Two synthetic CAPFs over the prefix `zz': the first offers only `aa', the second offers `zz-top'."
   "Provider ordering is not a simple merge: the exclusive first provider blocks fallback, while a non-exclusive first provider permits fallback."
   "A FIM overlay that assumes all completion sources are merged can fight CAPF ownership and show stale or duplicate candidates."
   "Completion integration must record provider order, replacement range, exclusivity, and whether the model path is separate from or participates in CAPF."
   (with-temp-buffer
     (insert "zz")
     (let ((completion--capf-safe-funs nil)
           (completion--capf-misbehave-funs nil)
           exclusive-calls nonexclusive-calls exclusive-result nonexclusive-result)
       (cl-labels ((first-exclusive
                    ()
                    (push "first" exclusive-calls)
                    (list (point-min) (point-max) (list "aa")))
                   (second-exclusive
                    ()
                    (push "second" exclusive-calls)
                    (list (point-min) (point-max) (list "zz-top")))
                   (first-nonexclusive
                    ()
                    (push "first" nonexclusive-calls)
                    (list (point-min) (point-max) (list "aa") :exclusive 'no))
                   (second-nonexclusive
                    ()
                    (push "second" nonexclusive-calls)
                    (list (point-min) (point-max) (list "zz-top"))))
         (let ((completion-at-point-functions
                (list #'first-exclusive #'second-exclusive)))
           (setq exclusive-result
                 (run-hook-wrapped 'completion-at-point-functions
                                   #'completion--capf-wrapper 'all)))
         (let ((completion-at-point-functions
                (list #'first-nonexclusive #'second-nonexclusive)))
           (setq nonexclusive-result
                 (run-hook-wrapped 'completion-at-point-functions
                                   #'completion--capf-wrapper 'all))))
       (setq exclusive-calls (nreverse exclusive-calls))
       (setq nonexclusive-calls (nreverse nonexclusive-calls))
       (soma-probe--alist
        (cons "observed"
              (format "exclusive-calls=%S exclusive-result=%S nonexclusive-calls=%S nonexclusive-result=%S"
                      exclusive-calls exclusive-result
                      nonexclusive-calls nonexclusive-result))
        (cons "checks"
              (list
               (soma-probe--check "exclusive first provider blocked fallback"
                                  (equal exclusive-calls '("first"))
                                  (format "%S" exclusive-calls))
               (soma-probe--check "nonexclusive first provider allowed fallback"
                                  (equal nonexclusive-calls '("first" "second"))
                                  (format "%S" nonexclusive-calls))
               (soma-probe--check "fallback result came from second provider"
                                  (and (consp nonexclusive-result)
                                       (equal (nth 3 nonexclusive-result) '("zz-top")))
                                  (format "%S" nonexclusive-result)))))))))

(defun soma-probe-capf-exit-function-acceptance ()
  (soma-probe--case
   "capf-exit-function-acceptance"
   "Completion-at-point exit function acceptance"
   "CAPF metadata can include a provider-owned `:exit-function'; faithful acceptance must run that function, not merely produce the same buffer text."
   "Accepting a sole candidate through `completion-at-point' should insert the candidate and call `:exit-function' with the accepted string and status, while a raw text replacement can produce identical text without that side effect."
   (list
    (soma-probe--source "completion-extra-properties" "minibuffer.el.gz" "2451-2498")
    (soma-probe--source "completion-at-point" "minibuffer.el.gz" "2986-3003")
    (soma-probe--source "completion-preview-insert" "completion-preview.el.gz" "482-491"))
   "Two temporary buffers starting with `fo': one uses a synthetic CAPF for `foobar', the other deletes and inserts the same final text directly."
   "The CAPF path and raw insertion path converge on the same text, but only the CAPF path records the provider exit event."
   "A ghost-text or model-completion accept path can appear text-correct while skipping provider side effects such as imports, snippets, caches, or backend state transitions."
   "Completion acceptance evidence must record whether a provider exit function existed, whether it ran, which candidate/status it saw, and whether the edit path was CAPF, preview, or a separate model overlay."
   (let (capf-text capf-events capf-return raw-text raw-events)
     (with-temp-buffer
       (insert "fo")
       (let ((completion-at-point-functions
              (list (lambda ()
                      (list (point-min) (point-max) (list "foobar")
                            :exit-function
                            (lambda (string status)
                              (push (list string status) capf-events)))))))
         (setq capf-return (completion-at-point))
         (setq capf-text (buffer-string))))
     (setq capf-events (nreverse capf-events))
     (with-temp-buffer
       (insert "fo")
       (setq raw-events nil)
       (delete-region (point-min) (point-max))
       (insert "foobar")
       (setq raw-text (buffer-string)))
     (soma-probe--alist
      (cons "observed"
            (format "capf-return=%S capf-text=%S capf-events=%S raw-text=%S raw-events=%S"
                    capf-return capf-text capf-events raw-text raw-events))
      (cons "checks"
            (list
             (soma-probe--check "CAPF acceptance produced the expected text"
                                (string= capf-text "foobar")
                                capf-text)
             (soma-probe--check "CAPF acceptance ran provider exit function"
                                (equal capf-events '(("foobar" finished)))
                                (format "%S" capf-events))
             (soma-probe--check "raw replacement produced the same final text"
                                (string= raw-text capf-text)
                                (format "raw=%S capf=%S" raw-text capf-text))
             (soma-probe--check "raw replacement skipped provider exit function"
                                (null raw-events)
                                (format "%S" raw-events))))))))

(defun soma-probe-comint-input-boundary ()
  (soma-probe--case
   "comint-process-mark-input-boundary"
   "Comint input depends on point relative to process mark"
   "`comint-send-input' sends text from the process mark to point when point is after the mark, but retrieves old input when point is before the process mark."
   "Sending from old history should submit `comint-get-old-input' and copy it to the process mark; sending from the live input area should submit the text after the process mark."
   (list
    (soma-probe--source "comint-send-input docstring and branch" "comint.el.gz" "1886-1955")
    (soma-probe--source "process mark helpers" "comint.el.gz" "3580-3612"))
   "Two temporary Comint buffers backed by a shell process that echoes submitted lines with a `got:' prefix."
   "The same visible buffer text can produce different submissions depending on point relative to `process-mark'."
   "A bridge that appends text and sends a newline can submit the wrong command when point is in scrollback or history."
   "Comint actions must observe point, process mark, live input text, old-input policy, process identity, and working directory before submitting."
   (require 'comint)
   (cl-labels
       ((send-at
         (where)
         (let ((buf (generate-new-buffer " *soma-comint-probe*"))
               proc observed-lines)
           (unwind-protect
               (with-current-buffer buf
                 (make-comint-in-buffer
                  "soma-comint-probe" buf "sh" nil "-c"
                  "while IFS= read -r line; do printf \"got:%s\\n\" \"$line\"; done")
                 (setq proc (get-buffer-process buf))
                 (set-process-query-on-exit-flag proc nil)
                 (let ((inhibit-read-only t))
                   (erase-buffer)
                   (insert "old-cmd\n")
                   (set-marker (process-mark proc) (point-max))
                   (insert "bottom-cmd"))
                 (setq-local comint-get-old-input (lambda () "old-cmd"))
                 (setq-local comint-input-autoexpand nil)
                 (setq-local comint-input-filter (lambda (_input) nil))
                 (pcase where
                   ('old (goto-char (point-min)))
                   ('live (goto-char (point-max))))
                 (comint-send-input nil t)
                 (let ((deadline (+ (float-time) 1.0)))
                   (while (and (< (float-time) deadline)
                               (not (save-excursion
                                      (goto-char (point-min))
                                      (re-search-forward "got:" nil t))))
                     (accept-process-output proc 0.05)))
                 (save-excursion
                   (goto-char (point-min))
                   (while (re-search-forward "got:[^\n\r]*" nil t)
                     (push (match-string 0) observed-lines)))
                 (nreverse observed-lines))
             (when (process-live-p proc)
               (delete-process proc))
             (when (buffer-live-p buf)
               (kill-buffer buf))))))
     (let ((old-sent (send-at 'old))
           (live-sent (send-at 'live)))
       (soma-probe--alist
        (cons "observed"
             (format "old-point-output=%S live-input-output=%S"
                     old-sent live-sent))
        (cons "checks"
              (list
               (soma-probe--check "old history submission used comint-get-old-input"
                                  (and (member "got:old-cmd" old-sent)
                                       (not (member "got:bottom-cmd" old-sent)))
                                  (format "%S" old-sent))
               (soma-probe--check "live input submission used text after process mark"
                                  (and (member "got:bottom-cmd" live-sent)
                                       (not (member "got:old-cmd" live-sent)))
                                  (format "%S" live-sent)))))))))

(defun soma-probe-dired-undo-filesystem ()
  (soma-probe--case
   "dired-undo-does-not-restore-files"
   "Dired undo does not restore filesystem changes"
   "`dired-undo' undoes changes in the Dired buffer; it explicitly does not recover files removed from the filesystem."
   "After a real file is deleted, attempting `dired-undo' should not recreate the deleted file; any undo effect is limited to Dired-buffer state."
   (list
    (soma-probe--source "dired-undo" "dired.el.gz" "2823-2832"))
   "One temporary directory, one file, one Dired buffer mark, and one real filesystem deletion."
   "The recovery attempt is a Dired-buffer operation; the deleted file remains absent."
   "A generic recovery strategy that runs undo after a Dired action can fail to undo even buffer-local state and still leaves destructive filesystem state unrecovered."
   "Dired benchmarks must grade filesystem state independently from Dired buffer state and must classify destructive file operations as externally consequential."
   (require 'dired)
   (let* ((dir (make-temp-file "soma-dired-probe-" t))
          (file (expand-file-name "victim.txt" dir))
          dbuf undo-result file-restored mark-present-before mark-present-after)
     (unwind-protect
         (progn
           (with-temp-file file
             (insert "data\n"))
           (setq dbuf (dired-noselect dir))
           (with-current-buffer dbuf
             (goto-char (point-min))
             (dired-next-line 1)
             (dired-mark 1)
             (setq mark-present-before
                   (save-excursion
                     (goto-char (point-min))
                     (search-forward "*" nil t)))
             (delete-file file)
             (setq undo-result
                   (condition-case err
                       (progn
                         (dired-undo)
                         'returned)
                     (error (list 'error (format "%S" err)))))
             (setq mark-present-after
                   (save-excursion
                     (goto-char (point-min))
                     (search-forward "*" nil t))))
           (setq file-restored (file-exists-p file))
           (soma-probe--alist
            (cons "observed"
                  (format "mark-before=%S undo-result=%S mark-after=%S file-exists-after-dired-undo=%S"
                          mark-present-before undo-result mark-present-after
                          file-restored))
            (cons "checks"
                  (list
                   (soma-probe--check "dired mark existed before filesystem deletion"
                                      mark-present-before
                                      (format "%S" mark-present-before))
                   (soma-probe--check "dired undo attempt was buffer-local"
                                      (or (eq undo-result 'returned)
                                          (and (consp undo-result)
                                               (eq (car undo-result) 'error)))
                                      (format "%S" undo-result))
                   (soma-probe--check "deleted file was not restored"
                                      (not file-restored)
                                      file)))))
       (when (buffer-live-p dbuf)
         (kill-buffer dbuf))
       (when (file-directory-p dir)
         (delete-directory dir t))))))

(defun soma-probe-with-timeout-limits ()
  (soma-probe--case
   "with-timeout-wait-boundary"
   "with-timeout only fires at wait points"
   "`with-timeout' checks its deadline when Emacs waits for external events; CPU-bound Lisp that does not wait can run past the deadline."
   "A body that calls `sleep-for' should return the timeout form, while a short busy loop should run past the deadline and return normally."
   (list
    (soma-probe--source "with-timeout" "emacs-lisp/timer.el.gz" "478-484"))
   "Two short timed forms: one waiting with `sleep-for', one CPU-bound loop guarded by an outer wall-clock limit."
   "Timeout behavior differs at wait points versus CPU-bound work."
   "In-Emacs timeouts cannot be the only watchdog for model actions or package code that can loop without waiting."
   "The gym supervisor needs an out-of-process watchdog, and operation results must distinguish failed, canceled, and outcome_unknown."
   (let* ((wait-result
           (with-timeout (0.02 'timed-out)
             (sleep-for 0.08)
             'finished))
          (start (float-time))
          (busy-result
           (with-timeout (0.02 'timed-out)
             (let ((deadline (+ (float-time) 0.08)))
               (while (< (float-time) deadline)))
             'finished))
          (busy-elapsed (- (float-time) start)))
     (soma-probe--alist
      (cons "observed"
            (format "wait-result=%S busy-result=%S busy-elapsed=%.4f"
                    wait-result busy-result busy-elapsed))
      (cons "checks"
            (list
             (soma-probe--check "waiting form timed out"
                                (eq wait-result 'timed-out)
                                (format "%S" wait-result))
             (soma-probe--check "busy loop ran past timeout and returned normally"
                                (and (eq busy-result 'finished)
                                     (> busy-elapsed 0.05))
                                (format "result=%S elapsed=%.4f"
                                        busy-result busy-elapsed))))))))

(defun soma-probe-insula-rootfs-contract ()
  (soma-probe--case
   "insula-rootfs-dependency-closure-contract"
   "Insula rootfs dependency closure contract"
   "Hermetic Insula evidence is a rootfs-owned tool/package/dependency closure with ABI accounting, not merely an environment variable or a bwrap namespace."
   "The current development Insula should truthfully report that it uses host system binds and has no ABI guarantee; a production Soma Insula must report a selected rootfs and dependency closure."
   (list
    (soma-probe--source "Soma development Insula plan" "scripts/insula/run-emacs-internals-probes.sh" "37-66")
    (soma-probe--source "Atlas rootfs requirement" "docs/internals-atlas/README.md" "55-60")
    (soma-probe--source "Soma Insula issue" ".scratch/emacs-gym/issues/0002-soma-insula-contract-spike.md" "14-36"))
   "The emitted `insula-plan.json' from the current run."
   "Development mode is allowed only if the plan explicitly says dependency closure is not rootfs-owned and ABI guarantee is absent."
   "Treating bwrap entry as hermetic even when `/usr' and toolchains come from the host hides unaccounted ABI and package dependencies."
   "Production benchmark runs must select a Soma-owned rootfs manifest before launching Emacs, LSP servers, Jupyter kernels, Comint shells, or language toolchains."
   (let* ((plan (soma-probe--insula-plan))
          (mode (or (soma-probe--plan-get plan "mode") ""))
          (rootfs (or (soma-probe--plan-get plan "rootfs") ""))
          (closure (or (soma-probe--plan-get plan "dependency_closure") ""))
          (abi (or (soma-probe--plan-get plan "abi_guarantee") ""))
          (tools (soma-probe--plan-get plan "production_required_tools"))
          (dev (soma-probe--development-insula-p plan))
          (prod (soma-probe--production-rootfs-p plan)))
     (soma-probe--alist
      (cons "observed"
            (format "mode=%S rootfs=%S dependency_closure=%S abi_guarantee=%S production_required_tools=%S"
                    mode rootfs closure abi tools))
      (cons "checks"
            (list
             (soma-probe--check "insula plan was readable"
                                (and plan t)
                                (or (getenv "SOMA_INSULA_PLAN") ""))
             (soma-probe--check "network policy is explicit"
                                (not (string-empty-p
                                      (or (soma-probe--plan-get plan "network") "")))
                                (or (soma-probe--plan-get plan "network") ""))
             (soma-probe--check "production required tool list is recorded"
                                (and (listp tools) (>= (length tools) 8))
                                (format "%S" tools))
             (soma-probe--check "rootfs dependency closure is truthfully classified"
                                (if dev
                                    (and (string= rootfs "/")
                                         (string= closure "not_rootfs_owned")
                                         (string= abi "none"))
                                  prod)
                                (format "dev=%S prod=%S rootfs=%S closure=%S abi=%S"
                                        dev prod rootfs closure abi))))))))

(defun soma-probe-insula-plan-runtime-consistency ()
  (soma-probe--case
   "insula-plan-runtime-consistency"
   "Insula plan and runtime consistency"
   "The emitted bwrap plan must agree with the runtime state observed from inside Emacs."
   "Run id, mode, cwd, private directories, and output directory should match between environment variables, `default-directory', and `insula-plan.json'."
   (list
    (soma-probe--source "Soma development Insula plan" "scripts/insula/run-emacs-internals-probes.sh" "37-66")
    (soma-probe--source "runtime environment setup" "scripts/insula/run-emacs-internals-probes.sh" "102-119"))
   "The current `insula-plan.json' plus Emacs runtime variables."
   "The plan and runtime should match for this episode, proving plan evidence is not detached from the process being probed."
   "A runner can emit a plausible plan and then launch Emacs with different cwd, environment, run id, or artifact paths."
   "The production Insula gateway must validate plan/runtime consistency before accepting Emacs, LSP, Jupyter, or checker evidence."
   (let* ((plan (soma-probe--insula-plan))
          (env-run-id (or (getenv "SOMA_INSULA_RUN_ID") ""))
          (env-mode (or (getenv "SOMA_INSULA_MODE") ""))
          (plan-run-id (or (soma-probe--plan-get plan "run_id") ""))
          (plan-mode (or (soma-probe--plan-get plan "mode") ""))
          (repo-bind (or (soma-probe--plan-get plan "repo_bind") ""))
          (output-dir (or (soma-probe--plan-get plan "output_dir") ""))
          (cwd (directory-file-name (expand-file-name default-directory)))
          (home (or (getenv "HOME") ""))
          (tmpdir (or (getenv "TMPDIR") ""))
          (runtime (or (getenv "XDG_RUNTIME_DIR") "")))
     (soma-probe--alist
      (cons "observed"
            (format "run_id=%S/%S mode=%S/%S cwd=%S repo_bind=%S output=%S probe_output=%S"
                    env-run-id plan-run-id env-mode plan-mode cwd repo-bind
                    output-dir soma-probe-output-dir))
      (cons "checks"
            (list
             (soma-probe--check "run id matches plan"
                                (and (not (string-empty-p env-run-id))
                                     (string= env-run-id plan-run-id))
                                (format "%S / %S" env-run-id plan-run-id))
             (soma-probe--check "runtime mode matches plan family"
                                (and (not (string-empty-p env-mode))
                                     (string-match-p (regexp-quote env-mode) plan-mode))
                                (format "%S / %S" env-mode plan-mode))
             (soma-probe--check "cwd matches plan repo bind"
                                (string= cwd repo-bind)
                                (format "%S / %S" cwd repo-bind))
             (soma-probe--check "output directory matches plan"
                                (string= (directory-file-name
                                          (expand-file-name soma-probe-output-dir))
                                         (directory-file-name
                                          (expand-file-name output-dir)))
                                (format "%S / %S" soma-probe-output-dir output-dir))
             (soma-probe--check "private runtime directories match development sentinels"
                                (and (soma-probe--known-private-path-p home)
                                     (soma-probe--known-private-path-p tmpdir)
                                     (soma-probe--known-private-path-p runtime))
                                (format "HOME=%S TMPDIR=%S XDG_RUNTIME_DIR=%S"
                                        home tmpdir runtime))))))))

(defun soma-probe-rootfs-mount-boundary-evidence ()
  (soma-probe--case
   "rootfs-mount-boundary-evidence"
   "Rootfs mount boundary evidence"
   "Mount evidence must distinguish writable task/harness state, read-only configured inputs, and broad development host binds."
   "The current development Insula should expose a writable trace directory and private HOME while truthfully declaring host-system binds rather than rootfs-owned closure."
   (list
    (soma-probe--source "bwrap mount vector" "scripts/insula/run-emacs-internals-probes.sh" "64-100")
    (soma-probe--source "plan visible mount classes" "scripts/insula/run-emacs-internals-probes.sh" "56-66"))
   "The current mount plan and several runtime-visible paths."
   "Development evidence should show writable Soma trace output, readable Doom inputs, private writable HOME, and broad host `/usr' visibility classified as non-production."
   "A plan that simply says `rootfs=/` can be mistaken for a sealed dependency closure if mount classes are not inspected."
   "Production plans must record normalized mounts or a mount digest and fail if undeclared host roots provide tools or libraries."
   (let* ((plan (soma-probe--insula-plan))
          (repo-bind (or (soma-probe--plan-get plan "repo_bind") ""))
          (doom-core (or (soma-probe--plan-get plan "doom_core_readonly") ""))
          (doom-user (or (soma-probe--plan-get plan "doom_user_readonly") ""))
          (ownership (or (soma-probe--plan-get plan "rootfs_ownership") ""))
          (tmp-file (make-temp-file
                     (expand-file-name "mount-boundary-" soma-probe-output-dir))))
     (unwind-protect
         (soma-probe--alist
          (cons "observed"
                (format "repo=%S doom_core=%S doom_user=%S ownership=%S usr-bin-env=%S temp=%S"
                        repo-bind doom-core doom-user ownership
                        (file-exists-p "/usr/bin/env") tmp-file))
          (cons "checks"
                (list
                 (soma-probe--check "trace output directory is writable"
                                    (file-exists-p tmp-file)
                                    tmp-file)
                 (soma-probe--check "private HOME is writable"
                                    (file-writable-p (or (getenv "HOME") ""))
                                    (or (getenv "HOME") ""))
                 (soma-probe--check "Doom inputs are readable"
                                    (and (file-readable-p doom-core)
                                         (file-readable-p doom-user))
                                    (format "core=%S user=%S" doom-core doom-user))
                 (soma-probe--check "development host bind is explicitly classified"
                                    (or (soma-probe--production-rootfs-p plan)
                                        (string= ownership "development-host-system-binds"))
                                    (format "ownership=%S rootfs=%S"
                                            ownership
                                            (soma-probe--plan-get plan "rootfs"))))))
       (when (file-exists-p tmp-file)
         (delete-file tmp-file))))))

(defun soma-probe-toolchain-resolution-closure ()
  (soma-probe--case
   "toolchain-resolution-closure-classification"
   "Toolchain resolution and closure classification"
   "A rootfs-owned gym must account for the tools Emacs and its subprocesses invoke, including language servers, compilers, package managers, and Jupyter commands."
   "The probe should inventory tool resolution from inside Emacs and classify whether the current run is a production rootfs closure or a development non-closure."
   (list
    (soma-probe--source "Soma development Insula PATH" "scripts/insula/run-emacs-internals-probes.sh" "102-119")
    (soma-probe--source "Doom lifecycle environment identity" "docs/internals-atlas/doom-lifecycle.md" "88-103"))
   "The current Emacs process PATH under the Soma Insula wrapper."
   "Emacs should be resolvable; production mode must additionally resolve the required toolchain inside a selected rootfs."
   "A benchmark can appear to work while silently resolving `python', `clangd', `rust-analyzer', or `jupyter' from the host."
   "Observation traces must record tool resolution, and production runners must fail before an episode if required tools are absent from the selected rootfs closure."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (tools '("emacs" "python3" "jupyter" "pyright" "pyright-langserver"
                   "clangd" "rust-analyzer" "clang" "gcc" "g++" "rustc" "cargo"))
          (inventory (soma-probe--tool-inventory tools))
          (missing (soma-probe--missing-tools inventory))
          (emacs-path (alist-get "path" (car inventory) nil nil #'string=))
          (classification (if prod "production-rootfs-closure"
                            "development-not-rootfs-owned")))
     (soma-probe--alist
      (cons "observed"
            (format "classification=%s inventory=%S missing=%S"
                    classification inventory missing))
      (cons "checks"
            (list
             (soma-probe--check "tool inventory was captured"
                                (= (length inventory) (length tools))
                                (format "%S" inventory))
             (soma-probe--check "Emacs command resolves in current PATH"
                                (and emacs-path (not (string-empty-p emacs-path)))
                                (format "%S" emacs-path))
             (soma-probe--check "toolchain closure classification is explicit"
                                (or (not prod) (null missing))
                                (format "prod=%S missing=%S classification=%s"
                                        prod missing classification))))))))

(defun soma-probe-toolchain-executable-provenance ()
  (soma-probe--case
   "toolchain-executable-provenance-matrix"
   "Toolchain executable provenance matrix"
   "Production Insula evidence must account for executable identity, not just PATH lookup."
   "Every production-required executable should resolve to an absolute rootfs-owned path, carry version/hash evidence where readable, and avoid forbidden host or Nix prefixes in its ABI surface."
   (list
    (soma-probe--source "Rootfs contract tool inventory" "/etc/soma-insula-rootfs-contract.json" "tool_inventory")
    (soma-probe--source "Selected rootfs plan" "probes/emacs-internals/traces/latest/insula-plan.json" "tool_inventory"))
   "The selected Insula PATH and its production_required_tools list."
   "Production mode should show absolute rootfs-owned executable paths for Emacs, Python, Jupyter, LSP servers, C/C++ tools, and Rust tools."
   "A tool can exist in PATH but be a host bind, symlink escape, or ABI-incompatible binary."
   "The Insula gateway and benchmark traces must record executable realpaths, versions, hashes, and ABI-relevant dynamic-library surfaces for tools used by Emacs tasks."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (tools (or (soma-probe--plan-get plan "production_required_tools")
                     '("emacs" "python3" "jupyter" "pyright" "pyright-langserver"
                       "clangd" "rust-analyzer" "clang" "gcc" "g++" "rustc" "cargo")))
          (inventory (mapcar #'soma-probe--tool-provenance tools))
          (missing (soma-probe--missing-tools inventory))
          (non-rootfs
           (seq-filter
            (lambda (entry)
              (not (eq (alist-get "rootfs_owned_path" entry nil nil #'string=) t)))
            inventory))
          (serialized (format "%S" inventory))
          (forbidden (or (string-match-p "/nix/" serialized)
                         (string-match-p "/data02/home/" serialized))))
     (soma-probe--alist
      (cons "observed"
            (format "prod=%S tools=%S missing=%S non-rootfs=%S forbidden-prefix=%S"
                    prod tools missing
                    (mapcar (lambda (entry)
                              (soma-probe--alist
                               (cons "tool" (alist-get "tool" entry nil nil #'string=))
                               (cons "path" (alist-get "path" entry nil nil #'string=))
                               (cons "realpath" (alist-get "realpath" entry nil nil #'string=))))
                            non-rootfs)
                    forbidden))
      (cons "checks"
            (list
             (soma-probe--check "provenance was captured for every required tool"
                                (= (length inventory) (length tools))
                                (format "%d/%d" (length inventory) (length tools)))
             (soma-probe--check "production required tools are present"
                                (or (not prod) (null missing))
                                (format "prod=%S missing=%S" prod missing))
             (soma-probe--check "production tools resolve to rootfs-owned executable paths"
                                (or (not prod) (null non-rootfs))
                                (format "%S" non-rootfs))
             (soma-probe--check "provenance has no forbidden host or Nix prefixes"
                                (or (not prod) (not forbidden))
                                (substring serialized 0 (min 1000 (length serialized))))))))))

(defun soma-probe-toolchain-smoke-build-rootfs-identity ()
  (soma-probe--case
   "toolchain-smoke-build-rootfs-identity"
   "Toolchain smoke build rootfs identity"
   "A language toolchain is not proven by path lookup; it must compile or execute inside the selected rootfs with the expected runtime identity."
   "Python should report rootfs-owned executable/prefix/sysconfig data, C/C++ should compile and run a tiny binary, and Rust should compile and run a tiny binary in production mode."
   (list
    (soma-probe--source "Rootfs build package layer" "scripts/insula/build-emacs-rootfs.sh" "129-144")
    (soma-probe--source "Rootfs runtime PATH" "scripts/insula/run-emacs-internals-probes-rootfs.sh" "202-218"))
   "A temporary directory under the private TMPDIR, one tiny C program, one tiny Rust program, and Python sysconfig introspection."
   "Production mode should successfully execute Python, compile C with a rootfs compiler, and compile Rust with the rootfs Rust toolchain."
   "A compiler command may resolve but fail because its sysroot, linker, headers, loader, or runtime libraries are missing from the rootfs closure."
   "Language-source-code benchmarks must smoke-test the relevant compiler/interpreter closure inside the Insula before trusting Emacs+LSP workflow evidence."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (tmp (make-temp-file "soma-toolchain-smoke-" t))
          (cc (or (executable-find "clang") (executable-find "gcc")))
          (rustc (executable-find "rustc"))
          python c-result rust-result)
     (unwind-protect
         (progn
           (setq python
                 (soma-probe--python-script-output
                  (concat
                   "import json, os, platform, sys, sysconfig\n"
                   "print(json.dumps({"
                   "'executable': sys.executable,"
                   "'prefix': sys.prefix,"
                   "'base_prefix': sys.base_prefix,"
                   "'platform': platform.platform(),"
                   "'paths': sysconfig.get_paths(),"
                   "'run_id': os.environ.get('SOMA_INSULA_RUN_ID','')"
                   "}, sort_keys=True))\n")
                  10))
           (when cc
             (with-temp-file (expand-file-name "main.c" tmp)
               (insert "#include <stdio.h>\nint main(void){puts(\"soma-c-ok\");return 0;}\n"))
             (let ((default-directory (file-name-as-directory tmp)))
               (setq c-result
                     (soma-probe--shell-output-timeout
                      20 "sh" "-lc"
                      (format "%s main.c -o main-c && ./main-c && ldd ./main-c"
                              (shell-quote-argument cc))))))
           (when rustc
             (with-temp-file (expand-file-name "main.rs" tmp)
               (insert "fn main(){println!(\"soma-rust-ok\");}\n"))
             (let ((default-directory (file-name-as-directory tmp)))
               (setq rust-result
                     (soma-probe--shell-output-timeout
                      30 "sh" "-lc"
                      (format "%s main.rs -o main-rust && ./main-rust && ldd ./main-rust"
                              (shell-quote-argument rustc))))))
           (let* ((python-status (alist-get "status" python nil nil #'string=))
                  (python-output (or (alist-get "output" python nil nil #'string=) ""))
                  (c-status (and c-result
                                 (alist-get "status" c-result nil nil #'string=)))
                  (c-output (or (and c-result
                                     (alist-get "output" c-result nil nil #'string=))
                                ""))
                  (rust-status (and rust-result
                                    (alist-get "status" rust-result nil nil #'string=)))
                  (rust-output (or (and rust-result
                                        (alist-get "output" rust-result nil nil #'string=))
                                   ""))
                  (forbidden (or (string-match-p "/nix/" (concat python-output c-output rust-output))
                                 (string-match-p "/data02/home/" (concat python-output c-output rust-output)))))
             (soma-probe--alist
              (cons "observed"
                    (format "prod=%S tmp=%S python-status=%S cc=%S c-status=%S rustc=%S rust-status=%S forbidden=%S"
                            prod tmp python-status cc c-status rustc rust-status forbidden))
              (cons "checks"
                    (list
                     (soma-probe--check "Python introspection ran"
                                        (equal python-status 0)
                                        python-output)
                     (soma-probe--check "production C toolchain compiled and ran"
                                        (or (not prod)
                                            (and cc (equal c-status 0)
                                                 (string-match-p "soma-c-ok" c-output)))
                                        (format "cc=%S status=%S output=%S" cc c-status c-output))
                     (soma-probe--check "production Rust toolchain compiled and ran"
                                        (or (not prod)
                                            (and rustc (equal rust-status 0)
                                                 (string-match-p "soma-rust-ok" rust-output)))
                                        (format "rustc=%S status=%S output=%S"
                                                rustc rust-status rust-output))
                     (soma-probe--check "smoke-build ABI output has no forbidden prefixes"
                                        (or (not prod) (not forbidden))
                                        (substring (concat python-output c-output rust-output)
                                                   0 (min 1500 (length (concat python-output c-output rust-output))))))))))
       (when (file-directory-p tmp)
         (delete-directory tmp t))))))

(defun soma-probe-subprocess-environment-closure ()
  (soma-probe--case
   "subprocess-environment-closure"
   "Subprocess environment closure"
   "Emacs subprocesses must inherit the same controlled environment contract as the Emacs process."
   "A shell subprocess should see the Insula allowlist variables, private directories, and no host package-manager leakage such as conda, mamba, `PYTHONPATH', or ambient `LD_LIBRARY_PATH'."
   (list
    (soma-probe--source "environment clear and setenv" "scripts/insula/run-emacs-internals-probes.sh" "102-119")
    (soma-probe--source "Comint subprocess contract" "docs/internals-atlas/package-contracts/comint.md" "18-27"))
   "One `env' subprocess launched from Emacs."
   "The subprocess environment should stay within the declared allowlist for the current development Insula."
   "A clean Emacs process can still launch subprocesses with leaked host variables if the process environment is mutated later."
   "Every benchmark subprocess, including LSP servers, Jupyter kernels, Comint shells, and checkers, must prove environment closure before task output is trusted."
   (let* ((result (soma-probe--shell-output "env"))
          (status (alist-get "status" result nil nil #'string=))
          (output (or (alist-get "output" result nil nil #'string=) ""))
          (env-names (soma-probe--env-names-from-lines output))
          (allowed '("HOME" "LANG" "LC_ALL" "PATH" "PWD" "SHELL"
                     "SHLVL" "_"
                     "CARGO_HOME" "RUSTUP_HOME"
                     "SOMA_INSULA_ACTIVE" "SOMA_INSULA_MODE"
                     "SOMA_INSULA_PLAN" "SOMA_INSULA_RUN_ID"
                     "TMPDIR" "XDG_CACHE_HOME" "XDG_CONFIG_HOME"
                     "XDG_DATA_HOME" "XDG_RUNTIME_DIR"))
          (unexpected (seq-remove (lambda (name) (member name allowed)) env-names))
          (forbidden (seq-filter
                      (lambda (name)
                        (member name '("PYTHONPATH" "LD_LIBRARY_PATH"
                                       "CONDA_PREFIX" "MAMBA_ROOT_PREFIX"
                                       "VIRTUAL_ENV")))
                      env-names)))
     (soma-probe--alist
      (cons "observed"
            (format "status=%S env_names=%S unexpected=%S forbidden=%S"
                    status env-names unexpected forbidden))
      (cons "checks"
            (list
             (soma-probe--check "env subprocess ran"
                                (equal status 0)
                                (format "%S" status))
             (soma-probe--check "subprocess environment is allowlisted"
                                (null unexpected)
                                (format "%S" unexpected))
             (soma-probe--check "subprocess has no host package-manager leakage"
                                (null forbidden)
                                (format "%S" forbidden))
             (soma-probe--check "subprocess sees the same Insula run id"
                                (and (not (string-empty-p
                                           (or (getenv "SOMA_INSULA_RUN_ID") "")))
                                     (string-match-p
                                      (regexp-quote (getenv "SOMA_INSULA_RUN_ID"))
                                      output))
                                (or (getenv "SOMA_INSULA_RUN_ID") ""))))))))

(defun soma-probe-native-comp-cache-closure ()
  (soma-probe--case
   "native-comp-cache-closure"
   "Native-comp cache closure"
   "Emacs native compilation introduces ABI-sensitive cache and runtime-library state that must be owned by the rootfs or sandbox cache."
   "The probe should record native-comp availability, eln load paths, and whether cache-like paths are sandbox-owned or development-classified."
   (list
    (soma-probe--source "Emacs native-comp feature" "system-configuration-features" "runtime")
    (soma-probe--source "Doom lifecycle environment identity" "docs/internals-atlas/doom-lifecycle.md" "88-103"))
   "The current Emacs runtime variables related to native compilation."
   "Development mode may include host Emacs native-lisp paths, but production benchmark evidence needs rootfs-owned ABI and cache manifests."
   "Ignoring native-comp paths can hide stale `.eln' artifacts compiled against a different Emacs or libgccjit ABI."
   "The rootfs contract must record native-comp enablement, libgccjit identity, eln paths, and sandbox-owned cache/write locations."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (feature (string-match-p "NATIVE_COMP" system-configuration-features))
          (eln-paths (and (boundp 'native-comp-eln-load-path)
                          (symbol-value 'native-comp-eln-load-path)))
          (cache-like (seq-filter
                       (lambda (path)
                         (and (stringp path)
                              (or (string-match-p "cache\\|eln" path)
                                  (soma-probe--known-private-path-p path))))
                       (or eln-paths nil)))
          (nonprivate-cache
           (seq-remove #'soma-probe--known-private-path-p cache-like)))
     (soma-probe--alist
      (cons "observed"
            (format "native-comp-feature=%S eln-paths=%S cache-like=%S nonprivate-cache=%S prod=%S"
                    feature eln-paths cache-like nonprivate-cache prod))
      (cons "checks"
            (list
             (soma-probe--check "native-comp availability was captured"
                                t
                                (format "%S" feature))
             (soma-probe--check "eln path state was captured"
                                (or (not feature) (listp eln-paths))
                                (format "%S" eln-paths))
             (soma-probe--check "native-comp nonprivate paths are classified"
                                (or (not prod) (null nonprivate-cache))
                                (format "prod=%S nonprivate-cache=%S"
                                        prod nonprivate-cache))))))))

(defun soma-probe-abi-surface-manifest ()
  (soma-probe--case
   "abi-surface-requires-rootfs-manifest"
   "ABI surface requires a rootfs manifest"
   "ABI guarantees come from a selected rootfs manifest and accounted shared-library closure, not from successfully starting Emacs once."
   "The current development Insula may inventory the Emacs dynamic-library surface, but it should not claim an ABI guarantee without a rootfs manifest."
   (list
    (soma-probe--source "Emacs identity fields" "probes/emacs-internals/elisp/run-probes.el" "24-42")
    (soma-probe--source "Soma development Insula ABI field" "scripts/insula/run-emacs-internals-probes.sh" "37-66"))
   "The running Emacs binary plus the emitted Insula plan."
   "Development mode reports `abi_guarantee=none'; production mode must report a selected rootfs or ABI manifest."
   "Recording only `emacs-version' misses glibc, libstdc++, libgccjit, tree-sitter, module, and dynamic-loader compatibility."
   "The Soma Insula contract must make ABI-relevant library/toolchain identity part of rootfs materialization and benchmark evidence."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (emacs-path (soma-probe--real-emacs-path))
          (ldd (and (executable-find "ldd")
                    (soma-probe--shell-output "ldd" emacs-path)))
          (ldd-output (or (and ldd (alist-get "output" ldd nil nil #'string=)) ""))
          (abi (or (soma-probe--plan-get plan "abi_guarantee") ""))
          (glibc-owner (or (soma-probe--plan-get plan "glibc_owner") ""))
          (nix-allowed (soma-probe--plan-get plan "nix_allowed"))
          (nix-refs (string-match-p "/nix/" ldd-output))
          (manifest (or (soma-probe--plan-get plan "abi_manifest")
                        (soma-probe--plan-get plan "selected_rootfs_manifest")
                        (soma-probe--plan-get plan "rootfs_manifest")
                        "")))
     (soma-probe--alist
      (cons "observed"
            (format "emacs=%S ldd-lines=%d abi_guarantee=%S manifest=%S glibc-owner=%S nix-refs=%S"
                    emacs-path (length (split-string ldd-output "\n" t))
                    abi manifest glibc-owner nix-refs))
      (cons "checks"
            (list
             (soma-probe--check "Emacs executable path exists"
                                (file-exists-p emacs-path)
                                emacs-path)
             (soma-probe--check "ABI inventory captured or manifest supplied"
                                (or (not (string-empty-p ldd-output))
                                    (not (string-empty-p manifest)))
                                (if (string-empty-p ldd-output)
                                    manifest
                                  (substring ldd-output 0 (min 500 (length ldd-output)))))
             (soma-probe--check "ABI guarantee requires production manifest"
                                (if prod
                                    (not (string-empty-p manifest))
                                  (string= abi "none"))
                                (format "prod=%S abi=%S manifest=%S"
                                        prod abi manifest))
             (soma-probe--check "production glibc comes from container rootfs"
                                (or (not prod)
                                    (string= glibc-owner "container-image-rootfs"))
                                (format "prod=%S glibc-owner=%S"
                                        prod glibc-owner))
             (soma-probe--check "production ABI surface has no Nix paths"
                                (or (not prod)
                                    (eq nix-allowed t)
                                    (not nix-refs))
                                (format "prod=%S nix-allowed=%S ldd=%S"
                                        prod nix-allowed
                                        (substring ldd-output 0
                                                   (min 500 (length ldd-output)))))))))))

(defun soma-probe-emacs-patch-provenance ()
  (soma-probe--case
   "emacs-patch-provenance-contract"
   "Emacs patch provenance contract"
   "A patched Emacs build is a distinct runtime substrate whose source, patches, build flags, executable identity, and ABI surface must be recorded before its observability evidence is trusted."
   "The current development Insula should explicitly classify Emacs as stock. Any non-stock variant must carry patch and stock-comparison provenance in the plan or rootfs manifest."
   (list
    (soma-probe--source "Emacs patch discipline" "AGENTS.md" "36-50")
    (soma-probe--source "Atlas patched-Emacs rule" "docs/internals-atlas/README.md" "69-75")
    (soma-probe--source "RFC Emacs adaptation decision" "docs/rfcs/0002-doom-emacs-gym-insula.md" "755-762"))
   "The running Emacs binary and emitted `insula-plan.json'."
   "Stock Emacs passes with an explicit stock variant and no patch set; patched variants require source-pinned provenance and stock-vs-patched evidence."
   "Using a locally patched Emacs binary because it has better traces can silently change the command loop, buffer machinery, redisplay, native-comp behavior, or ABI surface being benchmarked."
   "The Insula contract must treat patched Emacs as a named runtime variant and reject patched evidence that lacks source, patch, build, ABI, and stock-comparison provenance."
   (let* ((plan (soma-probe--insula-plan))
          (variant (or (soma-probe--plan-get plan "emacs_patch_variant") "stock"))
          (stock (string= variant "stock"))
          (patch-set (soma-probe--plan-get plan "emacs_patch_set"))
          (source-revision (or (soma-probe--plan-get plan "emacs_source_revision") ""))
          (configure-flags (or (soma-probe--plan-get plan "emacs_configure_flags") ""))
          (executable-hash (or (soma-probe--plan-get plan "emacs_executable_hash") ""))
          (abi-manifest (or (soma-probe--plan-get plan "abi_manifest") ""))
          (stock-comparison (or (soma-probe--plan-get plan "emacs_stock_comparison") ""))
          (emacs-path (soma-probe--real-emacs-path))
          (patched-fields-present
           (and (not (string-empty-p source-revision))
                (listp patch-set)
                patch-set
                (not (string-empty-p configure-flags))
                (not (string-empty-p executable-hash))
                (not (string-empty-p abi-manifest))
                (not (string-empty-p stock-comparison)))))
     (soma-probe--alist
      (cons "observed"
            (format "variant=%S patch-set=%S source-revision=%S configure-flags=%S executable-hash=%S abi-manifest=%S stock-comparison=%S emacs=%S"
                    variant patch-set source-revision configure-flags
                    executable-hash abi-manifest stock-comparison emacs-path))
      (cons "checks"
            (list
             (soma-probe--check "emacs patch variant is explicit"
                                (not (string-empty-p variant))
                                variant)
             (soma-probe--check "stock variant has no patch set"
                                (or (not stock) (null patch-set))
                                (format "stock=%S patch-set=%S" stock patch-set))
             (soma-probe--check "patched variant carries source, build, ABI, and comparison provenance"
                                (or stock patched-fields-present)
                                (format "stock=%S patched-fields-present=%S" stock patched-fields-present))
             (soma-probe--check "Emacs executable path exists"
                                (file-exists-p emacs-path)
                                emacs-path)))))))

(defun soma-probe-jupyter-kernel-rootfs-proxy ()
  (soma-probe--case
   "jupyter-kernel-rootfs-identity-proxy"
   "Jupyter kernel rootfs identity proxy"
   "A Jupyter benchmark is hermetic only if the kernel process and Python environment prove the same rootfs contract as Emacs."
   "A Python subprocess launched from inside Emacs should report cwd, executable, prefix, PATH, run id, and plan path; production mode must also have a Jupyter command in the rootfs closure."
   (list
    (soma-probe--source "Jupyter package contract" "docs/internals-atlas/package-contracts/jupyter.md" "27-38, 64-74")
    (soma-probe--source "Existing-kernel benchmark issue" ".scratch/emacs-gym/issues/0006-existing-kernel-jupyter-benchmark.md" "14-40"))
   "A Python subprocess used as a kernel-side identity proxy; no real notebook kernel is launched by this batch-safe probe."
   "The proxy can record Python identity and stable run id, while production Jupyter benchmarks must additionally launch or attach a real kernel through the rootfs gateway."
   "Checking only the Emacs process can miss a host Python kernel with the right printed result."
   "Jupyter actions must verify kernel-side rootfs manifest, Python executable/prefix, cwd, import path, run id, and bwrap plan before grading execution."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (python (or (executable-find "python3") (executable-find "python")))
          (jupyter (executable-find "jupyter"))
          (run-id (or (getenv "SOMA_INSULA_RUN_ID") ""))
          (script (concat
                   "import json, os, sys\n"
                   "print(json.dumps({"
                   "'executable': sys.executable,"
                   "'prefix': sys.prefix,"
                   "'cwd': os.getcwd(),"
                   "'path': os.environ.get('PATH',''),"
                   "'run_id': os.environ.get('SOMA_INSULA_RUN_ID',''),"
                   "'plan': os.environ.get('SOMA_INSULA_PLAN','')"
                   "}, sort_keys=True))\n"))
          (py-result (and python
                          (soma-probe--shell-output python "-c" script)))
          (py-status (and py-result (alist-get "status" py-result nil nil #'string=)))
          (py-output (or (and py-result
                              (alist-get "output" py-result nil nil #'string=))
                         "")))
     (soma-probe--alist
      (cons "observed"
            (format "python=%S jupyter=%S python-status=%S python-output=%S"
                    python jupyter py-status py-output))
      (cons "checks"
            (list
             (soma-probe--check "Python identity proxy ran or production is not claimed"
                                (or (not prod) (and python (equal py-status 0)))
                                (format "prod=%S python=%S status=%S"
                                        prod python py-status))
             (soma-probe--check "stable run id reaches Python when Python is available"
                                (or (not python)
                                    (and (not (string-empty-p run-id))
                                         (string-match-p (regexp-quote run-id) py-output)))
                                (format "run-id=%S output=%S" run-id py-output))
             (soma-probe--check "Jupyter command is required only for production kernel benchmark evidence"
                                (or (not prod) jupyter)
                                (format "prod=%S jupyter=%S" prod jupyter))))))))

(defun soma-probe-jupyter-kernelspec-rootfs-contract ()
  (soma-probe--case
   "jupyter-kernelspec-rootfs-contract"
   "Jupyter kernelspec rootfs contract"
   "Jupyter identity starts before execution: kernelspec argv and resource paths decide which interpreter/kernel will actually start."
   "`jupyter_client.kernelspec.KernelSpecManager' should expose at least one Python kernelspec in production, and its absolute argv/resource paths should resolve inside the selected rootfs rather than through host state."
   (list
    (soma-probe--source "TorchTitan Jupyter rootfs lifecycle" "/private-source/home/operator/workspace/torchtitan/experiments/jupyter_rootfs/run_common.sh" "5-47")
    (soma-probe--source "Jupyter package contract" "docs/internals-atlas/package-contracts/jupyter.md" "27-38, 64-74"))
   "The rootfs-owned Python `jupyter_client' kernelspec manager."
   "Production mode should contain a usable Python kernelspec whose argv points at the rootfs Python interpreter or a resolvable rootfs-owned command."
   "A notebook can print the expected result while attached to a host kernelspec with a different Python, package set, CUDA stack, or filesystem view."
   "Jupyter benchmarks must grade kernelspec identity before kernel execution and reject host-resolved kernel launchers."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (python (executable-find "python3"))
          (jupyter (executable-find "jupyter"))
          (script
           (concat
            "import json, shutil\n"
            "from jupyter_client.kernelspec import KernelSpecManager\n"
            "ksm = KernelSpecManager()\n"
            "out = {}\n"
            "for name, resource_dir in ksm.find_kernel_specs().items():\n"
            "    spec = ksm.get_kernel_spec(name)\n"
            "    out[name] = {'resource_dir': resource_dir, 'spec': {'argv': spec.argv, 'display_name': spec.display_name, 'language': spec.language}}\n"
            "print(json.dumps({'kernelspecs': out, 'jupyter': shutil.which('jupyter') or ''}, sort_keys=True))\n"))
          (result (and python
                       (soma-probe--python-script-output script 15)))
          (status (and result (alist-get "status" result nil nil #'string=)))
          (output (or (and result (alist-get "output" result nil nil #'string=)) ""))
          (parsed (and (equal status 0)
                       (condition-case nil
                           (soma-probe--read-json-string output)
                         (error nil))))
          (kernelspecs (and parsed
                            (soma-probe--plan-get parsed "kernelspecs")))
          (python-specs nil)
          (bad-paths nil))
     (dolist (entry kernelspecs)
       (let* ((name (car entry))
              (body (cdr entry))
              (spec (soma-probe--plan-get body "spec"))
              (language (or (soma-probe--plan-get spec "language") ""))
              (argv (or (soma-probe--plan-get spec "argv") nil))
              (resource-dir (or (soma-probe--plan-get body "resource_dir") "")))
         (when (or (string= language "python") (string= name "python3"))
           (push (soma-probe--alist
                  (cons "name" name)
                  (cons "resource_dir" resource-dir)
                  (cons "argv" argv))
                 python-specs))
         (when (and (stringp resource-dir)
                    (not (string-empty-p resource-dir))
                    (soma-probe--forbidden-rootfs-path-p resource-dir))
           (push resource-dir bad-paths))
         (dolist (arg argv)
           (when (and (stringp arg)
                      (file-name-absolute-p arg)
                      (soma-probe--forbidden-rootfs-path-p arg))
             (push arg bad-paths)))))
     (let* ((selected (car python-specs))
            (argv (and selected (alist-get "argv" selected nil nil #'string=)))
            (launcher (car argv))
            (launcher-path (cond
                            ((and (stringp launcher) (file-name-absolute-p launcher))
                             launcher)
                            ((stringp launcher)
                             (or (executable-find launcher) ""))
                            (t "")))
            (launcher-owned (soma-probe--rootfs-owned-executable-path-p
                             (or launcher-path ""))))
       (soma-probe--alist
        (cons "observed"
              (format "prod=%S python=%S jupyter=%S status=%S python-specs=%S selected-launcher=%S launcher-path=%S bad-paths=%S"
                      prod python jupyter status python-specs launcher launcher-path bad-paths))
        (cons "checks"
              (list
               (soma-probe--check "kernelspec manager ran in production"
                                  (or (not prod) (and python (equal status 0) parsed))
                                  (format "python=%S jupyter=%S status=%S output=%S"
                                          python jupyter status output))
               (soma-probe--check "production Python kernelspec exists"
                                  (or (not prod) python-specs)
                                  (format "%S" python-specs))
               (soma-probe--check "production kernelspec launcher is rootfs-owned"
                                  (or (not prod) launcher-owned)
                                  (format "launcher=%S launcher-path=%S" launcher launcher-path))
               (soma-probe--check "kernelspec paths have no forbidden host prefixes"
                                  (or (not prod) (null bad-paths))
                                  (format "%S" bad-paths)))))))))

(defun soma-probe-jupyter-real-kernel-rootfs-identity ()
  (soma-probe--case
   "jupyter-real-kernel-rootfs-identity"
   "Jupyter real kernel rootfs identity"
   "A Jupyter benchmark must prove the actual kernel process that executed a request, not only the Emacs process or `jupyter' CLI."
   "A temporary Python kernel launched through `jupyter_client' should execute one request, return a matching shell reply and IOPub idle state, and print rootfs identity from inside the kernel."
   (list
    (soma-probe--source "TorchTitan Emacs/Jupyter verifier" "/private-source/home/operator/workspace/torchtitan/experiments/jupyter_rootfs/verify_attach.el" "96-133")
    (soma-probe--source "Jupyter package contract" "docs/internals-atlas/package-contracts/jupyter.md" "27-38, 64-74"))
   "A temporary `jupyter_client.KernelManager' Python kernel launched from the selected rootfs."
   "Production mode should execute code in a real kernel with matching request ids, rootfs-owned Python executable, stable Insula run id, and idle completion evidence."
   "A checker that accepts the right printed value can miss execution on the wrong kernel, duplicate retries, or an unknown request outcome."
   "Jupyter actions must track kernel incarnation, request id, shell reply, parent-header outputs, idle/completion state, and kernel-side rootfs identity."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (run-id (or (getenv "SOMA_INSULA_RUN_ID") ""))
          (script
           (concat
            "import gc, json, os, pathlib, sys, time, traceback\n"
            "from jupyter_client import KernelManager\n"
            "km = KernelManager(kernel_name='python3')\n"
            "kc = None\n"
            "result = {'ok': False}\n"
            "try:\n"
            "    km.start_kernel(cwd=os.getcwd(), env=os.environ.copy())\n"
            "    kc = km.client()\n"
            "    kc.start_channels()\n"
            "    kc.wait_for_ready(timeout=20)\n"
            "    code = \"import json, os, sys\\nprint('SOMA_KERNEL_JSON:' + json.dumps({'executable': sys.executable, 'prefix': sys.prefix, 'cwd': os.getcwd(), 'run_id': os.environ.get('SOMA_INSULA_RUN_ID',''), 'plan': os.environ.get('SOMA_INSULA_PLAN',''), 'path0': sys.path[:5]}, sort_keys=True))\"\n"
            "    msg_id = kc.execute(code)\n"
            "    payload = None\n"
            "    idle = False\n"
            "    errors = []\n"
            "    deadline = time.time() + 25\n"
            "    while time.time() < deadline and not (payload is not None and idle):\n"
            "        try:\n"
            "            msg = kc.get_iopub_msg(timeout=1)\n"
            "        except Exception:\n"
            "            continue\n"
            "        if msg.get('parent_header', {}).get('msg_id') != msg_id:\n"
            "            continue\n"
            "        msg_type = msg.get('header', {}).get('msg_type')\n"
            "        content = msg.get('content', {})\n"
            "        if msg_type == 'stream':\n"
            "            for line in content.get('text', '').splitlines():\n"
            "                if line.startswith('SOMA_KERNEL_JSON:'):\n"
            "                    payload = json.loads(line[len('SOMA_KERNEL_JSON:'):])\n"
            "        elif msg_type == 'error':\n"
            "            errors.append(content)\n"
            "        elif msg_type == 'status' and content.get('execution_state') == 'idle':\n"
            "            idle = True\n"
            "    reply = None\n"
            "    deadline = time.time() + 15\n"
            "    while time.time() < deadline:\n"
            "        candidate = kc.get_shell_msg(timeout=max(0.1, deadline - time.time()))\n"
            "        if candidate.get('parent_header', {}).get('msg_id') == msg_id:\n"
            "            reply = candidate\n"
            "            break\n"
            "    provisioner = getattr(km, 'provisioner', None)\n"
            "    kernel_pid = getattr(provisioner, 'pid', None)\n"
            "    result = {\n"
            "        'ok': bool(payload and idle and reply and reply.get('content', {}).get('status') == 'ok'),\n"
            "        'msg_id': msg_id,\n"
            "        'reply_parent': reply.get('parent_header', {}).get('msg_id') if reply else '',\n"
            "        'reply_status': reply.get('content', {}).get('status') if reply else '',\n"
            "        'idle': idle,\n"
            "        'payload': payload,\n"
            "        'errors': errors,\n"
            "        'kernel_pid': kernel_pid,\n"
            "        'connection_file': getattr(km, 'connection_file', ''),\n"
            "    }\n"
            "except Exception as exc:\n"
            "    result = {'ok': False, 'error': repr(exc), 'traceback': traceback.format_exc()[-3000:]}\n"
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
            "    gc.collect()\n"
            "print('SOMA_JUPYTER_PROBE:' + json.dumps(result, sort_keys=True))\n"))
          (result (soma-probe--python-script-output script 60))
          (status (alist-get "status" result nil nil #'string=))
          (output (or (alist-get "output" result nil nil #'string=) ""))
          (json-line (soma-probe--json-line-with-prefix "SOMA_JUPYTER_PROBE:" output))
          (parsed (and json-line
                       (condition-case nil
                           (soma-probe--read-json-string json-line)
                         (error nil))))
          (ok (eq (soma-probe--plan-get parsed "ok") t))
          (reply-parent (or (soma-probe--plan-get parsed "reply_parent") ""))
          (msg-id (or (soma-probe--plan-get parsed "msg_id") ""))
          (payload (soma-probe--plan-get parsed "payload"))
          (kernel-run-id (or (and payload
                                  (soma-probe--plan-get payload "run_id"))
                             ""))
          (kernel-exe (or (and payload
                               (soma-probe--plan-get payload "executable"))
                          ""))
          (kernel-cwd (or (and payload
                               (soma-probe--plan-get payload "cwd"))
                          ""))
          (kernel-plan (or (and payload
                                (soma-probe--plan-get payload "plan"))
                           "")))
     (soma-probe--alist
      (cons "observed"
            (format "prod=%S status=%S ok=%S msg-id=%S reply-parent=%S kernel-exe=%S kernel-cwd=%S kernel-run-id=%S output=%S"
                    prod status ok msg-id reply-parent kernel-exe kernel-cwd
                    kernel-run-id
                    (substring output 0 (min 2000 (length output)))))
      (cons "checks"
            (list
             (soma-probe--check "real Jupyter kernel probe ran in production"
                                (or (not prod) (and (equal status 0) parsed))
                                (format "status=%S output=%S" status output))
             (soma-probe--check "kernel request reached ok reply and idle"
                                (or (not prod)
                                    (and ok (string= msg-id reply-parent)))
                                (format "ok=%S msg-id=%S reply-parent=%S parsed=%S"
                                        ok msg-id reply-parent parsed))
             (soma-probe--check "kernel sees stable Insula run id"
                                (or (not prod)
                                    (and (not (string-empty-p run-id))
                                         (string= run-id kernel-run-id)))
                                (format "run-id=%S kernel-run-id=%S"
                                        run-id kernel-run-id))
             (soma-probe--check "kernel executable and cwd are rootfs/runtime owned"
                                (or (not prod)
                                    (and (soma-probe--rootfs-owned-executable-path-p kernel-exe)
                                         (string= kernel-cwd "/workspace/soma")
                                         (string-prefix-p "/workspace/soma/" kernel-plan)))
                                (format "exe=%S cwd=%S plan=%S"
                                        kernel-exe kernel-cwd kernel-plan))))))))

(defun soma-probe-lsp-server-rootfs-resolution ()
  (soma-probe--case
   "lsp-server-rootfs-resolution-contract"
   "LSP server rootfs resolution contract"
   "A real Emacs+LSP benchmark depends on both the Emacs client and the language-server process resolving from the same accounted rootfs closure."
   "The probe should inventory likely Python, C/C++, and Rust language-server commands and classify missing or host-resolved servers as a development gap unless a production rootfs is selected."
   (list
    (soma-probe--source "LSP package contract" "docs/internals-atlas/package-contracts/lsp.md" "18-31")
    (soma-probe--source "Doom language modules" "docs/internals-atlas/doom-lifecycle.md" "35-39"))
   "Executable lookup for common LSP server commands in the current Insula PATH."
   "Production mode requires Python LSP, C/C++ LSP, and Rust LSP server commands in the selected rootfs closure."
   "A direct text edit can pass a filesystem checker while the intended LSP server never started or came from the host."
   "LSP tasks must prove server process identity, workspace root, synchronized document version, and rootfs-owned server/toolchain resolution."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (inventory (soma-probe--tool-inventory
                      '("pyright-langserver" "pyright" "basedpyright-langserver"
                        "pylsp" "clangd" "rust-analyzer")))
          (python-lsp (cl-some (lambda (entry)
                                 (and (member (alist-get "tool" entry nil nil #'string=)
                                              '("pyright-langserver" "pyright"
                                                "basedpyright-langserver" "pylsp"))
                                      (eq (alist-get "found" entry nil nil #'string=) t)))
                               inventory))
          (clangd (executable-find "clangd"))
          (rust-analyzer (executable-find "rust-analyzer")))
     (soma-probe--alist
      (cons "observed"
            (format "prod=%S inventory=%S python-lsp=%S clangd=%S rust-analyzer=%S"
                    prod inventory python-lsp clangd rust-analyzer))
      (cons "checks"
            (list
             (soma-probe--check "LSP server inventory was captured"
                                (= (length inventory) 6)
                                (format "%S" inventory))
             (soma-probe--check "production Python LSP server requirement is classified"
                                (or (not prod) python-lsp)
                                (format "prod=%S python-lsp=%S" prod python-lsp))
             (soma-probe--check "production C/C++ and Rust LSP requirements are classified"
                                (or (not prod) (and clangd rust-analyzer))
                                (format "prod=%S clangd=%S rust-analyzer=%S"
                                        prod clangd rust-analyzer))))))))

(defun soma-probe-lsp-stdio-initialize-rootfs-identity ()
  (soma-probe--case
   "lsp-stdio-initialize-rootfs-identity"
   "LSP stdio initialize rootfs identity"
   "LSP task evidence must include the language-server process, protocol lifecycle, workspace root, and rootfs-owned executable path."
   "Batch-safe JSON-RPC initialize/shutdown exchanges should succeed for the production C/C++, Python, and Rust language servers that are present in the selected rootfs."
   (list
    (soma-probe--source "LSP package contract" "docs/internals-atlas/package-contracts/lsp.md" "57-76")
    (soma-probe--source "Rootfs build package layer" "scripts/insula/build-emacs-rootfs.sh" "129-144"))
   "Temporary C++, Python, and Rust workspaces plus stdio JSON-RPC exchanges with clangd, pyright-langserver, and rust-analyzer."
   "Each production server should answer `initialize', accept `shutdown', receive `exit', and expose a rootfs-owned executable path."
   "A benchmark can pass via direct text mutation while the intended language server never starts, starts from the host, or cannot initialize against the workspace."
   "LSP benchmarks must record server executable identity, pid, workspace root, initialize response, shutdown status, and document-version evidence before trusting refactor results."
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (script
           (concat
            "import json, os, pathlib, select, shutil, subprocess, sys, tempfile, time\n"
            "def send(proc, msg):\n"
            "    body = json.dumps(msg, separators=(',', ':')).encode('utf-8')\n"
            "    proc.stdin.write(b'Content-Length: ' + str(len(body)).encode('ascii') + b'\\r\\n\\r\\n' + body)\n"
            "    proc.stdin.flush()\n"
            "def read_exact(fd, n, deadline):\n"
            "    chunks = []\n"
            "    total = 0\n"
            "    while total < n:\n"
            "        remaining = deadline - time.time()\n"
            "        if remaining <= 0:\n"
            "            raise TimeoutError('read body timeout')\n"
            "        ready, _, _ = select.select([fd], [], [], remaining)\n"
            "        if not ready:\n"
            "            continue\n"
            "        chunk = os.read(fd, n - total)\n"
            "        if not chunk:\n"
            "            raise EOFError('server stdout closed')\n"
            "        chunks.append(chunk)\n"
            "        total += len(chunk)\n"
            "    return b''.join(chunks)\n"
            "def read_msg(proc, timeout=15):\n"
            "    fd = proc.stdout.fileno()\n"
            "    deadline = time.time() + timeout\n"
            "    header = b''\n"
            "    while b'\\r\\n\\r\\n' not in header:\n"
            "        remaining = deadline - time.time()\n"
            "        if remaining <= 0:\n"
            "            raise TimeoutError('read header timeout')\n"
            "        ready, _, _ = select.select([fd], [], [], remaining)\n"
            "        if not ready:\n"
            "            continue\n"
            "        chunk = os.read(fd, 1)\n"
            "        if not chunk:\n"
            "            raise EOFError('server stdout closed')\n"
            "        header += chunk\n"
            "    header_text = header.decode('ascii', errors='replace')\n"
            "    length = None\n"
            "    for line in header_text.split('\\r\\n'):\n"
            "        if line.lower().startswith('content-length:'):\n"
            "            length = int(line.split(':', 1)[1].strip())\n"
            "    if length is None:\n"
            "        raise RuntimeError('missing Content-Length in ' + repr(header_text))\n"
            "    body = read_exact(fd, length, deadline)\n"
            "    return json.loads(body.decode('utf-8'))\n"
            "def exchange(name, cmd, files):\n"
            "    exe = shutil.which(cmd[0])\n"
            "    if not exe:\n"
            "        return {'name': name, 'status': 'missing', 'cmd': cmd}\n"
            "    with tempfile.TemporaryDirectory(prefix='soma-lsp-') as td:\n"
            "        root = pathlib.Path(td)\n"
            "        for rel, text in files.items():\n"
            "            path = root / rel\n"
            "            path.parent.mkdir(parents=True, exist_ok=True)\n"
            "            path.write_text(text, encoding='utf-8')\n"
            "        proc = subprocess.Popen(cmd, cwd=td, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)\n"
            "        try:\n"
            "            init = {'jsonrpc':'2.0','id':1,'method':'initialize','params':{'processId':os.getpid(),'rootUri':root.as_uri(),'capabilities':{},'workspaceFolders':[{'uri':root.as_uri(),'name':'soma-lsp'}]}}\n"
            "            send(proc, init)\n"
            "            response = None\n"
            "            deadline = time.time() + 20\n"
            "            while time.time() < deadline:\n"
            "                msg = read_msg(proc, timeout=max(0.1, deadline - time.time()))\n"
            "                if msg.get('id') == 1:\n"
            "                    response = msg\n"
            "                    break\n"
            "            send(proc, {'jsonrpc':'2.0','method':'initialized','params':{}})\n"
            "            send(proc, {'jsonrpc':'2.0','id':2,'method':'shutdown','params':None})\n"
            "            shutdown = None\n"
            "            deadline = time.time() + 10\n"
            "            while time.time() < deadline:\n"
            "                msg = read_msg(proc, timeout=max(0.1, deadline - time.time()))\n"
            "                if msg.get('id') == 2:\n"
            "                    shutdown = msg\n"
            "                    break\n"
            "            send(proc, {'jsonrpc':'2.0','method':'exit','params':None})\n"
            "            try:\n"
            "                exit_code = proc.wait(timeout=5)\n"
            "            except subprocess.TimeoutExpired:\n"
            "                proc.terminate()\n"
            "                exit_code = proc.wait(timeout=5)\n"
            "            return {'name': name, 'status': 'ok' if response and shutdown and 'error' not in response else 'bad', 'cmd': cmd, 'exe': exe, 'realpath': os.path.realpath(exe), 'pid': proc.pid, 'root': td, 'initialize_has_result': bool(response and response.get('result')), 'initialize_error': response.get('error') if response else 'no response', 'shutdown_has_result': bool(shutdown is not None and 'error' not in shutdown), 'exit_code': exit_code}\n"
            "        except Exception as exc:\n"
            "            try:\n"
            "                proc.terminate()\n"
            "                proc.wait(timeout=3)\n"
            "            except Exception:\n"
            "                try:\n"
            "                    proc.kill()\n"
            "                except Exception:\n"
            "                    pass\n"
            "            return {'name': name, 'status': 'error', 'cmd': cmd, 'exe': exe, 'realpath': os.path.realpath(exe), 'pid': proc.pid, 'error': repr(exc)}\n"
            "servers = [\n"
            "    ('clangd', ['clangd', '--log=error'], {'main.cc': 'int add(int a,int b){return a+b;}\\n'}),\n"
            "    ('pyright-langserver', ['pyright-langserver', '--stdio'], {'main.py': 'def add(a: int, b: int) -> int:\\n    return a + b\\n'}),\n"
            "    ('rust-analyzer', ['rust-analyzer'], {'Cargo.toml': '[package]\\nname=\"soma_lsp\"\\nversion=\"0.1.0\"\\nedition=\"2021\"\\n', 'src/main.rs': 'fn main(){println!(\"soma\");}\\n'}),\n"
            "]\n"
            "results = [exchange(name, cmd, files) for name, cmd, files in servers]\n"
            "print('SOMA_LSP_PROBE:' + json.dumps({'results': results}, sort_keys=True))\n"))
          (result (soma-probe--python-script-output script 75))
          (status (alist-get "status" result nil nil #'string=))
          (output (or (alist-get "output" result nil nil #'string=) ""))
          (json-line (soma-probe--json-line-with-prefix "SOMA_LSP_PROBE:" output))
          (parsed (and json-line
                       (condition-case nil
                           (soma-probe--read-json-string json-line)
                         (error nil))))
          (results (or (and parsed
                            (soma-probe--plan-get parsed "results"))
                       nil))
          (bad (seq-remove
                (lambda (entry)
                  (string= (or (soma-probe--plan-get entry "status") "") "ok"))
                results))
          (bad-paths (seq-filter
                      (lambda (entry)
                        (let ((realpath (or (soma-probe--plan-get entry "realpath") "")))
                          (and (not (string-empty-p realpath))
                               (not (soma-probe--rootfs-owned-executable-path-p realpath)))))
                      results)))
     (soma-probe--alist
      (cons "observed"
            (format "prod=%S status=%S results=%S bad=%S bad-paths=%S output=%S"
                    prod status results bad bad-paths
                    (substring output 0 (min 2000 (length output)))))
      (cons "checks"
            (list
             (soma-probe--check "LSP stdio initializer script ran in production"
                                (or (not prod) (and (equal status 0) parsed))
                                (format "status=%S output=%S" status output))
             (soma-probe--check "production LSP servers initialized and shut down"
                                (or (not prod) (and results (null bad)))
                                (format "%S" bad))
             (soma-probe--check "production LSP server executables are rootfs-owned"
                                (or (not prod) (null bad-paths))
                                (format "%S" bad-paths))))))))

(defun soma-probe-comint-subprocess-rootfs-identity ()
  (soma-probe--case
   "comint-subprocess-rootfs-identity"
   "Comint subprocess rootfs identity"
   "Comint subprocesses must inherit the same rootfs contract as Emacs when they are part of a benchmark task."
   "A shell process launched through Comint should report the same run id and current directory; production mode must additionally prove tool resolution inside the selected rootfs."
   (list
    (soma-probe--source "Comint process benchmark issue" ".scratch/emacs-gym/issues/0007-comint-process-benchmark.md" "16-40")
    (soma-probe--source "Comint contract" "docs/internals-atlas/package-contracts/comint.md" "18-33"))
   "One short-lived Comint subprocess that prints cwd, run id, PATH, and Python resolution."
   "The subprocess observes the Insula run id, but development mode remains a non-hermetic host-bind classification."
   "A process-buffer benchmark can pass with the right output while the shell and tools came from the host runtime."
   "Comint benchmark graders must record process identity, cwd, PATH/tool resolution, rootfs manifest, and timeout/unknown state before trusting process output."
   (require 'comint)
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (run-id (or (getenv "SOMA_INSULA_RUN_ID") ""))
          (buf (generate-new-buffer " *soma-comint-rootfs-probe*"))
          proc output)
     (unwind-protect
         (progn
           (make-comint-in-buffer
            "soma-comint-rootfs-probe" buf "sh" nil "-c"
            "printf 'cwd=%s\\nrun_id=%s\\npython=%s\\npath=%s\\n' \"$PWD\" \"$SOMA_INSULA_RUN_ID\" \"$(command -v python3 || true)\" \"$PATH\"")
           (setq proc (get-buffer-process buf))
           (set-process-query-on-exit-flag proc nil)
           (let ((deadline (+ (float-time) 1.5)))
             (while (and (process-live-p proc) (< (float-time) deadline))
               (accept-process-output proc 0.05)))
           (with-current-buffer buf
             (setq output (buffer-substring-no-properties (point-min) (point-max))))
           (soma-probe--alist
            (cons "observed"
                  (format "prod=%S run-id=%S output=%S" prod run-id output))
            (cons "checks"
                  (list
                   (soma-probe--check "Comint subprocess emitted identity"
                                      (and (string-match-p "^cwd=" output)
                                           (string-match-p "^run_id=" output))
                                      output)
                   (soma-probe--check "Insula run id reached Comint subprocess"
                                      (and (not (string-empty-p run-id))
                                           (string-match-p (regexp-quote run-id) output))
                                      (format "run-id=%S output=%S" run-id output))
                   (soma-probe--check "Comint rootfs closure requirement is classified"
                                      (or (not prod)
                                          (and (string-match-p "python=/" output)
                                               (soma-probe--production-rootfs-p plan)))
                                      (format "prod=%S output=%S" prod output))))))
       (when (process-live-p proc)
         (delete-process proc))
       (when (buffer-live-p buf)
         (kill-buffer buf))))))

(defun soma-probe-comint-long-lived-shell-cwd-tool-drift ()
  (soma-probe--case
   "comint-long-lived-shell-cwd-tool-drift"
   "Comint long-lived shell cwd and tool drift"
   "Comint owns a live process buffer whose cwd, input boundary, and environment can drift across submitted commands."
   "A persistent shell launched through Comint should retain cwd changes, preserve the Insula run id, and resolve tools through the selected rootfs after state changes."
   (list
    (soma-probe--source "Comint contract" "docs/internals-atlas/package-contracts/comint.md" "18-33")
    (soma-probe--source "Comint input boundary probe" "probes/emacs-internals/elisp/run-probes.el" "comint-process-mark-input-boundary"))
   "One long-lived `/bin/sh' Comint process, a submitted command that changes cwd, and tool/Python identity probes after the change."
   "Production mode should show a live shell process, cwd persistence into `/project/tmp', stable run id, and rootfs-owned tool resolution."
   "A one-shot subprocess check can miss cwd drift, stale process identity, user input below the process mark, or PATH mutation across later sends."
   "Comint actions must record process pid/executable, process mark, cwd, live input, tool resolution, and environment after every wait or submitted command."
   (require 'comint)
   (let* ((plan (soma-probe--insula-plan))
          (prod (soma-probe--production-rootfs-p plan))
          (run-id (or (getenv "SOMA_INSULA_RUN_ID") ""))
          (buf (generate-new-buffer " *soma-comint-long-lived*"))
          proc output pid proc-exe)
     (unwind-protect
         (progn
           (make-comint-in-buffer "soma-comint-long-lived" buf "sh" nil)
           (setq proc (get-buffer-process buf))
           (set-process-query-on-exit-flag proc nil)
           (setq pid (process-id proc))
           (setq proc-exe
                 (let ((proc-exe-path (format "/proc/%s/exe" pid)))
                   (if (file-exists-p proc-exe-path)
                       (file-truename proc-exe-path)
                     "")))
           (with-current-buffer buf
             (goto-char (process-mark proc))
             (insert
              "printf 'SOMA_BEFORE:%s\\n' \"$PWD\"; "
              "cd \"$TMPDIR\"; "
              "printf 'SOMA_AFTER:%s\\n' \"$PWD\"; "
              "printf 'SOMA_RUN:%s\\n' \"$SOMA_INSULA_RUN_ID\"; "
              "printf 'SOMA_PY:%s\\n' \"$(command -v python3 || true)\"; "
              "python3 -c 'import json, os, sys; print(\"SOMA_PY_JSON:\" + json.dumps({\"executable\": sys.executable, \"cwd\": os.getcwd(), \"run_id\": os.environ.get(\"SOMA_INSULA_RUN_ID\", \"\")}, sort_keys=True))'; "
              "printf 'SOMA_DONE\\n'")
             (comint-send-input))
           (soma-probe--wait-buffer-regexp buf "^SOMA_DONE$" 5)
           (with-current-buffer buf
             (setq output (buffer-substring-no-properties (point-min) (point-max))))
           (soma-probe--alist
            (cons "observed"
                  (format "prod=%S pid=%S proc-exe=%S output=%S"
                          prod pid proc-exe output))
            (cons "checks"
                  (list
                   (soma-probe--check "Comint shell process was live and identifiable"
                                      (and pid (> pid 0))
                                      (format "pid=%S proc-exe=%S" pid proc-exe))
                   (soma-probe--check "Comint shell retained cwd change"
                                      (and (string-match-p "SOMA_BEFORE:/workspace/soma" output)
                                           (string-match-p "SOMA_AFTER:/project/tmp" output))
                                      output)
                   (soma-probe--check "Insula run id persisted in long-lived shell"
                                      (and (not (string-empty-p run-id))
                                           (string-match-p
                                            (concat "SOMA_RUN:" (regexp-quote run-id))
                                            output))
                                      (format "run-id=%S output=%S" run-id output))
                   (soma-probe--check "production long-lived shell resolves rootfs Python"
                                      (or (not prod)
                                          (and (string-match-p "SOMA_PY:/usr/bin/python3" output)
                                               (string-match-p
                                                (concat "\"run_id\": \"" (regexp-quote run-id) "\"")
                                                output)))
                                      output)
                   (soma-probe--check "production shell executable path is rootfs-owned"
                                      (or (not prod)
                                          (soma-probe--rootfs-owned-executable-path-p proc-exe))
                                      proc-exe)))))
       (when (process-live-p proc)
         (delete-process proc))
       (when (buffer-live-p buf)
         (kill-buffer buf))))))

(defun soma-probe-comint-timeout-outcome-unknown-no-retry ()
  (soma-probe--case
   "comint-timeout-outcome-unknown-no-retry"
   "Comint timeout outcome_unknown without retry"
   "Timing out while waiting for process output means the submitted command's outcome is unknown, not that it did not run."
   "A slow non-idempotent command submitted once through Comint should be classified `outcome_unknown' at the short deadline and later prove a single side effect."
   (list
    (soma-probe--source "Comint contract" "docs/internals-atlas/package-contracts/comint.md" "18-33")
    (soma-probe--source "with-timeout boundary probe" "probes/emacs-internals/elisp/run-probes.el" "with-timeout-wait-boundary"))
   "One long-lived shell, one counter file, one submitted slow command, and two waits: short timeout then final completion."
   "The short wait should not see completion; the later wait should show the counter incremented exactly once."
   "A retry after a timeout can duplicate non-idempotent work while still showing the expected final output."
   "Comint and process actions must represent timeout as `outcome_unknown' and block blind retry unless the operation is proven idempotent or externally canceled."
   (require 'comint)
   (let* ((tmp (make-temp-file "soma-comint-timeout-" t))
          (counter (expand-file-name "counter" tmp))
          (buf (generate-new-buffer " *soma-comint-timeout*"))
          proc early-done final-done output counter-value)
     (unwind-protect
         (progn
           (with-temp-file counter
             (insert "0\n"))
           (make-comint-in-buffer "soma-comint-timeout" buf "sh" nil)
           (setq proc (get-buffer-process buf))
           (set-process-query-on-exit-flag proc nil)
           (with-current-buffer buf
             (goto-char (process-mark proc))
             (insert
              (format
               "sleep 0.6; n=$(cat %s); n=$((n+1)); printf '%%s\\n' \"$n\" > %s; printf 'SOMA_TIMEOUT_DONE:%%s\\n' \"$n\""
               (shell-quote-argument counter)
               (shell-quote-argument counter)))
             (comint-send-input))
           (setq early-done
                 (soma-probe--wait-buffer-regexp
                  buf "^SOMA_TIMEOUT_DONE:" 0.15))
           (setq final-done
                 (soma-probe--wait-buffer-regexp
                  buf "SOMA_TIMEOUT_DONE:1" 2.0))
           (setq counter-value
                 (string-trim
                  (with-temp-buffer
                    (insert-file-contents counter)
                    (buffer-string))))
           (with-current-buffer buf
             (setq output (buffer-substring-no-properties (point-min) (point-max))))
           (soma-probe--alist
            (cons "observed"
                  (format "early-done=%S outcome=outcome_unknown final-done=%S counter=%S output=%S"
                          early-done final-done counter-value output))
            (cons "checks"
                  (list
                   (soma-probe--check "short wait reached outcome_unknown before completion"
                                      (not early-done)
                                      (format "early-done=%S" early-done))
                   (soma-probe--check "submitted command eventually completed"
                                      final-done
                                      output)
                   (soma-probe--check "non-idempotent side effect occurred exactly once"
                                      (string= counter-value "1")
                                      (format "counter=%S output=%S"
                                              counter-value output))))))
       (when (process-live-p proc)
         (delete-process proc))
       (when (buffer-live-p buf)
         (kill-buffer buf))
       (when (file-directory-p tmp)
         (delete-directory tmp t))))))

(defun soma-probe-summary ()
  (let* ((records (reverse soma-probe-results))
         (total (length records))
         (passed (cl-count-if (lambda (r) (string= (alist-get "status" r nil nil #'string=) "pass"))
                              records))
         (insula-runtime (soma-probe--insula-runtime))
         (insula-passed (soma-probe--all-checks-pass-p
                         (alist-get "checks" insula-runtime nil nil #'string=)))
         (summary (soma-probe--alist
                   (cons "generated_at" (format-time-string "%FT%TZ" nil t))
                   (cons "total" total)
                   (cons "passed" passed)
                   (cons "insula_passed" (soma-probe--bool insula-passed))
                   (cons "insula_runtime" insula-runtime)
                   (cons "records" records))))
    (soma-probe--write-json (expand-file-name "summary.json" soma-probe-output-dir)
                            summary)
    (with-temp-file (expand-file-name "summary.md" soma-probe-output-dir)
      (insert "# Emacs Internals Probe Summary\n\n")
      (insert (format "- Generated: %s\n" (format-time-string "%FT%TZ" nil t)))
      (insert (format "- Total probes: %d\n" total))
      (insert (format "- Passing probes: %d\n\n" passed))
      (insert "## Insula Runtime\n\n")
      (insert (format "- Checks passed: `%S`\n" insula-passed))
      (insert (format "- Active: `%S`\n" (alist-get "active" insula-runtime nil nil #'string=)))
      (insert (format "- Mode: `%s`\n" (alist-get "mode" insula-runtime nil nil #'string=)))
      (insert (format "- HOME: `%s`\n" (alist-get "home" insula-runtime nil nil #'string=)))
      (insert (format "- TMPDIR: `%s`\n" (alist-get "tmpdir" insula-runtime nil nil #'string=)))
      (insert (format "- XDG_RUNTIME_DIR: `%s`\n" (alist-get "xdg_runtime_dir" insula-runtime nil nil #'string=)))
      (insert (format "- Unexpected environment names: `%S`\n\n"
                      (alist-get "unexpected_env_names" insula-runtime nil nil #'string=)))
      (dolist (record records)
        (insert (format "## %s\n\n" (alist-get "title" record nil nil #'string=)))
        (insert (format "- ID: `%s`\n" (alist-get "id" record nil nil #'string=)))
        (insert (format "- Status: `%s`\n" (alist-get "status" record nil nil #'string=)))
        (insert (format "- Observed: %s\n" (alist-get "observed_behavior" record nil nil #'string=)))
        (insert (format "- Requirement: %s\n\n" (alist-get "derived_gym_requirement" record nil nil #'string=)))))))

(defun soma-run-probes ()
  (let ((args (seq-remove (lambda (arg) (string= arg "--"))
                          command-line-args-left)))
    (setq soma-probe-output-dir
          (expand-file-name
           (or (car args)
               "probes/emacs-internals/traces/latest"))))
  (make-directory soma-probe-output-dir t)
  (setq soma-probe-results nil)
  (mapc #'funcall
        '(soma-probe-insula-rootfs-contract
          soma-probe-insula-plan-runtime-consistency
          soma-probe-rootfs-mount-boundary-evidence
          soma-probe-toolchain-resolution-closure
          soma-probe-toolchain-executable-provenance
          soma-probe-toolchain-smoke-build-rootfs-identity
          soma-probe-subprocess-environment-closure
          soma-probe-native-comp-cache-closure
          soma-probe-abi-surface-manifest
          soma-probe-emacs-patch-provenance
          soma-probe-jupyter-kernel-rootfs-proxy
          soma-probe-jupyter-kernelspec-rootfs-contract
          soma-probe-jupyter-real-kernel-rootfs-identity
          soma-probe-lsp-server-rootfs-resolution
          soma-probe-lsp-stdio-initialize-rootfs-identity
          soma-probe-comint-subprocess-rootfs-identity
          soma-probe-comint-long-lived-shell-cwd-tool-drift
          soma-probe-comint-timeout-outcome-unknown-no-retry
          soma-probe-current-buffer-window
          soma-probe-command-routes
          soma-probe-magit-wrapper
          soma-probe-indirect-buffer
          soma-probe-marker-movement
          soma-probe-narrowing-region-ticks
          soma-probe-minibuffer-transient
          soma-probe-reentrant-process-output
          soma-probe-doom-cold-warm
          soma-probe-org-src-propagation
          soma-probe-jupyter-delayed-retry
          soma-probe-fim-stale-acceptance
          soma-probe-observation-fontification
          soma-probe-capf-exclusivity
          soma-probe-capf-exit-function-acceptance
          soma-probe-comint-input-boundary
          soma-probe-dired-undo-filesystem
          soma-probe-with-timeout-limits))
  (soma-probe-summary)
  (let ((failed (cl-remove-if
                 (lambda (r) (string= (alist-get "status" r nil nil #'string=) "pass"))
                 soma-probe-results))
        (insula-runtime (soma-probe--insula-runtime))
        (insula-failed nil))
    (setq insula-failed
          (seq-remove
           (lambda (check) (eq (alist-get "passed" check nil nil #'string=) t))
           (alist-get "checks" insula-runtime nil nil #'string=)))
    (when (or failed insula-failed)
      (when failed
        (princ (format "Probe failures: %S\n"
                       (mapcar (lambda (r) (alist-get "id" r nil nil #'string=)) failed))))
      (when insula-failed
        (princ (format "Insula runtime failures: %S\n"
                       (mapcar (lambda (r) (alist-get "name" r nil nil #'string=)) insula-failed))))
      (kill-emacs 1))))

(soma-run-probes)

;;; run-probes.el ends here
