use std::ffi::{CString, OsStr};
use std::fs::File;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path};
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use sha2::{Digest, Sha256};

use crate::{StateError, StateResult};

const MIGRATION_V1: &str = include_str!("../migrations/0001_init.sql");
const MIGRATION_V2: &str = include_str!("../migrations/0002_execution_authority.sql");
const MIGRATION_V3: &str = include_str!("../migrations/0003_scratch_wall_lease.sql");
const MIGRATION_V4: &str = include_str!("../migrations/0004_failure_clock.sql");
const MIGRATION_V5: &str = include_str!("../migrations/0005_cli_activities.sql");
const MIGRATION_V6: &str = include_str!("../migrations/0006_workflow_jobs.sql");
const WORKFLOW_SCHEMA_VERSION: i64 = 6;
const WORKFLOW_OBJECTS: &[(&str, &str)] = &[
    ("table", "workflow_admissions"),
    ("table", "workflow_jobs"),
    ("table", "workflow_environments"),
    ("table", "workflow_decisions"),
    ("table", "workflow_events"),
    ("index", "workflow_events_run_idx"),
];
const BASE_SCHEMA_VERSION: i64 = 4;
const SCHEMA_VERSION: i64 = 5;
const CLI_EXTENSION_OBJECTS: &[(&str, &str)] = &[
    ("index", "cli_activities_attempt_idx"),
    ("index", "cli_activities_recovery_idx"),
    ("index", "cli_attempts_external_session_idx"),
    ("index", "cli_attempts_nonterminal_idx"),
    ("table", "cli_activities"),
    ("table", "cli_attempts"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Identity {
    device: u64,
    inode: u64,
}

struct PreparedPath {
    parent: OwnedFd,
    parent_identity: Identity,
    file_identity: Identity,
}

pub(crate) fn open(path: &Path) -> StateResult<Connection> {
    open_with_hook(path, || Ok(()))
}

pub(crate) fn open_with_hook<F>(path: &Path, hook: F) -> StateResult<Connection>
where
    F: FnOnce() -> StateResult<()>,
{
    let prepared = prepare_path(path, true)?;
    hook()?;

    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX
        | OpenFlags::SQLITE_OPEN_NOFOLLOW;
    let connection = Connection::open_with_flags(path, flags)
        .map_err(|source| StateError::sqlite("open database", source))?;
    verify_has_not_moved(&connection)?;
    verify_path_identity(path, &prepared)?;
    configure(&connection)?;
    migrate(&connection)?;
    verify_has_not_moved(&connection)?;
    verify_path_identity(path, &prepared)?;
    verify_database_integrity(&connection)?;
    verify_has_not_moved(&connection)?;
    verify_path_identity(path, &prepared)?;
    Ok(connection)
}

/// Open existing state without creation, migration, or durability pragma writes.
pub(crate) fn open_read_only(path: &Path) -> StateResult<Connection> {
    let prepared = prepare_path(path, false)?;
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(|source| StateError::sqlite("open read-only database", source))?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|source| StateError::sqlite("set inspection busy timeout", source))?;
    verify_has_not_moved(&connection)?;
    verify_path_identity(path, &prepared)?;
    verify_database_integrity(&connection)?;
    verify_has_not_moved(&connection)?;
    verify_path_identity(path, &prepared)?;
    Ok(connection)
}

fn prepare_path(path: &Path, create: bool) -> StateResult<PreparedPath> {
    if path.file_name().is_none() {
        return Err(StateError::invalid("database path must name a file"));
    }
    let parent_path = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = open_directory_path(parent_path)?;
    let parent_stat = stat_fd(parent.as_raw_fd())
        .map_err(|source| StateError::io("inspect database parent", parent_path, source))?;
    validate_parent(&parent_stat)?;
    let parent_entry = stat_path(parent_path)
        .map_err(|source| StateError::io("inspect database parent path", parent_path, source))?;
    let parent_identity = identity(&parent_stat);
    if identity(&parent_entry) != parent_identity {
        return Err(StateError::invalid(
            "database parent identity changed while opening",
        ));
    }

    let file_name = c_string(
        path.file_name().expect("checked above"),
        "database filename",
    )?;
    let file_stat = match stat_at(parent.as_raw_fd(), &file_name) {
        Ok(stat) => {
            validate_file(&stat)?;
            stat
        }
        Err(error) if create && error.kind() == io::ErrorKind::NotFound => {
            let raw = retry_fd(|| unsafe {
                libc::openat(
                    parent.as_raw_fd(),
                    file_name.as_ptr(),
                    libc::O_RDWR
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC,
                    0o600,
                )
            })
            .map_err(|source| StateError::io("create database", path, source))?;
            let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
            let stat = stat_fd(descriptor.as_raw_fd())
                .map_err(|source| StateError::io("inspect created database", path, source))?;
            validate_file(&stat)?;
            File::from(descriptor)
                .sync_all()
                .map_err(|source| StateError::io("sync created database", path, source))?;
            fsync_fd(parent.as_raw_fd())
                .map_err(|source| StateError::io("sync database parent", parent_path, source))?;
            stat
        }
        Err(source) => return Err(StateError::io("inspect database", path, source)),
    };
    let file_entry = stat_at(parent.as_raw_fd(), &file_name)
        .map_err(|source| StateError::io("reinspect database", path, source))?;
    if identity(&file_stat) != identity(&file_entry) {
        return Err(StateError::invalid(
            "database file identity changed while preparing",
        ));
    }

    Ok(PreparedPath {
        parent,
        parent_identity,
        file_identity: identity(&file_stat),
    })
}

fn open_directory_path(path: &Path) -> StateResult<OwnedFd> {
    let (start, components): (&Path, Vec<&OsStr>) = if path.is_absolute() {
        (
            Path::new("/"),
            path.components()
                .filter_map(|component| match component {
                    Component::RootDir => None,
                    Component::Normal(value) => Some(Ok(value)),
                    _ => Some(Err(StateError::invalid(
                        "database parent path contains a non-normal component",
                    ))),
                })
                .collect::<StateResult<Vec<_>>>()?,
        )
    } else {
        (
            Path::new("."),
            path.components()
                .filter_map(|component| match component {
                    Component::CurDir => None,
                    Component::Normal(value) => Some(Ok(value)),
                    _ => Some(Err(StateError::invalid(
                        "database parent path contains a non-normal component",
                    ))),
                })
                .collect::<StateResult<Vec<_>>>()?,
        )
    };
    let start_name = c_string(start.as_os_str(), "database parent root")?;
    let start_raw = retry_fd(|| unsafe {
        libc::open(
            start_name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    })
    .map_err(|source| StateError::io("open database parent root", start, source))?;
    let mut directory = unsafe { OwnedFd::from_raw_fd(start_raw) };
    for component in components {
        let component_name = c_string(component, "database parent component")?;
        let raw = retry_fd(|| unsafe {
            libc::openat(
                directory.as_raw_fd(),
                component_name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        })
        .map_err(|source| match source.raw_os_error() {
            Some(libc::ELOOP | libc::ENOTDIR | libc::EACCES | libc::EPERM) => {
                StateError::InvalidInput {
                    context: format!(
                        "database parent contains a symlink, inaccessible component, or non-directory: {}",
                        path.display()
                    ),
                    source: None,
                }
            }
            _ => StateError::io("open database parent component", path, source),
        })?;
        directory = unsafe { OwnedFd::from_raw_fd(raw) };
    }
    Ok(directory)
}

fn configure(connection: &Connection) -> StateResult<()> {
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|source| StateError::sqlite("set busy timeout", source))?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;\n\
             PRAGMA journal_mode = DELETE;\n\
             PRAGMA synchronous = FULL;",
        )
        .map_err(|source| StateError::sqlite("configure database", source))?;
    let foreign_keys: i64 = connection
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .map_err(|source| StateError::sqlite("verify foreign keys", source))?;
    let journal_mode: String = connection
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .map_err(|source| StateError::sqlite("verify journal mode", source))?;
    let synchronous: i64 = connection
        .pragma_query_value(None, "synchronous", |row| row.get(0))
        .map_err(|source| StateError::sqlite("verify synchronous mode", source))?;
    if foreign_keys != 1 || journal_mode != "delete" || synchronous != 2 {
        return Err(StateError::integrity(
            "SQLite refused required durability pragmas",
        ));
    }
    Ok(())
}

fn migrate(connection: &Connection) -> StateResult<()> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(|e| StateError::sqlite("read workflow schema version", e))?;
    if version == WORKFLOW_SCHEMA_VERSION {
        return Ok(());
    }
    if version != SCHEMA_VERSION {
        migrate_legacy(connection)?;
    }
    let transaction =
        rusqlite::Transaction::new_unchecked(connection, rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| StateError::sqlite("begin workflow migration", e))?;
    let version: i64 = transaction
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(|e| StateError::sqlite("recheck workflow schema version", e))?;
    if version == WORKFLOW_SCHEMA_VERSION {
        return transaction
            .commit()
            .map_err(|e| StateError::sqlite("finish concurrent migration", e));
    }
    verify_legacy_integrity(&transaction, SCHEMA_VERSION)?;
    transaction
        .execute_batch(MIGRATION_V6)
        .map_err(|e| StateError::sqlite("migrate workflow schema", e))?;
    transaction
        .pragma_update(None, "user_version", WORKFLOW_SCHEMA_VERSION)
        .map_err(|e| StateError::sqlite("record workflow schema version", e))?;
    transaction
        .commit()
        .map_err(|e| StateError::sqlite("commit workflow migration", e))
}

