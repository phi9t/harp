`startup.sh ROOT` prepares runtime directories for a local service. After it runs, `ROOT/run` must exist with mode `755`, and `ROOT/run/sessions` must exist with mode `1777`. The sessions directory is shared temporary storage, so the sticky bit must be present. The script must be idempotent: running it again must repair wrong modes without removing existing files.

Verify the behavior as the current unprivileged user; no root privileges are available.
