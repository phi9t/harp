use std::fs;
use std::path::{Path, PathBuf};

use harp_session_index::index::{index_rollout, IndexOptions, SessionSummary};
use harp_session_index::store::{session_dir, validate_session_id};
use serde_json::Value;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn read_summary(index_root: &Path, session_id: &str) -> SessionSummary {
    let path = session_dir(index_root, session_id)
        .expect("valid session id")
        .join("summary.json");
    let raw = fs::read_to_string(&path).expect("summary.json");
    serde_json::from_str(&raw).expect("parse summary")
}

#[test]
fn session_store_accepts_only_canonical_lowercase_hyphenated_uuids() {
    let root = Path::new("/tmp/index-root");
    let canonical = "018f22e2-7c3b-7def-8123-456789abcdef";
    assert_eq!(
        validate_session_id(canonical).expect("canonical UUID"),
        canonical
    );
    assert_eq!(
        session_dir(root, canonical).expect("canonical UUID"),
        root.join(canonical)
    );

    for invalid in [
        "",
        ".",
        "..",
        "../escape",
        "a/b",
        "/tmp/escape",
        "not-a-uuid",
        "018F22E2-7C3B-7DEF-8123-456789ABCDEF",
        "018f22e27c3b7def8123456789abcdef",
        "{018f22e2-7c3b-7def-8123-456789abcdef}",
        "018f22e2-7c3b-7def-8123-456789abcde\n",
    ] {
        assert!(
            validate_session_id(invalid).is_err(),
            "validator accepted {invalid:?}"
        );
        assert!(session_dir(root, invalid).is_err(), "accepted {invalid:?}");
    }
}

#[test]
fn indexes_dual_stream_and_is_reparse_stable() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("idx");
    let rollout = fixture("append-dual-stream.jsonl");

    let first = index_rollout(
        &rollout,
        &root,
        IndexOptions {
            oversize_line_bytes: 1024 * 1024,
            artifacts_dir: None,
        },
    )
    .expect("index");
    assert_eq!(first.dialect, "traecli.dual_stream.v1");
    assert_eq!(first.session_id, "018f22e2-7c3b-7def-8123-456789abc001");
    assert_eq!(first.history_mutation.append, 1);
    assert_eq!(first.turns.started, 1);
    assert_eq!(first.turns.completed, 1);
    assert_eq!(first.exec_command_end, 1);
    assert!(
        first
            .event_msg_counts
            .get("user_message")
            .copied()
            .unwrap_or(0)
            >= 1
    );

    let digest1 = first.index_digest.clone();
    let second = index_rollout(
        &rollout,
        &root,
        IndexOptions {
            oversize_line_bytes: 1024 * 1024,
            artifacts_dir: None,
        },
    )
    .expect("reindex");
    assert_eq!(
        digest1, second.index_digest,
        "reparse digest must be stable"
    );
}