fn migrate_legacy(connection: &Connection) -> StateResult<()> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|source| StateError::sqlite("read schema version", source))?;
    match version {
        SCHEMA_VERSION => Ok(()),
        0 => {
            let expected_fingerprint = expected_schema_fingerprint(BASE_SCHEMA_VERSION)?;
            connection
                .execute_batch(&format!(
                    "BEGIN IMMEDIATE;\n\
                     {MIGRATION_V1}\n\
                     {MIGRATION_V2}\n\
                     {MIGRATION_V3}\n\
                     {MIGRATION_V4}\n\
                     {MIGRATION_V5}\n\
                     INSERT INTO state_metadata(key, value)\n\
                     VALUES ('schema_fingerprint', '{expected_fingerprint}');\n\
                     PRAGMA user_version = {SCHEMA_VERSION};\n\
                     COMMIT;"
                ))
                .map_err(|source| StateError::sqlite("apply schema migration", source))?;
            Ok(())
        }
        1 => {
            verify_version_fingerprint(connection, 1)?;
            let expected_fingerprint = expected_schema_fingerprint(BASE_SCHEMA_VERSION)?;
            connection
                .execute_batch(&format!(
                    "BEGIN IMMEDIATE;\n\
                     {MIGRATION_V2}\n\
                     {MIGRATION_V3}\n\
                     {MIGRATION_V4}\n\
                     {MIGRATION_V5}\n\
                     UPDATE state_metadata\n\
                     SET value = '{expected_fingerprint}'\n\
                     WHERE key = 'schema_fingerprint';\n\
                     PRAGMA user_version = {SCHEMA_VERSION};\n\
                     COMMIT;"
                ))
                .map_err(|source| StateError::sqlite("upgrade schema from v1 to v5", source))?;
            Ok(())
        }
        2 => {
            verify_version_fingerprint(connection, 2)?;
            let expected_fingerprint = expected_schema_fingerprint(BASE_SCHEMA_VERSION)?;
            connection
                .execute_batch(&format!(
                    "BEGIN IMMEDIATE;\n\
                     {MIGRATION_V3}\n\
                     {MIGRATION_V4}\n\
                     {MIGRATION_V5}\n\
                     UPDATE state_metadata\n\
                     SET value = '{expected_fingerprint}'\n\
                     WHERE key = 'schema_fingerprint';\n\
                     PRAGMA user_version = {SCHEMA_VERSION};\n\
                     COMMIT;"
                ))
                .map_err(|source| StateError::sqlite("upgrade schema from v2 to v5", source))?;
            Ok(())
        }
        3 => {
            verify_version_fingerprint(connection, 3)?;
            let expected_fingerprint = expected_schema_fingerprint(BASE_SCHEMA_VERSION)?;
            connection
                .execute_batch(&format!(
                    "BEGIN IMMEDIATE;\n\
                     {MIGRATION_V4}\n\
                     {MIGRATION_V5}\n\
                     UPDATE state_metadata\n\
                     SET value = '{expected_fingerprint}'\n\
                     WHERE key = 'schema_fingerprint';\n\
                     PRAGMA user_version = {SCHEMA_VERSION};\n\
                     COMMIT;"
                ))
                .map_err(|source| StateError::sqlite("upgrade schema from v3 to v5", source))
        }
        4 => {
            verify_version_fingerprint(connection, BASE_SCHEMA_VERSION)?;
            connection
                .execute_batch(&format!(
                    "BEGIN IMMEDIATE;\n\
                     {MIGRATION_V5}\n\
                     PRAGMA user_version = {SCHEMA_VERSION};\n\
                     COMMIT;"
                ))
                .map_err(|source| StateError::sqlite("apply schema migration v5", source))
        }
        newer if newer > SCHEMA_VERSION => Err(StateError::Unsupported {
            context: format!(
                "database schema version {newer} is newer than supported version {SCHEMA_VERSION}"
            ),
        }),
        other => Err(StateError::Unsupported {
            context: format!("unsupported database schema version {other}"),
        }),
    }
}

