# Ticket01 repair round2/3

Base: 958ebdb04949aa06df1b282e93de7490023e7e35. Prior conformance passes and
18-test gate did not override the independent whole-branch rejection (job633).
Exact JSON/Markdown verdict data is retained at `../reviews/repair1/branch633.*`;
byte hashes are recorded in this round's source manifest. Root owns fresh scoped
conformance and exact-candidate branch review; no acceptance is claimed here.

Medium finding: admission/observation/cancellation event task identity now resolves
from the retained attempt returned by the command. Operation transitions resolve
through the retained operation and its same-project attempt. Execution events
never use caller task metadata. An operation without an attempt has no worker
task audience. Message participant guards remain unchanged; other events prefer
the authoritative result or authenticated task before caller metadata.

Public RED independently demonstrated missing own running observation and missing
own operation transition. GREEN tests prove reconnect visibility for running/
unknown attempt observations and applying/unknown operation transitions, hide
them from the other worker, and reject forged task routing to that other worker.
Cancellation's recorded event also keeps its actual task identity while revoking
the cancelled grant according to the existing authority contract.

Low finding: foreign-schema detection now uses the literal internal prefix via
`GLOB 'sqlite_*'`. The real on-disk foreign `sqlitecache` table is refused instead
of receiving controller schema, and its original data remains readable unchanged.
The public open test genuinely failed before correction and passes afterward.

Final verification:21 tests (18 public +3 real daemon/process) pass, including
all prior18 tests, plus formatting and clippy all-targets/all-features with
warnings denied. All commands reached terminal status. Exact source hashes and
RED/GREEN command logs are retained alongside `verification.txt`.

No schema/dependency/backend change, adapter, external provider, live trial,
landing, push or root-checkout mutation. Original source/evidence and earlier
repair records remain unchanged in history. Ticket remains claimed. Scope limits
from01/repair1 remain: supervised execution validation is02; real Git object
retention is04; restore/rollback and power-loss qualification remain unsupported.
