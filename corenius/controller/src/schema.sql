-- Fresh-project schema v1. No import from another backend.
CREATE TABLE projects (
 id TEXT PRIMARY KEY, base TEXT NOT NULL, revision INTEGER NOT NULL CHECK(revision >= 1),
 lead_epoch INTEGER NOT NULL DEFAULT 0, paused INTEGER NOT NULL DEFAULT 0 CHECK(paused IN (0,1))
);
CREATE TABLE briefs (
 id TEXT PRIMARY KEY, project TEXT NOT NULL REFERENCES projects(id), task TEXT NOT NULL,
 work_ref TEXT NOT NULL, source_revision TEXT NOT NULL, body TEXT NOT NULL, digest TEXT NOT NULL
);
CREATE TABLE tasks (
 project TEXT NOT NULL REFERENCES projects(id), id TEXT NOT NULL, brief TEXT NOT NULL REFERENCES briefs(id), work_ref TEXT NOT NULL,
 state TEXT NOT NULL CHECK(state IN ('created','active','submitted','repair-required','integration-required','accepted','landed','aborted')),
 PRIMARY KEY(project,id), UNIQUE(project,work_ref)
);
CREATE TABLE attempts (
 id TEXT PRIMARY KEY, project TEXT NOT NULL, task TEXT NOT NULL, brief TEXT NOT NULL REFERENCES briefs(id), base TEXT NOT NULL,
 state TEXT NOT NULL CHECK(state IN ('starting','running','exited','unknown')), scope_epoch INTEGER NOT NULL DEFAULT 1,
 cancel_requested INTEGER NOT NULL DEFAULT 0 CHECK(cancel_requested IN (0,1)), observation TEXT,
 FOREIGN KEY(project,task) REFERENCES tasks(project,id)
);
CREATE UNIQUE INDEX one_live_task ON attempts(project,task) WHERE state <> 'exited';
-- Two slots for the controller, not two per project. Unknown executions retain these rows.
CREATE TABLE slots (id INTEGER PRIMARY KEY CHECK(id IN (1,2)), attempt TEXT UNIQUE REFERENCES attempts(id));
INSERT INTO slots(id) VALUES (1),(2);
CREATE TABLE principals (
 id TEXT PRIMARY KEY, token_hash TEXT NOT NULL, project TEXT REFERENCES projects(id),
 role TEXT NOT NULL CHECK(role IN ('operator','lead','attempt')), epoch INTEGER NOT NULL,
 attempt TEXT REFERENCES attempts(id)
);
CREATE TABLE operations (
 id TEXT PRIMARY KEY, project TEXT NOT NULL REFERENCES projects(id), attempt TEXT REFERENCES attempts(id),
 kind TEXT NOT NULL, state TEXT NOT NULL CHECK(state IN ('prepared','applying','confirmed','failed','unknown')),
 payload TEXT NOT NULL, observation TEXT
);
CREATE TABLE events (
 seq INTEGER PRIMARY KEY AUTOINCREMENT, project TEXT NOT NULL REFERENCES projects(id),
 task TEXT, audience TEXT, kind TEXT NOT NULL, body TEXT NOT NULL
);
CREATE TABLE receipts (
 project TEXT NOT NULL REFERENCES projects(id), principal TEXT NOT NULL REFERENCES principals(id), epoch INTEGER NOT NULL,
 key TEXT NOT NULL, arguments TEXT NOT NULL, response TEXT NOT NULL, PRIMARY KEY(project,principal,epoch,key)
);
CREATE TABLE questions (
 id TEXT PRIMARY KEY, project TEXT NOT NULL, task TEXT NOT NULL, sender TEXT NOT NULL REFERENCES principals(id),
 text TEXT NOT NULL, answer TEXT, answered_by TEXT REFERENCES principals(id), FOREIGN KEY(project,task) REFERENCES tasks(project,id)
);
CREATE TABLE messages (
 id TEXT PRIMARY KEY, project TEXT NOT NULL REFERENCES projects(id), sender TEXT NOT NULL REFERENCES principals(id),
 recipient TEXT NOT NULL REFERENCES principals(id), text TEXT NOT NULL, acknowledged INTEGER NOT NULL DEFAULT 0 CHECK(acknowledged IN (0,1))
);
CREATE TABLE decisions (
 id TEXT PRIMARY KEY, project TEXT NOT NULL REFERENCES projects(id), author TEXT NOT NULL REFERENCES principals(id), text TEXT NOT NULL
);
-- Reserved durable contracts for later tickets: exact identity, separate evidence and acceptance.
CREATE TABLE candidates (
 id TEXT PRIMARY KEY, attempt TEXT NOT NULL REFERENCES attempts(id), commit_id TEXT NOT NULL,
 tree_id TEXT NOT NULL, base TEXT NOT NULL, report_digest TEXT NOT NULL
);
CREATE TABLE verification_reports (
 id TEXT PRIMARY KEY, candidate TEXT NOT NULL REFERENCES candidates(id), suite TEXT NOT NULL,
 environment TEXT NOT NULL, artifact_digest TEXT NOT NULL, verdict TEXT NOT NULL, UNIQUE(id,candidate)
);
CREATE TABLE acceptance_decisions (
 id TEXT PRIMARY KEY, candidate TEXT NOT NULL REFERENCES candidates(id), verification TEXT NOT NULL,
 principal TEXT NOT NULL REFERENCES principals(id), decision TEXT NOT NULL,
 FOREIGN KEY(verification,candidate) REFERENCES verification_reports(id,candidate)
);
CREATE TABLE artifact_manifests (digest TEXT PRIMARY KEY, path TEXT NOT NULL, bytes INTEGER NOT NULL CHECK(bytes >= 0));
CREATE TRIGGER briefs_no_update BEFORE UPDATE ON briefs BEGIN SELECT RAISE(ABORT,'immutable brief'); END;
CREATE TRIGGER briefs_no_delete BEFORE DELETE ON briefs BEGIN SELECT RAISE(ABORT,'immutable brief'); END;
CREATE TRIGGER events_no_update BEFORE UPDATE ON events BEGIN SELECT RAISE(ABORT,'append-only event'); END;
CREATE TRIGGER events_no_delete BEFORE DELETE ON events BEGIN SELECT RAISE(ABORT,'append-only event'); END;
CREATE TRIGGER decisions_no_update BEFORE UPDATE ON decisions BEGIN SELECT RAISE(ABORT,'append-only decision'); END;
CREATE TRIGGER decisions_no_delete BEFORE DELETE ON decisions BEGIN SELECT RAISE(ABORT,'append-only decision'); END;
CREATE TRIGGER candidates_no_update BEFORE UPDATE ON candidates BEGIN SELECT RAISE(ABORT,'immutable candidate'); END;
CREATE TRIGGER candidates_no_delete BEFORE DELETE ON candidates BEGIN SELECT RAISE(ABORT,'immutable candidate'); END;
CREATE TRIGGER verification_no_update BEFORE UPDATE ON verification_reports BEGIN SELECT RAISE(ABORT,'immutable verification'); END;
CREATE TRIGGER verification_no_delete BEFORE DELETE ON verification_reports BEGIN SELECT RAISE(ABORT,'immutable verification'); END;
CREATE TRIGGER acceptance_no_update BEFORE UPDATE ON acceptance_decisions BEGIN SELECT RAISE(ABORT,'immutable acceptance'); END;
CREATE TRIGGER acceptance_no_delete BEFORE DELETE ON acceptance_decisions BEGIN SELECT RAISE(ABORT,'immutable acceptance'); END;
PRAGMA user_version = 1;
