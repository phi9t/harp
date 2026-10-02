-- Additive assembly schema; accepted supervisor/mailbox/gateway version1 remains immutable.
CREATE TABLE kernel_schema_version(version integer PRIMARY KEY CHECK(version=1));
INSERT INTO kernel_schema_version VALUES(1);
CREATE TABLE kernel_mission_control(mission_id uuid PRIMARY KEY REFERENCES missions, paused boolean NOT NULL DEFAULT false, revision bigint NOT NULL DEFAULT 0);
CREATE TABLE kernel_control_events(id uuid PRIMARY KEY,mission_id uuid NOT NULL REFERENCES missions,principal_id uuid NOT NULL,recovery_epoch uuid NOT NULL,kind text NOT NULL,argument_digest text NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TRIGGER immutable_kernel_control_events BEFORE UPDATE OR DELETE ON kernel_control_events FOR EACH ROW EXECUTE FUNCTION immutable_record();
ALTER TABLE supervised_workspaces ALTER COLUMN attempt_id DROP NOT NULL;
CREATE TABLE episode_executions(
 execution_id uuid PRIMARY KEY REFERENCES supervised_workspaces,
 episode_id uuid NOT NULL UNIQUE REFERENCES mailbox_episodes,
 mission_id uuid NOT NULL REFERENCES missions,
 task_id uuid REFERENCES tasks,
 attempt_id uuid REFERENCES attempts,
 principal_id uuid NOT NULL,
 credential_id uuid NOT NULL REFERENCES credentials,
 role text NOT NULL CHECK(role IN ('lead','worker','verifier')),
 recovery_epoch uuid NOT NULL,
 input_digest text NOT NULL CHECK(length(input_digest)=64),
 owns_capacity boolean NOT NULL,
 capacity_released boolean NOT NULL DEFAULT false,
 CHECK((role='lead' AND task_id IS NULL AND attempt_id IS NULL) OR (role IN ('worker','verifier') AND task_id IS NOT NULL AND attempt_id IS NOT NULL))
);
CREATE FUNCTION immutable_episode_execution() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
 IF (to_jsonb(NEW)-'capacity_released') IS DISTINCT FROM (to_jsonb(OLD)-'capacity_released') OR (OLD.capacity_released AND NOT NEW.capacity_released) THEN RAISE EXCEPTION 'immutable episode execution binding'; END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER immutable_episode_execution BEFORE UPDATE ON episode_executions FOR EACH ROW EXECUTE FUNCTION immutable_episode_execution();
CREATE TRIGGER immutable_episode_execution_delete BEFORE DELETE ON episode_executions FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE OR REPLACE FUNCTION immutable_workspace_manifest() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
 IF NEW.id IS DISTINCT FROM OLD.id OR NEW.attempt_id IS DISTINCT FROM OLD.attempt_id OR NEW.manifest IS DISTINCT FROM OLD.manifest OR (OLD.init_process IS NOT NULL AND NEW.init_process IS DISTINCT FROM OLD.init_process) THEN RAISE EXCEPTION 'immutable workspace manifest'; END IF;
 RETURN NEW;
END $$;

CREATE TABLE kernel_commands(principal_id uuid NOT NULL,family text NOT NULL,key text NOT NULL,mission_id uuid NOT NULL REFERENCES missions,argument_digest text NOT NULL,result jsonb NOT NULL,recovery_epoch uuid NOT NULL,PRIMARY KEY(principal_id,family,key));
CREATE TRIGGER immutable_kernel_commands BEFORE UPDATE OR DELETE ON kernel_commands FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TABLE kernel_execution_results(execution_id uuid PRIMARY KEY REFERENCES supervised_workspaces,result jsonb NOT NULL);
CREATE TRIGGER immutable_kernel_execution_results BEFORE UPDATE OR DELETE ON kernel_execution_results FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TABLE kernel_verification_requirements(task_id uuid PRIMARY KEY REFERENCES tasks,recovery_epoch uuid NOT NULL,by_principal uuid NOT NULL);
CREATE TRIGGER immutable_kernel_verification_requirements BEFORE UPDATE OR DELETE ON kernel_verification_requirements FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TABLE episode_runtime_operations(
 id uuid PRIMARY KEY,episode_id uuid NOT NULL UNIQUE REFERENCES mailbox_episodes,
 execution_id uuid NOT NULL UNIQUE REFERENCES episode_executions(execution_id),
 principal_id uuid NOT NULL,credential_id uuid NOT NULL REFERENCES credentials,
 recovery_epoch uuid NOT NULL,digest text NOT NULL,request jsonb NOT NULL,
 state text NOT NULL CHECK(state IN ('prepared','dispatching','confirmed','failed_known','outcome_unknown','quarantined')),
 controller uuid,store text REFERENCES gateway_stores,thread_id text,turn_id text,effect_id text
);
CREATE FUNCTION immutable_episode_runtime_intent() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
 IF NEW.id IS DISTINCT FROM OLD.id OR NEW.episode_id IS DISTINCT FROM OLD.episode_id OR NEW.execution_id IS DISTINCT FROM OLD.execution_id OR NEW.principal_id IS DISTINCT FROM OLD.principal_id OR NEW.credential_id IS DISTINCT FROM OLD.credential_id OR NEW.recovery_epoch IS DISTINCT FROM OLD.recovery_epoch OR NEW.digest IS DISTINCT FROM OLD.digest OR NEW.request IS DISTINCT FROM OLD.request THEN RAISE EXCEPTION 'immutable episode runtime intent'; END IF; RETURN NEW;
END $$;
CREATE TRIGGER immutable_episode_runtime_intent BEFORE UPDATE ON episode_runtime_operations FOR EACH ROW EXECUTE FUNCTION immutable_episode_runtime_intent();
CREATE TRIGGER immutable_episode_runtime_intent_delete BEFORE DELETE ON episode_runtime_operations FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TABLE episode_runtime_evidence(id uuid PRIMARY KEY,operation_id uuid NOT NULL REFERENCES episode_runtime_operations,evidence jsonb NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TRIGGER immutable_episode_runtime_evidence BEFORE UPDATE OR DELETE ON episode_runtime_evidence FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TABLE episode_native_bindings(episode_id uuid PRIMARY KEY REFERENCES mailbox_episodes,operation_id uuid NOT NULL UNIQUE REFERENCES episode_runtime_operations,execution_id uuid NOT NULL UNIQUE REFERENCES episode_executions(execution_id),store text NOT NULL REFERENCES gateway_stores,thread_id text NOT NULL,turn_id text NOT NULL,input_packet uuid NOT NULL REFERENCES mailbox_inputs,input_digest text NOT NULL,recovery_epoch uuid NOT NULL,UNIQUE(store,thread_id));
CREATE TRIGGER immutable_episode_native_bindings BEFORE UPDATE OR DELETE ON episode_native_bindings FOR EACH ROW EXECUTE FUNCTION immutable_record();

-- Trusted local transport registration is captured by the service process,
-- never by agent-supplied PID or frame assertions.
CREATE TABLE kernel_transports(
 controller uuid PRIMARY KEY,
 store text NOT NULL REFERENCES gateway_stores(identity),
 runtime_identity jsonb NOT NULL,
 process_identity jsonb NOT NULL,
 group_id integer NOT NULL CHECK(group_id>0),
 workspace_root text NOT NULL,
 native_root text NOT NULL,
 native_directory jsonb NOT NULL,
 workspace_directory jsonb NOT NULL,
 contract text NOT NULL CHECK(contract IN ('inprocess-supervisor-os-v1','legacy-supervisor-os-v1')),
 recovery_epoch uuid NOT NULL,
 registered_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TRIGGER immutable_kernel_transports BEFORE UPDATE OR DELETE ON kernel_transports FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TABLE kernel_transport_observations(
 controller uuid NOT NULL REFERENCES kernel_transports,
 operation_id uuid NOT NULL REFERENCES episode_runtime_operations,
 evidence jsonb NOT NULL,
 PRIMARY KEY(controller,operation_id)
);
CREATE TRIGGER immutable_kernel_transport_observations BEFORE UPDATE OR DELETE ON kernel_transport_observations FOR EACH ROW EXECUTE FUNCTION immutable_record();

ALTER TABLE tasks DROP CONSTRAINT tasks_status_check;
ALTER TABLE tasks ADD CONSTRAINT tasks_status_check CHECK(status IN ('proposed','ready','running','submitted','verifying','accepted','repair_required'));
CREATE TABLE kernel_repair_charges(
 id uuid PRIMARY KEY, mission_id uuid NOT NULL REFERENCES missions,task_id uuid NOT NULL REFERENCES tasks,
 candidate_id text NOT NULL REFERENCES artifact_candidates,report_id text NOT NULL REFERENCES artifact_reports,
 principal_id uuid NOT NULL,recovery_epoch uuid NOT NULL,generation bigint NOT NULL,
 reason text NOT NULL CHECK(length(reason)>0 AND length(reason)<=8192),
 UNIQUE(candidate_id,report_id)
);
CREATE TRIGGER immutable_kernel_repair_charges BEFORE UPDATE OR DELETE ON kernel_repair_charges FOR EACH ROW EXECUTE FUNCTION immutable_record();

ALTER TABLE episode_runtime_operations ADD COLUMN usage jsonb NOT NULL DEFAULT '{"kind":"unknown"}';
ALTER TABLE episode_runtime_operations ADD CONSTRAINT kernel_usage_shape CHECK (
 COALESCE(usage = '{"kind":"unknown"}'::jsonb OR
 (jsonb_typeof(usage)='object' AND usage->>'kind'='measured_zero'
  AND usage - 'kind' - 'receipt_digest' = '{}'::jsonb AND jsonb_typeof(usage->'receipt_digest')='string' AND usage->>'receipt_digest' ~ '^[0-9a-f]{64}$'), false)
);
CREATE TABLE kernel_usage_evidence(
 id uuid PRIMARY KEY,operation_id uuid NOT NULL REFERENCES episode_runtime_operations,
 evidence jsonb NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TRIGGER immutable_kernel_usage_evidence BEFORE UPDATE OR DELETE ON kernel_usage_evidence FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE FUNCTION monotonic_kernel_usage() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
 IF OLD.usage->>'kind'='measured_zero' AND NEW.usage IS DISTINCT FROM OLD.usage THEN RAISE EXCEPTION 'immutable measured usage'; END IF;
 RETURN NEW;
END; $$;
CREATE TRIGGER monotonic_kernel_usage BEFORE UPDATE ON episode_runtime_operations FOR EACH ROW EXECUTE FUNCTION monotonic_kernel_usage();

-- Assembly policy is durable on the genuine task, so omitting an Episode on a
-- legacy request cannot bypass its usage reservation requirement.
ALTER TABLE gateway_operations ADD COLUMN usage jsonb NOT NULL DEFAULT '{"kind":"unknown"}';
ALTER TABLE gateway_operations ADD CONSTRAINT kernel_gateway_usage_shape CHECK (
 COALESCE(usage = '{"kind":"unknown"}'::jsonb OR
 (jsonb_typeof(usage)='object' AND usage->>'kind'='measured_zero'
  AND usage - 'kind' - 'receipt_digest' = '{}'::jsonb
  AND jsonb_typeof(usage->'receipt_digest')='string'
  AND usage->>'receipt_digest' ~ '^[0-9a-f]{64}$'), false)
);
CREATE TRIGGER monotonic_gateway_usage BEFORE UPDATE ON gateway_operations FOR EACH ROW EXECUTE FUNCTION monotonic_kernel_usage();
CREATE TABLE kernel_gateway_usage_evidence(
 id uuid PRIMARY KEY,operation_id uuid NOT NULL REFERENCES gateway_operations,
 evidence jsonb NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TRIGGER immutable_kernel_gateway_usage_evidence BEFORE UPDATE OR DELETE ON kernel_gateway_usage_evidence FOR EACH ROW EXECUTE FUNCTION immutable_record();

-- Cancellation intent revokes dispatch before environment observations.
ALTER TABLE attempts ADD COLUMN desired_cancel boolean NOT NULL DEFAULT false;
ALTER TABLE attempts DROP CONSTRAINT attempts_status_check;
ALTER TABLE attempts ADD CONSTRAINT attempts_status_check CHECK(status IN ('reserved','cancellation_requested','cancelled','quarantined'));
ALTER TABLE tasks DROP CONSTRAINT tasks_status_check;
ALTER TABLE tasks ADD CONSTRAINT tasks_status_check CHECK(status IN ('proposed','ready','running','submitted','verifying','accepted','repair_required','cancelled'));
ALTER TABLE mailbox_episodes DROP CONSTRAINT mailbox_episodes_state_check;
ALTER TABLE mailbox_episodes ADD CONSTRAINT mailbox_episodes_state_check CHECK(state IN ('admitted','active','yielded','completed','outcome_unknown','quarantined','cancellation_requested','cancelled'));
ALTER TABLE attempts ADD CONSTRAINT attempt_cancel_disposition CHECK((NOT desired_cancel AND status='reserved') OR (desired_cancel AND status IN ('cancellation_requested','cancelled','quarantined')));
CREATE FUNCTION monotonic_attempt_cancel() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
 IF OLD.desired_cancel AND NOT NEW.desired_cancel THEN RAISE EXCEPTION 'attempt cancellation cannot renew dispatch'; END IF;
 RETURN NEW;
END; $$;
CREATE TRIGGER monotonic_attempt_cancel BEFORE UPDATE ON attempts FOR EACH ROW EXECUTE FUNCTION monotonic_attempt_cancel();

-- A reservation can back exactly one immutable physical execution, including
-- the first imported Worker episode. Released reservations cannot be rebound.
ALTER TABLE reservations ADD COLUMN execution_id uuid UNIQUE REFERENCES supervised_workspaces(id);
-- Accepted legacy allocations already uniquely identify their Attempt. Adopt
-- that existing immutable association; never manufacture another reservation.
UPDATE reservations z SET execution_id=w.id FROM supervised_workspaces w WHERE w.attempt_id=z.attempt_id;
CREATE FUNCTION immutable_reservation_execution() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
 IF (OLD.execution_id IS NOT NULL AND NEW.execution_id IS DISTINCT FROM OLD.execution_id) OR (OLD.state='released' AND NEW.state IS DISTINCT FROM OLD.state) THEN RAISE EXCEPTION 'immutable physical reservation binding'; END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER immutable_reservation_execution BEFORE UPDATE ON reservations FOR EACH ROW EXECUTE FUNCTION immutable_reservation_execution();
