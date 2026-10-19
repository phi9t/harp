CREATE TABLE cli_attempts (
    attempt_id TEXT PRIMARY KEY CHECK (
        length(attempt_id) = 36
        AND substr(attempt_id, 9, 1) = '-'
        AND substr(attempt_id, 14, 1) = '-'
        AND substr(attempt_id, 15, 1) = '7'
        AND substr(attempt_id, 19, 1) = '-'
        AND substr(attempt_id, 20, 1) IN ('8', '9', 'a', 'b')
        AND substr(attempt_id, 24, 1) = '-'
        AND substr(attempt_id, 1, 8) NOT GLOB '*[^0-9a-f]*'
        AND substr(attempt_id, 10, 4) NOT GLOB '*[^0-9a-f]*'
        AND substr(attempt_id, 15, 4) NOT GLOB '*[^0-9a-f]*'
        AND substr(attempt_id, 20, 4) NOT GLOB '*[^0-9a-f]*'
        AND substr(attempt_id, 25, 12) NOT GLOB '*[^0-9a-f]*'
    ),
    state TEXT NOT NULL CHECK (
        state IN (
            'prepared',
            'running',
            'reconciling',
            'succeeded',
            'failed',
            'indeterminate',
            'cancelled'
        )
    ),
    logical_session_id TEXT NOT NULL CHECK (
        length(logical_session_id) BETWEEN 1 AND 256
        AND length(CAST(logical_session_id AS BLOB)) BETWEEN 1 AND 256
        AND instr(logical_session_id, char(0)) = 0
    ),
    external_session_id TEXT CHECK (
        external_session_id IS NULL
        OR (
            length(external_session_id) BETWEEN 1 AND 256
            AND length(CAST(external_session_id AS BLOB)) BETWEEN 1 AND 256
            AND instr(external_session_id, char(0)) = 0
        )
    ),
    continuation_count INTEGER NOT NULL DEFAULT 0 CHECK (
        continuation_count BETWEEN 0 AND 1
    ),
    terminal_failure_class TEXT CHECK (
        terminal_failure_class IS NULL
        OR terminal_failure_class IN (
            'resumable_interrupted',
            'resumable_cli_failure',
            'non_resumable_protocol_failure',
            'continuation_exhausted',
            'cancelled',
            'indeterminate'
        )
    ),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    CHECK (
        (
            state IN ('prepared', 'running', 'reconciling', 'succeeded')
            AND terminal_failure_class IS NULL
        )
        OR (
            state = 'failed'
            AND terminal_failure_class IN (
                'non_resumable_protocol_failure',
                'continuation_exhausted'
            )
        )
        OR (
            state = 'indeterminate'
            AND terminal_failure_class = 'indeterminate'
        )
        OR (
            state = 'cancelled'
            AND terminal_failure_class = 'cancelled'
        )
    ),
    FOREIGN KEY (attempt_id) REFERENCES attempts(attempt_id) ON DELETE RESTRICT
) STRICT;

