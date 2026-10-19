## The design I would adopt

**Build a Matt-style engineering profile on Corenius, with one collaboration controller and one protected path into trunk.**

Matt’s skills should guide investigation, design, and implementation. They should **not independently own task claims, agent dispatch, worktree creation, or merging** once Corenius is managing the work.

The division is:

> **Agents own the engineering approach. Git preserves the work definition. Corenius owns execution authority and coordination. Independent verification controls what reaches trunk.**

I would retain PostgreSQL and Temporal: your October 2 Corenius RFC explicitly selects them, and the October 7 scheduling refinement preserves that decision. Git-backed tickets complement that baseline rather than replacing the runtime authority store. Both documents describe proposed architecture, not an audited implementation. corenius-design-v0.1 DESIGN

I’ve put the detailed design, proposed configuration, agent instructions, ticket templates, and a small Git-protocol test suite into a bundle:

**DESIGN.md[Full design](sandbox:/mnt/data/corenius-engineering-profile/DESIGN.md)** · **corenius-engineering-profile.zip[Design, templates, and tests](sandbox:/mnt/data/corenius-engineering-profile.zip)**

---

## 1. Concrete components

I would choose this stack—not add another general-purpose agent framework.

| Subsystem | Concrete choice | Responsibility |
|---|---|---|
| Engineering guidance | Pinned, selectively adapted **Matt Pocock skills** | Research, decisions, specifications, decomposition, implementation discipline |
| Ticket and decision authoring | **Markdown with YAML front matter, tracked in the project’s Git repository** | Human-readable, reviewable work definitions |
| Ticket adapter | A small **`GitMarkdownWorkStore` module inside Corenius** | Parse documents, validate references, identify immutable revisions |
| Runtime authority | **Corenius broker + PostgreSQL** | Approved revisions, grants, attempts, claims, budgets, acceptance, operation journal |
| Durable coordination | **Temporal** | Waiting, wakeups, bounded retries, recovery and reconciliation |
| Initial agent runtime | **Codex app-server over stdio**, behind a supervised gateway | Native threads, turns, tools and context management |
| Physical execution | **Insula** | Workspace/environment construction, process supervision, execution receipts |
| Fleet information | **Armada** | Target discovery, reachability and capability evidence—not another executor |
| Workspace management | **Native Git CLI**, behind a `TaskWorkspace` adapter | Private attempt repositories and managed worktrees |
| Integration | **Gate-owned bare Git repository + serial verification/landing lane** | Conditional promotion of the exact verified revision |
| Evidence | Project-native tests plus an independent reviewer using Matt’s review criteria | Behavioral, numerical, performance and architectural checks |
| Large artifacts | **SHA-256-addressed filesystem store on persistent, backed-up storage** | Logs, experiment results, bundles, checkpoints and prototypes |
| Operator interface | **CLI/JSON + editable Markdown + existing runtime/tmux views** | Inspect, steer, approve and recover—without requiring a new dashboard |

The adapter names above are **proposed components**, not claims about already-installed commands.

This should remain a modular system, not twelve services. Your saved design already calls for a broker, Temporal worker, PostgreSQL, artifact storage, and the necessary gateways—not Redis, Kafka, a graph database, or a service per entity. corenius-design-v0.1

