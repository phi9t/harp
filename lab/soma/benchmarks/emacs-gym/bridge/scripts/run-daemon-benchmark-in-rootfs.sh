#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-/workspace/soma/benchmarks/emacs-gym/bridge/traces/latest-daemon}"
DAEMON_NAME="${SOMA_BRIDGE_DAEMON_NAME:-soma-gym-bridge-daemon}"
BRIDGE_SOCKET="${SOMA_BRIDGE_DAEMON_SOCKET:-/project/run/soma-gym-bridge-daemon.sock}"
READY_JSON="${SOMA_BRIDGE_DAEMON_READY:-$OUT_DIR/daemon-ready.json}"
BUFFER_NAME="${SOMA_BRIDGE_DAEMON_BUFFER:-soma-bridge-daemon-target}"
TMUX_SOCKET="${SOMA_BRIDGE_TERMINAL_TMUX_SOCKET:-/project/run/soma-gym-bridge-terminal-tmux.sock}"
TMUX_SESSION="${SOMA_BRIDGE_TERMINAL_TMUX_SESSION:-soma-gym-bridge-terminal}"
TERMINAL_CAPTURE="$OUT_DIR/terminal-frame-pane.txt"
TERMINAL_KEY_CAPTURE="$OUT_DIR/terminal-key-dispatch-pane.txt"
DAEMON_PID=""
TERMINAL_FRAME_BASELINE=""

die() { printf 'error: %s\n' "$*" >&2; exit 1; }

daemon_exited() {
  if ! kill -0 "$DAEMON_PID" >/dev/null 2>&1; then
    return 0
  fi
  [[ -r "/proc/$DAEMON_PID/stat" ]] || return 1
  [[ "$(awk '{print $3}' "/proc/$DAEMON_PID/stat" 2>/dev/null)" == "Z" ]]
}

[[ "${SOMA_INSULA_ACTIVE:-}" == "1" ]] || die "must run inside the selected Soma Insula"
[[ "${SOMA_INSULA_MODE:-}" == "selected-container-rootfs" ]] || die "unexpected SOMA_INSULA_MODE=${SOMA_INSULA_MODE:-}"

mkdir -p "$OUT_DIR" /project/run /project/logs

cleanup() {
  set +e
  local kill_client_pid=""
  tmux -S "$TMUX_SOCKET" kill-session -t "$TMUX_SESSION" \
    >"$OUT_DIR/terminal-frame-tmux-kill.out" 2>"$OUT_DIR/terminal-frame-tmux-kill.err"
  if [[ -n "$DAEMON_PID" ]]; then
    emacsclient -s "$DAEMON_NAME" --eval "(kill-emacs 0)" \
      >"$OUT_DIR/daemon-kill.out" 2>"$OUT_DIR/daemon-kill.err" &
    kill_client_pid="$!"
    for _ in $(seq 1 50); do
      if daemon_exited; then
        wait "$kill_client_pid" >/dev/null 2>&1
        wait "$DAEMON_PID" >/dev/null 2>&1
        return
      fi
      if ! kill -0 "$kill_client_pid" >/dev/null 2>&1; then
        wait "$kill_client_pid" >/dev/null 2>&1
        break
      fi
      sleep 0.1
    done
    if [[ -n "$kill_client_pid" ]] && kill -0 "$kill_client_pid" >/dev/null 2>&1; then
      kill "$kill_client_pid" >/dev/null 2>&1
      wait "$kill_client_pid" >/dev/null 2>&1
    fi
    kill "$DAEMON_PID" >/dev/null 2>&1
    for _ in $(seq 1 20); do
      if daemon_exited; then
        wait "$DAEMON_PID" >/dev/null 2>&1
        return
      fi
      sleep 0.1
    done
    kill -9 "$DAEMON_PID" >/dev/null 2>&1
    wait "$DAEMON_PID" >/dev/null 2>&1
  fi
}
trap cleanup EXIT

emacs --quick --fg-daemon="$DAEMON_NAME" \
  >"$OUT_DIR/daemon-emacs.log" 2>&1 &
