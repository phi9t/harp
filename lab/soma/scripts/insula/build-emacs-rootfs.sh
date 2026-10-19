#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STORE_ROOT="${SOMA_INSULA_ROOTFS_STORE:-$ROOT/.scratch/emacs-gym/insula/rootfs-store}"
BASE_IMAGE="${SOMA_INSULA_BASE_IMAGE:-ghcr.io/pytorch/pytorch:2.13.0-cuda13.2-cudnn9-runtime@sha256:7492928e093d67276716440161f694a0e4ea27796d599d64b23cb76cdd665e71}"
TAG="${SOMA_INSULA_IMAGE_TAG:-soma-emacs-rootfs:local}"
EMACS_VERSION="${SOMA_EMACS_SOURCE_VERSION:-30.2}"
EMACS_SOURCE_SHA256="${SOMA_EMACS_SOURCE_SHA256:-}"
DOOM_COMMIT="${SOMA_DOOM_COMMIT:-8b475fcb2574286d7f4413c22cc7902d80d21ced}"
RUST_TOOLCHAIN="${SOMA_RUST_TOOLCHAIN:-1.81.0}"
PYRIGHT_VERSION="${SOMA_PYRIGHT_VERSION:-1.1.414}"
RECIPE_KIND="source-built-emacs-v8"
REBUILD=0
NO_CACHE=0

usage() {
  cat <<'EOF'
Usage: scripts/insula/build-emacs-rootfs.sh [options]

Build and select Soma's CUDA-image-derived Emacs/Doom rootfs.

The image build is the materialization phase. It may use network, apt, npm,
rustup, git, and GNU source downloads, and records the resulting rootfs
contract. Probe/benchmark execution later consumes the selected rootfs through
bwrap; it must not project host Emacs, Doom, LSP, Jupyter, or toolchain state.

Options:
  --rebuild        Force docker build even if TAG already exists.
  --no-cache       Pass --no-cache to docker build.
  --tag TAG        Docker image tag to build/export.
  --store DIR      Rootfs store directory.
  --base-image IMG CUDA-enabled base image reference.
  --emacs VERSION  GNU Emacs source release version.
  --emacs-sha256 H Expected SHA256 for the GNU Emacs source tarball.
  --doom-commit C  Doom Emacs commit to clone into the rootfs.
  -h, --help       Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --rebuild) REBUILD=1; shift ;;
    --no-cache) NO_CACHE=1; shift ;;
    --tag) TAG="${2:?--tag requires a value}"; shift 2 ;;
    --tag=*) TAG="${1#*=}"; shift ;;
    --store) STORE_ROOT="${2:?--store requires a value}"; shift 2 ;;
    --store=*) STORE_ROOT="${1#*=}"; shift ;;
    --base-image) BASE_IMAGE="${2:?--base-image requires a value}"; shift 2 ;;
    --base-image=*) BASE_IMAGE="${1#*=}"; shift ;;
    --emacs) EMACS_VERSION="${2:?--emacs requires a value}"; shift 2 ;;
    --emacs=*) EMACS_VERSION="${1#*=}"; shift ;;
    --emacs-sha256) EMACS_SOURCE_SHA256="${2:?--emacs-sha256 requires a value}"; shift 2 ;;
    --emacs-sha256=*) EMACS_SOURCE_SHA256="${1#*=}"; shift ;;
    --doom-commit) DOOM_COMMIT="${2:?--doom-commit requires a value}"; shift 2 ;;
    --doom-commit=*) DOOM_COMMIT="${1#*=}"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'error: unknown argument: %s\n' "$1" >&2; usage >&2; exit 2 ;;
  esac
done

