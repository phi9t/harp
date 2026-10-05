# Credential-safe native Codex provider readiness

Read source commit a75c520761e5f93f078abdfd1d99f24369499b99. READ-ONLY repository inspection plus bounded actual INTERNAL proxy tests with newly generated fake credentials and owned loopback upstreams. No host auth contents read, account authentication, real inference endpoint, paid call, product/worktree edit, user tmux or repeated ticket02 topology qualification. Earlier native/TUI evidence remains unchanged; origin paths and hashes are retained in retained-evidence-summary.json.

## Supported native configuration and known live boundary

The custom provider path is usable without native account credentials. Actual retained native TUI account/read returned `{account:null,requiresOpenaiAuth:false,workspaceRouting:null}`; that same TUI resumed its registered thread and submitted input successfully. Both inspected owned fixture CODEX_HOME/auth.json paths are absent. Actual GPT6 native app-server plus companion completed fake Responses and dynamic-tool loops under this provider. Therefore neither a native sign-in nor mounting host auth/home is needed for this runtime/TUI bootstrap path. This does not prove an upstream credential or model entitlement.

Use a private pinned config with top-level model, model_provider and model_catalog_json, then:

    model = "gpt-6-sol"
    model_provider = "corenius-relay"
    model_catalog_json = "/PINNED-CATALOG.json"
    web_search = "disabled"

    [model_providers.corenius-relay]
    name = "Corenius trusted Responses relay"
    base_url = "http://127.0.0.1:FIXED_PRIVATE_PORT/v1"
    wire_api = "responses"
    requires_openai_auth = false
    supports_websockets = false
    supports_standalone_web_search = false
    request_max_retries = 0
    stream_max_retries = 0
    stream_idle_timeout_ms = 30000

This is a configuration skeleton, not a complete production profile. GPT6 requires the already pinned/qualified code_mode_host companion enabled. Preserve the earlier disabled telemetry/plugin/remote-service settings. If the local relay requires a bearer grant, env_key may name an attempt/role-scoped relay credential, never an upstream provider secret. A private fixed socket plus trusted bridge identity may instead authenticate the connection; qualify whichever mechanism is chosen. No native auth file or upstream secret enters either native control realm or tool realm.

The realm's private loopback HTTP adapter reaches only the controller-provisioned pathname socket and trusted relay. Trusted relay owns upstream TLS, credential lookup, rotation and fixed routing. Ticket02 is implementing the bridge/isolation path; this report neither reimplements nor certifies it. A model-provider route does not replace MCP, tool, TUI or namespace authorization.

