CREATE TABLE workflow_admissions (
    run_id TEXT PRIMARY KEY,
    namespace TEXT NOT NULL,
    submission_key TEXT NOT NULL,
    request_sha256 TEXT NOT NULL,
    request_json BLOB NOT NULL,
    revision INTEGER NOT NULL CHECK(revision >= 0),
    owner_epoch INTEGER NOT NULL CHECK(owner_epoch >= 0),
    cancel_requested INTEGER NOT NULL CHECK(cancel_requested IN (0,1)),
    created_at INTEGER NOT NULL,
    UNIQUE(namespace, submission_key)
);
CREATE TABLE workflow_jobs (
    job_id TEXT NOT NULL,
    run_id TEXT NOT NULL REFERENCES workflow_admissions(run_id),
    task_id TEXT NOT NULL,
    attempt INTEGER NOT NULL CHECK(attempt > 0),
    trial_id TEXT,
    record_json BLOB NOT NULL,
    PRIMARY KEY(run_id, job_id),
    UNIQUE(run_id, task_id, attempt)
);
CREATE TABLE workflow_environments (
    run_id TEXT NOT NULL REFERENCES workflow_admissions(run_id),
    environment_sha256 TEXT NOT NULL,
    record_json BLOB NOT NULL,
    PRIMARY KEY(run_id, environment_sha256)
);
CREATE TABLE workflow_decisions (
    run_id TEXT NOT NULL REFERENCES workflow_admissions(run_id),
    decision_id TEXT NOT NULL,
    request_sha256 TEXT NOT NULL,
    record_json BLOB NOT NULL,
    PRIMARY KEY(run_id, decision_id)
);
CREATE TABLE workflow_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL REFERENCES workflow_admissions(run_id),
    revision INTEGER NOT NULL CHECK(revision >= 0),
    event_type TEXT NOT NULL,
    payload_json BLOB NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX workflow_events_run_idx ON workflow_events(run_id, sequence);
