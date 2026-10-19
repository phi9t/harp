# Build the Scout Kit before launching the Scout Lead

Recorded: 2026-10-06
Supersedes: `20261006-first-role-epoch-is-scout-only.md`

Independent critique found that an inventory-only Scout could not safely create
its own schemas, source manifest, redaction pipeline and bootstrap authority
tooling. The owner therefore approved RoleEpoch 0 as a narrow Bootstrap Lead on
the B200 host. It may implement and qualify only the Scout Kit in its isolated
worktree and cannot inspect or alter the controller portfolio. Independent
review and human-authorized landing fence RoleEpoch 0.

RoleEpoch 1 is the Scout Lead. Its two Observation Rounds write outside the Git
estate and are sealed before any stabilization record is copied into the
dedicated evidence worktree. Pre-I2 direction uses one expiring byte-identified
Bootstrap Mandate and append-only receipt journal. I2 later reconciles and closes
that journal before becoming the sole Director command authority.
