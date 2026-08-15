#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
output_root="$repository_root/evidence/crouzeix_conjecture"
remote=https://github.com/jinshanmu/CrouzeixConjecture.git
web_remote=https://github.com/jinshanmu/CrouzeixConjecture
audited_revision=565b6a3e0659b6e0785f783b016c3f6d9f171fa5
head_revision=9df07838327b988e3924453daa29c8cd726d34b0
elan_version=v4.2.3
elan_sha256=7cae4c03b2f0de4053fb04a91359d5804551e6e37a6ddd1b2e0097dc561ae4a9

tmp_base=${TMPDIR:-/tmp}
tmp_root="$tmp_base/harp-crouzeix-acquire-$$"
test ! -e "$tmp_root" || {
  printf '%s\n' "temporary acquisition root already exists: $tmp_root" >&2
  exit 1
}
mkdir -m 700 "$tmp_root"
cleanup() {
  rm -rf "$tmp_root"
}
abort() {
  trap - EXIT
  cleanup
  exit 130
}
trap cleanup EXIT
trap abort HUP INT TERM

export GIT_CONFIG_NOSYSTEM=1
export GIT_CONFIG_GLOBAL=/dev/null
export GIT_NO_REPLACE_OBJECTS=1

available_kib=$(df -Pk "$tmp_root" | awk 'NR == 2 { print $4 }')
minimum_lean_kib=8388608
lean_blocked=false
if test "$available_kib" -lt "$minimum_lean_kib"; then
  lean_blocked=true
fi

if test "$lean_blocked" = false && ! command -v lake >/dev/null 2>&1; then
  elan_archive="$tmp_root/elan.tar.gz"
  curl -fsSL \
    "https://github.com/leanprover/elan/releases/download/$elan_version/elan-aarch64-apple-darwin.tar.gz" \
    -o "$elan_archive"
  test "$(shasum -a 256 "$elan_archive" | awk '{print $1}')" = "$elan_sha256"
  mkdir "$tmp_root/elan"
  tar -xzf "$elan_archive" -C "$tmp_root/elan"
  export ELAN_HOME="$tmp_root/elan-home"
  export PATH="$tmp_root/elan:$ELAN_HOME/bin:$PATH"
  "$tmp_root/elan/elan-init" -y --no-modify-path --default-toolchain none
fi

git clone --quiet --no-checkout --filter=blob:none "$remote" "$tmp_root/repository"
test "$(git -C "$tmp_root/repository" remote get-url origin)" = "$remote"
test ! -s "$tmp_root/repository/.git/objects/info/alternates"
test -z "$(git -C "$tmp_root/repository" replace -l)"
test -z "$(git -C "$tmp_root/repository" config --local --get-regexp '^include(\.|$)' || true)"

for revision in "$audited_revision" "$head_revision"; do
  git -C "$tmp_root/repository" cat-file -e "$revision^{commit}"
  short=$(printf '%s' "$revision" | cut -c1-8)
  checkout="$tmp_root/checkout-$short"
  git -C "$tmp_root/repository" worktree add --quiet --detach "$checkout" "$revision"
  test -z "$(git -C "$checkout" status --porcelain=v1)"
  test -z "$(find "$checkout" -type l -print -quit)"

  if test "$lean_blocked" = true; then
    {
      printf '%s\t%s\n' operation lean-build
      printf '%s\t%s\n' source_commit "$revision"
      printf '%s\t%s\n' result blocked
      printf '%s\t%s\n' reason insufficient-disk-for-pinned-mathlib-cache
      printf '%s\t%s\n' available_kib "$available_kib"
      printf '%s\t%s\n' required_kib "$minimum_lean_kib"
    } >"$tmp_root/$short-build.raw"
    printf '%s\n' blocked >"$tmp_root/$short-build.status"
  else
    set +e
    (
      cd "$checkout/Lean"
      lake exe cache get
    ) >"$tmp_root/$short-cache.raw" 2>&1
    cache_status=$?
    set -e
    if test "$cache_status" -ne 0; then
      {
        printf '%s\t%s\n' operation lean-build
        printf '%s\t%s\n' source_commit "$revision"
        printf '%s\t%s\n' result blocked
        printf '%s\t%s\n' reason mathlib-cache-materialization-failed
        printf '%s\t%s\n' cache_exit_code "$cache_status"
        printf '%s\t%s\n' cache_log_sha256 \
          "$(shasum -a 256 "$tmp_root/$short-cache.raw" | awk '{print $1}')"
      } >"$tmp_root/$short-build.raw"
      printf '%s\n' blocked >"$tmp_root/$short-build.status"
    else
      set +e
      (
        cd "$checkout/Lean"
        ./verify.sh
      ) >"$tmp_root/$short-build.raw" 2>&1
      build_status=$?
      set -e
      printf '%s\n' "$build_status" >"$tmp_root/$short-build.status"
    fi
  fi

  python3 - "$checkout" "$tmp_root/$short-scan.raw" <<'PY'
