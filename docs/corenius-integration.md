# Consolidated runtime and retained source

The selected lightweight replay runtime and the coding lifecycle are first-party
code under `durable/`. There is one SQLite execution journal and one admission,
capacity, cancellation, recovery, verification and approved landing chain.
Codex is the first provider for the consolidated qualification. Trae's adapter and
prior evidence are retained for later integration. There is no Temporal or
PostgreSQL execution dependency.

`durable/intent/` binds committed Markdown records and acceptance inputs;
`durable/collaboration/` binds assignments without dispatching them.
`durable/git/` owns candidate identity, independent checks, exact approval and
physical landing. The public command is `harp durable`.

`insula/` owns capsules, native rootfs identity and cgroup containment.
`insula/supervisor/` owns service lifecycle through the same kernel process gate.
Its service-operation journal is separate from the agent-job journal and does not
schedule agent work. `runner/` is SSH transport to the same runtime, with private
ignored host inventories. Read the [operator guide](durable-runtime.md) before
preparing a profile or reconciling an interrupted attempt.

## Preserved workloads and history

- `lab/benchmarks/` retains task fixtures, graders, readiness tasks, harnesses,
  inventories and results. `studies/` preserves the original effort trees.
- `lab/soma/` contains the Emacs workload and gym; `ai_lsp/` preserves the separately
  sourced AI/LSP benchmark work.
- `lab/math/` contains mathematical work, Lean libraries and proof reproduction.
- `lab/sicp/` contains the evaluator workload.

Historical captures retain their original names, statements and source bytes;
they do not grant authority to retired launchers or establish qualification for
new code. The former imported wrapper directory and duplicate runtime package
are removed. Original source branch and dirty-worktree snapshot commits remain
in Git ancestry.

The [absorption record](../evidence/provenance/absorption.json) accounts for every
original imported path, including explicit adaptations and reasoned retirement
of superseded controllers. It forbids silently retiring benchmark or study files.
Soma and the supplied supervisor carry their own source maps and upstream license
status. No repository-wide license is inferred.

```sh
python3 scripts/verify_absorption.py
python3 lab/soma/verify_preservation.py
python3 lab/math/relocation.py
```

New mixed-language Bazel, Triton compilation and training workloads remain
architecture follow-ups in the [migration tickets](../.scratch/repo-architecture/spec.md).
