#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
bundle_root="$repo_root/evidence/self_improving_agents_survey"
site_root="$bundle_root/site"
survey_root="$bundle_root/survey"
repo_root_out="$bundle_root/repos"
one_hop_root="$bundle_root/one_hop"
observed_at="${HARP_CAPTURED_AT:-2026-08-09T00:00:00Z}"
site_url="https://selfimproving-agent.github.io/"
site_repo="selfimproving-agent/selfimproving-agent.github.io"
site_commit="d8af6607ced118351108670f823cd106649cb757"
awesome_repo="selfimproving-agent/Awesome-Self-Improving-Agents"
arxiv_id="2607.13104"

if [[ "${1:-}" == "--refresh" ]]; then
  refresh=1
elif [[ -n "${1:-}" ]]; then
  echo "usage: $0 [--refresh]" >&2
  exit 2
else
  refresh=0
fi

for required in curl git jq pandoc pdftotext python3 shasum wc; do
  command -v "$required" >/dev/null
done

mkdir -p \
  "$site_root/static/images" \
  "$survey_root/metadata" \
  "$survey_root/artifacts/pdf" \
  "$survey_root/text" \
  "$repo_root_out/site_repo" \
  "$repo_root_out/awesome_repo" \
  "$one_hop_root"

test ! -L "$bundle_root"

fetch_file() {
  local url="$1"
  local destination="$2"
  local partial="$destination.part"
  mkdir -p "$(dirname "$destination")"
  if [[ -s "$destination" && "$refresh" -eq 0 ]]; then
    return 0
  fi
  rm -f -- "$partial"
  curl \
    --location \
    --fail \
    --silent \
    --show-error \
    --max-time 120 \
    --retry 2 \
    --retry-delay 1 \
    --user-agent 'harp-self-improving-agents-survey-capture/1.0' \
    "$url" \
    --output "$partial"
  test -s "$partial"
  mv -- "$partial" "$destination"
}

sha256_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

bytes_file() {
  wc -c < "$1" | tr -d ' '
}

capture_url() {
  local source_id="$1"
  local url="$2"
  local destination="$3"
  local status="$4"
  local claim_ceiling="$5"
  fetch_file "$url" "$destination"
  printf '%s\t%s\t%s\t%s\t%s\t%s\tsha256:%s\t%s\t%s\n' \
    "$source_id" \
    "$url" \
    "${destination#"$repo_root"/}" \
    "$observed_at" \
    "$(bytes_file "$destination")" \
    "$status" \
    "$(sha256_file "$destination")" \
    "$claim_ceiling" \
    ""
}

capture_manifest="$bundle_root/capture_manifest.tsv"
{
  printf 'source_id\turl\tlocal_path\tobserved_at\tbytes\tstatus\tsha256\tclaim_ceiling\tnote\n'
} > "$capture_manifest.part"

capture_url \
  "SIMAS-SITE" \
  "$site_url" \
  "$site_root/index.html" \
  "fetched-dated" \
  "Project-page taxonomy, bibliography topology, author framing, and dated page claims; primary linked sources own mechanisms and results" \
  >> "$capture_manifest.part"

capture_url \
  "SIMAS-SITE" \
  "${site_url}static/images/fig-si-main-001.png" \
  "$site_root/static/images/fig-si-main-001.png" \
  "fetched-dated" \
  "Same-site figure bytes only; interpretation remains tied to the captured page and paper" \
  >> "$capture_manifest.part"

capture_url \
  "SIMAS-PAPER" \
  "https://export.arxiv.org/api/query?id_list=${arxiv_id}" \
  "$survey_root/metadata/arxiv-${arxiv_id}.atom" \
  "fetched-arxiv" \
  "arXiv metadata identity, author list, abstract, categories, version, and linked project/repository comments" \
  >> "$capture_manifest.part"

capture_url \
  "SIMAS-PAPER" \
  "https://arxiv.org/abs/${arxiv_id}v1" \
  "$survey_root/metadata/arxiv-${arxiv_id}v1.html" \
  "fetched-arxiv" \
  "arXiv abstract page identity and version metadata" \
  >> "$capture_manifest.part"