DAEMON_PID="$!"

daemon_ready=0
for _ in $(seq 1 100); do
  if emacsclient -s "$DAEMON_NAME" --eval "(daemonp)" \
      >"$OUT_DIR/daemon-probe.out" 2>"$OUT_DIR/daemon-probe.err"; then
    daemon_ready=1
    break
  fi
  sleep 0.1
done
[[ "$daemon_ready" == "1" ]] || die "timed out waiting for Emacs daemon $DAEMON_NAME"

export SOMA_BRIDGE_DAEMON_SOCKET="$BRIDGE_SOCKET"
export SOMA_BRIDGE_DAEMON_READY="$READY_JSON"
export SOMA_BRIDGE_DAEMON_BUFFER="$BUFFER_NAME"
export SOMA_BRIDGE_DAEMON_TEXT="${SOMA_BRIDGE_DAEMON_TEXT:-daemon bridge buffer
}"

emacsclient -s "$DAEMON_NAME" \
  --eval "(load-file \"/workspace/soma/benchmarks/emacs-gym/bridge/elisp/daemon-start.el\")" \
  >"$OUT_DIR/daemon-bootstrap.out" 2>"$OUT_DIR/daemon-bootstrap.err"

socket_ready=0
for _ in $(seq 1 100); do
  if [[ -S "$BRIDGE_SOCKET" && -f "$READY_JSON" ]]; then
    socket_ready=1
    break
  fi
  sleep 0.1
done
[[ "$socket_ready" == "1" ]] || die "timed out waiting for bridge socket $BRIDGE_SOCKET"

terminal_frame_count_expr='(let ((n 0)) (dolist (frame (frame-list) n) (unless (display-graphic-p frame) (setq n (1+ n)))))'
TERMINAL_FRAME_BASELINE="$(
  emacsclient -s "$DAEMON_NAME" --eval "$terminal_frame_count_expr" \
    2>"$OUT_DIR/terminal-frame-baseline.err" | tr -dc '0-9'
)"
[[ -n "$TERMINAL_FRAME_BASELINE" ]] || die "could not read baseline terminal frame count"

tmux -S "$TMUX_SOCKET" new-session -d -s "$TMUX_SESSION" \
  "/bin/bash -lc 'TERM=xterm-256color exec emacsclient -s \"$DAEMON_NAME\" --tty --alternate-editor=false'" \
  >"$OUT_DIR/terminal-frame-tmux.out" 2>"$OUT_DIR/terminal-frame-tmux.err"

terminal_ready=0
for _ in $(seq 1 100); do
  current="$(
    emacsclient -s "$DAEMON_NAME" --eval "$terminal_frame_count_expr" \
      2>"$OUT_DIR/terminal-frame-wait.err" | tr -dc '0-9'
  )"
  if [[ -n "$current" && "$current" -gt "$TERMINAL_FRAME_BASELINE" ]]; then
    terminal_ready=1
    break
  fi
  sleep 0.1
done
[[ "$terminal_ready" == "1" ]] || die "timed out waiting for emacsclient terminal frame"
tmux -S "$TMUX_SOCKET" capture-pane -t "$TMUX_SESSION":0.0 -p \
  >"$TERMINAL_CAPTURE" 2>"$OUT_DIR/terminal-frame-capture.err"

python3 /workspace/soma/benchmarks/emacs-gym/bridge/scripts/daemon-client.py \
  --socket "$BRIDGE_SOCKET" \
  --out-dir "$OUT_DIR" \
  --plan "${SOMA_INSULA_PLAN:-}" \
  --ready "$READY_JSON" \
  --daemon-name "$DAEMON_NAME" \
  --terminal-frame-baseline "$TERMINAL_FRAME_BASELINE" \
  --terminal-tmux-socket "$TMUX_SOCKET" \
  --terminal-tmux-session "$TMUX_SESSION" \
  --terminal-pane-capture "$TERMINAL_CAPTURE" \
  --terminal-key-pane-capture "$TERMINAL_KEY_CAPTURE" \
  --buffer-name "$BUFFER_NAME"
