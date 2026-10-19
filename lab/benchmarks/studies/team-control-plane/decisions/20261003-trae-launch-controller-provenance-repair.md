# Freeze future launch control before admission

Commit read: `f1ec0a7a980fccc9b6dbdd2d7e0e50caa1f78bbc`.
This is offline preparation under the owner's full-goal authority, following
the partial result of the first fixed cleanup case. It does not repair or
upgrade that historical operation's provenance and grants no physical launch.

Implement an inspectable controller for future concrete cases in the existing
sole-writer Trae20 preparation tree. Its executable source and immutable case,
input hashes, command, operation and limits must be frozen before independent
review and admission. A changed source/case requires a new concrete grant.
The controller verifies exact admitted inputs, exclusively creates its
operation's started record, checks writes and fsyncs the record and parent
directory before the executor boundary. Existing or uncertain records refuse
execution without a retry or replacement. Terminal output and uncertainty are
retained without overwriting prior evidence. It must have no default launch
path or authority to widen an envelope.

Use public offline red-green tests at the executor boundary, with an injected
recording executor and temporary owner-controlled files. Cover duplicate
operation and concurrent admission, changed command/source/case, failed write
or fsync, and terminal/exception retention. Such tests qualify only controller
behavior, not an actual runtime, PID namespace or provider. The suite must not
start Trae, a namespace, a service or a model, and must not exercise the earlier
physical test again. Preserve all frozen old inputs and outputs byte-for-byte.

The writer hands off exact source/preimages, reviewed command and case,
red-green logs, retention checks and remaining limitations for independent
conformance. Future physical admission remains a separate root decision after
review; all earlier allowances remain spent and quarantines remain unchanged.
Runtime configuration, nested isolation, native turn-ID binding, terminal
observation and request accounting remain separate unproven prerequisites.
