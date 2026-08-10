ALTER TABLE attempts
ADD COLUMN semantic_failure_class TEXT CHECK (
    semantic_failure_class IS NULL
    OR length(semantic_failure_class) BETWEEN 1 AND 128
);

ALTER TABLE attempts
ADD COLUMN wall_last_observed_at INTEGER CHECK (
    wall_last_observed_at IS NULL OR wall_last_observed_at >= 0
);

UPDATE attempts
SET wall_last_observed_at = wall_started_at
WHERE wall_started_at IS NOT NULL
  AND wall_last_observed_at IS NULL;