from __future__ import annotations

import hashlib
import pathlib
import re
import subprocess
import sys

checkout = pathlib.Path(sys.argv[1])
output = pathlib.Path(sys.argv[2])
files = subprocess.check_output(
    ["git", "-C", str(checkout), "ls-files", "Lean/**/*.lean", "Lean/*.lean"],
    text=True,
).splitlines()
patterns = {
    "sorry": re.compile(r"\bsorry\b"),
    "admit": re.compile(r"\badmit\b"),
    "custom-axiom": re.compile(r"^\s*axiom\b"),
    "unsafe": re.compile(r"\bunsafe\b"),
    "native-decide": re.compile(r"\bnative_decide\b"),
    "implemented-by": re.compile(r"\bimplemented_by\b"),
    "option-override": re.compile(r"\bset_option\b"),
}
rows: list[str] = []
for relative in sorted(set(files)):
    path = checkout / relative
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        for token_class, pattern in patterns.items():
            if pattern.search(line):
                digest = hashlib.sha256(line.encode("utf-8")).hexdigest()
                rows.append(f"{relative}\t{number}\t{token_class}\t{digest}")
output.write_text("\n".join(rows) + ("\n" if rows else ""), encoding="utf-8")
PY
  printf '%s\n' 0 >"$tmp_root/$short-scan.status"
done

mkdir -p "$tmp_root/artifacts"

capture_git_file() {
  revision=$1
  path=$2
  output=$3
  git -C "$tmp_root/repository" show "$revision:$path" >"$output"
}

curl -fsSL "$web_remote/archive/$audited_revision.tar.gz" \
  -o "$tmp_root/artifacts/JIN-565-ARCHIVE"
capture_git_file "$audited_revision" preprint/the_numerical_range_is_a_2_spectral_set_v4.tex "$tmp_root/artifacts/JIN-565-V4-TEX"
capture_git_file "$audited_revision" Lean/FORMALIZATION_MAP.md "$tmp_root/artifacts/JIN-565-FORMALIZATION-MAP"
capture_git_file "$audited_revision" Lean/MANUSCRIPT_AUDIT.md "$tmp_root/artifacts/JIN-565-MANUSCRIPT-AUDIT"
capture_git_file "$audited_revision" Lean/AxiomAudit.lean "$tmp_root/artifacts/JIN-565-AXIOM-AUDIT"
capture_git_file "$audited_revision" Lean/verify.sh "$tmp_root/artifacts/JIN-565-VERIFY"
capture_git_file "$audited_revision" Lean/lean-toolchain "$tmp_root/artifacts/JIN-565-LEAN-TOOLCHAIN"
capture_git_file "$audited_revision" Lean/lake-manifest.json "$tmp_root/artifacts/JIN-565-LAKE-MANIFEST"

curl -fsSL "$web_remote/archive/$head_revision.tar.gz" \
  -o "$tmp_root/artifacts/JIN-HEAD-ARCHIVE"
