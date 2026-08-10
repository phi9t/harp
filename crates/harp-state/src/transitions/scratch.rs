use std::path::Path;

use super::*;

impl StateStore {
    pub fn bind_attempt_scratch(
        &mut self,
        lease: &LeaseToken,
        canonical_path: &Path,
        device: u64,
        inode: u64,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let path = canonical_path
            .to_str()
            .ok_or_else(|| StateError::invalid("attempt scratch path must be valid UTF-8"))?;
        if !canonical_path.is_absolute()
            || path.is_empty()
            || path.len() > 4096
            || path.chars().any(char::is_control)
        {
            return Err(StateError::invalid(
                "attempt scratch path must be a bounded canonical absolute path",
            ));
        }
        let device = checked_u64_to_i64("attempt scratch device", device)?;
        let inode = checked_u64_to_i64("attempt scratch inode", inode)?;
        let transaction = self.immediate("begin attempt scratch binding")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        match (
            attempt.scratch_path.as_deref(),
            attempt.scratch_device,
            attempt.scratch_inode,
        ) {
            (Some(current_path), Some(current_device), Some(current_inode))
                if current_path == path
                    && current_device == u64::try_from(device).expect("nonnegative")
                    && current_inode == u64::try_from(inode).expect("nonnegative") =>
            {
                transaction.commit().map_err(|source| {
                    StateError::sqlite("commit idempotent scratch binding", source)
                })?;
                return Ok(());
            }
            (None, None, None) => {}
            _ => {
                return Err(StateError::Conflict {
                    entity: format!("attempt {} scratch identity", attempt.attempt_id),
                    expected: format!("{path}@{device}:{inode}"),
                    actual: format!(
                        "{:?}@{:?}:{:?}",
                        attempt.scratch_path, attempt.scratch_device, attempt.scratch_inode
                    ),
                });
            }
        }
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET scratch_path = ?2,
                     scratch_device = ?3,
                     scratch_inode = ?4,
                     updated_at = ?5
                 WHERE attempt_id = ?1
                   AND scratch_path IS NULL
                   AND scratch_device IS NULL
                   AND scratch_inode IS NULL",
                params![attempt.attempt_id.to_string(), path, device, inode, now,],
            )
            .map_err(|source| StateError::sqlite("bind attempt scratch", source))?;
        require_changed(
            changed,
            format!("attempt {} scratch identity", attempt.attempt_id),
            "unbound",
            "changed before scratch binding CAS",
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "attempt_scratch_bound",
            &json!({"path": path, "device": device, "inode": inode}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit attempt scratch binding", source))
    }
}
