# Harp Lean workflow

Harp keeps Lean work under [`formalization/lean/`](../formalization/lean/). Contributors should treat [`mise.toml`](../mise.toml), [`formalization/lean/lean-toolchain`](../formalization/lean/lean-toolchain), [`formalization/lean/lakefile.toml`](../formalization/lean/lakefile.toml), [`scripts/harp_xdg_env.sh`](../scripts/harp_xdg_env.sh), and the Lean cache rules in [`AGENTS.md`](../AGENTS.md) as the retained workflow, not ad hoc `lake` commands.

## What is pinned

- The toolchain is pinned to Lean `v4.32.1` in [`formalization/lean/lean-toolchain`](../formalization/lean/lean-toolchain).
- The main dependency pin is `mathlib` `v4.32.1` in [`formalization/lean/lakefile.toml`](../formalization/lean/lakefile.toml); the resolved manifest is retained in [`formalization/lean/lake-manifest.json`](../formalization/lean/lake-manifest.json).
- `mise` tasks export the task-scoped Lean environment from [`scripts/harp_xdg_env.sh`](../scripts/harp_xdg_env.sh) and then call [`scripts/check_lean_library.sh`](../scripts/check_lean_library.sh). That script is the repository's Lean gate: it scans sources for forbidden placeholders, checks that required cached Mathlib `.olean` files already exist, and only then runs `lake --try-cache build`.

## Cache and worktree rules

- The primary checkout owns the canonical machine-local `formalization/lean/.lake` directory. Feature worktrees must point their own `formalization/lean/.lake` symlink back to that primary checkout, as enforced by `mise run lean-env` in [`mise.toml`](../mise.toml).
- The shared dependency cache is machine-local, not worktree-local. `lean-env` requires the primary checkout's `formalization/lean/.lake/packages` symlink to resolve into the XDG-backed cache root from [`scripts/harp_xdg_env.sh`](../scripts/harp_xdg_env.sh).
- Normal proof work assumes that cache is already warm. [`scripts/check_lean_library.sh`](../scripts/check_lean_library.sh) refuses to rebuild common dependencies when cached Mathlib artifacts are missing. The explicit hydrator is `mise run lean-cache`, and [`AGENTS.md`](../AGENTS.md) says not to run it unless the project owner asks for cache maintenance.
- Shared `.lake/build` artifacts are also treated as a coordinated resource. The repo guidance in [`AGENTS.md`](../AGENTS.md) requires worktree isolation and warns that expensive shared-output gates should only run on a frozen candidate.

## Which Lean checks to run

- Use the narrowest retained task that matches the surface you changed: `lean-training`, `lean-foundations`, `lean-nng4`, `lean-autodiff`, `lean-crouzeix`, `lean-crouzeix-textbook`, `lean-crouzeix-jin`, `lean-crouzeix-ls`, or `lean-crouzeix-harp` in [`mise.toml`](../mise.toml).
- Route-specific Crouzeix targets build the aggregate entrypoints [`formalization/lean/CrouzeixJin.lean`](../formalization/lean/CrouzeixJin.lean), [`formalization/lean/CrouzeixLoristSchwenninger.lean`](../formalization/lean/CrouzeixLoristSchwenninger.lean), and [`formalization/lean/CrouzeixHarp.lean`](../formalization/lean/CrouzeixHarp.lean). Their preflight and import-closure rules come from [`scripts/check_lean_library.sh`](../scripts/check_lean_library.sh), which keeps the Jin, Lorist-Schwenninger, and Harp routes isolated except for the explicitly allowed shared support modules.
- When a changed surface is wider than a recognized library or route, the shared gate is `mise run lean-all`. [`docs/agents/verification.md`](../docs/agents/verification.md) and [`scripts/verify_changes.py`](../scripts/verify_changes.py) make that the fallback for proof code or config that does not map to a narrower target.
- `mise run verify` is the ordinary changed-surface landing gate. `mise run verify-all` is the full release gate and includes textbook publication, Rust, `lean-all`, the Cauchy audit, and repository verification.

## What Lean evidence means here

- A local Lean pass means the pinned toolchain and retained dependency surface accepted the checked declarations. It does not by itself establish peer review, upstream publication status, or prose-to-proof correspondence. [`knowledge/crouzeix_textbook/source_registry.md`](../knowledge/crouzeix_textbook/source_registry.md) and [`knowledge/crouzeix_textbook/claim_evidence_ledger.md`](../knowledge/crouzeix_textbook/claim_evidence_ledger.md) set those ceilings explicitly.
- Route receipts under [`evidence/crouzeix_conjecture/routes/`](../evidence/crouzeix_conjecture/routes/) and [`evidence/crouzeix_conjecture/local_formalization/routes/`](../evidence/crouzeix_conjecture/local_formalization/routes/) are the retained proof evidence for the Crouzeix routes. The current retained receipts for Jin, Lorist-Schwenninger, and Harp each record the pinned toolchain, candidate commit and tree, cache identity, allowed axioms, build logs, provider audit, and axiom audit.
- Read those receipts as bounded claims about the exact checked targets such as [`formalization/lean/Crouzeix/Jin/Terminal.lean`](../formalization/lean/Crouzeix/Jin/Terminal.lean), [`formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean`](../formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean), and [`formalization/lean/Crouzeix/Harp/MainTheorem.lean`](../formalization/lean/Crouzeix/Harp/MainTheorem.lean). The textbook ledger is explicit that the Harp finite-horizon route is a local Harp-derived route, not a source-derived Jin or Lorist-Schwenninger result.
- The textbook aggregate [`formalization/lean/CrouzeixTextbook.lean`](../formalization/lean/CrouzeixTextbook.lean) imports the generated correspondence surface in [`formalization/lean/CrouzeixTextbook/Correspondence.lean`](../formalization/lean/CrouzeixTextbook/Correspondence.lean) and can export a bounded external receipt through [`formalization/lean/CrouzeixTextbook/ExportReceipt.lean`](../formalization/lean/CrouzeixTextbook/ExportReceipt.lean). That receipt is only for the checked declaration roster it names.

## How Lean relates to the retained textbook and PDFs

- The Lean proof sources stay in [`formalization/lean/CrouzeixTextbook/`](../formalization/lean/CrouzeixTextbook/) and the technical prose authority stays in [`knowledge/crouzeix_textbook/`](../knowledge/crouzeix_textbook/). For example, the retained Harp textbook chapter is paired between [`knowledge/crouzeix_textbook/part_06_constant_two_routes/36_harp_finite_horizon_proof.md`](../knowledge/crouzeix_textbook/part_06_constant_two_routes/36_harp_finite_horizon_proof.md) and [`formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean`](../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean).
- `mise run crouzeix-textbook-publication` is the retained bridge between Lean and Rust-owned publication ledgers: it compiles a fresh temporary Lean receipt, then runs Harp's `crouzeix-textbook publish` and `crouzeix-textbook check` steps, and finally checks projected counts. That flow is defined in [`mise.toml`](../mise.toml).
- The PDF tooling in [`tools/textbook_pdf/README.md`](../tools/textbook_pdf/README.md) reads canonical Markdown and requires one matching `ChapterNN.lean` file per chapter, but it does not compile Lean and does not treat a source link as proof certification. Its job is publication packaging, not proof verification.

When writing or reviewing Lean-facing docs in this repository, state the exact task or receipt that supports a claim, and stop at that claim boundary.