capture_git_file "$head_revision" README.md "$tmp_root/artifacts/JIN-HEAD-README"
capture_git_file "$head_revision" crouzeix_conjecture_prompt.txt "$tmp_root/artifacts/JIN-HEAD-PROMPT"
capture_git_file "$head_revision" preprint/the_numerical_range_is_a_2_spectral_set_v4.tex "$tmp_root/artifacts/JIN-HEAD-V4-TEX"
capture_git_file "$head_revision" AnnMath/the_numerical_range_is_a_2_spectral_set.tex "$tmp_root/artifacts/JIN-HEAD-ANNMATH-TEX"
capture_git_file "$head_revision" Lean/FORMALIZATION_MAP.md "$tmp_root/artifacts/JIN-HEAD-FORMALIZATION-MAP"
capture_git_file "$head_revision" Lean/MANUSCRIPT_AUDIT.md "$tmp_root/artifacts/JIN-HEAD-MANUSCRIPT-AUDIT"
capture_git_file "$head_revision" Lean/AxiomAudit.lean "$tmp_root/artifacts/JIN-HEAD-AXIOM-AUDIT"
capture_git_file "$head_revision" Lean/verify.sh "$tmp_root/artifacts/JIN-HEAD-VERIFY"
capture_git_file "$head_revision" Lean/lean-toolchain "$tmp_root/artifacts/JIN-HEAD-LEAN-TOOLCHAIN"
capture_git_file "$head_revision" Lean/lake-manifest.json "$tmp_root/artifacts/JIN-HEAD-LAKE-MANIFEST"

curl -fsSL 'https://export.arxiv.org/api/query?id_list=2608.03841' \
  -o "$tmp_root/artifacts/LS-ARXIV-V1-ATOM"
curl -fsSL 'https://export.arxiv.org/e-print/2608.03841v1' \
  -o "$tmp_root/artifacts/LS-ARXIV-V1-SOURCE-ARCHIVE"
mkdir "$tmp_root/ls-source"
tar -xzf "$tmp_root/artifacts/LS-ARXIV-V1-SOURCE-ARCHIVE" -C "$tmp_root/ls-source"
cp "$tmp_root/ls-source/CrouzeixConjecturev2.tex" "$tmp_root/artifacts/LS-ARXIV-V1-TEX"
curl -fsSL 'https://arxiv.org/pdf/2608.03841v1' \
  -o "$tmp_root/artifacts/LS-ARXIV-V1-PDF"

preprints_url=https://www.preprints.org/manuscript/202607.1919/v1
set +e
preprints_status=$(curl -A 'Mozilla/5.0 Harp evidence capture/1.0' -L -sS \
  -o "$tmp_root/preprints-body" -w '%{http_code}' "$preprints_url")
curl_status=$?
set -e
if test "$curl_status" -ne 0; then
  preprints_status="curl-$curl_status"
fi
printf '%s\n' "$preprints_status" >"$tmp_root/preprints-status"
if test "$preprints_status" = 200; then
  cp "$tmp_root/preprints-body" "$tmp_root/artifacts/JIN-PREPRINTS-V1-METADATA"
fi

python3 - "$repository_root" "$tmp_root" "$output_root" <<'PY'
from __future__ import annotations

import datetime as dt
import hashlib
import os
import pathlib
import re
import sys

repository_root = pathlib.Path(sys.argv[1])
tmp_root = pathlib.Path(sys.argv[2])
output_root = pathlib.Path(sys.argv[3])
artifacts = tmp_root / "artifacts"
observed = dt.datetime.now(dt.timezone.utc)
observed_date = observed.date().isoformat()
observed_at = observed.replace(microsecond=0).isoformat().replace("+00:00", "Z")

source_header = (
    "schema_version\treceipt_id\tsource_id\tsource_class\trole\t"
    "immutable_identity\tsource_url\tupstream_path\tbytes\tsha256\t"
    "local_path\tobserved\tlicense_status\tredistribution_status"
)
verification_header = (
    "schema_version\treceipt_id\tsource_id\tsource_commit\tsource_tree\t"
    "operation\tcommand_sha256\tacquisition_script_sha256\t"
    "normalization_version\ttoolchain\tmathlib_revision\tobserved_at_utc\t"
    "exit_code\tresult\tlog_path\tlog_bytes\tlog_sha256"
)

def digest_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def artifact_fields(receipt_id: str) -> tuple[str, str]:
    data = (artifacts / receipt_id).read_bytes()
    return str(len(data)), digest_bytes(data)

