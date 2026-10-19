CREATE TABLE supervisor_schema_version(version integer PRIMARY KEY CHECK(version=1));
INSERT INTO supervisor_schema_version VALUES(1);
CREATE TABLE supervised_workspaces(
 id uuid PRIMARY KEY,
 attempt_id uuid NOT NULL UNIQUE REFERENCES attempts,
 manifest jsonb NOT NULL,
 state text NOT NULL CHECK(state IN ('registered','preparing','ready','starting','running','cancelling','stopped','quarantined','deleting','released')),
 quarantine_reason text,
 desired_cancel boolean NOT NULL DEFAULT false,
 deadline timestamptz,
 argv jsonb,
 controller jsonb,
 launcher jsonb,
 init_process jsonb,
 members jsonb NOT NULL DEFAULT '[]',
 created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE FUNCTION immutable_workspace_manifest() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
 IF NEW.id<>OLD.id OR NEW.attempt_id<>OLD.attempt_id OR NEW.manifest<>OLD.manifest OR (OLD.init_process IS NOT NULL AND NEW.init_process IS DISTINCT FROM OLD.init_process) THEN RAISE EXCEPTION 'immutable workspace manifest'; END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER immutable_workspace_manifest BEFORE UPDATE ON supervised_workspaces FOR EACH ROW EXECUTE FUNCTION immutable_workspace_manifest();
CREATE TABLE supervised_observations(
 id uuid PRIMARY KEY,
 execution_id uuid NOT NULL REFERENCES supervised_workspaces,
 observed_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 state text NOT NULL,
 members jsonb NOT NULL
);
CREATE TRIGGER immutable_supervised_observations BEFORE UPDATE OR DELETE ON supervised_observations FOR EACH ROW EXECUTE FUNCTION immutable_record();
ALTER TABLE reservations DROP CONSTRAINT reservations_state_check;
ALTER TABLE reservations ADD CONSTRAINT reservations_state_check CHECK(state IN ('reserved','released'));