capture_url \
  "SIMAS-PAPER" \
  "https://arxiv.org/pdf/${arxiv_id}v1" \
  "$survey_root/artifacts/pdf/self-improvements-modern-agentic-systems-2607.13104v1.pdf" \
  "fetched-arxiv" \
  "Survey paper text and author-reported taxonomy; no independent reproduction of linked work claims" \
  >> "$capture_manifest.part"

capture_url \
  "SIMAS-SITE-REPO" \
  "https://api.github.com/repos/${site_repo}/commits/${site_commit}" \
  "$repo_root_out/site_repo/commit-${site_commit}.json" \
  "fetched-github-api" \
  "Pinned GitHub Pages source commit metadata and file-level patch identity" \
  >> "$capture_manifest.part"

capture_url \
  "SIMAS-SITE-REPO" \
  "https://raw.githubusercontent.com/${site_repo}/${site_commit}/index.html" \
  "$repo_root_out/site_repo/index-${site_commit}.html" \
  "fetched-pinned-raw" \
  "Pinned GitHub Pages source bytes for the hub index" \
  >> "$capture_manifest.part"

capture_url \
  "SIMAS-AWESOME-REPO" \
  "https://api.github.com/repos/${awesome_repo}" \
  "$repo_root_out/awesome_repo/repository.json" \
  "fetched-github-api" \
  "Linked bibliography/update repository metadata; not source-tree inspection" \
  >> "$capture_manifest.part"

capture_url \
  "SIMAS-AWESOME-REPO" \
  "https://api.github.com/repos/${awesome_repo}/commits/main" \
  "$repo_root_out/awesome_repo/default-branch-commit.json" \
  "fetched-github-api" \
  "Default-branch commit metadata at capture time" \
  >> "$capture_manifest.part"

awesome_commit="$(jq -r '.sha' "$repo_root_out/awesome_repo/default-branch-commit.json")"
if [[ ! "$awesome_commit" =~ ^[0-9a-f]{40}$ ]]; then
  echo "could not resolve Awesome repo default-branch commit" >&2
  exit 1
fi

capture_url \
  "SIMAS-AWESOME-REPO" \
  "https://raw.githubusercontent.com/${awesome_repo}/${awesome_commit}/README.md" \
  "$repo_root_out/awesome_repo/README.md" \
  "fetched-pinned-raw" \
  "README at observed default-branch commit ${awesome_commit}" \
  >> "$capture_manifest.part"

capture_url \
  "SIMAS-AWESOME-REPO" \
  "https://raw.githubusercontent.com/${awesome_repo}/${awesome_commit}/LICENSE" \
  "$repo_root_out/awesome_repo/LICENSE" \
  "fetched-pinned-raw" \
  "License file at observed default-branch commit ${awesome_commit}" \
  >> "$capture_manifest.part"

pdftotext \
  "$survey_root/artifacts/pdf/self-improvements-modern-agentic-systems-2607.13104v1.pdf" \
  "$survey_root/text/self-improvements-modern-agentic-systems-2607.13104v1.txt"
pandoc \
  --from html \
  --to plain \
  "$site_root/index.html" \
  --output "$site_root/index.txt"
pandoc \
  --from html \
  --to plain \
  "$repo_root_out/site_repo/index-${site_commit}.html" \
  --output "$repo_root_out/site_repo/index-${site_commit}.txt"

for generated in \
  "$survey_root/text/self-improvements-modern-agentic-systems-2607.13104v1.txt" \
  "$site_root/index.txt" \
  "$repo_root_out/site_repo/index-${site_commit}.txt"
