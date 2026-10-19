# Use strict YAML evidence and reviewed action proposals

Recorded: 2026-10-06

The owner selected strict YAML 1.2 as the storage syntax for pass-one evidence.
Records use a closed schema and the JSON-compatible YAML data model. Readers
reject duplicate keys, custom tags, anchors, aliases, merge keys, non-string map
keys and ambiguous extensions. Literal block scalars are available for multiline
commands, excerpts and diagnostics.

A record's identity is the SHA-256 digest of the RFC 8785 canonical JSON form of
its parsed typed data, not its YAML bytes. Comments, indentation and scalar style
therefore do not change identity. Floating-point fields are not admitted into
identity-bearing evidence unless a later schema defines an exact representation.

Human interpretation and pass-two proposals live in reviewed Markdown Action
Packets. Strict YAML frontmatter binds each packet to exact evidence-record IDs,
generations, preconditions and required authority. Prose cannot silently
override recorded facts, and a record cannot authorize its own action.
