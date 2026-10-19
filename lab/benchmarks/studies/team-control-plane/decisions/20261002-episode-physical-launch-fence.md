# Preserve legacy launch semantics; enforce episode uncertainty at physical boundaries

Status: bounded review560 repair direction, not acceptance. Independent Spec architecture assessment is complete; no new stack or schema is approved.

The accepted legacy additional-capacity predicate remains in place. Generalizing it to every legacy execution would reject its own currently dispatching Start, and the existing CLI carries no independent launch-origin certificate. A caller-provided execution ID or owner bearer does not justify bypassing this fence. Do not introduce that weaker exemption.

Add episode uncertainty checks for every actual Attempt-bound physical execution, including legacy workspace bindings. Public start and final monitor use the same predicate. This closes the cited preallocated-verifier bypass without expanding the legacy contract.

Own execution mapping proves ownership, not effect correspondence. Never exempt own outcome_unknown, quarantined or confirmed unknown usage merely because IDs match. Legitimate registered native original launch occurs during dispatching; any narrowly allowed original launch phase must retain current authority and avoid borrowing external effects. If a safe origin/phase distinction cannot be established, report uncertainty rather than silently exempting it.

Required controls: preallocated verifier and legacy workspace vs sibling episode uncertainty at both launch boundaries; registered original launch positive; Ready own unregistered confirmedUnknown/lostreply negative; measured-zero legitimate subsequent work; reservations retained. New generalized legacy certificates are outside this repair unless independently justified and approved.
