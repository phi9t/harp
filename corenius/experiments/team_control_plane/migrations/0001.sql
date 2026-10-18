CREATE TABLE schema_version(
  version integer PRIMARY KEY CHECK(version=1)
);

INSERT INTO schema_version VALUES(1);
CREATE TABLE missions(


 id uuid PRIMARY KEY, objective text NOT NULL CHECK(length(objective)>0), git_repository text NOT NULL, git_commit text NOT NULL CHECK(git_commit ~ '^[0-9a-f]{40}([0-9a-f]{24})?$'), git_path text NOT NULL, input_digest text NOT NULL CHECK(input_digest ~ '^[0-9a-f]{64}$'),
 revision bigint NOT NULL CHECK(revision>0), capacity bigint NOT NULL CHECK(capacity>0), max_attempts bigint NOT NULL CHECK(max_attempts>0), max_episodes bigint NOT NULL CHECK(max_episodes>0), max_repairs bigint NOT NULL CHECK(max_repairs>=0), deadline_seconds bigint NOT NULL CHECK(deadline_seconds>0), used_capacity bigint NOT NULL DEFAULT 0 CHECK(used_capacity>=0), admitted_attempts bigint NOT NULL DEFAULT 0 CHECK(admitted_attempts>=0), admitted_episodes bigint NOT NULL DEFAULT 0 CHECK(admitted_episodes>=0),
 CHECK(used_capacity<=capacity AND admitted_attempts<=max_attempts AND admitted_episodes<=max_episodes)
);
CREATE TABLE tasks(
  id uuid PRIMARY KEY,
   mission_id uuid NOT NULL REFERENCES missions,
   revision bigint NOT NULL CHECK(revision>0),
   generation bigint NOT NULL DEFAULT 0 CHECK(generation>=0),
   status text NOT NULL CHECK(status IN ('proposed','ready','running')),
   spec_digest text NOT NULL CHECK(spec_digest ~ '^[0-9a-f]{64}$'),
   approved_digest text,
   UNIQUE(id,mission_id)
);

CREATE TABLE task_specs(
  task_id uuid NOT NULL REFERENCES tasks,
   digest text NOT NULL CHECK(digest ~ '^[0-9a-f]{64}$'),
   objective text NOT NULL,
   git_repository text NOT NULL,
   git_commit text NOT NULL,
   git_path text NOT NULL,
   input_digest text NOT NULL,
   acceptance jsonb NOT NULL CHECK(jsonb_typeof(acceptance)='array'),
   deliverables jsonb NOT NULL CHECK(jsonb_typeof(deliverables)='array'),
   write_scope jsonb NOT NULL CHECK(jsonb_typeof(write_scope)='array'),
   dependencies jsonb NOT NULL CHECK(jsonb_typeof(dependencies)='array'),
   created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
   PRIMARY KEY(task_id,digest)
);

CREATE TABLE spec_dependencies(
  task_id uuid NOT NULL,
  spec_digest text NOT NULL,
  dependency_id uuid NOT NULL,
  mission_id uuid NOT NULL,
  PRIMARY KEY(task_id,spec_digest,dependency_id),
  FOREIGN KEY(task_id,spec_digest) REFERENCES task_specs(task_id,digest),
  FOREIGN KEY(task_id,mission_id) REFERENCES tasks(id,mission_id),
  FOREIGN KEY(dependency_id,mission_id) REFERENCES tasks(id,mission_id),
  CHECK(task_id<>dependency_id)
);

CREATE TABLE dependencies(
  task_id uuid NOT NULL,
   dependency_id uuid NOT NULL,
   mission_id uuid NOT NULL,
   PRIMARY KEY(task_id,dependency_id),
   FOREIGN KEY(task_id,mission_id) REFERENCES tasks(id,mission_id),
   FOREIGN KEY(dependency_id,mission_id) REFERENCES tasks(id,mission_id),
   CHECK(task_id<>dependency_id)
);

CREATE TABLE workers(
  id uuid PRIMARY KEY
);

