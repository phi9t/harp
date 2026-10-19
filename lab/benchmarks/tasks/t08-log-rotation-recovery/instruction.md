`rotate_logs.sh LOG_DIR RETAIN` rotates `LOG_DIR/app.log`. If `app.log` exists, archive its current contents to `app.log.TIMESTAMP`, where `TIMESTAMP` is `$ROTATE_NOW` when set and otherwise the current UTC timestamp in `YYYYmmddHHMMSS` form. Then leave a new empty `app.log`.

If the target archive name already exists, do not overwrite it; choose a deterministic numeric suffix such as `.1`, `.2`, and so on. After rotation, keep only the newest `RETAIN` archived logs by archive name, not counting the current `app.log`.

Fix `rotate_logs.sh`; keep it shell-only.