log() { printf '\n== %s ==\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

json_escape() {
  python3 -c 'import json, sys; print(json.dumps(sys.argv[1]))' "$1"
}

sha256_text() {
  printf '%s' "$1" | sha256sum | awk '{print $1}'
}

image_exists() {
  docker image inspect "$TAG" >/dev/null 2>&1
}

write_selection() {
  local store_id="$1"
  local selected_tmp
  selected_tmp="$(mktemp "$STORE_ROOT/.selected.json.XXXXXX")"
  cat >"$selected_tmp" <<EOF
{
  "schema_version": 1,
  "kind": "soma_rootfs_selection",
  "store_id": "$(printf '%s' "$store_id")"
}
EOF
  mv "$selected_tmp" "$STORE_ROOT/selected.json"
}

command -v docker >/dev/null 2>&1 || die "docker is required for rootfs materialization"
command -v python3 >/dev/null 2>&1 || die "python3 is required for host-side manifest writing"
command -v sha256sum >/dev/null 2>&1 || die "sha256sum is required"

if [[ "$BASE_IMAGE" != *@sha256:* ]]; then
  die "base image must be digest-pinned with @sha256: for production Insula materialization"
fi
if [[ "${BASE_IMAGE,,}" != *cuda* ]]; then
  die "base image must be CUDA-enabled for the Soma Emacs gym Insula"
fi
if [[ -z "$EMACS_SOURCE_SHA256" ]]; then
  case "$EMACS_VERSION" in
    30.2)
      EMACS_SOURCE_SHA256="b3f36f18a6dd2715713370166257de2fae01f9d38cfe878ced9b1e6ded5befd9"
      ;;
    *)
      die "unknown Emacs source SHA256 for version $EMACS_VERSION; pass --emacs-sha256 or set SOMA_EMACS_SOURCE_SHA256"
      ;;
  esac
fi
if [[ ! "$EMACS_SOURCE_SHA256" =~ ^[0-9a-f]{64}$ ]]; then
  die "Emacs source SHA256 must be a 64-character lowercase hex digest"
fi

mkdir -p "$STORE_ROOT/content"
if [[ -L "$STORE_ROOT" ]]; then
  die "rootfs store must not be a symlink: $STORE_ROOT"
fi

RECIPE_SHA="$(sha256_text "$BASE_IMAGE|$EMACS_VERSION|$EMACS_SOURCE_SHA256|$DOOM_COMMIT|$RUST_TOOLCHAIN|$PYRIGHT_VERSION|$RECIPE_KIND")"
if [[ "$TAG" == "soma-emacs-rootfs:local" ]]; then
  TAG="soma-emacs-rootfs:${RECIPE_SHA:0:16}"
fi

log "base image: $BASE_IMAGE"
log "docker tag: $TAG"
log "recipe: $RECIPE_SHA"
log "emacs source sha256: $EMACS_SOURCE_SHA256"
log "rootfs store: $STORE_ROOT"

build_args=()
if [[ "$NO_CACHE" -eq 1 ]]; then
  build_args+=(--no-cache)
fi

if image_exists && [[ "$REBUILD" -eq 0 ]]; then
  log "image already exists; use --rebuild to force"
else
  log "building source-built Emacs/Doom image"
  docker build "${build_args[@]}" -t "$TAG" \
    --build-arg BASE_IMAGE="$BASE_IMAGE" \
    --build-arg EMACS_VERSION="$EMACS_VERSION" \
    --build-arg EMACS_SOURCE_SHA256="$EMACS_SOURCE_SHA256" \
    --build-arg DOOM_COMMIT="$DOOM_COMMIT" \
    --build-arg RUST_TOOLCHAIN="$RUST_TOOLCHAIN" \
    --build-arg PYRIGHT_VERSION="$PYRIGHT_VERSION" \
    --build-arg ROOTFS_RECIPE_KIND="$RECIPE_KIND" \
    --build-arg ROOTFS_RECIPE_SHA256="$RECIPE_SHA" \
    -f - "$ROOT" <<'DOCKERFILE'
ARG BASE_IMAGE
FROM ${BASE_IMAGE}

ARG BASE_IMAGE
ARG EMACS_VERSION
ARG EMACS_SOURCE_SHA256
ARG DOOM_COMMIT
ARG RUST_TOOLCHAIN
ARG PYRIGHT_VERSION
ARG ROOTFS_RECIPE_KIND
ARG ROOTFS_RECIPE_SHA256

