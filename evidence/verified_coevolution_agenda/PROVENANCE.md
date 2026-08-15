# Verified Coevolution Agenda Evidence Provenance

This bundle supports the `knowledge/verified_coevolution_agenda/` packet. It is
an evidence boundary, not a literature review by itself.

## Scope

- `artifacts/supplied_research_agenda.txt` preserves the user-supplied research
  agenda from the planning transcript. It is design input only and must not be
  cited as source evidence for paper mechanisms, paper results, licensing, venue
  status, or rights.
- LADDER, PRIME-RL TTRL, NSRSA, SAHOO, and Scrivens verification are captured as
  full revision-pinned arXiv records because the local capture contract treats
  these revisions as CC-BY works for this packet. Each has Atom metadata,
  abstract page HTML, arXiv semantic HTML, PDF, and deterministic
  `pdftotext -raw` text.
- The model-collapse article is captured from Nature as first-party HTML plus a
  deterministic plain-text extraction.
- Godel Agent, Mendel Godel Machine, and the historical Godel Machine arXiv
  record are captured as metadata and abstract pages only. Their full texts are
  intentionally not vendored here because the redistribution boundary is unclear
  or nonexclusive for this packet.

## Local Authority

`manifest.tsv` records the 34 source records used by the packet: 33 public
source records plus the supplied agenda input. `artifact_inventory.tsv` records
all files in this evidence bundle, including provenance, acquisition, manifest,
and receipt metadata. `capture_receipt.tsv` records the acquisition-script and
manifest digests.

Captured files under `artifacts/` are upstream or supplied bytes and should not
be rewritten for Harp branding or formatting. Curated interpretation belongs in
`knowledge/verified_coevolution_agenda/`.

## Claim Boundary

This bundle can support claims about source identity, local captured text, and
author-reported mechanisms within the cited revisions. It does not reproduce
paper experiments, validate benchmark scores, prove broad alignment properties,
or establish that model-harness coevolution is already demonstrated as a general
historical law.
