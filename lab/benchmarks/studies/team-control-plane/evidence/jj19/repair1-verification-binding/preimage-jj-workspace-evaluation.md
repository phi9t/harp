# Pinned jj workspace evaluation (ticket 19)

Status: evaluation complete; independent conformance review and owner decision pending.
Recommendation: **defer backend adoption**, retain Git as the supported backend.
This recommendation is inference from the bounded results below, not an accepted architectural decision or authorization to implement ticket 25.

Read/evaluated: 2026-10-03. Corenius exact base read: `264756edd550225449202b6f20ac4b560808d14e`.
Jujutsu source read/executed: `af45d57de7163cc5f17f3df178a31577e4951ea3`.
Executable: `0.44.0-af45d57de7163cc5f17f3df178a31577e4951ea3`, SHA256 `15fe28e44398552177ce01ba69dd17b52151e8e604ccf82fe587acd40ba13be4`.
Only the absolute pinned executable was invoked. The unrelated PATH `jj` helper was never used.

## Outcome and recommendation

All five final behavior slices passed, and all five unsafe public controls failed at their intended behavioral assertion. The evaluation uses an executable, disposable shim exposing create_attempt, inspect, checkpoint, freeze_candidate, prepare_integration and release. It does not install a production workspace service or change kernel/provider/workflow/schema behavior. Every attempt starts in its own repository store at a recorded exact base; one trusted fixture writer owns every disposable workspace. The peer shared-store case deliberately measures unrestricted raw CLI authority.

The results show jj can preserve immutable exported candidates and recover a specific interrupted checkout, while its convenient defaults add authority-sensitive behavior. Status snapshots edits; shared workspaces can rewrite each other's changes; no-snapshot status can reconcile repository operation heads; ignored tracked content remains captured; successful Git export can leave a conflicted bookmark's old Git ref. These require explicit service authorization, exact-operation inspection, private/path/type validation and pre-export ambiguity rejection. No measured reduction in lost work, recovery effort or integration cost was established against Git. Keeping Git avoids committing to an unmeasured benefit and a new mediated mutation surface. A future adoption decision must explicitly authorize ticket 25 and its production broker integration, pin/toolchain maintenance and security review.

## Requirement and accepted-seam coverage

| Canonical requirement / seam | Actual final proof | Result and limit |
| --- | --- | --- |
| create_attempt / inspect: separate exact-base attempt/store/workspace identities | [authority](../evidence/jj19/verified-authority-green.json), [identity](../evidence/jj19/verified-identity-green.json) | Manifests record store/workspace, full operation/change, exact commit/tree and base. Exact-operation inspection plus no-snapshot operation lookup leaves edits uncaptured; ordinary status red changes operation/commit/tree. Evaluator checks explicit snapshot authority, not production broker credentials. |
| checkpoint: explicit snapshot authority and journal preparation | [authority](../evidence/jj19/verified-authority-green.json), all final JSON durable_preexecution_journal fields | Unauthorized checkpoint refuses; authorized status captures edits. Prepared command journal is fsynced before execution and retained byte-for-byte. This journal is an evaluation fixture, not a replacement for kernel state. |
| freeze_candidate: old immutable proof after content and metadata rewrite | [identity](../evidence/jj19/verified-identity-green.json), [red](../evidence/jj19/verified-identity-red.json) | Stable change ID with changed commit/tree cannot reuse proof; independent Git import retains original bytes/IDs. Description-only rewrite keeps the tree but changes commit and therefore candidate identity. No inferred verification transfer follows change/bookmark/@. |
| stale workspace and divergent operation recovery, preserved other-attempt authority | [authority](../evidence/jj19/verified-authority-green.json) | Peer content restore produces actual stale status refusal; update-stale repairs it. Two historical-operation descriptions create divergent revisions; automatic reconciliation is observed and ambiguous change selection refuses freeze. Separate-store other attempt is unchanged before/after. Raw shared-store isolation is explicitly disproven, not claimed as a security boundary. |
| interrupted operation before working-copy update, no assumed rollback | [crash](../evidence/jj19/verified-crash-green.json), [red](../evidence/jj19/verified-crash-red.json) | Real strace syscall-delay injection and SIGKILL; durable operation is observed while disk bytes remain old. Native stale refusal followed by update-stale restores exact target. Owned tracer waited, traced jj PID absent, fixture removed. Scope is this specific Linux checkout boundary. |
| private/ignored/out-of-scope files, capsule paths, content/bookmark ambiguity | [exclusions](../evidence/jj19/verified-exclusions-green.json), [red](../evidence/jj19/verified-exclusions-red.json) | Fresh ignored private file absent; already-tracked private file survives ignore but freeze refuses. Nonignored unexpected path and allowed-name symlink refuse. Real bwrap allows workspace write and denies sibling write. Actual content conflict and conflicted bookmark refuse candidate freeze; native export exits recorded, old Git bookmark retained. |
| prepare_integration: exact Git export and existing gate/evidence/GC/fast-forward tooling | [integration](../evidence/jj19/verified-integration-green.json), [red](../evidence/jj19/verified-integration-red.json) | Exported commit/tree independently inspected; failing existing land gate leaves disposable main unchanged; correct gate lands exactly the verified tip by fast-forward. Existing evidence verifies then rejects tampering. GC dry/dirty resources preserved; safe merged tree removed. Existing tooling suites: wt 12, land 8, evidence 15 tests passed. |
| release | Final identity/crash/integration release manifests and each JSON fixture_removed | jj workspace forget is followed by removal of owned disposable fixture repositories after commands/process cleanup. No kernel capacity/usage or unknown-runtime release is simulated. |

