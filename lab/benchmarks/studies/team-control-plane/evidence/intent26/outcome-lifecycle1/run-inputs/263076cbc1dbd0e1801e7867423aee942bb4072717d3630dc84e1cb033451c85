CREATE TABLE mailbox_schema_version(version integer NOT NULL CHECK(version=1));
INSERT INTO mailbox_schema_version VALUES(1);
CREATE TABLE mailbox_messages(
 id uuid PRIMARY KEY, mission_id uuid NOT NULL REFERENCES missions,
 recovery_epoch uuid NOT NULL, sender uuid NOT NULL, sender_role text NOT NULL CHECK(sender_role IN ('owner','lead','worker','verifier')),
 task_id uuid, task_generation bigint, attempt_id uuid REFERENCES attempts, approved_spec_digest text, spec_revision bigint,
 kind text NOT NULL CHECK(kind IN ('question','answer','proposal','notice')),
 text text NOT NULL CHECK(length(text)>0 AND octet_length(text)<=16384),
 reply_to uuid REFERENCES mailbox_messages, attention text NOT NULL CHECK(attention IN ('normal','critical')),
 artifacts jsonb NOT NULL, created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 FOREIGN KEY(task_id,mission_id) REFERENCES tasks(id,mission_id)
);
CREATE TABLE mailbox_recipients(message_id uuid NOT NULL REFERENCES mailbox_messages, principal_id uuid NOT NULL, PRIMARY KEY(message_id,principal_id));
CREATE TABLE mailbox_commands(
 principal_id uuid NOT NULL, family text NOT NULL, key text NOT NULL,
 digest text NOT NULL, result jsonb NOT NULL, disposition text NOT NULL CHECK(disposition IN ('applied','rejected')),
 scope_kind text NOT NULL CHECK(scope_kind IN ('owner','mission','task')), scope_mission uuid REFERENCES missions,scope_task uuid, recovery_epoch uuid NOT NULL,
 FOREIGN KEY(scope_task,scope_mission) REFERENCES tasks(id,mission_id),
 CHECK((scope_kind='owner' AND scope_mission IS NULL AND scope_task IS NULL) OR (scope_kind='mission' AND scope_mission IS NOT NULL AND scope_task IS NULL) OR (scope_kind='task' AND scope_mission IS NOT NULL AND scope_task IS NOT NULL)),
 PRIMARY KEY(principal_id,family,key)
);
CREATE TRIGGER immutable_mailbox_messages BEFORE UPDATE OR DELETE ON mailbox_messages FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_recipients BEFORE UPDATE OR DELETE ON mailbox_recipients FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_commands BEFORE UPDATE OR DELETE ON mailbox_commands FOR EACH ROW EXECUTE FUNCTION immutable_record();

ALTER TABLE credentials ADD COLUMN lead_generation bigint NOT NULL DEFAULT 0 CHECK(lead_generation>=0);
CREATE TABLE mission_leads(mission_id uuid PRIMARY KEY REFERENCES missions,generation bigint NOT NULL CHECK(generation>0),principal_id uuid NOT NULL);
CREATE TABLE lead_replacements(id uuid PRIMARY KEY,mission_id uuid NOT NULL REFERENCES missions,generation bigint NOT NULL,principal_id uuid NOT NULL,by_principal uuid NOT NULL,UNIQUE(mission_id,generation));
CREATE TRIGGER immutable_lead_replacements BEFORE UPDATE OR DELETE ON lead_replacements FOR EACH ROW EXECUTE FUNCTION immutable_record();

