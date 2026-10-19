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
    let root = dir.path().canonicalize().unwrap();
    let file_path = root.join("leak.txt");
    fs::write(
        &file_path,
        "Found 10.116.77.207 and user at /Users/david/secret",
    )
    .unwrap();

    let sanitizer = Sanitizer::with_default_config();
    let report = sanitizer.scan_path(&root).unwrap();

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

#[test]
fn text_scrubbing_preserves_utf8_and_original_line_endings() {
    let sanitizer = Sanitizer::with_default_config();
    assert_eq!(
        sanitizer.scrub_str("数学\r\nPlain text\nTail"),
        "数学\r\nPlain text\nTail"
    );
    assert_eq!(
        sanitizer.scrub_str("联系 user@private.corp\r\nDone\r\n"),
        "联系 [REDACTED_EMAIL]\r\nDone\r\n"
    );
}

#[test]
fn file_scrubbing_counts_changes_and_reports_invalid_utf8_without_writing() {
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let file = root.join("note.txt");
    fs::write(&file, "联系 user@private.corp\r\n").unwrap();
    let sanitizer = Sanitizer::with_default_config();
    assert_eq!(sanitizer.scrub_path(&file).unwrap(), 1);
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "联系 [REDACTED_EMAIL]\r\n"
    );
    assert_eq!(sanitizer.scrub_path(&file).unwrap(), 0);
    fs::write(&file, [0xff, 0xfe]).unwrap();
    assert_eq!(
        sanitizer.scrub_file(&file, true).unwrap_err().code(),
        "sanitize.encoding"
    );
    assert_eq!(fs::read(&file).unwrap(), [0xff, 0xfe]);
}

#[cfg(unix)]
#[test]
fn scan_and_scrub_reject_explicit_symlinks_and_linked_ancestors() {
    use std::os::unix::fs::symlink;
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let real = root.join("real");
    fs::create_dir(&real).unwrap();
    let file = real.join("note.txt");
    let original = "user@private.corp";
    fs::write(&file, original).unwrap();
    symlink(&real, root.join("linked-dir")).unwrap();
    symlink(&file, root.join("linked-file.txt")).unwrap();
    let sanitizer = Sanitizer::with_default_config();
    for path in [
        root.join("linked-dir"),
        root.join("linked-file.txt"),
        root.join("linked-dir/note.txt"),
    ] {
        assert!(
            sanitizer.scan_path(&path).is_err(),
            "scan followed {}",
            path.display()
        );
        assert!(
            sanitizer.scrub_path(&path).is_err(),
            "scrub followed {}",
            path.display()
        );
    }
    assert_eq!(fs::read_to_string(file).unwrap(), original);
}

#[cfg(unix)]
#[test]
fn directory_scan_skips_links_and_scrub_refuses_them_before_any_write() {
    use std::os::unix::fs::symlink;
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let selected = root.join("selected");
    fs::create_dir(&selected).unwrap();
    let external = root.join("outside.txt");
    fs::write(&external, "outside@private.corp").unwrap();
    symlink(&external, selected.join("linked.txt")).unwrap();
    let sanitizer = Sanitizer::with_default_config();
    let report = sanitizer.scan_path(&selected).unwrap();
    assert!(report.clean);
    assert_eq!(report.scanned_files, 0);
    fs::write(selected.join("ordinary.txt"), "inside@private.corp").unwrap();
    assert!(sanitizer.scrub_path(&selected).is_err());
    assert_eq!(
        fs::read_to_string(&external).unwrap(),
        "outside@private.corp"
    );
    assert_eq!(
        fs::read_to_string(selected.join("ordinary.txt")).unwrap(),
        "inside@private.corp"
    );
}

#[cfg(unix)]
#[test]
fn atomic_scrub_preserves_executable_mode_and_refuses_hardlinked_mutation() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let file = root.join("script.sh");
    fs::write(&file, "user@private.corp").unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o750)).unwrap();
    let sanitizer = Sanitizer::with_default_config();
    sanitizer.scrub_path(&file).unwrap();
    assert_eq!(
        fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o750
    );
    fs::write(&file, "user@private.corp").unwrap();
    let alias = root.join("alias.sh");
    fs::hard_link(&file, &alias).unwrap();
    assert!(sanitizer.scrub_path(&file).is_err());
    assert_eq!(fs::read_to_string(alias).unwrap(), "user@private.corp");
}
