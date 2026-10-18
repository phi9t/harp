Fix the audit log so events recorded from one translation unit are visible from the others.

After `start_session()` runs, `session_summary()` must return `count=2 last=ready`. After `append_session_event("verified")`, `session_summary()` must return `count=3 last=verified`. `reset_session()` must clear the shared log for the next run.

Keep the public audit-log API in `include/audit_log.hpp`; the project uses CMake and Ninja. Do not remove the tests.
