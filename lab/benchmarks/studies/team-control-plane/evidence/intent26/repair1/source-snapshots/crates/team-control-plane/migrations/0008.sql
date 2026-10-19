-- Additive v2 contracts. The old specifications, subjects, proof IDs, grants,
-- counters and optional original provenance are never rewritten or upgraded.
CREATE TABLE contract_schema_version(version integer PRIMARY KEY CHECK(version=2));
INSERT INTO contract_schema_version VALUES(2);
CREATE TABLE contract_legacy_bindings(
 task_id uuid NOT NULL REFERENCES tasks, spec_digest text NOT NULL,
 original_spec jsonb NOT NULL, disposition text NOT NULL CHECK(disposition='explicit_mapping_required'),
 PRIMARY KEY(task_id,spec_digest), FOREIGN KEY(task_id,spec_digest) REFERENCES task_specs(task_id,digest)
);
INSERT INTO contract_legacy_bindings SELECT task_id,digest,to_jsonb(s),'explicit_mapping_required' FROM task_specs s;
CREATE TABLE contract_work(
 id text PRIMARY KEY CHECK(id ~ '^[0-9a-f]{64}$'), task_id uuid NOT NULL UNIQUE REFERENCES tasks,
 work jsonb NOT NULL UNIQUE, mission_id uuid NOT NULL REFERENCES missions
);
CREATE TABLE contract_sources(
 id text PRIMARY KEY CHECK(id ~ '^[0-9a-f]{64}$'), work_id text NOT NULL REFERENCES contract_work,
 source_revision text NOT NULL, source jsonb NOT NULL, UNIQUE(work_id,source_revision)
);
CREATE TABLE contract_acceptance(
 id text PRIMARY KEY CHECK(id ~ '^[0-9a-f]{64}$'), contract jsonb NOT NULL
);
CREATE TABLE contract_intents(
 id text PRIMARY KEY CHECK(id ~ '^[0-9a-f]{64}$'), work_id text NOT NULL REFERENCES contract_work,
 source_id text NOT NULL REFERENCES contract_sources, acceptance_id text NOT NULL REFERENCES contract_acceptance,
 admitted_version bigint NOT NULL CHECK(admitted_version>0), intent jsonb NOT NULL,
 principal_id uuid NOT NULL, recovery_epoch uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 UNIQUE(work_id,admitted_version)
);
CREATE TABLE contract_current(
 task_id uuid PRIMARY KEY REFERENCES tasks, revision bigint NOT NULL CHECK(revision>0),
 intent_id text NOT NULL REFERENCES contract_intents, plan_id text, admission_open boolean NOT NULL
);
CREATE TABLE contract_commands(
 principal_id uuid NOT NULL, family text NOT NULL, key text NOT NULL, argument_digest text NOT NULL,
 task_id uuid NOT NULL REFERENCES tasks, disposition text NOT NULL CHECK(disposition IN ('applied','rejected')),
 result jsonb NOT NULL, recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(), PRIMARY KEY(principal_id,family,key)
);
CREATE TABLE contract_proposals(
 id text PRIMARY KEY, task_id uuid NOT NULL REFERENCES tasks, principal_id uuid NOT NULL,
 proposal jsonb NOT NULL, recovery_epoch uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE contract_delegations(
 id uuid PRIMARY KEY, task_id uuid NOT NULL REFERENCES tasks, mission_id uuid NOT NULL REFERENCES missions,
 principal_id uuid NOT NULL, role_epoch bigint NOT NULL CHECK(role_epoch>0), work_id text NOT NULL REFERENCES contract_work,
 delegation jsonb NOT NULL, by_principal uuid NOT NULL, recovery_epoch uuid NOT NULL, expires_at timestamptz NOT NULL
);
CREATE TABLE contract_plans(
 id text PRIMARY KEY, task_id uuid NOT NULL REFERENCES tasks, intent_id text NOT NULL REFERENCES contract_intents,
 acceptance_id text NOT NULL REFERENCES contract_acceptance, admitted_version bigint NOT NULL CHECK(admitted_version>0),
 plan jsonb NOT NULL, principal_id uuid NOT NULL, recovery_epoch uuid NOT NULL, UNIQUE(task_id,admitted_version)
);
ALTER TABLE contract_current ADD CONSTRAINT contract_current_plan FOREIGN KEY(plan_id) REFERENCES contract_plans;
CREATE TABLE contract_completed_steps(
 task_id uuid NOT NULL REFERENCES tasks, plan_id text NOT NULL REFERENCES contract_plans,
 step_id text NOT NULL, step jsonb NOT NULL, PRIMARY KEY(task_id,step_id)
);
CREATE TABLE contract_verification_bindings(
 id text PRIMARY KEY, task_id uuid NOT NULL REFERENCES tasks, episode_id uuid NOT NULL UNIQUE REFERENCES mailbox_episodes,
 intent_id text NOT NULL REFERENCES contract_intents, acceptance_id text NOT NULL REFERENCES contract_acceptance,
 verifier uuid NOT NULL, policy text NOT NULL, environment text NOT NULL, recovery_epoch uuid NOT NULL,
 created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE contract_proofs(
 id text PRIMARY KEY, task_id uuid NOT NULL REFERENCES tasks, intent_id text NOT NULL REFERENCES contract_intents,
 acceptance_id text NOT NULL REFERENCES contract_acceptance, verifier uuid NOT NULL,
 outcome text NOT NULL CHECK(outcome IN ('production','investigation_complete')), evidence jsonb NOT NULL,
 created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE contract_outcomes(
 subject text PRIMARY KEY, task_id uuid NOT NULL REFERENCES tasks, intent_id text NOT NULL REFERENCES contract_intents,
 proof_id text NOT NULL REFERENCES contract_proofs, outcome text NOT NULL CHECK(outcome IN ('production','investigation_complete')),
 policy text NOT NULL, by_principal uuid NOT NULL, recovery_epoch uuid NOT NULL,
 created_at timestamptz NOT NULL DEFAULT clock_timestamp(), UNIQUE(task_id,intent_id)
);
CREATE TABLE contract_dependencies(
 task_id uuid NOT NULL REFERENCES tasks, prerequisite_work text NOT NULL REFERENCES contract_work,
 intent_id text NOT NULL REFERENCES contract_intents, accepted_subject text NOT NULL REFERENCES contract_outcomes,
 policy text NOT NULL, disposition text NOT NULL CHECK(disposition IN ('active','suspended')),
 PRIMARY KEY(task_id,prerequisite_work)
);
CREATE TABLE contract_dependency_revalidations(
 id text PRIMARY KEY, task_id uuid NOT NULL REFERENCES tasks, intent_id text NOT NULL REFERENCES contract_intents,
 dependencies jsonb NOT NULL, principal_id uuid NOT NULL, recovery_epoch uuid NOT NULL,
 created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TRIGGER immutable_contract_legacy BEFORE UPDATE OR DELETE ON contract_legacy_bindings FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_work BEFORE UPDATE OR DELETE ON contract_work FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_sources BEFORE UPDATE OR DELETE ON contract_sources FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_acceptance BEFORE UPDATE OR DELETE ON contract_acceptance FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_intents BEFORE UPDATE OR DELETE ON contract_intents FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_commands BEFORE UPDATE OR DELETE ON contract_commands FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_proposals BEFORE UPDATE OR DELETE ON contract_proposals FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_delegations BEFORE UPDATE OR DELETE ON contract_delegations FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_plans BEFORE UPDATE OR DELETE ON contract_plans FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_completed BEFORE UPDATE OR DELETE ON contract_completed_steps FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_bindings BEFORE UPDATE OR DELETE ON contract_verification_bindings FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_proofs BEFORE UPDATE OR DELETE ON contract_proofs FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_outcomes BEFORE UPDATE OR DELETE ON contract_outcomes FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_contract_revalidations BEFORE UPDATE OR DELETE ON contract_dependency_revalidations FOR EACH ROW EXECUTE FUNCTION immutable_record();