## Concrete final identities

The complete full IDs, argv, exits, stdout/stderr, repository identities and prepared journal are retained in the linked JSON, indexed by [public-proof-index](../evidence/jj19/public-proof-index.json). Identifiers are observations, not mutable aliases.

- Retained verified candidate: commit `400b22815a25e559a93c7f2584fcab51f527dfd4`, tree `12ce35e9b9c1d3e3a2a2034871dfff2e623724de`, change `oypxvxukkpwlwunvwmotwutlutkzwsuz`, operation `26a688fbbc5c60f365a8d86acc6e940577e4af611b920e044ac8d62b0817892068161a46f5f140c69802e2dbad55b1d8a2564b6026871649e63008800afcb18d`. Retained bundle SHA256 `d8766f04707d58d6279a642a1b7d9c21d07d185a3f5731470a4dac92535bbf4e`.
- Crash operation before: `d0416b4e7e42e96af2d4a95dd15450de4444c3a40d6c63151c42d498b38605f3f660d00ff7b306e43e2ef686a54c9c801699f93fd21f084077b63065eee55b3e`; durable operation observed before SIGKILL: `c0bc0b90ad1c55c1f9af9c45a004edb968b74379f4b6d365e2b7fd3b25dce4f21bba5f437d3cc43246b0f627b77dcd9bca2e4d9ee5baf06c522ff75820f5f425`. Target commit `7a14e49d20f3577ffa02d739e3f3ca4a84e3cd2b`, tree `9c47ee7c5931ac59125b9e96ad7d013e6909291d`. Working-copy lock ordinal `5`; owned tracer terminal `-9`; its traced child PID entries were observed absent after wait.
- Exported/verified/landed disposable Git candidate: commit `c59b5c507f5f95f4e1f9e93f455719b26787bfac`, tree `855bfa8e43bd52235bbed4531cdec66a0e63998f`, base `bf88a286f6b3cb2e73eda99d3019f96a501a7c48`, bundle SHA256 `2d31eb936b9604f0f355008fb38bf4d4632d856be5985cc953bcc3f75c8ae90a`.
- Actual existing evidence record: `sha256:210ae0a4e7a8beb23dba8c4bb41cf6bf15a0cee9b258ad332e35eef7493fad2e`. This record's measured HEAD equals the exported candidate; the final fixture also records the tree independently. Altering the retained event input causes verification failure.

## Measured Git comparison

[The real Git baseline](../evidence/jj19/verified-git_baseline-green.json) creates a separate repository and attempt branch. Git status leaves HEAD unchanged after shell edits; checkpoint requires explicit add/commit. Its verified checkpoint `a8bd0ace59410a1ca1316517472ad86da0f2ea61` / tree `af3732e7ad58e86bbbd53aa2cf103f1da1caa069` stays available after a replacement candidate `93deee0533055bbf03b9af992881606a1161cfa3` / tree `f87eef25a5f7961514b935496d4ab4af95c6c74d`, and an independent bundle import preserves the old exact candidate. Repository-only update-ref deliberately leaves old checkout bytes; explicit reset to the recorded target reconciles repository/checkout state. This is public state-separation/reconciliation evidence, **not** a Git SIGKILL demonstration.

| Behavior | Git baseline | Pinned jj measurement |
| --- | --- | --- |
| Status after edits | HEAD unchanged | Ordinary status snapshots and changes operation/commit/tree |
| Checkpoint | Explicit add/commit | Explicit authorized snapshot; most raw commands can snapshot implicitly |
| Rewrite and retained proof | Exact old commit/tree survives branch replacement and bundle import | Exact old commit/tree survives stable-change rewrite and bundle import |
| Repository/checkout separation | Public update-ref then explicit target reset | Actual SIGKILL after operation publication then stale repair to exact target |
| Integration tooling | Native disposable Git gate/evidence/GC/FF fixtures | Same tooling demonstrated after exact jj export/import |