CREATE TABLE cli_activities (
    activity_id TEXT PRIMARY KEY CHECK (
        length(activity_id) = 36
        AND substr(activity_id, 9, 1) = '-'
        AND substr(activity_id, 14, 1) = '-'
        AND substr(activity_id, 15, 1) = '7'
        AND substr(activity_id, 19, 1) = '-'
        AND substr(activity_id, 20, 1) IN ('8', '9', 'a', 'b')
        AND substr(activity_id, 24, 1) = '-'
        AND substr(activity_id, 1, 8) NOT GLOB '*[^0-9a-f]*'
        AND substr(activity_id, 10, 4) NOT GLOB '*[^0-9a-f]*'
        AND substr(activity_id, 15, 4) NOT GLOB '*[^0-9a-f]*'
        AND substr(activity_id, 20, 4) NOT GLOB '*[^0-9a-f]*'
        AND substr(activity_id, 25, 12) NOT GLOB '*[^0-9a-f]*'
    ),
    attempt_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (
        kind IN ('start_activity', 'continue_activity', 'interrupt_activity')
    ),
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    state TEXT NOT NULL CHECK (
        state IN (
            'prepared',
            'dispatching',
            'running',
            'reconciling',
            'completed',
            'failed',
            'indeterminate',
            'cancelled'
        )
    ),
    logical_turn_id TEXT NOT NULL CHECK (
        length(logical_turn_id) = 36
        AND logical_turn_id = activity_id
    ),
    activity_dir TEXT NOT NULL CHECK (
        length(activity_dir) BETWEEN 1 AND 4096
        AND length(CAST(activity_dir AS BLOB)) BETWEEN 1 AND 4096
        AND substr(activity_dir, 1, 1) = '/'
        AND instr(activity_dir, char(0)) = 0
    ),
    invocation_sha256 TEXT NOT NULL CHECK (
        length(invocation_sha256) = 64
        AND invocation_sha256 NOT GLOB '*[^0-9a-f]*'
    ),
    process_record_sha256 TEXT CHECK (
        process_record_sha256 IS NULL
        OR (
            length(process_record_sha256) = 64
            AND process_record_sha256 NOT GLOB '*[^0-9a-f]*'
        )
    ),
    terminal_failure_class TEXT CHECK (
        terminal_failure_class IS NULL
        OR terminal_failure_class IN (
            'resumable_interrupted',
            'resumable_cli_failure',
            'non_resumable_protocol_failure',
            'continuation_exhausted',
            'cancelled',
            'indeterminate'
        )
    ),
    interrupt_purpose TEXT CHECK (
        interrupt_purpose IS NULL
        OR interrupt_purpose IN ('budget', 'scanner_integrity', 'cancellation')
    ),
    target_process_record_sha256 TEXT CHECK (
        target_process_record_sha256 IS NULL
        OR (
            length(target_process_record_sha256) = 64
            AND target_process_record_sha256 NOT GLOB '*[^0-9a-f]*'
        )
    ),
    signal_stage TEXT CHECK (
        signal_stage IS NULL
        OR signal_stage IN (
            'prepared',
            'sigint_prepared',
            'sigint_sent',
            'sigterm_prepared',
            'sigterm_sent',
            'sigkill_prepared',
            'sigkill_sent',
            'quiescent',
            'indeterminate'
        )
    ),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (attempt_id, kind, ordinal),
    CHECK (
        (
            state IN (
                'prepared',
                'dispatching',
                'running',
                'reconciling',
                'completed'
            )
            AND terminal_failure_class IS NULL
        )
        OR (
            state = 'failed'
            AND terminal_failure_class IN (
                'resumable_interrupted',
                'resumable_cli_failure',
                'non_resumable_protocol_failure',
                'continuation_exhausted'
            )
        )
        OR (
            state = 'indeterminate'
            AND terminal_failure_class = 'indeterminate'
        )
        OR (
            state = 'cancelled'
            AND terminal_failure_class = 'cancelled'
        )
    ),
    CHECK (
        kind = 'interrupt_activity'
        OR (
            interrupt_purpose IS NULL
            AND target_process_record_sha256 IS NULL
            AND signal_stage IS NULL
        )
    ),
    CHECK (
        kind <> 'interrupt_activity'
        OR (
            process_record_sha256 IS NULL
            AND interrupt_purpose IS NOT NULL
            AND target_process_record_sha256 IS NOT NULL
            AND signal_stage IS NOT NULL
        )
    ),
    CHECK (
        kind = 'interrupt_activity'
        OR state NOT IN ('running', 'reconciling', 'completed', 'failed')
        OR process_record_sha256 IS NOT NULL
    ),
    CHECK (
        kind <> 'interrupt_activity'
        OR (
            state = 'prepared'
            AND signal_stage IN (
                'prepared',
                'sigint_prepared',
                'sigint_sent',
                'sigterm_prepared',
                'sigterm_sent',
                'sigkill_prepared',
                'sigkill_sent'
            )
        )
        OR (
            state = 'completed'
            AND signal_stage = 'quiescent'
        )
        OR (
            state = 'indeterminate'
            AND signal_stage = 'indeterminate'
        )
        OR (
            state = 'cancelled'
            AND signal_stage = 'indeterminate'
        )
    ),
    FOREIGN KEY (attempt_id) REFERENCES cli_attempts(attempt_id) ON DELETE RESTRICT
) STRICT;

CREATE UNIQUE INDEX cli_attempts_external_session_idx
    ON cli_attempts(external_session_id)
    WHERE external_session_id IS NOT NULL;
CREATE INDEX cli_attempts_nonterminal_idx
    ON cli_attempts(state, attempt_id)
    WHERE state IN ('prepared', 'running', 'reconciling');
CREATE INDEX cli_activities_attempt_idx
    ON cli_activities(attempt_id, kind, ordinal);
CREATE INDEX cli_activities_recovery_idx
    ON cli_activities(state, attempt_id, kind, ordinal)
    WHERE state IN ('prepared', 'dispatching', 'running', 'reconciling');
