#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
RUN_DIR="$SCRIPT_DIR/run-bwrap"
WORKSPACE="$RUN_DIR/workspace"
LAYER="$RUN_DIR/layer"
rm -rf "$RUN_DIR"
mkdir -p "$WORKSPACE" "$LAYER"
printf '{"ok":"jq-ok"}\n' >"$WORKSPACE/input.json"
printf 'nested-layer-ok\n' >"$LAYER/marker"

write_arg_file() {
  local path="$1"
  shift
  printf '%s\n' "$@" >"$path"
}

base_bwrap_args=(
  --tmpfs /
  --unshare-user
  --unshare-ipc
  --unshare-pid
  --unshare-net
  --unshare-uts
  --unshare-cgroup-try
  --hostname corenius-bench
  --ro-bind /usr /usr
  --symlink usr/bin /bin
  --symlink usr/lib /lib
  --symlink usr/lib64 /lib64
  --symlink usr/sbin /sbin
  --dir /etc
  --dir /etc/ssl
  --ro-bind-try /etc/ld.so.cache /etc/ld.so.cache
  --ro-bind-try /etc/ssl/certs /etc/ssl/certs
  --file 3 /etc/passwd
  --file 4 /etc/group
  --file 5 /etc/nsswitch.conf
  --proc /proc
  --dev /dev
  --tmpfs /tmp
  --tmpfs /run
  --dir /cap
  --dir /cap/data
  --ro-bind "$LAYER" /cap/data/demo
  --tmpfs /home
  --dir /home/bench
  --bind "$WORKSPACE" /workspace
  --clearenv
  --setenv HOME /home/bench
  --setenv PATH /usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
  --setenv BENCH_ENV allow
  --setenv LC_ALL C
  --chdir /workspace
)

write_arg_file "$SCRIPT_DIR/root-tmpfs.argv.txt" \
  "bwrap" "${base_bwrap_args[@]}" \
  "/bin/sh" "-lc" \
  "python3 -V; jq -r .ok input.json; env | sort"

run_bwrap() {
  /usr/bin/bwrap "${base_bwrap_args[@]}" "$@" \
    3< <(printf 'bench:x:1000:1000:bench:/home/bench:/bin/sh\n') \
    4< <(printf 'bench:x:1000:\n') \
    5< <(printf 'hosts: files dns\npasswd: files\ngroup: files\n')
}

run_case() {
  local name="$1"
  shift
  printf '\n### %s\n' "$name"
  "$@"
  local status=$?
  printf 'exit=%s\n' "$status"
  return 0
}

run_case root_tmpfs_basic run_bwrap /bin/sh -lc '
set -u
echo "sh=$(command -v sh)"
python3 -V
jq -r .ok input.json
echo "cap_layer=$(cat /cap/data/demo/marker)"
printf workspace-ok >/workspace/write-test && echo "workspace_write=$(cat /workspace/write-test)"
if touch /usr/bin/insula-write-test 2>/tmp/usr-write.err; then
  echo "usr_write=unexpected"
else
  echo "usr_write=blocked:$(cat /tmp/usr-write.err)"
fi
echo "home_entries=$(ls -A /home | tr "\n" "," )"
if test -e /home/philip.yang; then echo host_home_visible=yes; else echo host_home_visible=no; fi
echo "route_lines=$(ip route | wc -l)"
echo "ps_count=$(ps -e -o pid= | wc -l)"
ps -e -o pid,ppid,comm | sed -n "1,8p"
'

run_case exact_env env -i SECRET_SHOULD_NOT_PASS=1 /usr/bin/bwrap "${base_bwrap_args[@]}" /usr/bin/env \
  3< <(printf 'bench:x:1000:1000:bench:/home/bench:/bin/sh\n') \
  4< <(printf 'bench:x:1000:\n') \
  5< <(printf 'hosts: files dns\npasswd: files\ngroup: files\n') \
  | sort

run_case nested_mount_under_ro_usr /usr/bin/bwrap \
  --tmpfs / \
  --ro-bind /usr /usr \
  --ro-bind "$LAYER" /usr/local/share \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --symlink usr/lib64 /lib64 \
  --symlink usr/sbin /sbin \
  --proc /proc \
  --dev /dev \
  --tmpfs /tmp \
  --clearenv \
  --setenv PATH /usr/bin:/bin:/usr/sbin:/sbin \
  /bin/sh -lc 'cat /usr/local/share/marker; if touch /usr/local/share/write-test 2>/tmp/layer.err; then echo nested_write=unexpected; else echo nested_write=blocked; fi'