do
  generated_source_id="SIMAS-SITE-REPO"
  case "$generated" in
    "$survey_root"/*)
      generated_source_id="SIMAS-PAPER"
      ;;
    "$site_root"/*)
      generated_source_id="SIMAS-SITE"
      ;;
  esac
  printf '%s\t%s\t%s\t%s\t%s\t%s\tsha256:%s\t%s\t%s\n' \
    "$generated_source_id" \
    "local-derived" \
    "${generated#"$repo_root"/}" \
    "$observed_at" \
    "$(bytes_file "$generated")" \
    "derived-text" \
    "$(sha256_file "$generated")" \
    "Derived text for local reading; cite raw PDF or HTML for source identity" \
    "" \
    >> "$capture_manifest.part"
done

mv -- "$capture_manifest.part" "$capture_manifest"

python3 - "$repo_root" "$observed_at" <<'PY'
import csv
import hashlib
import json
import re
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urldefrag, urljoin, urlparse

repo = Path(sys.argv[1])
observed_at = sys.argv[2]
bundle = repo / "evidence/self_improving_agents_survey"
site_html = bundle / "site/index.html"
html = site_html.read_text(encoding="utf-8", errors="replace")

class LinkParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
        self.stack = []
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "a" and attrs.get("href"):
            self.stack.append({"href": attrs["href"], "text": []})
        else:
            self.stack.append(None)
    def handle_endtag(self, tag):
        if not self.stack:
            return
        current = self.stack.pop()
        if tag == "a" and current:
            self.links.append(current)
        elif current and self.stack:
            for item in reversed(self.stack):
                if item:
                    item["text"].extend(current["text"])
                    break
    def handle_data(self, data):
        for item in reversed(self.stack):
            if item:
                item["text"].append(data)
                break

parser = LinkParser()
parser.feed(html)

def clean(value):
    return re.sub(r"\s+", " ", value or "").strip()

def classify(url):
    parsed = urlparse(url)
    host = parsed.netloc.lower()
    path = parsed.path
    if host == "selfimproving-agent.github.io":
        return "same-site"
    if host == "arxiv.org":
        return "arxiv"
    if host == "openreview.net":
        return "openreview"
    if host == "github.com":
        return "github"
    if host in {"aclanthology.org", "dl.acm.org", "ieeexplore.ieee.org", "link.springer.com"}:
        return "publisher"
    if host == "huggingface.co":
        return "huggingface"
    return "project-page"

rows = []
seen = set()
for link in parser.links:
    url = urldefrag(urljoin("https://selfimproving-agent.github.io/", link["href"]))[0]
    if url in seen:
        continue
    seen.add(url)
    parsed = urlparse(url)
    if parsed.scheme not in {"http", "https"}:
        continue
    rows.append({
        "url": url,
        "host": parsed.netloc,
        "kind": classify(url),
        "anchor_text": clean("".join(link["text"])),
        "observed_at": observed_at,
        "capture_status": "inventory-only" if parsed.netloc != "selfimproving-agent.github.io" else "captured-if-same-site",
        "claim_ceiling": "Link identity and hub bibliography topology only unless a local artifact row explicitly inspects this source",
    })

with (bundle / "link_inventory.tsv").open("w", newline="", encoding="utf-8") as f:
    writer = csv.DictWriter(
        f,
        fieldnames=[
            "url",
            "host",
            "kind",
            "anchor_text",
            "observed_at",
            "capture_status",
            "claim_ceiling",
        ],
        delimiter="\t",
    )
    writer.writeheader()
    writer.writerows(rows)

summary = {
    "schema_version": 1,
    "observed_at": observed_at,
    "site_url": "https://selfimproving-agent.github.io/",
    "site_repo_commit": "d8af6607ced118351108670f823cd106649cb757",
    "link_count": len(rows),
    "link_kinds": {kind: sum(1 for row in rows if row["kind"] == kind) for kind in sorted({row["kind"] for row in rows})},
}
(bundle / "one_hop/link_summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")

def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

artifact_rows = []
for path in sorted(bundle.rglob("*")):
    if not path.is_file():
        continue
    rel = path.relative_to(repo).as_posix()
    if rel.endswith(".part"):
        continue
    if rel.endswith("artifact_inventory.tsv"):
        continue
    artifact_rows.append({
        "path": rel,
        "kind": path.relative_to(bundle).parts[0],
        "bytes": str(path.stat().st_size),
        "sha256": "sha256:" + sha256(path),
    })

with (bundle / "artifact_inventory.tsv").open("w", newline="", encoding="utf-8") as f:
    writer = csv.DictWriter(
        f,
        fieldnames=["path", "kind", "bytes", "sha256"],
        delimiter="\t",
    )
    writer.writeheader()
    writer.writerows(artifact_rows)
PY

printf 'Captured Self-Improving Agents survey evidence under %s\n' "${bundle_root#"$repo_root"/}"
