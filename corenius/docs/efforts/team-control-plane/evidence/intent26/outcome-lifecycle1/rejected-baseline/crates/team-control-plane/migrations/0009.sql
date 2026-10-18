-- Explicit later Intent work retains every historical Attempt and reservation.
-- One immutable Attempt is permitted per task generation, never ID recycling.
ALTER TABLE attempts DROP CONSTRAINT attempts_task_id_key;
ALTER TABLE attempts ADD CONSTRAINT attempts_task_generation_key UNIQUE(task_id,generation);
CREATE TABLE contract_attempt_schema_version(version integer PRIMARY KEY CHECK(version=1));
INSERT INTO contract_attempt_schema_version VALUES(1);
