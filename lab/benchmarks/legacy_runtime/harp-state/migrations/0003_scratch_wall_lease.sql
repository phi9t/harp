ALTER TABLE attempts
ADD COLUMN lease_capability_sha256 TEXT CHECK (
    lease_capability_sha256 IS NULL
    OR (
        length(lease_capability_sha256) = 64
        AND lease_capability_sha256 NOT GLOB '*[^0-9a-f]*'
    )
);

ALTER TABLE attempts
ADD COLUMN scratch_path TEXT;

ALTER TABLE attempts
ADD COLUMN scratch_device INTEGER CHECK (
    scratch_device IS NULL OR scratch_device >= 0
);

ALTER TABLE attempts
ADD COLUMN scratch_inode INTEGER CHECK (
    scratch_inode IS NULL OR scratch_inode >= 0
);

ALTER TABLE attempts
ADD COLUMN wall_started_at INTEGER CHECK (
    wall_started_at IS NULL OR wall_started_at >= 0
);

ALTER TABLE attempts
ADD COLUMN observed_wall_seconds INTEGER NOT NULL DEFAULT 0
CHECK (observed_wall_seconds >= 0);
