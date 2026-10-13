-- Explicit opt-in kata-v1 extension. No import or historical-state migration.
CREATE TABLE kata_configuration (singleton INTEGER PRIMARY KEY CHECK(singleton=1), body TEXT NOT NULL);
CREATE TABLE kata_versions (
 id TEXT PRIMARY KEY, project TEXT NOT NULL REFERENCES projects(id), task TEXT NOT NULL,
 revision INTEGER NOT NULL, digest TEXT NOT NULL, body TEXT NOT NULL
);
CREATE TABLE kata_sources (
 project TEXT NOT NULL, task TEXT NOT NULL, item TEXT NOT NULL UNIQUE, source_files TEXT NOT NULL,
 admitted TEXT NOT NULL REFERENCES kata_versions(id), current TEXT NOT NULL REFERENCES kata_versions(id),
 acknowledged_revision INTEGER, acknowledged_summary TEXT, drift TEXT,
 PRIMARY KEY(project,task), FOREIGN KEY(project,task) REFERENCES tasks(project,id)
);
CREATE TABLE kata_proposals (
 id TEXT PRIMARY KEY, project TEXT NOT NULL, task TEXT NOT NULL, version TEXT NOT NULL REFERENCES kata_versions(id),
 UNIQUE(project,task,version), FOREIGN KEY(project,task) REFERENCES tasks(project,id)
);
CREATE TABLE followup_proposals (
 id TEXT PRIMARY KEY, project TEXT NOT NULL, task TEXT NOT NULL, author TEXT NOT NULL REFERENCES principals(id), text TEXT NOT NULL,
 FOREIGN KEY(project,task) REFERENCES tasks(project,id)
);
CREATE TABLE kata_effects (
 id TEXT PRIMARY KEY, project TEXT NOT NULL, task TEXT NOT NULL, event_seq INTEGER NOT NULL REFERENCES events(seq),
 controller_revision INTEGER NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('metadata','comment')),
 state TEXT NOT NULL CHECK(state IN ('prepared','applying','confirmed','unknown')),
 payload TEXT NOT NULL, prepared_at INTEGER NOT NULL, observation TEXT, acknowledged_revision INTEGER,
 FOREIGN KEY(project,task) REFERENCES tasks(project,id), UNIQUE(project,task,event_seq,kind)
);
CREATE TABLE kata_effect_history (
 seq INTEGER PRIMARY KEY AUTOINCREMENT, effect TEXT NOT NULL REFERENCES kata_effects(id), state TEXT NOT NULL, observation TEXT NOT NULL
);
CREATE TRIGGER kata_versions_no_update BEFORE UPDATE ON kata_versions BEGIN SELECT RAISE(ABORT,'immutable Kata/source version'); END;
CREATE TRIGGER kata_versions_no_delete BEFORE DELETE ON kata_versions BEGIN SELECT RAISE(ABORT,'immutable Kata/source version'); END;
CREATE TRIGGER kata_proposals_no_update BEFORE UPDATE ON kata_proposals BEGIN SELECT RAISE(ABORT,'immutable source proposal'); END;
CREATE TRIGGER kata_proposals_no_delete BEFORE DELETE ON kata_proposals BEGIN SELECT RAISE(ABORT,'immutable source proposal'); END;
CREATE TRIGGER followups_no_update BEFORE UPDATE ON followup_proposals BEGIN SELECT RAISE(ABORT,'immutable followup proposal'); END;
CREATE TRIGGER followups_no_delete BEFORE DELETE ON followup_proposals BEGIN SELECT RAISE(ABORT,'immutable followup proposal'); END;
CREATE TRIGGER kata_history_no_update BEFORE UPDATE ON kata_effect_history BEGIN SELECT RAISE(ABORT,'append-only effect history'); END;
CREATE TRIGGER kata_history_no_delete BEFORE DELETE ON kata_effect_history BEGIN SELECT RAISE(ABORT,'append-only effect history'); END;
