"""Check compiled reading PDFs and their accompanying source snapshots."""
import hashlib
import json
import logging
from pathlib import Path
from urllib.parse import unquote, urlsplit

import pdfplumber
from pypdf import PdfReader

logging.getLogger("pdfminer").setLevel(logging.ERROR)
root = Path(__file__).resolve().parents[2]
output = root / "output/pdf"
manifest = json.loads((output / "build-manifest.json").read_text())
assert manifest["schema_version"] == 2
assert manifest["release_validation"] is True
assert len(manifest["sources"]) == 38
assert all(source["origin"] == "filesystem-candidate" for source in manifest["sources"])
candidate = sorted(({"path": s["path"], "sha256": s["sha256"]} for s in manifest["linked_sources"]), key=lambda s: s["path"])
assert hashlib.sha256(json.dumps(candidate, ensure_ascii=False, separators=(",", ":")).encode()).hexdigest() == manifest["candidate_sha256"]
report = {"files": [], "snapshot_count": len(manifest["linked_sources"])}

for source in manifest["linked_sources"]:
    data = (output / source["path"]).read_bytes()
    assert hashlib.sha256(data).hexdigest() == source["sha256"], source["path"]

snapshots = {source["path"]: source["sha256"] for source in manifest["linked_sources"]}
for source in manifest["sources"]:
    assert snapshots["sources/" + source["path"]] == source["sha256"]

for item in manifest["outputs"]:
    assert not item["unresolved_references"]
    path = output / item["file"]
    assert hashlib.sha256(path.read_bytes()).hexdigest() == item["sha256"]
    reader = PdfReader(path)
    assert len(reader.pages) == item["pages"]
    for key, expected in item["contents_pages"].items():
        destination = reader.named_destinations.get("/" + key)
        assert destination is not None, key
        assert reader.get_destination_page_number(destination) + 1 == expected, key
    links = 0
    for number, page in enumerate(reader.pages, 1):
        text = page.extract_text() or ""
        assert len(text.strip()) > 40, (item["file"], number, "near-empty page")
        for ref in page.get("/Annots", []):
            annotation = ref.get_object()
            action = annotation.get("/A")
            if not action:
                continue
            uri = action.get("/URI")
            if not uri:
                continue
            parsed = urlsplit(str(uri))
            assert parsed.scheme != "file", uri
            if not parsed.scheme:
                target = (output / unquote(parsed.path)).resolve()
                assert target.is_relative_to(output.resolve()) and target.is_file(), uri
                links += 1
    assert links == item["portable_source_links"]
    geometry = []
    with pdfplumber.open(path) as document:
        for number, page in enumerate(document.pages, 1):
            for char in page.chars:
                if not char["text"].strip():
                    continue
                if char["x0"] < -1 or char["x1"] > page.width + 1 or char["top"] < -1 or char["bottom"] > page.height + 1:
                    geometry.append({"page": number, "text": char["text"], "bbox": [char["x0"], char["top"], char["x1"], char["bottom"]]})
            page.close()
    assert not geometry, geometry[:10]
    report["files"].append({"file": item["file"], "pages": len(reader.pages), "relative_source_links": links, "contents_destinations": len(item["contents_pages"]), "outside_page_glyphs": 0})

print(json.dumps(report, indent=2))
