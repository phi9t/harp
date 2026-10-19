# pi-env source review for Corenius

Read on 2026-10-06 at upstream commit **ae92585d3b3e5f1e4b123d14a34314d826d8d9f5**, repository `earendil-works/pi`. Corenius comparison baseline: `d7f674829e9844d011869b711d64fcf55313f7f1`. This is a research spike, not an integration design approval. **Source-verified** means traced through implementation; **probe-verified** means exercised locally below; **inference** marks an implication rather than an upstream guarantee. Links are immutable to the examined commit. No production endpoint or shared upstream checkout was modified.

## Finding

**pi-env is a useful remote file/command adapter, but does not supply durable detached execution or a sandbox.** Its explicit lifecycle is one daemon per live transport: EOF or inbound silence kills tracked running process groups; subsequent requests start a fresh daemon. Inflight mutations have unknown outcomes, and handles cannot be reused after reconnect. Durable orchestration must live above this layer. The implementation itself documents mutation uncertainty. [Connection lifecycle](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/connection.ts#L164-L225), [daemon lifecycle](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/main.rs#L438-L488).

The most important qualification is stronger than “disconnect kills commands”: on Linux, daemon SIGKILL, a descendant moving into a new session, or a background child whose original shell has already settled can leave work alive. All three cases were probe-verified. Normal EOF killed a still-active ordinary command in the same probe. This is process-group cleanup, not crash containment or proof that every descendant is gone. [Unix process setup and kill](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/sys/unix.rs#L250-L296), [group tracking and removal](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/exec.rs#L286-L323), [settlement](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/exec.rs#L372-L449).

## Architecture and API map

```text
Durable worker and custom tool JavaScript (local process)
  -> RemoteExecutionEnv (Node-compatible filesystem/shell semantics)
    -> Connection (framing, lazy start, request correlation, session fencing)
      -> child process stdin/stdout, normally system ssh
        -> pi-env serve (Rust, on execution machine)
          -> files, process groups, watchers, temporary output files
```

The local/remote split keeps the Durable worker's storage and credentials local only insofar as applications keep those credentials there; it does not restrict custom tool code. Durable obtains an environment and calls `tool.execute(args, {...api, env}, context)` as ordinary local JavaScript. A custom tool may use Node filesystem/network/process APIs directly instead of `env`. [Package overview](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/README.md#L1-L10), [tool invocation](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/tool.ts#L251-L273).

| Surface | Concrete responsibilities / seam |
|---|---|
| `Connection` | Public, transport-independent launch command or asynchronous command factory; `info`, `session`, `request`, `kill`, `close`. A wrapper can place the daemon inside an approved Insula before `serve --token …` is appended. |
| `RemoteExecutionEnv` | Implements Durable's `ExecutionEnv`: paths, binary/text/line readers, directory readers, writes/appends, truncate/flush/rename/remove, temp files, watchers, and shell/argv execution. Options: connection, namespace `id`, `cwd`, shell path/environment, watch configuration. |
| `sshConnection` / `connectSsh` | Lazy / eager detection and deployment; strict host-key checking; package binary lookup; reconnect verifies daemon content again. |
| `RemoteWatcher` | Reopens watch after connection loss and reports missed coverage as `overflow`. |
| Rust `main.rs` | RPC dispatcher, handles, cancellation registration, fixed file worker pool, per-exec/watch threads, liveness and shutdown. |
| Rust `exec.rs`, `window.rs`, `output.rs` | Process setup/settlement, optional complete spill, output coalescing/skips, priority queues. |
| Rust `fs.rs`, `scan.rs`, `decode.rs`, `watch.rs`, `sys/*` | Node semantic compatibility, streaming scans/decoding, snapshots and native/polling watches, OS-specific system calls. |

Sources: [exports](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/index.ts#L1-L28), [environment options](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/remote-env.ts#L48-L58), [dispatch](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/main.rs#L130-L257), [SSH construction](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/ssh.ts#L454-L530).

## Protocol and operational behavior

Protocol v1 uses `u32 length | u8 type | u32 requestId | u32 jsonLength | JSON | payload`, big-endian, maximum 16 MiB frame. Requests, results, errors, events, cancellation, and pings share the pipe. A random 16-byte client token matches the daemon's initial stdout marker to discard shell banners. **The token is synchronization, not authentication:** it is passed on argv and printed; daemon dispatch does not validate a caller credential. SSH or a trusted local launcher supplies authentication. `hello` returns protocol/version/platform/home/tmpdir/pid; the client checks protocol 1, while the daemon ignores the requested protocol field. The latter was verified with `hello {protocol:999}` returning 1. No negotiated operation capabilities or durable operation identifier exists in this protocol. [Protocol](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/docs/protocol.md#L1-L61), [client startup](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/connection.ts#L255-L304), [hello implementation](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/main.rs#L137-L154).

File calls use 16 worker threads, at most 4096 open handles; each exec/watch starts its own thread with a 16 MiB stack reservation. Chunk writes to the same handle are serialized in arrival order, and failed writes poison that handle. Queues are not an admission-controlled job scheduler: the file queue is an unbounded Rust channel; there is no explicit exec concurrency budget. Cancellation is registered before dispatch so an immediate cancel is not lost. Most ordinary filesystem calls do not consult cancellation once running; blocking FIFO reads can occupy workers. **Inference:** embedding needs an external concurrency/resource policy and should not promise every file operation is promptly cancellable. [Limits and handles](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/main.rs#L30-L76), [worker scheduling](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/main.rs#L329-L435), [blocking open](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/main.rs#L187-L215).

Small results, watch events, and pings are queued ahead of bulk output; exec terminal results deliberately remain behind output. Unwindowed readers pause above a 4 MiB unsent bulk threshold. This is a scheduling preference, not independent transport lanes or a strict total memory bound: a frame already blocked on stdout cannot be preempted, and other queues/messages remain. Windowed output preserves a tail and explicit skipped-byte/newline counts; it is not a complete log. Optional spill stores complete raw interleaved output in a remote temporary file after thresholds are crossed, without per-stream metadata. It has no replay cursor, output acknowledgement, automatic artifact upload, or archival lifecycle. [Output writer](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/output.rs#L13-L100), [terminal ordering](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/main.rs#L347-L359), [window](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/window.rs#L1-L25), [spill](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/exec.rs#L129-L188).

String commands use a shell; argv executes a program directly. Ordinary command stdin is null, and no interactive stdin or PTY operation is exposed. Timeout/abort kills immediately, without a graceful TERM phase. Pipes drain after leader exit until EOF or 100 ms without output; timeout continues during drain. Client callback failure outranks daemon status; daemon precedence is timeout, abort, spill error, then exit status. `cleanup()` sends kill frames and clears its map **without awaiting process exit**; it does not close the shared Connection. This matters if “cleanup complete” is treated as permission to reuse a worktree. [Execution client](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/remote-env.ts#L834-L935), [stdin and spawn](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/exec.rs#L191-L211), [settlement precedence](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/exec.rs#L372-L482).

## Durability, retries, and recovery

| Event | Source-verified behavior | Consequence for a supervisor |
|---|---|---|
| Transport start fails | Waiting requests fail; next request recomputes command and retries start. | Startup retry is available; operation replay is not. |
| Transport lost mid-operation | Pending calls reject with `RemoteError` code `unknown`, `lost:true`; no journal/query API records whether side effects occurred. | Reconcile before retrying non-idempotent writes or commands. |
| Reconnect | New daemon/session, new handle map; old session-pinned requests fail. | Reopen readers explicitly; cannot reattach to existing exec. |
| Duplicate completed request id | Dispatcher executes operation again; probe appended `x` twice using id 7. | Request ids correlate responses; they are not idempotency keys. |
| Watch disconnect | Reopen with 1–30 second exponential delay; emit overflow after ready. | Refresh a snapshot, not replay a supposedly complete event history. |
| Client stops sending bytes | Daemon checks every 5 seconds and kills tracked groups after >30 seconds. | A long partition terminates work by design; clock/check timing means this is not an exact 30-second deadline. |
| Daemon crash / completed shell with background descendants | Normal cleanup path is bypassed / group removed on settlement. Linux probes found survivors. | Need an independently owned Insula/cgroup/process-lifetime boundary. |

Sources: [request/session handling](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/connection.ts#L190-L248), [teardown](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/connection.ts#L307-L320), [watch recovery](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/watch.ts#L114-L156), [liveness](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/main.rs#L459-L488).

File writes create/truncate or append directly, then stream 512 KiB chunks (up to eight inflight). This is not atomic replacement; cancellation/failure can leave a prefix. `flushFile` exists separately, but ordinary writes do not fsync. Rename exists, but the exposed fsync rejects directories, so the RPC surface does not by itself supply the usual directory-fsync step for crash-durable publish. Remote files surviving a daemon restart is filesystem persistence, not exactly-once command execution. [Client writes](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/remote-env.ts#L583-L678), [write/flush/rename](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/fs.rs#L107-L161).

## Security and Insula boundary

`cwd` is a resolution base, not a filesystem root. Absolute paths are accepted; `..` resolves normally; daemon filesystem operations use the supplied paths under the daemon user's OS authority. `noFollow` is an optional final-component read protection, not a root/path allowlist and not a guard on writes. `id` declares file namespace identity to Durable; it does not enforce access. Environment inheritance defaults to true: daemon environment, then configured `shellEnv`, then per-call overrides. [Path resolution](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/remote-env.ts#L370-L399), [Unix open semantics](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/sys/unix.rs#L110-L137), [environment merge](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/remote-env.ts#L850-L878).

**Integration inference:** if explored, launch the whole daemon **inside the approved Insula boundary** so filesystem RPCs and subprocesses share the approved mounts, network, environment, and lifetime. Merely replacing its shell command with `bwrap` leaves direct read/write/rename/remove RPCs outside confinement. The local Durable/custom-tool process also remains a separate authority surface. Corenius's permission decisions, allowed-path plan and Rust SQLite controller must remain authoritative; pi-env supplies none of their ticket/claim/attempt fencing. No claim is made that Corenius's complete live two-worker controller baseline is already qualified.

SSH uses system OpenSSH with BatchMode, strict checking against the application's known-hosts file under a stable alias, no global known-hosts, no forwarding, no connection sharing, no configured remote/local commands, and no locale forwarding. Optional identity file adds IdentitiesOnly. It still supports user SSH config/aliases and jump routes; it is not a hermetic replacement for SSH configuration. Host scanning temporarily uses `accept-new`, returns fingerprints, and explicitly requires out-of-band verification before acceptance. `acceptHostKey` rejects changed same-type keys; rewriting is atomic and serialized **within one JS process**, not protected against multiple processes updating the file concurrently. [SSH arguments](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/ssh.ts#L59-L120), [trust workflow](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/ssh.ts#L177-L285).

Deployment hashes the locally packaged binary, checks the remote full SHA-256, uploads to a temporary file, verifies, chmods 0700 on POSIX, then renames. Names use the first 32 hex characters of the hash. Reconnect verifies again. It removes older `pi-env-*` binaries in the shared remote `~/.pi/mobile/tools` directory when uploading: **inference**, concurrent versions/deployments sharing an account can race between verification and launch or delete one another's not-yet-started binaries. Hash verification protects against accidental/tampered content relative to the trusted local artifact, not a compromised remote account or malicious SSH server. There is no signed artifact verification in this module. [Deployment](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/ssh.ts#L369-L451), [verification on start](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/ssh.ts#L480-L501).

Windows uses Job Objects and asynchronous `taskkill /F /T`, with breakaway flags and unchecked job-assignment return values. Those paths were examined statically, not validated here; Linux probe results must not be generalized into identical Windows guarantees. [Windows lifecycle](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/src/sys/windows.rs#L519-L563).

## Packaging, maturity, and upstream verification

The package manifest says `@earendil-works/pi-env` 1.0.4, Node >=22.19.0, ESM, runtime dependencies on `@earendil-works/chord` and `@earendil-works/pi-durable` ^1.0.4. The Rust daemon is a non-published binary package at 0.1.0, edition 2024, with pinned encoding_rs, notify, serde_json, libc/Windows bindings. `build.rs` derives reported version from adjacent npm package.json. Toolchain file pins Rust 1.96.0. **These are source metadata, not a registry publication or platform certification claim.** [npm manifest](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/package.json#L1-L55), [Cargo manifest](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/Cargo.toml#L1-L29), [toolchain](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/daemon/rust-toolchain.toml#L1-L3).

The initial-release changelog entry is dated 2026-10-05, one day before this review. Thus version 1.0.4 should not be interpreted as long production history for this new package. Root license is MIT with copyright notice retention required. The release workflow builds Linux musl, macOS, Android, Windows for x64/arm64; Windows static CRT avoids a redistributable dependency. Android requires Termux assumptions and may suspend without a wake lock. [Changelog](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/CHANGELOG.md#L1-L10), [license](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/LICENSE#L1-L21), [build matrix](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/.github/workflows/env-daemons.yml#L20-L67), [Termux warnings](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/src/ssh.ts#L317-L325).

Upstream test assets are substantial for semantic parity: shared Durable environment conformance; differential tests with 60 seeds ×25 file operations, special files and actual read/bash tools; dedicated output-window, timeout, immediate-abort, stale-handle, retry-start, large-transfer tests; local SSH host-trust/tampered-binary/reconnect/login-shell tests; external SSH conformance gated by environment variables. These tests demonstrate intended scope, not execution results from this review or evidence of detached durability. [Conformance registration](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/test/conformance.test.ts#L1-L30), [differential suite](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/test/differential.test.ts#L227-L361), [remote tests](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/test/remote.test.ts#L28-L137), [SSH tests](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/test/ssh.test.ts#L118-L264), [external gating](https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/env/test/ssh-external.test.ts#L24-L92).

## Reuse candidates and concrete seams

These are research recommendations, not approved architectural changes.

1. **First candidate: protocol/client behind an existing permissioned execution adapter.** Keep Corenius's durable attempt record, dispatch authority, reconciliation and evidence ownership. Use pi-env only for explicitly connection-scoped operations; represent transport loss as outcome-unknown. Its command factory is the seam for an approved Insula launcher. A Rust caller would need a protocol client or Node sidecar; this Rust binary does not expose a reusable library API.
2. **Smallest source reuse: output-window accounting, UTF-8 streaming/line scans, error parity, and conformance fixtures.** These are separable modules with direct tests. Preserve MIT notices and record divergence from Node semantics when transplanting. This avoids adopting the entire TS Durable stack merely for remote I/O.
3. **SSH bootstrap as a reference or adapted component.** Preserve explicit owner trust, private known-hosts, strict options, content verification. Replace shared deployment-directory cleanup if concurrent controller instances need isolated versions, and make Insula launch an explicit server-side authority decision.
4. **Watcher as an invalidation signal.** Overflow must trigger rescan; never use the event stream as the sole durable evidence log.
5. **Do not reuse pi-env as a detached job service unchanged.** Long-running supervised work needs stable attempt IDs, durable terminal evidence, fenced authority, explicit reconciliation after uncertain delivery, and lifetime containment independent of daemon health. Adding those changes the protocol/lifecycle contract and merits its own design.

These seams derive from the public launch options, private process tracking, dispatcher and output modules cited above. Before adoption, a bounded qualification should prove full-daemon Insula confinement including file RPCs, crash/partition/setsid cleanup, cancellation completion barriers, duplicate-side-effect reconciliation, spill retrieval, and slow-link/resource bounds. This is a proposed evidence set, not work performed in this research spike.

## Local verification evidence

All builds/probes ran in the research worktree's ignored `.scratch/pi-env-probes/`, using copies of the upstream `packages/env/daemon`, adjacent package.json, and `src/connection.ts`. The shared upstream tree remained read-only. Host was Linux; installed tools were Rust/Cargo **1.95.0** and Node **v24.13.0**. The explicit `+1.95.0` override differs from upstream's 1.96.0 pin; source was unchanged. No npm workspace install, full TS suite, SSH integration, macOS/Windows/Android run, or 30-second liveness experiment was performed.

Commands and observed results:

```text
cargo +1.95.0 test --manifest-path .scratch/pi-env-probes/packages/env/daemon/Cargo.toml --locked
  7 passed; 0 failed: decode (3), frame (1), scan (1), window (2)
cargo +1.95.0 build --manifest-path .scratch/pi-env-probes/packages/env/daemon/Cargo.toml --locked
  success
python3 .scratch/pi-env-probes/probe.py
  hello_wrong_version 1
  duplicate_id_append xx
  eof_kills_active True
  completed_parent_descendant_survives True
  daemon_sigkill_child_survives True
  setsid_descendant_survives True
node .scratch/pi-env-probes/client.mjs
  {"restarted":true,"staleHandleLost":true}
```

The Python probe encodes actual protocol frames to the built daemon. It sends two sequential append operations under the same id; starts an ordinary `sleep 30` command then closes stdin; runs a shell that backgrounds redirected `sleep 30` and exits before EOF; kills the daemon while an ordinary command runs; and starts a separate-session descendant before EOF. PID files synchronize startup; `/proc/<pid>/stat` distinguishes a live process from a zombie. Every surviving test process/group is explicitly SIGKILLed afterward. The Node probe imports the copied unmodified `Connection`, kills an idle daemon after opening a file, reconnects, then checks `isConnectionLost` for the old session's handle. These are bounded behavioral probes, not a security audit or exhaustive race proof.

### Reproducing the behavioral probes

From a clean copy of the pinned upstream checkout, create `.scratch/pi-env-probes/packages/env/`, copy `packages/env/daemon` and its sibling `package.json` there, and copy `packages/env/src/connection.ts` to `.scratch/pi-env-probes/connection.ts`. Run the build commands above. Save the following as `.scratch/pi-env-probes/probe.py` and `.scratch/pi-env-probes/client.mjs`, respectively, then run the commands above. These paths keep generated files and builds outside tracked documentation. The probe only launches local disposable commands; it does not contact SSH hosts.

```python
import subprocess,struct,json,os,time,pathlib,signal
root=pathlib.Path(__file__).parent.resolve(); binary=root/'packages/env/daemon/target/debug/pi-env'
def start():
 p=subprocess.Popen([str(binary),'serve','--token','probe'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE); assert p.stdout.readline()==b'PI-ENV probe\n'; return p
def send(p,i,obj,payload=b'',kind=1):
 j=json.dumps(obj).encode(); p.stdin.write(struct.pack('>IBII',9+len(j)+len(payload),kind,i,len(j))+j+payload); p.stdin.flush()
def read(p):
 n=struct.unpack('>I',p.stdout.read(4))[0]; b=p.stdout.read(n); t,i,l=struct.unpack('>BII',b[:9]); return t,i,json.loads(b[9:9+l]),b[9+l:]
def reply(p,i):
 while True:
  r=read(p)
  if r[1]==i and r[0] in (2,3):return r

def alive(pid):
 try: return pathlib.Path(f'/proc/{pid}/stat').read_text().split()[2]!='Z'
 except FileNotFoundError:return False
p=start(); send(p,1,{'op':'hello','protocol':999});print('hello_wrong_version',reply(p,1)[2]['protocol'])
f=root/'append.txt';f.unlink(missing_ok=True)
for _ in range(2):send(p,7,{'op':'write','path':str(f),'append':True},b'x');reply(p,7)
print('duplicate_id_append',f.read_text())
pidfile=root/'child.pid';pidfile.unlink(missing_ok=True);send(p,8,{'op':'exec','command':f'echo $$ > {pidfile}; sleep 30','cwd':str(root)})
while not pidfile.exists():time.sleep(.01)
pid=int(pidfile.read_text());p.stdin.close();p.wait(timeout=3);time.sleep(.1);print('eof_kills_active',not alive(pid))
p=start();pidfile.unlink();send(p,9,{'op':'exec','command':f'sleep 30 </dev/null >/dev/null 2>&1 & echo $! > {pidfile}','cwd':str(root)});r=reply(p,9);pid=int(pidfile.read_text());p.stdin.close();p.wait(timeout=3);time.sleep(.1);print('completed_parent_descendant_survives',alive(pid));os.kill(pid,signal.SIGKILL) if alive(pid) else None
p=start();pidfile.unlink();send(p,10,{'op':'exec','command':f'echo $$ > {pidfile}; sleep 30','cwd':str(root)})
while not pidfile.exists():time.sleep(.01)
pid=int(pidfile.read_text());p.kill();p.wait();time.sleep(.1);print('daemon_sigkill_child_survives',alive(pid));os.killpg(pid,signal.SIGKILL) if alive(pid) else None
p=start();pidfile.unlink();send(p,11,{'op':'exec','command':f'setsid sh -c \'echo $$ > {pidfile}; sleep 30\' </dev/null >/dev/null 2>&1 & sleep 30','cwd':str(root)})
while not pidfile.exists():time.sleep(.01)
pid=int(pidfile.read_text());p.stdin.close();p.wait(timeout=3);time.sleep(.1);print('setsid_descendant_survives',alive(pid));os.killpg(pid,signal.SIGKILL) if alive(pid) else None
```

```javascript
import { Connection, isConnectionLost } from './connection.ts';
import { fileURLToPath } from 'node:url';
const root=fileURLToPath(new URL('.',import.meta.url));
const c=new Connection({command:[root+'packages/env/daemon/target/debug/pi-env']});
const info=await c.info();
const h=await c.request('open',{path:root+'append.txt'});
process.kill(info.pid,'SIGKILL');await new Promise(r=>setTimeout(r,100));
const next=await c.info();let lost=false;
try {await c.request('pread',{handle:h.json.handle,length:2},{session:h.session});}catch(e){lost=isConnectionLost(e)}
console.log(JSON.stringify({restarted:info.pid!==next.pid,staleHandleLost:lost}));c.close();
```
