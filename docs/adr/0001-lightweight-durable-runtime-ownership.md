# Use the lightweight durable runtime as the single execution authority

Status: accepted

Harp adopts the imported Corenius Python + SQLite call-journal runtime for its
integrated coding-worker lifecycle, with both Codex and Trae. It supersedes the
other Harp controller efforts as their callers migrate, avoiding competing
admission, recovery and landing authorities. The PostgreSQL-backed controller
version is dropped from the active product. The selected engine provides
Temporal-style replay without a Temporal SDK/server; its existing demonstration
workflows must be extended with real provider, verification and landing support.
Git-managed Markdown remains work intent; operational history and pinned admitted
inputs belong to the durable runtime. Human approval binds the exact verified
candidate and cannot be inferred from an agent role.