run_case file_bind_loader /usr/bin/bwrap \
  --tmpfs / \
  --ro-bind /usr /usr \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --symlink usr/lib64 /lib64 \
  --symlink usr/sbin /sbin \
  --ro-bind /lib/x86_64-linux-gnu/ld-linux-x86-64.so.2 /ld-linux-x86-64.so.2 \
  --clearenv \
  --setenv PATH /usr/bin:/bin \
  /bin/sh -lc 'test -f /ld-linux-x86-64.so.2 && echo loader_file_bind=ok'

run_case net_loopback_only /usr/bin/bwrap \
  --tmpfs / \
  --ro-bind /usr /usr \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --symlink usr/lib64 /lib64 \
  --symlink usr/sbin /sbin \
  --proc /proc \
  --dev /dev \
  --tmpfs /tmp \
  --unshare-user \
  --unshare-net \
  --clearenv \
  --setenv PATH /usr/bin:/bin:/usr/sbin:/sbin \
  /bin/sh -lc 'ip link set lo up >/dev/null 2>&1 || true; echo route_lines=$(ip route | wc -l); python3 - <<'"'"'PY'"'"'
import socket, threading
s = socket.socket()
s.bind(("127.0.0.1", 0))
s.listen(1)
port = s.getsockname()[1]
ok = []
def accept():
    c, _ = s.accept()
    c.close()
    ok.append(True)
t = threading.Thread(target=accept, daemon=True)
t.start()
socket.create_connection(("127.0.0.1", port), timeout=1).close()
t.join(1)
print("loopback_ipv4=" + str(bool(ok)))
try:
    s6 = socket.socket(socket.AF_INET6)
    s6.bind(("::1", 0))
    s6.close()
    print("loopback_ipv6=True")
except OSError as e:
    print("loopback_ipv6=blocked:" + type(e).__name__)
try:
    socket.create_connection(("198.51.100.1", 9), timeout=1).close()
    print("non_loopback=unexpected")
except OSError as e:
    print("non_loopback=blocked:" + type(e).__name__)
PY'

run_case dev_bind_nvidia /usr/bin/bwrap \
  --tmpfs / \
  --ro-bind /usr /usr \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --symlink usr/lib64 /lib64 \
  --symlink usr/sbin /sbin \
  --proc /proc \
  --dev /dev \
  --dev-bind-try /dev/nvidiactl /dev/nvidiactl \
  --dev-bind-try /dev/nvidia-uvm /dev/nvidia-uvm \
  --dev-bind-try /dev/nvidia-uvm-tools /dev/nvidia-uvm-tools \
  --dev-bind-try /dev/nvidia0 /dev/nvidia0 \
  --clearenv \
  --setenv PATH /usr/bin:/bin \
  /bin/sh -lc 'ls -l /dev/nvidia* 2>/dev/null | sed -n "1,12p" || true'

run_case new_session /usr/bin/bwrap \
  --tmpfs / \
  --ro-bind /usr /usr \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --symlink usr/lib64 /lib64 \
  --symlink usr/sbin /sbin \
  --proc /proc \
  --dev /dev \
  --new-session \
  --clearenv \
  --setenv PATH /usr/bin:/bin \
  /usr/bin/python3 -c 'import os; print(f"pid={os.getpid()} sid={os.getsid(0)} same={os.getpid()==os.getsid(0)}")'

run_case cap_drop /usr/bin/bwrap \
  --tmpfs / \
  --ro-bind /usr /usr \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --symlink usr/lib64 /lib64 \
  --symlink usr/sbin /sbin \
  --proc /proc \
  --dev /dev \
  --unshare-user \
  --cap-drop CAP_NET_RAW \
  --clearenv \
  --setenv PATH /usr/bin:/bin \
  /bin/sh -lc 'grep "^CapEff:" /proc/self/status'

python3 - "$RUN_DIR/allow-all.bpf" <<'PY'
import struct
import sys
with open(sys.argv[1], "wb") as f:
    f.write(struct.pack("HBBI", 0x06, 0, 0, 0x7FFF0000))
PY

run_case seccomp_allow_all /usr/bin/bwrap \
  --tmpfs / \
  --ro-bind /usr /usr \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --symlink usr/lib64 /lib64 \
  --symlink usr/sbin /sbin \
  --clearenv \
  --setenv PATH /usr/bin:/bin \
  --seccomp 9 \
  /bin/sh -lc 'echo seccomp=allow_all_ok' 9<"$RUN_DIR/allow-all.bpf"

run_case add_seccomp_allow_all /usr/bin/bwrap \
  --tmpfs / \
  --ro-bind /usr /usr \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --symlink usr/lib64 /lib64 \
  --symlink usr/sbin /sbin \
  --clearenv \
  --setenv PATH /usr/bin:/bin \
  --add-seccomp-fd 9 \
  /bin/sh -lc 'echo add_seccomp=allow_all_ok' 9<"$RUN_DIR/allow-all.bpf"