def source_row(
    receipt_id: str,
    source_id: str,
    source_class: str,
    role: str,
    identity: str,
    url: str,
    path: str,
    license_status: str,
    redistribution: str,
    measured: bool = True,
) -> list[str]:
    if measured:
        byte_count, digest = artifact_fields(receipt_id)
    else:
        byte_count, digest = "-", "-"
    return [
        "crouzeix-source-receipt/v1",
        receipt_id,
        source_id,
        source_class,
        role,
        identity,
        url,
        path,
        byte_count,
        digest,
        "-",
        observed_date,
        license_status,
        redistribution,
    ]

jin_remote = "https://github.com/jinshanmu/CrouzeixConjecture"
audited = "565b6a3e0659b6e0785f783b016c3f6d9f171fa5"
head = "9df07838327b988e3924453daa29c8cd726d34b0"
rows: list[list[str]] = []
rows.append(source_row(
    "JIN-565-ARCHIVE", "JIN-V4-AUDITED", "git-artifact", "history",
    f"git:{audited}", f"{jin_remote}/archive/{audited}.tar.gz", "-",
    "not-present-at-revision", "remote-only",
))
for receipt_id, path, role in [
    ("JIN-565-V4-TEX", "preprint/the_numerical_range_is_a_2_spectral_set_v4.tex", "manuscript"),
    ("JIN-565-FORMALIZATION-MAP", "Lean/FORMALIZATION_MAP.md", "formalization"),
    ("JIN-565-MANUSCRIPT-AUDIT", "Lean/MANUSCRIPT_AUDIT.md", "formalization"),
    ("JIN-565-AXIOM-AUDIT", "Lean/AxiomAudit.lean", "formalization"),
    ("JIN-565-VERIFY", "Lean/verify.sh", "formalization"),
    ("JIN-565-LEAN-TOOLCHAIN", "Lean/lean-toolchain", "formalization"),
    ("JIN-565-LAKE-MANIFEST", "Lean/lake-manifest.json", "formalization"),
]:
    rows.append(source_row(
        receipt_id, "JIN-V4-AUDITED", "git-artifact", role,
        f"git:{audited}", f"{jin_remote}/blob/{audited}/{path}", path,
        "not-present-at-revision", "quotation-only",
    ))
rows.append(source_row(
    "JIN-HEAD-ARCHIVE", "JIN-REPO-HEAD", "git-artifact", "history",
    f"git:{head}", f"{jin_remote}/archive/{head}.tar.gz", "-",
    "not-present-at-revision", "remote-only",
))
for receipt_id, source_id, path, role in [
    ("JIN-HEAD-README", "JIN-REPO-HEAD", "README.md", "metadata"),
    ("JIN-HEAD-PROMPT", "JIN-REPO-HEAD", "crouzeix_conjecture_prompt.txt", "prompt"),
    ("JIN-HEAD-V4-TEX", "JIN-V4-HEAD", "preprint/the_numerical_range_is_a_2_spectral_set_v4.tex", "manuscript"),
    ("JIN-HEAD-ANNMATH-TEX", "JIN-ANNMATH", "AnnMath/the_numerical_range_is_a_2_spectral_set.tex", "manuscript"),
    ("JIN-HEAD-FORMALIZATION-MAP", "JIN-REPO-HEAD", "Lean/FORMALIZATION_MAP.md", "formalization"),
    ("JIN-HEAD-MANUSCRIPT-AUDIT", "JIN-REPO-HEAD", "Lean/MANUSCRIPT_AUDIT.md", "formalization"),
    ("JIN-HEAD-AXIOM-AUDIT", "JIN-REPO-HEAD", "Lean/AxiomAudit.lean", "formalization"),
    ("JIN-HEAD-VERIFY", "JIN-REPO-HEAD", "Lean/verify.sh", "formalization"),
    ("JIN-HEAD-LEAN-TOOLCHAIN", "JIN-REPO-HEAD", "Lean/lean-toolchain", "formalization"),
    ("JIN-HEAD-LAKE-MANIFEST", "JIN-REPO-HEAD", "Lean/lake-manifest.json", "formalization"),
]:
    rows.append(source_row(
        receipt_id, source_id, "git-artifact", role,
        f"git:{head}", f"{jin_remote}/blob/{head}/{path}", path,
        "not-present-at-revision", "quotation-only",
    ))

