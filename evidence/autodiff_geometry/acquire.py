#!/usr/bin/env python3
"""Acquire and parse source material for the autodiff geometry curriculum."""

from __future__ import annotations

import argparse
import csv
import hashlib
import html.parser
import json
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
import zipfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
BUNDLE = ROOT / "evidence" / "autodiff_geometry"
METADATA = BUNDLE / "metadata"
TEXT = BUNDLE / "text"
PARSED = BUNDLE / "parsed"
ARTIFACTS = BUNDLE / "artifacts"

JAX_COMMIT = "0eb0f676ba22b15b9dfe29518ab0d40277b7138d"
SPIVAK_COMMIT = "ac453ea3c0a43fe2bc3ee5b144bee5d9d1cf174f"
TAO_ANALYSIS_COMMIT = "8f9e0fc5f063d0839f9b2bfc3ed9607b417877fb"

JAX_NOTEBOOK_URL = (
    "https://raw.githubusercontent.com/jax-ml/jax/"
    f"{JAX_COMMIT}/docs/notebooks/autodiff_cookbook.ipynb"
)
JAX_RENDERED_URL = "https://docs.jax.dev/en/latest/notebooks/autodiff_cookbook.html"
JAX_API_URL = f"https://raw.githubusercontent.com/jax-ml/jax/{JAX_COMMIT}/jax/_src/api.py"
SPIVAK_API_ROOT = (
    "https://api.github.com/repos/zongpingding/"
    "Calculus_On_Manifolds_Michael_Spivak/contents"
)
SPIVAK_RAW_ROOT = (
    "https://raw.githubusercontent.com/zongpingding/"
    f"Calculus_On_Manifolds_Michael_Spivak/{SPIVAK_COMMIT}"
)
SICM_COURSE_URL = "https://groups.csail.mit.edu/mac/users/gjs/6946/"
SICM_ZIP_URL = (
    "https://mitp-content-server.mit.edu/books/content/sectbyfn/books_pres_0/"
    "9579/sicm_edition_2.zip"
)
FDG_PDF_URL = (
    "https://mitp-content-server.mit.edu/books/content/sectbyfn/books_pres_0/"
    "9580/9580.pdf?dl=1"
)
TAO_RAW_ROOT = f"https://raw.githubusercontent.com/teorth/analysis/{TAO_ANALYSIS_COMMIT}"
TAO_API_ROOT = "https://api.github.com/repos/teorth/analysis/contents"


