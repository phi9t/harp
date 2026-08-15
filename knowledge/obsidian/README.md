# Harp Obsidian navigation assets

Open the repository root as an Obsidian vault. This directory contains
portable navigation projections:

- `harp_knowledge.base` filters maintained Markdown under `knowledge/`;
- `harp_knowledge_map.canvas` is a small route map, not a claim graph.

These assets do not introduce another technical-prose authority. Maintained
technical prose belongs in `knowledge/`; captured sources belong in
`evidence/`. Graph edges and Base membership are navigation only, not
evidence.

To install the reviewed core-only workspace profile into a specific vault, run:

```sh
python3 tools/obsidian/apply_profile.py --vault /absolute/path/to/harp
```

The installer writes only its allowlisted files below the chosen vault's
ignored `.obsidian/` directory. It is create-only by default; use `--replace`
only to update those same portable files. It never reads arbitrary personal
Obsidian state.

Validate the committed navigation assets without Obsidian installed:

```sh
python3 scripts/validate_obsidian_assets.py
```
