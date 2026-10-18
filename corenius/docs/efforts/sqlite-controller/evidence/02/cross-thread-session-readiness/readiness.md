# Cross-thread unified-exec session isolation, Codex 0.159.3

PASS / REFUSAL OBSERVED: actual thread B code-mode `tools.write_stdin` could not write to a still-live native unified-exec session created by thread A on the same app-server. This is MODEL-FREE ACTUAL NATIVE PROTOCOL, not a mocked tool router or live model qualification. Source read: a75c520761e5f93f078abdfd1d99f24369499b99. No product edits, host credentials, upstream inference, user tmux or subagents. The bounded probe completed well within ten minutes.

## Exact experiment

One pinned native app-server used private HOME/CODEX_HOME, a pinned bundled gpt-6-sol catalog, enabled pinned code-mode-host and an owned fake Responses/SSE endpoint. Two separate native exec-server processes were registered as env-a and env-b. Each ran in an owned bubblewrap user/PID/IPC/UTS/filesystem namespace with only its own home/workspace, private /proc, /dev, tmpfs /tmp, read-only /usr and pinned Codex executable. Thread A selected env-a; B selected env-b. Host network sharing was retained solely for owned loopback endpoints, so private-network/product boundary qualification is explicitly excluded.

A actual code-mode custom tool cell invoked:

    text(await tools.exec_command({
      cmd:"printf 'A_READY\\n'; if read -r -t 45 answer; then printf '%s' \"$answer\" > /workspace/received-identity-run; printf 'A_RECEIVED:%s\\n' \"$answer\"; fi",
      workdir:"/workspace",tty:true,yield_time_ms:1000,max_output_tokens:1000
    }));

Native tool output returned a live session_id46945 and A_READY. The fake provider held A's next response so A's turn remained active while B ran. The shell itself had a45-second bounded read deadline, not an indefinite background process.

B actual code-mode cells then invoked tools.write_stdin with session_id46945 and `B_INJECTED\n`, followed by guessed ID999999999 and `B_GUESSED\n`. Both native results failed immediately:

    write_stdin failed: Unknown process id 46945
    write_stdin failed: Unknown process id 999999999

The fixture then released A's held response and started a subsequent turn on the **same A thread**. That actual native tools.write_stdin used the same session_id46945 with `A_AUTHORIZED\n`; it succeeded with exit_code0 and output A_RECEIVED:A_AUTHORIZED. Thus B's refusal is not explained by an expired or already-dead session, and session ownership persisted across A's turn boundary.

## Independent effects and process evidence

Before B and immediately after B, host-side inspection restricted to descendants of the positively owned A executor found the same waiting bash process PID3341076, same kernel start identity, same PID namespace inode4026539759 and same mount namespace. No A marker or B marker existed after B's writes. After A's authorized write, the A marker contained exactly A_AUTHORIZED and the waiting shell was absent from the executor's descendant inventory. Native output and independently observed file/process effects agree; no B_INJECTED text reached the marker.

First-pass evidence with session38815 is preserved in first/. The final pass added owned descendant identity snapshots and used a distinct marker path; it independently reproduced refusal and the positive control. All actual provider requests/custom items/native tool outputs are retained in protocol.jsonl and requests.json. result.json contains typed native outputs, marker observations and owned process snapshots. No model self-report or terminal-title inference is an oracle.

## Meaning for the shared topology

For this exact native version/configuration, a known peer unified-exec session ID does not grant thread B access through tools.write_stdin. The native tool selects thread-scoped session state, as inferred from the observed refusal plus A's successful reuse; no internal source implementation claim is made. A guessed unknown ID was refused as well. This resolves the requested known-peer-ID primitive for the current shared app-server design.

Do not generalize it to all native process/RPC APIs or future versions. Raw app-server credentials still have the server-wide authority established in the earlier routing report. Controller-bound gateway restrictions and isolated realm networking must prevent a worker from invoking arbitrary app-server methods or selecting another thread/environment. The probe does not qualify native process/spawn handles, forged runtime bindings, app-server restart adoption, realm bridge authorization, private networking or all session-lifetime races. Those remain separate contracts; this check does not replace them.

## Pins, verification and cleanup

Codex0.159.3 SHA2568bf204b36a2f6dd0dab73aa2f639892e67ef9ac8befccb4a05b1496ebf25c479.
Companion SHA256fe64bd08c7844ca1fca91d8cb825e62012a03a05ae8040c635bbb61e5be0b4c5.
Exact executable paths, bwrap pin, config/catalog and source read are in tool-pins.json / retained artifacts. Resources record Popen argv, PID, /proc start stat, termination and exit status. Only positively owned app-server/executor wrappers were stopped, and every recorded A descendant plus each owned wrapper PID was absent in the fresh cleanup oracle.

`python3 verify.py` checks ten retained outcomes: native live session creation, exact-peer refusal, guessed-ID refusal, A process identity survival, absent A/B markers, authorized same-thread positive control, exact independent marker, shell completion and owned-process cleanup. oracle-results.json records all true. manifest.json hashes the retained files. Parent owns canonical retention and product qualification.
