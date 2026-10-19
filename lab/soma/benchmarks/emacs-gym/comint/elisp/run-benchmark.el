;;; run-benchmark.el --- Comint benchmark for the Emacs gym -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'comint)
(require 'json)
(require 'subr-x)

(defvar soma-comint-benchmark-output-dir nil)
(defvar soma-comint-benchmark-results nil)

(defun soma-comint-bench--alist (&rest pairs)
  pairs)

(defun soma-comint-bench--bool (value)
  (if value t :json-false))

(defun soma-comint-bench--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-comint-bench--read-json-file (path)
  (when (and path (not (string-empty-p path)) (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-comint-bench--plan ()
  (soma-comint-bench--read-json-file (or (getenv "SOMA_INSULA_PLAN") "")))

(defun soma-comint-bench--get (alist key)
  (alist-get key alist nil nil #'string=))

(defun soma-comint-bench--check (name passed detail)
  (soma-comint-bench--alist
   (cons "name" name)
   (cons "passed" (soma-comint-bench--bool passed))
   (cons "detail" detail)))

(defun soma-comint-bench--all-checks-pass-p (checks)
  (cl-every
   (lambda (check)
     (eq (soma-comint-bench--get check "passed") t))
   checks))

(defun soma-comint-bench--rootfs-owned-path-p (path)
  (and (stringp path)
       (file-name-absolute-p path)
       (not (string-empty-p path))
       (not (string-prefix-p "/nix/" path))
       (not (string-prefix-p "/data02/home/" path))
       (not (string-prefix-p "/workspace/" path))
       (not (string-prefix-p "/project/" path))))

(defun soma-comint-bench--wait-buffer-regexp (buffer regexp timeout &optional start)
  (let ((deadline (+ (float-time) timeout))
        found)
    (while (and (not found) (< (float-time) deadline))
      (accept-process-output (get-buffer-process buffer) 0.05)
      (with-current-buffer buffer
        (setq found
              (save-excursion
                (goto-char
                 (cond
                  ((markerp start)
                   (or (marker-position start) (point-min)))
                  ((integerp start)
                   start)
                  (t
                   (point-min))))
                (re-search-forward regexp nil t)))))
    found))

(defun soma-comint-bench--output-line-regexp (prefix label)
  (format "^\\(?:[$#] \\)?%s:%s$"
          (regexp-quote prefix)
          (regexp-quote label)))

(defun soma-comint-bench--output-prefix-regexp (prefix)
  (format "^\\(?:[$#] \\)?%s" (regexp-quote prefix)))

(defun soma-comint-bench--buffer-text (buffer)
  (with-current-buffer buffer
    (buffer-substring-no-properties (point-min) (point-max))))

(defun soma-comint-bench--line-count (path)
  (if (file-exists-p path)
      (with-temp-buffer
        (insert-file-contents path)
        (length (split-string (buffer-string) "\n" t)))
    0))

(defun soma-comint-bench--file-lines (path)
  (if (file-exists-p path)
      (with-temp-buffer
        (insert-file-contents path)
        (split-string (buffer-string) "\n" t))
    nil))

(defun soma-comint-bench--make-shell (name)
  (let* ((buffer (generate-new-buffer (format " *soma-bench-%s*" name)))
         process)
    (with-current-buffer buffer
      (make-comint-in-buffer name buffer "sh" nil)
      (setq process (get-buffer-process buffer))
      (set-process-query-on-exit-flag process nil))
    (cons buffer process)))

(defun soma-comint-bench--cleanup-shell (cell)
  (let ((buffer (car cell))
        (process (cdr cell)))
    (when (and process (process-live-p process))
      (delete-process process))
    (when (buffer-live-p buffer)
      (kill-buffer buffer))))

(defun soma-comint-bench--command (label counter)
  (format
   "printf 'SOMA_BENCH_SENT:%s\\n'; printf 'SOMA_BENCH_RUN:%%s\\n' \"$SOMA_INSULA_RUN_ID\"; printf 'SOMA_BENCH_CWD:%%s\\n' \"$PWD\"; printf '%s\\n' >> %s; printf 'SOMA_BENCH_DONE:%s\\n'"
   label label (shell-quote-argument counter) label))

(defun soma-comint-bench--timeout-command (counter)
  (format
   "sleep 0.6; printf 'x\\n' >> %s; printf 'SOMA_TIMEOUT_DONE:%%s\\n' \"$(wc -l < %s)\""
   (shell-quote-argument counter)
   (shell-quote-argument counter)))

(defun soma-comint-bench--async-output-command (label)
  (format "sleep 0.1; printf 'SOMA_ASYNC_MARK_DRIFT:%s\\n'"
          label))

(defun soma-comint-bench--partial-output-command (label counter)
  (format
   "printf 'SOMA_PARTIAL_START:%s\\n'; sleep 0.4; printf '%s\\n' >> %s; printf 'SOMA_PARTIAL_DONE:%s\\n'"
   label label (shell-quote-argument counter) label))

(defun soma-comint-bench--interrupt-command (counter)
  (format
   "printf 'SOMA_INTERRUPT_START\\n'; sleep 5 && { printf 'finished\\n' >> %s; printf 'SOMA_INTERRUPT_DONE\\n'; }"
   (shell-quote-argument counter)))

(defun soma-comint-bench--send-live (buffer process command)
  (with-current-buffer buffer
    (goto-char (process-mark process))
    (insert command)
    (comint-send-input)
    (copy-marker (marker-position (process-mark process)) nil)))

(defun soma-comint-bench--send-raw (process command)
  (let ((start (copy-marker (marker-position (process-mark process)) nil)))
    (process-send-string process (concat command "\n"))
    start))

(defun soma-comint-bench--send-at-position (buffer position command)
  (with-current-buffer buffer
    (goto-char position)
    (let ((inhibit-read-only t))
      (insert command))
    (condition-case err
        (progn
          (comint-send-input)
          (soma-comint-bench--alist
           (cons "error" :json-false)
           (cons "position" position)
           (cons "wait_start"
                 (marker-position (process-mark
                                   (get-buffer-process buffer))))))
      ((quit error)
       (soma-comint-bench--alist
        (cons "error" (error-message-string err))
        (cons "position" position)
        (cons "wait_start" position))))))

(defun soma-comint-bench--last-input-range (buffer)
  (with-current-buffer buffer
    (let ((start (and (boundp 'comint-last-input-start)
                      comint-last-input-start))
          (end (and (boundp 'comint-last-input-end)
                    comint-last-input-end)))
      (soma-comint-bench--alist
       (cons "start" (and (markerp start) (marker-position start)))
       (cons "end" (and (markerp end) (marker-position end)))))))

(defun soma-comint-bench--add-checks (result extra-checks)
  (let ((cell (assoc "checks" result)))
    (if cell
        (setcdr cell (append (cdr cell) extra-checks))
      (setq result (append result (list (cons "checks" extra-checks)))))
    result))

(defun soma-comint-bench--process-exe (process)
  (let ((path (format "/proc/%s/exe" (process-id process))))
    (if (file-exists-p path)
        (file-truename path)
      "")))

(defun soma-comint-bench--base-observation (buffer process)
  (with-current-buffer buffer
    (soma-comint-bench--alist
     (cons "buffer" (buffer-name buffer))
     (cons "process" (process-name process))
     (cons "pid" (process-id process))
     (cons "process_exe" (soma-comint-bench--process-exe process))
     (cons "default_directory" default-directory)
     (cons "point" (point))
     (cons "process_mark" (marker-position (process-mark process)))
     (cons "point_after_process_mark"
           (soma-comint-bench--bool (>= (point) (marker-position (process-mark process)))))
     (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
     (cons "plan" (or (getenv "SOMA_INSULA_PLAN") "")))))

(defun soma-comint-bench--grade (name expected-label buffer process counter action
                                      &optional target-buffer)
  (let* ((output (soma-comint-bench--buffer-text buffer))
         (target-output (and target-buffer
                             (soma-comint-bench--buffer-text target-buffer)))
         (run-id (or (getenv "SOMA_INSULA_RUN_ID") ""))
         (last-start (and (boundp 'comint-last-input-start)
                          (with-current-buffer buffer comint-last-input-start)))
         (last-end (and (boundp 'comint-last-input-end)
                        (with-current-buffer buffer comint-last-input-end)))
         (last-markers-live (and (markerp last-start)
                                 (markerp last-end)
                                 (marker-position last-start)
                                 (marker-position last-end)
                                 (< (marker-position last-start)
                                    (marker-position last-end))))
         (counter-lines (soma-comint-bench--file-lines counter))
         (expected-output-sent
          (soma-comint-bench--output-line-regexp
           "SOMA_BENCH_SENT" expected-label))
         (expected-output-done
          (soma-comint-bench--output-line-regexp
           "SOMA_BENCH_DONE" expected-label))
         (checks
          (list
           (soma-comint-bench--check
            "action used comint-send-input"
            (string= (soma-comint-bench--get action "operation") "comint-send-input")
            (format "%S" action))
           (soma-comint-bench--check
            "comint input markers were updated"
            last-markers-live
            (format "start=%S end=%S" last-start last-end))
           (soma-comint-bench--check
            "output came from intended buffer"
            (and (string-match-p expected-output-done output)
                 (or (not target-buffer)
                     (not (and target-output
                               (string-match-p expected-output-done
                                               target-output)))))
            (format "intended=%S unintended=%S" output target-output))
           (soma-comint-bench--check
            "submitted expected live input"
            (and (string-match-p
                  expected-output-sent
                  output)
                 (string-match-p
                  expected-output-done
                  output))
            output)
           (soma-comint-bench--check
            "run id reached subprocess"
            (and (not (string-empty-p run-id))
                 (string-match-p (regexp-quote run-id) output))
            output)
           (soma-comint-bench--check
            "side effect occurred exactly once"
            (= (soma-comint-bench--line-count counter) 1)
            (format "counter-lines=%d" (soma-comint-bench--line-count counter)))
           (soma-comint-bench--check
            "side effect label matches intended input"
            (equal counter-lines (list expected-label))
            (format "counter-lines=%S" counter-lines))
           (soma-comint-bench--check
            "process executable is rootfs-owned"
            (soma-comint-bench--rootfs-owned-path-p
             (soma-comint-bench--process-exe process))
            (soma-comint-bench--process-exe process)))))
    (soma-comint-bench--alist
     (cons "name" name)
     (cons "checks" checks)
     (cons "output" output)
     (cons "counter_lines" (soma-comint-bench--line-count counter)))))

(defun soma-comint-bench--scenario-live-correct ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "correct-live"))
         (buffer (car cell))
         (process (cdr cell))
         (command (soma-comint-bench--command "live" counter))
         action observation result)
    (unwind-protect
        (progn
          (setq observation (soma-comint-bench--base-observation buffer process))
          (let ((wait-start
                 (soma-comint-bench--send-live buffer process command)))
            (soma-comint-bench--wait-buffer-regexp
             buffer (soma-comint-bench--output-line-regexp
                     "SOMA_BENCH_DONE" "live")
             3 wait-start))
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-send-input")
                 (cons "buffer" (buffer-name buffer))))
          (setq result
                (soma-comint-bench--grade
                 "correct-live-submit" "live" buffer process counter action))
          (cons (cons "observation" observation) result))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-raw-send ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "raw-send"))
         (buffer (car cell))
         (process (cdr cell))
         (command (soma-comint-bench--command "live" counter))
         action observation result)
    (unwind-protect
        (progn
          (setq observation (soma-comint-bench--base-observation buffer process))
          (let ((wait-start (soma-comint-bench--send-raw process command)))
            (soma-comint-bench--wait-buffer-regexp
             buffer (soma-comint-bench--output-line-regexp
                     "SOMA_BENCH_DONE" "live")
             3 wait-start))
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "process-send-string")
                 (cons "buffer" (buffer-name buffer))))
          (setq result
                (soma-comint-bench--grade
                 "bad-raw-send" "live" buffer process counter action))
          (cons (cons "observation" observation) result))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-old-history ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "old-history"))
         (buffer (car cell))
         (process (cdr cell))
         (old-command (soma-comint-bench--command "old" counter))
         (live-command (soma-comint-bench--command "live" counter))
         action observation result)
    (unwind-protect
        (progn
          (with-current-buffer buffer
            (let ((inhibit-read-only t))
              (erase-buffer)
              (insert old-command "\n")
              (set-marker (process-mark process) (point-max))
              (insert live-command))
            (setq-local comint-get-old-input (lambda () old-command))
            (setq-local comint-input-autoexpand nil)
            (setq-local comint-input-filter (lambda (_input) nil))
            (goto-char (point-min)))
          (setq observation (soma-comint-bench--base-observation buffer process))
          (with-current-buffer buffer
            (let ((wait-start
                   (progn
                     (comint-send-input)
                     (copy-marker (marker-position (process-mark process))
                                  nil))))
              (soma-comint-bench--wait-buffer-regexp
               buffer (soma-comint-bench--output-line-regexp
                       "SOMA_BENCH_DONE" "old")
               3 wait-start)))
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-send-input")
                 (cons "buffer" (buffer-name buffer))
                 (cons "point_relation" "before-process-mark")))
          (setq result
                (soma-comint-bench--grade
                 "bad-old-history" "live" buffer process counter action))
          (cons (cons "observation" observation) result))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-wrong-buffer ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (target (soma-comint-bench--make-shell "target"))
         (decoy (soma-comint-bench--make-shell "target-decoy"))
         (target-buffer (car target))
         (target-process (cdr target))
         (decoy-buffer (car decoy))
         (decoy-process (cdr decoy))
         (command (soma-comint-bench--command "live" counter))
         action observation result)
    (unwind-protect
        (progn
          (setq observation
                (soma-comint-bench--base-observation target-buffer target-process))
          (let ((wait-start
                 (soma-comint-bench--send-live decoy-buffer decoy-process command)))
            (soma-comint-bench--wait-buffer-regexp
             decoy-buffer (soma-comint-bench--output-line-regexp
                           "SOMA_BENCH_DONE" "live")
             3 wait-start))
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-send-input")
                 (cons "buffer" (buffer-name decoy-buffer))
                 (cons "intended_buffer" (buffer-name target-buffer))))
          (setq result
                (soma-comint-bench--grade
                 "bad-wrong-buffer" "live" target-buffer target-process
                 counter action decoy-buffer))
          (cons (cons "observation" observation) result))
      (soma-comint-bench--cleanup-shell target)
      (soma-comint-bench--cleanup-shell decoy)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-process-mark-drift-reobserve ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "mark-drift"))
         (buffer (car cell))
         (process (cdr cell))
         (command (soma-comint-bench--command "drift" counter))
         action observation before-mark after-async-mark wait-start
         async-seen result input-range)
    (unwind-protect
        (progn
          (setq observation (soma-comint-bench--base-observation buffer process))
          (setq before-mark
                (soma-comint-bench--get observation "process_mark"))
          (process-send-string
           process
           (concat (soma-comint-bench--async-output-command "ready") "\n"))
          (setq async-seen
                (soma-comint-bench--wait-buffer-regexp
                 buffer
                 (soma-comint-bench--output-line-regexp
                  "SOMA_ASYNC_MARK_DRIFT" "ready")
                 3 before-mark))
          (setq after-async-mark
                (with-current-buffer buffer
                  (marker-position (process-mark process))))
          (setq wait-start (soma-comint-bench--send-live buffer process command))
          (soma-comint-bench--wait-buffer-regexp
           buffer
           (soma-comint-bench--output-line-regexp
            "SOMA_BENCH_DONE" "drift")
           3 wait-start)
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-send-input")
                 (cons "buffer" (buffer-name buffer))
                 (cons "process_mark_source" "post-async-reobserve")
                 (cons "observed_process_mark_before_async" before-mark)
                 (cons "process_mark_before_submit" after-async-mark)))
          (setq input-range (soma-comint-bench--last-input-range buffer))
          (setq result
                (soma-comint-bench--grade
                 "process-mark-drift-reobserve" "drift" buffer process
                 counter action))
          (soma-comint-bench--add-checks
           (append
            (soma-comint-bench--alist
             (cons "observation" observation)
             (cons "async_output_seen" (soma-comint-bench--bool async-seen))
             (cons "process_mark_before_async" before-mark)
             (cons "process_mark_after_async" after-async-mark)
             (cons "last_input_range" input-range))
            result)
           (list
            (soma-comint-bench--check
             "asynchronous output was observed before submit"
             async-seen
             (soma-comint-bench--buffer-text buffer))
            (soma-comint-bench--check
             "process mark drifted after asynchronous output"
             (> after-async-mark before-mark)
             (format "before=%S after=%S" before-mark after-async-mark))
            (soma-comint-bench--check
             "policy submitted from the reobserved process mark"
             (and (= (soma-comint-bench--get action "process_mark_before_submit")
                     after-async-mark)
                  (>= (or (soma-comint-bench--get input-range "start") 0)
                      after-async-mark))
             (format "action=%S input-range=%S" action input-range)))))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-stale-process-mark-submit ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "stale-mark"))
         (buffer (car cell))
         (process (cdr cell))
         (command (soma-comint-bench--command "drift" counter))
         action observation before-mark after-async-mark submit wait-start
         async-seen result input-range)
    (unwind-protect
        (progn
          (setq observation (soma-comint-bench--base-observation buffer process))
          (setq before-mark
                (soma-comint-bench--get observation "process_mark"))
          (process-send-string
           process
           (concat (soma-comint-bench--async-output-command "ready") "\n"))
          (setq async-seen
                (soma-comint-bench--wait-buffer-regexp
                 buffer
                 (soma-comint-bench--output-line-regexp
                  "SOMA_ASYNC_MARK_DRIFT" "ready")
                 3 before-mark))
          (setq after-async-mark
                (with-current-buffer buffer
                  (marker-position (process-mark process))))
          (setq submit
                (soma-comint-bench--send-at-position
                 buffer before-mark command))
          (setq wait-start (soma-comint-bench--get submit "wait_start"))
          (soma-comint-bench--wait-buffer-regexp
           buffer
           (soma-comint-bench--output-line-regexp
            "SOMA_BENCH_DONE" "drift")
           3 wait-start)
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-send-input")
                 (cons "buffer" (buffer-name buffer))
                 (cons "process_mark_source" "stale-pre-async-observation")
                 (cons "observed_process_mark_before_async" before-mark)
                 (cons "actual_process_mark_before_submit" after-async-mark)
                 (cons "process_mark_before_submit" before-mark)
                 (cons "submit_error"
                       (soma-comint-bench--get submit "error"))))
          (setq input-range (soma-comint-bench--last-input-range buffer))
          (setq result
                (soma-comint-bench--grade
                 "bad-stale-process-mark-submit" "drift" buffer process
                 counter action))
          (soma-comint-bench--add-checks
           (append
            (soma-comint-bench--alist
             (cons "observation" observation)
             (cons "async_output_seen" (soma-comint-bench--bool async-seen))
             (cons "process_mark_before_async" before-mark)
             (cons "process_mark_after_async" after-async-mark)
             (cons "last_input_range" input-range)
             (cons "submit" submit))
            result)
           (list
            (soma-comint-bench--check
             "asynchronous output was observed before submit"
             async-seen
             (soma-comint-bench--buffer-text buffer))
            (soma-comint-bench--check
             "process mark drifted after asynchronous output"
             (> after-async-mark before-mark)
             (format "before=%S after=%S" before-mark after-async-mark))
            (soma-comint-bench--check
             "policy submitted from the reobserved process mark"
             (and (= (soma-comint-bench--get action "process_mark_before_submit")
                     after-async-mark)
                  (>= (or (soma-comint-bench--get input-range "start") 0)
                      after-async-mark))
             (format "action=%S input-range=%S" action input-range)))))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-timeout-unknown ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "timeout-unknown"))
         (buffer (car cell))
         (process (cdr cell))
         (command (soma-comint-bench--timeout-command counter))
         action observation wait-start early final output checks)
    (unwind-protect
        (progn
          (setq observation (soma-comint-bench--base-observation buffer process))
          (setq wait-start (soma-comint-bench--send-live buffer process command))
          (setq early
                (soma-comint-bench--wait-buffer-regexp
                 buffer (soma-comint-bench--output-prefix-regexp
                         "SOMA_TIMEOUT_DONE:")
                 0.15 wait-start))
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-send-input")
                 (cons "timeout_state" "outcome_unknown")
                 (cons "retry_count" 0)))
          (setq final
                (soma-comint-bench--wait-buffer-regexp
                 buffer (soma-comint-bench--output-line-regexp
                         "SOMA_TIMEOUT_DONE" "1")
                 3 wait-start))
          (setq output (soma-comint-bench--buffer-text buffer))
          (setq checks
                (list
                 (soma-comint-bench--check
                  "short wait returned outcome_unknown"
                  (and (not early)
                       (string= (soma-comint-bench--get action "timeout_state")
                                "outcome_unknown"))
                  (format "early=%S action=%S" early action))
                 (soma-comint-bench--check
                  "command eventually completed once"
                  (and final (= (soma-comint-bench--line-count counter) 1))
                  (format "final=%S counter-lines=%d output=%S"
                          final (soma-comint-bench--line-count counter) output))
                 (soma-comint-bench--check
                  "policy did not retry"
                  (= (soma-comint-bench--get action "retry_count") 0)
                  (format "%S" action))
                 (soma-comint-bench--check
                  "process executable is rootfs-owned"
                  (soma-comint-bench--rootfs-owned-path-p
                   (soma-comint-bench--process-exe process))
                  (soma-comint-bench--process-exe process))))
          (soma-comint-bench--alist
           (cons "observation" observation)
           (cons "name" "timeout-outcome-unknown")
           (cons "checks" checks)
           (cons "output" output)
           (cons "counter_lines" (soma-comint-bench--line-count counter))))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-timeout-retry ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "timeout-retry"))
         (buffer (car cell))
         (process (cdr cell))
         (command (soma-comint-bench--timeout-command counter))
         action observation wait-start early final output checks)
    (unwind-protect
        (progn
          (setq observation (soma-comint-bench--base-observation buffer process))
          (setq wait-start (soma-comint-bench--send-live buffer process command))
          (setq early
                (soma-comint-bench--wait-buffer-regexp
                 buffer (soma-comint-bench--output-prefix-regexp
                         "SOMA_TIMEOUT_DONE:")
                 0.15 wait-start))
          (soma-comint-bench--send-live buffer process command)
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-send-input")
                 (cons "timeout_state" "retried_after_timeout")
                 (cons "retry_count" 1)))
          (setq final
                (soma-comint-bench--wait-buffer-regexp
                 buffer (soma-comint-bench--output-line-regexp
                         "SOMA_TIMEOUT_DONE" "2")
                 3 wait-start))
          (setq output (soma-comint-bench--buffer-text buffer))
          (setq checks
                (list
                 (soma-comint-bench--check
                  "short wait returned outcome_unknown"
                  (not early)
                  (format "early=%S" early))
                 (soma-comint-bench--check
                  "policy did not retry"
                  (= (soma-comint-bench--get action "retry_count") 0)
                  (format "%S" action))
                 (soma-comint-bench--check
                  "non-idempotent side effect occurred exactly once"
                  (= (soma-comint-bench--line-count counter) 1)
                  (format "final=%S counter-lines=%d output=%S"
                          final (soma-comint-bench--line-count counter) output))))
          (soma-comint-bench--alist
           (cons "observation" observation)
           (cons "name" "bad-timeout-retry")
           (cons "checks" checks)
           (cons "output" output)
           (cons "counter_lines" (soma-comint-bench--line-count counter))))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-partial-output-collect ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "partial-output"))
         (buffer (car cell))
         (process (cdr cell))
         (command (soma-comint-bench--partial-output-command "partial" counter))
         action observation wait-start partial final early-done output checks)
    (unwind-protect
        (progn
          (setq observation (soma-comint-bench--base-observation buffer process))
          (setq wait-start (soma-comint-bench--send-live buffer process command))
          (setq partial
                (soma-comint-bench--wait-buffer-regexp
                 buffer
                 (soma-comint-bench--output-line-regexp
                  "SOMA_PARTIAL_START" "partial")
                 3 wait-start))
          (setq early-done
                (soma-comint-bench--wait-buffer-regexp
                 buffer
                 (soma-comint-bench--output-line-regexp
                  "SOMA_PARTIAL_DONE" "partial")
                 0.05 wait-start))
          (setq final
                (soma-comint-bench--wait-buffer-regexp
                 buffer
                 (soma-comint-bench--output-line-regexp
                  "SOMA_PARTIAL_DONE" "partial")
                 3 wait-start))
          (setq output (soma-comint-bench--buffer-text buffer))
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-send-input")
                 (cons "completion_state" "succeeded_after_final_marker")
                 (cons "partial_output_seen"
                       (soma-comint-bench--bool partial))
                 (cons "early_done_seen"
                       (soma-comint-bench--bool early-done))
                 (cons "final_output_seen"
                       (soma-comint-bench--bool final))))
          (setq checks
                (list
                 (soma-comint-bench--check
                  "partial output was observed before final marker"
                  (and partial (not early-done))
                  (format "partial=%S early-done=%S output=%S"
                          partial early-done output))
                 (soma-comint-bench--check
                  "policy waited for final completion evidence"
                  (and final
                       (string= (soma-comint-bench--get action "completion_state")
                                "succeeded_after_final_marker"))
                  (format "final=%S action=%S" final action))
                 (soma-comint-bench--check
                  "side effect occurred exactly once after completion"
                  (= (soma-comint-bench--line-count counter) 1)
                  (format "counter-lines=%d"
                          (soma-comint-bench--line-count counter)))
                 (soma-comint-bench--check
                  "side effect label matches partial command"
                  (equal (soma-comint-bench--file-lines counter)
                         (list "partial"))
                  (format "%S" (soma-comint-bench--file-lines counter)))
                 (soma-comint-bench--check
                  "process executable is rootfs-owned"
                  (soma-comint-bench--rootfs-owned-path-p
                   (soma-comint-bench--process-exe process))
                  (soma-comint-bench--process-exe process))))
          (soma-comint-bench--alist
           (cons "observation" observation)
           (cons "name" "partial-output-collect")
           (cons "action" action)
           (cons "checks" checks)
           (cons "output" output)
           (cons "counter_lines" (soma-comint-bench--line-count counter))))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-partial-output-premature-done ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "partial-premature"))
         (buffer (car cell))
         (process (cdr cell))
         (command (soma-comint-bench--partial-output-command "partial" counter))
         action observation wait-start partial final-before-success output checks)
    (unwind-protect
        (progn
          (setq observation (soma-comint-bench--base-observation buffer process))
          (setq wait-start (soma-comint-bench--send-live buffer process command))
          (setq partial
                (soma-comint-bench--wait-buffer-regexp
                 buffer
                 (soma-comint-bench--output-line-regexp
                  "SOMA_PARTIAL_START" "partial")
                 3 wait-start))
          (setq final-before-success
                (soma-comint-bench--wait-buffer-regexp
                 buffer
                 (soma-comint-bench--output-line-regexp
                  "SOMA_PARTIAL_DONE" "partial")
                 0.05 wait-start))
          (setq output (soma-comint-bench--buffer-text buffer))
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-send-input")
                 (cons "completion_state" "succeeded_on_partial_output")
                 (cons "partial_output_seen"
                       (soma-comint-bench--bool partial))
                 (cons "final_output_seen"
                       (soma-comint-bench--bool final-before-success))))
          (setq checks
                (list
                 (soma-comint-bench--check
                  "partial output was observed"
                  partial
                  output)
                 (soma-comint-bench--check
                  "policy did not treat partial output as completion"
                  (and final-before-success
                       (string= (soma-comint-bench--get action "completion_state")
                                "succeeded_after_final_marker"))
                  (format "final-before-success=%S action=%S"
                          final-before-success action))
                 (soma-comint-bench--check
                  "side effect completed before success"
                  (= (soma-comint-bench--line-count counter) 1)
                  (format "counter-lines=%d"
                          (soma-comint-bench--line-count counter)))))
          (soma-comint-bench--alist
           (cons "observation" observation)
           (cons "name" "bad-partial-output-premature-done")
           (cons "action" action)
           (cons "checks" checks)
           (cons "output" output)
           (cons "counter_lines" (soma-comint-bench--line-count counter))))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--scenario-interrupt-partial-output-reconcile ()
  (let* ((tmp (make-temp-file "soma-comint-bench-" t))
         (counter (expand-file-name "counter" tmp))
         (cell (soma-comint-bench--make-shell "interrupt-partial"))
         (buffer (car cell))
         (process (cdr cell))
         (command (soma-comint-bench--interrupt-command counter))
         action observation wait-start partial output checks)
    (unwind-protect
        (progn
          (setq observation (soma-comint-bench--base-observation buffer process))
          (setq wait-start (soma-comint-bench--send-live buffer process command))
          (setq partial
                (soma-comint-bench--wait-buffer-regexp
                 buffer "SOMA_INTERRUPT_START" 3 wait-start))
          (with-current-buffer buffer
            (goto-char (process-mark process))
            (comint-interrupt-subjob))
          (accept-process-output process 0.3)
          (setq output (soma-comint-bench--buffer-text buffer))
          (setq action
                (soma-comint-bench--alist
                 (cons "operation" "comint-interrupt-subjob")
                 (cons "interrupted_after_partial_output"
                       (soma-comint-bench--bool partial))
                 (cons "post_interrupt_process_status"
                       (format "%S" (process-status process)))))
          (setq checks
                (list
                 (soma-comint-bench--check
                  "partial output was observed before interrupt"
                  partial
                  output)
                 (soma-comint-bench--check
                  "policy used Comint interrupt operation"
                  (string= (soma-comint-bench--get action "operation")
                           "comint-interrupt-subjob")
                  (format "%S" action))
                 (soma-comint-bench--check
                  "interrupted command did not reach completion marker"
                  (not (string-match-p
                        "^\\(?:[$#] \\)?SOMA_INTERRUPT_DONE$"
                        output))
                  output)
                 (soma-comint-bench--check
                  "interrupted command side effect did not occur"
                  (= (soma-comint-bench--line-count counter) 0)
                  (format "counter-lines=%d"
                          (soma-comint-bench--line-count counter)))
                 (soma-comint-bench--check
                  "Comint shell process survived interrupt"
                  (process-live-p process)
                  (format "status=%S" (process-status process)))
                 (soma-comint-bench--check
                  "process executable is rootfs-owned"
                  (soma-comint-bench--rootfs-owned-path-p
                   (soma-comint-bench--process-exe process))
                  (soma-comint-bench--process-exe process))))
          (soma-comint-bench--alist
           (cons "observation" observation)
           (cons "name" "interrupt-partial-output-reconcile")
           (cons "action" action)
           (cons "checks" checks)
           (cons "output" output)
           (cons "counter_lines" (soma-comint-bench--line-count counter))))
      (soma-comint-bench--cleanup-shell cell)
      (when (file-directory-p tmp)
        (delete-directory tmp t)))))

