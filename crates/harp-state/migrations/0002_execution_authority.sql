ALTER TABLE attempts
ADD COLUMN latest_turn_observed_tokens INTEGER NOT NULL DEFAULT 0
CHECK (latest_turn_observed_tokens >= 0);

ALTER TABLE operations
ADD COLUMN intent_json BLOB CHECK (
    intent_json IS NULL OR length(intent_json) <= 262144
);

ALTER TABLE operations
ADD COLUMN intent_sha256 TEXT CHECK (
    intent_sha256 IS NULL
    OR (
        length(intent_sha256) = 64
        AND intent_sha256 NOT GLOB '*[^0-9a-f]*'
    )
);

ALTER TABLE operations
ADD COLUMN checkpoint_sha256 TEXT CHECK (
    checkpoint_sha256 IS NULL
    OR (
        length(checkpoint_sha256) = 64
        AND checkpoint_sha256 NOT GLOB '*[^0-9a-f]*'
    )
);

ALTER TABLE budget_reservations
ADD COLUMN observed_wall_seconds INTEGER NOT NULL DEFAULT 0
CHECK (observed_wall_seconds >= 0);
