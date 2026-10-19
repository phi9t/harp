# Interception spike (M3.1): can corenius enforce path claims?

Read at corenius commit 86eaff3 (branch `claude/m3-interception`). Marks: **[V]** verified here (file:line
or a command run), **[U]** unverified assumption. No live TraeCLI or Codex session was run.

## Answer

Claims are **advisory** for anything an ACP provider does with its own tools, and **enforceable only
below the protocol** (sandbox path allowlist). The ACP protocol gate is real but covers a slice the
provider may never use. Corenius today does not implement it at all.

## 1. What corenius implements today

- **[V]** `initialize` advertises `"clientCapabilities": {}` (`terminal_agent/src/provider/acp.rs:199`). No `fs`, no
  `terminal`. A spec-following agent must then not call `fs/*` or `terminal/*` on us.
- **[V]** Corenius handles exactly three inbound shapes: responses to ids 1/2 (`expect_response`,
  `acp.rs:320-338`), the response to id 3 (`acp.rs:247`), and `session/update` notifications, of which
  only `agent_message_chunk` text is used (`acp.rs:256`, `340-350`). Every other line, including
  agent-to-client *requests* (`fs/write_text_file`, `fs/read_text_file`, `session/request_permission`,
  `terminal/*`), is silently dropped and **never answered**. An agent that blocks on
  `session/request_permission` would hang until the 120 s timeout (`acp.rs:53`, `148-153`).
- **[V]** Latent bug: the id-3 check ignores `method`, so an agent request that reuses id 3 is taken as
  the prompt response and ends the turn early (`acp.rs:247`). Pinned by test
  `agent_request_reusing_prompt_id_ends_turn_early`.
- **[V]** Tool declarations and the executor are discarded for ACP (`acp.rs:75-77`, `104-106`). The
  provider runs its own tool loop; `PolicyGate` (`terminal_agent/src/core/policy.rs:27-46`) and the tool registry
  (`terminal_agent/src/tools.rs:44-103`, only `read_file` and `shell`, no write tool) never see its actions.
- **[V]** The provider process is spawned directly on the host with `current_dir(cwd)`
  (`acp.rs:161-169`), not inside insula. Only corenius's own `shell` tool goes through
  `InsulaSandbox` (`terminal_agent/src/cli.rs:510-521`).
- **[V]** Insula plan: rootfs `--ro-bind` to `/`, the whole workspace `--bind` read-write at
  `/workspace` (`terminal_agent/src/insula.rs:177-182`), `--unshare-pid` only (no `--unshare-net`, no user/ipc unshare;
  `insula.rs:174-200`). There is no per-path write scope today, and the plan is built per shell command,
  not per provider.
- **[V]** The Codex provider is not ACP: it calls the Responses endpoint directly and runs tools through
  corenius's executor (`terminal_agent/src/provider/codex.rs:65,116-118,268`). Its writes are as gateable as our tool
  registry (which has no write tool; `shell` is policy-`Ask`, `policy.rs:32-33`). Only TraeCLI is ACP.
- **[V]** `scripted_acp.rs` (`terminal_agent/tests/support/scripted_acp.rs:54-59`) scripts only text, error, stall and
  malformed-line steps; it cannot emit agent requests, so this spike added its own fixture inside
  `tests/interception.rs`.

## 2. ACP surface available locally

No ACP schema/docs are checked out; no network used. Best local evidence is the installed TraeCLI
binary (`~/.local/share/traex/releases/0.207.1-linux-x86_64.Cduewb/traex`), by `grep -a` on its strings:

- **[V]** It statically embeds `agent-client-protocol 2.1.0` and `agent-client-protocol-schema 1.7.0`
  (source paths in the binary), so it is an ACP agent built on the reference crate.
- **[V]** Strings present: `FileSystemCapabilities`, `read_text_file`/`write_text_file` (and camelCase
  `readTextFile`/`writeTextFile`), `session/request_permission`, `session/new`, `session/load`,
  `session/prompt`, `session/cancel`, `session/set_mode`, `ToolKind`, `toolCallId`, `allow_once`,
  `allow_always`, `reject_once`, `reject_always`, a `terminal` capability field.
- **[U]** Wire method names `fs/read_text_file`, `fs/write_text_file`, `terminal/*` and the exact
  request/response shapes come from my knowledge of the public spec, not a local file. The fixture uses
  those names; a live probe would confirm TraeCLI does.
- **[U]** Whether TraeCLI, when the client advertises `fs.writeTextFile`, routes its *edit tool* through
  `fs/write_text_file` (versus writing locally), and whether it emits `session/request_permission` before
  edits or shell commands. Spec-wise the fs methods are the client's *opportunity* to own file I/O
  (for unsaved editor buffers), not an obligation on the agent's own shell tool.
- **[U]** Codex-over-ACP is not what corenius uses (it uses HTTP), so nothing to check there.

## 3. Experiment (offline, `cargo test --offline --test interception`, 3 passed)

Fixture agent (Python, inline): sends `fs/write_text_file` for `claimed.txt`, sends
`session/request_permission`, waits 1 s per answer, then writes `bypass.txt` itself.

