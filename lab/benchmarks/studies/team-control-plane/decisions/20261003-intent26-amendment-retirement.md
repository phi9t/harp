# Ticket26: retire proven historical Episodes after an explicit amendment

Commit read: `8d249219534a3269819bbca11704200126afd1aa`.

Final repair3 remains the existing allocation in
[branch632's repair contract](20261003-intent26-branch632-final-repair.md).
Both review findings now have actual public HTTP/PostgreSQL REDs. Root read the
terminal logs: delegated Plan expansion after source amendment failed its
expected refusal (SHA256 `10a73043a602e85cff13735f070ae079e950c5267663eaab16f9934f1ae5de64`);
production amendment left AssignAttempt refused with dependency_unmet
(SHA256 `5686f1201b2fb6d287f8b238aaafc2f01d76d48ddecc05bd0cfa3387766fd543`).
The earlier credential-schema setup failure remains separately retained and
is not a behavioral RED.

An explicit new-work amendment must fence the old task generation immediately,
before assignment. Source inspection also establishes a consumer implication:
supervisor retirement currently requires Episode generation to equal the task's
current generation. A physically released old verifier can therefore remain
admitted and block its successor after amendment. This is a source-confirmed
consequence, not yet a verified retirement fix.

The permitted consumer repair is narrow. The current authenticated caller may
retire an older Episode only when a broker-authored immutable task.intent.amend
event binds that exact task and original generation, which is strictly below
the current generation. Preserve every existing principal, role, recovery epoch,
deadline, manifest/result/log, observed namespace shutdown, workspace release,
capacity release and unresolved-runtime-usage check. Retirement closes historical
state; it never refunds spend, transfers authority or revives the old generation.
Task-free lead jobs and nullable original Attempt provenance remain unchanged.

Require an actual public-seam RED for the resulting retirement blockage before
the consumer edit, then positive fresh-verification evidence and refusal before
expiry and with unresolved effects. Preserve initial v1 mapping as historical;
later explicit work must not reuse its old PASS as new acceptance. Fresh
Standards/Spec, root verification and branch review still determine conformance.
No acceptance, landing, extra repair iteration or runtime admission follows
from this decision.
