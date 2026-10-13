---
name: harp-scope-review
description: Review a Harp candidate against its requested behavior, operating scope, and real evidence; use for independent verification and review repair without turning optional infrastructure into blockers.
---

# Scope review

Review the actual candidate against the user's request, accepted requirements, repository instructions, and intended operation. Use a reviewer independent of implementation when required; do not treat a worker summary as proof.

## Review input

Identify the source range or exact candidate, intended behavior, operating conditions, acceptance criteria, and checks already run. Inspect the relevant diff and artifacts. Distinguish commands executed, evidence inspected, and conclusions inferred.

## Findings

For each blocking finding, state:

- The violated requirement or concrete failure/risk in the current operation.
- The file/location and supporting evidence.
- The smallest repair and a meaningful resolving check.

Classify relevance before severity. Missing capabilities of a future fleet, service, or threat model are follow-ups unless explicitly requested. A severity label does not expand scope. Real data loss, authorization failures, isolation defects, and credential exposure remain blocking even when the written plan omitted them.

Check requirements, error handling, maintainability, integration, and behavior tests to the extent relevant to the change. Apply [the test-selection guidance](../../../AGENTS.md#test-selection): assess the regressions covered, not test count. Flag assertions coupled to private structure or incidental call order when they obstruct a contract-preserving change; do not demand tests for every helper. For command launchers, run the actual command and inspect what it selected or produced. Exercise applicable parser, process, cache, environment, timeout, and cleanup boundaries; do not substitute a general hardening matrix for testing the current operation.

A report should include findings, required repairs, advisory follow-ups, evidence coverage, and readiness for the requested operation. Technical readiness is not merge or deployment authority.

## Repair and finish

Evaluate feedback against code and evidence. Repair relevant critical and important defects before dependent execution or acceptance; push back on unsupported scope expansion with reasons. Rerun affected checks and required final gates, and obtain the necessary independent review of repairs.

When findings repeatedly expand supporting infrastructure, return to the useful operation and classify dependencies with [harp-delivery-plan](../harp-delivery-plan/SKILL.md). Preserve explicit final requirements. An actionable next slice can proceed while later uncertainties have named resolving probes, provided its own requirements and safety conditions are met.
