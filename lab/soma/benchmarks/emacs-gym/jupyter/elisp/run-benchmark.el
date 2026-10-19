;;; run-benchmark.el --- Existing-kernel Jupyter benchmark for the Emacs gym -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'eieio)
(require 'json)
(require 'seq)
(require 'subr-x)

(defvar soma-jupyter-benchmark-output-dir nil)
(defvar soma-jupyter-benchmark-results nil)

(defun soma-jupyter-bench--alist (&rest pairs)
  pairs)

(defun soma-jupyter-bench--bool (value)
  (if value t :json-false))

(defun soma-jupyter-bench--write-json (path object)
  (let ((json-encoding-pretty-print t))
    (with-temp-file path
      (insert (json-encode object))
      (insert "\n"))))

(defun soma-jupyter-bench--read-json-file (path)
  (when (and path (not (string-empty-p path)) (file-readable-p path))
    (let ((json-object-type 'alist)
          (json-array-type 'list)
          (json-key-type 'string)
          (json-false :json-false))
      (json-read-file path))))

(defun soma-jupyter-bench--read-json-string (text)
  (let ((json-object-type 'alist)
        (json-array-type 'list)
        (json-key-type 'string)
        (json-false :json-false))
    (json-read-from-string text)))

(defun soma-jupyter-bench--plan ()
  (soma-jupyter-bench--read-json-file (or (getenv "SOMA_INSULA_PLAN") "")))

(defun soma-jupyter-bench--get (alist key)
  (alist-get key alist nil nil #'string=))

(defun soma-jupyter-bench--key-string (key)
  (cond
   ((keywordp key)
    (substring (symbol-name key) 1))
   ((symbolp key)
    (symbol-name key))
   ((stringp key)
    key)
   (t
    (format "%S" key))))

(defun soma-jupyter-bench--value (object key)
  (cond
   ((hash-table-p object)
    (or (gethash key object)
        (gethash (intern key) object)
        (gethash (intern (concat ":" key)) object)))
   ((and (listp object)
         (keywordp (car object)))
    (plist-get object (intern (concat ":" key))))
   ((listp object)
    (or (alist-get key object nil nil #'string=)
        (alist-get (intern key) object nil nil #'eq)
        (alist-get (intern (concat ":" key)) object nil nil #'eq)))))

(defun soma-jupyter-bench--jsonish (object)
  (cond
   ((hash-table-p object)
    (let (items)
      (maphash
       (lambda (key value)
         (push (cons (soma-jupyter-bench--key-string key)
                     (soma-jupyter-bench--jsonish value))
               items))
       object)
      (nreverse items)))
   ((and (listp object)
         (keywordp (car object)))
    (let (items)
      (while object
        (push (cons (soma-jupyter-bench--key-string (car object))
                    (soma-jupyter-bench--jsonish (cadr object)))
              items)
        (setq object (cddr object)))
      (nreverse items)))
   ((and (listp object)
         (consp (car object)))
    (mapcar
     (lambda (item)
       (cons (soma-jupyter-bench--key-string (car item))
             (soma-jupyter-bench--jsonish (cdr item))))
     object))
   ((vectorp object)
    (mapcar #'soma-jupyter-bench--jsonish object))
   (t
    object)))

(defun soma-jupyter-bench--keys (object)
  (cond
   ((hash-table-p object)
    (let (keys)
      (maphash
       (lambda (key _value)
         (push (soma-jupyter-bench--key-string key) keys))
       object)
      (sort keys #'string<)))
   ((and (listp object)
         (keywordp (car object)))
    (let (keys)
      (while object
        (push (soma-jupyter-bench--key-string (car object)) keys)
        (setq object (cddr object)))
      (sort keys #'string<)))
   ((and (listp object)
         (consp (car object)))
    (sort (mapcar (lambda (item)
                    (soma-jupyter-bench--key-string (car item)))
                  object)
          #'string<))))

(defun soma-jupyter-bench--check (name passed detail)
  (soma-jupyter-bench--alist
   (cons "name" name)
   (cons "passed" (soma-jupyter-bench--bool passed))
   (cons "detail" detail)))

(defun soma-jupyter-bench--all-checks-pass-p (checks)
  (cl-every
   (lambda (check)
     (eq (soma-jupyter-bench--get check "passed") t))
   checks))

(defun soma-jupyter-bench--rootfs-owned-path-p (path)
  (and (stringp path)
       (file-name-absolute-p path)
       (not (string-empty-p path))
       (not (string-prefix-p "/nix/" path))
       (not (string-prefix-p "/data02/home/" path))
       (not (string-prefix-p "/workspace/" path))
       (not (string-prefix-p "/project/" path))))

(defun soma-jupyter-bench--doom-straight-build-dir ()
  (expand-file-name
   (format "~/.emacs.d/.local/straight/build-%d.%d"
           emacs-major-version emacs-minor-version)))

(defun soma-jupyter-bench--add-straight-build-paths ()
  (let ((build (soma-jupyter-bench--doom-straight-build-dir)))
    (when (file-directory-p build)
      (dolist (dir (directory-files build t directory-files-no-dot-files-regexp))
        (when (file-directory-p dir)
          (add-to-list 'load-path dir))))))

(defun soma-jupyter-bench--load-jupyter ()
  (soma-jupyter-bench--add-straight-build-paths)
  (require 'python)
  (require 'jupyter)
  (require 'jupyter-client)
  (require 'jupyter-messages)
  (require 'jupyter-repl))

(defun soma-jupyter-bench--process-output (process)
  (let ((buffer (and process (process-buffer process))))
    (if (buffer-live-p buffer)
        (with-current-buffer buffer
          (buffer-substring-no-properties (point-min) (point-max)))
      "")))

(defun soma-jupyter-bench--write-fixture-helper (path)
  (with-temp-file path
    (insert
     "import gc, json, os, pathlib, sys, time, traceback\n"
     "from jupyter_client import KernelManager\n"
     "\n"
     "tmp = pathlib.Path(sys.argv[1])\n"
     "token = sys.argv[2]\n"
     "ready_path = tmp / 'ready.json'\n"
     "stop_path = tmp / 'stop'\n"
     "km = KernelManager(kernel_name='python3')\n"
     "kc = None\n"
     "\n"
     "def write_ready(payload):\n"
     "    ready_path.write_text(json.dumps(payload, sort_keys=True) + '\\n', encoding='utf-8')\n"
     "    sys.stdout.write('SOMA_FIXTURE_READY:' + str(ready_path) + '\\n')\n"
     "    sys.stdout.flush()\n"
     "\n"
     "def execute(code, timeout=30):\n"
     "    msg_id = kc.execute(code, store_history=False, allow_stdin=False)\n"
     "    streams = []\n"
     "    errors = []\n"
     "    idle = False\n"
     "    deadline = time.time() + timeout\n"
     "    while time.time() < deadline and not idle:\n"
     "        try:\n"
     "            msg = kc.get_iopub_msg(timeout=0.2)\n"
     "        except Exception:\n"
     "            continue\n"
     "        if msg.get('parent_header', {}).get('msg_id') != msg_id:\n"
     "            continue\n"
     "        msg_type = msg.get('header', {}).get('msg_type')\n"
     "        content = msg.get('content', {})\n"
     "        if msg_type == 'stream':\n"
     "            streams.append(content.get('text', ''))\n"
     "        elif msg_type == 'error':\n"
     "            errors.append(content)\n"
     "        elif msg_type == 'status' and content.get('execution_state') == 'idle':\n"
     "            idle = True\n"
     "    reply = None\n"
     "    deadline = time.time() + timeout\n"
     "    while time.time() < deadline:\n"
     "        try:\n"
     "            candidate = kc.get_shell_msg(timeout=0.2)\n"
     "        except Exception:\n"
     "            continue\n"
     "        if candidate.get('parent_header', {}).get('msg_id') == msg_id:\n"
     "            reply = candidate\n"
     "            break\n"
     "    return {\n"
     "        'msg_id': msg_id,\n"
     "        'idle': idle,\n"
     "        'reply_parent': reply.get('parent_header', {}).get('msg_id') if reply else '',\n"
     "        'reply_status': reply.get('content', {}).get('status') if reply else '',\n"
     "        'streams': streams,\n"
     "        'errors': errors,\n"
     "    }\n"
     "\n"
     "try:\n"
     "    km.start_kernel(cwd=os.getcwd(), env=os.environ.copy())\n"
     "    kc = km.client()\n"
     "    kc.start_channels()\n"
     "    kc.wait_for_ready(timeout=30)\n"
     "    prelude = (\n"
     "        \"import os, sys\\n\"\n"
     "        \"SOMA_BENCH_RUN_ID = os.environ.get('SOMA_INSULA_RUN_ID', '')\\n\"\n"
     "        f\"SOMA_BENCH_KERNEL_TOKEN = {token!r}\\n\"\n"
     "        \"SOMA_BENCH_COUNTER = 0\\n\"\n"
     "        \"SOMA_TRAINING_EXECUTIONS = 0\\n\"\n"
     "    )\n"
     "    prelude_result = execute(prelude)\n"
     "    provisioner = getattr(km, 'provisioner', None)\n"
     "    kernel_pid = getattr(provisioner, 'pid', None)\n"
     "    kernel_exe = ''\n"
     "    if kernel_pid:\n"
     "        proc_exe = pathlib.Path('/proc') / str(kernel_pid) / 'exe'\n"
     "        if proc_exe.exists():\n"
     "            kernel_exe = os.path.realpath(str(proc_exe))\n"
     "    connection_file = pathlib.Path(km.connection_file)\n"
     "    if not connection_file.is_absolute():\n"
     "        connection_file = connection_file.resolve()\n"
     "    write_ready({\n"
     "        'ok': bool(prelude_result['idle'] and prelude_result['reply_status'] == 'ok'),\n"
     "        'connection_file': str(connection_file),\n"
     "        'helper_executable': sys.executable,\n"
     "        'helper_prefix': sys.prefix,\n"
     "        'kernel_pid': kernel_pid,\n"
     "        'kernel_exe': kernel_exe,\n"
     "        'kernel_token': token,\n"
     "        'run_id': os.environ.get('SOMA_INSULA_RUN_ID', ''),\n"
     "        'plan': os.environ.get('SOMA_INSULA_PLAN', ''),\n"
     "        'cwd': os.getcwd(),\n"
     "        'prelude': prelude_result,\n"
     "    })\n"
     "    while not stop_path.exists():\n"
     "        time.sleep(0.1)\n"
     "except Exception as exc:\n"
     "    write_ready({'ok': False, 'error': repr(exc), 'traceback': traceback.format_exc()[-3000:]})\n"
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
     "    gc.collect()\n")))

(defun soma-jupyter-bench--fixture-get (fixture key)
  (soma-jupyter-bench--get fixture key))

(defun soma-jupyter-bench--start-fixture (name)
  (let* ((tmp (make-temp-file (format "soma-jupyter-%s-" name) t))
         (helper (expand-file-name "kernel_fixture.py" tmp))
         (ready-path (expand-file-name "ready.json" tmp))
         (stop-path (expand-file-name "stop" tmp))
         (python (or (executable-find "python3")
                     (error "python3 is required in the Insula rootfs")))
         (token (format "%s-%s-%06x"
                        (or (getenv "SOMA_INSULA_RUN_ID") "no-run")
                        name
                        (random #x1000000)))
         (buffer (generate-new-buffer (format " *soma-jupyter-%s-fixture*" name)))
         process
         ready)
    (soma-jupyter-bench--write-fixture-helper helper)
    (setq process
          (make-process
           :name (format "soma-jupyter-%s-fixture" name)
           :buffer buffer
           :command (list python helper tmp token)
           :connection-type 'pipe
           :noquery t))
    (let ((deadline (+ (float-time) 45)))
      (while (and (not ready) (< (float-time) deadline))
        (accept-process-output process 0.1)
        (when (file-readable-p ready-path)
          (setq ready
                (condition-case nil
                    (soma-jupyter-bench--read-json-file ready-path)
                  (error nil))))
        (when (and (not ready)
                   (not (process-live-p process))
                   (not (file-readable-p ready-path)))
          (error "Jupyter fixture exited before ready: %s"
                 (soma-jupyter-bench--process-output process)))))
    (unless ready
      (error "Timed out waiting for Jupyter fixture: %s"
             (soma-jupyter-bench--process-output process)))
    (unless (eq (soma-jupyter-bench--get ready "ok") t)
      (error "Jupyter fixture failed to initialize: %S output=%s"
             ready
             (soma-jupyter-bench--process-output process)))
    (soma-jupyter-bench--alist
     (cons "name" name)
     (cons "tmp" tmp)
     (cons "ready_path" ready-path)
     (cons "stop_path" stop-path)
     (cons "process" process)
     (cons "process_buffer" buffer)
     (cons "ready" ready)
     (cons "token" token))))

(defun soma-jupyter-bench--stop-fixture (fixture)
  (when fixture
    (let* ((process (soma-jupyter-bench--fixture-get fixture "process"))
           (buffer (soma-jupyter-bench--fixture-get fixture "process_buffer"))
           (stop-path (soma-jupyter-bench--fixture-get fixture "stop_path"))
           (tmp (soma-jupyter-bench--fixture-get fixture "tmp")))
      (when stop-path
        (ignore-errors
          (with-temp-file stop-path
            (insert "stop\n"))))
      (when process
        (let ((deadline (+ (float-time) 8)))
          (while (and (process-live-p process)
                      (< (float-time) deadline))
            (accept-process-output process 0.1)))
        (when (process-live-p process)
          (delete-process process)))
      (when (buffer-live-p buffer)
        (kill-buffer buffer))
      (when (and tmp (file-directory-p tmp))
        (delete-directory tmp t)))))

(defun soma-jupyter-bench--public-fixture (fixture)
  (let ((ready (soma-jupyter-bench--fixture-get fixture "ready")))
    (soma-jupyter-bench--alist
     (cons "name" (soma-jupyter-bench--fixture-get fixture "name"))
     (cons "token" (soma-jupyter-bench--fixture-get fixture "token"))
     (cons "ready" ready)
     (cons "process_output"
           (soma-jupyter-bench--process-output
            (soma-jupyter-bench--fixture-get fixture "process"))))))

(defun soma-jupyter-bench--connect-existing (fixture name)
  (let* ((ready (soma-jupyter-bench--fixture-get fixture "ready"))
         (connection-file (soma-jupyter-bench--get ready "connection_file"))
         (client (jupyter-connect-repl
                  connection-file
                  (format "soma-%s" name)
                  nil
                  'jupyter-repl-client
                  nil)))
    client))

(defun soma-jupyter-bench--disconnect-client (client &optional shutdown)
  (when client
    (when shutdown
      (ignore-errors
        (jupyter-shutdown-kernel client)))
    (ignore-errors
      (jupyter-disconnect client))
    (let ((buffer (ignore-errors (oref client buffer))))
      (when (buffer-live-p buffer)
        (with-current-buffer buffer
          (remove-hook 'kill-buffer-query-functions
                       #'jupyter-repl-kill-buffer-query-function t)
          (let ((kill-buffer-query-functions nil))
            (kill-buffer buffer)))))))

(defun soma-jupyter-bench--source-code (&optional slow)
  (concat
   "import json, os, sys\n"
   (when slow "import time\n")
   (when slow "time.sleep(0.6)\n")
   "SOMA_BENCH_COUNTER += 1\n"
   "print('SOMA_SELECTED_JSON:' + json.dumps({\n"
   "    'counter': SOMA_BENCH_COUNTER,\n"
   "    'kernel_token': SOMA_BENCH_KERNEL_TOKEN,\n"
   "    'run_id': SOMA_BENCH_RUN_ID,\n"
   "    'training_executions': SOMA_TRAINING_EXECUTIONS,\n"
   "}, sort_keys=True))\n"))

(defun soma-jupyter-bench--training-code ()
  (concat
   "SOMA_TRAINING_EXECUTIONS += 1\n"
   "print('SOMA_TRAINING_EXECUTED')\n"))

(defun soma-jupyter-bench--display-update-code (display-id)
  (concat
   "import json\n"
   "from IPython.display import display, update_display\n"
   "SOMA_BENCH_COUNTER += 1\n"
   "_soma_display_id = " (json-encode display-id) "\n"
   "_soma_initial = {\n"
   "    'stage': 'initial',\n"
   "    'kernel_token': SOMA_BENCH_KERNEL_TOKEN,\n"
   "    'display_id': _soma_display_id,\n"
   "}\n"
   "_soma_updated = {\n"
   "    'stage': 'updated',\n"
   "    'kernel_token': SOMA_BENCH_KERNEL_TOKEN,\n"
   "    'display_id': _soma_display_id,\n"
   "}\n"
   "display({\n"
   "    'application/json': _soma_initial,\n"
   "    'text/plain': json.dumps(_soma_initial, sort_keys=True),\n"
   "}, raw=True, display_id=_soma_display_id)\n"
   "update_display({\n"
   "    'application/json': _soma_updated,\n"
   "    'text/plain': json.dumps(_soma_updated, sort_keys=True),\n"
   "}, raw=True, display_id=_soma_display_id)\n"
   "print('SOMA_SELECTED_JSON:' + json.dumps({\n"
   "    'counter': SOMA_BENCH_COUNTER,\n"
   "    'kernel_token': SOMA_BENCH_KERNEL_TOKEN,\n"
   "    'run_id': SOMA_BENCH_RUN_ID,\n"
   "    'training_executions': SOMA_TRAINING_EXECUTIONS,\n"
   "}, sort_keys=True))\n"))

(defun soma-jupyter-bench--make-source-with-selected (name selected)
  (let* ((buffer (generate-new-buffer (format "soma-jupyter-%s.py" name)))
         (training (soma-jupyter-bench--training-code))
         beg end)
    (with-current-buffer buffer
      (insert "# soma-selected-begin\n")
      (setq beg (point))
      (insert selected)
      (setq end (point))
      (insert "# soma-selected-end\n\n")
      (insert "# soma-training-cell\n")
      (insert training))
    (soma-jupyter-bench--alist
     (cons "buffer" buffer)
     (cons "beg" beg)
     (cons "end" end)
     (cons "selected_code" selected)
     (cons "training_code" training))))

(defun soma-jupyter-bench--make-source (name &optional slow)
  (soma-jupyter-bench--make-source-with-selected
   name
   (soma-jupyter-bench--source-code slow)))

(defun soma-jupyter-bench--source-get (source key)
  (soma-jupyter-bench--get source key))

(defun soma-jupyter-bench--public-source (source)
  (let ((buffer (soma-jupyter-bench--source-get source "buffer"))
        (selected (soma-jupyter-bench--source-get source "selected_code")))
    (soma-jupyter-bench--alist
     (cons "buffer" (and (buffer-live-p buffer) (buffer-name buffer)))
     (cons "beg" (soma-jupyter-bench--source-get source "beg"))
     (cons "end" (soma-jupyter-bench--source-get source "end"))
     (cons "selected_sha256" (secure-hash 'sha256 selected))
     (cons "selected_code" selected))))

(defun soma-jupyter-bench--prepare-source-mode (source client)
  (let ((buffer (soma-jupyter-bench--source-get source "buffer")))
    (with-current-buffer buffer
      (let ((mode (ignore-errors (jupyter-kernel-language-mode client))))
        (cond
         ((and (symbolp mode) (fboundp mode))
          (funcall mode))
         ((fboundp 'python-mode)
          (python-mode)))))))

(defun soma-jupyter-bench--associate-source (source client)
  (soma-jupyter-bench--prepare-source-mode source client)
  (with-current-buffer (soma-jupyter-bench--source-get source "buffer")
    (jupyter-repl-associate-buffer client)
    (soma-jupyter-bench--alist
     (cons "buffer" (buffer-name))
     (cons "major_mode" (symbol-name major-mode))
     (cons "associated"
           (soma-jupyter-bench--bool (eq jupyter-current-client client)))
     (cons "interaction_mode"
           (soma-jupyter-bench--bool
            (and (boundp 'jupyter-repl-interaction-mode)
                 jupyter-repl-interaction-mode))))))

(defun soma-jupyter-bench--eval-region-capture (source timeout)
  (let ((buffer (soma-jupyter-bench--source-get source "buffer"))
        (beg (soma-jupyter-bench--source-get source "beg"))
        (end (soma-jupyter-bench--source-get source "end"))
        (submitted nil)
        (req nil)
        (error-text "")
        (idle nil))
    (with-current-buffer buffer
      (let ((orig (symbol-function 'jupyter-eval-string)))
        (condition-case err
            (cl-letf (((symbol-function 'jupyter-eval-string)
                       (lambda (str &optional insert rb re)
                         (setq submitted str)
                         (funcall orig str insert rb re))))
              (setq req (jupyter-eval-region nil beg end)))
          (error
           (setq error-text (format "%S" err))))))
    (when req
      (setq idle (jupyter-wait-until-idle req timeout)))
    (cons
     (append
      (soma-jupyter-bench--alist
       (cons "operation" "jupyter-eval-region")
       (cons "buffer" (and (buffer-live-p buffer) (buffer-name buffer)))
       (cons "idle" (soma-jupyter-bench--bool idle))
       (cons "wait_timeout_seconds" timeout)
       (cons "error" error-text)
       (cons "submitted_code" (or submitted "")))
      (soma-jupyter-bench--request-evidence req))
     req)))

(defun soma-jupyter-bench--eval-record-with-final-idle (eval-call timeout)
  (let* ((record (car eval-call))
         (req (cdr eval-call))
         (idle (and req (jupyter-wait-until-idle req timeout))))
    (append
     record
     (list
      (cons "final_idle" (soma-jupyter-bench--bool idle))
      (cons "final_wait_timeout_seconds" timeout)))))

(defun soma-jupyter-bench--stream-payload (streams prefix)
  (catch 'found
    (dolist (text streams)
      (dolist (line (split-string text "\n" t))
        (when (string-prefix-p prefix line)
          (throw
           'found
           (condition-case nil
               (soma-jupyter-bench--read-json-string
                (substring line (length prefix)))
             (error nil))))))))

(defun soma-jupyter-bench--request-evidence (req)
  (let ((messages (and req (jupyter-request-messages req)))
        message-types
        parent-ids
        stream-texts
        display-events
        display-update-events
        reply-status
        reply-parent)
    (cl-loop for msg in messages
             for index from 0 do
      (let* ((type (jupyter-message-type msg))
             (content (jupyter-message-content msg))
             (parent-id (or (jupyter-message-id
                             (jupyter-message-parent-header msg))
                            "")))
        (push type message-types)
        (unless (string-empty-p parent-id)
          (push parent-id parent-ids))
        (when (string= type "stream")
          (push (or (plist-get content :text) "") stream-texts))
        (when (member type '("display_data" "update_display_data"))
          (let* ((data (soma-jupyter-bench--value content "data"))
                 (transient (soma-jupyter-bench--value content "transient"))
                 (event
                  (soma-jupyter-bench--alist
                   (cons "index" index)
                   (cons "message_type" type)
                   (cons "parent_id" parent-id)
                   (cons "display_id"
                         (or (soma-jupyter-bench--value transient "display_id")
                             ""))
                   (cons "data_keys" (or (soma-jupyter-bench--keys data) nil))
                   (cons "application_json"
                         (or (soma-jupyter-bench--jsonish
                              (soma-jupyter-bench--value
                               data "application/json"))
                             :json-false))
                   (cons "text_plain"
                         (or (soma-jupyter-bench--value data "text/plain")
                             "")))))
            (if (string= type "display_data")
                (push event display-events)
              (push event display-update-events))))
        (when (string= type "execute_reply")
          (setq reply-status (or (plist-get content :status) ""))
          (setq reply-parent parent-id))))
    (soma-jupyter-bench--alist
     (cons "request_id" (if req (jupyter-request-id req) ""))
     (cons "request_idle_flag"
           (soma-jupyter-bench--bool (and req (jupyter-request-idle-p req))))
     (cons "message_types" (nreverse message-types))
     (cons "parent_ids" (nreverse parent-ids))
     (cons "stream_texts" (nreverse stream-texts))
     (cons "display_events" (nreverse display-events))
     (cons "display_update_events" (nreverse display-update-events))
     (cons "reply_status" (or reply-status ""))
     (cons "reply_parent" (or reply-parent "")))))

(defun soma-jupyter-bench--state-query (client)
  (let* ((state-path (make-temp-file "soma-jupyter-state-" nil ".json"))
         (code
          (concat
           "import json, os, pathlib, sys\n"
           "pathlib.Path(" (json-encode state-path) ").write_text(json.dumps({\n"
           "    'counter': globals().get('SOMA_BENCH_COUNTER', None),\n"
           "    'training_executions': globals().get('SOMA_TRAINING_EXECUTIONS', None),\n"
           "    'kernel_token': globals().get('SOMA_BENCH_KERNEL_TOKEN', ''),\n"
           "    'run_id': globals().get('SOMA_BENCH_RUN_ID', ''),\n"
           "    'executable': sys.executable,\n"
           "    'prefix': sys.prefix,\n"
           "    'cwd': os.getcwd(),\n"
           "    'plan': os.environ.get('SOMA_INSULA_PLAN', ''),\n"
           "}, sort_keys=True) + '\\n', encoding='utf-8')\n"
           "print('SOMA_STATE_JSON_WRITTEN:" state-path "')\n"))
         req idle error-text payload)
    (condition-case err
        (let ((jupyter-current-client client))
          (setq req (jupyter-eval-string code nil nil nil)))
      (error
       (setq error-text (format "%S" err))))
    (when req
      (setq idle (jupyter-wait-until-idle req 20)))
    (when (file-readable-p state-path)
      (setq payload
            (condition-case nil
                (soma-jupyter-bench--read-json-file state-path)
              (error nil))))
    (prog1
        (append
         (soma-jupyter-bench--alist
          (cons "operation" "jupyter-eval-string")
          (cons "idle" (soma-jupyter-bench--bool idle))
          (cons "state_path" state-path)
          (cons "state_file_exists" (soma-jupyter-bench--bool
                                     (file-readable-p state-path)))
          (cons "error" (or error-text ""))
          (cons "payload" (or payload :json-false)))
         (soma-jupyter-bench--request-evidence req))
      (when (file-exists-p state-path)
        (delete-file state-path)))))

(defun soma-jupyter-bench--client-session-id (client)
  (or (ignore-errors
        (jupyter-session-id (oref client session)))
      ""))

(defun soma-jupyter-bench--observation (fixture client source)
  (let* ((ready (soma-jupyter-bench--fixture-get fixture "ready"))
         (source-buffer (soma-jupyter-bench--source-get source "buffer"))
         (repl-buffer (ignore-errors (oref client buffer))))
    (soma-jupyter-bench--alist
     (cons "run_id" (or (getenv "SOMA_INSULA_RUN_ID") ""))
     (cons "plan" (or (getenv "SOMA_INSULA_PLAN") ""))
     (cons "connection_file" (soma-jupyter-bench--get ready "connection_file"))
     (cons "kernel_pid" (soma-jupyter-bench--get ready "kernel_pid"))
     (cons "kernel_exe" (soma-jupyter-bench--get ready "kernel_exe"))
     (cons "kernel_token" (soma-jupyter-bench--fixture-get fixture "token"))
     (cons "client_class"
           (if client (symbol-name (eieio-object-class client)) ""))
     (cons "client_session_id" (if client
                                   (soma-jupyter-bench--client-session-id client)
                                 ""))
     (cons "source_buffer"
           (and (buffer-live-p source-buffer) (buffer-name source-buffer)))
     (cons "source_major_mode"
           (and (buffer-live-p source-buffer)
                (with-current-buffer source-buffer (symbol-name major-mode))))
     (cons "repl_buffer"
           (and (buffer-live-p repl-buffer) (buffer-name repl-buffer))))))

(defun soma-jupyter-bench--grade (name intended-fixture source action eval-record state-query)
  (let* ((ready (soma-jupyter-bench--fixture-get intended-fixture "ready"))
         (expected-token (soma-jupyter-bench--fixture-get intended-fixture "token"))
         (expected-connection-file
          (soma-jupyter-bench--get ready "connection_file"))
         (expected-source-buffer
          (buffer-name (soma-jupyter-bench--source-get source "buffer")))
         (expected-code (soma-jupyter-bench--source-get source "selected_code"))
         (payload (soma-jupyter-bench--get state-query "payload"))
         (payload-token (and (listp payload)
                             (or (soma-jupyter-bench--get payload "kernel_token") "")))
         (payload-run-id (and (listp payload)
                              (or (soma-jupyter-bench--get payload "run_id") "")))
         (payload-counter (and (listp payload)
                               (soma-jupyter-bench--get payload "counter")))
         (payload-training (and (listp payload)
                                (soma-jupyter-bench--get payload "training_executions")))
         (payload-executable (and (listp payload)
                                  (or (soma-jupyter-bench--get payload "executable") "")))
         (payload-cwd (and (listp payload)
                           (or (soma-jupyter-bench--get payload "cwd") "")))
         (payload-plan (and (listp payload)
                            (or (soma-jupyter-bench--get payload "plan") "")))
         (run-id (or (getenv "SOMA_INSULA_RUN_ID") ""))
         (submitted (or (soma-jupyter-bench--get eval-record "submitted_code") ""))
         (eval-idle (eq (soma-jupyter-bench--get eval-record "idle") t))
         (eval-final-idle (eq (soma-jupyter-bench--get eval-record "final_idle") t))
         (state-idle (eq (soma-jupyter-bench--get state-query "idle") t))
         (state-reply-ok
          (string= (or (soma-jupyter-bench--get state-query "reply_status") "")
                   "ok"))
         (state-reply-parent
          (or (soma-jupyter-bench--get state-query "reply_parent") ""))
         (state-request-id
          (or (soma-jupyter-bench--get state-query "request_id") ""))
         (state-parents
          (or (soma-jupyter-bench--get state-query "parent_ids") nil))
         (retry-count (or (soma-jupyter-bench--get action "retry_count") 0))
         (timeout-state (or (soma-jupyter-bench--get action "timeout_state") ""))
         (checks
          (list
           (soma-jupyter-bench--check
            "fixture kernel existed before Emacs connected"
            (and (eq (soma-jupyter-bench--get ready "ok") t)
                 (string= expected-token
                          (or (soma-jupyter-bench--get ready "kernel_token") "")))
            (format "%S" ready))
           (soma-jupyter-bench--check
            "kernel process executable is rootfs-owned"
            (soma-jupyter-bench--rootfs-owned-path-p
             (or (soma-jupyter-bench--get ready "kernel_exe") ""))
            (or (soma-jupyter-bench--get ready "kernel_exe") ""))
           (soma-jupyter-bench--check
            "action connected through existing connection file"
            (and (string= (or (soma-jupyter-bench--get action "connect_operation") "")
                          "jupyter-connect-repl")
                 (string= (or (soma-jupyter-bench--get action "connection_file") "")
                          expected-connection-file))
            (format "action=%S expected=%S" action expected-connection-file))
           (soma-jupyter-bench--check
            "source buffer associated with intended REPL client"
            (and (eq (soma-jupyter-bench--get action "associated") t)
                 (string= (or (soma-jupyter-bench--get action "associated_buffer") "")
                          expected-source-buffer))
            (format "action=%S expected-source=%S" action expected-source-buffer))
           (soma-jupyter-bench--check
            "action evaluated the intended source buffer"
            (string= (or (soma-jupyter-bench--get action "source_buffer") "")
                     expected-source-buffer)
            (format "action=%S expected-source=%S" action expected-source-buffer))
           (soma-jupyter-bench--check
            "operation used jupyter-eval-region"
            (string= (or (soma-jupyter-bench--get eval-record "operation") "")
                     "jupyter-eval-region")
            (format "%S" eval-record))
           (soma-jupyter-bench--check
            "selected-region request id reached idle"
            (and (not (string-empty-p
                       (or (soma-jupyter-bench--get eval-record "request_id") "")))
                 (or eval-idle eval-final-idle))
            (format "%S" eval-record))
           (soma-jupyter-bench--check
            "submitted exact selected region"
            (string= submitted expected-code)
            (format "submitted-sha=%s expected-sha=%s submitted=%S"
                    (secure-hash 'sha256 submitted)
                    (secure-hash 'sha256 expected-code)
                    submitted))
           (soma-jupyter-bench--check
            "submitted code excludes training cell"
            (not (string-match-p "SOMA_TRAINING_EXECUTIONS += 1"
                                 submitted))
            submitted)
           (soma-jupyter-bench--check
            "timeout after submit is treated as outcome_unknown without retry"
            (or (string-empty-p timeout-state)
                (and (string= timeout-state "outcome_unknown")
                     (= retry-count 0)))
            (format "timeout_state=%S retry_count=%S"
                    timeout-state retry-count))
           (soma-jupyter-bench--check
            "state query reached ok reply and idle"
            (and state-idle
                 state-reply-ok
                 (or (string= state-reply-parent state-request-id)
                     (member state-request-id state-parents)))
            (format "%S" state-query))
           (soma-jupyter-bench--check
            "kernel continuity token matches intended existing kernel"
            (string= payload-token expected-token)
            (format "payload-token=%S expected=%S payload=%S"
                    payload-token expected-token payload))
           (soma-jupyter-bench--check
            "kernel sees stable Insula run id"
            (and (not (string-empty-p run-id))
                 (string= payload-run-id run-id))
            (format "run-id=%S payload-run-id=%S" run-id payload-run-id))
           (soma-jupyter-bench--check
            "selected side effect occurred exactly once"
            (equal payload-counter 1)
            (format "counter=%S payload=%S" payload-counter payload))
           (soma-jupyter-bench--check
            "training cell was not executed"
            (equal payload-training 0)
            (format "training=%S payload=%S" payload-training payload))
           (soma-jupyter-bench--check
            "kernel payload is rootfs/runtime owned"
            (and (soma-jupyter-bench--rootfs-owned-path-p payload-executable)
                 (string= payload-cwd "/workspace/soma")
                 (string-prefix-p "/workspace/soma/" payload-plan))
            (format "exe=%S cwd=%S plan=%S"
                    payload-executable payload-cwd payload-plan)))))
    (soma-jupyter-bench--alist
     (cons "name" name)
     (cons "checks" checks)
     (cons "action" action)
     (cons "evaluation" eval-record)
     (cons "state_query" state-query)
     (cons "source" (soma-jupyter-bench--public-source source))
     (cons "fixture" (soma-jupyter-bench--public-fixture intended-fixture)))))

(defun soma-jupyter-bench--display-update-checks
    (eval-record expected-display-id expected-token)
  (let* ((display-events
          (or (soma-jupyter-bench--get eval-record "display_events") nil))
         (display-update-events
          (or (soma-jupyter-bench--get eval-record "display_update_events") nil))
         (all-events
          (sort (append (copy-sequence display-events)
                        (copy-sequence display-update-events))
                (lambda (left right)
                  (< (or (soma-jupyter-bench--get left "index") -1)
                     (or (soma-jupyter-bench--get right "index") -1)))))
         (request-id (or (soma-jupyter-bench--get eval-record "request_id") ""))
         (first-display (car display-events))
         (first-update (car display-update-events))
         (first-display-data
          (and first-display
               (soma-jupyter-bench--get first-display "application_json")))
         (first-update-data
          (and first-update
               (soma-jupyter-bench--get first-update "application_json"))))
    (list
     (soma-jupyter-bench--check
      "display/update messages were captured separately"
      (and (= (length display-events) 1)
           (= (length display-update-events) 1))
      (format "display=%S update=%S"
              display-events display-update-events))
     (soma-jupyter-bench--check
      "display/update messages retain request parent ids"
      (and (not (string-empty-p request-id))
           (cl-every
            (lambda (event)
              (string= (or (soma-jupyter-bench--get event "parent_id") "")
                       request-id))
            all-events))
      (format "request=%S events=%S" request-id all-events))
     (soma-jupyter-bench--check
      "display/update messages retain one display id"
      (and (string= (or (soma-jupyter-bench--get first-display "display_id") "")
                    expected-display-id)
           (string= (or (soma-jupyter-bench--get first-update "display_id") "")
                    expected-display-id))
      (format "expected=%S display=%S update=%S"
              expected-display-id first-display first-update))
     (soma-jupyter-bench--check
      "display payload precedes update payload"
      (and first-display first-update
           (< (or (soma-jupyter-bench--get first-display "index") -1)
              (or (soma-jupyter-bench--get first-update "index") -1)))
      (format "events=%S" all-events))
     (soma-jupyter-bench--check
      "display payload carries initial state"
      (and (listp first-display-data)
           (string= (or (soma-jupyter-bench--get first-display-data "stage") "")
                    "initial")
           (string= (or (soma-jupyter-bench--get first-display-data "kernel_token") "")
                    expected-token)
           (string= (or (soma-jupyter-bench--get first-display-data "display_id") "")
                    expected-display-id))
      (format "%S" first-display-data))
     (soma-jupyter-bench--check
      "update payload carries final state"
      (and (listp first-update-data)
           (string= (or (soma-jupyter-bench--get first-update-data "stage") "")
                    "updated")
           (string= (or (soma-jupyter-bench--get first-update-data "kernel_token") "")
                    expected-token)
           (string= (or (soma-jupyter-bench--get first-update-data "display_id") "")
                    expected-display-id))
      (format "%S" first-update-data)))))

(defun soma-jupyter-bench--grade-display-update
    (name intended-fixture source action eval-record state-query display-id)
  (let* ((record (soma-jupyter-bench--grade
                  name intended-fixture source action eval-record state-query))
         (checks (soma-jupyter-bench--get record "checks"))
         (expected-token (soma-jupyter-bench--fixture-get
                          intended-fixture "token")))
    (setcdr
     (assoc "checks" record)
     (append checks
             (soma-jupyter-bench--display-update-checks
              eval-record display-id expected-token)))
    record))

(defun soma-jupyter-bench--scenario-correct-existing ()
  (let ((fixture nil)
        (client nil)
        (source nil))
    (unwind-protect
        (progn
          (setq fixture (soma-jupyter-bench--start-fixture "correct"))
          (setq client (soma-jupyter-bench--connect-existing fixture "correct"))
          (setq source (soma-jupyter-bench--make-source "correct"))
          (let* ((assoc (soma-jupyter-bench--associate-source source client))
                 (observation (soma-jupyter-bench--observation fixture client source))
                 (eval-call (soma-jupyter-bench--eval-region-capture source 20))
                 (eval-record (car eval-call))
                 (state-query (soma-jupyter-bench--state-query client))
                 (ready (soma-jupyter-bench--fixture-get fixture "ready"))
                 (action
                  (soma-jupyter-bench--alist
                   (cons "connect_operation" "jupyter-connect-repl")
                   (cons "connection_file"
                         (soma-jupyter-bench--get ready "connection_file"))
                   (cons "associate_operation" "jupyter-repl-associate-buffer")
                   (cons "associated"
                         (soma-jupyter-bench--get assoc "associated"))
                   (cons "associated_buffer"
                         (soma-jupyter-bench--get assoc "buffer"))
                   (cons "source_buffer"
                         (buffer-name
                          (soma-jupyter-bench--source-get source "buffer")))
                   (cons "retry_count" 0))))
            (cons (cons "observation" observation)
                  (soma-jupyter-bench--grade
                   "correct-existing-kernel-selected-region"
                   fixture source action eval-record state-query))))
      (when client
        (soma-jupyter-bench--disconnect-client client))
      (when (and source (buffer-live-p (soma-jupyter-bench--source-get source "buffer")))
        (kill-buffer (soma-jupyter-bench--source-get source "buffer")))
      (soma-jupyter-bench--stop-fixture fixture))))

(defun soma-jupyter-bench--scenario-missing-association ()
  (let ((fixture nil)
        (client nil)
        (source nil))
    (unwind-protect
        (progn
          (setq fixture (soma-jupyter-bench--start-fixture "missing-assoc"))
          (setq client (soma-jupyter-bench--connect-existing fixture "missing-assoc"))
          (setq source (soma-jupyter-bench--make-source "missing-assoc"))
          (soma-jupyter-bench--prepare-source-mode source client)
          (let* ((observation (soma-jupyter-bench--observation fixture client source))
                 (eval-call (soma-jupyter-bench--eval-region-capture source 2))
                 (eval-record (car eval-call))
                 (state-query (soma-jupyter-bench--state-query client))
                 (ready (soma-jupyter-bench--fixture-get fixture "ready"))
                 (action
                  (soma-jupyter-bench--alist
                   (cons "connect_operation" "jupyter-connect-repl")
                   (cons "connection_file"
                         (soma-jupyter-bench--get ready "connection_file"))
                   (cons "associate_operation" "none")
                   (cons "associated" :json-false)
                   (cons "associated_buffer" "")
                   (cons "source_buffer"
                         (buffer-name
                          (soma-jupyter-bench--source-get source "buffer")))
                   (cons "retry_count" 0))))
            (cons (cons "observation" observation)
                  (soma-jupyter-bench--grade
                   "bad-missing-association"
                   fixture source action eval-record state-query))))
      (when client
        (soma-jupyter-bench--disconnect-client client))
      (when (and source (buffer-live-p (soma-jupyter-bench--source-get source "buffer")))
        (kill-buffer (soma-jupyter-bench--source-get source "buffer")))
      (soma-jupyter-bench--stop-fixture fixture))))

(defun soma-jupyter-bench--scenario-wrong-source-buffer ()
  (let ((fixture nil)
        (client nil)
        (source nil)
        (decoy nil))
    (unwind-protect
        (progn
          (setq fixture (soma-jupyter-bench--start-fixture "wrong-source"))
          (setq client (soma-jupyter-bench--connect-existing fixture "wrong-source"))
          (setq source (soma-jupyter-bench--make-source "wrong-source-intended"))
          (setq decoy (soma-jupyter-bench--make-source "wrong-source-decoy"))
          (let* ((assoc (soma-jupyter-bench--associate-source decoy client))
                 (observation (soma-jupyter-bench--observation fixture client source))
                 (eval-call (soma-jupyter-bench--eval-region-capture decoy 20))
                 (eval-record (car eval-call))
                 (state-query (soma-jupyter-bench--state-query client))
                 (ready (soma-jupyter-bench--fixture-get fixture "ready"))
                 (action
                  (soma-jupyter-bench--alist
                   (cons "connect_operation" "jupyter-connect-repl")
                   (cons "connection_file"
                         (soma-jupyter-bench--get ready "connection_file"))
                   (cons "associate_operation" "jupyter-repl-associate-buffer")
                   (cons "associated"
                         (soma-jupyter-bench--get assoc "associated"))
                   (cons "associated_buffer"
                         (soma-jupyter-bench--get assoc "buffer"))
                   (cons "source_buffer"
                         (buffer-name
                          (soma-jupyter-bench--source-get decoy "buffer")))
                   (cons "retry_count" 0))))
            (cons (cons "observation" observation)
                  (soma-jupyter-bench--grade
                   "bad-wrong-source-buffer"
                   fixture source action eval-record state-query))))
      (when client
        (soma-jupyter-bench--disconnect-client client))
      (dolist (src (list source decoy))
        (when (and src (buffer-live-p (soma-jupyter-bench--source-get src "buffer")))
          (kill-buffer (soma-jupyter-bench--source-get src "buffer"))))
      (soma-jupyter-bench--stop-fixture fixture))))

(defun soma-jupyter-bench--scenario-wrong-region ()
  (let ((fixture nil)
        (client nil)
        (source nil))
    (unwind-protect
        (progn
          (setq fixture (soma-jupyter-bench--start-fixture "wrong-region"))
          (setq client (soma-jupyter-bench--connect-existing fixture "wrong-region"))
          (setq source (soma-jupyter-bench--make-source "wrong-region"))
          (let* ((assoc (soma-jupyter-bench--associate-source source client))
                 (buffer (soma-jupyter-bench--source-get source "buffer"))
                 (observation (soma-jupyter-bench--observation fixture client source))
                 eval-call eval-record state-query action)
            (with-current-buffer buffer
              (setq eval-call
                    (let ((original-beg (soma-jupyter-bench--source-get source "beg"))
                          (original-end (soma-jupyter-bench--source-get source "end")))
                      (setcdr (assoc "beg" source) (point-min))
                      (setcdr (assoc "end" source) (point-max))
                      (unwind-protect
                          (soma-jupyter-bench--eval-region-capture source 20)
                        (setcdr (assoc "beg" source) original-beg)
                        (setcdr (assoc "end" source) original-end)))))
            (setq eval-record (car eval-call))
            (setq state-query (soma-jupyter-bench--state-query client))
            (let ((ready (soma-jupyter-bench--fixture-get fixture "ready")))
              (setq action
                    (soma-jupyter-bench--alist
                     (cons "connect_operation" "jupyter-connect-repl")
                     (cons "connection_file"
                           (soma-jupyter-bench--get ready "connection_file"))
                     (cons "associate_operation" "jupyter-repl-associate-buffer")
                     (cons "associated"
                           (soma-jupyter-bench--get assoc "associated"))
                     (cons "associated_buffer"
                           (soma-jupyter-bench--get assoc "buffer"))
                     (cons "source_buffer" (buffer-name buffer))
                     (cons "region_policy" "whole-buffer")
                     (cons "retry_count" 0))))
            (cons (cons "observation" observation)
                  (soma-jupyter-bench--grade
                   "bad-whole-buffer-region"
                   fixture source action eval-record state-query))))
      (when client
        (soma-jupyter-bench--disconnect-client client))
      (when (and source (buffer-live-p (soma-jupyter-bench--source-get source "buffer")))
        (kill-buffer (soma-jupyter-bench--source-get source "buffer")))
      (soma-jupyter-bench--stop-fixture fixture))))

(defun soma-jupyter-bench--scenario-wrong-kernel ()
  (let ((intended nil)
        (fresh nil)
        (client nil)
        (source nil))
    (unwind-protect
        (progn
          (setq intended (soma-jupyter-bench--start-fixture "intended"))
          (setq fresh (soma-jupyter-bench--start-fixture "fresh"))
          (setq client (soma-jupyter-bench--connect-existing fresh "fresh"))
          (setq source (soma-jupyter-bench--make-source "wrong-kernel"))
          (let* ((assoc (soma-jupyter-bench--associate-source source client))
                 (observation (soma-jupyter-bench--observation intended client source))
                 (eval-call (soma-jupyter-bench--eval-region-capture source 20))
                 (eval-record (car eval-call))
                 (state-query (soma-jupyter-bench--state-query client))
                 (fresh-ready (soma-jupyter-bench--fixture-get fresh "ready"))
                 (action
                  (soma-jupyter-bench--alist
                   (cons "connect_operation" "jupyter-connect-repl")
                   (cons "connection_file"
                         (soma-jupyter-bench--get fresh-ready "connection_file"))
                   (cons "associate_operation" "jupyter-repl-associate-buffer")
                   (cons "associated"
                         (soma-jupyter-bench--get assoc "associated"))
                   (cons "associated_buffer"
                         (soma-jupyter-bench--get assoc "buffer"))
                   (cons "source_buffer"
                         (buffer-name
                          (soma-jupyter-bench--source-get source "buffer")))
                   (cons "kernel_policy" "new-kernel-after-observation")
                   (cons "retry_count" 0))))
            (cons (cons "observation" observation)
                  (soma-jupyter-bench--grade
                   "bad-new-kernel-continuity"
                   intended source action eval-record state-query))))
      (when client
        (soma-jupyter-bench--disconnect-client client))
      (when (and source (buffer-live-p (soma-jupyter-bench--source-get source "buffer")))
        (kill-buffer (soma-jupyter-bench--source-get source "buffer")))
      (soma-jupyter-bench--stop-fixture fresh)
      (soma-jupyter-bench--stop-fixture intended))))

(defun soma-jupyter-bench--scenario-timeout-unknown ()
  (let ((fixture nil)
        (client nil)
        (source nil))
    (unwind-protect
        (progn
          (setq fixture (soma-jupyter-bench--start-fixture "timeout-ok"))
          (setq client (soma-jupyter-bench--connect-existing fixture "timeout-ok"))
          (setq source (soma-jupyter-bench--make-source "timeout-ok" t))
          (let* ((assoc (soma-jupyter-bench--associate-source source client))
                 (observation (soma-jupyter-bench--observation fixture client source))
                 (eval-call (soma-jupyter-bench--eval-region-capture source 0.1))
                 (eval-record (soma-jupyter-bench--eval-record-with-final-idle
                               eval-call 10))
                 (state-query (soma-jupyter-bench--state-query client))
                 (ready (soma-jupyter-bench--fixture-get fixture "ready"))
                 (action
                  (soma-jupyter-bench--alist
                   (cons "connect_operation" "jupyter-connect-repl")
                   (cons "connection_file"
                         (soma-jupyter-bench--get ready "connection_file"))
                   (cons "associate_operation" "jupyter-repl-associate-buffer")
                   (cons "associated"
                         (soma-jupyter-bench--get assoc "associated"))
                   (cons "associated_buffer"
                         (soma-jupyter-bench--get assoc "buffer"))
                   (cons "source_buffer"
                         (buffer-name
                          (soma-jupyter-bench--source-get source "buffer")))
                   (cons "timeout_state" "outcome_unknown")
                   (cons "retry_count" 0))))
            (cons (cons "observation" observation)
                  (soma-jupyter-bench--grade
                   "timeout-outcome-unknown"
                   fixture source action eval-record state-query))))
      (when client
        (soma-jupyter-bench--disconnect-client client))
      (when (and source (buffer-live-p (soma-jupyter-bench--source-get source "buffer")))
        (kill-buffer (soma-jupyter-bench--source-get source "buffer")))
      (soma-jupyter-bench--stop-fixture fixture))))

(defun soma-jupyter-bench--scenario-timeout-retry ()
  (let ((fixture nil)
        (client nil)
        (source nil))
    (unwind-protect
        (progn
          (setq fixture (soma-jupyter-bench--start-fixture "timeout-retry"))
          (setq client (soma-jupyter-bench--connect-existing fixture "timeout-retry"))
          (setq source (soma-jupyter-bench--make-source "timeout-retry" t))
          (let* ((assoc (soma-jupyter-bench--associate-source source client))
                 (observation (soma-jupyter-bench--observation fixture client source))
                 (first-call (soma-jupyter-bench--eval-region-capture source 0.1))
                 (second-call (soma-jupyter-bench--eval-region-capture source 20))
                 (_first-final (soma-jupyter-bench--eval-record-with-final-idle
                                first-call 10))
                 (eval-record (soma-jupyter-bench--eval-record-with-final-idle
                               second-call 10))
                 (state-query (soma-jupyter-bench--state-query client))
                 (ready (soma-jupyter-bench--fixture-get fixture "ready"))
                 (action
                  (soma-jupyter-bench--alist
                   (cons "connect_operation" "jupyter-connect-repl")
                   (cons "connection_file"
                         (soma-jupyter-bench--get ready "connection_file"))
                   (cons "associate_operation" "jupyter-repl-associate-buffer")
                   (cons "associated"
                         (soma-jupyter-bench--get assoc "associated"))
                   (cons "associated_buffer"
                         (soma-jupyter-bench--get assoc "buffer"))
                   (cons "source_buffer"
                         (buffer-name
                          (soma-jupyter-bench--source-get source "buffer")))
                   (cons "timeout_state" "retried_after_timeout")
                   (cons "retry_count" 1)
                   (cons "first_evaluation" (car first-call)))))
            (cons (cons "observation" observation)
                  (soma-jupyter-bench--grade
                   "bad-timeout-retry"
                   fixture source action eval-record state-query))))
      (when client
        (soma-jupyter-bench--disconnect-client client))
      (when (and source (buffer-live-p (soma-jupyter-bench--source-get source "buffer")))
        (kill-buffer (soma-jupyter-bench--source-get source "buffer")))
      (soma-jupyter-bench--stop-fixture fixture))))

(defun soma-jupyter-bench--scenario-display-update ()
  (let ((fixture nil)
        (client nil)
        (source nil))
    (unwind-protect
        (progn
          (setq fixture (soma-jupyter-bench--start-fixture "display-update"))
          (let* ((display-id
                  (concat "soma-display-"
                          (soma-jupyter-bench--fixture-get fixture "token"))))
            (setq client
                  (soma-jupyter-bench--connect-existing fixture "display-update"))
            (setq source
                  (soma-jupyter-bench--make-source-with-selected
                   "display-update"
                   (soma-jupyter-bench--display-update-code display-id)))
            (let* ((assoc (soma-jupyter-bench--associate-source source client))
                   (observation (soma-jupyter-bench--observation fixture client source))
                   (eval-call (soma-jupyter-bench--eval-region-capture source 20))
                   (eval-record (car eval-call))
                   (state-query (soma-jupyter-bench--state-query client))
                   (ready (soma-jupyter-bench--fixture-get fixture "ready"))
                   (action
                    (soma-jupyter-bench--alist
                     (cons "connect_operation" "jupyter-connect-repl")
                     (cons "connection_file"
                           (soma-jupyter-bench--get ready "connection_file"))
                     (cons "associate_operation" "jupyter-repl-associate-buffer")
                     (cons "associated"
                           (soma-jupyter-bench--get assoc "associated"))
                     (cons "associated_buffer"
                           (soma-jupyter-bench--get assoc "buffer"))
                     (cons "source_buffer"
                           (buffer-name
                            (soma-jupyter-bench--source-get source "buffer")))
                     (cons "display_id" display-id)
                     (cons "retry_count" 0))))
              (cons (cons "observation" observation)
                    (soma-jupyter-bench--grade-display-update
                     "display-update-existing-kernel"
                     fixture source action eval-record state-query display-id)))))
      (when client
        (soma-jupyter-bench--disconnect-client client))
      (when (and source (buffer-live-p (soma-jupyter-bench--source-get source "buffer")))
        (kill-buffer (soma-jupyter-bench--source-get source "buffer")))
      (soma-jupyter-bench--stop-fixture fixture))))

(defun soma-jupyter-bench--record-scenario (index id title expected-pass thunk)
  (let* ((result (funcall thunk))
         (checks (soma-jupyter-bench--get result "checks"))
         (policy-pass (soma-jupyter-bench--all-checks-pass-p checks))
         (as-expected (if expected-pass policy-pass (not policy-pass)))
         (record
          (append
           (soma-jupyter-bench--alist
            (cons "id" id)
            (cons "title" title)
            (cons "expected_policy_pass" (soma-jupyter-bench--bool expected-pass))
            (cons "policy_passed_grader" (soma-jupyter-bench--bool policy-pass))
            (cons "as_expected" (soma-jupyter-bench--bool as-expected)))
           result)))
    (push record soma-jupyter-benchmark-results)
    (soma-jupyter-bench--write-json
     (expand-file-name (format "%02d-%s.json" index id)
                       soma-jupyter-benchmark-output-dir)
     record)
    record))

(defun soma-jupyter-bench--insula-checks ()
  (let* ((plan (soma-jupyter-bench--plan))
         (mode (or (getenv "SOMA_INSULA_MODE") ""))
         (plan-mode (or (soma-jupyter-bench--get plan "mode") ""))
         (closure (or (soma-jupyter-bench--get plan "dependency_closure") ""))
         (glibc-owner (or (soma-jupyter-bench--get plan "glibc_owner") ""))
         (benchmark (or (soma-jupyter-bench--get plan "benchmark") ""))
         (tools (or (soma-jupyter-bench--get plan "tool_inventory") nil))
         (jupyter-tool (and tools (soma-jupyter-bench--get tools "jupyter")))
         (python-tool (and tools (soma-jupyter-bench--get tools "python3"))))
    (list
     (soma-jupyter-bench--check
      "SOMA_INSULA_ACTIVE is set"
      (string= (or (getenv "SOMA_INSULA_ACTIVE") "") "1")
      (or (getenv "SOMA_INSULA_ACTIVE") ""))
     (soma-jupyter-bench--check
      "runtime mode is selected rootfs"
      (and (string= mode "selected-container-rootfs")
           (string= plan-mode "selected-container-rootfs-insula"))
      (format "env=%S plan=%S" mode plan-mode))
     (soma-jupyter-bench--check
      "benchmark plan is for existing-kernel Jupyter"
      (string= benchmark "jupyter-existing-kernel")
      benchmark)
     (soma-jupyter-bench--check
      "dependency closure is rootfs-owned"
      (string= closure "rootfs_owned")
      closure)
     (soma-jupyter-bench--check
      "glibc owner is container-image-rootfs"
      (string= glibc-owner "container-image-rootfs")
      glibc-owner)
     (soma-jupyter-bench--check
      "cwd is rootfs repo mount"
      (string= (directory-file-name (expand-file-name default-directory))
               "/workspace/soma")
      default-directory)
     (soma-jupyter-bench--check
      "python and jupyter tools are in rootfs inventory"
      (and python-tool jupyter-tool
           (eq (soma-jupyter-bench--get python-tool "executable") t)
           (eq (soma-jupyter-bench--get jupyter-tool "executable") t)
           (soma-jupyter-bench--rootfs-owned-path-p
            (or (soma-jupyter-bench--get python-tool "path") ""))
           (soma-jupyter-bench--rootfs-owned-path-p
            (or (soma-jupyter-bench--get jupyter-tool "path") "")))
      (format "python3=%S jupyter=%S" python-tool jupyter-tool)))))

(defun soma-jupyter-bench--summary ()
  (let* ((records (reverse soma-jupyter-benchmark-results))
         (total (length records))
         (expected (cl-count-if
                    (lambda (record)
                      (eq (soma-jupyter-bench--get record "as_expected") t))
                    records))
         (insula-checks (soma-jupyter-bench--insula-checks))
         (insula-pass (soma-jupyter-bench--all-checks-pass-p insula-checks))
         (summary
          (soma-jupyter-bench--alist
           (cons "generated_at" (format-time-string "%FT%TZ" nil t))
           (cons "benchmark" "jupyter-existing-kernel")
           (cons "total_scenarios" total)
           (cons "scenarios_as_expected" expected)
           (cons "insula_checks_passed" (soma-jupyter-bench--bool insula-pass))
           (cons "insula_checks" insula-checks)
           (cons "plan" (soma-jupyter-bench--plan))
           (cons "records" records))))
    (soma-jupyter-bench--write-json
     (expand-file-name "summary.json" soma-jupyter-benchmark-output-dir)
     summary)
    (with-temp-file (expand-file-name "summary.md" soma-jupyter-benchmark-output-dir)
      (insert "# Jupyter Existing-Kernel Benchmark Summary\n\n")
      (insert (format "- Generated: %s\n" (format-time-string "%FT%TZ" nil t)))
      (insert "- Benchmark: `jupyter-existing-kernel`\n")
      (insert (format "- Total scenarios: %d\n" total))
      (insert (format "- Scenarios as expected: %d\n" expected))
      (insert (format "- Insula checks passed: `%S`\n" insula-pass))
      (insert (format "- Mode: `%s`\n" (or (getenv "SOMA_INSULA_MODE") "")))
      (insert (format "- Run id: `%s`\n\n" (or (getenv "SOMA_INSULA_RUN_ID") "")))
      (dolist (record records)
        (insert (format "## %s\n\n" (soma-jupyter-bench--get record "title")))
        (insert (format "- ID: `%s`\n" (soma-jupyter-bench--get record "id")))
        (insert (format "- Expected policy pass: `%S`\n"
                        (soma-jupyter-bench--get record "expected_policy_pass")))
        (insert (format "- Policy passed grader: `%S`\n"
                        (soma-jupyter-bench--get record "policy_passed_grader")))
        (insert (format "- As expected: `%S`\n\n"
                        (soma-jupyter-bench--get record "as_expected")))))
    summary))

(defun soma-jupyter-bench-run ()
  (let ((args (seq-remove (lambda (arg) (string= arg "--"))
                          command-line-args-left)))
    (setq soma-jupyter-benchmark-output-dir
          (expand-file-name
           (or (car args)
               "benchmarks/emacs-gym/jupyter/traces/latest"))))
  (make-directory soma-jupyter-benchmark-output-dir t)
  (setq soma-jupyter-benchmark-results nil)
  (soma-jupyter-bench--load-jupyter)
  (soma-jupyter-bench--record-scenario
   1 "correct-existing-kernel"
   "Correct policy attaches to an existing kernel and executes only the selected region"
   t
   #'soma-jupyter-bench--scenario-correct-existing)
  (soma-jupyter-bench--record-scenario
   2 "bad-missing-association"
   "Incorrect policy evaluates without associating the source buffer"
   nil
   #'soma-jupyter-bench--scenario-missing-association)
  (soma-jupyter-bench--record-scenario
   3 "bad-wrong-source-buffer"
   "Incorrect policy associates and executes a similarly named source buffer"
   nil
   #'soma-jupyter-bench--scenario-wrong-source-buffer)
  (soma-jupyter-bench--record-scenario
   4 "bad-whole-buffer-region"
   "Incorrect policy expands the selected region into the training cell"
   nil
   #'soma-jupyter-bench--scenario-wrong-region)
  (soma-jupyter-bench--record-scenario
   5 "bad-new-kernel-continuity"
   "Incorrect policy connects to a new kernel instead of the observed existing kernel"
   nil
   #'soma-jupyter-bench--scenario-wrong-kernel)
  (soma-jupyter-bench--record-scenario
   6 "timeout-outcome-unknown"
   "Correct policy treats a post-submit timeout as outcome_unknown and does not retry"
   t
   #'soma-jupyter-bench--scenario-timeout-unknown)
  (soma-jupyter-bench--record-scenario
   7 "bad-timeout-retry"
   "Incorrect policy retries a non-idempotent region after timeout"
   nil
   #'soma-jupyter-bench--scenario-timeout-retry)
  (soma-jupyter-bench--record-scenario
   8 "display-update-existing-kernel"
   "Correct policy captures display_data and update_display_data for an existing kernel request"
   t
   #'soma-jupyter-bench--scenario-display-update)
  (let* ((summary (soma-jupyter-bench--summary))
         (total (soma-jupyter-bench--get summary "total_scenarios"))
         (expected (soma-jupyter-bench--get summary "scenarios_as_expected"))
         (insula-pass (eq (soma-jupyter-bench--get summary "insula_checks_passed")
                          t)))
    (unless (and (= total expected) insula-pass)
      (princ
       (format "jupyter-benchmark: failed total=%s expected=%s insula=%S\n"
               total expected insula-pass))
      (kill-emacs 1))
    (princ
     (format "jupyter-benchmark: wrote results to %s\n"
             soma-jupyter-benchmark-output-dir))))

(soma-jupyter-bench-run)

;;; run-benchmark.el ends here