SHELL ["/bin/bash", "-o", "pipefail", "-c"]
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -y && \
    apt-get install -y --no-install-recommends \
      bash ca-certificates curl xz-utils gzip bzip2 tar unzip patch \
      git openssh-client rsync jq ripgrep fd-find shellcheck tmux \
      build-essential binutils gcc g++ make autoconf automake pkg-config texinfo \
      libacl1-dev libselinux1-dev libgnutls28-dev libxml2-dev \
      libncurses-dev libjansson-dev libsqlite3-dev libtree-sitter-dev \
      libgccjit-13-dev zlib1g-dev \
      clang clangd cmake ninja-build \
      python3 python3-venv python3-pip python3-ipykernel python3-pylsp \
      jupyter-notebook nodejs npm && \
    rm -rf /var/lib/apt/lists/*

RUN npm install -g "pyright@${PYRIGHT_VERSION}"

ENV RUSTUP_HOME=/opt/soma/rustup
ENV CARGO_HOME=/opt/soma/cargo
ENV PATH=/opt/soma/cargo/bin:/opt/soma/emacs/bin:/usr/local/bin:/usr/bin:/bin
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
      sh -s -- -y --no-modify-path --profile minimal \
        --default-toolchain "${RUST_TOOLCHAIN}" \
        --component rustfmt --component clippy --component rust-analyzer \
        --component rust-src && \
    rustc --version && cargo --version && rust-analyzer --version

RUN mkdir -p /opt/soma/src /opt/soma/build && \
    curl -fsSL -o "/opt/soma/src/emacs-${EMACS_VERSION}.tar.xz" \
      "https://ftp.gnu.org/gnu/emacs/emacs-${EMACS_VERSION}.tar.xz" && \
    printf '%s  %s\n' "${EMACS_SOURCE_SHA256}" "/opt/soma/src/emacs-${EMACS_VERSION}.tar.xz" | sha256sum -c - && \
    sha256sum "/opt/soma/src/emacs-${EMACS_VERSION}.tar.xz" \
      > /opt/soma/src/emacs-source.sha256 && \
    tar -C /opt/soma/build -xf "/opt/soma/src/emacs-${EMACS_VERSION}.tar.xz"

RUN cd "/opt/soma/build/emacs-${EMACS_VERSION}" && \
    ./configure \
      --prefix=/opt/soma/emacs \
      --with-native-compilation \
      --with-tree-sitter \
      --with-sqlite3 \
      --without-x \
      --without-sound \
      --without-dbus \
      --without-gconf \
      --without-gsettings \
      --without-toolkit-scroll-bars \
      --without-xwidgets \
      --without-mailutils && \
    make -j"$(nproc)" && \
    make install && \
    /opt/soma/emacs/bin/emacs --quick --batch --eval '(princ emacs-version)'

RUN mkdir -p /opt/soma/doom-home/.emacs.d /opt/soma/doom-home/.doom.d && \
    git -C /opt/soma/doom-home/.emacs.d init && \
    git -C /opt/soma/doom-home/.emacs.d remote add origin https://github.com/doomemacs/doomemacs && \
    git -C /opt/soma/doom-home/.emacs.d fetch --depth 1 origin "${DOOM_COMMIT}" && \
    git -C /opt/soma/doom-home/.emacs.d checkout FETCH_HEAD && \
    { \
      printf '%s\n' ';;; init.el -*- lexical-binding: t; -*-'; \
      printf '%s\n' '(doom!'; \
      printf '%s\n' ' :emacs dired electric undo vc'; \
      printf '%s\n' ' :tools eval lookup lsp magit'; \
      printf '%s\n' ' :lang emacs-lisp (org +jupyter) python rust cc sh'; \
      printf '%s\n' ' :config (default +bindings))'; \
    } > /opt/soma/doom-home/.doom.d/init.el && \
    printf '%s\n' ';;; packages.el -*- lexical-binding: t; -*-' \
      > /opt/soma/doom-home/.doom.d/packages.el && \
    printf '%s\n' ';;; config.el -*- lexical-binding: t; -*-' \
      > /opt/soma/doom-home/.doom.d/config.el

RUN HOME=/opt/soma/doom-home \
    DOOMDIR=/opt/soma/doom-home/.doom.d \
    PATH=/opt/soma/emacs/bin:/opt/soma/cargo/bin:/usr/local/bin:/usr/bin:/bin \
    /opt/soma/doom-home/.emacs.d/bin/doom sync -e -U -B --no-color \
      --doomdir /opt/soma/doom-home/.doom.d \
      --emacsdir /opt/soma/doom-home/.emacs.d

RUN mkdir -p /workspace/soma /project/home /project/tmp /project/run /project/cache /project/logs /project/results \
      /opt/soma/doom-home/.emacs.d/eln-cache && \
    emacs_path=/opt/soma/emacs/bin/emacs && \
    emacs_sha="$(sha256sum "$emacs_path" | awk '{print $1}')" && \
    emacs_source_sha="$(awk '{print $1}' /opt/soma/src/emacs-source.sha256)" && \
    doom_head="$(git -C /opt/soma/doom-home/.emacs.d rev-parse HEAD)" && \
    export BASE_IMAGE ROOTFS_RECIPE_KIND ROOTFS_RECIPE_SHA256 EMACS_VERSION EMACS_SOURCE_SHA256 emacs_sha emacs_source_sha doom_head && \
    python3 - <<PY
import json
import os
import pathlib
import subprocess
import hashlib

tools = [
    "emacs", "emacsclient", "python3", "jupyter", "pylsp",
    "pyright", "pyright-langserver", "clang", "clangd", "gcc", "g++",
    "rustc", "cargo", "rust-analyzer", "git", "sh", "bash", "tmux",
    "rg", "fd", "jq", "shellcheck",
]
path = os.environ["PATH"]
inventory = {}

def file_sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

def elf_metadata(path):
    metadata = {"elf": False, "dynamic_loader": "", "ldd": []}
    try:
        with open(path, "rb") as fh:
            metadata["elf"] = fh.read(4) == b"\x7fELF"
    except OSError:
        return metadata
    if not metadata["elf"]:
        return metadata
    readelf = subprocess.run(["readelf", "-l", path], text=True, capture_output=True)
    if readelf.returncode == 0:
        for line in readelf.stdout.splitlines():
            if "Requesting program interpreter:" in line:
                metadata["dynamic_loader"] = line.split("Requesting program interpreter:", 1)[1].strip(" ]")
                break
    ldd = subprocess.run(["ldd", path], text=True, capture_output=True)
    if ldd.stdout:
        metadata["ldd"] = ldd.stdout.splitlines()
    elif ldd.stderr:
        metadata["ldd"] = ldd.stderr.splitlines()
    return metadata

for tool in tools:
    found = subprocess.run(["bash", "-lc", f"command -v {tool} || true"], text=True, capture_output=True, check=True).stdout.strip()
    version = ""
    realpath = ""
    executable = False
    sha256 = ""
    metadata = {"elf": False, "dynamic_loader": "", "ldd": []}
    if found:
        realpath = os.path.realpath(found)
        executable = os.access(realpath, os.X_OK)
        if os.path.isfile(realpath):
            sha256 = file_sha256(realpath)
            metadata = elf_metadata(realpath)
        probe = {
            "emacs": [found, "--quick", "--batch", "--eval", "(princ emacs-version)"],
            "emacsclient": [found, "--version"],
            "python3": [found, "-c", "import sys; print(sys.version.split()[0])"],
            "jupyter": [found, "--version"],
            "pylsp": [found, "--version"],
            "pyright": [found, "--version"],
            "pyright-langserver": [found, "--version"],
            "clang": [found, "--version"],
            "clangd": [found, "--version"],
            "gcc": [found, "--version"],
            "g++": [found, "--version"],
            "rustc": [found, "--version"],
            "cargo": [found, "--version"],
            "rust-analyzer": [found, "--version"],
            "git": [found, "--version"],
            "tmux": [found, "-V"],
            "rg": [found, "--version"],
            "fd": [found, "--version"],
            "jq": [found, "--version"],
            "shellcheck": [found, "--version"],
            "bash": [found, "--version"],
            "sh": [found, "-c", "echo sh"],
        }.get(tool)
        if probe:
            r = subprocess.run(probe, text=True, capture_output=True)
            version = (r.stdout or r.stderr).splitlines()[0] if (r.stdout or r.stderr) else ""
    inventory[tool] = {
        "path": found,
        "realpath": realpath,
        "sha256": sha256,
        "executable": executable,
        "elf": metadata["elf"],
        "dynamic_loader": metadata["dynamic_loader"],
        "ldd": metadata["ldd"],
        "version": version,
    }

def run(argv):
    return subprocess.run(argv, text=True, capture_output=True, check=False)

def first_line(result):
    output = result.stdout or result.stderr
    return output.splitlines()[0] if output else ""

def dynamic_loader(executable):
    result = run(["readelf", "-l", executable])
    if result.returncode != 0:
        return ""
    for line in result.stdout.splitlines():
        if "Requesting program interpreter:" in line:
            return line.split("Requesting program interpreter:", 1)[1].strip(" ]")
    return ""

def emacs_eval(expr):
    result = run(["/opt/soma/emacs/bin/emacs", "--quick", "--batch", "--eval", expr])
    return first_line(result), result.returncode

ldd = run(["ldd", "/opt/soma/emacs/bin/emacs"])
glibc_version = first_line(run(["getconf", "GNU_LIBC_VERSION"]))
dynamic_loader_path = dynamic_loader("/opt/soma/emacs/bin/emacs")
json_builtin, json_status = emacs_eval("(princ (if (fboundp 'json-parse-string) \"t\" \"nil\"))")
native_comp_available, native_status = emacs_eval("(princ (if (and (fboundp 'native-comp-available-p) (native-comp-available-p)) \"t\" \"nil\"))")
tree_sitter_available, tree_status = emacs_eval("(princ (if (and (fboundp 'treesit-available-p) (treesit-available-p)) \"t\" \"nil\"))")
contract = {
    "schema_version": 1,
    "kind": "soma_emacs_rootfs_contract",
    "rootfs_ownership": "soma-selected-container-rootfs",
    "dependency_closure": "rootfs_owned",
    "abi_guarantee": "container-rootfs-abi-manifest",
    "abi_base": "container-image-rootfs",
    "glibc_owner": "container-image-rootfs",
    "nix_allowed": False,
    "base_image": os.environ.get("BASE_IMAGE", ""),
    "rootfs_recipe_sha256": os.environ.get("ROOTFS_RECIPE_SHA256", ""),
    "forbidden_abi_prefixes": ["/nix", "/data02/home"],
    "materialization_kind": os.environ.get("ROOTFS_RECIPE_KIND", ""),
    "base_image_digest_required": True,
    "materialization": {
        "kind": "docker-cuda-source-build",
        "builder": "scripts/insula/build-emacs-rootfs.sh",
        "network_policy": "materialization may use network; benchmark episodes consume the exported rootfs",
        "glibc_policy": "glibc and dynamic loader come from the base container-image rootfs",
    },
    "abi": {
        "glibc_version": glibc_version,
        "dynamic_loader": dynamic_loader_path,
        "emacs_ldd": ldd.stdout.splitlines(),
    },
    "emacs": {
        "build_kind": "gnu-source-tarball",
        "build_location": "inside-rootfs",
        "patch_variant": "stock",
        "patch_set": [],
        "version": os.environ.get("EMACS_VERSION", ""),
        "source_url": f"https://ftp.gnu.org/gnu/emacs/emacs-{os.environ.get('EMACS_VERSION', '')}.tar.xz",
        "source_archive": f"/opt/soma/src/emacs-{os.environ.get('EMACS_VERSION', '')}.tar.xz",
        "source_tree": f"/opt/soma/build/emacs-{os.environ.get('EMACS_VERSION', '')}",
        "source_sha256": os.environ.get("emacs_source_sha", ""),
        "expected_source_sha256": os.environ.get("EMACS_SOURCE_SHA256", ""),
        "source_sha256_verified_before_extract": os.environ.get("emacs_source_sha", "") == os.environ.get("EMACS_SOURCE_SHA256", ""),
        "prefix": "/opt/soma/emacs",
        "executable": "/opt/soma/emacs/bin/emacs",
        "executable_sha256": os.environ.get("emacs_sha", ""),
        "configure_flags": [
            "--prefix=/opt/soma/emacs",
            "--with-native-compilation",
            "--with-tree-sitter",
            "--with-sqlite3",
            "--without-x",
            "--without-sound",
            "--without-dbus",
            "--without-gconf",
            "--without-gsettings",
            "--without-toolkit-scroll-bars",
            "--without-xwidgets",
            "--without-mailutils",
        ],
        "capabilities": {
            "json_parse_string": json_builtin == "t" and json_status == 0,
            "native_comp_available": native_comp_available == "t" and native_status == 0,
            "tree_sitter_available": tree_sitter_available == "t" and tree_status == 0,
        },
        "ldd": ldd.stdout.splitlines(),
    },
    "doom": {
        "kind": "rootfs-owned-doom-sync",
        "commit": os.environ.get("doom_head", ""),
        "emacsdir": "/opt/soma/doom-home/.emacs.d",
        "doomdir": "/opt/soma/doom-home/.doom.d",
        "profile_source": "minimal-rootfs-profile",
    },
    "tool_inventory": inventory,
}
pathlib.Path("/etc/soma-insula-rootfs-contract.json").write_text(
    json.dumps(contract, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY

ENV HOME=/project/home
ENV TMPDIR=/project/tmp
ENV XDG_RUNTIME_DIR=/project/run
ENV XDG_CACHE_HOME=/project/cache
ENV XDG_CONFIG_HOME=/project/home/.config
ENV XDG_DATA_HOME=/project/home/.local/share
ENV PATH=/opt/soma/emacs/bin:/opt/soma/cargo/bin:/usr/local/bin:/usr/bin:/bin
ENV SHELL=/bin/bash
DOCKERFILE
fi

log "exporting image to rootfs store"
STAGE="$(mktemp -d "$STORE_ROOT/.rootfs.stage.XXXXXX")"
cid=""
trap '[[ -n "${cid:-}" ]] && docker rm -f "$cid" >/dev/null 2>&1 || true; rm -rf "$STAGE"' EXIT
cid="$(docker create "$TAG")"
docker export "$cid" | tar -C "$STAGE" -xf -
docker rm "$cid" >/dev/null
cid=""

[[ -x "$STAGE/opt/soma/emacs/bin/emacs" ]] || die "exported rootfs lacks source-built Emacs"
[[ -f "$STAGE/etc/soma-insula-rootfs-contract.json" ]] || die "exported rootfs lacks Soma contract"
if grep -Rqs '/nix/store' "$STAGE/etc/soma-insula-rootfs-contract.json"; then
  die "rootfs contract references /nix/store; refusing selection"
fi
python3 "$ROOT/scripts/insula/rootfs_tree_manifest.py" write "$STAGE" --update-contract >/dev/null
python3 "$ROOT/scripts/insula/validate-emacs-rootfs-contract.py" "$STAGE" >/dev/null

IMAGE_ID="$(docker image inspect --format '{{.Id}}' "$TAG")"
STORE_ID="soma-emacs-${RECIPE_SHA:0:16}-${IMAGE_ID#sha256:}"
STORE_ID="${STORE_ID:0:80}"
TARGET="$STORE_ROOT/content/$STORE_ID"
if [[ -e "$TARGET" ]]; then
  rm -rf "$TARGET"
fi
mv "$STAGE" "$TARGET"
trap - EXIT

python3 - "$TARGET" "$TAG" "$IMAGE_ID" "$BASE_IMAGE" <<'PY'
import json
import pathlib
import sys

rootfs, tag, image_id, base_image = sys.argv[1:5]
contract_path = pathlib.Path(rootfs) / "etc" / "soma-insula-rootfs-contract.json"
data = json.loads(contract_path.read_text(encoding="utf-8"))
data["image_tag"] = tag
data["image_id"] = image_id
data["base_image"] = base_image
contract_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

python3 "$ROOT/scripts/insula/validate-emacs-rootfs-contract.py" "$TARGET" >/dev/null

write_selection "$STORE_ID"

log "selected rootfs"
printf 'rootfs: %s\n' "$TARGET"
printf 'selection: %s\n' "$STORE_ROOT/selected.json"
printf 'contract: %s\n' "$TARGET/etc/soma-insula-rootfs-contract.json"