The two recovery demonstrations have different fault scopes. Neither supports performance, arbitrary crash completeness or automatic rollback claims.

## Primary-source cross-check

The pinned [working-copy documentation](https://github.com/jj-vcs/jj/blob/af45d57de7163cc5f17f3df178a31577e4951ea3/docs/working-copy.md) describes automatic snapshots, shared workspaces, tracked ignore behavior and the repository-operation/checkout gap. The [operation-log documentation](https://github.com/jj-vcs/jj/blob/af45d57de7163cc5f17f3df178a31577e4951ea3/docs/operation-log.md) describes concurrent view reconciliation. The execution results above, rather than documentation alone, establish the measured outcomes.

Pinned source [cli_util.rs](https://github.com/jj-vcs/jj/blob/af45d57de7163cc5f17f3df178a31577e4951ea3/cli/src/cli_util.rs#L775) resolves operation heads separately from snapshot suppression and records the repository transaction before working-copy update. [Git export implementation](https://github.com/jj-vcs/jj/blob/af45d57de7163cc5f17f3df178a31577e4951ea3/lib/src/git.rs#L1554) can skip a conflicted new bookmark target. The fixture independently observes the old Git ref after export. Source read does not prove arbitrary crash or service-level authority correctness.

## Reproduction and evidence classification

From the author tree, run the executable fixture with the absolute binary, `--slice identity|authority|crash|exclusions|integration|git_baseline` and an explicit JSON output path. `--unsafe-control` intentionally falsifies unsafe identity, inspection, rollback, ignore-only or failed-gate assumptions. It must exit 1 at the recorded assertion; ordinary slices must exit 0. [verified-run-ledger](../evidence/jj19/verified-run-ledger.json) records every exact final command and actual terminal.

Final runner session `45882` terminated 0 after eleven commands: five behavioral reds (each exit 1), five jj green slices and one Git baseline (each exit 0). Every final record binds the exact executable fixture source SHA256. Earlier candidate runner `50170` terminated 0 before the Git baseline method and source-hash binding were added. Earlier runner `33283` also terminated 0 before exact-base and symlink additions; those records are superseded. Integration child session `52875` terminated 0; exclusions `66033` terminated 0. [Run classification](../evidence/jj19/run-classification.json) preserves every intermediate setup/oracle failure and earlier green; file names containing green are not by themselves passing evidence. The first identity red reused the historical preflight helper whose declared Corenius base constant is older; current ticket acceptance uses the final candidate records with the correct exact base.

Intermediate authority failures were fixture issues: an operation query omitted snapshot suppression, a description-only rewrite changed metadata without the anticipated content-stale refusal, a peer workspace repository pointer required resolution, and native divergent-selection failure needed refusal classification. These were preserved rather than called passing tests or meaningful regression reds. Tracker lint initially rejected a comment without a full timestamp; the timestamp was corrected and lint rerun. No independent repair/review round has yet occurred.

## Limits and dependent handoff

No shared checkout was converted; no real mainline/ref, remote, provider, credentials, paid calls, Cargo build, Postgres or Temporal service was touched. Private home/config/cache and disabled inherited Git configuration/hooks were used. Deliberate local hook tests are owned disposable fixtures. Browser/source research used primary pinned upstream material.

Crash evidence covers SIGKILL after durable operation publication before one working-copy lock/update, not storage corruption, arbitrary asynchronous writes, lost historical operations, power loss, remote pushes or external job rollback. Capsule evidence covers real write confinement, not confidentiality or a production insula integration. Separate repository stores avoid cross-attempt mutable sharing; this evaluation does not prove a mediated shared-store authorization service. Existing Git gate/evidence/GC compatibility is demonstrated after exact export/import, not unchanged support for native jj workspace directories. No throughput or comparative human-effort benefit was measured.

Final syntax, report-link/proof-hash checks, tracker lint and diff checks pass; [exact checks and terminal handles](../evidence/jj19/final-checks.json) distinguish these from the full repository gate. The author did not run check.sh because its team-control-plane hook launches real Cargo/Postgres/Temporal while root retains that exclusive slot.

Root owns Standards/Spec conformance, fresh independent branch verification and integration; ticket 19 stays claimed until those gates are accepted. Ticket 25 stays blocked on an explicit owner adopt/defer decision. This report recommends defer while retaining the executable measurements for a future bounded decision.