fn expected_schema_fingerprint(version: i64) -> StateResult<String> {
    let connection = Connection::open_in_memory()
        .map_err(|source| StateError::sqlite("open expected schema database", source))?;
    let migrations = match version {
        1 => MIGRATION_V1.to_owned(),
        2 => format!("{MIGRATION_V1}\n{MIGRATION_V2}"),
        3 => format!("{MIGRATION_V1}\n{MIGRATION_V2}\n{MIGRATION_V3}"),
        4 => format!("{MIGRATION_V1}\n{MIGRATION_V2}\n{MIGRATION_V3}\n{MIGRATION_V4}"),
        _ => {
            return Err(StateError::Unsupported {
                context: format!("unsupported expected schema version {version}"),
            });
        }
    };
    connection
        .execute_batch(&format!("{migrations}\nPRAGMA user_version = {version};"))
        .map_err(|source| StateError::sqlite("apply expected schema migration", source))?;
    schema_fingerprint(&connection)
}

fn verify_version_fingerprint(connection: &Connection, version: i64) -> StateResult<()> {
    let expected = expected_schema_fingerprint(version)?;
    let actual = schema_fingerprint(connection)?;
    let stored: Option<String> = connection
        .query_row(
            "SELECT value FROM state_metadata WHERE key = 'schema_fingerprint'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|source| StateError::sqlite("read source schema fingerprint", source))?;
    if actual != expected || stored.as_deref() != Some(expected.as_str()) {
        return Err(StateError::integrity(format!(
            "schema version {version} fingerprint mismatch"
        )));
    }
    Ok(())
}

fn schema_fingerprint(connection: &Connection) -> StateResult<String> {
    filtered_schema_fingerprint(connection, |_, _| true)
}

fn base_schema_fingerprint(connection: &Connection) -> StateResult<String> {
    filtered_schema_fingerprint(connection, |object_type, name| {
        !is_cli_extension_object(object_type, name)
            && !WORKFLOW_OBJECTS.contains(&(object_type, name))
    })
}

fn cli_extension_schema_fingerprint(connection: &Connection) -> StateResult<String> {
    filtered_schema_fingerprint(connection, is_cli_extension_object)
}

fn is_cli_extension_object(object_type: &str, name: &str) -> bool {
    CLI_EXTENSION_OBJECTS.contains(&(object_type, name))
}

fn filtered_schema_fingerprint<F>(connection: &Connection, mut include: F) -> StateResult<String>
where
    F: FnMut(&str, &str) -> bool,
{
    let mut statement = connection
        .prepare(
            "SELECT type, name, COALESCE(sql, '')
             FROM sqlite_master
             WHERE type IN ('table', 'index', 'trigger', 'view')
               AND name NOT LIKE 'sqlite_%'
             ORDER BY type, name",
        )
        .map_err(|source| StateError::sqlite("prepare schema fingerprint", source))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|source| StateError::sqlite("query schema fingerprint", source))?;
    let mut digest = Sha256::new();
    let mut count = 0usize;
    for row in rows {
        let (object_type, name, sql) =
            row.map_err(|source| StateError::sqlite("read schema fingerprint", source))?;
        if !include(&object_type, &name) {
            continue;
        }
        for value in [object_type.as_bytes(), name.as_bytes(), sql.as_bytes()] {
            digest.update(
                u64::try_from(value.len())
                    .expect("schema string length fits u64")
                    .to_le_bytes(),
            );
            digest.update(value);
        }
        count += 1;
        if count > 10_000 {
            return Err(StateError::LimitExceeded {
                context: "schema object count".to_owned(),
                limit: 10_000,
                actual: count as u64,
            });
        }
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn expected_cli_extension_schema_fingerprint() -> StateResult<String> {
    let connection = Connection::open_in_memory()
        .map_err(|source| StateError::sqlite("open expected extension database", source))?;
    connection
        .execute_batch(&format!(
            "{MIGRATION_V1}\n\
             {MIGRATION_V2}\n\
             {MIGRATION_V3}\n\
             {MIGRATION_V4}\n\
             {MIGRATION_V5}"
        ))
        .map_err(|source| StateError::sqlite("apply expected extension migration", source))?;
    cli_extension_schema_fingerprint(&connection)
}

fn verify_database_integrity(connection: &Connection) -> StateResult<()> {
    verify_legacy_integrity(connection, WORKFLOW_SCHEMA_VERSION)?;
    let expected = Connection::open_in_memory()
        .map_err(|e| StateError::sqlite("open workflow schema template", e))?;
    expected
        .execute_batch(MIGRATION_V6)
        .map_err(|e| StateError::sqlite("prepare workflow schema template", e))?;
    let select = |kind: &str, name: &str| WORKFLOW_OBJECTS.contains(&(kind, name));
    if filtered_schema_fingerprint(connection, select)?
        != filtered_schema_fingerprint(&expected, select)?
    {
        return Err(StateError::integrity(
            "workflow schema fingerprint mismatch",
        ));
    }
    Ok(())
}

fn verify_legacy_integrity(connection: &Connection, expected_version: i64) -> StateResult<()> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|source| StateError::sqlite("verify schema version", source))?;
    if version != expected_version {
        return Err(StateError::integrity(format!(
            "database schema version {version} does not equal {expected_version}"
        )));
    }

    let mut quick_check = connection
        .prepare("PRAGMA quick_check(1)")
        .map_err(|source| StateError::sqlite("prepare quick check", source))?;
    let rows = quick_check
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|source| StateError::sqlite("query quick check", source))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|source| StateError::sqlite("read quick check", source))?);
        if results.len() > 2 {
            return Err(StateError::integrity(
                "SQLite quick_check returned too many rows",
            ));
        }
    }
    if results != ["ok"] {
        return Err(StateError::integrity(format!(
            "SQLite quick_check failed: {}",
            results.join("; ")
        )));
    }

    let mut foreign_keys = connection
        .prepare("PRAGMA foreign_key_check")
        .map_err(|source| StateError::sqlite("prepare foreign key check", source))?;
    let mut rows = foreign_keys
        .query([])
        .map_err(|source| StateError::sqlite("query foreign key check", source))?;
    if rows
        .next()
        .map_err(|source| StateError::sqlite("read foreign key check", source))?
        .is_some()
    {
        return Err(StateError::integrity(
            "database contains foreign key violations",
        ));
    }

    let expected_base = expected_schema_fingerprint(BASE_SCHEMA_VERSION)?;
    let actual_base = base_schema_fingerprint(connection)?;
    if actual_base != expected_base {
        return Err(StateError::integrity(format!(
            "database base schema fingerprint mismatch: expected {expected_base}, got {actual_base}"
        )));
    }
    let stored: Option<String> = connection
        .query_row(
            "SELECT value FROM state_metadata WHERE key = 'schema_fingerprint'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|source| StateError::sqlite("read stored schema fingerprint", source))?;
    if stored.as_deref() != Some(expected_base.as_str()) {
        return Err(StateError::integrity(
            "stored base schema fingerprint is missing or mismatched",
        ));
    }
    let expected_extension = expected_cli_extension_schema_fingerprint()?;
    let actual_extension = cli_extension_schema_fingerprint(connection)?;
    if actual_extension != expected_extension {
        return Err(StateError::integrity(format!(
            "database CLI extension schema fingerprint mismatch: expected {expected_extension}, got {actual_extension}"
        )));
    }
    Ok(())
}

