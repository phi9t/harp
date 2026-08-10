# Self-Improving Agents survey evidence provenance

This bundle captures the public survey hub for `Self-Improvements in Modern
Agentic Systems: A Survey` and keeps the evidence tiers separate:

- `SIMAS-PAPER`: arXiv `2607.13104v1`, including Atom metadata, abstract HTML,
  PDF bytes, and local extracted text.
- `SIMAS-SITE`: the dated project page at
  `https://selfimproving-agent.github.io/` plus same-site figure bytes.
- `SIMAS-SITE-REPO`: the GitHub Pages source repository pinned at
  `d8af6607ced118351108670f823cd106649cb757`.
- `SIMAS-AWESOME-REPO`: the linked bibliography/update repository metadata,
  default-branch commit observation, README, and license.

`capture_manifest.tsv` records each local artifact with URL, local path,
observation time, byte count, SHA-256, status, and claim ceiling.
`artifact_inventory.tsv` is a path-oriented digest inventory for every file in
this bundle. `link_inventory.tsv` records the one-hop links observed in the hub.

The one-hop closure is intentionally bounded. The project page links hundreds
of papers, repositories, benchmark pages, and community URLs. This packet
records those links as topology evidence but does not crawl child
bibliographies, clone full source trees, or promote linked works into
mechanism evidence unless a local artifact row explicitly says so.

Captured upstream files below `site/`, `survey/`, and `repos/` are source
artifacts. Do not rewrite them for formatting or branding. Derived text files
are local reading aids and should cite the raw PDF or HTML for source identity.

Missing or mutable evidence remains explicit. The survey and hub can support
taxonomy, bibliography topology, and author framing. Linked captures support
source identity by default, not correctness, venue status, benchmark
reproduction, implementation behavior, or RSI efficacy.