preprints_status = (tmp_root / "preprints-status").read_text().strip()
preprints_measured = preprints_status == "200"
if preprints_measured:
    preprints_digest = artifact_fields("JIN-PREPRINTS-V1-METADATA")[1]
else:
    preprints_digest = digest_bytes(b"https://www.preprints.org/manuscript/202607.1919/v1")
rows.append(source_row(
    "JIN-PREPRINTS-V1-METADATA", "JIN-PREPRINTS-V1", "dated-page", "metadata",
    f"sha256:{preprints_digest}",
    "https://www.preprints.org/manuscript/202607.1919/v1", "-",
    "unknown", "metadata-only", measured=preprints_measured,
))

for receipt_id, role, url, path in [
    ("LS-ARXIV-V1-ATOM", "metadata", "https://export.arxiv.org/api/query?id_list=2608.03841v1", "atom.xml"),
    ("LS-ARXIV-V1-SOURCE-ARCHIVE", "source", "https://export.arxiv.org/e-print/2608.03841v1", "source.tar.gz"),
    ("LS-ARXIV-V1-TEX", "manuscript", "https://export.arxiv.org/e-print/2608.03841v1", "CrouzeixConjecturev2.tex"),
    ("LS-ARXIV-V1-PDF", "pdf", "https://arxiv.org/pdf/2608.03841v1", "paper.pdf"),
]:
    rows.append(source_row(
        receipt_id, "LS-ARXIV-V1", "arxiv-artifact", role,
        "arxiv:2608.03841v1", url, path,
        "arxiv-nonexclusive", "quotation-only",
    ))

for receipt_id, doi in [
    ("CROUZEIX-2007", "10.1016/j.jfa.2006.10.013"),
    ("CROUZEIX-PALENCIA-2017", "10.1137/17M1116672"),
    ("DELYON-DELYON-1999", "10.24033/bsmf.2340"),
    ("RANSFORD-SCHWENNINGER-2018", "10.1137/17M1143757"),
    ("SCHWENNINGER-DEVRIES-2025", "10.1007/s00020-025-02800-2"),
]:
    rows.append(source_row(
        receipt_id, receipt_id, "dated-page", "prerequisite",
        f"doi:{doi}", f"https://doi.org/{doi}", "-",
        "publisher-record", "metadata-only", measured=False,
    ))

rows.sort(key=lambda row: row[1])
assert len(rows) == 29, len(rows)

output_root.mkdir(parents=True, exist_ok=True)
verification_root = output_root / "verification"
verification_root.mkdir(parents=True, exist_ok=True)

source_text = source_header + "\n" + "\n".join("\t".join(row) for row in rows) + "\n"
(output_root / "source_manifest.tsv").write_text(source_text, encoding="utf-8")

script_path = output_root / "acquire.sh"
script_digest = digest_bytes(script_path.read_bytes())
secret_pattern = re.compile(
    r"(?i)(api[_-]?key|access[_-]?token|authorization:\s*bearer|"
    r"-----BEGIN [A-Z ]*PRIVATE KEY-----)"
)

def normalized_build(short: str, commit: str) -> tuple[str, str, str]:
    raw = (tmp_root / f"{short}-build.raw").read_text(errors="replace")
    checkout = str(tmp_root / f"checkout-{short}")
    normalized = raw.replace(checkout, "<source-checkout>")
    if secret_pattern.search(normalized):
        raise SystemExit(f"secret-like text detected in {short} build log")
    raw_lines = {
        line.strip()
        for path in (tmp_root / f"checkout-{short}" / "Lean").rglob("*.lean")
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    }
    excerpt = any(
        len(line.strip()) >= 32 and line.strip() in raw_lines
        for line in normalized.splitlines()
    )
    status_text = (tmp_root / f"{short}-build.status").read_text().strip()
    if status_text == "blocked":
        if len(normalized.encode()) > 2 * 1024 * 1024:
            raise SystemExit(f"{short} build log exceeds 2 MiB")
        return normalized, "-", "blocked"
    status = int(status_text)
    if excerpt:
        normalized = (
            f"operation\tlean-build\nsource_commit\t{commit}\n"
            f"exit_code\t{status}\nsource_excerpt_detected\ttrue\n"
            f"raw_output_sha256\t{digest_bytes(raw.encode())}\n"
        )
    if len(normalized.encode()) > 2 * 1024 * 1024:
        raise SystemExit(f"{short} build log exceeds 2 MiB")
    return normalized, str(status), "passed" if status == 0 else "failed"

