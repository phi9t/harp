CREATE TABLE temporal_schema_version(version integer NOT NULL CHECK(version=1));
INSERT INTO temporal_schema_version VALUES(1);
CREATE TABLE workflow_registry(
 aggregate_kind text NOT NULL CHECK(aggregate_kind IN ('mission','task')),
 aggregate_id uuid NOT NULL,
 generation bigint NOT NULL CHECK(generation>0),
 recovery_epoch uuid NOT NULL REFERENCES deployment_epochs(epoch),
 workflow_id text NOT NULL UNIQUE,
 cursor bigint NOT NULL DEFAULT 0 CHECK(cursor>=0),
 next_sequence bigint NOT NULL DEFAULT 1 CHECK(next_sequence>0),
 closed boolean NOT NULL DEFAULT false,
 observed_run_id text,
 incarnation bigint NOT NULL DEFAULT 0 CHECK(incarnation>=0),
 lifecycle text NOT NULL DEFAULT 'registered' CHECK(lifecycle IN ('registered','running','completed','failed','canceled','terminated','continued_as_new','timed_out','paused')),
 CHECK(cursor<next_sequence),
 PRIMARY KEY(aggregate_kind,aggregate_id,generation)
);
CREATE TABLE workflow_commands(
 workflow_id text NOT NULL REFERENCES workflow_registry(workflow_id),
 sequence bigint NOT NULL CHECK(sequence>0),
 principal_id uuid NOT NULL,
 family text NOT NULL,
 command_key text NOT NULL,
 digest text NOT NULL,
 envelope jsonb NOT NULL,
 disposition text CHECK(disposition IN ('applied','rejected')),
 result jsonb,
 PRIMARY KEY(workflow_id,sequence),
 UNIQUE(workflow_id,principal_id,family,command_key),
 CHECK((disposition IS NULL)=(result IS NULL))
);
CREATE TABLE workflow_wakeups(
 id uuid PRIMARY KEY,
 workflow_id text NOT NULL REFERENCES workflow_registry(workflow_id),
 sequence bigint NOT NULL CHECK(sequence>=0),
 delivered boolean NOT NULL DEFAULT false,
 UNIQUE(workflow_id,sequence)
);

CREATE FUNCTION immutable_workflow_command() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF OLD.disposition IS NOT NULL OR ROW(OLD.workflow_id,OLD.sequence,OLD.principal_id,OLD.family,OLD.command_key,OLD.digest,OLD.envelope)
  IS DISTINCT FROM ROW(NEW.workflow_id,NEW.sequence,NEW.principal_id,NEW.family,NEW.command_key,NEW.digest,NEW.envelope) THEN
  RAISE EXCEPTION 'workflow intent and recorded disposition are immutable';
 END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER immutable_workflow_intent BEFORE UPDATE ON workflow_commands FOR EACH ROW EXECUTE FUNCTION immutable_workflow_command();
CREATE TRIGGER no_delete_workflow_command BEFORE DELETE ON workflow_commands FOR EACH ROW EXECUTE FUNCTION immutable_record();

CREATE TABLE workflow_incarnations(
 workflow_id text NOT NULL REFERENCES workflow_registry(workflow_id),
 run_id text NOT NULL,
 previous_run_id text,
 incarnation bigint NOT NULL CHECK(incarnation>0),
 PRIMARY KEY(workflow_id,run_id),
 UNIQUE(workflow_id,incarnation)
);
CREATE TRIGGER immutable_workflow_incarnation BEFORE UPDATE OR DELETE ON workflow_incarnations FOR EACH ROW EXECUTE FUNCTION immutable_record();
ALTER TABLE workflow_registry ADD FOREIGN KEY(workflow_id,observed_run_id) REFERENCES workflow_incarnations(workflow_id,run_id);
