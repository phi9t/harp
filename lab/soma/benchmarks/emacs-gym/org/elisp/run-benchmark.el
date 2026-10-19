;;; run-benchmark.el --- Org benchmark for the Emacs gym -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'json)
(require 'seq)
(require 'subr-x)

(defvar soma-org-benchmark-output-dir nil)
(defvar soma-org-benchmark-results nil)

(defun soma-org-bench--alist (&rest pairs)
  pairs)

(defun soma-org-bench--bool (value)
  (if value t :json-false))

(defun soma-org-bench--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-org-bench--read-json-file (path)
  (when (and path (not (string-empty-p path)) (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-org-bench--plan ()
  (soma-org-bench--read-json-file (or (getenv "SOMA_INSULA_PLAN") "")))

(defun soma-org-bench--get (alist key)
  (alist-get key alist nil nil #'string=))

(defun soma-org-bench--check (name passed detail)
  (soma-org-bench--alist
   (cons "name" name)
   (cons "passed" (soma-org-bench--bool passed))
   (cons "detail" detail)))

(defun soma-org-bench--all-checks-pass-p (checks)
  (cl-every
   (lambda (check)
     (eq (soma-org-bench--get check "passed") t))
   checks))

(defun soma-org-bench--doom-straight-build-dir ()
  (expand-file-name
   (format "~/.emacs.d/.local/straight/build-%d.%d"
           emacs-major-version emacs-minor-version)))

(defun soma-org-bench--add-straight-build-paths ()
  (let ((build (soma-org-bench--doom-straight-build-dir)))
    (when (file-directory-p build)
      (dolist (dir (directory-files build t directory-files-no-dot-files-regexp))
        (when (file-directory-p dir)
          (add-to-list 'load-path dir))))))

(defun soma-org-bench--load-org ()
  (soma-org-bench--add-straight-build-paths)
  (require 'org)
  (require 'org-src)
  (require 'org-refile)
  (require 'ob-core)
  (require 'ob-emacs-lisp))

(defun soma-org-bench--write-file (path text)
  (make-directory (file-name-directory path) t)
  (with-temp-file path
    (insert text)))

(defun soma-org-bench--read-file (path)
  (with-temp-buffer
    (insert-file-contents path)
    (buffer-substring-no-properties (point-min) (point-max))))

(defun soma-org-bench--count-regexp (regexp text)
  (let ((start 0)
        (count 0))
    (while (string-match regexp text start)
      (setq count (1+ count))
      (setq start (match-end 0)))
    count))

(defun soma-org-bench--source-file-text ()
  "* Notebook\n#+begin_src emacs-lisp\n(+ 1 2)\n#+end_src\n")

(defun soma-org-bench--refile-file-text ()
  (concat
   "* Inbox\n"
   "** TODO Train model\n"
   ":PROPERTIES:\n"
   ":ID: soma-task-1\n"
   ":CUSTOM_ID: keep-me\n"
   ":END:\n"
   "Body\n"
   "*** Child\n"
   "child body\n"
   "* Archive\n"
   "** Existing\n"
   "old\n"))

(defun soma-org-bench--babel-file-text ()
  (concat
   "* Run\n"
   "#+begin_src emacs-lisp :results output replace\n"
   "(princ (format \"SOMA_ORG_RUN=%s\\nSOMA_ORG_DIR=%s\\n\" "
   "(getenv \"SOMA_INSULA_RUN_ID\") default-directory))\n"
   "#+end_src\n"))

(defun soma-org-bench--source-observation (parent edit)
  (soma-org-bench--alist
   (cons "parent_buffer" (buffer-name parent))
   (cons "edit_buffer" (and (buffer-live-p edit) (buffer-name edit)))
   (cons "same_buffer" (soma-org-bench--bool (eq parent edit)))
   (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
   (cons "plan" (or (getenv "SOMA_INSULA_PLAN") ""))))

(defun soma-org-bench--grade-source-edit (name action before after-edit after-final parent edit)
  (let ((checks
         (list
          (soma-org-bench--check
           "action used org-edit-src-exit"
           (string= (soma-org-bench--get action "operation") "org-edit-src-exit")
           (format "%S" action))
          (soma-org-bench--check
           "source edit buffer is separate"
           (eq (soma-org-bench--get action "source_edit_buffer_was_separate")
               t)
           (format "%S" action))
          (soma-org-bench--check
           "parent unchanged before source-edit exit"
           (string= before after-edit)
           after-edit)
          (soma-org-bench--check
           "parent updated after source-edit exit"
           (and (string-match-p (regexp-quote "(+ 3 4)") after-final)
                (not (string-match-p (regexp-quote "(+ 1 2)") after-final)))
           after-final))))
    (soma-org-bench--alist
     (cons "name" name)
     (cons "checks" checks)
     (cons "parent_text_before" before)
     (cons "parent_text_after_edit" after-edit)
     (cons "parent_text_after_final" after-final))))

(defun soma-org-bench--scenario-source-edit-correct ()
  (soma-org-bench--load-org)
  (let* ((tmp (make-temp-file "soma-org-src-correct-" t))
         (file (expand-file-name "notebook.org" tmp))
         parent edit before after-edit after-final action result)
    (unwind-protect
        (progn
          (soma-org-bench--write-file file (soma-org-bench--source-file-text))
          (setq parent (find-file-noselect file))
          (with-current-buffer parent
            (org-mode)
            (goto-char (point-min))
            (search-forward "(+ 1 2)")
            (setq before (buffer-string))
            (org-edit-src-code)
            (setq edit (current-buffer)))
          (with-current-buffer edit
            (erase-buffer)
            (insert "(+ 3 4)\n")
            (setq after-edit
                  (with-current-buffer parent
                    (buffer-string)))
            (org-edit-src-exit))
          (setq action
                (soma-org-bench--alist
                 (cons "operation" "org-edit-src-exit")
                 (cons "source_edit_buffer_was_separate"
                       (soma-org-bench--bool (not (eq parent edit))))
                 (cons "parent_buffer" (buffer-name parent))
                 (cons "edit_buffer" (buffer-name edit))))
          (with-current-buffer parent
            (save-buffer)
            (setq after-final (buffer-string)))
          (append
           (soma-org-bench--alist
            (cons "observation" (soma-org-bench--source-observation parent edit)))
           (soma-org-bench--grade-source-edit
            "correct-source-edit-exit" action before after-edit after-final parent edit)))
      (when (buffer-live-p edit) (kill-buffer edit))
      (when (buffer-live-p parent) (kill-buffer parent))
      (when (file-directory-p tmp) (delete-directory tmp t)))))

(defun soma-org-bench--scenario-source-edit-buffer-only ()
  (soma-org-bench--load-org)
  (let* ((tmp (make-temp-file "soma-org-src-bad-" t))
         (file (expand-file-name "notebook.org" tmp))
         parent edit before after-edit after-final action)
    (unwind-protect
        (progn
          (soma-org-bench--write-file file (soma-org-bench--source-file-text))
          (setq parent (find-file-noselect file))
          (with-current-buffer parent
            (org-mode)
            (goto-char (point-min))
            (search-forward "(+ 1 2)")
            (setq before (buffer-string))
            (org-edit-src-code)
            (setq edit (current-buffer)))
          (with-current-buffer edit
            (erase-buffer)
            (insert "(+ 3 4)\n")
            (setq after-edit
                  (with-current-buffer parent
                    (buffer-string))))
          (setq action
                (soma-org-bench--alist
                 (cons "operation" "edit-source-buffer-only")
                 (cons "source_edit_buffer_was_separate"
                       (soma-org-bench--bool (not (eq parent edit))))
                 (cons "parent_buffer" (buffer-name parent))
                 (cons "edit_buffer" (buffer-name edit))))
          (setq after-final
                (with-current-buffer parent
                  (buffer-string)))
          (append
           (soma-org-bench--alist
            (cons "observation" (soma-org-bench--source-observation parent edit)))
           (soma-org-bench--grade-source-edit
            "bad-source-edit-buffer-only" action before after-edit after-final parent edit)))
      (when (buffer-live-p edit) (kill-buffer edit))
      (when (buffer-live-p parent) (kill-buffer parent))
      (when (file-directory-p tmp) (delete-directory tmp t)))))

(defun soma-org-bench--subtree-text (title)
  (save-excursion
    (goto-char (point-min))
    (when (re-search-forward
           (format "^\\*+\\s-+\\(?:TODO\\s-+\\)?%s\\s-*$" (regexp-quote title))
           nil t)
      (beginning-of-line)
      (let ((begin (point))
            (end (save-excursion (org-end-of-subtree t t))))
        (buffer-substring-no-properties begin end)))))

(defun soma-org-bench--find-heading (title)
  (goto-char (point-min))
  (unless (re-search-forward
           (format "^\\*+\\s-+\\(?:TODO\\s-+\\)?%s\\s-*$" (regexp-quote title))
           nil t)
    (error "heading not found: %s" title))
  (beginning-of-line)
  (point))

(defun soma-org-bench--grade-refile (name action text)
  (let* ((id-count (soma-org-bench--count-regexp "^:ID: soma-task-1$" text))
         (custom-count (soma-org-bench--count-regexp "^:CUSTOM_ID: keep-me$" text))
         (inbox (with-temp-buffer
                  (insert text)
                  (org-mode)
                  (soma-org-bench--subtree-text "Inbox")))
         (archive (with-temp-buffer
                    (insert text)
                    (org-mode)
                    (soma-org-bench--subtree-text "Archive")))
         (checks
          (list
           (soma-org-bench--check
            "action used org-refile move"
            (and (string= (soma-org-bench--get action "operation") "org-refile")
                 (not (eq (soma-org-bench--get action "keep") t)))
            (format "%S" action))
           (soma-org-bench--check
            "source subtree removed from Inbox"
            (and inbox (not (string-match-p "Train model" inbox)))
            (or inbox "missing Inbox"))
           (soma-org-bench--check
            "subtree inserted under Archive"
            (and archive (string-match-p "^\\*\\* TODO Train model$" archive))
            (or archive "missing Archive"))
           (soma-org-bench--check
            "ID and custom property preserved exactly once"
            (and (= id-count 1) (= custom-count 1))
            (format "id-count=%d custom-count=%d" id-count custom-count))
           (soma-org-bench--check
            "child subtree preserved"
            (and archive (string-match-p "^\\*\\*\\* Child$" archive))
            (or archive "missing Archive")))))
    (soma-org-bench--alist
     (cons "name" name)
     (cons "checks" checks)
     (cons "final_text" text))))

(defun soma-org-bench--scenario-refile-correct ()
  (soma-org-bench--load-org)
  (let* ((tmp (make-temp-file "soma-org-refile-correct-" t))
         (file (expand-file-name "tasks.org" tmp))
         buffer archive-pos action text)
    (unwind-protect
        (progn
          (soma-org-bench--write-file file (soma-org-bench--refile-file-text))
          (setq buffer (find-file-noselect file))
          (with-current-buffer buffer
            (org-mode)
            (let ((org-log-refile nil)
                  (org-auto-align-tags nil)
                  (org-refile-use-cache nil))
              (setq archive-pos (soma-org-bench--find-heading "Archive"))
              (soma-org-bench--find-heading "Train model")
              (org-refile nil nil (list "Archive" file nil archive-pos) "Refile"))
            (save-buffer)
            (setq text (buffer-string)))
          (setq action
                (soma-org-bench--alist
                 (cons "operation" "org-refile")
                 (cons "keep" :json-false)
                 (cons "target" "Archive")))
          (append
           (soma-org-bench--alist
            (cons "observation"
                  (soma-org-bench--alist
                   (cons "buffer" (buffer-name buffer))
                   (cons "target" "Archive"))))
           (soma-org-bench--grade-refile "correct-refile-subtree" action text)))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p tmp) (delete-directory tmp t)))))

(defun soma-org-bench--scenario-refile-copy-duplicates-id ()
  (soma-org-bench--load-org)
  (let* ((tmp (make-temp-file "soma-org-refile-copy-" t))
         (file (expand-file-name "tasks.org" tmp))
         buffer archive-pos action text)
    (unwind-protect
        (progn
          (soma-org-bench--write-file file (soma-org-bench--refile-file-text))
          (setq buffer (find-file-noselect file))
          (with-current-buffer buffer
            (org-mode)
            (let ((org-log-refile nil)
                  (org-auto-align-tags nil)
                  (org-refile-use-cache nil))
              (setq archive-pos (soma-org-bench--find-heading "Archive"))
              (soma-org-bench--find-heading "Train model")
              (org-refile 3 nil (list "Archive" file nil archive-pos) "Refile"))
            (save-buffer)
            (setq text (buffer-string)))
          (setq action
                (soma-org-bench--alist
                 (cons "operation" "org-refile")
                 (cons "keep" t)
                 (cons "target" "Archive")))
          (append
           (soma-org-bench--alist
            (cons "observation"
                  (soma-org-bench--alist
                   (cons "buffer" (buffer-name buffer))
                   (cons "target" "Archive"))))
           (soma-org-bench--grade-refile "bad-refile-copy-duplicates-id" action text)))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p tmp) (delete-directory tmp t)))))

(defun soma-org-bench--scenario-refile-text-loses-metadata ()
  (soma-org-bench--load-org)
  (let* ((tmp (make-temp-file "soma-org-refile-text-" t))
         (file (expand-file-name "tasks.org" tmp))
         buffer action text)
    (unwind-protect
        (progn
          (soma-org-bench--write-file file (soma-org-bench--refile-file-text))
          (setq buffer (find-file-noselect file))
          (with-current-buffer buffer
            (org-mode)
            (let* ((source-begin (soma-org-bench--find-heading "Train model"))
                   (source-end (save-excursion (org-end-of-subtree t t))))
              (delete-region source-begin source-end)
              (soma-org-bench--find-heading "Archive")
              (goto-char (or (save-excursion (org-end-of-subtree t t))
                             (point-max)))
              (unless (bolp) (insert "\n"))
              (insert "** TODO Train model\nBody\n"))
            (save-buffer)
            (setq text (buffer-string)))
          (setq action
                (soma-org-bench--alist
                 (cons "operation" "manual-text-move")
                 (cons "target" "Archive")))
          (append
           (soma-org-bench--alist
            (cons "observation"
                  (soma-org-bench--alist
                   (cons "buffer" (buffer-name buffer))
                   (cons "target" "Archive"))))
           (soma-org-bench--grade-refile "bad-refile-text-loses-metadata" action text)))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p tmp) (delete-directory tmp t)))))

(defun soma-org-bench--scenario-babel-result-correct ()
  (soma-org-bench--load-org)
  (let* ((tmp (make-temp-file "soma-org-babel-correct-" t))
         (file (expand-file-name "run.org" tmp))
         buffer text action
         (run-id (or (getenv "SOMA_INSULA_RUN_ID") "")))
    (unwind-protect
        (progn
          (soma-org-bench--write-file file (soma-org-bench--babel-file-text))
          (setq buffer (find-file-noselect file))
          (with-current-buffer buffer
            (org-mode)
            (let ((org-confirm-babel-evaluate nil)
                  (default-directory (file-name-directory file)))
              (goto-char (point-min))
              (search-forward "#+begin_src")
              (org-babel-execute-src-block))
            (save-buffer)
            (setq text (buffer-string)))
          (setq action
                (soma-org-bench--alist
                 (cons "operation" "org-babel-execute-src-block")
                 (cons "language" "emacs-lisp")))
          (soma-org-bench--alist
           (cons "observation"
                 (soma-org-bench--alist
                  (cons "buffer" (buffer-name buffer))
                  (cons "default_directory" (file-name-directory file))))
           (cons "name" "correct-babel-result-parent")
           (cons "checks"
                 (list
                  (soma-org-bench--check
                   "action used org-babel-execute-src-block"
                   (string= (soma-org-bench--get action "operation")
                            "org-babel-execute-src-block")
                   (format "%S" action))
                  (soma-org-bench--check
                   "results inserted in parent Org buffer"
                   (string-match-p "^#\\+RESULTS:" text)
                   text)
                  (soma-org-bench--check
                   "run id reached Babel execution"
                   (and (not (string-empty-p run-id))
                        (string-match-p (regexp-quote run-id) text))
                   text)
                  (soma-org-bench--check
                   "default-directory came from source file directory"
                   (string-match-p (regexp-quote (file-name-directory file)) text)
                   text)))
           (cons "final_text" text)))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p tmp) (delete-directory tmp t)))))

(defun soma-org-bench--record-scenario (index id title expected-pass thunk)
  (let* ((result (funcall thunk))
         (checks (soma-org-bench--get result "checks"))
         (policy-pass (soma-org-bench--all-checks-pass-p checks))
         (as-expected (if expected-pass policy-pass (not policy-pass)))
         (record
          (append
           (soma-org-bench--alist
            (cons "id" id)
            (cons "title" title)
            (cons "expected_policy_pass" (soma-org-bench--bool expected-pass))
            (cons "policy_passed_grader" (soma-org-bench--bool policy-pass))
            (cons "as_expected" (soma-org-bench--bool as-expected)))
           result)))
    (push record soma-org-benchmark-results)
    (soma-org-bench--write-json
     (expand-file-name (format "%02d-%s.json" index id)
                       soma-org-benchmark-output-dir)
     record)
    record))

(defun soma-org-bench--insula-checks ()
  (let* ((plan (soma-org-bench--plan))
         (mode (or (getenv "SOMA_INSULA_MODE") ""))
         (plan-mode (or (soma-org-bench--get plan "mode") ""))
         (closure (or (soma-org-bench--get plan "dependency_closure") ""))
         (glibc-owner (or (soma-org-bench--get plan "glibc_owner") ""))
         (benchmark (or (soma-org-bench--get plan "benchmark") "")))
    (list
     (soma-org-bench--check
      "SOMA_INSULA_ACTIVE is set"
      (string= (or (getenv "SOMA_INSULA_ACTIVE") "") "1")
      (or (getenv "SOMA_INSULA_ACTIVE") ""))
     (soma-org-bench--check
      "runtime mode is selected rootfs"
      (and (string= mode "selected-container-rootfs")
           (string= plan-mode "selected-container-rootfs-insula"))
      (format "env=%S plan=%S" mode plan-mode))
     (soma-org-bench--check
      "benchmark plan is for Org"
      (string= benchmark "org-structure")
      benchmark)
     (soma-org-bench--check
      "dependency closure is rootfs-owned"
      (string= closure "rootfs_owned")
      closure)
     (soma-org-bench--check
      "glibc owner is container-image-rootfs"
      (string= glibc-owner "container-image-rootfs")
      glibc-owner)
     (soma-org-bench--check
      "cwd is rootfs repo mount"
      (string= (directory-file-name (expand-file-name default-directory))
               "/workspace/soma")
      default-directory))))

(defun soma-org-bench--summary ()
  (let* ((records (reverse soma-org-benchmark-results))
         (total (length records))
         (expected (cl-count-if
                    (lambda (record)
                      (eq (soma-org-bench--get record "as_expected") t))
                    records))
         (insula-checks (soma-org-bench--insula-checks))
         (insula-pass (soma-org-bench--all-checks-pass-p insula-checks))
         (summary
          (soma-org-bench--alist
           (cons "generated_at" (format-time-string "%FT%TZ" nil t))
           (cons "benchmark" "org-structure")
           (cons "total_scenarios" total)
           (cons "scenarios_as_expected" expected)
           (cons "insula_checks_passed" (soma-org-bench--bool insula-pass))
           (cons "insula_checks" insula-checks)
           (cons "plan" (soma-org-bench--plan))
           (cons "records" records))))
    (soma-org-bench--write-json
     (expand-file-name "summary.json" soma-org-benchmark-output-dir)
     summary)
    (with-temp-file (expand-file-name "summary.md" soma-org-benchmark-output-dir)
      (insert "# Org Benchmark Summary\n\n")
      (insert (format "- Generated: %s\n" (format-time-string "%FT%TZ" nil t)))
      (insert "- Benchmark: `org-structure`\n")
      (insert (format "- Total scenarios: %d\n" total))
      (insert (format "- Scenarios as expected: %d\n" expected))
      (insert (format "- Insula checks passed: `%S`\n" insula-pass))
      (insert (format "- Mode: `%s`\n" (or (getenv "SOMA_INSULA_MODE") "")))
      (insert (format "- Run id: `%s`\n\n" (or (getenv "SOMA_INSULA_RUN_ID") "")))
      (dolist (record records)
        (insert (format "## %s\n\n" (soma-org-bench--get record "title")))
        (insert (format "- ID: `%s`\n" (soma-org-bench--get record "id")))
        (insert (format "- Expected policy pass: `%S`\n"
                        (soma-org-bench--get record "expected_policy_pass")))
        (insert (format "- Policy passed grader: `%S`\n"
                        (soma-org-bench--get record "policy_passed_grader")))
        (insert (format "- As expected: `%S`\n\n"
                        (soma-org-bench--get record "as_expected")))))
    summary))

(defun soma-org-bench-run ()
  (let ((args (seq-remove (lambda (arg) (string= arg "--"))
                          command-line-args-left)))
    (setq soma-org-benchmark-output-dir
          (expand-file-name
           (or (car args)
               "benchmarks/emacs-gym/org/traces/latest"))))
  (make-directory soma-org-benchmark-output-dir t)
  (setq soma-org-benchmark-results nil)
  (soma-org-bench--record-scenario
   1 "correct-source-edit-exit"
   "Correct policy edits an Org source block through org-edit-src-exit"
   t
   #'soma-org-bench--scenario-source-edit-correct)
  (soma-org-bench--record-scenario
   2 "bad-source-edit-buffer-only"
   "Incorrect policy edits only the source-edit buffer"
   nil
   #'soma-org-bench--scenario-source-edit-buffer-only)
  (soma-org-bench--record-scenario
   3 "correct-refile-subtree"
   "Correct policy refiles a subtree while preserving metadata and children"
   t
   #'soma-org-bench--scenario-refile-correct)
  (soma-org-bench--record-scenario
   4 "bad-refile-copy-duplicates-id"
   "Incorrect policy copies during refile and duplicates the ID"
   nil
   #'soma-org-bench--scenario-refile-copy-duplicates-id)
  (soma-org-bench--record-scenario
   5 "bad-refile-text-loses-metadata"
   "Incorrect policy moves only visible text and drops Org metadata"
   nil
   #'soma-org-bench--scenario-refile-text-loses-metadata)
  (soma-org-bench--record-scenario
   6 "correct-babel-result-parent"
   "Correct policy executes a Babel block and grades the parent Org buffer"
   t
   #'soma-org-bench--scenario-babel-result-correct)
  (let* ((summary (soma-org-bench--summary))
         (total (soma-org-bench--get summary "total_scenarios"))
         (expected (soma-org-bench--get summary "scenarios_as_expected"))
         (insula-pass (eq (soma-org-bench--get summary "insula_checks_passed")
                          t)))
    (unless (and (= total expected) insula-pass)
      (princ
       (format "org-benchmark: failed total=%s expected=%s insula=%S\n"
               total expected insula-pass))
      (kill-emacs 1))
    (princ
     (format "org-benchmark: wrote results to %s\n"
             soma-org-benchmark-output-dir))))

(soma-org-bench-run)

;;; run-benchmark.el ends here