CREATE TABLE mailbox_inputs(id uuid PRIMARY KEY,mission_id uuid NOT NULL REFERENCES missions,task_id uuid,principal_id uuid NOT NULL,recovery_epoch uuid NOT NULL,digest text NOT NULL CHECK(digest ~ '^[0-9a-f]{64}$'),packet jsonb NOT NULL,FOREIGN KEY(task_id,mission_id) REFERENCES tasks(id,mission_id));
CREATE TABLE mailbox_input_messages(input_id uuid NOT NULL REFERENCES mailbox_inputs,message_id uuid NOT NULL REFERENCES mailbox_messages,PRIMARY KEY(input_id,message_id));
CREATE TRIGGER immutable_mailbox_inputs BEFORE UPDATE OR DELETE ON mailbox_inputs FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_input_messages BEFORE UPDATE OR DELETE ON mailbox_input_messages FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TABLE mailbox_episodes(
 id uuid PRIMARY KEY,mission_id uuid NOT NULL REFERENCES missions,principal_id uuid NOT NULL,
 role text NOT NULL CHECK(role IN ('lead','worker','verifier')),purpose text NOT NULL CHECK(purpose IN ('planning','implementation','verification')),
 attempt_id uuid REFERENCES attempts,task_id uuid,task_generation bigint,lead_generation bigint,
 recovery_epoch uuid NOT NULL, input_digest text NOT NULL CHECK(input_digest ~ '^[0-9a-f]{64}$'),
 deadline timestamptz NOT NULL,state text NOT NULL CHECK(state IN ('admitted','active','yielded','completed','outcome_unknown','quarantined')),
 input_packet uuid REFERENCES mailbox_inputs, owned_capacity boolean NOT NULL,event_digest text NOT NULL CHECK(event_digest ~ '^[0-9a-f]{64}$'),turn_id text, start_operation uuid REFERENCES gateway_operations,
 FOREIGN KEY(task_id,mission_id) REFERENCES tasks(id,mission_id)
);
CREATE UNIQUE INDEX mailbox_one_live_episode ON mailbox_episodes(principal_id) WHERE state IN ('admitted','active','outcome_unknown','quarantined');

