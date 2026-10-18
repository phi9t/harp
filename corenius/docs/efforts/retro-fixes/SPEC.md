# Specification

Implement the exit criteria in EFFORT.md. Preserve default gate suite coverage and opt-in service switches. Bounded runs report failures and timeouts distinctly and clean owned descendants on cancellation. Short temporary roots must preserve ownership and mode checks; sandbox isolation remains unchanged.

Review artifacts must include complete content including untracked and binary changes, resolved base and head identities, and clear snapshot limitations. Refuse unsupported or unsafe file kinds instead of silently omitting them. Existing index and source files remain unchanged.

Add a noncommitting effort snapshot and an implementation handoff field/operation that preserves the existing ticket lifecycle and accepted-landed requirements. Existing dehydrate behavior remains available with explicit Git authority. Record validation evidence references.

Qualification runs both process-group and real cgroup durable tests, checks production SQLite admission, and fails on skips, missing code or capabilities. CI self-hosted execution is restricted to trusted default-branch manual runs; normal PR checks use hosted runners. Runner provisioning and hosting settings are explicitly outside this local edit.
