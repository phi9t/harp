;;; run-benchmark.el --- Magit benchmark for the Emacs gym -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'eieio)
(require 'json)
(require 'seq)
(require 'subr-x)

(add-to-list
 'load-path
 (expand-file-name "../../elisp"
                   (file-name-directory (or load-file-name buffer-file-name))))
(require 'soma-gym-bridge)

(defvar soma-magit-benchmark-output-dir nil)
(defvar soma-magit-benchmark-results nil)

(defvar magit-auto-revert-mode)
(defvar magit-display-buffer-function)
(defvar magit-display-buffer-noselect)
(defvar magit-no-confirm)
(defvar magit-post-stage-hook)
(defvar magit-refresh-buffer-hook)
(defvar magit-root-section)

(defvar soma-magit-bench--post-stage-runs nil)
(defvar soma-magit-bench--refresh-runs nil)

(defun soma-magit-bench--alist (&rest pairs)
  pairs)

(defun soma-magit-bench--bool (value)
  (if value t :json-false))

(defun soma-magit-bench--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-magit-bench--read-json-file (path)
  (when (and path (not (string-empty-p path)) (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-magit-bench--plan ()
  (soma-magit-bench--read-json-file (or (getenv "SOMA_INSULA_PLAN") "")))

(defun soma-magit-bench--get (alist key)
  (alist-get key alist nil nil #'string=))

(defun soma-magit-bench--check (name passed detail)
  (soma-magit-bench--alist
   (cons "name" name)
   (cons "passed" (soma-magit-bench--bool passed))
   (cons "detail" detail)))

(defun soma-magit-bench--all-checks-pass-p (checks)
  (cl-every
   (lambda (check)
     (eq (soma-magit-bench--get check "passed") t))
   checks))

(defun soma-magit-bench--doom-straight-build-dir ()
  (expand-file-name
   (format "~/.emacs.d/.local/straight/build-%d.%d"
           emacs-major-version emacs-minor-version)))

(defun soma-magit-bench--add-straight-build-paths ()
  (let ((build (soma-magit-bench--doom-straight-build-dir)))
    (when (file-directory-p build)
      (dolist (dir (directory-files build t directory-files-no-dot-files-regexp))
        (when (file-directory-p dir)
          (add-to-list 'load-path dir))))))

(defun soma-magit-bench--load-magit ()
  (soma-magit-bench--add-straight-build-paths)
  (require 'magit-status)
  (require 'magit-apply)
  (require 'magit-section))

(defun soma-magit-bench--run-process (program &rest args)
  (with-temp-buffer
    (let ((status (apply #'process-file program nil t nil args))
          (output (buffer-substring-no-properties (point-min) (point-max))))
      (soma-magit-bench--alist
       (cons "status" status)
       (cons "output" output)))))

(defun soma-magit-bench--git (repo &rest args)
  (let* ((default-directory (file-name-as-directory repo))
         (result (apply #'soma-magit-bench--run-process "git" args))
         (status (soma-magit-bench--get result "status"))
         (output (soma-magit-bench--get result "output")))
    (unless (equal status 0)
      (error "git %S failed with %S: %s" args status output))
    output))

(defun soma-magit-bench--git-lines (repo &rest args)
  (sort (split-string (string-trim (apply #'soma-magit-bench--git repo args))
                      "\n" t)
        #'string<))

(defun soma-magit-bench--read-file (path)
  (when (file-exists-p path)
    (with-temp-buffer
      (insert-file-contents path)
      (buffer-substring-no-properties (point-min) (point-max)))))

(defun soma-magit-bench--write-file (path text)
  (make-directory (file-name-directory path) t)
  (with-temp-file path
    (insert text)))

(defun soma-magit-bench--target-content (&optional selected other external)
  (let (lines)
    (dotimes (idx 30)
      (let ((line (1+ idx)))
        (push
         (cond
          ((= line 5)
           (if selected "target-05-selected" "target-05-base"))
          ((= line 25)
           (cond
            (external "target-25-external")
            (other "target-25-other")
            (t "target-25-base")))
          (t
           (format "target-%02d-base" line)))
         lines)))
    (concat (string-join (nreverse lines) "\n") "\n")))

(defun soma-magit-bench--index-file (repo file)
  (soma-magit-bench--git repo "show" (concat ":" file)))

(defun soma-magit-bench--worktree-file (repo file)
  (soma-magit-bench--read-file (expand-file-name file repo)))

(defun soma-magit-bench--make-repo (name)
  (let ((repo (make-temp-file (format "soma-magit-%s-" name) t)))
    (soma-magit-bench--git repo "init" "-q")
    (soma-magit-bench--git repo "config" "user.name" "Soma Benchmark")
    (soma-magit-bench--git repo "config" "user.email" "soma-bench@example.invalid")
    (soma-magit-bench--write-file
     (expand-file-name "target.txt" repo)
     (soma-magit-bench--target-content nil nil nil))
    (soma-magit-bench--write-file
     (expand-file-name "debug.txt" repo)
     "debug-base\n")
    (soma-magit-bench--write-file
     (expand-file-name "staged.txt" repo)
     "staged-base\n")
    (soma-magit-bench--write-file
     (expand-file-name "alpha.txt" repo)
     "alpha-base\n")
    (soma-magit-bench--write-file
     (expand-file-name "beta.txt" repo)
     "beta-base\n")
    (soma-magit-bench--write-file
     (expand-file-name "gamma.txt" repo)
     "gamma-base\n")
    (soma-magit-bench--git repo "add" ".")
    (soma-magit-bench--git repo "commit" "-q" "-m" "initial")
    repo))

(defun soma-magit-bench--add-unrelated-state (repo)
  (soma-magit-bench--write-file
   (expand-file-name "staged.txt" repo)
   "staged-expected\n")
  (soma-magit-bench--git repo "add" "staged.txt")
  (soma-magit-bench--write-file
   (expand-file-name "debug.txt" repo)
   "debug-base\ndebug-worktree-only\n"))

(defun soma-magit-bench--setup-target-two-hunks (repo)
  (soma-magit-bench--add-unrelated-state repo)
  (soma-magit-bench--write-file
   (expand-file-name "target.txt" repo)
   (soma-magit-bench--target-content t t nil)))

(defun soma-magit-bench--setup-target-one-hunk (repo)
  (soma-magit-bench--add-unrelated-state repo)
  (soma-magit-bench--write-file
   (expand-file-name "target.txt" repo)
   (soma-magit-bench--target-content t nil nil)))

(defun soma-magit-bench--setup-region-files (repo)
  (soma-magit-bench--add-unrelated-state repo)
  (soma-magit-bench--write-file
   (expand-file-name "alpha.txt" repo)
   "alpha-selected\n")
  (soma-magit-bench--write-file
   (expand-file-name "beta.txt" repo)
   "beta-selected\n")
  (soma-magit-bench--write-file
   (expand-file-name "gamma.txt" repo)
   "gamma-worktree-only\n"))

(defun soma-magit-bench--stale-external-edit (repo)
  (soma-magit-bench--write-file
   (expand-file-name "target.txt" repo)
   (soma-magit-bench--target-content t nil t)))

(defun soma-magit-bench--display-buffer (buffer)
  (set-window-buffer (selected-window) buffer)
  (selected-window))

(defun soma-magit-bench--status-buffer (repo)
  (let ((default-directory (file-name-as-directory repo))
        (magit-display-buffer-function #'soma-magit-bench--display-buffer)
        (magit-display-buffer-noselect nil)
        (magit-auto-revert-mode nil))
    (magit-status-setup-buffer repo)))

(defun soma-magit-bench--section-slot (section slot)
  (eieio-oref section slot))

(defun soma-magit-bench--position-value (value)
  (cond
   ((markerp value)
    (or (marker-position value) :json-false))
   ((integerp value)
    value)
   (t
    :json-false)))

(defun soma-magit-bench--section-parent-types (section)
  (let (types)
    (while section
      (when (soma-magit-bench--section-slot section 'type)
        (push (symbol-name
               (soma-magit-bench--section-slot section 'type))
              types))
      (setq section (soma-magit-bench--section-slot section 'parent)))
    (nreverse types)))

(defun soma-magit-bench--section-has-parent-type-p (section type)
  (let ((cursor (soma-magit-bench--section-slot section 'parent))
        found)
    (while cursor
      (when (eq (soma-magit-bench--section-slot cursor 'type) type)
        (setq found t))
      (setq cursor (soma-magit-bench--section-slot cursor 'parent)))
    found))

(defun soma-magit-bench--section-ancestor-file-p (section file)
  (let ((cursor section)
        found)
    (while cursor
      (when (and (eq (soma-magit-bench--section-slot cursor 'type) 'file)
                 (equal (soma-magit-bench--section-slot cursor 'value)
                        file))
        (setq found t))
      (setq cursor (soma-magit-bench--section-slot cursor 'parent)))
    found))

(defun soma-magit-bench--section-text (section)
  (buffer-substring-no-properties
   (soma-magit-bench--section-slot section 'start)
   (soma-magit-bench--section-slot section 'end)))

(defun soma-magit-bench--section-observation (section)
  (when section
    (let ((text (soma-magit-bench--section-text section)))
      (soma-magit-bench--alist
       (cons "type"
             (format "%S" (soma-magit-bench--section-slot section 'type)))
       (cons "value"
             (format "%S" (soma-magit-bench--section-slot section 'value)))
       (cons "start"
             (soma-magit-bench--position-value
              (soma-magit-bench--section-slot section 'start)))
       (cons "content"
             (soma-magit-bench--position-value
              (soma-magit-bench--section-slot section 'content)))
       (cons "end"
             (soma-magit-bench--position-value
              (soma-magit-bench--section-slot section 'end)))
       (cons "parent_types"
             (soma-magit-bench--section-parent-types section))
       (cons "text_excerpt" (substring text 0 (min 400 (length text))))))))

(defun soma-magit-bench--find-section (buffer predicate)
  (let (found)
    (with-current-buffer buffer
      (magit-map-sections
       (lambda (section)
         (when (and (not found) (funcall predicate section))
           (setq found section)))))
    found))

(defun soma-magit-bench--find-file-section (buffer file &optional parent-type)
  (soma-magit-bench--find-section
   buffer
   (lambda (section)
     (and (eq (soma-magit-bench--section-slot section 'type) 'file)
          (equal (soma-magit-bench--section-slot section 'value) file)
          (or (not parent-type)
              (soma-magit-bench--section-has-parent-type-p
               section parent-type))))))

(defun soma-magit-bench--find-hunk-section (buffer file needle)
  (soma-magit-bench--find-section
   buffer
   (lambda (section)
     (and (eq (soma-magit-bench--section-slot section 'type) 'hunk)
          (soma-magit-bench--section-ancestor-file-p section file)
          (string-match-p (regexp-quote needle)
                          (with-current-buffer buffer
                            (soma-magit-bench--section-text section)))))))

(defun soma-magit-bench--find-typed-section (buffer type)
  (soma-magit-bench--find-section
   buffer
   (lambda (section)
     (eq (soma-magit-bench--section-slot section 'type) type))))

(defun soma-magit-bench--region-section-observation ()
  (mapcar #'soma-magit-bench--section-observation
          (or (magit-region-sections nil t) nil)))

(defun soma-magit-bench--capture-post-stage-hook ()
  (push (format "%S" this-command)
        soma-magit-bench--post-stage-runs))

(defun soma-magit-bench--capture-refresh-hook ()
  (push (buffer-name (current-buffer))
        soma-magit-bench--refresh-runs))

(defun soma-magit-bench--with-capture (thunk)
  (let ((soma-magit-bench--post-stage-runs nil)
        (soma-magit-bench--refresh-runs nil)
        (magit-post-stage-hook '(soma-magit-bench--capture-post-stage-hook))
        (magit-refresh-buffer-hook '(soma-magit-bench--capture-refresh-hook))
        (magit-no-confirm '(stage-all-changes))
        (magit-auto-revert-mode nil))
    (funcall thunk)
    (soma-magit-bench--alist
     (cons "post_stage_hook_runs"
           (nreverse soma-magit-bench--post-stage-runs))
     (cons "refresh_buffer_hook_runs"
           (nreverse soma-magit-bench--refresh-runs)))))

(defun soma-magit-bench--repo-state (repo)
  (soma-magit-bench--alist
   (cons "cached_names"
         (soma-magit-bench--git-lines repo "diff" "--cached" "--name-only"))
   (cons "unstaged_names"
         (soma-magit-bench--git-lines repo "diff" "--name-only"))
   (cons "status_porcelain"
         (soma-magit-bench--git-lines repo "status" "--porcelain=v1"))
   (cons "target_index"
         (ignore-errors (soma-magit-bench--index-file repo "target.txt")))
   (cons "target_worktree"
         (soma-magit-bench--worktree-file repo "target.txt"))
   (cons "debug_index"
         (ignore-errors (soma-magit-bench--index-file repo "debug.txt")))
   (cons "debug_worktree"
         (soma-magit-bench--worktree-file repo "debug.txt"))
   (cons "staged_index"
         (ignore-errors (soma-magit-bench--index-file repo "staged.txt")))
   (cons "staged_worktree"
         (soma-magit-bench--worktree-file repo "staged.txt"))
   (cons "alpha_index"
         (ignore-errors (soma-magit-bench--index-file repo "alpha.txt")))
   (cons "beta_index"
         (ignore-errors (soma-magit-bench--index-file repo "beta.txt")))
   (cons "gamma_index"
         (ignore-errors (soma-magit-bench--index-file repo "gamma.txt")))))

(defun soma-magit-bench--string-contains-p (string needle)
  (and (stringp string)
       (string-match-p (regexp-quote needle) string)))

(defun soma-magit-bench--file-content-checks (state)
  (let ((target-index (soma-magit-bench--get state "target_index"))
        (target-worktree (soma-magit-bench--get state "target_worktree"))
        (debug-index (soma-magit-bench--get state "debug_index"))
        (debug-worktree (soma-magit-bench--get state "debug_worktree"))
        (staged-index (soma-magit-bench--get state "staged_index")))
    (list
     (soma-magit-bench--check
      "target selected change is staged"
      (soma-magit-bench--string-contains-p target-index
                                           "target-05-selected")
      (or target-index ""))
     (soma-magit-bench--check
      "target unrelated hunk remains unstaged"
      (and (soma-magit-bench--string-contains-p target-index
                                                "target-25-base")
           (soma-magit-bench--string-contains-p target-worktree
                                                "target-25-other"))
      (format "index=%S worktree=%S" target-index target-worktree))
     (soma-magit-bench--check
      "debug worktree change remains unstaged"
      (and (string= debug-index "debug-base\n")
           (soma-magit-bench--string-contains-p
            debug-worktree "debug-worktree-only"))
      (format "index=%S worktree=%S" debug-index debug-worktree))
     (soma-magit-bench--check
      "unrelated staged file is preserved"
      (string= staged-index "staged-expected\n")
      (or staged-index "")))))

(defun soma-magit-bench--faithful-command-checks (action capture
                                                        &optional
                                                        post-stage-optional)
  (let ((post-stage-runs
         (soma-magit-bench--get capture "post_stage_hook_runs"))
        (refresh-runs
         (soma-magit-bench--get capture "refresh_buffer_hook_runs"))
        (bridge
         (soma-magit-bench--get action "bridge")))
    (list
     (soma-magit-bench--check
      "action used expected Magit operation"
      (string= (soma-magit-bench--get action "operation") "magit-stage")
      (format "%S" action))
     (soma-magit-bench--check
      "checked bridge command dispatch succeeded"
      (string= (soma-magit-bench--get bridge "state") "succeeded")
      (format "%S" bridge))
     (if post-stage-optional
         (soma-magit-bench--check
          "post-stage hook status recorded"
          t
          (format "%S" post-stage-runs))
       (soma-magit-bench--check
        "command route preserved Magit command identity"
        (member "magit-stage" post-stage-runs)
        (format "%S" post-stage-runs)))
     (soma-magit-bench--check
      "Magit refresh ran after staging"
      (> (length refresh-runs) 0)
      (format "%S" refresh-runs)))))

(defun soma-magit-bench--common-tool-checks ()
  (let ((git (file-truename (or (executable-find "git") "")))
        (magit-apply (or (locate-library "magit-apply") "")))
    (list
     (soma-magit-bench--check
      "git executable resolves inside rootfs"
      (and (or (string-prefix-p "/usr/" git)
               (string-prefix-p "/bin/" git)
               (string-prefix-p "/usr/local/" git))
           (not (string-prefix-p "/data02/home/" git))
           (not (string-prefix-p "/nix/" git)))
      git)
     (soma-magit-bench--check
      "Magit loads from rootfs-owned Doom mount"
      (and (string-prefix-p "/project/home/.emacs.d/.local/straight/"
                            magit-apply)
           (not (string-prefix-p "/data02/home/" magit-apply))
           (not (string-prefix-p "/nix/" magit-apply)))
      magit-apply))))

(defun soma-magit-bench--cleanup (repo)
  (dolist (buffer (buffer-list))
    (when (with-current-buffer buffer
            (or (derived-mode-p 'magit-mode)
                (string-prefix-p "magit" (buffer-name buffer))
                (string-prefix-p "*magit" (buffer-name buffer))))
      (kill-buffer buffer)))
  (when (and repo (file-directory-p repo))
    (delete-directory repo t)))

(defun soma-magit-bench--base-result (name observation action state capture checks)
  (append
   (soma-magit-bench--alist
    (cons "observation" observation)
    (cons "action" action)
    (cons "name" name)
    (cons "checks" checks)
    (cons "repo_state" state))
   capture))

(defun soma-magit-bench--bridge-preconditions (buffer)
  (with-current-buffer buffer
    (soma-magit-bench--alist
     (cons "buffer_name" (buffer-name buffer))
     (cons "selected_window_buffer" (buffer-name buffer))
     (cons "chars_tick" (buffer-chars-modified-tick))
     (cons "point" (point))
     (cons "mark_active" (soma-magit-bench--bool mark-active))
     (cons "region_begin" (if mark-active (region-beginning) :json-false))
     (cons "region_end" (if mark-active (region-end) :json-false))
     (cons "minibuffer_active" :json-false))))

(defun soma-magit-bench--stage-section-command (buffer section)
  (with-current-buffer buffer
    (goto-char (soma-magit-bench--section-slot section 'start))
    (soma-gym-bridge-command-dispatch
     buffer 'magit-stage (soma-magit-bench--bridge-preconditions buffer))))

(defun soma-magit-bench--scenario-hunk-correct ()
  (let ((repo (soma-magit-bench--make-repo "hunk-correct")))
    (unwind-protect
        (let (buffer section observation action bridge capture state checks)
          (soma-magit-bench--setup-target-two-hunks repo)
          (setq buffer (soma-magit-bench--status-buffer repo))
          (with-current-buffer buffer
            (setq section
                  (soma-magit-bench--find-hunk-section
                   buffer "target.txt" "target-05-selected"))
            (unless section
              (error "could not locate target hunk section"))
            (goto-char (soma-magit-bench--section-slot section 'start))
            (setq observation
                  (soma-magit-bench--alist
                   (cons "repo" repo)
                   (cons "buffer" (buffer-name buffer))
                   (cons "selected_section"
                         (soma-magit-bench--section-observation section))
                   (cons "diff_type" (format "%S" (magit-diff-type)))
                   (cons "diff_scope" (format "%S" (magit-diff-scope))))))
          (setq capture
                (soma-magit-bench--with-capture
                 (lambda ()
                   (setq bridge
                         (soma-magit-bench--stage-section-command
                          buffer section)))))
          (setq action
                (soma-magit-bench--alist
                 (cons "operation" "magit-stage")
                 (cons "route" "checked-bridge-command-dispatch")
                 (cons "bridge" bridge)
                 (cons "selection" "unstaged hunk")))
          (setq state (soma-magit-bench--repo-state repo))
          (setq checks
                (append
                 (soma-magit-bench--common-tool-checks)
                 (soma-magit-bench--faithful-command-checks action capture)
                 (soma-magit-bench--file-content-checks state)
                 (list
                  (soma-magit-bench--check
                   "cached names are exact"
                   (equal (soma-magit-bench--get state "cached_names")
                          '("staged.txt" "target.txt"))
                   (format "%S" (soma-magit-bench--get state "cached_names")))
                  (soma-magit-bench--check
                   "unstaged names preserve unrelated work"
                   (equal (soma-magit-bench--get state "unstaged_names")
                          '("debug.txt" "target.txt"))
                   (format "%S" (soma-magit-bench--get state
                                                       "unstaged_names"))))))
          (soma-magit-bench--base-result
           "correct-hunk-stage" observation action state capture checks))
      (soma-magit-bench--cleanup repo))))

(defun soma-magit-bench--scenario-file-correct ()
  (let ((repo (soma-magit-bench--make-repo "file-correct")))
    (unwind-protect
        (let (buffer section observation action bridge capture state checks)
          (soma-magit-bench--setup-target-one-hunk repo)
          (setq buffer (soma-magit-bench--status-buffer repo))
          (with-current-buffer buffer
            (setq section
                  (soma-magit-bench--find-file-section
                   buffer "target.txt" 'unstaged))
            (unless section
              (error "could not locate target file section"))
            (goto-char (soma-magit-bench--section-slot section 'start))
            (setq observation
                  (soma-magit-bench--alist
                   (cons "repo" repo)
                   (cons "buffer" (buffer-name buffer))
                   (cons "selected_section"
                         (soma-magit-bench--section-observation section))
                   (cons "diff_type" (format "%S" (magit-diff-type)))
                   (cons "diff_scope" (format "%S" (magit-diff-scope))))))
          (setq capture
                (soma-magit-bench--with-capture
                 (lambda ()
                   (setq bridge
                         (soma-magit-bench--stage-section-command
                          buffer section)))))
          (setq action
                (soma-magit-bench--alist
                 (cons "operation" "magit-stage")
                 (cons "route" "checked-bridge-command-dispatch")
                 (cons "bridge" bridge)
                 (cons "selection" "unstaged file")))
          (setq state (soma-magit-bench--repo-state repo))
          (setq checks
                (append
                 (soma-magit-bench--common-tool-checks)
                 (soma-magit-bench--faithful-command-checks action capture)
                 (list
                  (soma-magit-bench--check
                   "target file is fully staged"
                   (and (soma-magit-bench--string-contains-p
                         (soma-magit-bench--get state "target_index")
                         "target-05-selected")
                        (not (member "target.txt"
                                     (soma-magit-bench--get
                                      state "unstaged_names"))))
                   (format "index=%S unstaged=%S"
                           (soma-magit-bench--get state "target_index")
                           (soma-magit-bench--get state "unstaged_names")))
                  (soma-magit-bench--check
                   "cached names are exact"
                   (equal (soma-magit-bench--get state "cached_names")
                          '("staged.txt" "target.txt"))
                   (format "%S" (soma-magit-bench--get state "cached_names")))
                  (soma-magit-bench--check
                   "unrelated unstaged debug change remains"
                   (equal (soma-magit-bench--get state "unstaged_names")
                          '("debug.txt"))
                   (format "%S" (soma-magit-bench--get state
                                                       "unstaged_names")))
                  (soma-magit-bench--check
                   "unrelated staged file is preserved"
                   (string= (soma-magit-bench--get state "staged_index")
                            "staged-expected\n")
                   (or (soma-magit-bench--get state "staged_index") "")))))
          (soma-magit-bench--base-result
           "correct-file-stage" observation action state capture checks))
      (soma-magit-bench--cleanup repo))))

(defun soma-magit-bench--scenario-region-correct ()
  (let ((repo (soma-magit-bench--make-repo "region-correct")))
    (unwind-protect
        (let (buffer alpha beta observation action bridge capture state checks
                     region-sections)
          (soma-magit-bench--setup-region-files repo)
          (setq buffer (soma-magit-bench--status-buffer repo))
          (with-current-buffer buffer
            (setq alpha
                  (soma-magit-bench--find-file-section
                   buffer "alpha.txt" 'unstaged))
            (setq beta
                  (soma-magit-bench--find-file-section
                   buffer "beta.txt" 'unstaged))
            (unless (and alpha beta)
              (error "could not locate alpha/beta file sections"))
            (goto-char (soma-magit-bench--section-slot beta 'start))
            (set-mark (soma-magit-bench--section-slot alpha 'start))
            (setq mark-active t
                  transient-mark-mode t)
            (setq region-sections
                  (soma-magit-bench--region-section-observation))
            (setq observation
                  (soma-magit-bench--alist
                   (cons "repo" repo)
                   (cons "buffer" (buffer-name buffer))
                   (cons "region_sections" region-sections)
                   (cons "diff_type" (format "%S" (magit-diff-type)))
                   (cons "diff_scope" (format "%S" (magit-diff-scope))))))
          (setq capture
                (soma-magit-bench--with-capture
                 (lambda ()
                   (with-current-buffer buffer
                     (goto-char (soma-magit-bench--section-slot beta 'start))
                     (set-mark (soma-magit-bench--section-slot alpha 'start))
                     (setq mark-active t
                           transient-mark-mode t)
                     (setq bridge
                           (soma-gym-bridge-command-dispatch
                            buffer 'magit-stage
                            (soma-magit-bench--bridge-preconditions
                             buffer)))))))
          (setq action
                (soma-magit-bench--alist
                 (cons "operation" "magit-stage")
                 (cons "route" "checked-bridge-command-dispatch")
                 (cons "bridge" bridge)
                 (cons "selection" "unstaged files region")))
          (setq state (soma-magit-bench--repo-state repo))
          (setq checks
                (append
                 (soma-magit-bench--common-tool-checks)
                 (soma-magit-bench--faithful-command-checks action capture)
                 (list
                  (soma-magit-bench--check
                   "region selected exactly two file sections"
                   (= (length region-sections) 2)
                   (format "%S" region-sections))
                  (soma-magit-bench--check
                   "alpha and beta are staged"
                   (and (string= (soma-magit-bench--get state "alpha_index")
                                 "alpha-selected\n")
                        (string= (soma-magit-bench--get state "beta_index")
                                 "beta-selected\n"))
                   (format "alpha=%S beta=%S"
                           (soma-magit-bench--get state "alpha_index")
                           (soma-magit-bench--get state "beta_index")))
                  (soma-magit-bench--check
                   "gamma remains unstaged"
                   (and (string= (soma-magit-bench--get state "gamma_index")
                                 "gamma-base\n")
                        (member "gamma.txt"
                                (soma-magit-bench--get state
                                                       "unstaged_names")))
                   (format "gamma=%S unstaged=%S"
                           (soma-magit-bench--get state "gamma_index")
                           (soma-magit-bench--get state "unstaged_names")))
                  (soma-magit-bench--check
                   "cached names are exact"
                   (equal (soma-magit-bench--get state "cached_names")
                          '("alpha.txt" "beta.txt" "staged.txt"))
                   (format "%S" (soma-magit-bench--get state "cached_names")))
                  (soma-magit-bench--check
                   "unrelated debug and gamma worktree changes remain"
                   (equal (soma-magit-bench--get state "unstaged_names")
                          '("debug.txt" "gamma.txt"))
                   (format "%S" (soma-magit-bench--get state
                                                       "unstaged_names"))))))
          (soma-magit-bench--base-result
           "correct-region-files-stage" observation action state capture checks))
      (soma-magit-bench--cleanup repo))))

(defun soma-magit-bench--scenario-git-add-all-bad ()
  (let ((repo (soma-magit-bench--make-repo "git-add-all-bad")))
    (unwind-protect
        (let (observation action state checks)
          (soma-magit-bench--setup-target-two-hunks repo)
          (setq observation
                (soma-magit-bench--alist
                 (cons "repo" repo)
                 (cons "intended_selection" "first target.txt hunk")
                 (cons "visible_state" (soma-magit-bench--repo-state repo))))
          (let ((default-directory (file-name-as-directory repo)))
            (call-process "git" nil nil nil "add" "-u"))
          (setq action
                (soma-magit-bench--alist
                 (cons "operation" "git add -u")
                 (cons "route" "raw git subprocess")
                 (cons "bridge"
                       (soma-gym-bridge-reject-direct-call
                        'git-add-u
                        "raw git subprocess is outside checked bridge command dispatch"))
                 (cons "selection" "all tracked changes")))
          (setq state (soma-magit-bench--repo-state repo))
          (setq checks
                (append
                 (soma-magit-bench--common-tool-checks)
                 (soma-magit-bench--faithful-command-checks action nil)
                 (soma-magit-bench--file-content-checks state)
                 (list
                  (soma-magit-bench--check
                   "cached names are exact"
                   (equal (soma-magit-bench--get state "cached_names")
                          '("staged.txt" "target.txt"))
                   (format "%S" (soma-magit-bench--get state "cached_names")))
                  (soma-magit-bench--check
                   "unstaged names preserve unrelated work"
                   (equal (soma-magit-bench--get state "unstaged_names")
                          '("debug.txt" "target.txt"))
                   (format "%S" (soma-magit-bench--get state
                                                       "unstaged_names"))))))
          (soma-magit-bench--base-result
           "bad-git-add-all" observation action state
           (soma-magit-bench--alist
            (cons "post_stage_hook_runs" nil)
            (cons "refresh_buffer_hook_runs" nil))
           checks))
      (soma-magit-bench--cleanup repo))))

(defun soma-magit-bench--scenario-wrapper-context-bad ()
  (let ((repo (soma-magit-bench--make-repo "wrapper-context-bad")))
    (unwind-protect
        (let (observation action capture state checks)
          (soma-magit-bench--setup-target-one-hunk repo)
          (setq observation
                (soma-magit-bench--alist
                 (cons "repo" repo)
                 (cons "intended_selection" "target.txt file section")
                 (cons "visible_state" (soma-magit-bench--repo-state repo))))
          (setq capture
                (soma-magit-bench--with-capture
                 (lambda ()
                   (let ((default-directory (file-name-as-directory repo))
                         (this-command 'soma-bridge-stage))
                     (magit-stage-1 "-u" (list "target.txt"))))))
          (setq action
                (soma-magit-bench--alist
                 (cons "operation" "magit-stage-1")
                 (cons "route" "bridge-wrapper-direct-call")
                 (cons "bridge"
                       (soma-gym-bridge-reject-direct-call
                        'magit-stage-1
                        "direct lower-level Magit mutation bypasses checked command dispatch"))
                 (cons "selection" "target.txt")
                 (cons "wrapper_this_command" "soma-bridge-stage")))
          (setq state (soma-magit-bench--repo-state repo))
          (setq checks
                (append
                 (soma-magit-bench--common-tool-checks)
                 (soma-magit-bench--faithful-command-checks action capture)
                 (list
                  (soma-magit-bench--check
                   "filesystem effect matches requested file staging"
                   (and (equal (soma-magit-bench--get state "cached_names")
                               '("staged.txt" "target.txt"))
                        (not (member "target.txt"
                                     (soma-magit-bench--get
                                      state "unstaged_names"))))
                   (format "cached=%S unstaged=%S"
                           (soma-magit-bench--get state "cached_names")
                           (soma-magit-bench--get state "unstaged_names")))
                  (soma-magit-bench--check
                   "unrelated staged file is preserved"
                   (string= (soma-magit-bench--get state "staged_index")
                            "staged-expected\n")
                   (or (soma-magit-bench--get state "staged_index") "")))))
          (soma-magit-bench--base-result
           "bad-wrapper-command-context" observation action state capture checks))
      (soma-magit-bench--cleanup repo))))

(defun soma-magit-bench--scenario-list-stage-bad ()
  (let ((repo (soma-magit-bench--make-repo "list-stage-bad")))
    (unwind-protect
        (let (buffer section observation action bridge capture state checks)
          (soma-magit-bench--setup-target-two-hunks repo)
          (setq buffer (soma-magit-bench--status-buffer repo))
          (with-current-buffer buffer
            (setq section
                  (soma-magit-bench--find-typed-section buffer 'unstaged))
            (unless section
              (error "could not locate unstaged list section"))
            (goto-char (soma-magit-bench--section-slot section 'start))
            (setq observation
                  (soma-magit-bench--alist
                   (cons "repo" repo)
                   (cons "buffer" (buffer-name buffer))
                   (cons "selected_section"
                         (soma-magit-bench--section-observation section))
                   (cons "diff_type" (format "%S" (magit-diff-type)))
                   (cons "diff_scope" (format "%S" (magit-diff-scope))))))
          (setq capture
                (soma-magit-bench--with-capture
                 (lambda ()
                   (setq bridge
                         (soma-magit-bench--stage-section-command
                          buffer section)))))
          (setq action
                (soma-magit-bench--alist
                 (cons "operation" "magit-stage")
                 (cons "route" "checked-bridge-command-dispatch")
                 (cons "bridge" bridge)
                 (cons "selection" "unstaged list")))
          (setq state (soma-magit-bench--repo-state repo))
          (setq checks
                (append
                 (soma-magit-bench--common-tool-checks)
                 (soma-magit-bench--faithful-command-checks action capture)
                 (soma-magit-bench--file-content-checks state)
                 (list
                  (soma-magit-bench--check
                   "cached names are exact"
                   (equal (soma-magit-bench--get state "cached_names")
                          '("staged.txt" "target.txt"))
                   (format "%S" (soma-magit-bench--get state "cached_names")))
                  (soma-magit-bench--check
                   "unstaged names preserve unrelated work"
                   (equal (soma-magit-bench--get state "unstaged_names")
                          '("debug.txt" "target.txt"))
                   (format "%S" (soma-magit-bench--get state
                                                       "unstaged_names"))))))
          (soma-magit-bench--base-result
           "bad-unstaged-list-stage" observation action state capture checks))
      (soma-magit-bench--cleanup repo))))

(defun soma-magit-bench--scenario-stale-view-bad ()
  (let ((repo (soma-magit-bench--make-repo "stale-view-bad")))
    (unwind-protect
        (let (buffer section observation action bridge capture state checks)
          (soma-magit-bench--setup-target-one-hunk repo)
          (setq buffer (soma-magit-bench--status-buffer repo))
          (with-current-buffer buffer
            (setq section
                  (soma-magit-bench--find-file-section
                   buffer "target.txt" 'unstaged))
            (unless section
              (error "could not locate target file section"))
            (goto-char (soma-magit-bench--section-slot section 'start))
            (setq observation
                  (soma-magit-bench--alist
                   (cons "repo" repo)
                   (cons "buffer" (buffer-name buffer))
                   (cons "selected_section"
                         (soma-magit-bench--section-observation section))
                   (cons "view_refreshed_after_observation" :json-false))))
          (soma-magit-bench--stale-external-edit repo)
          (setq capture
                (soma-magit-bench--with-capture
                 (lambda ()
                   (setq bridge
                         (soma-magit-bench--stage-section-command
                          buffer section)))))
          (setq action
                (soma-magit-bench--alist
                 (cons "operation" "magit-stage")
                 (cons "route" "checked-bridge-command-dispatch")
                 (cons "bridge" bridge)
                 (cons "selection" "stale unstaged file section")
                 (cons "revalidated_after_external_change" :json-false)))
          (setq state (soma-magit-bench--repo-state repo))
          (setq checks
                (append
                 (soma-magit-bench--common-tool-checks)
                 (soma-magit-bench--faithful-command-checks action capture)
                 (list
                  (soma-magit-bench--check
                   "policy revalidated stale view before staging"
                   (eq (soma-magit-bench--get
                        action "revalidated_after_external_change")
                       t)
                   (format "%S" action))
                  (soma-magit-bench--check
                   "index excludes external change"
                   (and (soma-magit-bench--string-contains-p
                         (soma-magit-bench--get state "target_index")
                         "target-25-base")
                        (soma-magit-bench--string-contains-p
                         (soma-magit-bench--get state "target_worktree")
                         "target-25-external"))
                   (format "index=%S worktree=%S"
                           (soma-magit-bench--get state "target_index")
                           (soma-magit-bench--get state "target_worktree")))
                  (soma-magit-bench--check
                   "cached names are exact"
                   (equal (soma-magit-bench--get state "cached_names")
                          '("staged.txt" "target.txt"))
                   (format "%S" (soma-magit-bench--get state
                                                       "cached_names"))))))
          (soma-magit-bench--base-result
           "bad-stale-view-stage" observation action state capture checks))
      (soma-magit-bench--cleanup repo))))

(defun soma-magit-bench--record-scenario (index id title expected-pass thunk)
  (let* ((result (funcall thunk))
         (checks (soma-magit-bench--get result "checks"))
         (policy-pass (soma-magit-bench--all-checks-pass-p checks))
         (as-expected (if expected-pass policy-pass (not policy-pass)))
         (record
          (append
           (soma-magit-bench--alist
            (cons "id" id)
            (cons "title" title)
            (cons "expected_policy_pass" (soma-magit-bench--bool expected-pass))
            (cons "policy_passed_grader" (soma-magit-bench--bool policy-pass))
            (cons "as_expected" (soma-magit-bench--bool as-expected)))
           result)))
    (push record soma-magit-benchmark-results)
    (soma-magit-bench--write-json
     (expand-file-name (format "%02d-%s.json" index id)
                       soma-magit-benchmark-output-dir)
     record)
    record))

(defun soma-magit-bench--insula-checks ()
  (let* ((plan (soma-magit-bench--plan))
         (mode (or (getenv "SOMA_INSULA_MODE") ""))
         (plan-mode (or (soma-magit-bench--get plan "mode") ""))
         (closure (or (soma-magit-bench--get plan "dependency_closure") ""))
         (glibc-owner (or (soma-magit-bench--get plan "glibc_owner") ""))
         (benchmark (or (soma-magit-bench--get plan "benchmark") "")))
    (list
     (soma-magit-bench--check
      "SOMA_INSULA_ACTIVE is set"
      (string= (or (getenv "SOMA_INSULA_ACTIVE") "") "1")
      (or (getenv "SOMA_INSULA_ACTIVE") ""))
     (soma-magit-bench--check
      "runtime mode is selected rootfs"
      (and (string= mode "selected-container-rootfs")
           (string= plan-mode "selected-container-rootfs-insula"))
      (format "env=%S plan=%S" mode plan-mode))
     (soma-magit-bench--check
      "benchmark plan is for Magit"
      (string= benchmark "magit-selective-staging")
      benchmark)
     (soma-magit-bench--check
      "dependency closure is rootfs-owned"
      (string= closure "rootfs_owned")
      closure)
     (soma-magit-bench--check
      "glibc owner is container-image-rootfs"
      (string= glibc-owner "container-image-rootfs")
      glibc-owner)
     (soma-magit-bench--check
      "cwd is rootfs repo mount"
      (string= (directory-file-name (expand-file-name default-directory))
               "/workspace/soma")
      default-directory))))

(defun soma-magit-bench--summary ()
  (let* ((records (reverse soma-magit-benchmark-results))
         (total (length records))
         (expected (cl-count-if
                    (lambda (record)
                      (eq (soma-magit-bench--get record "as_expected") t))
                    records))
         (insula-checks (soma-magit-bench--insula-checks))
         (insula-pass (soma-magit-bench--all-checks-pass-p insula-checks))
         (summary
          (soma-magit-bench--alist
           (cons "generated_at" (format-time-string "%FT%TZ" nil t))
           (cons "benchmark" "magit-selective-staging")
           (cons "total_scenarios" total)
           (cons "scenarios_as_expected" expected)
           (cons "insula_checks_passed" (soma-magit-bench--bool insula-pass))
           (cons "insula_checks" insula-checks)
           (cons "plan" (soma-magit-bench--plan))
           (cons "records" records))))
    (soma-magit-bench--write-json
     (expand-file-name "summary.json" soma-magit-benchmark-output-dir)
     summary)
    (with-temp-file (expand-file-name "summary.md" soma-magit-benchmark-output-dir)
      (insert "# Magit Benchmark Summary\n\n")
      (insert (format "- Generated: %s\n" (format-time-string "%FT%TZ" nil t)))
      (insert "- Benchmark: `magit-selective-staging`\n")
      (insert (format "- Total scenarios: %d\n" total))
      (insert (format "- Scenarios as expected: %d\n" expected))
      (insert (format "- Insula checks passed: `%S`\n" insula-pass))
      (insert (format "- Mode: `%s`\n" (or (getenv "SOMA_INSULA_MODE") "")))
      (insert (format "- Run id: `%s`\n\n" (or (getenv "SOMA_INSULA_RUN_ID") "")))
      (dolist (record records)
        (insert (format "## %s\n\n" (soma-magit-bench--get record "title")))
        (insert (format "- ID: `%s`\n" (soma-magit-bench--get record "id")))
        (insert (format "- Expected policy pass: `%S`\n"
                        (soma-magit-bench--get record
                                               "expected_policy_pass")))
        (insert (format "- Policy passed grader: `%S`\n"
                        (soma-magit-bench--get record
                                               "policy_passed_grader")))
        (insert (format "- As expected: `%S`\n\n"
                        (soma-magit-bench--get record "as_expected")))))
    summary))

(defun soma-magit-bench-run ()
  (let ((args (seq-remove (lambda (arg) (string= arg "--"))
                          command-line-args-left)))
    (setq soma-magit-benchmark-output-dir
          (expand-file-name
           (or (car args)
               "benchmarks/emacs-gym/magit/traces/latest"))))
  (make-directory soma-magit-benchmark-output-dir t)
  (setq soma-magit-benchmark-results nil)
  (soma-magit-bench--load-magit)
  (soma-magit-bench--record-scenario
   1 "correct-hunk-stage"
   "Correct policy stages only the selected Magit hunk"
   t
   #'soma-magit-bench--scenario-hunk-correct)
  (soma-magit-bench--record-scenario
   2 "correct-file-stage"
   "Correct policy stages the selected Magit file section"
   t
   #'soma-magit-bench--scenario-file-correct)
  (soma-magit-bench--record-scenario
   3 "correct-region-files-stage"
   "Correct policy stages an active region of Magit file sections"
   t
   #'soma-magit-bench--scenario-region-correct)
  (soma-magit-bench--record-scenario
   4 "bad-git-add-all"
   "Incorrect policy bypasses Magit and stages too much with git add -u"
   nil
   #'soma-magit-bench--scenario-git-add-all-bad)
  (soma-magit-bench--record-scenario
   5 "bad-wrapper-command-context"
   "Incorrect policy stages the right file through a bridge wrapper"
   nil
   #'soma-magit-bench--scenario-wrapper-context-bad)
  (soma-magit-bench--record-scenario
   6 "bad-unstaged-list-stage"
   "Incorrect policy selects the unstaged list and stages too much"
   nil
   #'soma-magit-bench--scenario-list-stage-bad)
  (soma-magit-bench--record-scenario
   7 "bad-stale-view-stage"
   "Incorrect policy acts through a stale Magit view after external edit"
   nil
   #'soma-magit-bench--scenario-stale-view-bad)
  (let* ((summary (soma-magit-bench--summary))
         (total (soma-magit-bench--get summary "total_scenarios"))
         (expected (soma-magit-bench--get summary "scenarios_as_expected"))
         (insula-pass (eq (soma-magit-bench--get summary
                                                 "insula_checks_passed")
                          t)))
    (unless (and (= total expected) insula-pass)
      (princ
       (format "magit-benchmark: failed total=%s expected=%s insula=%S\n"
               total expected insula-pass))
      (kill-emacs 1))
    (princ
     (format "magit-benchmark: wrote results to %s\n"
             soma-magit-benchmark-output-dir))))

(soma-magit-bench-run)

;;; run-benchmark.el ends here
