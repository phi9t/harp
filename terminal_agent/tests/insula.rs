use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::Path;

use corenius::insula::InsulaSandbox;
use tempfile::tempdir;

fn marker(root: &Path, owner: &str, digest: &str) {
    fs::write(
        root.join(".corenius-insula-rootfs.json"),
        format!(
            r#"{{"schema_version":1,"owner":"{owner}","digest":"{digest}","node_path":"/usr/bin/node"}}"#
        ),
    )
    .unwrap();
}

#[test]
fn validates_selection_and_rejects_bad_rootfs() {
    let temp = tempdir().unwrap();
    let store = temp.path().join("store");
    let valid = store.join("valid");
    fs::create_dir_all(&valid).unwrap();
    marker(&valid, "corenius-insula", "abc");

    let selected = InsulaSandbox::validate_selection(&valid, &store, temp.path(), None).unwrap();
    assert_eq!(selected.marker.digest, "abc");
    assert!(InsulaSandbox::validate_selection(&valid, &store, temp.path(), Some("def")).is_err());

    let outside = temp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    marker(&outside, "corenius-insula", "abc");
    assert!(InsulaSandbox::validate_selection(&outside, &store, temp.path(), None).is_err());

    let wrong = store.join("wrong");
    fs::create_dir_all(&wrong).unwrap();
    marker(&wrong, "other", "abc");
    assert!(InsulaSandbox::validate_selection(&wrong, &store, temp.path(), None).is_err());

    let malformed = store.join("malformed");
    fs::create_dir_all(&malformed).unwrap();
    fs::write(malformed.join(".corenius-insula-rootfs.json"), "{").unwrap();
    assert!(InsulaSandbox::validate_selection(&malformed, &store, temp.path(), None).is_err());

    #[cfg(unix)]
    {
        let link = store.join("link");
        symlink(&valid, &link).unwrap();
        assert!(InsulaSandbox::validate_selection(&link, &store, temp.path(), None).is_err());
    }
}

#[test]
fn selection_file_round_trips_and_enforces_digest() {
    let temp = tempdir().unwrap();
    let store = temp.path().join("store");
    let rootfs = store.join("rootfs");
    fs::create_dir_all(rootfs.join("usr/bin")).unwrap();
    fs::write(rootfs.join("usr/bin/node"), "").unwrap();
    marker(&rootfs, "corenius-insula", "abc");

    let selected = InsulaSandbox::write_selection(temp.path(), &rootfs, &store).unwrap();
    assert_eq!(selected.marker.digest, "abc");
    let loaded = InsulaSandbox::load_selection(temp.path()).unwrap();
    assert_eq!(loaded.selection.marker.digest, "abc");

    marker(&rootfs, "corenius-insula", "changed");
    assert!(InsulaSandbox::load_selection(temp.path()).is_err());
}

#[test]
fn bwrap_plan_is_deterministic_and_filters_environment() {
    let temp = tempdir().unwrap();
    let store = temp.path().join("store");
    let rootfs = store.join("rootfs");
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(&rootfs).unwrap();
    fs::create_dir_all(&workspace).unwrap();
    marker(&rootfs, "corenius-insula", "abc");
    let sandbox = InsulaSandbox::new(
        InsulaSandbox::validate_selection(&rootfs, &store, &workspace, None).unwrap(),
    );

    let plan = sandbox
        .build_command(
            "pwd",
            &workspace,
            &[("SECRET", "no"), ("TERM", "xterm")],
            1_000,
        )
        .unwrap();
    assert_eq!(
        &plan.args[0..6],
        [
            "--die-with-parent",
            "--unshare-pid",
            "--ro-bind",
            rootfs.to_str().unwrap(),
            "/",
            "--bind"
        ]
    );
    assert!(
        plan.args
            .ends_with(&["/bin/sh".into(), "-lc".into(), "pwd".into()])
    );
    assert_eq!(plan.env.get("SECRET"), None);
    assert_eq!(plan.env.get("TERM").map(String::as_str), Some("xterm"));
    assert_eq!(
        plan.env.get("CORENIUS_IN_INSULA").map(String::as_str),
        Some("1")
    );
}
