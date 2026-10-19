REJECT — one blocking Spec finding (P1).

The research verifier’s candidate-import route bypasses exact intent pinning. `Supervisor::create_episode_from_candidate` passes `Some(candidate)` (crates/team-control-plane/src/supervisor.rs:477–498), but `create_episode_inner` only validates/captures research authority when `source.is_none()` (:559–578). A research task can contain a real submitted candidate: artifacts.rs:740–777 does not restrict submission to production outcomes.

Concrete sequence: admit an independent research Verifier Episode with R1 input and a current candidate; admit source-only R2 while the work remains unaccepted; import that candidate through `create_episode_from_candidate`. Generation/spec/candidate checks still pass, while the obsolete R1 input is never compared with R2. The resulting manifest has `research_verification=None`. The start guard consequently skips current research-contract validation (supervisor.rs:298–304), permitting fresh execution with obsolete input (:942–966). Preparing before R2 likewise leaves the subsequent start unfenced.

This violates v02-reconciliation.md:69–72: “Every consequential grant checks … admitted IntentRevision/PlanRevision,” and the adopted lifecycle proposal:62–64 preserving “Exact current-intent proof checks.” The new route’s required preparation/start binding is incomplete. The source-only regression test exercises only `create_episode`, so its GREEN does not cover this alternative public Rust service entry point. This is a static source finding, not a newly executed reproduction.

Identity verification passed: 320 sealed records, 22 live/frozen source pairs, exact reconstruction of both patches, and all 59 final run inputs. Migrations0008/0009 match rejected repair3. All sixteen consumer classes, seven proof cases and four critique guardrails were traced; detailed coordinates and evidence limits are in metadata.json.

The recorded final local run passed 96 tests; sixteen chronological research measurements and nine final production artifacts match retained bytes/logs. These do not establish conformance for the bypass. No tests, services, providers, containers or archived scripts were launched. No full-gate, Temporal, live-v2 or restore qualification is claimed. Recorded resource retention still includes 17 roots and 29 unresolved runtime rows; process absence does not prove Stop or reconciled usage.

This allocated pass grants no repair, integration or implementation authority.
