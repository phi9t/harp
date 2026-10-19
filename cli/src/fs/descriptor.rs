#[cfg(any(target_os = "macos", target_os = "linux"))]
#[path = "descriptor/secure.rs"]
mod secure;
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(crate) use secure::*;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
#[path = "descriptor/unsupported.rs"]
mod unsupported;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(crate) use unsupported::*;

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
#[path = "descriptor/unsupported.rs"]
mod unsupported_static;

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod cfg_tests {
    use std::ffi::CStr;
    use std::path::Path;

    use super::unsupported_static::{AnchoredDirectory, FilePolicy, ObjectIdentity};

    // Compile-only inventory of the descriptor surface publication uses. Keeping it
    // here makes the host fallback fail to compile when secure adds a publication
    // method without the typed unsupported counterpart.
    #[allow(dead_code)]
    fn publication_descriptor_inventory(directory: &AnchoredDirectory, name: &CStr) {
        let identity = ObjectIdentity {
            device: 0,
            inode: 0,
        };
        let _ = directory.duplicate("publication");
        let _ = directory.walk(Path::new("publication"), "publication");
        let _ = directory.walk_or_create(Path::new("publication"), 0o755, "publication");
        let _ = directory.open_optional_directory(name, "publication");
        let _ = directory.verify_owner_mode(0o755, "publication");
        let _ = directory.identity("publication");
        let _ = directory.verify_namespace("publication");
        let _ = directory.verify_entry_identity(name, identity, false, "publication");
        let _ = directory.entry_names_bounded("publication", 6);
        let _ = directory.read_optional_bounded(name, "publication", 64, FilePolicy::REGULAR);
        let _ = directory.read_optional_bounded_with_hook(
            name,
            "publication",
            64,
            FilePolicy::REGULAR,
            || {},
        );
        let created_file = directory.create_file_noreplace(name, b"ledger\n", 0o444, "publication");
        if let Ok(snapshot) = created_file {
            let _ = snapshot.matches_moved_object(&snapshot);
        }
        let staged = directory.create_staged_directory_with_cleanup_sync_hook(
            ".prepared-generation",
            0o755,
            "publication",
            |_| Ok(()),
        );
        if let Ok(mut staged) = staged {
            let _ = staged.directory();
            let _ = staged.create_file_noreplace_with_cleanup_sync_hook(
                name,
                b"ledger\n",
                0o444,
                "publication",
                |_| Ok(()),
            );
            let _ = directory.publish_staged_directory_noreplace_with_hook(
                &mut staged,
                name,
                "publication",
                || Ok(()),
            );
            staged.disarm();
            let _ = staged.cleanup_with_parent_sync("publication", |_| Ok(()));
        }
        if let Ok(Some(lock)) = directory.acquire_file_lock(name, true, true, "publication") {
            let _ = lock.verify("publication");
        }
        let _ = directory.sync("publication");
        let _ = AnchoredDirectory::cstring("current.json", "publication");
    }

    #[test]
    fn unsupported_descriptor_fallback_is_typed() {
        assert_eq!(
            AnchoredDirectory::open(Path::new("."), "unsupported descriptor")
                .expect_err("unsupported descriptor must fail closed")
                .code(),
            "fs.unsupported"
        );
    }
}
