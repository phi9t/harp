use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn scrubs_ipv4_addresses() {
    let sanitizer = Sanitizer::with_default_config();
    let text = "Connecting to 10.116.77.207:22 and proxy 10.105.215.43 now.";
    let scrubbed = sanitizer.scrub_str(text);
    assert_eq!(
        scrubbed,
        "Connecting to [REDACTED_IP]:22 and proxy [REDACTED_IP] now."
    );
}

#[test]
fn scrubs_user_home_paths() {
    let sanitizer = Sanitizer::with_default_config();
    let text = "Files located at /data02/home/alice.smith/workspace/repo and /Users/bob/repo and /home/charlie/code.";
    let scrubbed = sanitizer.scrub_str(text);
    assert_eq!(
        scrubbed,
        "Files located at [REDACTED_USER_HOME]/workspace/repo and [REDACTED_USER_HOME]/repo and [REDACTED_USER_HOME]/code."
    );
}

#[test]
fn scrubs_emails() {
    let sanitizer = Sanitizer::with_default_config();
    let text = "Contact developer at user.name+tag@example.corp.com or admin@internal.net.";
    let scrubbed = sanitizer.scrub_str(text);
    assert_eq!(
        scrubbed,
        "Contact developer at [REDACTED_EMAIL] or [REDACTED_EMAIL]."
    );
}

#[test]
fn scrubs_node_hostnames() {
    let sanitizer = Sanitizer::with_default_config();
    let text = "Host n116-077-207 is running devx_b200_dbg with jumper_general_us_proxy.";
    let scrubbed = sanitizer.scrub_str(text);
    assert_eq!(
        scrubbed,
        "Host [REDACTED_HOST] is running [REDACTED_HOST] with [REDACTED_HOST]."
    );
}

#[test]
fn scrubs_private_keys_and_tokens() {
    let sanitizer = Sanitizer::with_default_config();
    let text = "Authorization: Bearer secret_token_value_1234567890;\n-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----\nEnd of cert.";
    let scrubbed = sanitizer.scrub_str(text);
    assert!(scrubbed.contains("Bearer [REDACTED_TOKEN]"));
    assert!(scrubbed.contains("[REDACTED_PRIVATE_KEY]"));
    assert!(!scrubbed.contains("MIIEow"));
}

#[test]
fn scrubs_proprietary_terms() {
    let sanitizer = Sanitizer::with_default_config();
    let text = "Syncing with bytedance internal services and lark feishu bot.";
    let scrubbed = sanitizer.scrub_str(text);
    assert_eq!(
        scrubbed,
        "Syncing with [REDACTED_ORG] internal services and [REDACTED_ORG] [REDACTED_ORG] bot."
    );
}

#[test]
fn scan_detects_violations_in_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("leak.txt");
    fs::write(
        &file_path,
        "Found 10.116.77.207 and user at /Users/david/secret",
    )
    .unwrap();

    let sanitizer = Sanitizer::with_default_config();
    let report = sanitizer.scan_path(dir.path()).unwrap();

    assert!(!report.clean);
    assert!(report.total_findings >= 2);
    assert_eq!(report.scanned_files, 1);
}

#[test]
fn scrubs_emails_with_multibyte_utf8_prefix_and_punctuation() {
    let sanitizer = Sanitizer::with_default_config();
    // \u{a0} is non-breaking space (2 bytes in UTF-8), \u{2014} is em dash (3 bytes)
    let text = "Prefix\u{a0}user@example.corp.com\u{2014}and\u{a0}another.person@domain.org.";
    let scrubbed = sanitizer.scrub_str(text);
    assert_eq!(
        scrubbed,
        "Prefix\u{a0}[REDACTED_EMAIL]\u{2014}and\u{a0}[REDACTED_EMAIL]."
    );
}

#[test]
fn scrubs_myriatron_and_qianchuan() {
    let sanitizer = Sanitizer::with_default_config();
    let text = "Running myriatron training job with qianchuan checkpoints.";
    let scrubbed = sanitizer.scrub_str(text);
    assert_eq!(
        scrubbed,
        "Running [REDACTED_ORG] training job with [REDACTED_ORG] checkpoints."
    );
}

#[test]
fn ignores_loopback_and_zero_ip() {
    let sanitizer = Sanitizer::with_default_config();
    let text = "Binding to 127.0.0.1:8080 and 0.0.0.0:4000.";
    let scrubbed = sanitizer.scrub_str(text);
    assert_eq!(text, scrubbed);
}
