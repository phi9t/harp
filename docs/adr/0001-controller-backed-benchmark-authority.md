# Keep benchmark grading independent of controller orchestration

Controller-backed benchmark runs reuse task definitions and protected verifiers, while the actual Corenius lead/controller path owns admissions, executions, candidate verification, acceptance and landing. The outer benchmark submits missions and independently observes and grades them; it must not use the existing fleet scheduler to manufacture controller transitions. This trades adapter work for evidence that the product collaboration path actually works.

The existing fleet mode remains a baseline. Automatic landing is confined to run-owned disposable repositories. Deterministic participant results and actual Codex lead/two-worker results remain separately labeled; task correctness and orchestration correctness are separate verdicts. The first milestone exercises independent two-ticket work and stale-base reintegration, not broad recovery or permanent readiness.
