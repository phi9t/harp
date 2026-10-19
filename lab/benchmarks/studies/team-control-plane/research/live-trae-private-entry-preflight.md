# Private Trae entry preflight — proposed, not exercised

Read 2026-10-01 at Corenius commit `963f58b31e518a8162dfe719e386ef9d9843321f`. Installed version/hash and exact generated contracts are recorded in [observations](live-trae-private-entry-observations.json). Schema generation exited0; no server, model, provider, credential file or effective-config dump was accessed. Existing jj evidence is unchanged.

## Entry and configuration

Installed help advertises `--no-daemon` (embedded service without shared daemon probing/start), direct `app-server --listen unix://PATH`, dotted TOML `-c key=value`, and `--profile NAME` layering `$TRAE_HOME/NAME.traecli.toml` over base configuration. InitializeResponse explicitly describes codexHome as absolute `$TRAE_HOME`; CLI default config help names `~/.trae/traecli.toml`. This documents an isolation knob but does not prove all defaults, system configuration, credentials or external context obey it.

Future proposed entry: independent supervisor launches the pinned absolute executable with `--no-daemon app-server --listen unix:///PRIVATE/socket`, child-only `TRAE_HOME=/PRIVATE/store`, a finite allowlisted environment, no inherited provider credentials, and a capsule excluding shared runtime/config/plugin paths. Do not change shell HOME/CODEX_HOME. Where library HOME lookups remain possible, capsule access must deny them; relocation of TRAE_HOME alone is insufficient evidence. Exact argv parsing and store/config exclusion still require a separately authorized model-free initialize/config-read fixture. Reject mismatched returned codexHome, unexpected config origins, or shared daemon/socket access. No executable launch package is approved by this document.

Existing ACP source disables memories/goals/multi_agent/multi_agent_v2/code_mode_host/plugins/plugin_hooks/hooks via features flags and passes an empty MCP list. These are existing adapter choices, not proof of app-server semantics. Disabling all shell tools conflicts with the proposed bounded background-job probe: a later package must explicitly allow only its fixture job through the independent boundary.

## Complete retained observation contracts

Twenty generated schemas under [live-trae-entry-schema](live-trae-entry-schema/) retain initialization, read/list/loaded/turns/items and background-terminal list/stop/clean request/response pairs, plus config-read. Initialization requires clientInfo.name/version; capabilities.experimentalApi opts into experimental APIs. Retain notifications rather than opting them out. Config read exposes config, origins and optional layers; future evidence must redact values and retain only approved nonsecret keys/origin paths.

Thread read requires threadId and optionally includeTurns. Thread list has cursor/limit, cwd, ancestry, archive, provider/source/originator and state-db filters. Loaded-list is cursor/limit. Turns and items require threadId, have pagination and sort; items optionally filters turnId. These cursors paginate queries: they do not establish a durable notification replay cursor. Background list requires threadId and returns processIds; stop requires threadId/processId; clean requires threadId. A processId string or stop reply is not complete OS descendant shutdown proof. Independent Execution inventory remains mandatory.

## Synthetic provider and spend bounds

Installed help advertises `--oss` and local-provider choices lmstudio/ollama. ConfigReadResponse advertises model_provider but does not expose a custom provider definition/endpoint schema. No verified arbitrary OpenAI-compatible or scripted synthetic-provider knob was found in these retained contracts; do not invent model_providers/base_url keys from another product. Whether app-server consumes OSS selection and supports a synthetic implementation of either protocol is untested. Next prerequisite is primary installed configuration documentation or an authorized isolated configuration/provider handshake with network blocked except the owned synthetic endpoint.

No hard ordinary-turn total token or monetary ceiling is established. Config fields model_context_window and model_auto_compact_token_limit concern context/compaction, not mission spend. Goal tokenBudget is not an ordinary-turn ceiling and goals remain disabled. A future paid run needs an independently enforced request/token/cost budget at the only permitted provider egress, including retries/continuations, or provider-side quota with verified hard refusal. Request/turn counts alone are insufficient. Synthetic-first preparation remains blocked on exact protocol/configuration support, not on owner launch approval.

## Scope and remaining gaps

This is installed help/schema evidence and a proposed entry, not observed private runtime behavior. No adoption, live recovery, cancellation, store persistence or feature exclusion is claimed. Before launch review: verify actual private-store/config origins; resolve synthetic provider routing; implement finite egress budgeting and injector; complete kernel16 supervision; then obtain scoped resource/provider approval. Missing capabilities must yield predetermined quarantine rather than fabricated recovery.