(defun soma-comint-bench--record-scenario (index id title expected-pass thunk)
  (let* ((result (funcall thunk))
         (checks (soma-comint-bench--get result "checks"))
         (policy-pass (soma-comint-bench--all-checks-pass-p checks))
         (as-expected (if expected-pass policy-pass (not policy-pass)))
         (record
          (append
           (soma-comint-bench--alist
            (cons "id" id)
            (cons "title" title)
            (cons "expected_policy_pass" (soma-comint-bench--bool expected-pass))
            (cons "policy_passed_grader" (soma-comint-bench--bool policy-pass))
            (cons "as_expected" (soma-comint-bench--bool as-expected)))
           result)))
    (push record soma-comint-benchmark-results)
    (soma-comint-bench--write-json
     (expand-file-name (format "%02d-%s.json" index id)
                       soma-comint-benchmark-output-dir)
     record)
    record))

(defun soma-comint-bench--insula-checks ()
  (let* ((plan (soma-comint-bench--plan))
         (mode (or (getenv "SOMA_INSULA_MODE") ""))
         (plan-mode (or (soma-comint-bench--get plan "mode") ""))
         (closure (or (soma-comint-bench--get plan "dependency_closure") ""))
         (glibc-owner (or (soma-comint-bench--get plan "glibc_owner") ""))
         (benchmark (or (soma-comint-bench--get plan "benchmark") "")))
    (list
     (soma-comint-bench--check
      "SOMA_INSULA_ACTIVE is set"
      (string= (or (getenv "SOMA_INSULA_ACTIVE") "") "1")
      (or (getenv "SOMA_INSULA_ACTIVE") ""))
     (soma-comint-bench--check
      "runtime mode is selected rootfs"
      (and (string= mode "selected-container-rootfs")
           (string= plan-mode "selected-container-rootfs-insula"))
      (format "env=%S plan=%S" mode plan-mode))
     (soma-comint-bench--check
      "benchmark plan is for comint"
      (string= benchmark "comint-process")
      benchmark)
     (soma-comint-bench--check
      "dependency closure is rootfs-owned"
      (string= closure "rootfs_owned")
      closure)
     (soma-comint-bench--check
      "glibc owner is container-image-rootfs"
      (string= glibc-owner "container-image-rootfs")
      glibc-owner)
     (soma-comint-bench--check
      "cwd is rootfs repo mount"
      (string= (directory-file-name (expand-file-name default-directory))
               "/workspace/soma")
      default-directory))))

