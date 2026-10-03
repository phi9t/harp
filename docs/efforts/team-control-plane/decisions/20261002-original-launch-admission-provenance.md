# Original episode launch admission survives asynchronous confirmation

Status: reviewed bounded kernel repair contract under approved baseline, not acceptance. Supersedes the dispatching-only exemption proposal where it races final monitor launch. No new schema or runtime provider authority.

Independent reviewer inspected actual start/monitor ordering. Public Supervisor start moves Ready to Starting and returns before final monitor gate; native reply can confirm first. Neither same execution ID nor Starting status proves original authorized runtime origin.

The Ready-to-Starting transaction can permit the exact own Dispatching episode operation only after authoritative execution/episode/current epoch/input/request argv binding and independently validated registered controller origin, including actual local caller process frame matching registration. Append immutable launch-admission evidence in the existing episode evidence table within the SAME transaction as Starting. Bind operation, immutable request, controller, recovery epoch, execution and actual argv. No caller-invented identity suffices.

Final monitor continuation may recognize that exact certificate and current persisted Starting argv/request even after own journal changes to confirmed/unknown. This permits only the previously authorized original launch, never a second launch/retry. Other same-Attempt operations remain fenced. Existing cancellation, generation, epoch, deadline and mission-pause checks remain independent and mandatory. Public start replay cannot create another monitor or effect.

The certificate records launch authorization only. It proves neither stopped effects nor measured-zero usage, and cannot release capacity or accept results.

Required behavioral controls: registered public first launch positive; monitor deliberately paused after authorized start, own reply becomes Unknown before final check, then original launch succeeds once; Ready own Unknown without certificate refused; foreign/unregistered/caller-ID origin refused; sibling unknown introduced before final gate blocks; current stored argv/request/certificate corruption refused. Legacy eligibility scope remains unchanged.
