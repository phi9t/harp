# Ticket01 repair round3/3

Base: ed85fe9ed7df6390d53584cbe3d0917b32593471. Prior scoped conformance and
21-test gate passes did not override independent whole-branch job634's ownership
finding. Exact verdict JSON/Markdown is retained byte-for-byte under
`../reviews/repair2/branch634.*`, with byte hashes in this source manifest.
This is the final normal repair round; another unresolved blocker requires the
root-owned architectural escalation, not an unbounded author patch loop.

The public RED test reproduced actual competing ownership: the first controller
opened a dangling database alias, and another controller could then acquire the
same database through either the alias or target (alias=true, target=true).
The correction is a six-line fail-closed check using `symlink_metadata` before
lock selection or target creation. A dangling database symlink is refused.

GREEN proves the refused alias creates no target, canonical target ownership
excludes both names, existing-alias ownership excludes the target, and ownership
is released normally. Compatibility coverage preserves hardlink refusal through
both names. A real-daemon test confirms competing processes through target and
existing alias both fail ownership, then the service restarts through the alias
and reads its durable revision. Logs record owned PIDs and observed dispositions.

Final validation:24 tests (20 public +4 real daemon/process), formatting and
clippy all-targets/all-features with warnings denied pass. All commands are
terminal; exact source hashes and RED/GREEN logs are retained here. Root owns
fresh scoped conformance and exact-candidate branch review. Ticket remains claimed.

No unrelated audit, schema/API/role change, backend change, provider, downstream
adapter, live trial, landing, push or root-checkout mutation. Earlier candidates
and source/evidence remain unchanged in history.01 qualification limits remain:
execution identity validation is02, actual Git object retention is04, and restore/
rollback plus power-loss qualification are unsupported. Directory/path metadata
is controller-owned outside worker mounts; hostile host-level path replacement
is outside this local baseline ownership contract.
