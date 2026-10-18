-- Explicit opt-in schema v1 -> v2; no historical backend import.
CREATE TABLE codex_configuration (id INTEGER PRIMARY KEY CHECK(id=1), document TEXT NOT NULL);
CREATE TABLE codex_servers (id TEXT PRIMARY KEY, document TEXT NOT NULL, state TEXT NOT NULL);
CREATE TABLE codex_bindings (
 id TEXT PRIMARY KEY, project TEXT NOT NULL REFERENCES projects(id), role TEXT NOT NULL,
 attempt TEXT REFERENCES attempts(id), authority_epoch INTEGER NOT NULL,
 document TEXT NOT NULL, state TEXT NOT NULL,
 UNIQUE(project,role,attempt,authority_epoch)
);
CREATE TABLE codex_operations (
 id TEXT PRIMARY KEY, project TEXT NOT NULL REFERENCES projects(id),
 binding TEXT NOT NULL REFERENCES codex_bindings(id), kind TEXT NOT NULL,
 state TEXT NOT NULL CHECK(state IN ('prepared','applying','confirmed','failed','unknown')),
 intent TEXT NOT NULL, observation TEXT
);
PRAGMA user_version = 2;