| Test | Shows |
|---|---|
| `shipped_client_advertises_nothing_and_answers_nothing` | corenius advertises `{}`, gives the agent no reply to either request; `claimed.txt` never written, `bypass.txt` written, corenius has no record of it |
| `protocol_client_can_observe_and_deny_fs_write_but_not_direct_write` | a wire-level client (test code, not corenius) that advertises `fs.writeTextFile` sees both requests with full path and content, denies the claimed path with a JSON-RPC error and rejects the permission; the denied write never lands; the direct write still does and is not among the observed calls |
| `agent_request_reusing_prompt_id_ends_turn_early` | the id-3 bug above |

Caveat: the second test proves the protocol seam works for a **cooperative fixture** that honors denial
and chooses to send the requests. It says nothing about TraeCLI's behavior.

### Sandbox experiment (real `bwrap 0.8.0`, `userns-ok`, run in the scratchpad)

- **[V]** `--ro-bind / / --bind <repo>/allowed <repo>/allowed`: write in `allowed/` ok, writes elsewhere in
  the repo fail with `Read-only file system`, `/tmp` scratch (tmpfs) ok.
- **[V]** File-level binds work: only the bound file is writable; sibling creates and `rm` of the bound
  file fail `EROFS`. Consequence **[V by reasoning]**: editors/agents that write temp+rename cannot
  update a file-bound path, so grant directories or use a per-claim scratch dir, not single files.
- **[V]** bwrap 0.8.0 lacks `--overlay-src` (added later), so a copy-on-write overlay is not available on
  this host as installed. Alternative: give the agent a private git worktree (writable) and merge
  through the lander, which fits the ledger design better than an overlay.
- **[V]** Unsandboxed writes show up in `git status --porcelain` (untracked dirs listed).
- inotify/fanotify: `inotifywait` not installed, fanotify tools absent; `/proc/sys/fs/inotify/*` exists.
  **[U]** Not exercised. inotify needs one watch per directory and races with fast create-then-delete;
  fanotify (`FAN_MARK_FILESYSTEM`) needs CAP_SYS_ADMIN, which the rootless bwrap setup lacks.

## 4. Enforcement options, ranked

1. **Sandbox path allowlist (enforced, covers all tools).** Run the provider process (not just `shell`)
   inside a bwrap plan: ro rootfs, ro workspace, `--bind` only the claimed directories (or a private
   worktree), tmpfs `/tmp` and `$HOME`. Kernel-enforced regardless of which tool the agent uses;
   violations surface as EROFS to the agent, which usually reports it in its turn. Costs: new claims need
   a restart of the provider process (bind set is fixed at spawn; ACP `session/load` may restore context
   **[U]**); the agent needs credentials/config/network, so the plan must bind `~/.trae*` (writable state)
   and drop `--unshare-net` only as needed; TraeCLI must run inside the insula rootfs (node/glibc
   compatibility **[U]**). Gaps: only path-level, not content-level; network/side effects outside files
   stay open (no `--unshare-net` today, `insula.rs:174-200`).
2. **Post-turn diff and reject (detect, cheap, always available).** After each turn, `git status` /
   `git diff --name-status base..worktree` and compare against the claim footprint; violation reverts
   (`git checkout`/`clean` in the agent's own worktree) or fails the landing. Enforced at the *landing*
   boundary, not the write boundary: the harm window is the turn. Needs each agent in its own worktree
   (the workflow already requires it). Blind spots: ignored files, writes outside the repo, and
   transient create-then-delete.
3. **Protocol gate (enforced only for calls that use it).** Advertise `fs.writeTextFile` and
   `readTextFile`, implement both plus `session/request_permission`, and check claims in the handler;
   also fixes the unanswered-request hang. Precise and cheap, gives real-time claim conflict detection
   with reasons the agent can read. But the provider decides whether its edit path goes through it, and
   its shell tool bypasses it entirely **[U for TraeCLI]**. Treat as a signal and a UX layer, never as
   the enforcement point.
4. **Watchers (inotify/fanotify).** Real-time detection, more complex than 2, weaker than 1; only useful
   to name *when* a violation happened. Skip until 2 proves too slow.

## 5. Recommendation: enforced vs advisory

| Mechanism | Claim status |
|---|---|
| Codex provider (corenius executor, `PolicyGate`) | **Enforced** for any write tool we add, since we execute it; today no write tool exists |
| ACP provider inside bwrap with claimed dirs writable | **Enforced** (kernel) for file writes under the workspace |
| ACP provider with `fs/write_text_file` handled | Enforced for calls made through it; **advisory** overall until [U] above is probed |
| ACP provider unsandboxed (today) | **Advisory**; only post-turn diff detects violations |
| Network, external processes, writes outside bound paths | Not covered by any mechanism above |

Order of work: (a) make the ACP client not drop requests (answer unknown methods with JSON-RPC
`-32601`, key responses on `method` absent, fixing `acp.rs:247`); (b) post-turn diff against the
claim footprint as the always-on backstop; (c) `spawn provider in insula` with claim-derived binds as
the enforced tier; (d) implement `fs/*` + permission handlers for observability and early conflict
messages. Until (c) ships, call claims **advisory with post-turn detection**.

## What a live probe would need (not run; costs money, has side effects)

Run `traecli acp serve` under `bwrap` in a throwaway git repo with a corenius build that logs every
inbound JSON-RPC line and answers with `-32601`, prompt "create foo.txt and run `touch bar.txt`", once
with `clientCapabilities.fs.writeTextFile=true` and once `{}`; record which requests arrive, whether
`foo.txt`/`bar.txt` appear, and the EROFS behavior under the allowlist plan. That settles every
**[U]** in sections 2 and 4.
