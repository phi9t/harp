#[cfg(any(target_os = "macos", target_os = "linux"))]
#[path = "fs/secure.rs"]
mod secure;
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(crate) use secure::*;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
#[path = "fs/unsupported.rs"]
mod unsupported;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(crate) use unsupported::*;

#[path = "fs/descriptor.rs"]
pub(crate) mod descriptor;

// Keep the fallback type-checkable on secure hosts even when no unsupported
// target standard library is installed locally.
#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
#[path = "fs/unsupported.rs"]
mod unsupported_static;

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod cfg_tests {
    use std::path::Path;

    use super::unsupported_static::{FileTreeLimits, HeldDirectory, PublicMutationEvent};

    #[allow(dead_code)]
    fn publication_rollback_inventory(directory: &HeldDirectory, snapshot: &super::FileSnapshot) {
        let _ = directory.bounded_regular_single_link_file_snapshots_with_extension(
            Path::new("packet"),
            "md",
            "packet inventory",
            64,
            FileTreeLimits {
                max_depth: 2,
                max_entries: 4,
                max_matching_files: 2,
                max_aggregate_bytes: 64,
            },
        );
        let _ = directory.compare_and_remove_public_regular_file_with_events(
            Path::new("current.json"),
            snapshot,
            "publication rollback",
            64,
            |_event: PublicMutationEvent| Ok(()),
        );
    }

    #[test]
    fn unsupported_filesystem_fallback_is_typed() {
        assert_eq!(
            HeldDirectory::open(Path::new("."), "unsupported filesystem")
                .expect_err("unsupported filesystem must fail closed")
                .code(),
            "fs.unsupported"
        );
    }
}