(defun soma-comint-bench--summary ()
  (let* ((records (reverse soma-comint-benchmark-results))
         (total (length records))
         (expected (cl-count-if
                    (lambda (record)
                      (eq (soma-comint-bench--get record "as_expected") t))
                    records))
         (insula-checks (soma-comint-bench--insula-checks))
         (insula-pass (soma-comint-bench--all-checks-pass-p insula-checks))
         (summary
          (soma-comint-bench--alist
           (cons "generated_at" (format-time-string "%FT%TZ" nil t))
           (cons "benchmark" "comint-process")
           (cons "total_scenarios" total)
           (cons "scenarios_as_expected" expected)
           (cons "insula_checks_passed" (soma-comint-bench--bool insula-pass))
           (cons "insula_checks" insula-checks)
           (cons "plan" (soma-comint-bench--plan))
           (cons "records" records))))
    (soma-comint-bench--write-json
     (expand-file-name "summary.json" soma-comint-benchmark-output-dir)
     summary)
    (with-temp-file (expand-file-name "summary.md" soma-comint-benchmark-output-dir)
      (insert "# Comint Benchmark Summary\n\n")
      (insert (format "- Generated: %s\n" (format-time-string "%FT%TZ" nil t)))
      (insert "- Benchmark: `comint-process`\n")
      (insert (format "- Total scenarios: %d\n" total))
      (insert (format "- Scenarios as expected: %d\n" expected))
      (insert (format "- Insula checks passed: `%S`\n" insula-pass))
      (insert (format "- Mode: `%s`\n" (or (getenv "SOMA_INSULA_MODE") "")))
      (insert (format "- Run id: `%s`\n\n" (or (getenv "SOMA_INSULA_RUN_ID") "")))
      (dolist (record records)
        (insert (format "## %s\n\n" (soma-comint-bench--get record "title")))
        (insert (format "- ID: `%s`\n" (soma-comint-bench--get record "id")))
        (insert (format "- Expected policy pass: `%S`\n"
                        (soma-comint-bench--get record "expected_policy_pass")))
        (insert (format "- Policy passed grader: `%S`\n"
                        (soma-comint-bench--get record "policy_passed_grader")))
        (insert (format "- As expected: `%S`\n\n"
                        (soma-comint-bench--get record "as_expected")))))
    summary))