CREATE TABLE credentials(
  id uuid PRIMARY KEY,
   principal_id uuid NOT NULL,
   role text NOT NULL CHECK(role IN ('owner','lead','worker')),
   mission_id uuid REFERENCES missions,
   task_id uuid REFERENCES tasks,
   generation bigint,
   recovery_epoch uuid NOT NULL,
   token_hash text NOT NULL UNIQUE CHECK(token_hash ~ '^[0-9a-f]{64}$'),
   expires_at timestamptz NOT NULL,
   FOREIGN KEY(task_id,mission_id) REFERENCES tasks(id,mission_id),
   CHECK((role='owner' AND mission_id IS NULL AND task_id IS NULL AND generation IS NULL) OR (role='lead' AND mission_id IS NOT NULL AND generation IS NULL) OR (role='worker' AND mission_id IS NOT NULL AND task_id IS NOT NULL AND generation>0))
);

CREATE TABLE attempts(
  id uuid PRIMARY KEY,
   task_id uuid NOT NULL UNIQUE REFERENCES tasks,
   worker_id uuid NOT NULL REFERENCES workers,
   spec_digest text NOT NULL,
   generation bigint NOT NULL CHECK(generation>0),
   recovery_epoch uuid NOT NULL,
   status text NOT NULL CHECK(status='reserved'),
   FOREIGN KEY(task_id,spec_digest) REFERENCES task_specs(task_id,digest)
);

CREATE TABLE reservations(
  id uuid PRIMARY KEY,
   attempt_id uuid NOT NULL UNIQUE REFERENCES attempts,
   mission_id uuid NOT NULL REFERENCES missions,
   attempts bigint NOT NULL CHECK(attempts=1),
   episodes bigint NOT NULL CHECK(episodes=1),
   state text NOT NULL CHECK(state='reserved')
);

CREATE TABLE operations(
  id uuid PRIMARY KEY,
   attempt_id uuid NOT NULL UNIQUE REFERENCES attempts,
   kind text NOT NULL CHECK(kind='prepare_attempt'),
   state text NOT NULL CHECK(state='prepared'),
   argument_digest text NOT NULL CHECK(argument_digest ~ '^[0-9a-f]{64}$'),
   generation bigint NOT NULL,
   recovery_epoch uuid NOT NULL
);

CREATE TABLE events(
  id uuid PRIMARY KEY,
   aggregate_kind text NOT NULL CHECK(aggregate_kind IN ('mission','task','credential')),
   aggregate_id uuid NOT NULL,
   sequence bigint NOT NULL CHECK(sequence>0),
   version integer NOT NULL CHECK(version=1),
   principal_id uuid NOT NULL,
   transaction_id bigint NOT NULL,
   kind text NOT NULL,
   payload jsonb NOT NULL,
   UNIQUE(aggregate_kind,aggregate_id,sequence)
);

CREATE TABLE outbox(
  id uuid PRIMARY KEY,
   event_id uuid NOT NULL UNIQUE REFERENCES events,
   operation_id uuid NOT NULL UNIQUE REFERENCES operations,
   version integer NOT NULL CHECK(version=1),
   state text NOT NULL CHECK(state='pending')
);

CREATE TABLE commands(
  principal_id uuid NOT NULL,
   family text NOT NULL,
   key text NOT NULL,
   argument_digest text NOT NULL CHECK(argument_digest ~ '^[0-9a-f]{64}$'),
   aggregate_id uuid,
   scope_kind text NOT NULL CHECK(scope_kind IN ('owner','mission','task')),
   scope_mission uuid,
   scope_task uuid,
   CHECK((scope_kind='owner' AND scope_mission IS NULL AND scope_task IS NULL) OR (scope_kind='mission' AND scope_mission IS NOT NULL AND scope_task IS NULL) OR (scope_kind='task' AND scope_mission IS NOT NULL AND scope_task IS NOT NULL)),
   disposition text NOT NULL CHECK(disposition IN ('applied','rejected')),
   result jsonb NOT NULL,
   PRIMARY KEY(principal_id,family,key)
);

CREATE FUNCTION immutable_record() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'immutable record'; END $$;
CREATE TRIGGER immutable_specs BEFORE UPDATE OR DELETE ON task_specs FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_spec_dependencies BEFORE UPDATE OR DELETE ON spec_dependencies FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_events BEFORE UPDATE OR DELETE ON events FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_commands BEFORE UPDATE OR DELETE ON commands FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TABLE deployment_epochs(
  epoch uuid PRIMARY KEY
);

CREATE TABLE deployment(
  singleton boolean PRIMARY KEY DEFAULT true CHECK(singleton),
   active_epoch uuid NOT NULL
);