fn verify_path_identity(path: &Path, prepared: &PreparedPath) -> StateResult<()> {
    let parent_path = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = stat_path(parent_path)
        .map_err(|source| StateError::io("verify database parent", parent_path, source))?;
    if identity(&parent) != prepared.parent_identity
        || identity(&stat_fd(prepared.parent.as_raw_fd()).map_err(|source| {
            StateError::io("verify opened database parent", parent_path, source)
        })?) != prepared.parent_identity
    {
        return Err(StateError::invalid(
            "database parent identity changed across SQLite open",
        ));
    }
    let file_name = c_string(
        path.file_name().expect("prepared path"),
        "database filename",
    )?;
    let file = stat_at(prepared.parent.as_raw_fd(), &file_name)
        .map_err(|source| StateError::io("verify database file", path, source))?;
    validate_file(&file)?;
    if identity(&file) != prepared.file_identity {
        return Err(StateError::invalid(
            "database file identity changed across SQLite open",
        ));
    }
    Ok(())
}

fn verify_has_not_moved(connection: &Connection) -> StateResult<()> {
    let mut moved: libc::c_int = 0;
    let result = unsafe {
        rusqlite::ffi::sqlite3_file_control(
            connection.handle(),
            c"main".as_ptr(),
            rusqlite::ffi::SQLITE_FCNTL_HAS_MOVED,
            (&mut moved as *mut libc::c_int).cast(),
        )
    };
    if result != rusqlite::ffi::SQLITE_OK {
        return Err(StateError::Integrity {
            context: format!("SQLITE_FCNTL_HAS_MOVED failed with code {result}"),
            source: None,
        });
    }
    if moved != 0 {
        return Err(StateError::invalid(
            "database pathname no longer identifies the opened SQLite file",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct FileStat {
    device: u64,
    inode: u64,
    uid: u32,
    mode: libc::mode_t,
}

fn validate_parent(stat: &FileStat) -> StateResult<()> {
    if stat.mode & libc::S_IFMT != libc::S_IFDIR {
        return Err(StateError::invalid(
            "database parent must be a real directory",
        ));
    }
    if stat.uid != unsafe { libc::geteuid() } {
        return Err(StateError::invalid(
            "database parent must be owned by the current user",
        ));
    }
    if stat.mode & 0o777 != 0o700 {
        return Err(StateError::invalid(
            "database parent permissions must be exactly 0700",
        ));
    }
    Ok(())
}

fn validate_file(stat: &FileStat) -> StateResult<()> {
    if stat.mode & libc::S_IFMT != libc::S_IFREG {
        return Err(StateError::invalid(
            "database path must be a regular file, not a symlink",
        ));
    }
    if stat.uid != unsafe { libc::geteuid() } {
        return Err(StateError::invalid(
            "database file must be owned by the current user",
        ));
    }
    if stat.mode & 0o777 != 0o600 {
        return Err(StateError::invalid(
            "database file permissions must be exactly 0600",
        ));
    }
    Ok(())
}

fn stat_fd(fd: libc::c_int) -> io::Result<FileStat> {
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    retry_zero(|| unsafe { libc::fstat(fd, stat.as_mut_ptr()) })?;
    Ok(convert_stat(unsafe { stat.assume_init() }))
}

fn stat_at(parent: libc::c_int, name: &CString) -> io::Result<FileStat> {
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    retry_zero(|| unsafe {
        libc::fstatat(
            parent,
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    })?;
    Ok(convert_stat(unsafe { stat.assume_init() }))
}

fn stat_path(path: &Path) -> io::Result<FileStat> {
    let name = c_string_io(path.as_os_str())?;
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    retry_zero(|| unsafe { libc::lstat(name.as_ptr(), stat.as_mut_ptr()) })?;
    Ok(convert_stat(unsafe { stat.assume_init() }))
}

#[allow(clippy::unnecessary_cast)]
fn convert_stat(stat: libc::stat) -> FileStat {
    FileStat {
        device: stat.st_dev as u64,
        inode: stat.st_ino as u64,
        uid: stat.st_uid,
        mode: stat.st_mode,
    }
}

fn identity(stat: &FileStat) -> Identity {
    Identity {
        device: stat.device,
        inode: stat.inode,
    }
}

fn c_string(value: &OsStr, context: &str) -> StateResult<CString> {
    c_string_io(value).map_err(|source| StateError::InvalidInput {
        context: format!("{context} contains NUL: {source}"),
        source: None,
    })
}

fn c_string_io(value: &OsStr) -> io::Result<CString> {
    CString::new(value.as_bytes())
        .map_err(|source| io::Error::new(io::ErrorKind::InvalidInput, source))
}

fn fsync_fd(fd: libc::c_int) -> io::Result<()> {
    retry_zero(|| unsafe { libc::fsync(fd) })
}

fn retry_fd(mut operation: impl FnMut() -> libc::c_int) -> io::Result<libc::c_int> {
    loop {
        let result = operation();
        if result >= 0 {
            return Ok(result);
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

fn retry_zero(mut operation: impl FnMut() -> libc::c_int) -> io::Result<()> {
    loop {
        if operation() == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}
