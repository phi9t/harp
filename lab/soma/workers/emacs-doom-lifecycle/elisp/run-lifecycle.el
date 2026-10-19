;;; run-lifecycle.el --- Doom lifecycle worker for the Soma Insula -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'json)
(require 'seq)
(require 'subr-x)

(defvar soma-lifecycle-results nil)

(defconst soma-lifecycle-child-token-env "SOMA_LIFECYCLE_CHILD_TOKEN")

(defconst soma-lifecycle-important-features
  '(doom doom-lib doom-start doom-straight doom-keybinds doom-ui doom-projects
    doom-editor use-package magit transient with-editor org ob-jupyter jupyter
    jupyter-repl lsp-mode lsp-pyright python rust-mode cc-mode sh-script
    corfu vertico popup))

(defconst soma-lifecycle-capabilities
  '((("name" . "corfu")
     ("module_patterns" . ("corfu"))
     ("packages" . ("corfu"))
     ("features" . (corfu))
     ("mode_functions" . (global-corfu-mode corfu-mode))
     ("external_tools" . ()))
    (("name" . "vertico")
     ("module_patterns" . ("vertico"))
     ("packages" . ("vertico"))
     ("features" . (vertico))
     ("mode_functions" . (vertico-mode))
     ("external_tools" . ()))
    (("name" . "popup")
     ("module_patterns" . ("popup"))
     ("packages" . ("popper" "shackle"))
     ("features" . (doom-ui))
     ("mode_functions" . ())
     ("external_tools" . ()))
    (("name" . "tty")
     ("module_patterns" . ("tty"))
     ("packages" . ())
     ("features" . ())
     ("mode_functions" . ())
     ("external_tools" . ("tmux")))
    (("name" . "magit")
     ("module_patterns" . ("magit" "(:tools . magit)"))
     ("packages" . ("magit" "transient" "with-editor"))
     ("features" . (magit))
     ("mode_functions" . ())
     ("external_tools" . ("git")))
    (("name" . "org+jupyter")
     ("module_patterns" . ("(org +jupyter)" ":lang.*org"))
     ("packages" . ("org" "emacs-jupyter" "jupyter"))
     ("features" . (org jupyter))
     ("mode_functions" . (org-mode))
     ("external_tools" . ("jupyter" "python3")))
    (("name" . "lsp")
     ("module_patterns" . ("lsp" "(:tools . lsp)"))
     ("packages" . ("lsp-mode" "lsp-pyright"))
     ("features" . (lsp-mode))
     ("mode_functions" . ())
     ("external_tools" . ("pyright-langserver" "clangd" "rust-analyzer")))
    (("name" . "python")
     ("module_patterns" . ("python" "(:lang . python)"))
     ("packages" . ("python" "lsp-pyright"))
     ("features" . (python))
     ("mode_functions" . (python-mode))
     ("external_tools" . ("python3" "pyright-langserver")))
    (("name" . "rust")
     ("module_patterns" . ("rust" "(:lang . rust)"))
     ("packages" . ("rust-mode"))
     ("features" . (rust-mode))
     ("mode_functions" . (rust-mode))
     ("external_tools" . ("rustc" "cargo" "rust-analyzer")))
    (("name" . "cc")
     ("module_patterns" . ("cc" "(:lang . cc)"))
     ("packages" . ())
     ("features" . (cc-mode))
     ("mode_functions" . (c++-mode c-mode))
     ("external_tools" . ("clang" "clangd" "gcc" "g++")))
    (("name" . "sh")
     ("module_patterns" . ("sh" "(:lang . sh)"))
     ("packages" . ())
     ("features" . (sh-script))
     ("mode_functions" . (sh-mode))
     ("external_tools" . ("sh" "bash" "shellcheck")))))

(defun soma-lifecycle--alist (&rest pairs)
  pairs)

(defun soma-lifecycle--bool (value)
  (if value t :json-false))