For the first runtime adapter, Codex app-server is a concrete fit because its documented interface exposes native thread/turn lifecycle and structured events over stdio. Corenius should adapt that interface rather than scrape terminal output or recreate a model loop. [OpenAI Developers](https://developers.openai.com/codex/app-server)

**Default deployment:** one lead, one implementing worker, and separately scheduled verification. The same design supports multiple implementation workers, but parallelism is enabled by the mission’s grant and the independence of the work—not by an always-on agent organizational chart.

---

## 2. Git-backed tickets: yes, but not Git-backed leases

I would use Git-managed files for ticketing. The important distinction is between **the definition of work** and **the current execution of work**.

| Git owns | PostgreSQL owns |
|---|---|
| Intent, specifications and acceptance definitions | Which exact revision is currently authorized |
| Research conclusions and architectural decisions | Active attempts and runtime bindings |
| Ticket descriptions and proposed dependencies | Claims, leases and resource reservations |
| Durable context and evidence references | Current budgets and permissions |
| Optional historical acceptance exports | Authoritative acceptance and promotion records |

This is not two independently editable copies of the same truth. **Git stores the authored bytes; PostgreSQL records their operational applicability.** Your RFC already makes that distinction for approved contract artifacts. corenius-design-v0.1

Two agents editing `Status: claimed` in different branches have not acquired a mutual-exclusion lock. Consequently, I would adapt Matt’s local-file tracker rather than use its status-line claiming procedure unchanged. Matt’s setup supports a custom tracker documented in `docs/agents/issue-tracker.md`, which provides the appropriate extension point. [GitHub](https://github.com/mattpocock/skills/blob/main/skills/engineering/setup-matt-pocock-skills/issue-tracker-local.md)

### A deliberately small authoring layout

```text
AGENTS.md
docs/agents/
  issue-tracker.md
  execution.md
.corenius/project.toml

work/
  M-recovery/
    map.md
    spec.md
    decisions/
      D-promotion-recovery.md
    tickets/
      T-conditional-landing.md
      T-reconcile-interrupted-landing.md
```

Not every task creates every file. **A bounded fix needs one ticket.** A map is useful for unresolved decisions; a spec is useful for a substantial behavioral contract. Existing canonical documentation locations take precedence over this illustrative layout.

A ticket should look approximately like this:

```markdown
---
id: T-reconcile-interrupted-landing
mission: M-recovery
kind: implementation
depends_on:
  - T-conditional-landing
---

# Recover an interrupted local landing

## Outcome
After a controller restart, determine whether a prepared landing occurred
without applying it twice or incorrectly reporting success.

## Invariants
- Only the integration gate may advance trunk.
- Acceptance applies to an immutable integration candidate.
- An uncertain operation blocks conflicting promotion.

## Acceptance
Inject interruption before and after the Git update.
Demonstrate correct reconciliation, stale-base rejection, and preservation
of unrelated operator changes.

## Context
See the mission specification and D-promotion-recovery.

## Engineering discretion
Choose the implementation and test strategy.
Escalate changes to the invariants or protected authority.
```

No prescribed file edits, mandatory twenty-step plan, or guessed implementation architecture.

At admission, Corenius pins the ticket, referenced specification, acceptance recipe and relevant policy versions. An attempt does not follow floating links whose meaning can change halfway through execution.

### One migration caveat for Harp

Harp’s current contributor guide already identifies **Kata as its local intent ledger**. I would not silently add a second writable tracker. For Harp, either retain Kata behind the adapter initially or explicitly migrate identities/history and update the repository guidance. The same guide also requires explicit permission to push. 

For newly enrolled projects, I would select the Git-Markdown backend directly.

---

## 3. Keep Matt’s workflow, replace its execution ownership

The workflow should be available as:

**Wayfinding ⇄ research/prototype ⇄ decisions → specification → ticket frontier → implementation → verification → trunk**

But it should be a **graph of useful activities**, not a mandatory waterfall through every skill.

Here is the exact adaptation I would make.

| Matt capability | Preserve | Change for this system |
|---|---|---|
| `wayfinder` | Decision map, unresolved questions, exploration frontier | Claims and execution state come from Corenius |
| `research` | Investigation and evidence gathering | Findings become durable artifacts/decisions; no automatic push |
| `prototype` | Cheap experiments that answer a concrete question | Experimental workspace; no direct production promotion |
| `grill-with-docs` | Challenge assumptions and expose consequential choices | Human-owned decisions genuinely wait for the human |
| `to-spec` | Capture intended behavior and important constraints | Pin the accepted revision; do not freeze incidental implementation guesses |
| `to-tickets` | Vertical, independently verifiable slices and dependency edges | Create canonical Git tickets; admit graph revisions through Corenius |
| `implement` | Agent-owned implementation and testing | Operate inside the assigned workspace and authority envelope |
| `implement-spec` | Ready-frontier reasoning and delegation | **Replace the orchestration body with the collaboration controller** |
| `code-review` | Specification fidelity and architectural/code standards | Review immutable candidates, without merge authority |

Matt’s wayfinder is especially compatible with this: its map represents decisions and uncertainty rather than a complete implementation plan. Its prototype workflow is explicitly experimental, which is another reason not to treat a prototype as production-ready evidence. [GitHub](https://raw.githubusercontent.com/mattpocock/skills/main/skills/engineering/wayfinder/SKILL.md)

### The substantial change: no stock `implement-spec` integration branch

Matt’s current `implement-spec` accumulates worker results on an integration branch, with workers starting from and catching up to that branch, followed by final review. That is not the integration model I would use here. [GitHub](https://github.com/mattpocock/skills/blob/main/skills/engineering/implement-spec/SKILL.md)

Instead:

> **Each accepted vertical slice reaches trunk. The next dependent slice starts from accepted trunk.**

Keep Matt’s decomposition philosophy; replace its branch orchestration. Its ticketing guidance already favors tracer-bullet slices and expand/migrate/contract for broad changes, which fits this direction. [GitHub](https://github.com/mattpocock/skills/blob/main/skills/engineering/to-tickets/SKILL.md)

I would maintain a small, pinned local skill profile with these adaptations made explicitly. Do not load an unchanged orchestration skill and hope an unrelated paragraph in `AGENTS.md` consistently overrides every conflicting instruction.

There should be **one effective implementation coordinator**, not Matt spawning a team inside Corenius spawning another team.

---

## 4. Collaboration: one engineering lead, replaceable attempts

The lead should make engineering decisions. The broker should make admission decisions.

**The lead decides:** which uncertainty matters, whether to prototype, how to decompose the next useful frontier, which work can proceed concurrently, and when the strategy needs revision.

**The broker decides:** whether the proposed operation is authorized, whether its dependencies are satisfied, whether capacity and budget exist, whether its workspace/conflict reservations are valid, and whether its result is eligible for acceptance.

This preserves the autonomy you want without making the model itself the permission system.

### Dispatch rules

A coding ticket becomes dispatchable only when its required predecessor contributions are **accepted and landed**, not merely when another worker says “done.” A research prerequisite can instead use an explicit “decision resolved” condition, including human approval where required.

Then check authorization, capacity, budgets and semantic conflict domains before ranking ready work. These checks follow the ordering in your scheduling refinement. DESIGN

“Different files” is not enough to establish independence. Two workers can touch disjoint files while making incompatible assumptions about an interface, serialization format, migration, build rule or shared resource.

I would therefore reserve **conceptual conflict domains**, not only paths. These are coordination aids; they should not become a requirement to predict every file an agent will touch.

### Delegation without another hierarchy

A worker can use native helpers for investigation or bounded assistance within its admitted envelope. But another writable workspace requires a registered child attempt. Two agents should not independently mutate the same worktree.

Likewise, each native runtime has one continuation owner. Corenius must not issue another turn because it mistakenly interprets an autonomously advancing native session as idle. This ownership distinction is already part of the saved runtime design. corenius-design-v0.1

Plans remain revisable. Changing implementation strategy is ordinary engineering. Weakening acceptance, broadening protected access, or increasing the budget is a different operation.

---

## 5. Worktree design: isolation without shared authority

This is where I would be stricter than a typical local coding setup.

**Do not attach arbitrary agent worktrees directly to the repository that owns protected trunk.**

Git worktrees have separate working directories and indexes, but share common repository metadata and refs. A worktree is therefore a useful editing boundary—not a security boundary. [Git](https://git-scm.com/docs/git-worktree)

Use this arrangement:

```text
Protected integration area
  authority.git/                  gate-owned bare repository

Attempt A
  repo.git/                       private repository metadata
  worktree/                       managed Git worktree
  scratch/

Attempt B
  repo.git/                       separate private repository metadata
  worktree/
  scratch/

Operator
  ordinary checkout              never an agent scratch workspace
```

Each attempt still gets normal Git and worktree ergonomics. It simply cannot mutate protected refs by reaching shared `.git` metadata.

The capsule must also exclude the gate repository, peer workspaces, operator home and privileged credentials. Separate repositories provide little protection when all host paths remain writable.

### Worktree rules I would enforce

| Area | Rule |
|---|---|
| Ownership | One writable attempt per workspace generation |
| Starting point | Record the exact accepted base commit—not just a branch name |
| Naming | `work/<ticket>/<attempt>`, `research/<decision>/<attempt>`, `prototype/<decision>/<attempt>` |
| Lifecycle | Workspace manager owns allocation, registration, sealing, archival and removal |
| Agent freedom | Private commits, experimentation and unsubmitted history cleanup are allowed |
| Protected operations | No trunk mutation, peer-checkout reset, global cleanup or rewriting sealed submissions |
| Restart | Lease expiry does not authorize workspace reuse; reconcile the previous writer first |
| Verification | Use an isolated verification workspace and protected receipt collection |
| Cleanup | Preserve required artifacts, then use ordinary worktree removal; uncertain state is quarantined |
| Retention | Jobs, candidates, evidence and unresolved operations pin their dependencies |

For materialization, use fetch/bundles or an appropriate non-local clone into a private repository. Initially avoid shared alternates and cross-boundary writable/hardlinked object stores. Git’s clone documentation describes both the local-hardlink behavior and the object-lifetime hazards of `--shared`. [Git](https://git-scm.com/docs/git-clone)

### Locks are useful—but limited

Use managed worktree locks and machine-readable inventory:

```text
git worktree add --lock ...
git worktree list --porcelain -z
```

The lock protects against routine removal/pruning. It is not an agent mutex, permission boundary or process-fencing mechanism. Normal worktree removal also provides useful protection against deleting uncommitted work. [Git](https://git-scm.com/docs/git-worktree)

### Build outputs need isolation too

Private source trees with shared mutable build outputs can still corrupt each other’s evidence.

Default to private virtual environments, Rust target directories, Bazel output roots and Torch extension outputs. Share dependency caches only where their concurrency and integrity behavior is understood.

Harp already documents this exact issue for Lean: shared `.lake/build` includes Harp-owned artifacts, so writes must be serialized through receipt extraction or use private outputs for concurrent iteration. Preserve that rule rather than assuming a symlinked cache is harmless. 

**Cleanup is a recorded operation, not `rm -rf` at the end of a prompt.** An old timestamp is not proof of abandonment, and a completed ticket with failed cleanup should remain “landed; cleanup pending.”

---

## 6. Trunk-based development: integrate the result you actually verified

I would choose a **local-first protected trunk**, consistent with the existing no-implicit-push policy.

The gate-owned bare repository is authoritative. The operator checkout is synchronized separately and non-destructively. GitHub is the publication remote, not another independently writable integration authority.

Use the actual configured branch name—`master` for the accessible Harp repository—not a hard-coded assumption that every project uses `main`.

### One serial final-verification/landing lane

Implementation and focused testing can run concurrently. Final preparation, required verification and promotion are serialized per repository.

For each candidate:

1. **Resolve trunk at `B`.** Import the worker’s sealed candidate `C`.
2. **Construct integration candidate `I` on `B`.** Replay, resolve conflicts or squash now—not after verification.
3. **Verify exact `I`.** Run the applicable protected checks and independent review.
4. **Prepare promotion `P`.** Record the candidate, predecessor, evidence and current grant.
5. **Conditionally advance `B → I`.** Check ancestry and expected-old-value; record an attributable receipt.
6. **Persist/reconcile acceptance.** Only then release dependent tickets and schedule cleanup.

The lane is a durable reservation, not a PostgreSQL transaction held open during a long GPU test.

This is the exact-candidate integration pattern already specified in your Corenius RFC. It also explicitly requires a new candidate and applicable evidence when the target changes. corenius-design-v0.1

### Why “test branch, then merge” is insufficient

Suppose A and B start from the same trunk. A lands first. B’s tests prove something about B’s old tree—not necessarily about the combination of A and B.

The integration gate may reuse B’s implementation, but it must construct and verify the resulting tree on current trunk. A clean textual merge is not evidence of semantic compatibility.

### Concrete Git promotion primitive

Inside the protected gate, the ref update can use an expected predecessor and a create-only receipt ref:

```text
start
update refs/heads/<trunk> <I> <B>
create refs/corenius/promotions/<P> <I>
prepare
commit
```

This is input to `git update-ref --stdin`.

Validate object IDs, ref names and ancestry separately: expected-old-value checking does not itself enforce a fast-forward policy. The Git operation also does not make PostgreSQL and Git atomic or update the operator’s checkout. [Git](https://git-scm.com/docs/git-update-ref)

A lost acknowledgment therefore triggers reconciliation of the prepared operation, gate history, receipt ref and actual target—not blind replay. An unresolved promotion blocks conflicting target mutations.

### How large changes stay trunk-based

Default to one coherent integration commit per independently useful ticket, while permitting any private working history.

For broad changes, prefer expand/migrate/contract, compatibility-preserving interfaces and reversible activation. Do not create a permanent “feature integration branch” to avoid designing intermediate green states.

Some changes genuinely need a larger atomic candidate. Allow that explicitly; don’t disguise it as many independent tickets that cannot actually land independently.

Local landing, remote push and release remain separate permissions. A mission can delegate routine local landing to avoid per-ticket human approval, but that is an explicit grant—not an automatic reinterpretation of Harp’s current policy.

---

## 7. Prevent drift by protecting the contract—not the implementation plan

The worker briefing should contain:

> The outcome and why it matters; relevant accepted context; real invariants; observable acceptance; available resources and authority; known uncertainties.

Then allow the worker to choose algorithms, module structure, experiments, implementation order and local tooling.

A useful distinction is:

| Agent may revise autonomously | Requires new authorization |
|---|---|
| Implementation strategy | Success criteria |
| Local decomposition | Hard invariants |
| Choice of experiment | Protected access |
| Internal design within the contract | External effects |
| Next investigation | Budget enlargement |

This is consistent with your saved separation between task revision, plan revision and engineering decisions. corenius-design-v0.1

### Use meaningful review boundaries

I would not require an independent reviewer after every tiny task or a plan refresh every fixed number of minutes.

Re-anchor when an important assumption is disproven, a public contract changes, repeated attempts stop generating useful evidence, a significant candidate is submitted, or the agent claims the mission is complete.

The reviewer receives the **approved original contract and actual artifacts**, not only the implementer’s persuasive summary. It checks specification fidelity, unsupported assumptions, missing behaviors, unnecessary abstractions and evidence limitations.

### Candidate code cannot change its own acceptance rules

The applicable verification policy is pinned outside the candidate’s control.

An agent can propose changes to tests, CI, skills or `AGENTS.md`; those edits do not redefine the current candidate’s acceptance contract. Test execution also must not expose gate credentials or receipt-signing authority.

Your RFC explicitly distinguishes a fresh model conversation from genuine verification independence: immutable subjects, protected policy, separate permissions and trustworthy evidence collection matter first. corenius-design-v0.1 corenius-design-v0.1

For ML/systems work, acceptance must include the relevant numerical tolerances, datasets/seeds, hardware environment, performance conditions or fault-injection scenarios. A missing required GPU run means blocked verification—not an undisclosed skip followed by “done.”

---

## 8. Long-horizon recovery is part of normal execution

A mission should survive a lead replacement without losing useful work or duplicating live jobs.

| Event | Required behavior |
|---|---|
| Lead disappears | Replace the lead and reconstruct its briefing; preserve valid child work |
| Worker lease expires | Reject stale result admission; do not assume its process stopped |
| Launch response is lost | Look up the existing operation; do not blindly launch again |
| Workspace ownership is uncertain | Preserve it; allocate a new generation rather than reuse it |
| Trunk moves | Build a new integration candidate and reverify |
| Promotion acknowledgment is lost | Reconcile before allowing another conflicting mutation |
| Cleanup fails | Keep acceptance and cleanup state separate |

These distinctions are already present in your scheduler refinement, including the warning that leases do not physically fence a process. DESIGN

The restart briefing should be generated from durable facts: current mission revision, accepted trunk, relevant decisions and failed hypotheses, active executions, unresolved operations, remaining budget and the next uncertainty.

**Preserve decisions and evidence—not a mandatory transcript of everything the agent thought.**

Temporal coordinates the waits and retries; PostgreSQL supplies current authorization. Neither a replayed workflow event nor an old conversation becomes fresh permission. Git, PostgreSQL, the runtime and remote execution remain separate transactional domains with explicit reconciliation. corenius-design-v0.1

---

## 9. Build it in vertical slices

I would implement the substrate before adding sophisticated agent behavior:

| Slice | End-to-end result |
|---|---|
| **1. Scripted worker** | Git ticket → admission → private workspace → candidate → verification → conditional landing → receipt → safe cleanup |
| **2. Native runtime** | Replace scripted work with Codex execution and the adapted Matt profile |
| **3. Recovery** | Survive controller/worker interruption without duplicate effects or lost accepted state |
| **4. Parallel implementation** | Two independent workers, dependency release, stale-base handling and conflict reservations |
| **5. Research loop** | Wayfinder decisions, experiments, prototypes and genuine human decision gates |
| **6. Hardening** | Gate fencing, cancellation races, retention, corrupted state, expensive hardware checks |

The first slice is intentionally a complete round trip. A sophisticated planner sitting on unreliable workspace and landing mechanics would only make failures harder to diagnose.

The bundle includes **seven passing Git-mechanics tests** covering conditional promotion, stale-base rejection, receipt conflicts, acknowledgment reconciliation, separation of attempt refs, locked worktree removal and preservation of untracked files. They ran against temporary repositories. They do **not** establish end-to-end Corenius correctness, operating-system isolation, or PostgreSQL/Temporal crash safety.

---

## Bottom line

I would adopt **Git-Markdown authoring + Corenius/PostgreSQL authority + Temporal coordination + adapted Matt skills + private attempt worktrees + exact-candidate trunk promotion**.

The most important departures from an off-the-shelf Matt workflow are small in number but substantial in effect: claims become transactional; orchestration has one owner; worktrees do not share protected Git authority; research artifacts do not imply pushes; and useful slices land incrementally rather than accumulating on a feature-sized integration branch.

**Give the agent freedom over engineering. Make the contract, workspace ownership, evidence and path into trunk precise.**