#[test]
fn indexes_replace_and_context_compacted() {
    let tmp = tempfile::tempdir().unwrap();
    let summary = index_rollout(
        &fixture("replace-compacted.jsonl"),
        tmp.path(),
        IndexOptions::default(),
    )
    .expect("index");
    assert_eq!(summary.history_mutation.replace, 1);
    assert_eq!(
        summary
            .event_msg_counts
            .get("context_compacted")
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn records_empty_stdio_exec_divergence() {
    let tmp = tempfile::tempdir().unwrap();
    let summary = index_rollout(
        &fixture("exec-empty-stdio.jsonl"),
        tmp.path(),
        IndexOptions::default(),
    )
    .expect("index");
    assert_eq!(summary.exec_command_end, 1);
    assert_eq!(summary.divergence.exec_empty_output_nonzero, 1);
    assert_eq!(summary.turns.completed, 1);
}

#[test]
fn records_tool_result_reference_from_artifacts_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let art = tmp.path().join("artifacts").join("tool-results");
    fs::create_dir_all(&art).unwrap();
    let blob = art.join("exec_command-call_ARTIFACTBRIDGE1.txt");
    fs::write(&blob, b"hello artifact").unwrap();
    // Nested TraeCLI filenames append `_exec-<uuid>`; extraction must stop
    // before that suffix or the HM bridge fails.
    let nested = art.join(
        "exec_command-code-mode-nested_29_call_ARTIFACTBRIDGE1_exec-2911916b-4a00-4c4a-acc7-f96d3f7b0429-5.txt",
    );
    fs::write(&nested, b"nested").unwrap();

    let summary = index_rollout(
        &fixture("exec-artifact-bridge.jsonl"),
        &tmp.path().join("idx"),
        IndexOptions {
            oversize_line_bytes: 1024 * 1024,
            artifacts_dir: Some(art),
        },
    )
    .expect("index");
    assert_eq!(summary.tool_result_refs, 2);
    // Unique HM call_ids with ≥1 artifact, not per-file count.
    assert_eq!(summary.divergence.hm_call_ids_with_artifact, 1);
}

#[test]
fn verify_reindex_matches_claimed_summary() {
    use harp_session_index::verify::{verify_session, VerifyOptions, VerifyVerdict};

    let tmp = tempfile::tempdir().unwrap();
    let art = tmp.path().join("artifacts").join("tool-results");
    fs::create_dir_all(&art).unwrap();
    fs::write(
        art.join(
            "exec_command-code-mode-nested_29_call_ARTIFACTBRIDGE1_exec-aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.txt",
        ),
        b"nested",
    )
    .unwrap();
    let root = tmp.path().join("idx");
    let rollout = fixture("exec-artifact-bridge.jsonl");
    let indexed = index_rollout(
        &rollout,
        &root,
        IndexOptions {
            oversize_line_bytes: 1024 * 1024,
            artifacts_dir: Some(art.clone()),
        },
    )
    .expect("index");

    let report = verify_session(
        &rollout,
        &indexed.session_id,
        VerifyOptions {
            index_root: root,
            artifacts_dir: Some(art),
            oversize_line_bytes: 1024 * 1024,
            write_report: true,
        },
    )
    .expect("verify");
    assert_eq!(report.verdict, VerifyVerdict::Pass);
    assert_eq!(report.checks_passed, report.checks_total);
    assert!(!report.feedback.is_empty());
}

#[test]
fn spills_oversize_jsonl_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let rollout = tmp.path().join("oversize.jsonl");
    // Build a line larger than threshold.
    let big_msg = "x".repeat(8_000);
    let record = serde_json::json!({
        "timestamp": "2026-01-01T00:00:00.000Z",
        "type": "session_meta",
        "payload": {
            "id": "018f22e2-7c3b-7def-8123-456789abc099",
            "cwd": "/tmp",
            "originator": "test",
            "cli_version": "0",
            "source": "cli",
            "model_provider": "trae",
            "history_mode": "legacy",
            "pad": big_msg
        }
    });
    let line = serde_json::to_string(&record).unwrap();
    assert!(line.len() > 4_000);
    let mut body = line;
    body.push('\n');
    body.push_str(r#"{"timestamp":"2026-01-01T00:00:00.001Z","type":"event_msg","payload":{"type":"user_message","message":"hi","images":[],"local_images":[],"text_elements":[]}}"#);
    body.push('\n');
    body.push_str(r#"{"timestamp":"2026-01-01T00:00:00.002Z","type":"history_mutation","payload":{"version":1,"commit_id":"c","operation":"append","items":[{"type":"message","role":"user","content":[]}]}}"#);
    body.push('\n');
    fs::write(&rollout, body).unwrap();

    let summary = index_rollout(
        &rollout,
        &tmp.path().join("idx"),
        IndexOptions {
            oversize_line_bytes: 4_000,
            artifacts_dir: None,
        },
    )
    .expect("index");
    assert_eq!(summary.oversize_lines, 1);
    let spill_dir = session_dir(tmp.path().join("idx").as_path(), &summary.session_id)
        .expect("valid session id")
        .join("spills");
    assert!(spill_dir.is_dir());
    let spills: Vec<_> = fs::read_dir(&spill_dir).unwrap().collect();
    assert_eq!(spills.len(), 1);
}

#[test]
fn rejects_malformed_oversize_jsonl_without_publishing_a_session() {
    let tmp = tempfile::tempdir().unwrap();
    let rollout = tmp.path().join("malformed-oversize.jsonl");
    let valid = serde_json::json!({
        "timestamp": "2026-01-01T00:00:00.000Z",
        "type": "session_meta",
        "payload": {
            "id": "018f22e2-7c3b-7def-8123-456789abc099",
            "cwd": "/tmp",
            "originator": "test",
            "cli_version": "0",
            "source": "cli",
            "model_provider": "trae",
            "history_mode": "legacy"
        }
    });
    let mut body = serde_json::to_string(&valid).unwrap();
    body.push('\n');
    body.push_str(&format!(
        r#"{{"type":"event_msg","payload":{{"pad":"{}"}}"#,
        "x".repeat(8_000)
    ));
    body.push('\n');
    fs::write(&rollout, body).unwrap();

    let root = tmp.path().join("idx");
    let error = index_rollout(
        &rollout,
        &root,
        IndexOptions {
            oversize_line_bytes: 4_000,
            artifacts_dir: None,
        },
    )
    .expect_err("malformed JSON must fail closed even when oversize");

    assert!(matches!(
        error,
        harp_session_index::SessionIndexError::InvalidJsonl { line: 2, .. }
    ));
    assert!(!root.exists(), "failed indexing must not publish a session");
}

#[test]
fn reports_missing_session_identity_as_a_typed_error() {
    let tmp = tempfile::tempdir().unwrap();
    let rollout = tmp.path().join("missing-session-id.jsonl");
    fs::write(
        &rollout,
        concat!(
            "{\"type\":\"session_meta\",\"payload\":{}}\n",
            "{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\"}}\n",
            "{\"type\":\"history_mutation\",\"payload\":{\"operation\":\"append\",\"items\":[]}}\n"
        ),
    )
    .unwrap();

    let error = index_rollout(&rollout, tmp.path(), IndexOptions::default())
        .expect_err("missing identity must fail");
    assert!(matches!(
        error,
        harp_session_index::SessionIndexError::MissingSessionId { .. }
    ));
}

#[test]
fn indexes_collab_spawn_without_turn_id() {
    let tmp = tempfile::tempdir().unwrap();
    let summary = index_rollout(
        &fixture("collab-no-turn-id.jsonl"),
        tmp.path(),
        IndexOptions::default(),
    )
    .expect("index");
    assert_eq!(summary.collab_spawn_end, 1);
    assert_eq!(summary.divergence.collab_events_missing_turn_id, 1);
}

#[test]
fn summary_json_round_trips() {
    let tmp = tempfile::tempdir().unwrap();
    let summary = index_rollout(
        &fixture("append-dual-stream.jsonl"),
        tmp.path(),
        IndexOptions::default(),
    )
    .expect("index");
    let loaded = read_summary(tmp.path(), &summary.session_id);
    assert_eq!(loaded.index_digest, summary.index_digest);
    let raw: Value = serde_json::from_str(
        &fs::read_to_string(
            session_dir(tmp.path(), &summary.session_id)
                .expect("valid session id")
                .join("summary.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(raw.get("top_type_counts").is_some());
}