(defun soma-lifecycle--get (alist key)
  (alist-get key alist nil nil #'string=))

(defun soma-lifecycle--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-lifecycle--read-json-file (path)
  (when (and path (not (string-empty-p path)) (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-lifecycle--plan ()
  (soma-lifecycle--read-json-file (or (getenv "SOMA_INSULA_PLAN") "")))

(defun soma-lifecycle--contract ()
  (soma-lifecycle--read-json-file "/etc/soma-insula-rootfs-contract.json"))

(defun soma-lifecycle--check (name passed detail)
  (soma-lifecycle--alist
   (cons "name" name)
   (cons "passed" (soma-lifecycle--bool passed))
   (cons "detail" (or detail ""))))

(defun soma-lifecycle--all-checks-pass-p (checks)
  (cl-every
   (lambda (check)
     (eq (soma-lifecycle--get check "passed") t))
   checks))

(defun soma-lifecycle--child-token (phase)
  (secure-hash
   'sha256
   (format "%s:%s:%s:%s"
           (or (getenv "SOMA_INSULA_RUN_ID") "")
           phase
           (format-time-string "%FT%T.%NZ" nil t)
           (random))))

(defun soma-lifecycle--proc-file-string (path)
  (when (file-readable-p path)
    (with-temp-buffer
      (insert-file-contents-literally path)
      (buffer-substring-no-properties (point-min) (point-max)))))

(defun soma-lifecycle--proc-token-processes (token)
  (let ((needle (format "%s=%s" soma-lifecycle-child-token-env token))
        processes)
    (when (file-directory-p "/proc")
      (dolist (pid (directory-files "/proc" nil "\\`[0-9]+\\'"))
        (let* ((proc-dir (expand-file-name pid "/proc"))
               (environ-file (expand-file-name "environ" proc-dir))
               (environ (ignore-errors
                          (soma-lifecycle--proc-file-string environ-file))))
          (when (and environ (string-search needle environ))
            (let ((cmdline (or (ignore-errors
                                 (soma-lifecycle--proc-file-string
                                  (expand-file-name "cmdline" proc-dir)))
                               ""))
                  (comm (or (ignore-errors
                              (soma-lifecycle--proc-file-string
                               (expand-file-name "comm" proc-dir)))
                            "")))
              (push
               (soma-lifecycle--alist
                (cons "pid" pid)
                (cons "comm" (string-trim comm))
                (cons "cmdline"
                      (string-trim (replace-regexp-in-string "\0" " " cmdline))))
               processes))))))
    (sort processes
          (lambda (a b)
            (< (string-to-number (soma-lifecycle--get a "pid"))
               (string-to-number (soma-lifecycle--get b "pid")))))))

(defun soma-lifecycle--teardown-checks (phase preexisting remaining)
  (list
   (soma-lifecycle--check
    (format "%s child token was unique before launch" phase)
    (null preexisting)
    (format "%S" preexisting))
   (soma-lifecycle--check
    (format "%s child left no token-marked processes after exit" phase)
    (null remaining)
    (format "%S" remaining))))

(defun soma-lifecycle--warmed-phase-p (phase)
  (or (string= phase "warmed")
      (string-prefix-p "warmed-" phase)))

(defun soma-lifecycle--shell-output-timeout (timeout program &rest args)
  (let ((buffer (generate-new-buffer " *soma-lifecycle-process*"))
        (status nil)
        (timed-out nil)
        (error-text "")
        process)
    (unwind-protect
        (progn
          (condition-case err
              (setq process
                    (make-process
                     :name "soma-lifecycle-process"
                     :buffer buffer
                     :command (cons program args)
                     :connection-type 'pipe
                     :sentinel #'ignore
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
          (soma-lifecycle--alist
           (cons "status" status)
           (cons "timed_out" (soma-lifecycle--bool timed-out))
           (cons "error" error-text)
           (cons "output"
                 (with-current-buffer buffer
                   (buffer-substring-no-properties (point-min) (point-max))))))
      (when (buffer-live-p buffer)
        (kill-buffer buffer)))))

(defun soma-lifecycle--file-sha256 (path)
  (when (file-readable-p path)
    (with-temp-buffer
      (insert-file-contents-literally path)
      (secure-hash 'sha256 (current-buffer)))))

(defun soma-lifecycle--file-info (name path)
  (let ((attrs (and (file-exists-p path) (file-attributes path))))
    (soma-lifecycle--alist
     (cons "name" name)
     (cons "path" path)
     (cons "exists" (soma-lifecycle--bool (file-exists-p path)))
     (cons "readable" (soma-lifecycle--bool (file-readable-p path)))
     (cons "directory" (soma-lifecycle--bool (file-directory-p path)))
     (cons "size" (or (and attrs (file-attribute-size attrs)) 0))
     (cons "sha256" (or (and (not (file-directory-p path))
                             (soma-lifecycle--file-sha256 path))
                        "")))))

(defun soma-lifecycle--git-head (dir)
  (if (file-directory-p dir)
      (let ((result (soma-lifecycle--shell-output-timeout
                     3 "git" "-C" dir "rev-parse" "HEAD")))
        (soma-lifecycle--alist
         (cons "path" dir)
         (cons "status" (format "%S" (soma-lifecycle--get result "status")))
         (cons "head"
               (or (seq-find
                    (lambda (line)
                      (string-match-p "\\`[0-9a-f]\\{40,\\}\\'" line))
                    (split-string
                     (string-trim
                      (or (soma-lifecycle--get result "output") ""))
                     "\n" t))
                   ""))))
    (soma-lifecycle--alist
     (cons "path" dir)
     (cons "status" "missing")
     (cons "head" ""))))

(defun soma-lifecycle--doom-emacs-dir ()
  (file-name-as-directory
   (or (and (boundp 'doom-emacs-dir)
            (soma-lifecycle--safe-symbol-value 'doom-emacs-dir))
       (getenv "EMACSDIR")
       "~/.emacs.d")))

(defun soma-lifecycle--doom-user-dir ()
  (file-name-as-directory
   (or (and (boundp 'doom-user-dir)
            (soma-lifecycle--safe-symbol-value 'doom-user-dir))
       (getenv "DOOMDIR")
       "~/.doom.d")))

(defun soma-lifecycle--generated-profile-init ()
  (expand-file-name
   (format ".local/etc/@/init.%d.%d.el"
           emacs-major-version emacs-minor-version)
   (soma-lifecycle--doom-emacs-dir)))

(defun soma-lifecycle--profile-cache-file ()
  (expand-file-name ".local/cache/profiles.@.el"
                    (soma-lifecycle--doom-emacs-dir)))

(defun soma-lifecycle--load-generated-profile ()
  (let ((profile (soma-lifecycle--generated-profile-init))
        loaded startup)
    (condition-case err
        (progn
          (when (and (file-readable-p profile)
                     (fboundp 'doom-load))
            (setq user-init-file profile)
            (let ((noninteractive nil))
              (require 'doom-start)
              (doom-load profile t)
              (setq loaded t)
              (when (fboundp 'doom-startup)
                (doom-startup)
                (setq startup t))))
          (soma-lifecycle--alist
           (cons "path" profile)
           (cons "loaded" (soma-lifecycle--bool loaded))
           (cons "startup_ran" (soma-lifecycle--bool startup))
           (cons "error" "")))
      (error
       (soma-lifecycle--alist
        (cons "path" profile)
        (cons "loaded" :json-false)
        (cons "startup_ran" :json-false)
        (cons "error" (format "%S" err)))))))

(defun soma-lifecycle--text-contains-any-p (path patterns)
  (when (file-readable-p path)
    (with-temp-buffer
      (insert-file-contents path)
      (let ((text (buffer-substring-no-properties (point-min) (point-max))))
        (cl-some (lambda (token)
                   (string-search token text))
                 patterns)))))

(defun soma-lifecycle--directory-entries (dir &optional limit)
  (if (file-directory-p dir)
      (cl-subseq
       (sort (directory-files dir nil directory-files-no-dot-files-regexp)
             #'string<)
       0 (min (or limit 200)
              (length (directory-files dir nil directory-files-no-dot-files-regexp))))
    nil))

(defun soma-lifecycle--find-files-matching (dir regexp limit)
  (let (matches)
    (when (file-directory-p dir)
      (dolist (path (directory-files-recursively dir regexp nil nil t))
        (when (< (length matches) limit)
          (push path matches))))
    (reverse matches)))

(defun soma-lifecycle--path-prefix-p (prefix path)
  (and (stringp prefix)
       (stringp path)
       (let ((pfx (file-name-as-directory prefix)))
         (or (string= path (directory-file-name pfx))
             (string-prefix-p pfx path)))))

(defun soma-lifecycle--forbidden-path-p (path)
  (and (stringp path)
       (file-name-absolute-p path)
       (or (soma-lifecycle--path-prefix-p "/nix" path)
           (soma-lifecycle--path-prefix-p "/data02/home" path))))

(defun soma-lifecycle--declared-runtime-path-p (path)
  (or (not (stringp path))
      (string-empty-p path)
      (not (file-name-absolute-p path))
      (and (not (soma-lifecycle--forbidden-path-p path))
           (cl-some
            (lambda (prefix) (soma-lifecycle--path-prefix-p prefix path))
            '("/opt/soma"
              "/project/home/.emacs.d"
              "/project/home/.doom.d"
              "/project/cache"
              "/project/tmp"
              "/project/run"
              "/usr"
              "/bin"
              "/sbin"
              "/lib"
              "/lib64"
              "/etc")))))

(defun soma-lifecycle--path-checks (paths)
  (let* ((absolute (seq-filter (lambda (p)
                                 (and (stringp p)
                                      (file-name-absolute-p p)))
                               paths))
         (forbidden (seq-filter #'soma-lifecycle--forbidden-path-p absolute))
         (undeclared (seq-remove #'soma-lifecycle--declared-runtime-path-p
                                 absolute)))
    (list
     (soma-lifecycle--check "no forbidden host ABI paths"
                            (null forbidden)
                            (format "%S" forbidden))
     (soma-lifecycle--check "important runtime paths are declared"
                            (null undeclared)
                            (format "%S" undeclared)))))

(defun soma-lifecycle--safe-symbol-value (symbol)
  (when (boundp symbol)
    (symbol-value symbol)))

(defun soma-lifecycle--string-list (items)
  (delq nil
        (mapcar (lambda (item)
                  (cond
                   ((null item) nil)
                   ((symbolp item) (symbol-name item))
                   ((stringp item) item)
                   (t (format "%S" item))))
                items)))

(defun soma-lifecycle--doom-paths ()
  (let ((vars '(doom-emacs-dir doom-user-dir doom-local-dir doom-etc-dir
                doom-cache-dir doom-data-dir doom-state-dir doom-profile-dir
                doom-profile-data-dir doom-profile-cache-file
                doom-profile-init-file doom-user-init-file)))
    (mapcar
     (lambda (sym)
       (let ((value (soma-lifecycle--safe-symbol-value sym)))
         (soma-lifecycle--alist
          (cons "symbol" (symbol-name sym))
          (cons "bound" (soma-lifecycle--bool (boundp sym)))
          (cons "value"
                (cond
                 ((stringp value) value)
                 ((symbolp value) (symbol-name value))
                 ((null value) "")
                 (t (format "%S" value)))))))
     vars)))

(defun soma-lifecycle--module-state ()
  (let* ((init-file (expand-file-name "init.el" (soma-lifecycle--doom-user-dir)))
         (profile-file (soma-lifecycle--generated-profile-init))
         (profile-cache (soma-lifecycle--profile-cache-file))
         (module-vars '(doom-modules doom-module-list doom-disabled-packages))
         (module-vars-state
          (mapcar
           (lambda (sym)
             (let ((value (soma-lifecycle--safe-symbol-value sym)))
               (soma-lifecycle--alist
                (cons "symbol" (symbol-name sym))
                (cons "bound" (soma-lifecycle--bool (boundp sym)))
                (cons "value_sample"
                      (substring (format "%S" value)
                                 0 (min 2000 (length (format "%S" value))))))))
           module-vars)))
    (soma-lifecycle--alist
     (cons "doom_user_init" (soma-lifecycle--file-info "doom-user-init" init-file))
     (cons "generated_profile_init"
           (soma-lifecycle--file-info "generated-profile-init" profile-file))
     (cons "profile_cache"
           (soma-lifecycle--file-info "profile-cache" profile-cache))
     (cons "active_modules" (soma-lifecycle--active-doom-modules))
     (cons "module_variables" module-vars-state))))

(defun soma-lifecycle--doom-module-table ()
  (let ((value (soma-lifecycle--safe-symbol-value 'doom-modules)))
    (and (hash-table-p value) value)))

(defun soma-lifecycle--doom-module-field (module accessor)
  (when (and module (fboundp accessor))
    (ignore-errors (funcall accessor module))))

(defun soma-lifecycle--doom-module-key-pair (key)
  (cond
   ((and (consp key) (keywordp (car key)) (symbolp (cdr key)))
    (cons (car key) (cdr key)))
   ((and (consp key) (keywordp (car key)) (symbolp (cadr key)))
    (cons (car key) (cadr key)))
   (t nil)))

(defun soma-lifecycle--active-doom-modules ()
  (let ((table (soma-lifecycle--doom-module-table))
        records)
    (when table
      (maphash
       (lambda (key module)
         (let ((pair (soma-lifecycle--doom-module-key-pair key)))
           (push
            (soma-lifecycle--alist
             (cons "key" (format "%S" key))
             (cons "group"
                   (if pair
                       (symbol-name (car pair))
                     (or (and (symbolp
                               (soma-lifecycle--doom-module-field
                                module 'doom-module-group))
                              (symbol-name
                               (soma-lifecycle--doom-module-field
                                module 'doom-module-group)))
                         "")))
             (cons "name"
                   (if pair
                       (symbol-name (cdr pair))
                     (or (and (symbolp
                               (soma-lifecycle--doom-module-field
                                module 'doom-module-name))
                              (symbol-name
                               (soma-lifecycle--doom-module-field
                                module 'doom-module-name)))
                         "")))
             (cons "flags"
                   (soma-lifecycle--string-list
                    (soma-lifecycle--doom-module-field
                     module 'doom-module-flags)))
             (cons "path"
                   (or (soma-lifecycle--doom-module-field
                        module 'doom-module-path)
                       "")))
            records)))
       table))
    (sort records
          (lambda (a b)
            (string< (soma-lifecycle--get a "key")
                     (soma-lifecycle--get b "key"))))))

(defun soma-lifecycle--package-dir (kind package)
  (expand-file-name
   package
   (expand-file-name
    (format ".local/straight/%s-%d.%d"
            kind emacs-major-version emacs-minor-version)
    (soma-lifecycle--doom-emacs-dir))))

(defun soma-lifecycle--repo-dir (package)
  (expand-file-name
   package
   (expand-file-name ".local/straight/repos"
                     (soma-lifecycle--doom-emacs-dir))))

(defun soma-lifecycle--autoload-file (package)
  (let ((build-dir (soma-lifecycle--package-dir "build" package)))
    (car (soma-lifecycle--find-files-matching
          build-dir
          (format "\\`%s-autoloads\\.el\\(?:c\\)?\\'" (regexp-quote package))
          1))))

(defun soma-lifecycle--package-state (package)
  (let* ((build-dir (soma-lifecycle--package-dir "build" package))
         (repo-dir (soma-lifecycle--repo-dir package))
         (autoload (soma-lifecycle--autoload-file package)))
    (soma-lifecycle--alist
     (cons "package" package)
     (cons "build_dir" build-dir)
     (cons "build_exists" (soma-lifecycle--bool (file-directory-p build-dir)))
     (cons "repo_dir" repo-dir)
     (cons "repo_exists" (soma-lifecycle--bool (file-directory-p repo-dir)))
     (cons "autoload_file" (or autoload ""))
     (cons "autoload_available" (soma-lifecycle--bool autoload))
     (cons "revision" (soma-lifecycle--git-head repo-dir)))))

(defun soma-lifecycle--tool-state (tool)
  (let* ((path (executable-find tool))
         (version (and path
                       (apply #'soma-lifecycle--shell-output-timeout
                              3 path
                              (cond
                               ((string= tool "tmux") '("-V"))
                               ((string= tool "python3")
                                '("-c" "import sys; print(sys.version.split()[0])"))
                               (t '("--version")))))))
    (soma-lifecycle--alist
     (cons "tool" tool)
     (cons "path" (or path ""))
     (cons "found" (soma-lifecycle--bool path))
     (cons "declared_rootfs_path"
           (soma-lifecycle--bool
            (and path (soma-lifecycle--declared-runtime-path-p path))))
     (cons "version_status"
           (if version (format "%S" (soma-lifecycle--get version "status")) ""))
     (cons "version_output"
           (if version
               (substring (soma-lifecycle--get version "output")
                          0 (min 800 (length (soma-lifecycle--get version "output"))))
             "")))))

(defun soma-lifecycle--feature-state (features)
  (mapcar
   (lambda (feature)
     (soma-lifecycle--alist
      (cons "feature" (symbol-name feature))
      (cons "loaded" (soma-lifecycle--bool (featurep feature)))
      (cons "function_available"
            (soma-lifecycle--bool
             (fboundp (intern-soft (format "%s-mode" feature)))))))
   features))

(defun soma-lifecycle--generated-module-token (name)
  (pcase name
    ("corfu" "(:completion . corfu)")
    ("vertico" "(:completion . vertico)")
    ("popup" "(:ui . popup)")
    ("tty" "(:os . tty)")
    ("magit" "(:tools . magit)")
    ("org+jupyter" "(:lang . org)")
    ("lsp" "(:tools . lsp)")
    ("python" "(:lang . python)")
    ("rust" "(:lang . rust)")
    ("cc" "(:lang . cc)")
    ("sh" "(:lang . sh)")
    (_ nil)))

(defun soma-lifecycle--capability-module-specs (name)
  (pcase name
    ("corfu" '(((:completion . corfu) nil)))
    ("vertico" '(((:completion . vertico) nil)))
    ("popup" '(((:ui . popup) nil)))
    ("tty" '(((:os . tty) nil)))
    ("magit" '(((:tools . magit) nil)))
    ("org+jupyter" '(((:lang . org) (+jupyter))))
    ("lsp" '(((:tools . lsp) nil)))
    ("python" '(((:lang . python) nil)))
    ("rust" '(((:lang . rust) nil)))
    ("cc" '(((:lang . cc) nil)))
    ("sh" '(((:lang . sh) nil)))
    (_ nil)))

(defun soma-lifecycle--doom-module-spec-state (spec)
  (let* ((key (car spec))
         (flags (cadr spec))
         (table (soma-lifecycle--doom-module-table))
         (module (and table (gethash key table)))
         (present (if module t nil))
         (active
          (and (consp key)
               (fboundp 'doom-module-active-p)
               (ignore-errors
                 (doom-module-active-p (car key) (cdr key) flags)))))
    (soma-lifecycle--alist
     (cons "key" (format "%S" key))
     (cons "required_flags" (soma-lifecycle--string-list flags))
     (cons "present_in_doom_modules" (soma-lifecycle--bool present))
     (cons "active_with_required_flags" (soma-lifecycle--bool active))
     (cons "actual_flags"
           (soma-lifecycle--string-list
            (soma-lifecycle--doom-module-field module 'doom-module-flags)))
     (cons "path"
           (or (soma-lifecycle--doom-module-field
                module 'doom-module-path)
               "")))))

(defun soma-lifecycle--capability-declared-in-loaded-profile-p (module-states)
  (and module-states
       (cl-every
        (lambda (state)
          (eq (soma-lifecycle--get state "active_with_required_flags") t))
        module-states)))

(defun soma-lifecycle--capability-state (capability)
  (let* ((name (soma-lifecycle--get capability "name"))
         (patterns (soma-lifecycle--get capability "module_patterns"))
         (packages (soma-lifecycle--get capability "packages"))
         (features (soma-lifecycle--get capability "features"))
         (mode-functions (soma-lifecycle--get capability "mode_functions"))
         (external-tools (soma-lifecycle--get capability "external_tools"))
         (init-file (expand-file-name "init.el" (soma-lifecycle--doom-user-dir)))
         (profile-file (soma-lifecycle--generated-profile-init))
         (generated-token (soma-lifecycle--generated-module-token name))
         (doom-module-states
          (mapcar #'soma-lifecycle--doom-module-spec-state
                  (soma-lifecycle--capability-module-specs name))))
    (soma-lifecycle--alist
     (cons "name" name)
     (cons "module_declared_in_user_init"
           (soma-lifecycle--bool
            (soma-lifecycle--text-contains-any-p init-file patterns)))
     (cons "doom_module_state" doom-module-states)
     (cons "generated_profile_text_contains_token"
           (soma-lifecycle--bool
            (and generated-token
                 (soma-lifecycle--text-contains-any-p
                  profile-file
                  (list generated-token)))))
     (cons "module_active_in_loaded_profile"
           (soma-lifecycle--bool
            (soma-lifecycle--capability-declared-in-loaded-profile-p
             doom-module-states)))
     (cons "module_declared_in_generated_profile"
           (soma-lifecycle--bool
            (soma-lifecycle--capability-declared-in-loaded-profile-p
             doom-module-states)))
     (cons "packages_installed"
           (mapcar #'soma-lifecycle--package-state packages))
     (cons "autoload_available"
           (soma-lifecycle--bool
            (cl-some
             (lambda (pkg)
               (eq (soma-lifecycle--get
                    (soma-lifecycle--package-state pkg)
                    "autoload_available")
                   t))
             packages)))
     (cons "features_loaded" (soma-lifecycle--feature-state features))
     (cons "mode_functions_available"
           (mapcar (lambda (fn)
                     (soma-lifecycle--alist
                      (cons "function" (symbol-name fn))
                      (cons "available" (soma-lifecycle--bool (fboundp fn)))))
                   mode-functions))
     (cons "external_services"
           (mapcar #'soma-lifecycle--tool-state external-tools)))))

(defun soma-lifecycle--important-paths ()
  (let ((doom-path-values
         (delq nil
               (mapcar (lambda (entry)
                         (soma-lifecycle--get entry "value"))
                       (soma-lifecycle--doom-paths))))
        (native-paths
         (and (boundp 'native-comp-eln-load-path)
              (soma-lifecycle--safe-symbol-value 'native-comp-eln-load-path)))
        (tree-sitter-paths
         (and (boundp 'treesit-extra-load-path)
              (soma-lifecycle--safe-symbol-value 'treesit-extra-load-path))))
    (append
     (list (or (and invocation-directory
                    (expand-file-name invocation-name invocation-directory))
               invocation-name)
           exec-directory
           source-directory
           user-emacs-directory
           (soma-lifecycle--doom-emacs-dir)
           (soma-lifecycle--doom-user-dir)
           (or (getenv "EMACSDIR") "")
           (or (getenv "DOOMDIR") ""))
     doom-path-values
     (seq-filter #'stringp load-path)
     (seq-filter #'stringp native-paths)
     (seq-filter #'stringp tree-sitter-paths))))

(defun soma-lifecycle--emacs-identity ()
  (let ((emacs-path (or (and invocation-directory
                             (expand-file-name invocation-name invocation-directory))
                        (executable-find invocation-name)
                        invocation-name)))
    (soma-lifecycle--alist
     (cons "version" emacs-version)
     (cons "major" emacs-major-version)
     (cons "minor" emacs-minor-version)
     (cons "system_configuration" system-configuration)
     (cons "system_features" system-configuration-features)
     (cons "executable" emacs-path)
     (cons "executable_truename" (file-truename emacs-path))
     (cons "executable_sha256" (or (soma-lifecycle--file-sha256 emacs-path) ""))
     (cons "exec_directory" exec-directory)
     (cons "source_directory" source-directory)
     (cons "native_comp_available" (soma-lifecycle--bool (native-comp-available-p)))
     (cons "treesit_available" (soma-lifecycle--bool
                                (and (fboundp 'treesit-available-p)
                                     (treesit-available-p)))))))

(defun soma-lifecycle--runtime-identity ()
  (let* ((env-names (sort (mapcar (lambda (item) (car (split-string item "=" t)))
                                  process-environment)
                          #'string<))
         (allowed '("HOME" "LANG" "LC_ALL" "PATH" "PWD" "SHELL"
                    "SHLVL" "_"
                    "CARGO_HOME" "DOOMDIR" "EMACSDIR" "RUSTUP_HOME"
                    "SOMA_INSULA_ACTIVE" "SOMA_INSULA_MODE"
                    "SOMA_INSULA_PLAN" "SOMA_INSULA_RUN_ID"
                    "SOMA_LIFECYCLE_CHILD_TOKEN"
                    "TMPDIR" "XDG_CACHE_HOME" "XDG_CONFIG_HOME"
                    "XDG_DATA_HOME" "XDG_RUNTIME_DIR"))
         (unexpected (seq-remove (lambda (name) (member name allowed)) env-names)))
    (soma-lifecycle--alist
     (cons "home" (or (getenv "HOME") ""))
     (cons "tmpdir" (or (getenv "TMPDIR") ""))
     (cons "xdg_runtime_dir" (or (getenv "XDG_RUNTIME_DIR") ""))
     (cons "xdg_cache_home" (or (getenv "XDG_CACHE_HOME") ""))
     (cons "xdg_data_home" (or (getenv "XDG_DATA_HOME") ""))
     (cons "user_emacs_directory" user-emacs-directory)
     (cons "doomdir" (soma-lifecycle--doom-user-dir))
     (cons "emacsdir" (soma-lifecycle--doom-emacs-dir))
     (cons "insula_active" (soma-lifecycle--bool
                            (string= (getenv "SOMA_INSULA_ACTIVE") "1")))
     (cons "insula_mode" (or (getenv "SOMA_INSULA_MODE") ""))
     (cons "insula_plan" (or (getenv "SOMA_INSULA_PLAN") ""))
     (cons "env_names" env-names)
     (cons "unexpected_env_names" unexpected))))

(defun soma-lifecycle--library-surface ()
  (let* ((emacs-path (or (and invocation-directory
                              (expand-file-name invocation-name invocation-directory))
                         (executable-find invocation-name)
                         invocation-name))
         (ldd (soma-lifecycle--shell-output-timeout 3 "ldd" emacs-path))
         (output (or (soma-lifecycle--get ldd "output") "")))
    (soma-lifecycle--alist
     (cons "ldd_status" (format "%S" (soma-lifecycle--get ldd "status")))
     (cons "ldd_output" output)
     (cons "libgccjit_lines"
           (seq-filter (lambda (line) (string-match-p "libgccjit" line))
                       (split-string output "\n" t)))
     (cons "tree_sitter_lines"
           (seq-filter (lambda (line) (string-match-p "tree-sitter" line))
                       (split-string output "\n" t))))))

(defun soma-lifecycle--native-comp-state ()
  (let ((eln-paths (and (boundp 'native-comp-eln-load-path)
                        (soma-lifecycle--safe-symbol-value
                         'native-comp-eln-load-path))))
    (soma-lifecycle--alist
     (cons "available" (soma-lifecycle--bool (native-comp-available-p)))
     (cons "eln_load_path" (soma-lifecycle--string-list eln-paths))
     (cons "async_compilation"
           (soma-lifecycle--bool
            (and (boundp 'native-comp-deferred-compilation)
                 (soma-lifecycle--safe-symbol-value
                  'native-comp-deferred-compilation))))
     (cons "comp_libgccjit_reproducer"
           (if (boundp 'comp-libgccjit-reproducer)
               (format "%S" (soma-lifecycle--safe-symbol-value
                             'comp-libgccjit-reproducer))
             "")))))

(defun soma-lifecycle--tree-sitter-state ()
  (soma-lifecycle--alist
   (cons "available" (soma-lifecycle--bool
                      (and (fboundp 'treesit-available-p)
                           (treesit-available-p))))
   (cons "extra_load_path"
         (soma-lifecycle--string-list
          (and (boundp 'treesit-extra-load-path)
               (soma-lifecycle--safe-symbol-value
                'treesit-extra-load-path))))))

(defun soma-lifecycle--jupyter-kernelspec-state ()
  (let* ((python (executable-find "python3"))
         (result (and python
                      (soma-lifecycle--shell-output-timeout
                       8 python "-c"
                       "import json; from jupyter_client.kernelspec import KernelSpecManager; print(json.dumps(KernelSpecManager().get_all_specs(), sort_keys=True))"))))
    (soma-lifecycle--alist
     (cons "python" (or python ""))
     (cons "status" (if result
                        (format "%S" (soma-lifecycle--get result "status"))
                      "missing"))
     (cons "timed_out" (if result
                           (soma-lifecycle--get result "timed_out")
                         :json-false))
     (cons "output"
           (if result
               (substring (soma-lifecycle--get result "output")
                          0 (min 4000 (length (soma-lifecycle--get result "output"))))
             "")))))

(defun soma-lifecycle--ai-config-files ()
  (let ((doomdir (soma-lifecycle--doom-user-dir)))
    (mapcar
     (lambda (path)
       (soma-lifecycle--file-info "ai-config" path))
     (soma-lifecycle--find-files-matching
      doomdir
      ".*\\(minuet\\|tabby\\|tabbymacs\\|copilot\\|codeium\\|gptel\\|ellama\\|aider\\|devx\\).*"
      50))))

(defun soma-lifecycle--try-require (feature)
  (condition-case err
      (progn
        (require feature)
        (soma-lifecycle--alist
         (cons "feature" (symbol-name feature))
         (cons "status" "loaded")
         (cons "loaded" t)
         (cons "error" "")))
    (error
     (soma-lifecycle--alist
      (cons "feature" (symbol-name feature))
      (cons "status" "error")
      (cons "loaded" :json-false)
      (cons "error" (format "%S" err))))))

(defun soma-lifecycle--try-mode (mode-fn sample)
  (condition-case err
      (with-temp-buffer
        (insert sample)
        (when (fboundp mode-fn)
          (funcall mode-fn))
        (soma-lifecycle--alist
         (cons "function" (symbol-name mode-fn))
         (cons "available" (soma-lifecycle--bool (fboundp mode-fn)))
         (cons "status" (if (fboundp mode-fn) "activated" "missing"))
         (cons "major_mode" (symbol-name major-mode))
         (cons "error" "")))
    (error
     (soma-lifecycle--alist
      (cons "function" (symbol-name mode-fn))
      (cons "available" (soma-lifecycle--bool (fboundp mode-fn)))
      (cons "status" "error")
      (cons "major_mode" "")
      (cons "error" (format "%S" err))))))

(defun soma-lifecycle--warm-first-use ()
  (let* ((profile-load (soma-lifecycle--load-generated-profile))
         (requires (mapcar #'soma-lifecycle--try-require
                           '(magit org jupyter lsp-mode python rust-mode
                             cc-mode sh-script)))
         (modes (list
                 (soma-lifecycle--try-mode 'emacs-lisp-mode "(message \"x\")\n")
                 (soma-lifecycle--try-mode 'org-mode "* Heading\n")
                 (soma-lifecycle--try-mode 'python-mode "print('x')\n")
                 (soma-lifecycle--try-mode 'rust-mode "fn main() {}\n")
                 (soma-lifecycle--try-mode 'c++-mode "int main() { return 0; }\n")
                 (soma-lifecycle--try-mode 'sh-mode "echo x\n"))))
    (soma-lifecycle--alist
     (cons "profile_load" profile-load)
     (cons "requires" requires)
     (cons "mode_activations" modes)
     (cons "jupyter_kernelspec" (soma-lifecycle--jupyter-kernelspec-state)))))

(defun soma-lifecycle--warmup-checks (warmup)
  (when (and warmup (listp warmup))
    (let* ((profile-load (soma-lifecycle--get warmup "profile_load"))
           (requires (or (soma-lifecycle--get warmup "requires") nil))
           (modes (or (soma-lifecycle--get warmup "mode_activations") nil))
           (kernelspec (soma-lifecycle--get warmup "jupyter_kernelspec"))
           (require-errors
            (seq-remove
             (lambda (entry)
               (string= (soma-lifecycle--get entry "status") "loaded"))
             requires))
           (mode-errors
            (seq-remove
             (lambda (entry)
               (string= (soma-lifecycle--get entry "status") "activated"))
             modes)))
      (list
       (soma-lifecycle--check
        "warmed profile loaded through Doom loader"
        (eq (soma-lifecycle--get profile-load "loaded") t)
        (format "%S" profile-load))
       (soma-lifecycle--check
        "warmed Doom startup ran"
        (eq (soma-lifecycle--get profile-load "startup_ran") t)
        (format "%S" profile-load))
       (soma-lifecycle--check
        "warmed requested features loaded"
        (null require-errors)
        (format "%S" require-errors))
       (soma-lifecycle--check
        "warmed major modes activated"
        (null mode-errors)
        (format "%S" mode-errors))
       (soma-lifecycle--check
        "rootfs Jupyter kernelspec inspection succeeded"
        (string= (soma-lifecycle--get kernelspec "status") "0")
        (format "%S" kernelspec))))))

(defun soma-lifecycle--snapshot (phase warmup)
  (let* ((plan (soma-lifecycle--plan))
         (contract (soma-lifecycle--contract))
         (module-state (soma-lifecycle--module-state))
         (important-paths (soma-lifecycle--important-paths))
         (path-checks (soma-lifecycle--path-checks important-paths))
         (runtime (soma-lifecycle--runtime-identity))
         (emacs (soma-lifecycle--emacs-identity))
         (library-surface (soma-lifecycle--library-surface))
         (contract-emacs (soma-lifecycle--get contract "emacs"))
         (contract-doom (soma-lifecycle--get contract "doom"))
         (plan-mode (or (soma-lifecycle--get plan "mode") ""))
         (checks
          (append
           path-checks
           (soma-lifecycle--warmup-checks warmup)
           (list
            (soma-lifecycle--check
             "SOMA_INSULA_ACTIVE is set"
             (eq (soma-lifecycle--get runtime "insula_active") t)
             (format "%S" (soma-lifecycle--get runtime "insula_active")))
            (soma-lifecycle--check
             "selected rootfs mode is active"
             (string= plan-mode "selected-container-rootfs-insula")
             plan-mode)
            (soma-lifecycle--check
             "Emacs executable matches rootfs contract"
             (string= (file-truename
                       (soma-lifecycle--get emacs "executable_truename"))
                      (file-truename
                       (soma-lifecycle--get contract-emacs "executable")))
             (format "%s vs %s"
                     (soma-lifecycle--get emacs "executable_truename")
                     (soma-lifecycle--get contract-emacs "executable")))
            (soma-lifecycle--check
             "Emacs version matches rootfs contract"
             (string= (soma-lifecycle--get emacs "version")
                      (soma-lifecycle--get contract-emacs "version"))
             (format "%s vs %s"
                     (soma-lifecycle--get emacs "version")
                     (soma-lifecycle--get contract-emacs "version")))
            (soma-lifecycle--check
             "Doom core path is private rootfs mount"
             (string= (file-name-as-directory
                       (file-truename (soma-lifecycle--doom-emacs-dir)))
                      "/project/home/.emacs.d/")
             (file-truename (soma-lifecycle--doom-emacs-dir)))
            (soma-lifecycle--check
             "Doom user config path is private rootfs mount"
             (string= (file-name-as-directory
                       (file-truename (soma-lifecycle--doom-user-dir)))
                      "/project/home/.doom.d/")
             (file-truename (soma-lifecycle--doom-user-dir)))
            (soma-lifecycle--check
             "Doom commit matches rootfs contract"
             (string= (string-trim
                       (soma-lifecycle--get
                        (soma-lifecycle--git-head (soma-lifecycle--doom-emacs-dir))
                        "head"))
                      (soma-lifecycle--get contract-doom "commit"))
             (format "%s vs %s"
                     (soma-lifecycle--get
                      (soma-lifecycle--git-head (soma-lifecycle--doom-emacs-dir))
                      "head")
                     (soma-lifecycle--get contract-doom "commit")))
            (soma-lifecycle--check
             "generated profile init exists"
             (eq (soma-lifecycle--get
                  (soma-lifecycle--get module-state "generated_profile_init")
                  "exists")
                 t)
             (soma-lifecycle--get
              (soma-lifecycle--get module-state "generated_profile_init")
              "path"))
            (soma-lifecycle--check
             "profile cache exists"
             (eq (soma-lifecycle--get
                  (soma-lifecycle--get module-state "profile_cache")
                  "exists")
                 t)
             (soma-lifecycle--get
              (soma-lifecycle--get module-state "profile_cache")
              "path"))
            (soma-lifecycle--check
             "native compilation is available"
             (eq (soma-lifecycle--get emacs "native_comp_available") t)
             (format "%S" (soma-lifecycle--get emacs "native_comp_available")))
            (soma-lifecycle--check
             "libgccjit is in the Emacs dynamic surface"
             (not (null (soma-lifecycle--get library-surface "libgccjit_lines")))
             (format "%S" (soma-lifecycle--get library-surface
                                               "libgccjit_lines")))
            (soma-lifecycle--check
             "tree-sitter is in the Emacs dynamic surface"
             (not (null (soma-lifecycle--get library-surface "tree_sitter_lines")))
             (format "%S" (soma-lifecycle--get library-surface
                                               "tree_sitter_lines")))))))
    (soma-lifecycle--alist
     (cons "schema_version" 1)
     (cons "kind" "soma_doom_lifecycle_trace")
     (cons "phase" phase)
     (cons "generated_at" (format-time-string "%FT%TZ" nil t))
     (cons "plan" plan)
     (cons "contract" contract)
     (cons "runtime" runtime)
     (cons "emacs" emacs)
     (cons "doom_paths" (soma-lifecycle--doom-paths))
     (cons "doom_core_revision" (soma-lifecycle--git-head
                                 (soma-lifecycle--doom-emacs-dir)))
     (cons "module_state" module-state)
     (cons "straight_roots"
           (soma-lifecycle--alist
            (cons "build_dir"
                  (expand-file-name
                   (format ".local/straight/build-%d.%d"
                           emacs-major-version emacs-minor-version)
                   (soma-lifecycle--doom-emacs-dir)))
            (cons "repo_dir"
                  (expand-file-name ".local/straight/repos"
                                    (soma-lifecycle--doom-emacs-dir)))
            (cons "build_entries"
                  (soma-lifecycle--directory-entries
                   (expand-file-name
                    (format ".local/straight/build-%d.%d"
                            emacs-major-version emacs-minor-version)
                    (soma-lifecycle--doom-emacs-dir))
                   300))))
     (cons "capabilities"
           (mapcar #'soma-lifecycle--capability-state
                   soma-lifecycle-capabilities))
     (cons "loaded_features"
           (mapcar
            (lambda (feature)
              (soma-lifecycle--alist
               (cons "feature" (symbol-name feature))
               (cons "loaded" (soma-lifecycle--bool (featurep feature)))))
            soma-lifecycle-important-features))
     (cons "first_use" (or warmup :json-false))
     (cons "native_comp" (soma-lifecycle--native-comp-state))
     (cons "tree_sitter" (soma-lifecycle--tree-sitter-state))
     (cons "dynamic_library_surface" library-surface)
     (cons "devx_ai_config_files" (soma-lifecycle--ai-config-files))
     (cons "important_paths" (soma-lifecycle--string-list important-paths))
     (cons "checks" checks)
     (cons "status" (if (soma-lifecycle--all-checks-pass-p checks)
                        "pass"
                      "fail")))))

(defun soma-lifecycle--child-main (phase out-file)
  (let* ((warmup (when (soma-lifecycle--warmed-phase-p phase)
                   (soma-lifecycle--warm-first-use)))
         (snapshot (soma-lifecycle--snapshot phase warmup)))
    (soma-lifecycle--write-json out-file snapshot)
    (unless (string= (soma-lifecycle--get snapshot "status") "pass")
      (princ (format "Lifecycle %s checks failed\n" phase))
      (kill-emacs 1))))

(defun soma-lifecycle--run-child (phase out-dir)
  (let* ((emacs (or (and invocation-directory
                         (expand-file-name invocation-name invocation-directory))
                    "/opt/soma/emacs/bin/emacs"))
         (script (or load-file-name
                     "/workspace/soma/workers/emacs-doom-lifecycle/elisp/run-lifecycle.el"))
         (out-file (expand-file-name (format "%s.json" phase) out-dir))
         (log-file (expand-file-name (format "%s.log" phase) out-dir))
         (token (soma-lifecycle--child-token phase))
         (preexisting (soma-lifecycle--proc-token-processes token))
         (status
          (with-temp-file log-file
            (let ((process-environment
                   (cons (format "%s=%s" soma-lifecycle-child-token-env token)
                         process-environment)))
              (call-process
               emacs nil (current-buffer) nil
               "--quick"
               "--batch"
               "-l" "/project/home/.emacs.d/early-init.el"
               "-l" script
               "--" "--child" phase out-file))))
         (_ (sleep-for 0.2))
         (remaining (soma-lifecycle--proc-token-processes token))
         (teardown-checks
          (soma-lifecycle--teardown-checks phase preexisting remaining)))
    (soma-lifecycle--alist
     (cons "phase" phase)
     (cons "status" (format "%S" status))
     (cons "out_file" out-file)
     (cons "log_file" log-file)
     (cons "teardown_probe"
           (soma-lifecycle--alist
            (cons "env_var" soma-lifecycle-child-token-env)
            (cons "token" token)
            (cons "preexisting_token_processes" preexisting)
            (cons "post_exit_wait_seconds" 0.2)
            (cons "remaining_token_processes" remaining)
            (cons "checks" teardown-checks)
            (cons "status" (if (soma-lifecycle--all-checks-pass-p
                                teardown-checks)
                               "pass"
                             "fail"))))
     (cons "trace_exists" (soma-lifecycle--bool (file-readable-p out-file)))
     (cons "trace"
           (or (soma-lifecycle--read-json-file out-file)
               :json-false)))))

(defun soma-lifecycle--child-by-phase (child-results phase)
  (seq-find
   (lambda (result)
     (string= (soma-lifecycle--get result "phase") phase))
   child-results))

(defun soma-lifecycle--teardown-check-list (child-results)
  (apply #'append
         (mapcar
          (lambda (result)
            (or (soma-lifecycle--get
                 (soma-lifecycle--get result "teardown_probe")
                 "checks")
                nil))
          child-results)))

(defun soma-lifecycle--child-token-summaries (child-results)
  (mapcar
   (lambda (result)
     (let ((probe (soma-lifecycle--get result "teardown_probe")))
       (soma-lifecycle--alist
        (cons "phase" (soma-lifecycle--get result "phase"))
        (cons "token" (or (soma-lifecycle--get probe "token") ""))
        (cons "status" (soma-lifecycle--get probe "status"))
        (cons "preexisting_count"
              (length (or (soma-lifecycle--get
                           probe "preexisting_token_processes")
                          nil)))
        (cons "remaining_count"
              (length (or (soma-lifecycle--get
                           probe "remaining_token_processes")
                          nil))))))
   child-results))

(defun soma-lifecycle--child-tokens-unique-p (child-results)
  (let ((tokens (mapcar
                 (lambda (result)
                   (soma-lifecycle--get
                    (soma-lifecycle--get result "teardown_probe")
                    "token"))
                 child-results)))
    (and (cl-every (lambda (token)
                     (and (stringp token)
                          (not (string-empty-p token))))
                   tokens)
         (= (length tokens)
            (length (cl-remove-duplicates tokens :test #'string=))))))

(defun soma-lifecycle--summary (out-dir child-results)
  (let* ((teardown-checks (soma-lifecycle--teardown-check-list child-results))
         (token-summaries (soma-lifecycle--child-token-summaries child-results))
         (token-unique-check
          (soma-lifecycle--check
           "child teardown tokens are pairwise unique"
           (soma-lifecycle--child-tokens-unique-p child-results)
           (format "%S" token-summaries)))
         (aggregate-teardown-checks (append teardown-checks
                                            (list token-unique-check)))
         (checks
          (append
           (mapcar
            (lambda (result)
              (soma-lifecycle--check
               (format "%s child exited successfully"
                       (soma-lifecycle--get result "phase"))
               (string= (soma-lifecycle--get result "status") "0")
               (soma-lifecycle--get result "status")))
            child-results)
           (mapcar
            (lambda (result)
              (soma-lifecycle--check
               (format "%s trace was emitted"
                       (soma-lifecycle--get result "phase"))
               (eq (soma-lifecycle--get result "trace_exists") t)
               (soma-lifecycle--get result "out_file")))
            child-results)
           (mapcar
            (lambda (result)
              (let ((trace (soma-lifecycle--get result "trace")))
                (soma-lifecycle--check
                 (format "%s trace checks passed"
                         (soma-lifecycle--get result "phase"))
                 (and (listp trace)
                      (string= (soma-lifecycle--get trace "status") "pass"))
                 (if (listp trace)
                     (soma-lifecycle--get trace "status")
                   "missing"))))
            child-results)
           aggregate-teardown-checks))
         (cold-result (soma-lifecycle--child-by-phase child-results "cold"))
         (warm-result (soma-lifecycle--child-by-phase child-results "warmed"))
         (cold (soma-lifecycle--get cold-result "trace"))
         (warm (soma-lifecycle--get warm-result "trace"))
         (repeat-results
          (seq-filter
           (lambda (result)
             (member (soma-lifecycle--get result "phase")
                     '("cold-repeat-1" "warmed-repeat-1")))
           child-results))
         (summary (soma-lifecycle--alist
                   (cons "schema_version" 1)
                   (cons "kind" "soma_doom_lifecycle_summary")
                   (cons "generated_at" (format-time-string "%FT%TZ" nil t))
                   (cons "status" (if (soma-lifecycle--all-checks-pass-p checks)
                                      "pass"
                                    "fail"))
                   (cons "checks" checks)
                   (cons "children" child-results)
                   (cons "child_count" (length child-results))
                   (cons "repeat_phases" repeat-results)
                   (cons "teardown_probe"
                         (soma-lifecycle--alist
                          (cons "env_var" soma-lifecycle-child-token-env)
                          (cons "status"
                                (if (soma-lifecycle--all-checks-pass-p
                                     aggregate-teardown-checks)
                                    "pass"
                                  "fail"))
                          (cons "child_tokens_pairwise_unique"
                                (soma-lifecycle--bool
                                 (soma-lifecycle--child-tokens-unique-p
                                  child-results)))
                          (cons "phases" token-summaries)))
                   (cons "cold_phase"
                         (if (listp cold)
                             (soma-lifecycle--alist
                              (cons "status" (soma-lifecycle--get cold "status"))
                              (cons "loaded_features"
                                    (soma-lifecycle--get cold "loaded_features"))
                              (cons "capabilities"
                                    (soma-lifecycle--get cold "capabilities")))
                           :json-false))
                   (cons "warmed_phase"
                         (if (listp warm)
                             (soma-lifecycle--alist
                              (cons "status" (soma-lifecycle--get warm "status"))
                              (cons "loaded_features"
                                    (soma-lifecycle--get warm "loaded_features"))
                              (cons "first_use"
                                    (soma-lifecycle--get warm "first_use"))
                              (cons "capabilities"
                                    (soma-lifecycle--get warm "capabilities")))
                           :json-false)))))
    (soma-lifecycle--write-json (expand-file-name "summary.json" out-dir)
                                summary)
    (with-temp-file (expand-file-name "summary.md" out-dir)
      (insert "# Doom Lifecycle Worker Summary\n\n")
      (insert (format "- Generated: %s\n"
                      (soma-lifecycle--get summary "generated_at")))
      (insert (format "- Status: `%s`\n"
                      (soma-lifecycle--get summary "status")))
      (insert "- Runtime: selected CUDA-image-derived Soma Insula rootfs\n")
      (insert "- Emacs: source-built `/opt/soma/emacs/bin/emacs`\n")
      (insert "- Doom core: rootfs-owned `/project/home/.emacs.d`\n")
      (insert "- Doom user config: rootfs-owned `/project/home/.doom.d`\n\n")
      (insert (format "- Children: `%d`\n"
                      (soma-lifecycle--get summary "child_count")))
      (insert (format "- Teardown probe: `%s`\n\n"
                      (soma-lifecycle--get
                       (soma-lifecycle--get summary "teardown_probe")
                       "status")))
      (insert "## Checks\n\n")
      (dolist (check checks)
        (insert (format "- `%s`: `%S` %s\n"
                        (soma-lifecycle--get check "name")
                        (soma-lifecycle--get check "passed")
                        (soma-lifecycle--get check "detail"))))
      (when (listp warm)
        (insert "\n## Warmed First Use\n\n")
        (let ((first-use (soma-lifecycle--get warm "first_use")))
          (dolist (require-result (soma-lifecycle--get first-use "requires"))
            (insert (format "- require `%s`: `%s`\n"
                            (soma-lifecycle--get require-result "feature")
                            (soma-lifecycle--get require-result "status"))))
          (dolist (mode-result (soma-lifecycle--get first-use "mode_activations"))
            (insert (format "- mode `%s`: `%s` -> `%s`\n"
                            (soma-lifecycle--get mode-result "function")
                            (soma-lifecycle--get mode-result "status")
                            (soma-lifecycle--get mode-result "major_mode")))))
        (insert "\n## Capability Module State\n\n")
        (dolist (capability (soma-lifecycle--get warm "capabilities"))
          (insert (format "- `%s`: user-init `%S`, loaded-profile `%S`, text-token `%S`\n"
                          (soma-lifecycle--get capability "name")
                          (soma-lifecycle--get capability
                                               "module_declared_in_user_init")
                          (soma-lifecycle--get capability
                                               "module_active_in_loaded_profile")
                          (soma-lifecycle--get capability
                                               "generated_profile_text_contains_token"))))))
    summary))

(defun soma-lifecycle--main (out-dir)
  (make-directory out-dir t)
  (let* ((children (mapcar
                    (lambda (phase)
                      (soma-lifecycle--run-child phase out-dir))
                    '("cold" "warmed" "cold-repeat-1" "warmed-repeat-1")))
         (summary (soma-lifecycle--summary out-dir children)))
    (unless (string= (soma-lifecycle--get summary "status") "pass")
      (princ (format "Doom lifecycle worker failed; see %s\n" out-dir))
      (kill-emacs 1))))

(defun soma-lifecycle-run ()
  (let ((args (seq-remove (lambda (arg) (string= arg "--"))
                          command-line-args-left)))
    (if (string= (car args) "--child")
        (soma-lifecycle--child-main (cadr args) (caddr args))
      (soma-lifecycle--main
       (expand-file-name
        (or (car args)
            "workers/emacs-doom-lifecycle/traces/latest"))))))

(soma-lifecycle-run)

;;; run-lifecycle.el ends here
