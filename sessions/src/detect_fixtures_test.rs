use std::io::Write;
use std::path::PathBuf;

use harp_session_index::detect::detect_dialect;
use harp_session_index::DialectId;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn detects_dual_stream_fixture() {
    let path = fixture("append-dual-stream.jsonl");
    assert!(path.is_file(), "missing fixture {}", path.display());
    let detected = detect_dialect(&path).expect("detect");
    assert_eq!(detected, DialectId::TraeCliDualStreamV1);
}

#[test]
fn refuses_response_item_era_as_unsupported() {
    let path = fixture("detect-response-item-refuse.jsonl");
    let err = detect_dialect(&path).expect_err("should refuse");
    match err {
        harp_session_index::SessionIndexError::UnsupportedDialect { dialect, .. } => {
            assert!(dialect.contains("response_item"), "{dialect}");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

fn generated_rollout(nonblank: usize) -> tempfile::NamedTempFile {
    assert!(nonblank >= 4);
    let mut file = tempfile::NamedTempFile::new().expect("temporary rollout");
    for record in [
        serde_json::json!({"type": "session_meta", "payload": {"id": "018f22e2-7c3b-7def-8123-456789abcdef"}}),
        serde_json::json!({"type": "event_msg", "payload": {"type": "user_message"}}),
        serde_json::json!({"type": "history_mutation", "payload": {"operation": "append", "items": []}}),
        serde_json::json!({"type": "response_item", "payload": {}}),
    ] {
        writeln!(file, "{}", record).expect("write material record");
    }
    for index in 4..nonblank {
        writeln!(file, "{{\"type\":\"neutral-{index}\"}}").expect("write neutral record");
    }
    file
}

#[test]
fn accepts_history_mutation_at_exactly_one_percent_of_nonblank_records() {
    let rollout = generated_rollout(100);

    assert_eq!(
        detect_dialect(rollout.path()).expect("one percent is material"),
        DialectId::TraeCliDualStreamV1
    );
}

#[test]
fn rejects_history_mutation_below_one_percent_of_nonblank_records() {
    let rollout = generated_rollout(101);

    let error = detect_dialect(rollout.path()).expect_err("below one percent is not material");
    assert!(matches!(
        error,
        harp_session_index::SessionIndexError::DetectFailed { .. }
    ));
}
