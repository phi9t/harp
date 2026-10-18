# Synthetic-only private Trae executable preflight

Commit read: `264756edd550225449202b6f20ac4b560808d14e`. Observed
2026-10-03 against pinned Trae0.207.1 SHA256
`9f5531a1599cd7d81df35a9b6a7568f6652fbf58141d33da9853a62046caf8c2`.
This prerequisite probe does not complete tickets04/20 or authorize adapter21.

Four private server incarnations ran inside independent bubblewrap PID/network
namespaces with cleared environments, the pinned binary, system libraries, and
only owned writable fixture/store paths. Shared user homes and `/etc` were absent.
All four reached terminal waits and empty namespace membership observations;
fixtures remain retained. Physical shutdown does not resolve the unsupported
turn's logical quarantine.

The first launch rejected `--no-daemon app-server`: this option supports only
interactive new/resume/fork. The second used an incorrect newline Unix codec.
Installed transport symbols and the third actual handshake established Unix
WebSocket transport. Initialize returned `/fixture/store`; config/read named
session overrides and private/absent system file layers. Explicit
`model_provider="lmstudio"` selected the local provider. A custom model name
failed metadata resolution before any provider request.

The fourth incarnation queried actual model/list, selected advertised `gpt-5.5`,
and started one turn. With the owned `CODEX_OSS_BASE_URL` ending `/v1`, Trae sent
`POST /v1/responses` to the synthetic endpoint. No synthetic GET discovery was
observed. Ordinary request keys omitted `max_output_tokens`. Synthetic streaming
was deliberately refused with501; the runtime automatically retried. Six requests
were accepted and three additional attempts were refused with429 before shutdown.
The final endpoint journal contains **nine attempts**, superseding the earlier
four-request snapshot. The six-request/no-automatic-retry envelope was not met.
No external or paid provider was accessible or contacted.

The turn remains unsupported/quarantined; no successful streaming, terminal
notification, hard token/cost ceiling, recovery, or adoption is claimed. The
retained owned rollout identifies turn `01a1009d-546d-7072-a482-5b3b5176adb4`;
only selected IDs/event kinds enter public evidence. A request-count ceiling
alone cannot establish a monetary/token ceiling. Embedded local strings mention
`request_max_retries`, `stream_max_retries`, and `max_output_tokens`; their qualified
configuration semantics remain hypotheses, requiring a newly reviewed executable
envelope before another launch. All four incarnation slots are spent. The
retained script now refuses launch before creating a fixture or process.

Public evidence selection is `.scratch/team-control-plane/trae20-preflight/freeze/`:
summary, launch arguments/environment names, store/config origins, request
hashes/key names, and process/terminal observations. Private stores, databases,
config values, credentials, raw request bodies and service logs are excluded.
First-three-launch script/capsule digests were not captured before execution;
this limitation is retained rather than reconstructed as observed evidence.
