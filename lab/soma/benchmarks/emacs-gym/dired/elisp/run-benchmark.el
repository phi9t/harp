;;; run-benchmark.el --- Dired benchmark for the Emacs gym -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'dired)
(require 'dired-aux)
(require 'json)
(require 'seq)
(require 'subr-x)

(defvar soma-dired-benchmark-output-dir nil)
(defvar soma-dired-benchmark-results nil)

(defun soma-dired-bench--alist (&rest pairs)
  pairs)

(defun soma-dired-bench--bool (value)
  (if value t :json-false))

(defun soma-dired-bench--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-dired-bench--read-json-file (path)
  (when (and path (not (string-empty-p path)) (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-dired-bench--plan ()
  (soma-dired-bench--read-json-file (or (getenv "SOMA_INSULA_PLAN") "")))

(defun soma-dired-bench--get (alist key)
  (alist-get key alist nil nil #'string=))

(defun soma-dired-bench--check (name passed detail)
  (soma-dired-bench--alist
   (cons "name" name)
   (cons "passed" (soma-dired-bench--bool passed))
   (cons "detail" detail)))

(defun soma-dired-bench--all-checks-pass-p (checks)
  (cl-every
   (lambda (check)
     (eq (soma-dired-bench--get check "passed") t))
   checks))

(defun soma-dired-bench--write-file (path text)
  (make-directory (file-name-directory path) t)
  (with-temp-file path
    (insert text)))

(defun soma-dired-bench--read-file (path)
  (when (file-exists-p path)
    (with-temp-buffer
      (insert-file-contents path)
      (buffer-substring-no-properties (point-min) (point-max)))))

(defun soma-dired-bench--file-hash (path)
  (when (file-exists-p path)
    (with-temp-buffer
      (set-buffer-multibyte nil)
      (insert-file-contents-literally path)
      (secure-hash 'sha256 (buffer-string)))))

(defun soma-dired-bench--file-signature (path)
  (if (not (file-exists-p path))
      (soma-dired-bench--alist
       (cons "exists" :json-false)
       (cons "path" path))
    (let ((attrs (file-attributes path 'integer)))
      (soma-dired-bench--alist
       (cons "exists" t)
       (cons "path" path)
       (cons "truename" (file-truename path))
       (cons "size" (nth 7 attrs))
       (cons "mode" (nth 8 attrs))
       (cons "inode" (nth 10 attrs))
       (cons "device" (nth 11 attrs))
       (cons "mtime" (format "%S" (nth 5 attrs)))
       (cons "ctime" (format "%S" (nth 6 attrs)))
       (cons "sha256" (soma-dired-bench--file-hash path))))))

(defun soma-dired-bench--make-dir (name)
  (let ((dir (make-temp-file (format "soma-dired-%s-" name) t)))
    (soma-dired-bench--write-file (expand-file-name "keep.txt" dir) "keep\n")
    (soma-dired-bench--write-file (expand-file-name "target-a.txt" dir) "target-a\n")
    (soma-dired-bench--write-file (expand-file-name "target-b.txt" dir) "target-b\n")
    (soma-dired-bench--write-file (expand-file-name "flagged-decoy.txt" dir) "flagged-decoy\n")
    (soma-dired-bench--write-file (expand-file-name "source.txt" dir) "source\n")
    (soma-dired-bench--write-file (expand-file-name "collision.txt" dir) "collision-original\n")
    dir))

(defun soma-dired-bench--files-state (dir)
  (mapcar
   (lambda (name)
     (let ((path (expand-file-name name dir)))
       (soma-dired-bench--alist
        (cons "name" name)
        (cons "exists" (soma-dired-bench--bool (file-exists-p path)))
        (cons "contents" (or (soma-dired-bench--read-file path) "")))))
   '("keep.txt" "target-a.txt" "target-b.txt" "flagged-decoy.txt"
     "source.txt" "renamed.txt" "collision.txt")))

(defun soma-dired-bench--file-visible-p (buffer file)
  (with-current-buffer buffer
    (save-excursion
      (if (ignore-errors (dired-goto-file file))
          t
        nil))))

(defun soma-dired-bench--marked-basenames (buffer)
  (with-current-buffer buffer
    (sort
     (mapcar #'file-name-nondirectory
             (or (dired-get-marked-files nil 'marked nil nil nil) nil))
     #'string<)))

(defun soma-dired-bench--mark-file (buffer file)
  (with-current-buffer buffer
    (dired-goto-file file)
    (dired-mark 1)))

(defun soma-dired-bench--flag-delete-file (buffer file)
  (with-current-buffer buffer
    (dired-goto-file file)
    (dired-flag-file-deletion 1)))

(defun soma-dired-bench--dired-buffer (dir)
  (let ((buffer (dired-noselect dir)))
    (with-current-buffer buffer
      (setq-local dired-no-confirm t)
      (setq-local delete-by-moving-to-trash nil)
      (setq-local dired-recursive-deletes nil)
      (revert-buffer))
    buffer))

(defun soma-dired-bench--delete-marked (buffer)
  (with-current-buffer buffer
    (let ((dired-no-confirm t)
          (dired-deletion-confirmer (lambda (&rest _) t))
          (delete-by-moving-to-trash nil)
          (dired-recursive-deletes nil))
      (dired-do-delete 'marked))))

(defun soma-dired-bench--delete-flagged (buffer)
  (with-current-buffer buffer
    (let ((dired-no-confirm t)
          (dired-deletion-confirmer (lambda (&rest _) t))
          (delete-by-moving-to-trash nil)
          (dired-recursive-deletes nil))
      (dired-do-flagged-delete t))))

(defun soma-dired-bench--grade-delete (name action dir buffer)
  (with-current-buffer buffer
    (revert-buffer))
  (let* ((a (expand-file-name "target-a.txt" dir))
         (b (expand-file-name "target-b.txt" dir))
         (keep (expand-file-name "keep.txt" dir))
         (decoy (expand-file-name "flagged-decoy.txt" dir))
         (checks
          (list
           (soma-dired-bench--check
            "action used dired-do-delete for marked files"
            (and (string= (soma-dired-bench--get action "operation") "dired-do-delete")
                 (string= (soma-dired-bench--get action "selection") "marked"))
            (format "%S" action))
           (soma-dired-bench--check
            "marked targets deleted from filesystem"
            (and (not (file-exists-p a)) (not (file-exists-p b)))
            (format "target-a=%S target-b=%S"
                    (file-exists-p a)
                    (file-exists-p b)))
           (soma-dired-bench--check
            "unrelated files preserved"
            (and (file-exists-p keep) (file-exists-p decoy))
            (format "keep=%S decoy=%S"
                    (file-exists-p keep)
                    (file-exists-p decoy)))
           (soma-dired-bench--check
            "Dired buffer no longer shows deleted files after refresh"
            (and (not (soma-dired-bench--file-visible-p buffer a))
                 (not (soma-dired-bench--file-visible-p buffer b))
                 (soma-dired-bench--file-visible-p buffer keep))
            (format "visible-a=%S visible-b=%S visible-keep=%S"
                    (soma-dired-bench--file-visible-p buffer a)
                    (soma-dired-bench--file-visible-p buffer b)
                    (soma-dired-bench--file-visible-p buffer keep))))))
    (soma-dired-bench--alist
     (cons "name" name)
     (cons "checks" checks)
     (cons "files" (soma-dired-bench--files-state dir)))))

(defun soma-dired-bench--scenario-delete-marked-correct ()
  (let* ((dir (soma-dired-bench--make-dir "delete-marked-correct"))
         (buffer (soma-dired-bench--dired-buffer dir))
         action result)
    (unwind-protect
        (progn
          (soma-dired-bench--mark-file buffer (expand-file-name "target-a.txt" dir))
          (soma-dired-bench--mark-file buffer (expand-file-name "target-b.txt" dir))
          (setq action
                (soma-dired-bench--alist
                 (cons "operation" "dired-do-delete")
                 (cons "selection" "marked")
                 (cons "marked_before" (soma-dired-bench--marked-basenames buffer))))
          (soma-dired-bench--delete-marked buffer)
          (setq result
                (soma-dired-bench--grade-delete
                 "correct-marked-delete" action dir buffer))
          (cons
           (cons "observation"
                 (soma-dired-bench--alist
                  (cons "buffer" (buffer-name buffer))
                  (cons "directory" dir)
                  (cons "marked_before"
                        (soma-dired-bench--get action "marked_before"))))
           result))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p dir) (delete-directory dir t)))))

(defun soma-dired-bench--scenario-flagged-delete-confused ()
  (let* ((dir (soma-dired-bench--make-dir "delete-flagged-bad"))
         (buffer (soma-dired-bench--dired-buffer dir))
         action result)
    (unwind-protect
        (progn
          (soma-dired-bench--mark-file buffer (expand-file-name "target-a.txt" dir))
          (soma-dired-bench--mark-file buffer (expand-file-name "target-b.txt" dir))
          (soma-dired-bench--flag-delete-file
           buffer (expand-file-name "flagged-decoy.txt" dir))
          (setq action
                (soma-dired-bench--alist
                 (cons "operation" "dired-do-flagged-delete")
                 (cons "selection" "deletion-flags")
                 (cons "marked_before" (soma-dired-bench--marked-basenames buffer))))
          (soma-dired-bench--delete-flagged buffer)
          (setq result
                (soma-dired-bench--grade-delete
                 "bad-flagged-delete-confused-with-marked" action dir buffer))
          (cons
           (cons "observation"
                 (soma-dired-bench--alist
                  (cons "buffer" (buffer-name buffer))
                  (cons "directory" dir)
                  (cons "marked_before"
                        (soma-dired-bench--get action "marked_before"))))
           result))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p dir) (delete-directory dir t)))))

(defun soma-dired-bench--grade-rename (name action dir buffer outcome)
  (with-current-buffer buffer
    (revert-buffer))
  (let* ((source (expand-file-name "source.txt" dir))
         (renamed (expand-file-name "renamed.txt" dir))
         (checks
          (list
           (soma-dired-bench--check
            "action used dired-rename-file without overwrite"
            (and (string= (soma-dired-bench--get action "operation") "dired-rename-file")
                 (not (eq (soma-dired-bench--get action "ok_if_already_exists") t)))
            (format "%S" action))
           (soma-dired-bench--check
            "source moved to requested target"
            (and (not (file-exists-p source))
                 (file-exists-p renamed)
                 (string= (soma-dired-bench--read-file renamed) "source\n"))
            (format "source=%S renamed=%S contents=%S"
                    (file-exists-p source)
                    (file-exists-p renamed)
                    (soma-dired-bench--read-file renamed)))
           (soma-dired-bench--check
            "Dired buffer reflects renamed target after refresh"
            (and (not (soma-dired-bench--file-visible-p buffer source))
                 (soma-dired-bench--file-visible-p buffer renamed))
            (format "source-visible=%S renamed-visible=%S"
                    (soma-dired-bench--file-visible-p buffer source)
                    (soma-dired-bench--file-visible-p buffer renamed))))))
    (soma-dired-bench--alist
     (cons "name" name)
     (cons "checks" checks)
     (cons "outcome" outcome)
     (cons "files" (soma-dired-bench--files-state dir)))))

(defun soma-dired-bench--scenario-rename-correct ()
  (let* ((dir (soma-dired-bench--make-dir "rename-correct"))
         (buffer (soma-dired-bench--dired-buffer dir))
         (source (expand-file-name "source.txt" dir))
         (renamed (expand-file-name "renamed.txt" dir))
         action outcome)
    (unwind-protect
        (progn
          (with-current-buffer buffer
            (dired-goto-file source)
            (dired-rename-file source renamed nil))
          (setq action
                (soma-dired-bench--alist
                 (cons "operation" "dired-rename-file")
                 (cons "ok_if_already_exists" :json-false)
                 (cons "target" "renamed.txt")))
          (setq outcome "renamed")
          (cons
           (cons "observation"
                 (soma-dired-bench--alist
                  (cons "buffer" (buffer-name buffer))
                  (cons "directory" dir)))
           (soma-dired-bench--grade-rename
            "correct-rename-no-collision" action dir buffer outcome)))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p dir) (delete-directory dir t)))))

(defun soma-dired-bench--grade-collision (name action dir outcome)
  (let* ((source (expand-file-name "source.txt" dir))
         (target (expand-file-name "collision.txt" dir))
         (checks
          (list
           (soma-dired-bench--check
            "collision did not overwrite target"
            (string= (soma-dired-bench--read-file target) "collision-original\n")
            (or (soma-dired-bench--read-file target) "missing target"))
           (soma-dired-bench--check
            "source remains after collision"
            (and (file-exists-p source)
                 (string= (soma-dired-bench--read-file source) "source\n"))
            (format "source=%S contents=%S"
                    (file-exists-p source)
                    (soma-dired-bench--read-file source)))
           (soma-dired-bench--check
            "action reported collision"
            (string= outcome "collision")
            (format "outcome=%S action=%S" outcome action)))))
    (soma-dired-bench--alist
     (cons "name" name)
     (cons "checks" checks)
     (cons "outcome" outcome)
     (cons "files" (soma-dired-bench--files-state dir)))))

(defun soma-dired-bench--scenario-collision-rejected ()
  (let* ((dir (soma-dired-bench--make-dir "collision-correct"))
         (buffer (soma-dired-bench--dired-buffer dir))
         (source (expand-file-name "source.txt" dir))
         (target (expand-file-name "collision.txt" dir))
         (action (soma-dired-bench--alist
                  (cons "operation" "dired-rename-file")
                  (cons "ok_if_already_exists" :json-false)
                  (cons "target" "collision.txt")))
         outcome)
    (unwind-protect
        (progn
          (with-current-buffer buffer
            (dired-goto-file source)
            (setq outcome
                  (condition-case nil
                      (progn
                        (dired-rename-file source target nil)
                        "renamed")
                    (file-already-exists "collision")
                    (file-error "collision")
                    (error "error"))))
          (cons
           (cons "observation"
                 (soma-dired-bench--alist
                  (cons "buffer" (buffer-name buffer))
                  (cons "directory" dir)))
           (soma-dired-bench--grade-collision
            "collision-rejected-no-overwrite" action dir outcome)))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p dir) (delete-directory dir t)))))

(defun soma-dired-bench--scenario-collision-overwrite ()
  (let* ((dir (soma-dired-bench--make-dir "collision-bad"))
         (buffer (soma-dired-bench--dired-buffer dir))
         (source (expand-file-name "source.txt" dir))
         (target (expand-file-name "collision.txt" dir))
         (action (soma-dired-bench--alist
                  (cons "operation" "dired-rename-file")
                  (cons "ok_if_already_exists" t)
                  (cons "target" "collision.txt")))
         outcome)
    (unwind-protect
        (progn
          (with-current-buffer buffer
            (dired-goto-file source)
            (setq outcome
                  (condition-case err
                      (progn
                        (dired-rename-file source target t)
                        "renamed")
                    (error (format "error:%S" err)))))
          (cons
           (cons "observation"
                 (soma-dired-bench--alist
                  (cons "buffer" (buffer-name buffer))
                  (cons "directory" dir)))
           (soma-dired-bench--grade-collision
            "bad-collision-overwrite" action dir outcome)))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p dir) (delete-directory dir t)))))

(defun soma-dired-bench--scenario-undo-recovery-bad ()
  (let* ((dir (make-temp-file "soma-dired-undo-bad-" t))
         (victim (expand-file-name "victim.txt" dir))
         buffer undo-result action)
    (unwind-protect
        (progn
          (soma-dired-bench--write-file victim "victim\n")
          (setq buffer (soma-dired-bench--dired-buffer dir))
          (soma-dired-bench--mark-file buffer victim)
          (soma-dired-bench--delete-marked buffer)
          (with-current-buffer buffer
            (setq undo-result
                  (condition-case err
                      (progn
                        (dired-undo)
                        "returned")
                    (error (format "error:%S" err)))))
          (setq action
                (soma-dired-bench--alist
                 (cons "operation" "dired-undo")
                 (cons "recovery_target" "filesystem")))
          (soma-dired-bench--alist
           (cons "observation"
                 (soma-dired-bench--alist
                  (cons "buffer" (buffer-name buffer))
                  (cons "directory" dir)
                  (cons "undo_result" undo-result)))
           (cons "name" "bad-dired-undo-filesystem-recovery")
           (cons "checks"
                 (list
                  (soma-dired-bench--check
                   "action treated Dired undo as filesystem recovery"
                   (string= (soma-dired-bench--get action "operation") "filesystem-restore")
                   (format "%S" action))
                  (soma-dired-bench--check
                   "deleted file was restored"
                   (file-exists-p victim)
                   (format "undo-result=%S exists=%S"
                           undo-result
                           (file-exists-p victim)))))
           (cons "files" (soma-dired-bench--files-state dir))))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p dir) (delete-directory dir t)))))

(defun soma-dired-bench--grade-stale-mark-revalidation
    (name action dir buffer target replacement-content)
  (with-current-buffer buffer
    (revert-buffer))
  (let* ((keep (expand-file-name "keep.txt" dir))
         (marked-after-refresh (soma-dired-bench--marked-basenames buffer))
         (marked-after-policy
          (or (soma-dired-bench--get action "marked_after_policy")
              marked-after-refresh))
         (checks
          (list
           (soma-dired-bench--check
            "action revalidated Dired buffer before destructive operation"
            (and (string= (soma-dired-bench--get action "operation")
                          "dired-revert-and-revalidate")
                 (eq (soma-dired-bench--get action "stale_detected") t))
            (format "%S" action))
           (soma-dired-bench--check
            "action detected same-path replacement by file signature"
            (and (eq (soma-dired-bench--get action "signature_mismatch") t)
                 (not (equal (soma-dired-bench--get action "pre_change_signature")
                             (soma-dired-bench--get action
                                                    "post_revalidate_signature"))))
            (format "%S" action))
           (soma-dired-bench--check
            "action did not trust stale visible Dired mark"
            (and (member (file-name-nondirectory target)
                         (or (soma-dired-bench--get action
                                                    "marked_after_revalidate")
                             nil))
                 (eq (soma-dired-bench--get action
                                            "destructive_operation_aborted")
                     t)
                 (not (member (file-name-nondirectory target)
                              marked-after-policy)))
            (format "marked-after-policy=%S action=%S"
                    marked-after-policy action))
           (soma-dired-bench--check
            "external replacement file was preserved"
            (and (file-exists-p target)
                 (string= (soma-dired-bench--read-file target)
                          replacement-content))
            (format "exists=%S contents=%S"
                    (file-exists-p target)
                    (soma-dired-bench--read-file target)))
           (soma-dired-bench--check
            "unrelated file was preserved"
            (and (file-exists-p keep)
                 (string= (soma-dired-bench--read-file keep) "keep\n"))
            (format "keep=%S contents=%S"
                    (file-exists-p keep)
                    (soma-dired-bench--read-file keep)))
           (soma-dired-bench--check
            "same-path entry stayed visible while stale mark was cleared"
            (and (soma-dired-bench--file-visible-p buffer target)
                 (member (file-name-nondirectory target)
                         (or (soma-dired-bench--get action
                                                    "marked_after_revalidate")
                             nil))
                 (not (member (file-name-nondirectory target)
                              marked-after-policy)))
            (format "visible=%S marked-after-revalidate=%S marked-after-policy=%S"
                    (soma-dired-bench--file-visible-p buffer target)
                    (soma-dired-bench--get action "marked_after_revalidate")
                    marked-after-policy)))))
    (soma-dired-bench--alist
     (cons "name" name)
     (cons "checks" checks)
     (cons "marked_after_refresh" marked-after-refresh)
     (cons "marked_after_policy" marked-after-policy)
     (cons "files" (soma-dired-bench--files-state dir)))))

(defun soma-dired-bench--scenario-stale-mark-revalidated ()
  (let* ((dir (soma-dired-bench--make-dir "stale-mark-correct"))
         (buffer (soma-dired-bench--dired-buffer dir))
         (target (expand-file-name "target-a.txt" dir))
         (replacement-content "external-replacement\n")
         pre-change-signature post-revalidate-signature
         marked-before marked-after-revalidate marked-after-policy
         stale-detected action result)
    (unwind-protect
        (progn
          (soma-dired-bench--mark-file buffer target)
          (setq marked-before (soma-dired-bench--marked-basenames buffer))
          (setq pre-change-signature
                (soma-dired-bench--file-signature target))
          (delete-file target)
          (soma-dired-bench--write-file target replacement-content)
          (with-current-buffer buffer
            (revert-buffer))
          (setq post-revalidate-signature
                (soma-dired-bench--file-signature target))
          (setq marked-after-revalidate
                (soma-dired-bench--marked-basenames buffer))
          (setq stale-detected
                (not (equal pre-change-signature post-revalidate-signature)))
          (when stale-detected
            (with-current-buffer buffer
              (dired-goto-file target)
              (dired-unmark 1)))
          (setq marked-after-policy
                (soma-dired-bench--marked-basenames buffer))
          (setq action
                (soma-dired-bench--alist
                 (cons "operation" "dired-revert-and-revalidate")
                 (cons "marked_before_external_change" marked-before)
                 (cons "marked_after_revalidate" marked-after-revalidate)
                 (cons "marked_after_policy" marked-after-policy)
                 (cons "pre_change_signature" pre-change-signature)
                 (cons "post_revalidate_signature"
                       post-revalidate-signature)
                 (cons "signature_mismatch"
                       (soma-dired-bench--bool stale-detected))
                 (cons "destructive_operation_aborted"
                       (soma-dired-bench--bool stale-detected))
                 (cons "stale_detected"
                       (soma-dired-bench--bool stale-detected))))
          (setq result
                (soma-dired-bench--grade-stale-mark-revalidation
                 "stale-mark-revalidated-after-external-replace"
                 action dir buffer target replacement-content))
          (cons
           (cons "observation"
                 (soma-dired-bench--alist
                  (cons "buffer" (buffer-name buffer))
                  (cons "directory" dir)
                  (cons "target" target)
                  (cons "marked_before_external_change" marked-before)
                  (cons "marked_after_revalidate" marked-after-revalidate)
                  (cons "marked_after_policy" marked-after-policy)
                  (cons "pre_change_signature" pre-change-signature)
                  (cons "post_revalidate_signature"
                        post-revalidate-signature)))
           result))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p dir) (delete-directory dir t)))))

(defun soma-dired-bench--scenario-stale-mark-trusted-bad ()
  (let* ((dir (soma-dired-bench--make-dir "stale-mark-bad"))
         (buffer (soma-dired-bench--dired-buffer dir))
         (target (expand-file-name "target-a.txt" dir))
         (replacement-content "external-replacement\n")
         marked-before action result)
    (unwind-protect
        (progn
          (soma-dired-bench--mark-file buffer target)
          (setq marked-before (soma-dired-bench--marked-basenames buffer))
          (delete-file target)
          (soma-dired-bench--write-file target replacement-content)
          (setq action
                (soma-dired-bench--alist
                 (cons "operation" "dired-do-delete")
                 (cons "selection" "stale-marked")
                 (cons "marked_before_external_change" marked-before)
                 (cons "stale_detected" :json-false)))
          (soma-dired-bench--delete-marked buffer)
          (setq result
                (soma-dired-bench--grade-stale-mark-revalidation
                 "bad-stale-mark-deletes-external-replacement"
                 action dir buffer target replacement-content))
          (cons
           (cons "observation"
                 (soma-dired-bench--alist
                  (cons "buffer" (buffer-name buffer))
                  (cons "directory" dir)
                  (cons "target" target)
                  (cons "marked_before_external_change" marked-before)))
           result))
      (when (buffer-live-p buffer) (kill-buffer buffer))
      (when (file-directory-p dir) (delete-directory dir t)))))

(defun soma-dired-bench--record-scenario (index id title expected-pass thunk)
  (let* ((result (funcall thunk))
         (checks (soma-dired-bench--get result "checks"))
         (policy-pass (soma-dired-bench--all-checks-pass-p checks))
         (as-expected (if expected-pass policy-pass (not policy-pass)))
         (record
          (append
           (soma-dired-bench--alist
            (cons "id" id)
            (cons "title" title)
            (cons "expected_policy_pass" (soma-dired-bench--bool expected-pass))
            (cons "policy_passed_grader" (soma-dired-bench--bool policy-pass))
            (cons "as_expected" (soma-dired-bench--bool as-expected)))
           result)))
    (push record soma-dired-benchmark-results)
    (soma-dired-bench--write-json
     (expand-file-name (format "%02d-%s.json" index id)
                       soma-dired-benchmark-output-dir)
     record)
    record))

(defun soma-dired-bench--insula-checks ()
  (let* ((plan (soma-dired-bench--plan))
         (mode (or (getenv "SOMA_INSULA_MODE") ""))
         (plan-mode (or (soma-dired-bench--get plan "mode") ""))
         (closure (or (soma-dired-bench--get plan "dependency_closure") ""))
         (glibc-owner (or (soma-dired-bench--get plan "glibc_owner") ""))
         (benchmark (or (soma-dired-bench--get plan "benchmark") "")))
    (list
     (soma-dired-bench--check
      "SOMA_INSULA_ACTIVE is set"
      (string= (or (getenv "SOMA_INSULA_ACTIVE") "") "1")
      (or (getenv "SOMA_INSULA_ACTIVE") ""))
     (soma-dired-bench--check
      "runtime mode is selected rootfs"
      (and (string= mode "selected-container-rootfs")
           (string= plan-mode "selected-container-rootfs-insula"))
      (format "env=%S plan=%S" mode plan-mode))
     (soma-dired-bench--check
      "benchmark plan is for Dired"
      (string= benchmark "dired-filesystem")
      benchmark)
     (soma-dired-bench--check
      "dependency closure is rootfs-owned"
      (string= closure "rootfs_owned")
      closure)
     (soma-dired-bench--check
      "glibc owner is container-image-rootfs"
      (string= glibc-owner "container-image-rootfs")
      glibc-owner)
     (soma-dired-bench--check
      "cwd is rootfs repo mount"
      (string= (directory-file-name (expand-file-name default-directory))
               "/workspace/soma")
      default-directory))))

(defun soma-dired-bench--summary ()
  (let* ((records (reverse soma-dired-benchmark-results))
         (total (length records))
         (expected (cl-count-if
                    (lambda (record)
                      (eq (soma-dired-bench--get record "as_expected") t))
                    records))
         (insula-checks (soma-dired-bench--insula-checks))
         (insula-pass (soma-dired-bench--all-checks-pass-p insula-checks))
         (summary
          (soma-dired-bench--alist
           (cons "generated_at" (format-time-string "%FT%TZ" nil t))
           (cons "benchmark" "dired-filesystem")
           (cons "total_scenarios" total)
           (cons "scenarios_as_expected" expected)
           (cons "insula_checks_passed" (soma-dired-bench--bool insula-pass))
           (cons "insula_checks" insula-checks)
           (cons "plan" (soma-dired-bench--plan))
           (cons "records" records))))
    (soma-dired-bench--write-json
     (expand-file-name "summary.json" soma-dired-benchmark-output-dir)
     summary)
    (with-temp-file (expand-file-name "summary.md" soma-dired-benchmark-output-dir)
      (insert "# Dired Benchmark Summary\n\n")
      (insert (format "- Generated: %s\n" (format-time-string "%FT%TZ" nil t)))
      (insert "- Benchmark: `dired-filesystem`\n")
      (insert (format "- Total scenarios: %d\n" total))
      (insert (format "- Scenarios as expected: %d\n" expected))
      (insert (format "- Insula checks passed: `%S`\n" insula-pass))
      (insert (format "- Mode: `%s`\n" (or (getenv "SOMA_INSULA_MODE") "")))
      (insert (format "- Run id: `%s`\n\n" (or (getenv "SOMA_INSULA_RUN_ID") "")))
      (dolist (record records)
        (insert (format "## %s\n\n" (soma-dired-bench--get record "title")))
        (insert (format "- ID: `%s`\n" (soma-dired-bench--get record "id")))
        (insert (format "- Expected policy pass: `%S`\n"
                        (soma-dired-bench--get record "expected_policy_pass")))
        (insert (format "- Policy passed grader: `%S`\n"
                        (soma-dired-bench--get record "policy_passed_grader")))
        (insert (format "- As expected: `%S`\n\n"
                        (soma-dired-bench--get record "as_expected")))))
    summary))

(defun soma-dired-bench-run ()
  (let ((args (seq-remove (lambda (arg) (string= arg "--"))
                          command-line-args-left)))
    (setq soma-dired-benchmark-output-dir
          (expand-file-name
           (or (car args)
               "benchmarks/emacs-gym/dired/traces/latest"))))
  (make-directory soma-dired-benchmark-output-dir t)
  (setq soma-dired-benchmark-results nil)
  (soma-dired-bench--record-scenario
   1 "correct-marked-delete"
   "Correct policy deletes the Dired marked file set"
   t
   #'soma-dired-bench--scenario-delete-marked-correct)
  (soma-dired-bench--record-scenario
   2 "bad-flagged-delete-confused-with-marked"
   "Incorrect policy confuses deletion flags with ordinary Dired marks"
   nil
   #'soma-dired-bench--scenario-flagged-delete-confused)
  (soma-dired-bench--record-scenario
   3 "correct-rename-no-collision"
   "Correct policy renames a Dired file without a collision"
   t
   #'soma-dired-bench--scenario-rename-correct)
  (soma-dired-bench--record-scenario
   4 "collision-rejected-no-overwrite"
   "Correct policy refuses a Dired rename collision without overwrite"
   t
   #'soma-dired-bench--scenario-collision-rejected)
  (soma-dired-bench--record-scenario
   5 "bad-collision-overwrite"
   "Incorrect policy overwrites an existing file during a Dired rename"
   nil
   #'soma-dired-bench--scenario-collision-overwrite)
  (soma-dired-bench--record-scenario
   6 "bad-dired-undo-filesystem-recovery"
   "Incorrect policy treats Dired undo as filesystem recovery"
   nil
   #'soma-dired-bench--scenario-undo-recovery-bad)
  (soma-dired-bench--record-scenario
   7 "stale-mark-revalidated-after-external-replace"
   "Correct policy revalidates Dired marks after an external same-path replacement"
   t
   #'soma-dired-bench--scenario-stale-mark-revalidated)
  (soma-dired-bench--record-scenario
   8 "bad-stale-mark-deletes-external-replacement"
   "Incorrect policy trusts a stale Dired mark after external replacement"
   nil
   #'soma-dired-bench--scenario-stale-mark-trusted-bad)
  (let* ((summary (soma-dired-bench--summary))
         (total (soma-dired-bench--get summary "total_scenarios"))
         (expected (soma-dired-bench--get summary "scenarios_as_expected"))
         (insula-pass (eq (soma-dired-bench--get summary "insula_checks_passed")
                          t)))
    (unless (and (= total expected) insula-pass)
      (princ
       (format "dired-benchmark: failed total=%s expected=%s insula=%S\n"
               total expected insula-pass))
      (kill-emacs 1))
    (princ
     (format "dired-benchmark: wrote results to %s\n"
             soma-dired-benchmark-output-dir))))

(soma-dired-bench-run)

;;; run-benchmark.el ends here