The retained official OpenAI gateway documentation explicitly supports custom Responses providers and keeps upstream credentials gateway-side; its compatibility requirements include incremental SSE, continuation and tool identifiers. These official pages were fetched and hashed earlier on 2026-10-04, then re-read locally here: [connection](https://learn.chatgpt.com/docs/enterprise/connect-to-a-gateway), [compatibility](https://learn.chatgpt.com/docs/enterprise/gateway-compatibility). No new web/model requests were made.

## Upstream choice: facts versus unresolved credentials

**Documented option:** an owner-approved public Responses API credential or existing organization gateway, held only by the trusted relay. Configure one approved HTTPS origin/path and exact model route. For public API access, trusted-side Authorization is the selected API bearer; optional organization/project routing must come from trusted configuration. No caller-supplied ChatGPT account header belongs on that route by default. This option is not evidence that such an approved credential currently exists here.

**Distinct documented option:** the authorized Sign in with ChatGPT plan-usage flow obtains its own OAuth token and calls public `https://api.openai.com/v1/responses`, with `store:false`, `stream:true`, replayed HTTP input and its published tool/parameter limitations. That flow explicitly excludes ChatGPT backend-api routing. A relay can retain that flow's token and renewal authority trusted-side; moving credentials out of the app-server is application design, not a completed implementation. Existing native login tokens must not be assumed interchangeable with this authorization flow. Sources retained: [plan inference](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference), [app-server example](https://developers.openai.com/siwc/token-sharing-open-source/codex-app-server), [limitations](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations).

**Still unqualified:** routing the user's existing native Codex ChatGPT login through a custom relay to `chatgpt.com/backend-api/codex/responses`. Corenius source sends a native access token plus ChatGPT-Account-Id there, but that code is not a supported native auth contract, has no token refresh implementation, and provides no backend entitlement proof. The prior pinned-source lookup did not establish exact native refresh/account/header behavior. This task deliberately did not inspect real credentials. Neither fake-token acceptance by a local proxy nor existing code is evidence that transplanting a native login token into a public/API-key route works. Native ChatGPT reuse needs a pinned supported trusted-side auth/renewal contract and bounded authorized live verification before it can be claimed ready.

## Existing repository components

`src/provider/codex.rs:62` creates a **direct non-native provider**, not an HTTP relay for Codex CLI. At :80 it calls load_codex_credentials; :102–103 sends its access token and account ID; :111 buffers the complete response with response.text(). Its :187 request loop constructs a reduced Responses payload, and :280 recognizes only ordinary function_call items. It cannot transparently carry the proven GPT6 additional_tools/custom_tool_call/code-mode shape. Reusing it as the runtime would replace actual Codex and fail the native qualification contract.

`load_codex_credentials` at :352 verifies a private regular auth file, reads four token/account fields and requires chatgpt auth_mode. It does not refresh: refresh_token is loaded/stored only. Its private-file checks are a reusable pattern; executing this reader in a worker/control realm would violate the boundary. This review read source code only, never that auth file.

`parse_sse_events` and tests/provider_codex.rs supply small fixtures for malformed/truncated SSE, function continuation and usage. They operate on whole buffered strings and are not an incremental relay/parser or native tool pass-through qualification. Cargo.toml already includes reqwest with rustls-tls/stream and futures-util; those are useful building blocks, not an implemented relay.

`crates/team-control-plane/src/gateway.rs` is the parked PostgreSQL runtime-operation/identity journal, not an inference HTTP gateway. Do not revive that backend or confuse its name with model transport. The SQLite service/supervisor Unix request framing and owned lifecycle patterns can inform supervision, but those bounded JSON command sockets are not SSE HTTP forwarding. No credential-safe fixed Responses relay implementation was found in the read root source. Active ticket02 work was intentionally not audited or duplicated.

## Newly observed pinned internal proxy primitive

Pinned `codex responses-api-proxy --help` exists but labels itself INTERNAL. It accepts --upstream-url (default public Responses), --port, --server-info, optional --http-shutdown and --dump-dir. It is hidden from ordinary top-level help. This is a version-pinned internal transport primitive, not a declared supported security boundary. `codex app-server proxy` is a different command that proxies app-server stdio to a control socket; it is not a model relay.

Two bounded executions supplied only synthetic credentials over parent-owned stdin and fixed local fake upstream URLs. Every endpoint, request and resource is retained. Observed:

* The proxy injects its stdin value as Authorization: Bearer, replacing a caller Authorization header. A second run accepted an opaque fake OAuth-like string without an API-key prefix. It does not derive identity from native login. This shows bearer transport only, not real OAuth/backend support.
* Host was rewritten to the configured upstream. POST /v1/responses forwarded; POST /v1/responses/compact, GET /v1/models, an absolute-form target and CONNECT returned403 without reaching the fake upstream.
* Caller ChatGPT-Account-Id, X-Forwarded-Host and Originator passed through. There is no shown CLI option for trusted account-header injection, account validation, scoped client authentication or model allowlisting. An unqualified model string was forwarded unchanged.
* A307 from the configured upstream was followed to a second owned local origin. Authorization was omitted at that other origin, but the account header and request body were forwarded. Thus --upstream-url alone does NOT enforce a no-redirect fixed-origin boundary.
* A small first SSE event was delivered only with completion about300ms later. A64KiB first chunk arrived in about1ms while completion remained about300ms later. This demonstrates streaming with buffering, not reliable immediate small-event flushing and not universal whole-response buffering.
* A synthetic401 status and JSON error body reached the caller. No renewal/retry occurred in that observed request. This is not evidence of refresh, revocation policy or comprehensive error handling. The process uses a fixed supplied bearer for its lifetime; external rotation/restart would require further qualification.

No request/response dump flag or HTTP shutdown endpoint was enabled. The synthetic upstream logs intentionally retain fake bearer/account values; none are production secrets. Both positively owned proxy Popen processes were terminated and waited; fresh oracle confirms no recorded PID remains.

## Concrete missing ticket06 relay work

A trusted-side wrapper or new narrow relay still needs to enforce the selected credential/route contract. The internal helper as-is is insufficient because it accepts arbitrary models, forwards account/routing headers and follows redirects. Prefer a transport with redirects disabled and explicit small-event flushing; if the internal helper is reused, prove its wrapper and network policy close those gaps rather than trusting its fixed URL flag.

The relay's minimum contract is:

1. Authenticate a fixed realm/role/attempt grant or protected bridge identity against current controller authority. Never let native client parameters select credential references or upstream origins. Keep upstream credentials and refresh/rotation storage outside every native/control/tool realm.
2. Allow only required method/path combinations. The proven native requests use POST /v1/responses. Model catalog is pinned locally; model/list RPC is not upstream entitlement. Reject compact/discovery until the actual selected long-running profile needs them and their fixed routes are qualified. Do not silently send unsupported paths somewhere else.
3. Pin scheme/host/port/path, disable cross-origin redirects and generic proxy/CONNECT/Upgrade behavior. Ignore inherited proxy environment. Remove all caller Authorization, account/org/project/Host and forwarded routing headers, then inject only the trusted route's required values. Allowlist safe attribution metadata separately; caller-controlled metadata is not principal identity.
4. Validate model/feature route, bounded request size/concurrency/timeouts and JSON/HTTP framing; preserve the native payload and tool/response identifiers. Do not rebuild it with CodexDirectProvider. Actual GPT6 payload includes additional_tools and nested custom code-mode results, encrypted reasoning include, prompt_cache_key and client_metadata. Selected plan-usage routes additionally need their published parameter/tool restrictions. Store:false and stream:true were observed in every retained GPT6 request.
5. Stream SSE incrementally with cancellation/backpressure; preserve terminal completed/failed/error/incomplete distinctions, HTTP statuses and redacted correlation. Tool failure plus a completed model turn is not task success. If the downstream disappears or a response is ambiguous, do not blindly replay the inference request. Native/relay retries must be jointly bounded and observed.
6. Implement and qualify the chosen trusted-side credential lifecycle: expiration,401, refresh/rotation locking, revoked local grants and upstream denial. Do not print tokens in environment dumps, logs, errors or config. Turn/model/account access must be confirmed from trusted relay/upstream observations during the later owner-authorized live run, never model self-report.

Before live inference, fixed-loopback fake upstream tests must exercise those exact implemented policies, redirects, hostile headers, grant revocation, streaming and actual native custom-tool round trips through the relay. This report supplies source/protocol facts, not that implementation or permission to perform a paid call. The owner-authorized upstream identity/route remains an explicit prerequisite for the actual live trial.
