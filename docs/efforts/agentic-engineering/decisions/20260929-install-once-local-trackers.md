# Sync installs local tracker files once and never overwrites

Date: 20260929
Supersedes: 

## Decision

The sync tool creates missing local tracker files and does not enforce their content afterwards.

## Why

Repos keep repo-specific tracker text (for example torchtitan's Kata paragraph).

## Consequences

Corenius edits its own issue-tracker.md freely; siblings are not touched.
