# Redact evidence before persistence

Recorded: 2026-10-06

Independent critique found that a prohibition on secrets did not define a safe
collection mechanism. The design now requires every source adapter to expose a
closed field allowlist and to apply versioned streaming redaction before any
evidence bytes reach storage. Process, tmux and provider adapters return typed
identities and bounded sanitized fields, not raw environments, unrestricted
argv, transcripts or logs. Credential projections are references only.

Streaming rules carry overlap across chunks. A complete-record scanner runs
after assembly. Any remaining match fails closed and writes only a typed failure
without matched bytes. Evidence directories use mode 0700 and files mode 0600.
Negative tests cover fake secrets in structured fields, env, argv, terminal and
provider text, logs, chunk boundaries and truncation boundaries.
