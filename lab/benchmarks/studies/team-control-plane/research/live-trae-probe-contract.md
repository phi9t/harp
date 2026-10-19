# Proposed live Trae probe contract

Read Corenius commit `963f58b31e518a8162dfe719e386ef9d9843321f` plus the
accepted10–12 content manifest. This prepares tickets04/20; it does not authorize
a launch, close either ticket, or assert observed recovery guarantees.

## Observed entry surface

Fresh read-only installed help/version checks identify TraeCLI
`0.207.1(internal edition)`, executable SHA256
`9f5531a1599cd7d81df35a9b6a7568f6652fbf58141d33da9853a62046caf8c2`.
`app-server --help` advertises private `unix://PATH` transport. Top-level
`--no-daemon` help states that it avoids probing/starting the shared background
server. These are advertised interfaces, not exercised private-entry guarantees.
Retained schema hashes are in [trae-schema/manifest.json](trae-schema/manifest.json).
Ordinary start correlation is not demonstrated deduplication; retained schemas
provide no demonstrated durable notification cursor.

Recommendation: use a privately supervised app-server and reconnectable Unix
transport. An exec fallback may offer bounded process/artifact execution, but
cannot stand in for measured thread/turn steering and recovery. Keep any such
fallback a separately named capability limitation.

## Identity, isolation and ownership

Use one disposable repository, dedicated runtime store, private socket and an
independent supervisor Execution. Record executable/schema/configuration and
capsule digests, exact Git base, store path/device/inode, deployment UUID,
Execution handle and each server incarnation. Verify initialization's returned
`codexHome` against that store. Reopen the same store after restart; an empty
replacement store is not recovery.

The executable preflight must prove how configuration/store isolation works
without changing the user's shell HOME or shared runtime installation. Do not
mount the shared runtime home or inherit its hooks, plugins or MCP connections.
Provider credential provisioning is separate and must expose only approved
credentials. Disable goals, memories, unsolicited tools and agent spawning;
allow only bounded fixture jobs and writable fixture/runtime directories.
Ticket16 must first establish namespace membership and shutdown proof; outer
process-group termination alone is insufficient.

## Proposed matrix

| Probe | Action | Required verdict/evidence |
| --- | --- | --- |
| Private entry/store | Initialize, create without a turn, read/resume; stop and reopen same store | Exact store/thread retained, no shared daemon or model access; unsupported behavior named |
| Unknown thread creation | Journal create; injector forwards it and withholds response/notifications | Independently inventory supported store/thread records; adopt only an unambiguous match, otherwise quarantine unknown binding; never issue another create as recovery |
| Known-thread lost start reply | Journal start with known thread, unique input digest and correlation field; hide reply | Original turn/history and independent effects establish adoption or precise quarantine; correlation alone never deduplicates; never repeat start |
| Client detach | Disconnect while the same turn is active, retain server/supervisor, reconnect | Distinguish continued execution from already-finished execution using exact turn and effects |
| Server restart | Stop server during active turn, inspect survivors, reopen same store | Recover original execution, prove stop, or quarantine; retained history does not prove surviving executor |
| Steering race | Wrong expected turn ID, matching active ID, then original ID after completion | Wrong/ended IDs cannot affect another turn; successful steer binds exact turn; racing delivery remains pending rather than triggering a new start |
| Interrupt/background jobs | Bounded allowlisted heartbeat job; exact-ID interrupt and supported job-stop operations | Independent supervisor proves complete owned membership stopped and writes ceased; acknowledgment alone is insufficient |
| Goal ownership | Keep goals disabled; inspect state across restart and account for provider requests/turns | No autonomous extra episode; unsupported inspection recorded; setting/enabling goals excluded |

Injector forwarding and selective hiding must be durably recorded. Keep
independent oracle observations separate from recovery-adapter inputs. A hidden
reply may prove what happened for the test; it cannot secretly give the adapter
the missing ID.

## Proposed finite resource envelope

This envelope is a proposal requiring explicit owner approval when the executable
package is reviewable. It is not covered by model-free baseline approval.

- One provider turn active at a time; three planned starts (lost reply/detach/
  restart, steering, cancellation), plus one reserved contingency: four maximum.
- Six provider requests maximum, including continuations and failed requests
  reaching the provider; 120 seconds per turn, four server incarnations maximum,
  fifteen minutes total including shutdown. Read/control RPCs have finite deadlines.
- No automatic retry or budget extension. Synthetic provider first if supported.
  Paid-provider use additionally needs an approved provider/model and an
  enforceable monetary or token ceiling; turn counts alone do not bound spend.
- Stop dispatch on exhaustion, identity drift, shared-service access, missing
  required control, uncertain execution, unexpected goal continuation or
  incomplete survivor inventory. Preserve evidence and quarantine affected
  resources. Delete only inactive owned fixtures whose stop is proven.

## Launch prerequisites still missing

Completed kernel16; executable private-entry/configuration verification;
complete initialization/read/list/background-job schemas; selected provider and
budget enforcement; tested injector boundary; independent membership/shutdown
proof; redacted evidence rules; and explicit owner scope/resource approval.
Unsupported capabilities need predetermined quarantine verdicts.

The proposed architecture/matrix was prepared in a separate read-only context;
the primary independently checked installed version, executable hash and help.
No app-server, provider, goal or model was launched for this preparation.

## Private-entry/schema follow-up

[Installed-contract preflight](live-trae-private-entry-preflight.md) now retains
20 complete initialization/read/list/background-terminal/config-read schemas
and documents TRAE_HOME/privateUnix/daemonavoidance knobs. These remain
unexercised isolation contracts. Exact syntheticprovider routing and hard
ordinary-turn spend ceiling remain unverified; no server/provider launched.