(defun soma-comint-bench-run ()
  (let ((args (seq-remove (lambda (arg) (string= arg "--"))
                          command-line-args-left)))
    (setq soma-comint-benchmark-output-dir
          (expand-file-name
           (or (car args)
               "benchmarks/emacs-gym/comint/traces/latest"))))
  (make-directory soma-comint-benchmark-output-dir t)
  (setq soma-comint-benchmark-results nil)
  (soma-comint-bench--record-scenario
   1 "correct-live-submit"
   "Correct policy submits live input through Comint"
   t
   #'soma-comint-bench--scenario-live-correct)
  (soma-comint-bench--record-scenario
   2 "bad-raw-send"
   "Incorrect policy sends raw process bytes"
   nil
   #'soma-comint-bench--scenario-raw-send)
  (soma-comint-bench--record-scenario
   3 "bad-old-history"
   "Incorrect policy submits old history instead of live input"
   nil
   #'soma-comint-bench--scenario-old-history)
  (soma-comint-bench--record-scenario
   4 "bad-wrong-buffer"
   "Incorrect policy submits to a similarly named process buffer"
   nil
   #'soma-comint-bench--scenario-wrong-buffer)
  (soma-comint-bench--record-scenario
   5 "timeout-outcome-unknown"
   "Correct policy treats timeout after submission as outcome_unknown"
   t
   #'soma-comint-bench--scenario-timeout-unknown)
  (soma-comint-bench--record-scenario
   6 "bad-timeout-retry"
   "Incorrect policy retries non-idempotent command after timeout"
   nil
   #'soma-comint-bench--scenario-timeout-retry)
  (soma-comint-bench--record-scenario
   7 "process-mark-drift-reobserve"
   "Correct policy reobserves the process mark after asynchronous output"
   t
   #'soma-comint-bench--scenario-process-mark-drift-reobserve)
  (soma-comint-bench--record-scenario
   8 "bad-stale-process-mark-submit"
   "Incorrect policy submits using a process mark observed before asynchronous output"
   nil
   #'soma-comint-bench--scenario-stale-process-mark-submit)
  (soma-comint-bench--record-scenario
   9 "partial-output-collect"
   "Correct policy waits for final evidence after partial output"
   t
   #'soma-comint-bench--scenario-partial-output-collect)
  (soma-comint-bench--record-scenario
   10 "bad-partial-output-premature-done"
   "Incorrect policy treats partial output as command completion"
   nil
   #'soma-comint-bench--scenario-partial-output-premature-done)
  (soma-comint-bench--record-scenario
   11 "interrupt-partial-output-reconcile"
   "Correct policy interrupts after partial output and reconciles state"
   t
   #'soma-comint-bench--scenario-interrupt-partial-output-reconcile)
  (let* ((summary (soma-comint-bench--summary))
         (total (soma-comint-bench--get summary "total_scenarios"))
         (expected (soma-comint-bench--get summary "scenarios_as_expected"))
         (insula-pass (eq (soma-comint-bench--get summary "insula_checks_passed")
                          t)))
    (unless (and (= total expected) insula-pass)
      (princ
       (format "comint-benchmark: failed total=%s expected=%s insula=%S\n"
               total expected insula-pass))
      (kill-emacs 1))
    (princ
     (format "comint-benchmark: wrote results to %s\n"
             soma-comint-benchmark-output-dir))))

(soma-comint-bench-run)

;;; run-benchmark.el ends here
