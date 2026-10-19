CREATE TABLE runs (
    run_id TEXT PRIMARY KEY,
    graph_sha256 TEXT NOT NULL CHECK (
        length(graph_sha256) = 64
        AND graph_sha256 NOT GLOB '*[^0-9a-f]*'
    ),
    graph_json BLOB NOT NULL CHECK (length(graph_json) <= 1048576),
    provenance_json BLOB NOT NULL CHECK (length(provenance_json) <= 262144),
    max_tokens INTEGER NOT NULL CHECK (max_tokens > 0),
    max_storage_bytes INTEGER NOT NULL CHECK (max_storage_bytes > 0),
    max_wall_seconds INTEGER NOT NULL CHECK (max_wall_seconds > 0),
    state TEXT NOT NULL CHECK (state IN ('active', 'completed', 'failed', 'cancelled')),
    cancellation_requested INTEGER NOT NULL DEFAULT 0 CHECK (cancellation_requested IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

CREATE TABLE state_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

CREATE TABLE tasks (
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('analysis', 'reducer')),
    state TEXT NOT NULL CHECK (
        state IN (
            'pending',
            'ready',
            'running',
            'result_published',
            'completed',
            'failed',
            'cancelled'
        )
    ),
    accepted_attempt_id TEXT,
    max_transient_attempts INTEGER NOT NULL CHECK (max_transient_attempts BETWEEN 0 AND 3),
    task_json BLOB NOT NULL CHECK (length(task_json) <= 262144),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (run_id, task_id),
    FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE RESTRICT,
    FOREIGN KEY (accepted_attempt_id, run_id, task_id)
        REFERENCES attempts(attempt_id, run_id, task_id)
        DEFERRABLE INITIALLY DEFERRED
) STRICT;

CREATE TABLE task_dependencies (
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    depends_on_task_id TEXT NOT NULL,
    PRIMARY KEY (run_id, task_id, depends_on_task_id),
    CHECK (task_id <> depends_on_task_id),
    FOREIGN KEY (run_id, task_id)
        REFERENCES tasks(run_id, task_id) ON DELETE RESTRICT,
    FOREIGN KEY (run_id, depends_on_task_id)
        REFERENCES tasks(run_id, task_id) ON DELETE RESTRICT
) STRICT;

CREATE TABLE attempts (
    attempt_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    state TEXT NOT NULL CHECK (
        state IN (
            'prepared',
            'dispatching_thread',
            'thread_started',
            'dispatching_turn',
            'turn_started',
            'reconciling',
            'result_published',
            'succeeded',
            'failed',
            'indeterminate',
            'cancelled'
        )
    ),
    lease_owner TEXT,
    lease_expires_at INTEGER,
    last_lease_expires_at INTEGER,
    thread_id TEXT,
    latest_turn_id TEXT,
    latest_operation_marker TEXT,
    continuation_count INTEGER NOT NULL DEFAULT 0 CHECK (continuation_count >= 0),
    observed_tokens INTEGER NOT NULL DEFAULT 0 CHECK (observed_tokens >= 0),
    result_sha256 TEXT CHECK (
        result_sha256 IS NULL
        OR (
            length(result_sha256) = 64
            AND result_sha256 NOT GLOB '*[^0-9a-f]*'
        )
    ),
    result_status TEXT CHECK (
        result_status IS NULL
        OR result_status IN ('success', 'partial', 'failed', 'cancelled')
    ),
    result_token_usage INTEGER CHECK (
        result_token_usage IS NULL OR result_token_usage >= 0
    ),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (run_id, task_id, ordinal),
    UNIQUE (attempt_id, run_id, task_id),
    CHECK (
        (lease_owner IS NULL AND lease_expires_at IS NULL)
        OR (lease_owner IS NOT NULL AND lease_expires_at IS NOT NULL)
    ),
    FOREIGN KEY (run_id, task_id)
        REFERENCES tasks(run_id, task_id) ON DELETE RESTRICT
) STRICT;

CREATE TABLE operations (
    operation_id TEXT PRIMARY KEY,
    attempt_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (
        kind IN (
            'start_thread',
            'start_turn',
            'continue_turn',
            'interrupt_turn',
            'publish_result',
            'evaluate'
        )
    ),
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    state TEXT NOT NULL CHECK (
        state IN ('prepared', 'dispatching', 'completed', 'failed', 'indeterminate', 'cancelled')
    ),
    external_id TEXT,
    target_external_id TEXT,
    operation_marker TEXT,
    consumed_at INTEGER,
    transient INTEGER CHECK (transient IS NULL OR transient IN (0, 1)),
    failure_class TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (attempt_id, kind, ordinal),
    FOREIGN KEY (attempt_id) REFERENCES attempts(attempt_id) ON DELETE RESTRICT
) STRICT;

CREATE TABLE budget_reservations (
    attempt_id TEXT PRIMARY KEY,
    reserved_tokens INTEGER NOT NULL CHECK (reserved_tokens >= 0),
    reserved_storage_bytes INTEGER NOT NULL CHECK (reserved_storage_bytes >= 0),
    reserved_wall_seconds INTEGER NOT NULL CHECK (reserved_wall_seconds >= 0),
    observed_tokens INTEGER NOT NULL DEFAULT 0 CHECK (observed_tokens >= 0),
    observed_storage_bytes INTEGER NOT NULL DEFAULT 0 CHECK (observed_storage_bytes >= 0),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (attempt_id) REFERENCES attempts(attempt_id) ON DELETE RESTRICT
) STRICT;

CREATE TABLE artifacts (
    sha256 TEXT PRIMARY KEY CHECK (
        length(sha256) = 64
        AND sha256 NOT GLOB '*[^0-9a-f]*'
    ),
    media_type TEXT NOT NULL CHECK (length(media_type) BETWEEN 1 AND 256),
    size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
    metadata_json BLOB NOT NULL CHECK (length(metadata_json) <= 262144),
    created_at INTEGER NOT NULL
) STRICT;

CREATE TABLE events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT,
    task_id TEXT,
    attempt_id TEXT,
    event_type TEXT NOT NULL CHECK (length(event_type) BETWEEN 1 AND 128),
    payload_json BLOB NOT NULL CHECK (length(payload_json) <= 262144),
    timestamp INTEGER NOT NULL,
    CHECK (attempt_id IS NULL OR task_id IS NOT NULL),
    FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE RESTRICT,
    FOREIGN KEY (run_id, task_id)
        REFERENCES tasks(run_id, task_id) ON DELETE RESTRICT,
    FOREIGN KEY (attempt_id, run_id, task_id)
        REFERENCES attempts(attempt_id, run_id, task_id) ON DELETE RESTRICT
) STRICT;

CREATE INDEX tasks_ready_idx ON tasks(run_id, state, task_id);
CREATE INDEX dependencies_target_idx
    ON task_dependencies(run_id, depends_on_task_id, task_id);
CREATE INDEX attempts_task_idx ON attempts(run_id, task_id, ordinal);
CREATE INDEX attempts_lease_idx ON attempts(run_id, lease_expires_at)
    WHERE lease_expires_at IS NOT NULL;
CREATE INDEX operations_attempt_idx ON operations(attempt_id, kind, ordinal);
CREATE UNIQUE INDEX operations_marker_idx ON operations(operation_marker)
    WHERE operation_marker IS NOT NULL;
CREATE INDEX events_run_sequence_idx ON events(run_id, sequence);