class TextExtractor(html.parser.HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self._skip_depth = 0
        self._chunks: list[str] = []
        self._title: list[str] = []
        self._in_title = False

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag in {"script", "style", "noscript"}:
            self._skip_depth += 1
        if tag == "title":
            self._in_title = True
        if tag in {"p", "div", "section", "article", "br", "li", "tr", "h1", "h2", "h3"}:
            self._chunks.append("\n")

    def handle_endtag(self, tag: str) -> None:
        if tag in {"script", "style", "noscript"} and self._skip_depth:
            self._skip_depth -= 1
        if tag == "title":
            self._in_title = False
        if tag in {"p", "div", "section", "article", "li", "tr", "h1", "h2", "h3"}:
            self._chunks.append("\n")

    def handle_data(self, data: str) -> None:
        if self._skip_depth:
            return
        stripped = " ".join(data.split())
        if not stripped:
            return
        if self._in_title:
            self._title.append(stripped)
        self._chunks.append(stripped)
        self._chunks.append(" ")

    @property
    def text(self) -> str:
        lines = []
        for line in "".join(self._chunks).splitlines():
            line = " ".join(line.split())
            if line:
                lines.append(line)
        return "\n".join(lines).strip() + "\n"

    @property
    def title(self) -> str:
        return " ".join(self._title).strip()


@dataclass(frozen=True)
class Artifact:
    path: Path
    note: str


def fetch(url: str, destination: Path) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": "Harp autodiff geometry capture"})
    with urllib.request.urlopen(request, timeout=60) as response:
        data = response.read()
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(data)
    return data


def fetch_text(url: str, destination: Path) -> str:
    data = fetch(url, destination)
    return data.decode("utf-8", errors="replace")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def write_tsv(path: Path, header: list[str], rows: list[list[object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(header)
        writer.writerows(rows)


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def parse_html_text(html: str) -> tuple[str, str]:
    parser = TextExtractor()
    parser.feed(html)
    parser.close()
    return parser.title, parser.text


def heading_for_markdown(source: str) -> str:
    for line in source.splitlines():
        if line.startswith("#"):
            return line.lstrip("#").strip() or "(untitled)"
    return "(no heading)"


def acquire_jax(artifacts: list[Artifact], parse_rows: list[list[object]]) -> None:
    notebook_path = METADATA / "jax_autodiff_cookbook.ipynb"
    notebook = json.loads(fetch_text(JAX_NOTEBOOK_URL, notebook_path))
    artifacts.append(Artifact(notebook_path, "Pinned JAX autodiff cookbook notebook."))

    rendered_path = METADATA / "jax_autodiff_cookbook.html"
    rendered_html = fetch_text(JAX_RENDERED_URL, rendered_path)
    rendered_title, rendered_text = parse_html_text(rendered_html)
    rendered_text_path = TEXT / "jax_autodiff_cookbook_rendered.txt"
    rendered_text_path.write_text(rendered_text, encoding="utf-8")
    artifacts.append(Artifact(rendered_path, "Rendered mutable JAX documentation page."))
    artifacts.append(Artifact(rendered_text_path, "Text extracted from rendered JAX documentation."))

    sections = []
    full_cells = []
    for index, cell in enumerate(notebook.get("cells", [])):
        source = "".join(cell.get("source", []))
        cell_type = cell.get("cell_type", "unknown")
        full_cells.append(f"\n\n--- cell {index} {cell_type} ---\n{source}")
        keywords = ",".join(
            keyword
            for keyword in [
                "grad",
                "argnums",
                "jvp",
                "vjp",
                "jacfwd",
                "jacrev",
                "hessian",
                "Spivak",
                "Functional Differential Geometry",
            ]
            if keyword in source
        )
        sections.append([index, cell_type, heading_for_markdown(source), len(source), keywords])
    notebook_text_path = TEXT / "jax_autodiff_cookbook_cells.txt"
    notebook_text_path.write_text("".join(full_cells).strip() + "\n", encoding="utf-8")
    sections_path = PARSED / "jax_autodiff_cookbook_sections.tsv"
    write_tsv(sections_path, ["cell_index", "cell_type", "heading", "chars", "keywords"], sections)
    artifacts.append(Artifact(notebook_text_path, "Notebook cells concatenated for source review."))
    artifacts.append(Artifact(sections_path, "Parsed JAX notebook section inventory."))

    api_path = METADATA / "jax_api.py"
    api_source = fetch_text(JAX_API_URL, api_path)
    defs = []
    for name in ["grad", "value_and_grad", "jacfwd", "jacrev", "hessian", "jvp", "vjp"]:
        match = re.search(rf"^def {name}\(", api_source, flags=re.MULTILINE)
        defs.append([name, "found" if match else "missing", api_source[: match.start()].count("\n") + 1 if match else ""])
    defs_path = PARSED / "jax_api_public_autodiff_defs.tsv"
    write_tsv(defs_path, ["symbol", "status", "line"], defs)
    artifacts.append(Artifact(api_path, "Pinned JAX API source file."))
    artifacts.append(Artifact(defs_path, "Parsed public autodiff API definition locations."))

    parse_rows.append(
        [
            "JAX-AUTODIFF-COOKBOOK",
            "notebook+rendered-html",
            rel(sections_path),
            "parsed",
            len(sections),
            f"rendered_title={rendered_title}",
            "Use section inventory to map cookbook examples into curriculum slices.",
        ]
    )
    parse_rows.append(
        [
            "JAX-API-SOURCE",
            "python-source",
            rel(defs_path),
            "parsed",
            len(defs),
            "public autodiff transformation defs located",
            "Keep as API-surface evidence only; do not infer implementation correctness.",
        ]
    )


def acquire_spivak(artifacts: list[Artifact], parse_rows: list[list[object]]) -> None:
    root_contents_path = METADATA / "spivak_repo_root_contents.json"
    chapter_contents_path = METADATA / "spivak_chapter_contents.json"
    root_contents = fetch_text(f"{SPIVAK_API_ROOT}?ref={SPIVAK_COMMIT}", root_contents_path)
    chapter_contents = fetch_text(
        f"{SPIVAK_API_ROOT}/chapter?ref={SPIVAK_COMMIT}", chapter_contents_path
    )
    artifacts.append(Artifact(root_contents_path, "Spivak repository root listing."))
    artifacts.append(Artifact(chapter_contents_path, "Spivak repository chapter listing."))

    for name in ["README.md", "LICENSE", "Calculus_On_Manifolds.tex"]:
        destination = METADATA / f"spivak_{name.replace('/', '_')}"
        fetch_text(f"{SPIVAK_RAW_ROOT}/{name}", destination)
        artifacts.append(Artifact(destination, f"Spivak repository {name}."))

    root_tex = (METADATA / "spivak_Calculus_On_Manifolds.tex").read_text(encoding="utf-8")
    outline_rows = []
    current_chapter = ""
    for line_number, line in enumerate(root_tex.splitlines(), start=1):
        chapter_match = re.search(r"\\chapter\{([^}]*)\}", line)
        if chapter_match:
            current_chapter = chapter_match.group(1)
            outline_rows.append([line_number, "chapter", current_chapter, ""])
        input_match = re.search(r"\\input\{([^}]*)\}", line)
        if input_match:
            outline_rows.append([line_number, "input", current_chapter, input_match.group(1)])
    outline_path = PARSED / "spivak_tex_outline.tsv"
    write_tsv(outline_path, ["line", "kind", "chapter", "input"], outline_rows)
    artifacts.append(Artifact(outline_path, "Parsed Spivak TeX root outline."))

    parse_rows.append(
        [
            "SPIVAK-CALCULUS-ON-MANIFOLDS-REPO",
            "repo-metadata+root-tex",
            rel(outline_path),
            "structure-parsed-content-gated",
            len(outline_rows),
            "README warns personal use; full book bytes not imported",
            "Use outline for curriculum alignment; acquire direct theorem text only after rights review.",
        ]
    )


def acquire_sicm(artifacts: list[Artifact], parse_rows: list[list[object]]) -> None:
    course_path = METADATA / "sicm_course_page.html"
    course_html = fetch_text(SICM_COURSE_URL, course_path)
    course_title, course_text = parse_html_text(course_html)
    course_text_path = TEXT / "sicm_course_page.txt"
    course_text_path.write_text(course_text, encoding="utf-8")
    artifacts.append(Artifact(course_path, "Gerald Jay Sussman SICM course page."))
    artifacts.append(Artifact(course_text_path, "Text extracted from SICM course page."))

    with tempfile.TemporaryDirectory() as tmpdir:
        zip_path = Path(tmpdir) / "sicm_edition_2.zip"
        zip_bytes = fetch(SICM_ZIP_URL, zip_path)
        listing_rows = []
        text_parts = []
        with zipfile.ZipFile(zip_path) as archive:
            for member in sorted(archive.infolist(), key=lambda item: item.filename):
                listing_rows.append([member.filename, member.file_size, member.CRC])
                if member.filename.endswith(".html"):
                    html = archive.read(member).decode("utf-8", errors="replace")
                    title, text = parse_html_text(html)
                    text_parts.append([member.filename, title, text])
        listing_path = METADATA / "sicm_edition_2_zip_listing.tsv"
        write_tsv(listing_path, ["path", "bytes", "crc32"], listing_rows)
        combined_path = TEXT / "sicm_edition_2_html_text.txt"
        combined_path.write_text(
            "\n\n".join(f"--- {name} | {title} ---\n{text}" for name, title, text in text_parts)
            + "\n",
            encoding="utf-8",
        )
        units_path = PARSED / "sicm_html_units.tsv"
        write_tsv(
            units_path,
            ["path", "title", "chars"],
            [[name, title, len(text)] for name, title, text in text_parts],
        )
        zip_digest_path = METADATA / "sicm_edition_2_zip_digest.txt"
        zip_digest_path.write_text(
            f"url\t{SICM_ZIP_URL}\nbytes\t{len(zip_bytes)}\nsha256\tsha256:{sha256_bytes(zip_bytes)}\n",
            encoding="utf-8",
        )
    artifacts.append(Artifact(listing_path, "SICM open-access zip member listing."))
    artifacts.append(Artifact(combined_path, "Text extracted from SICM HTML zip."))
    artifacts.append(Artifact(units_path, "Parsed SICM HTML unit inventory."))
    artifacts.append(Artifact(zip_digest_path, "Digest receipt for non-vendored SICM zip."))
    parse_rows.append(
        [
            "SICM-OPEN-ACCESS-HTML",
            "html-zip-derived-text",
            rel(units_path),
            "parsed",
            len(text_parts),
            f"course_title={course_title}; zip_digest_recorded_not_vendored",
            "Use chapter text to align mechanics notation after claim-specific review.",
        ]
    )


def acquire_fdg(artifacts: list[Artifact], parse_rows: list[list[object]]) -> None:
    pdf_path = ARTIFACTS / "functional_differential_geometry_9580.pdf"
    fetch(FDG_PDF_URL, pdf_path)
    artifacts.append(Artifact(pdf_path, "Open-access MIT Press FDG PDF."))
    text_path = TEXT / "functional_differential_geometry_9580.txt"
    pdftotext = shutil.which("pdftotext")
    if pdftotext:
        subprocess.run([pdftotext, "-layout", str(pdf_path), str(text_path)], check=True)
        text = text_path.read_text(encoding="utf-8", errors="replace")
        status = "parsed"
        units = text.count("\f") + 1
        observation = "pdftotext -layout"
    else:
        text_path.write_text("", encoding="utf-8")
        text = ""
        status = "blocked-parser-missing"
        units = 0
        observation = "pdftotext unavailable"
    markers = []
    for marker in ["Prologue", "Our notation", "derivative", "differential", "manifold"]:
        markers.append([marker, text.lower().count(marker.lower())])
    marker_path = PARSED / "fdg_text_markers.tsv"
    write_tsv(marker_path, ["marker", "count"], markers)
    artifacts.append(Artifact(text_path, "Text extracted from FDG PDF."))
    artifacts.append(Artifact(marker_path, "Simple FDG marker inventory for curriculum triage."))
    parse_rows.append(
        [
            "FDG-OPEN-ACCESS-PDF",
            "pdf-derived-text",
            rel(marker_path),
            status,
            units,
            observation,
            "Start with the Prologue and notation appendix before manifold-level Lean targets.",
        ]
    )


def acquire_tao_analysis(artifacts: list[Artifact], parse_rows: list[list[object]]) -> None:
    contents_path = METADATA / "tao_analysis_root_contents.json"
    fetch_text(f"{TAO_API_ROOT}?ref={TAO_ANALYSIS_COMMIT}", contents_path)
    artifacts.append(Artifact(contents_path, "Tao analysis repository root listing."))
    for name in ["README.md", "LICENSE", "Analysis.lean", "lakefile.lean", "lean-toolchain"]:
        destination = METADATA / f"tao_analysis_{name.replace('/', '_')}"
        fetch_text(f"{TAO_RAW_ROOT}/{name}", destination)
        artifacts.append(Artifact(destination, f"Tao analysis {name}."))
    analysis = (METADATA / "tao_analysis_Analysis.lean").read_text(encoding="utf-8")
    imports = []
    for line_number, line in enumerate(analysis.splitlines(), start=1):
        if line.startswith("import "):
            imports.append([line_number, line.removeprefix("import ").strip()])
    imports_path = PARSED / "tao_analysis_imports.tsv"
    write_tsv(imports_path, ["line", "import"], imports)
    artifacts.append(Artifact(imports_path, "Parsed Tao analysis top-level Lean imports."))
    parse_rows.append(
        [
            "TAO-ANALYSIS-REPO",
            "lean-repo-guidance",
            rel(imports_path),
            "structure-parsed",
            len(imports),
            "top-level imports and project metadata captured",
            "Use as proof-engineering guidance, not as a direct dependency.",
        ]
    )


def write_manifest(parse_rows: list[list[object]]) -> None:
    observed = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    rows = [
        [
            "JAX-AUTODIFF-COOKBOOK",
            JAX_NOTEBOOK_URL,
            "captured-pinned-git",
            observed,
            "Cookbook notation and API descriptions for autodiff transformations.",
            "Notebook, rendered page, extracted text, and section inventory are captured locally.",
        ],
        [
            "JAX-API-SOURCE",
            JAX_API_URL,
            "captured-pinned-git",
            observed,
            "Definition locations for public autodiff API entry points.",
            "API source is local evidence for surface identity, not implementation correctness.",
        ],
        [
            "SPIVAK-CALCULUS-ON-MANIFOLDS-REPO",
            f"https://github.com/zongpingding/Calculus_On_Manifolds_Michael_Spivak/tree/{SPIVAK_COMMIT}",
            "metadata-captured-content-gated",
            observed,
            "Bibliographic/source availability and TeX outline for Spivak notation lineage.",
            "README warns personal use; full book text is not vendored.",
        ],
        [
            "SICM-COURSE-PAGE",
            SICM_COURSE_URL,
            "captured-html",
            observed,
            "Public-course pointer to SICM second edition and open-access book route.",
            "Course page is mutable dated evidence.",
        ],
        [
            "SICM-OPEN-ACCESS-HTML",
            SICM_ZIP_URL,
            "parsed-open-access-html",
            observed,
            "Open-access SICM text for notation and mechanics curriculum alignment.",
            "Raw zip is not vendored; digest and listing are captured, parsed text is local.",
        ],
        [
            "FDG-OPEN-ACCESS-PDF",
            FDG_PDF_URL,
            "captured-open-access-pdf",
            observed,
            "FDG notation and differential geometry curriculum alignment.",
            "PDF and derived text are captured locally.",
        ],
        [
            "TAO-ANALYSIS-REPO",
            f"https://github.com/teorth/analysis/tree/{TAO_ANALYSIS_COMMIT}",
            "captured-pinned-git",
            observed,
            "Proof-engineering guidance from Tao's analysis formalization effort.",
            "Captured as guidance only; Harp stays standalone.",
        ],
    ]
    write_tsv(
        BUNDLE / "manifest.tsv",
        ["source_id", "url", "status", "observed_at", "claim_ceiling", "note"],
        rows,
    )
    write_tsv(
        BUNDLE / "source_parse_report.tsv",
        [
            "source_id",
            "source_kind",
            "local_parse_artifact",
            "parse_status",
            "parsed_units",
            "parser_observation",
            "next_action",
        ],
        parse_rows,
    )


def write_inventory(artifacts: list[Artifact]) -> None:
    inventory_rows = []
    for artifact in sorted(artifacts, key=lambda item: rel(item.path)):
        inventory_rows.append(
            [
                rel(artifact.path),
                artifact.path.stat().st_size,
                f"sha256:{sha256_file(artifact.path)}",
                artifact.note,
            ]
        )
    write_tsv(BUNDLE / "artifact_inventory.tsv", ["path", "bytes", "sha256", "note"], inventory_rows)


def write_receipt() -> None:
    values = {
        "repository": "Harp",
        "source_id": "AUTODIFF-GEOMETRY",
        "capture_state": "captured-and-parsed",
        "captured_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace(
            "+00:00", "Z"
        ),
        "acquisition_script_sha256": f"sha256:{sha256_file(BUNDLE / 'acquire.py')}",
        "manifest_sha256": f"sha256:{sha256_file(BUNDLE / 'manifest.tsv')}",
        "artifact_inventory_sha256": f"sha256:{sha256_file(BUNDLE / 'artifact_inventory.tsv')}",
        "source_parse_report_sha256": f"sha256:{sha256_file(BUNDLE / 'source_parse_report.tsv')}",
        "rights_boundary": (
            "JAX, SICM, FDG, and Tao analysis bytes are local evidence; "
            "Spivak full book text remains content-gated by repository warning."
        ),
    }
    with (BUNDLE / "capture_receipt.tsv").open("w", encoding="utf-8") as handle:
        for key, value in values.items():
            handle.write(f"{key}\t{value}\n")


def acquire() -> None:
    for directory in [METADATA, TEXT, PARSED, ARTIFACTS]:
        directory.mkdir(parents=True, exist_ok=True)
    artifacts: list[Artifact] = [Artifact(BUNDLE / "acquire.py", "Acquisition and parser script.")]
    parse_rows: list[list[object]] = []
    acquire_jax(artifacts, parse_rows)
    acquire_spivak(artifacts, parse_rows)
    acquire_sicm(artifacts, parse_rows)
    acquire_fdg(artifacts, parse_rows)
    acquire_tao_analysis(artifacts, parse_rows)
    write_manifest(parse_rows)
    artifacts.extend(
        [
            Artifact(BUNDLE / "manifest.tsv", "Source acquisition manifest."),
            Artifact(BUNDLE / "source_parse_report.tsv", "Per-source parser status report."),
        ]
    )
    write_inventory(artifacts)
    write_receipt()


def verify() -> None:
    inventory = read_tsv(BUNDLE / "artifact_inventory.tsv")
    for row in inventory:
        path = ROOT / row["path"]
        if not path.is_file():
            raise SystemExit(f"missing artifact: {row['path']}")
        if path.stat().st_size != int(row["bytes"]):
            raise SystemExit(f"size mismatch: {row['path']}")
        if f"sha256:{sha256_file(path)}" != row["sha256"]:
            raise SystemExit(f"digest mismatch: {row['path']}")

    receipt = {}
    for line in (BUNDLE / "capture_receipt.tsv").read_text(encoding="utf-8").splitlines():
        key, value = line.split("\t", 1)
        receipt[key] = value
    for key, relative in [
        ("acquisition_script_sha256", "acquire.py"),
        ("manifest_sha256", "manifest.tsv"),
        ("artifact_inventory_sha256", "artifact_inventory.tsv"),
        ("source_parse_report_sha256", "source_parse_report.tsv"),
    ]:
        expected = receipt.get(key)
        actual = f"sha256:{sha256_file(BUNDLE / relative)}"
        if expected != actual:
            raise SystemExit(f"receipt digest mismatch for {relative}: {expected} != {actual}")

    parse_rows = read_tsv(BUNDLE / "source_parse_report.tsv")
    statuses = {row["source_id"]: row["parse_status"] for row in parse_rows}
    required = {
        "JAX-AUTODIFF-COOKBOOK": "parsed",
        "JAX-API-SOURCE": "parsed",
        "SICM-OPEN-ACCESS-HTML": "parsed",
        "FDG-OPEN-ACCESS-PDF": "parsed",
        "TAO-ANALYSIS-REPO": "structure-parsed",
    }
    for source_id, status in required.items():
        if statuses.get(source_id) != status:
            raise SystemExit(f"{source_id} parse status is {statuses.get(source_id)!r}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true", help="verify checked-in artifacts offline")
    args = parser.parse_args()
    if args.verify:
        verify()
    else:
        acquire()
    return 0


if __name__ == "__main__":
    sys.exit(main())
