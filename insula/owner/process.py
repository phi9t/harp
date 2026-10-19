"""One launch gate for durable guardians and the service execution owner.

Only trusted single-threaded owners may use this module: Python's preexec_fn is
not thread-safe. The child joins its identity-checked cgroup before exec. Root
ownership descriptors survive fork until the caller's membership marker is
written, preventing recovery from racing a delayed child.
"""

from __future__ import annotations

import subprocess
from typing import Any, Callable


def launch(
    argv: list[str],
    *,
    group: Any = None,
    after_join: Callable[[], None] | None = None,
    **kwargs: Any,
) -> subprocess.Popen:
    """Launch using the caller's admitted capsule command, never a shell.

    The owner supplies stdin/log sinks, immutable environment, ownership FDs and
    namespace argv. This primitive does not infer policy or weaken containment.
    Process-group-only compatibility is reserved for existing diagnostic callers;
    service admission must require a real cgroup.
    """
    if "preexec_fn" in kwargs:
        raise ValueError("launch gate owns preexec_fn")
    if after_join is not None and group is None:
        raise ValueError("membership callback requires cgroup ownership")

    def enter() -> None:
        group.join_current_process()
        if after_join is not None:
            after_join()

    return subprocess.Popen(
        argv, preexec_fn=enter if group is not None else None, **kwargs
    )
