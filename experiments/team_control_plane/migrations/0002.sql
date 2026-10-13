CREATE TABLE artifact_schema_version(version integer PRIMARY KEY CHECK(version=1));
INSERT INTO artifact_schema_version VALUES(1);
ALTER TABLE credentials DROP CONSTRAINT credentials_role_check;
ALTER TABLE credentials ADD CONSTRAINT credentials_role_check CHECK(role IN ('owner','lead','worker','verifier'));
ALTER TABLE credentials DROP CONSTRAINT credentials_check;
ALTER TABLE credentials ADD CONSTRAINT credentials_check CHECK(
 (role='owner' AND mission_id IS NULL AND task_id IS NULL AND generation IS NULL)
 OR (role='lead' AND mission_id IS NOT NULL AND generation IS NULL)
 OR (role IN ('worker','verifier') AND mission_id IS NOT NULL AND task_id IS NOT NULL AND generation>0)
);
ALTER TABLE tasks DROP CONSTRAINT tasks_status_check;
ALTER TABLE tasks ADD CONSTRAINT tasks_status_check CHECK(status IN ('proposed','ready','running','submitted','verifying','accepted'));
CREATE TABLE artifact_policies(task_id uuid PRIMARY KEY REFERENCES tasks, verifier uuid NOT NULL, lead_acceptance boolean NOT NULL, base text NOT NULL, suite text NOT NULL, environment text NOT NULL, mandatory_commands jsonb NOT NULL);
CREATE TABLE artifact_objects(digest text PRIMARY KEY CHECK(digest ~ '^[0-9a-f]{64}$'), manifest jsonb NOT NULL);
CREATE TABLE artifact_candidates(id text PRIMARY KEY, task_id uuid NOT NULL REFERENCES tasks, implementer uuid NOT NULL, candidate jsonb NOT NULL, artifact text NOT NULL REFERENCES artifact_objects);
CREATE TABLE artifact_reports(id text PRIMARY KEY, candidate_id text NOT NULL REFERENCES artifact_candidates, verifier uuid NOT NULL, verification jsonb NOT NULL);
CREATE TABLE artifact_current(task_id uuid PRIMARY KEY REFERENCES tasks, candidate_id text NOT NULL REFERENCES artifact_candidates, accepted_report text REFERENCES artifact_reports);
CREATE TABLE artifact_report_logs(report_id text NOT NULL REFERENCES artifact_reports, digest text NOT NULL REFERENCES artifact_objects, PRIMARY KEY(report_id,digest));
CREATE TRIGGER immutable_artifact_objects BEFORE UPDATE OR DELETE ON artifact_objects FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_artifact_candidates BEFORE UPDATE OR DELETE ON artifact_candidates FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_artifact_reports BEFORE UPDATE OR DELETE ON artifact_reports FOR EACH ROW EXECUTE FUNCTION immutable_record();
CREATE TRIGGER immutable_artifact_logs BEFORE UPDATE OR DELETE ON artifact_report_logs FOR EACH ROW EXECUTE FUNCTION immutable_record();
