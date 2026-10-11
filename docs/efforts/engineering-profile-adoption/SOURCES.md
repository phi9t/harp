# Inputs and source anchors

Repository source read: Corenius `b7bee20e00c822feec48362a1dbd8d62fe9c3641`.
Learning changes: lab Candidate `00f7f2ee94557d4f3cb9402d6d519d2a9798283c`.
This planning update makes no claim to reverify rolling external links archived
in the originals. The repository anchors below establish current compatibility;
future-contract choices in SPEC.md are proposals, not implementation findings.

## Verbatim inputs

| Archived file | Original input | SHA-256 |
| --- | --- | --- |
| [original/pasted-design.md](original/pasted-design.md) | Session attachment `Pasted text.txt` | `97c6264f0887a11e776547c856bb93e730c16fa79f14250c490d602645314dcc` |
| [original/DESIGN.md](original/DESIGN.md) | Session attachment `DESIGN.md` | `c3886eac4e501caa37311fc5ed7a3448cf89cd27c1a17479176d4a2de76d960d` |

These are exact-byte copies, without new headings, corrected links, whitespace
normalization or editorial changes. Diff whitespace checks exclude only these
archival files; SHA-256 and byte comparison verify their preservation.

## Existing machinery to reuse

- [Domain vocabulary](../../../CONTEXT.md): IntentRevision, AcceptanceContract,
  ExecutionScope, Attempt, Quarantine, Mechanism Promotion and Qualification Rung.
- [Retained controller decision](../team-control-plane/decisions/20261004-owner-sqlite-controller-first-supersession.md)
  and [portfolio specification](../controller-portfolio-stabilization/SPEC.md):
  I2 product core, I3 parked; promotion of mechanisms rather than wholesale imports.
- Concrete retained seams: `crates/sqlite-controller/src/git.rs` (`recover_git`,
  outstanding promotion reconciliation), `git_workspace.rs` (private workspaces),
  and `codex.rs` (unknown-start history reconciliation). The parked source
  `crates/team-control-plane/src/admission.rs`, `storage.rs`, `auth.rs` and
  `temporal.rs` supplies the tested admission/Bridge mechanisms. Qualify these mechanisms
  before proposing replacements; current tests are not blanket production proof.
- [I2 specification](../sqlite-controller/SPEC.md),
  [current assignment](../sqlite-controller/CURRENT.json) and
  [controller source](../../../crates/sqlite-controller/README.md): sole writer,
  durable execution, native identities, unknown effects, workspace and Git seams.
- [I3 retained source](../../../crates/team-control-plane/README.md) and
  [kernel fixtures](../../../crates/team-control-plane/tests/kernel.rs),
  [Temporal fixtures](../../../crates/team-control-plane/tests/temporal.rs):
  bounded evidence sources for admission, fencing, waits and replay; not current
  product assignments or blanket accepted code.
- [Current landing tool](../../../tools/agents/land.py) and
  [tracker](../../../tools/agents/tracker.py): frozen exact tip, conditional
  trunk advance, accepted-and-landed dependency check; proposed protected bare
  repository/receipt integration still needs qualification.
- [Independent benchmark authority ADR](../../adr/0001-controller-backed-benchmark-authority.md):
  grading remains outside controller orchestration; disposable auto-landing only.
- [Five learning labs](../engineering-profile-labs/README.md) and
  [bounded evidence](../engineering-profile-labs/evidence/summary.md): fresh proof
  and explicit operational/native gaps used to order the future frontier.
- Trondheim profile at `017457a10b2bb3299466f0c5d7c2d701902cef1f`,
  `docs/agents/engineering-execution.md`, `trondheim/engineering/planner.py`:
  compile pinned proposals and project supplied state; no production authority.
  This is a cross-repository provenance pointer, not a runtime sibling import.
