You receive a frozen candidate and two sealed finding sets. Produce at most one
repaired candidate.

For every finding, record exactly one disposition:

- `fixed` when the new candidate contains a complete repair;
- `rejected_with_reason` when the finding is demonstrably inapplicable; or
- `unresolved` when no rigorous repair is available.

Do not erase an obligation by changing its name. Return every remaining
unproved obligation and classify theorem-strength obligations explicitly.
The harness, not you, decides promotion.