CREATE TABLE mailbox_presentations(id uuid PRIMARY KEY,message_id uuid NOT NULL REFERENCES mailbox_messages,episode_id uuid NOT NULL REFERENCES mailbox_episodes,principal_id uuid NOT NULL,input_digest text NOT NULL,runtime_turn text,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(),UNIQUE(message_id,episode_id,principal_id));
CREATE TABLE mailbox_acknowledgments(id uuid PRIMARY KEY,presentation_id uuid NOT NULL UNIQUE REFERENCES mailbox_presentations,principal_id uuid NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TABLE mailbox_resolutions(message_id uuid PRIMARY KEY REFERENCES mailbox_messages,answer_id uuid NOT NULL REFERENCES mailbox_messages,principal_id uuid NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TRIGGER immutable_mailbox_presentations BEFORE UPDATE OR DELETE ON mailbox_presentations FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_acknowledgments BEFORE UPDATE OR DELETE ON mailbox_acknowledgments FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_resolutions BEFORE UPDATE OR DELETE ON mailbox_resolutions FOR EACH ROW EXECUTE FUNCTION immutable_record();

CREATE TABLE mailbox_checkpoints(id uuid PRIMARY KEY,episode_id uuid NOT NULL REFERENCES mailbox_episodes,principal_id uuid NOT NULL,checkpoint jsonb NOT NULL,executions jsonb NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TABLE mailbox_waits(episode_id uuid NOT NULL REFERENCES mailbox_episodes,kind text NOT NULL CHECK(kind IN ('question','task')),reference_id uuid NOT NULL,workflow_id text NOT NULL REFERENCES workflow_registry(workflow_id),satisfied boolean NOT NULL,PRIMARY KEY(episode_id,kind,reference_id));
CREATE TABLE mailbox_wait_satisfactions(episode_id uuid NOT NULL,kind text NOT NULL,reference_id uuid NOT NULL,evidence_id uuid NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(),PRIMARY KEY(episode_id,kind,reference_id),FOREIGN KEY(episode_id,kind,reference_id) REFERENCES mailbox_waits(episode_id,kind,reference_id));
CREATE TRIGGER immutable_mailbox_checkpoints BEFORE UPDATE OR DELETE ON mailbox_checkpoints FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_wait_satisfactions BEFORE UPDATE OR DELETE ON mailbox_wait_satisfactions FOR EACH ROW EXECUTE FUNCTION immutable_record();

ALTER TABLE mailbox_recipients ADD COLUMN lead_seat boolean NOT NULL DEFAULT false;

ALTER TABLE mailbox_messages ADD COLUMN sender_lead_generation bigint;
CREATE TABLE mailbox_mcp_sessions(id uuid PRIMARY KEY,principal_id uuid NOT NULL,token_hash text NOT NULL,recovery_epoch uuid NOT NULL,initialized boolean NOT NULL DEFAULT false,expires_at timestamptz NOT NULL);

CREATE TABLE mailbox_approval_requests(id uuid PRIMARY KEY,mission_id uuid NOT NULL REFERENCES missions,task_id uuid,generation bigint,requester uuid NOT NULL,requester_lead_generation bigint,executor uuid NOT NULL,operation jsonb NOT NULL,argument_digest text NOT NULL,recovery_epoch uuid NOT NULL,expires_at timestamptz NOT NULL,FOREIGN KEY(task_id,mission_id) REFERENCES tasks(id,mission_id),CHECK((task_id IS NULL)=(generation IS NULL)));
CREATE TABLE mailbox_approval_grants(request_id uuid PRIMARY KEY REFERENCES mailbox_approval_requests,by_principal uuid NOT NULL,rationale text NOT NULL,argument_digest text NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TABLE mailbox_approval_consumptions(id uuid PRIMARY KEY,request_id uuid NOT NULL UNIQUE REFERENCES mailbox_approval_grants,executor uuid NOT NULL,argument_digest text NOT NULL,recovery_epoch uuid NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TRIGGER immutable_mailbox_approval_requests BEFORE UPDATE OR DELETE ON mailbox_approval_requests FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_approval_grants BEFORE UPDATE OR DELETE ON mailbox_approval_grants FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_approval_consumptions BEFORE UPDATE OR DELETE ON mailbox_approval_consumptions FOR EACH ROW EXECUTE FUNCTION immutable_record();

CREATE TABLE mailbox_proposals(id uuid PRIMARY KEY,mission_id uuid NOT NULL REFERENCES missions,principal_id uuid NOT NULL,lead_generation bigint NOT NULL,proposal jsonb NOT NULL,digest text NOT NULL,recovery_epoch uuid NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TABLE mailbox_decisions(id uuid PRIMARY KEY,proposal_id uuid NOT NULL UNIQUE REFERENCES mailbox_proposals,by_principal uuid NOT NULL,lead_generation bigint,approved boolean NOT NULL,rationale text NOT NULL,proposal_digest text NOT NULL,recovery_epoch uuid NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TRIGGER immutable_mailbox_proposals BEFORE UPDATE OR DELETE ON mailbox_proposals FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_decisions BEFORE UPDATE OR DELETE ON mailbox_decisions FOR EACH ROW EXECUTE FUNCTION immutable_record();

CREATE TABLE mailbox_episode_runtime_bindings(episode_id uuid PRIMARY KEY REFERENCES mailbox_episodes,start_operation uuid NOT NULL UNIQUE REFERENCES gateway_operations,runtime_turn text NOT NULL CHECK(octet_length(runtime_turn)>0 AND octet_length(runtime_turn)<=1024),by_principal uuid NOT NULL,recovery_epoch uuid NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TRIGGER immutable_mailbox_episode_runtime_bindings BEFORE UPDATE OR DELETE ON mailbox_episode_runtime_bindings FOR EACH ROW EXECUTE FUNCTION immutable_record();

CREATE TABLE mailbox_delivery_intents(id uuid PRIMARY KEY,message_id uuid NOT NULL REFERENCES mailbox_messages,episode_id uuid NOT NULL REFERENCES mailbox_episodes,gateway_intent jsonb NOT NULL,input_digest text NOT NULL,episode_input_digest text NOT NULL,expected_turn text NOT NULL,required_decision uuid,required_approval uuid UNIQUE,by_principal uuid NOT NULL,recovery_epoch uuid NOT NULL,workflow_reference jsonb NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(),UNIQUE(message_id,episode_id));
CREATE TABLE mailbox_delivery_claims(delivery_id uuid PRIMARY KEY REFERENCES mailbox_delivery_intents,gateway_operation uuid NOT NULL UNIQUE REFERENCES gateway_operations,controller uuid NOT NULL,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TRIGGER immutable_mailbox_delivery_intents BEFORE UPDATE OR DELETE ON mailbox_delivery_intents FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_mailbox_delivery_claims BEFORE UPDATE OR DELETE ON mailbox_delivery_claims FOR EACH ROW EXECUTE FUNCTION immutable_record();

CREATE TABLE mailbox_episode_turn_completions(episode_id uuid PRIMARY KEY REFERENCES mailbox_episodes,delivery_id uuid NOT NULL UNIQUE REFERENCES mailbox_delivery_intents,expected_turn text NOT NULL,observed_turn text,recorded_at timestamptz NOT NULL DEFAULT clock_timestamp());
CREATE TRIGGER immutable_mailbox_episode_turn_completions BEFORE UPDATE OR DELETE ON mailbox_episode_turn_completions FOR EACH ROW EXECUTE FUNCTION immutable_record();
