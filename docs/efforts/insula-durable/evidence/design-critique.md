# Independent design critique

Base read: e045d4a. Reviewer: clean-context design_critique agent.

Findings: cancellation discarded started manual-effect uncertainty; temporary environment was not inherited; opaque demo contract did not bind Insula; launch scripts modified sys.path; component ownership needed clarification.

Resolution: public cancellation/sibling-failure regressions and UNCERTAIN/BLOCKED preservation; runtime launch injects approved temporaries and forbids overrides; verified full capsule/code/state/backend binding with private temporary mount; module launches from repository/admitted import root; controller/durable ownership recorded in ADR-0003. User clarified trying process-group and cgroup backends. Cgroup control and guardian metadata remain outside payload mounts.

Evidence ceiling: source critique only, not runtime verification. Real cgroup qualification needs explicit delegation, unavailable to current UID.
