# Original launch provenance applies to every episode role

Status: bounded clarification of the reviewed repair contract under the user-approved baseline; implementation and acceptance remain pending.

Extends `20261002-original-launch-admission-provenance.md` explicitly to task-free Lead episodes. An Attempt groups sibling operations; absence of an Attempt does not exempt an episode from checking its own uncertain runtime operation.

Inspection of the repair source based on commit `58c96816ecfd8224d4e2b6f8f01e59eda1eff249` found an early return for an execution without an Attempt. That bypasses original-launch provenance for task-free Leads. The repair must query uncertain operations for the exact execution regardless of Attempt, and additionally query siblings when an Attempt exists. Keep legacy eligibility unchanged. No schema, protocol or provider authority expansion.

Require a real public task-free Lead Ready/own-Unknown regression, red against the early-return predicate and green after correction, with owned effects stopped before the assertion. Also prove registered task-free original launch survives Unknown confirmation before the final monitor gate exactly once. Existing certificates remain authorization evidence only; unknown usage and capacity are not refunded.

The intermediate full matrix passed 149 tests, with six child entrypoints and no failures or filtered tests. It predates this correction and cannot prove its acceptance. Repeat focused checks, the full matrix and strict checks before freezing the candidate for independent review.