def scan_log(short: str) -> tuple[str, int]:
    text = (tmp_root / f"{short}-scan.raw").read_text()
    if secret_pattern.search(text):
        raise SystemExit(f"secret-like text detected in {short} scan log")
    if len(text.encode()) > 2 * 1024 * 1024:
        raise SystemExit(f"{short} scan log exceeds 2 MiB")
    status = int((tmp_root / f"{short}-scan.status").read_text().strip())
    return text, status

verification_rows: list[list[str]] = []
for short, label, source_id, commit, tree in [
    ("565b6a3e", "565b6a3", "JIN-V4-AUDITED", "565b6a3e0659b6e0785f783b016c3f6d9f171fa5", "40aafa503bd32762dbf6d1a67ddef3e2b067f0e1"),
    ("9df07838", "9df0783", "JIN-REPO-HEAD", "9df07838327b988e3924453daa29c8cd726d34b0", "ff9ff787a91707ddf747d2c670bf9e729e1c0cab"),
]:
    for operation, suffix, command in [
        ("lean-build", "build", ["./verify.sh"]),
        (
            "source-scan",
            "scan",
            ["python3", "-", "<source-checkout>", "<raw-output>"],
        ),
    ]:
        if operation == "lean-build":
            text, exit_code, result = normalized_build(short, commit)
        else:
            text, status = scan_log(short)
            exit_code = str(status)
            result = "passed" if status == 0 else "failed"
        log_name = f"jin-{label}-{suffix}.log"
        log_path = verification_root / log_name
        log_path.write_text(text, encoding="utf-8")
        log_bytes = log_path.read_bytes()
        command_digest = digest_bytes(b"\0".join(arg.encode() for arg in command))
        verification_rows.append([
            "crouzeix-verification-receipt/v1",
            f"JIN-{label.upper()}-{suffix.upper()}",
            source_id,
            commit,
            tree,
            operation,
            command_digest,
            script_digest,
            "crouzeix-log-normalization/v1",
            "leanprover/lean4:v4.28.0",
            "8f9d9cff6bd728b17a24e163c9402775d9e6a365",
            observed_at,
            exit_code,
            result,
            f"verification/{log_name}",
            str(len(log_bytes)),
            digest_bytes(log_bytes),
        ])

verification_rows.sort(key=lambda row: row[1])
verification_text = (
    verification_header
    + "\n"
    + "\n".join("\t".join(row) for row in verification_rows)
    + "\n"
)
(output_root / "verification_manifest.tsv").write_text(
    verification_text, encoding="utf-8"
)

provenance = f"""# Crouzeix conjecture evidence provenance

Captured at: `{observed_at}`

This bundle keeps upstream proof artifacts remote-only. `source_manifest.tsv`
records immutable identities, byte counts, SHA-256 digests, rights status, and
semantic source paths. It does not vendor manuscript, PDF, TeX, Lean, prompt,
or repository bytes.

`acquire.sh` created fresh detached checkouts for the two Jin revisions,
verified remote and Git identity, ran each revision's `Lean/verify.sh`, and
performed a metadata-only tracked-source scan. Build logs replace the checkout
root with `<source-checkout>`, abort on secret-like text or output above 2 MiB,
and collapse any detected source excerpt to metadata and digests. Scan logs
contain only path, line number, token class, and line SHA-256.

Preprints.org returned status `{preprints_status}` on `{observed_date}`. The
receipt is metadata-only unless status 200 produced measured bytes. No claim
maps the posted manuscript to a Git revision without byte equality.

The five prerequisite DOI records are metadata-only publisher routes. Their
presence does not prove access to full text, reproduce a theorem, or establish
the correctness of either candidate proof.
"""
(output_root / "PROVENANCE.md").write_text(provenance, encoding="utf-8")

print(
    f"Crouzeix evidence captured: {len(rows)} source receipts, "
    f"{len(verification_rows)} verification receipts"
)
PY
