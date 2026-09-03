# Harp Workstreams Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a first-class, offline `#workstreams` operations dashboard to Harp Atlas with the reviewed schematic visual language, typed reference data, accessible interactions, responsive layouts, and unchanged existing reader routes.

**Architecture:** `AtlasApp` remains the hash-route owner and selects a dedicated dark console shell only for the new Workstreams route. A validated repository reference snapshot and a versioned session-queue adapter provide trusted data to focused presentational components; no component reads private runtime state, executes commands, or fetches data. Dashboard CSS is fully scoped beneath `.workstreams-shell`, and the existing Atlas build continues to produce the offline single-file export.

**Tech Stack:** React 19, TypeScript 5.9, Vite 8, Vitest, Testing Library, scoped CSS, inline SVG, Lucide React 1.40.0, pnpm 11, agent-browser, Rust-backed Atlas content generation.

---

## Execution contract

### Fixed authority and boundaries

- Approved design: `docs/superpowers/specs/2026-09-03-harp-workstreams-dashboard-design.md`.
- Reviewed design commit: `34a5903f4f83aa1dc332ac890094408626def927`.
- The DeepSeek diagram supplies schematic grammar, not source code or redistributed assets.
- Initial data authority is one repository-owned `repository-reference` snapshot. No private XDG state enters the browser bundle.
- Preserve every existing Atlas route and reader behavior.
- The browser never executes commands. The command field appends only a validated, tab-local queued record.
- Do not add fetch, WebSocket, polling, authentication, or a live runtime adapter.
- Do not add external fonts, images, stylesheets, or URL assets to the offline export.
- Do not reduce text below 12px, mobile controls below 16px, desktop targets below 24×24px, or touch targets below 44×44px.
- Keep all Workstreams styles below `.workstreams-shell`; do not retheme reader routes.
- Reuse the shared Atlas dependency cache. Do not reinstall the full dependency tree.
- Do not run Lean during UI iteration. Run it once at the final repository gate
  through the verified primary-checkout cache link. Never hydrate or update
  common Lean dependencies for this feature.
- The reviewed design's `9/14 lemmas` was an illustrative metric, not a tracked
  Harp fact. The implementation uses the checked-in `3/3 routes
  complete-local` program result and keeps the external-review caveat visible.

### Implementation workspace

```sh
git worktree add .worktrees/harp-workstreams-dashboard -b feat/harp-workstreams-dashboard docs/harp-workstreams-dashboard-design
cd .worktrees/harp-workstreams-dashboard
mise trust mise.toml
primary=/Users/bytedance/workspace/harp
base=91e57411939c7a19ac1e694aaaab1f9f3a8b244a
git merge-base --is-ancestor "$base" HEAD
test -d "$primary/formalization/lean/.lake"
cmp formalization/lean/lean-toolchain "$primary/formalization/lean/lean-toolchain"
cmp formalization/lean/lake-manifest.json "$primary/formalization/lean/lake-manifest.json"
if [ ! -L formalization/lean/.lake ]; then
  test ! -e formalization/lean/.lake
  ln -s "$primary/formalization/lean/.lake" formalization/lean/.lake
fi
test "$(readlink formalization/lean/.lake)" = "$primary/formalization/lean/.lake"
shared_modules=/Users/bytedance/.cache/harp/node/atlas-node_modules
test -d "$shared_modules"
if [ ! -L atlas/node_modules ]; then
  test ! -e atlas/node_modules
  ln -s "$shared_modules" atlas/node_modules
fi
test "$(readlink atlas/node_modules)" = "$shared_modules"
scripts/check_atlas_dependencies.sh atlas
```

Expected: a clean feature worktree containing the reviewed design and this
committed plan, with ignored `formalization/lean/.lake` and `atlas/node_modules`
symlinks to the verified machine-local caches.

## File and ownership map

| Path | Responsibility |
|-|-|
| `atlas/package.json`, `atlas/pnpm-lock.yaml` | exact Lucide dependency |
| `atlas/src/app/routes.ts`, `routes.test.ts` | Workstreams route contract |
| `atlas/src/content/workstreams.ts`, `workstreams.test.ts` | types, parser, reference snapshot |
| `atlas/src/app/workstreamsSession.ts`, `workstreamsSession.test.ts` | bounded session queue |
| `atlas/src/app/workstreamsPresentation.ts` | shared exhaustive status labels and Lucide icons |
| `atlas/src/app/WorkstreamsDashboard.tsx` | console shell and composition |
| `atlas/src/app/WorkstreamCard.tsx` | lifecycle schematic |
| `atlas/src/app/AgentVelocityCard.tsx` | chart, exact table, recent runs, command form |
| `atlas/src/app/ResearchInsights.tsx` | per-card tabs |
| `atlas/src/app/WatchlistTable.tsx` | filtered semantic table |
| `atlas/src/app/WorkstreamsDashboard.test.tsx` | integrated behavior and accessibility |
| `atlas/src/app/AtlasApp.tsx`, `ReaderApp.test.tsx` | shell switching and reader regression |
| `atlas/src/styles/workstreams.css` | scoped reviewed visual system |
| `atlas/tests/static-export.test.mjs` | offline export guardrails |
| `README.md`, `atlas/README.md` | discoverability and accurate authority claims |
| `atlas/dist/harp-atlas.html`, `harp-atlas.receipt.json` | regenerated static export |
| `docs/import-receipt.md` | final tracked payload digest |

## Type and state contracts

Use discriminated unions and validate `unknown` at the boundary. No `any`, broad optional-field bags, or unchecked casts.

```ts
export type WorkstreamStatus =
  | "active"
  | "blocked"
  | "planning"
  | "drafting"
  | "complete-local";

export type StageStatus = "complete" | "active" | "queued" | "blocked";
export type InsightArtifactKind = "research" | "spec" | "formalization";
export type EvidenceStatus = "supported" | "missing";
export type DashboardHref =
  | "#crouzeix-conjecture"
  | "#mathematical-foundations"
  | "#knowledge";

export type StageSnapshot = {
  name: string;
  status: StageStatus;
  detail: string;
};

export type WorkstreamSnapshot = {
  id: string;
  title: string;
  status: WorkstreamStatus;
  priority: number;
  lastRun: string;
  progressLabel: string;
  stages: readonly StageSnapshot[];
  blocker: string | null;
  nextAction: string;
  href: DashboardHref;
};

export type RecentRun =
  | { kind: "completed"; id: string; label: string; sha: string; elapsed: string; relativeTime: string }
  | { kind: "failed"; id: string; label: string; sha: string; elapsed: string; relativeTime: string }
  | { kind: "queued"; id: string; command: string; queuedAt: string };
export type QueuedRun = Extract<RecentRun, { kind: "queued" }>;

export type VelocitySample = { label: string; value: number };

export type AgentSnapshot = {
  id: string;
  name: string;
  status: "online" | "offline";
  initials: string;
  velocity: {
    windowLabel: "12h";
    unit: "runs / hour";
    deltaLabel: string;
    summary: string;
    samples: readonly VelocitySample[];
  };
  runs: readonly RecentRun[];
};

export type InsightArtifact = {
  kind: InsightArtifactKind;
  summary: string;
  updatedAt: string;
  href: DashboardHref;
  evidence: EvidenceStatus;
};

export type InsightSnapshot = {
  id: string;
  title: string;
  evaluatorArtifact: InsightArtifactKind;
  artifacts: readonly InsightArtifact[];
};

export type WatchlistRow = {
  id: string;
  workstreamId: string;
  workstream: string;
  status: WorkstreamStatus;
  blocker: string | null;
  nextAction: string;
  href: DashboardHref;
};

export type WorkstreamsSnapshot = {
  schemaVersion: "harp-workstreams/v1";
  source: "repository-reference";
  observedAt: string;
  health: { status: "operational"; label: string; detail: string };
  workstreams: readonly WorkstreamSnapshot[];
  agent: AgentSnapshot;
  insights: readonly InsightSnapshot[];
  watchlist: readonly WatchlistRow[];
};
```

The parser verifies all fields and bounds, exact ISO datetimes for `observedAt`
and queued-run timestamps, `YYYY-MM-DD` artifact dates, unique IDs, exactly
4–6 ordered stages per workstream, exactly one artifact of each kind per
insight, exactly twelve non-negative velocity samples, referential integrity
between watchlist and workstream IDs, and non-contradictory terminal states. A
`blocked` workstream has a non-empty blocker and at least one blocked stage; a
`complete-local` workstream has only complete stages and no blocker. Queue
persistence uses `harp-workstreams-queue/v1`, at most eight entries, and
commands of 1–240 iterated Unicode characters after trim. It accepts only the
three `DashboardHref` hash routes above, so snapshot actions cannot introduce
network navigation.

---

### Task 1: Prepare the exact Atlas dependency boundary

**Files:**
- Modify: `atlas/package.json`
- Modify: `atlas/pnpm-lock.yaml`

- [ ] **Step 1: Verify the shared tools without installing anything**

```sh
test -L atlas/node_modules
test "$(readlink atlas/node_modules)" = /Users/bytedance/.cache/harp/node/atlas-node_modules
scripts/check_atlas_dependencies.sh atlas
(cd atlas && corepack pnpm --version)
```

- [ ] **Step 2: Attempt the one exact offline dependency addition**

```sh
cd atlas
corepack pnpm add --save-exact --offline lucide-react@1.40.0
```

Expected: only `package.json` and `pnpm-lock.yaml` change. If pnpm reports that this exact package is absent from the local store, stop and request approval for only `corepack pnpm add --save-exact lucide-react@1.40.0`. Do not run a broad install or replace Lucide with ad hoc glyphs.

- [ ] **Step 3: Verify and commit the dependency pin**

```sh
node -p "require('./atlas/node_modules/lucide-react/package.json').version"
git diff -- atlas/package.json atlas/pnpm-lock.yaml
git add atlas/package.json atlas/pnpm-lock.yaml
git diff --cached --check
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "build(atlas): pin lucide icons"
```

Expected version: `1.40.0`, with no unrelated resolution changes.

### Task 2: Add the first-class Workstreams route

**Files:**
- Modify: `atlas/src/app/routes.ts`
- Modify: `atlas/src/app/routes.test.ts`

- [ ] **Step 1: Write the failing route test**

```ts
it("parses and formats the workstreams console route", () => {
  expect(parseRoute("#workstreams")).toEqual({ kind: "workstreams" });
  expect(formatRoute({ kind: "workstreams" })).toBe("#workstreams");
});
```

- [ ] **Step 2: Run RED**

```sh
cd atlas
node_modules/.bin/vitest run src/app/routes.test.ts --configLoader runner --no-cache
```

Expected: assertion or compile failure because the variant is absent.

- [ ] **Step 3: Add the variant and exhaustive branches**

```ts
export type AtlasRoute =
  | { kind: "weng"; sectionId: WengSectionId }
  | { kind: "systems" }
  | { kind: "system"; systemId: SystemId; returnTo: WengSectionId | null }
  | { kind: "document"; documentId: DocumentId; sectionId: string | null }
  | { kind: "chapter"; conceptId: ConceptId }
  | { kind: "legacy"; routeId: ReaderRouteId }
  | { kind: "lesson"; lessonId: string }
  | { kind: "diagnose"; caseId: string | null }
  | { kind: "sources" }
  | { kind: "workstreams" };

if (family === "workstreams" && encodedId === "") {
  return { kind: "workstreams" };
}

case "workstreams":
  return "#workstreams";
```

Do not change the default Weng route.

- [ ] **Step 4: Run GREEN and commit**

```sh
cd atlas
node_modules/.bin/vitest run src/app/routes.test.ts --configLoader runner --no-cache
node_modules/.bin/tsc -b
cd ..
git add atlas/src/app/routes.ts atlas/src/app/routes.test.ts
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "feat(atlas): add workstreams route"
```

### Task 3: Define and validate the reference snapshot

**Files:**
- Create: `atlas/src/content/workstreams.ts`
- Create: `atlas/src/content/workstreams.test.ts`

- [ ] **Step 1: Write boundary-validation RED tests**

Test a valid snapshot plus duplicate workstream IDs, empty stages,
blocked-without-blocker, complete-with-active-stage, missing next action, wrong
sample counts, duplicate insight IDs, and malformed timestamps. Assert the
reference contains exactly four required workstreams, three insights, four
watchlist rows, and twelve velocity samples. Build each invalid input by copying
`workstreamsReference` and replacing only the relevant nested value; do not use
an untyped fixture builder. For example:

```ts
it("rejects duplicate workstream ids", () => {
  const [first, second, ...rest] = workstreamsReference.workstreams;
  const input = {
    ...workstreamsReference,
    workstreams: [first, { ...second, id: first.id }, ...rest],
  };
  expect(() => parseWorkstreamsSnapshot(input)).toThrow(/unique/i);
});

it("rejects a blocked workstream without blocker evidence", () => {
  const [first, second, ...rest] = workstreamsReference.workstreams;
  const input = {
    ...workstreamsReference,
    workstreams: [first, { ...second, status: "blocked", blocker: null }, ...rest],
  };
  expect(() => parseWorkstreamsSnapshot(input)).toThrow(/blocker/i);
});
```

Use the same explicit copy pattern for the remaining cases. The valid-snapshot
test asserts the four titles in order, `source === "repository-reference"`,
`agent.velocity.samples.length === 12`, and the 3/4 insight/watchlist counts.

- [ ] **Step 2: Run RED**

```sh
cd atlas
node_modules/.bin/vitest run src/content/workstreams.test.ts --configLoader runner --no-cache
```

- [ ] **Step 3: Implement validation helpers and types**

```ts
function record(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as Record<string, unknown>;
}

function nonEmptyString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${label} must be a non-empty string`);
  }
  return value;
}
```

Use discriminant switches and `satisfies`; any cast in the boundary parser must
follow complete field validation. IDs match
`/^[a-z0-9]+(?:-[a-z0-9]+)*$/`. Reject an ISO timestamp unless parsing it
produces a finite time and `new Date(value).toISOString() === value`. Export
only the types, `parseWorkstreamsSnapshot`, and `workstreamsReference`; keep the
raw input private.

- [ ] **Step 4: Add and parse the fixed reference snapshot**

```ts
const referenceInput = {
  schemaVersion: "harp-workstreams/v1",
  source: "repository-reference",
  observedAt: "2026-09-03T16:00:00.000Z",
  health: {
    status: "operational",
    label: "Operational",
    detail: "Reference schema validated; no live service is connected.",
  },
  workstreams: [
    {
      id: "crouzeix-conjecture",
      title: "Crouzeix Conjecture",
      status: "active",
      priority: 2,
      lastRun: "program gate recorded 2026-08-27",
      progressLabel: "3/3 routes complete-local",
      stages: [
        { name: "Research", status: "complete", detail: "Jin and Lorist–Schwenninger sources mapped" },
        { name: "Spec", status: "complete", detail: "Three route contracts frozen" },
        { name: "Formalize", status: "complete", detail: "Jin, LS, and Harp routes compiled" },
        { name: "Verify", status: "complete", detail: "Program verification recorded" },
        { name: "Publish", status: "active", detail: "Prepare public presentation and external review" },
      ],
      blocker: null,
      nextAction: "Package the verified local routes for external mathematical review.",
      href: "#crouzeix-conjecture",
    },
    {
      id: "autodiff-geometry",
      title: "Autodiff Geometry",
      status: "blocked",
      priority: 1,
      lastRun: "foundation snapshot recorded 2026-08-18",
      progressLabel: "5/7 planned",
      stages: [
        { name: "Acquire", status: "complete", detail: "Core source registry captured" },
        { name: "Parse", status: "complete", detail: "Normalize source structure and notation" },
        { name: "Curriculum", status: "blocked", detail: "Source-to-lesson dependency graph remains incomplete" },
        { name: "Formalize", status: "queued", detail: "Extend finite-coordinate foundations" },
        { name: "Verify", status: "queued", detail: "Run focused Lean route only after formalization" },
      ],
      blocker: "The source-to-curriculum map is incomplete beyond finite-coordinate identities.",
      nextAction: "Finish the JAX, Spivak, SICM, and FDG concept dependency map.",
      href: "#mathematical-foundations",
    },
    {
      id: "agentic-research",
      title: "Agentic Research",
      status: "drafting",
      priority: 3,
      lastRun: "reference packet recorded 2026-09-03",
      progressLabel: "Drafting",
      stages: [
        { name: "Research", status: "complete", detail: "Public evidence graph captured" },
        { name: "Synthesize", status: "complete", detail: "RSI and harness model assembled" },
        { name: "Design", status: "active", detail: "Production-control interfaces under review" },
        { name: "Prototype", status: "queued", detail: "Bounded implementation slices remain" },
        { name: "Evaluate", status: "queued", detail: "Matched-cost evaluation remains" },
      ],
      blocker: null,
      nextAction: "Turn the evidence-backed operating model into one bounded prototype.",
      href: "#knowledge",
    },
    {
      id: "nng4-foundations",
      title: "NNG4 / Foundations",
      status: "complete-local",
      priority: 4,
      lastRun: "focused verifier recorded 2026-08-18",
      progressLabel: "All levels proved",
      stages: [
        { name: "Natural", status: "complete", detail: "Natural-number levels complete" },
        { name: "Addition", status: "complete", detail: "Addition levels complete" },
        { name: "Multiplication", status: "complete", detail: "Multiplication levels complete" },
        { name: "Power", status: "complete", detail: "Power levels complete" },
        { name: "Advanced", status: "complete", detail: "Introductory game closure complete" },
      ],
      blocker: null,
      nextAction: "Keep the route as the fast natural-number regression surface.",
      href: "#mathematical-foundations",
    },
  ],
  agent: {
    id: "harp-agent",
    name: "Harp agent",
    status: "online",
    initials: "HA",
    velocity: {
      windowLabel: "12h",
      unit: "runs / hour",
      deltaLabel: "+18%",
      summary: "Reference throughput rises from 2 to 10 runs per hour.",
      samples: [
        { label: "05:00", value: 2 }, { label: "06:00", value: 3 },
        { label: "07:00", value: 3 }, { label: "08:00", value: 5 },
        { label: "09:00", value: 4 }, { label: "10:00", value: 6 },
        { label: "11:00", value: 7 }, { label: "12:00", value: 6 },
        { label: "13:00", value: 8 }, { label: "14:00", value: 8 },
        { label: "15:00", value: 9 }, { label: "16:00", value: 10 },
      ],
    },
    runs: [
      { kind: "completed", id: "jin-route", label: "Jin route", sha: "e08152d", elapsed: "25s", relativeTime: "program gate 2026-08-27" },
      { kind: "completed", id: "ls-route", label: "Lorist–Schwenninger route", sha: "e08152d", elapsed: "10s", relativeTime: "program gate 2026-08-27" },
      { kind: "completed", id: "harp-route", label: "Harp route", sha: "e08152d", elapsed: "16s", relativeTime: "program gate 2026-08-27" },
    ],
  },
  insights: [
    {
      id: "crouzeix-route-closure",
      title: "Crouzeix route closure",
      evaluatorArtifact: "formalization",
      artifacts: [
        { kind: "research", summary: "Two source-faithful routes and one explicitly derived route are documented.", updatedAt: "2026-08-27", href: "#crouzeix-conjecture", evidence: "supported" },
        { kind: "spec", summary: "The three-route completion contract and claim ceiling are frozen.", updatedAt: "2026-08-27", href: "#crouzeix-conjecture", evidence: "supported" },
        { kind: "formalization", summary: "All three route validators report complete-local; external mathematical review is not claimed.", updatedAt: "2026-08-27", href: "#crouzeix-conjecture", evidence: "supported" },
      ],
    },
    {
      id: "autodiff-curriculum",
      title: "Autodiff curriculum",
      evaluatorArtifact: "spec",
      artifacts: [
        { kind: "research", summary: "JAX, Spivak, SICM, and FDG are registered as the source family.", updatedAt: "2026-08-18", href: "#mathematical-foundations", evidence: "supported" },
        { kind: "spec", summary: "A complete source-to-curriculum dependency graph is still required.", updatedAt: "2026-08-18", href: "#mathematical-foundations", evidence: "missing" },
        { kind: "formalization", summary: "Finite-coordinate gradient, JVP, VJP, HVP, and duality identities are available.", updatedAt: "2026-08-18", href: "#mathematical-foundations", evidence: "supported" },
      ],
    },
    {
      id: "agentic-production",
      title: "Agentic path to production",
      evaluatorArtifact: "research",
      artifacts: [
        { kind: "research", summary: "The RSI, harness, verifier, and agentic-engineering evidence graph is assembled.", updatedAt: "2026-09-03", href: "#knowledge", evidence: "supported" },
        { kind: "spec", summary: "Human authority, bounded execution, independent verification, and observability are separated.", updatedAt: "2026-09-03", href: "#knowledge", evidence: "supported" },
        { kind: "formalization", summary: "No theorem connects the engineering loop to recursive self-improvement guarantees.", updatedAt: "2026-09-03", href: "#knowledge", evidence: "missing" },
      ],
    },
  ],
  watchlist: [
    { id: "watch-crouzeix", workstreamId: "crouzeix-conjecture", workstream: "Crouzeix Conjecture", status: "active", blocker: null, nextAction: "Prepare external mathematical review packet", href: "#crouzeix-conjecture" },
    { id: "watch-autodiff", workstreamId: "autodiff-geometry", workstream: "Autodiff Geometry", status: "blocked", blocker: "Curriculum dependency map incomplete", nextAction: "Map JAX to geometry sources", href: "#mathematical-foundations" },
    { id: "watch-agentic", workstreamId: "agentic-research", workstream: "Agentic Research", status: "drafting", blocker: null, nextAction: "Select a bounded prototype", href: "#knowledge" },
    { id: "watch-nng4", workstreamId: "nng4-foundations", workstream: "NNG4 / Foundations", status: "complete-local", blocker: null, nextAction: "Retain as regression surface", href: "#mathematical-foundations" },
  ],
} satisfies WorkstreamsSnapshot;

export const workstreamsReference = parseWorkstreamsSnapshot(referenceInput);
```

The route labels, reviewed SHA, durations, and date above come from the
checked-in Crouzeix program-verification record. The velocity series and its
delta are presentation fixtures, not measured telemetry; retain the word
`Reference` in their summary. Keep all proof language at the repository's
`complete-local` ceiling.

- [ ] **Step 5: Run GREEN and commit**

```sh
cd atlas
node_modules/.bin/vitest run src/content/workstreams.test.ts --configLoader runner --no-cache
node_modules/.bin/tsc -b
cd ..
git add atlas/src/content/workstreams.ts atlas/src/content/workstreams.test.ts
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "feat(atlas): define workstreams snapshot"
```

### Task 4: Add the bounded session queue adapter

**Files:**
- Create: `atlas/src/app/workstreamsSession.ts`
- Create: `atlas/src/app/workstreamsSession.test.ts`

- [ ] **Step 1: Write queue RED tests**

Test empty input, over-240 input, malformed JSON, unknown schema, invalid entry shape, max-eight truncation, round trip, and storage exceptions with an injected `Storage` fake.

```ts
class MemoryStorage implements Storage {
  #values = new Map<string, string>();
  get length(): number { return this.#values.size; }
  clear(): void { this.#values.clear(); }
  getItem(key: string): string | null { return this.#values.get(key) ?? null; }
  key(index: number): string | null { return [...this.#values.keys()][index] ?? null; }
  removeItem(key: string): void { this.#values.delete(key); }
  setItem(key: string, value: string): void { this.#values.set(key, value); }
}

it("persists a bounded versioned queue", () => {
  const storage = new MemoryStorage();
  const first = queueLocalCommand({
    storage, command: "verify Jin route", now: "2026-09-03T16:12:00.000Z",
  });
  expect(first[0]).toMatchObject({ kind: "queued", command: "verify Jin route" });
  expect(readLocalQueue(storage)).toEqual(first);
});
```

- [ ] **Step 2: Run RED**

```sh
cd atlas
node_modules/.bin/vitest run src/app/workstreamsSession.test.ts --configLoader runner --no-cache
```

- [ ] **Step 3: Implement the adapter**

```ts
const QUEUE_KEY = "harp.workstreams.queue.v1";
const MAX_COMMANDS = 8;
const MAX_COMMAND_LENGTH = 240;

export function queueLocalCommand({
  storage, command, now, createId = () => crypto.randomUUID(),
}: {
  storage: Storage;
  command: string;
  now: string;
  createId?: () => string;
}): readonly QueuedRun[] {
  const normalized = command.trim();
  if (normalized.length === 0 || [...normalized].length > MAX_COMMAND_LENGTH) {
    throw new Error("Command must contain 1–240 characters");
  }
  const run = {
    kind: "queued",
    id: `local-${createId()}`,
    command: normalized,
    queuedAt: now,
  } satisfies QueuedRun;
  const runs = [run, ...readLocalQueue(storage)].slice(0, MAX_COMMANDS);
  storage.setItem(QUEUE_KEY, JSON.stringify({
    schemaVersion: "harp-workstreams-queue/v1", runs,
  }));
  return runs;
}
```

Implement `readLocalQueue(storage): readonly QueuedRun[]` by parsing `unknown`,
requiring exactly `schemaVersion: "harp-workstreams-queue/v1"`, validating
every queued run and ISO `queuedAt`, and returning at most eight entries. It
catches `getItem`, JSON, and validation failures and returns `[]`;
`queueLocalCommand` deliberately lets `setItem` failures reach the UI. In tests,
inject `createId: () => "run-1"` so assertions are deterministic.

- [ ] **Step 4: Run GREEN and commit**

```sh
cd atlas
node_modules/.bin/vitest run src/app/workstreamsSession.test.ts --configLoader runner --no-cache
cd ..
git add atlas/src/app/workstreamsSession.ts atlas/src/app/workstreamsSession.test.ts
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "feat(atlas): persist local workstream queue"
```

### Task 5: Build the console shell and labeled workstream schematics

**Files:**
- Create: `atlas/src/app/workstreamsPresentation.ts`
- Create: `atlas/src/app/WorkstreamCard.tsx`
- Create: `atlas/src/app/WorkstreamsDashboard.tsx`
- Create: `atlas/src/app/WorkstreamsDashboard.test.tsx`

- [ ] **Step 1: Write the shell and card RED test**

```tsx
it("renders the reference workstreams as labeled lifecycle schematics", () => {
  render(
    <WorkstreamsDashboard snapshot={workstreamsReference} onNavigate={vi.fn()} />,
  );

  expect(screen.getByRole("heading", { name: "Harp Workstreams" })).toBeInTheDocument();
  expect(screen.getByText("REFERENCE SNAPSHOT")).toBeInTheDocument();
  for (const title of [
    "Crouzeix Conjecture",
    "Autodiff Geometry",
    "Agentic Research",
    "NNG4 / Foundations",
  ]) {
    expect(screen.getByRole("article", { name: title })).toBeInTheDocument();
  }
  expect(screen.getAllByRole("list", { name: /lifecycle/i })).toHaveLength(4);
  expect(screen.getByText("3/3 routes complete-local")).toBeInTheDocument();
});
```

Also assert the highest-priority blocked workstream drives the `FOCUS NOW` strip, stage names are visible without tooltips, and every node exposes state text to assistive technology.

- [ ] **Step 2: Run RED**

```sh
cd atlas
node_modules/.bin/vitest run src/app/WorkstreamsDashboard.test.tsx --configLoader runner --no-cache
```

- [ ] **Step 3: Implement shared status presentation and `WorkstreamCard`**

```tsx
// workstreamsPresentation.ts
export const stagePresentation: Record<
  StageStatus,
  { label: string; Icon: LucideIcon }
> = {
  complete: { label: "Complete", Icon: Check },
  active: { label: "Active", Icon: CircleDot },
  queued: { label: "Queued", Icon: Circle },
  blocked: { label: "Blocked", Icon: OctagonX },
};

export const workstreamPresentation: Record<
  WorkstreamStatus,
  { label: string; Icon: LucideIcon }
> = {
  active: { label: "Active", Icon: Activity },
  blocked: { label: "Blocked", Icon: OctagonX },
  planning: { label: "Planning", Icon: ListTodo },
  drafting: { label: "Drafting", Icon: Pencil },
  "complete-local": { label: "Complete locally", Icon: BadgeCheck },
};

// WorkstreamCard.tsx
export function WorkstreamCard({ workstream }: { workstream: WorkstreamSnapshot }) {
  const status = workstreamPresentation[workstream.status];
  return (
    <article className="workstream-card ops-field" aria-labelledby={`${workstream.id}-title`}>
      <header className="workstream-card__header">
        <h3 id={`${workstream.id}-title`}>{workstream.title}</h3>
        <span className="status-chip" data-state={workstream.status}>
          <status.Icon aria-hidden="true" size={14} /> {status.label}
        </span>
        <span className="workstream-last-run">{workstream.lastRun}</span>
      </header>
      <ol className="lifecycle-rail" aria-label={`${workstream.title} lifecycle`}>
        {workstream.stages.map((stage, index) => {
          const presentation = stagePresentation[stage.status];
          return (
            <li data-state={stage.status} key={stage.name}>
              <button
                className="lifecycle-node"
                type="button"
                aria-label={
                  `${stage.name}: ${presentation.label}. ${stage.detail}`
                }
                aria-describedby={`${workstream.id}-stage-${index}-detail`}
              >
                <presentation.Icon aria-hidden="true" size={14} />
              </button>
              <span className="lifecycle-label">{stage.name}</span>
              <span
                className="lifecycle-tooltip"
                id={`${workstream.id}-stage-${index}-detail`}
                role="tooltip"
              >
                {presentation.label}. {stage.detail}
              </span>
            </li>
          );
        })}
      </ol>
      <p className="workstream-evidence">
        {workstream.progressLabel} · {workstream.lastRun}
      </p>
      <details className="lifecycle-details">
        <summary>Lifecycle details</summary>
        <ul>
          {workstream.stages.map((stage) => (
            <li key={stage.name}>
              {stage.name}: {stagePresentation[stage.status].label}. {stage.detail}
            </li>
          ))}
        </ul>
      </details>
    </article>
  );
}
```

The node buttons reveal `.lifecycle-tooltip` on hover and focus but do not
change state. The `details` disclosure supplies the same secondary content in a
persistent form. Neither surface owns the stage name or state.

- [ ] **Step 4: Implement the shell and focus strip**

Use this initial public contract:

```ts
export type WorkstreamsDashboardProps = {
  snapshot: WorkstreamsSnapshot;
  onNavigate: (href: DashboardHref) => void;
};
```

`WorkstreamsDashboard` computes focus with:

```ts
const focus = snapshot.workstreams
  .filter((workstream) => workstream.status === "blocked")
  .sort((left, right) => left.priority - right.priority)[0] ?? null;
```

Render `<aside aria-label="Workstreams navigation">`, a short header with
search disabled until Task 7, the exact `REFERENCE SNAPSHOT` chip and
`snapshot.observedAt`, a `<section aria-labelledby="active-workstreams">`
containing all cards, and the focus strip only when `focus !== null`. Its action
button calls `onNavigate(focus.href)`. Render `<section id="agent-velocity"
aria-labelledby="agent-velocity-title">` and `<section id="insights"
aria-labelledby="insights-title">` as named empty sections that Tasks 6 and 7
replace. Use a `recentlyActiveRef` on the sidebar list; its three entries are
the first three workstreams by ascending priority. The Workstreams control
scrolls to and focuses the `Harp Workstreams` heading. `Runs` focuses
`#agent-velocity`; `Agents` focuses the Recently Active list; `Knowledge` calls
`onNavigate("#knowledge")`. Settings is introduced in Task 7.
Render the agent status as `Online at snapshot`, never as an unqualified live
state.

Import named Lucide components only. Use `LayoutDashboard`, `History`,
`Bot`, `ShieldCheck`, `Library`, and `Settings` for the six sidebar
entries; `Activity` for model health; `ArrowRight` for action controls; and
`Activity`, `OctagonX`, `ListTodo`, `Pencil`, and `BadgeCheck` in the
shared status map. Each icon is `aria-hidden="true"` because adjacent visible
text or the containing button owns the accessible name. Do not render emoji or
text-glyph icon substitutes.

- [ ] **Step 5: Run GREEN and commit**

```sh
cd atlas
node_modules/.bin/vitest run src/app/WorkstreamsDashboard.test.tsx --configLoader runner --no-cache
node_modules/.bin/tsc -b
cd ..
git add atlas/src/app/workstreamsPresentation.ts atlas/src/app/WorkstreamCard.tsx atlas/src/app/WorkstreamsDashboard.tsx atlas/src/app/WorkstreamsDashboard.test.tsx
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "feat(atlas): render workstream schematics"
```

### Task 6: Add Agent Velocity, chart equivalence, and command interaction

**Files:**
- Create: `atlas/src/app/AgentVelocityCard.tsx`
- Modify: `atlas/src/app/WorkstreamsDashboard.tsx`
- Modify: `atlas/src/app/WorkstreamsDashboard.test.tsx`

- [ ] **Step 1: Write failing interaction tests**

Cover visible unit/min/max/trend text, the sample disclosure table, focus-equivalent point detail, Recent Runs focusing, empty command error, Shift+Enter, send-button submission, queue insertion, and persistence across unmount/remount.

```tsx
it("queues a local command without executing it and restores it after remount", async () => {
  const user = userEvent.setup();
  const first = render(
    <WorkstreamsDashboard snapshot={workstreamsReference} onNavigate={vi.fn()} />,
  );
  await user.type(
    screen.getByLabelText("Local command"),
    "verify Jin route{shift>}{enter}{/shift}",
  );
  expect(screen.getByRole("status")).toHaveTextContent("Queued locally");
  expect(screen.getByText("verify Jin route")).toBeInTheDocument();
  first.unmount();
  render(
    <WorkstreamsDashboard snapshot={workstreamsReference} onNavigate={vi.fn()} />,
  );
  expect(screen.getByText("verify Jin route")).toBeInTheDocument();
});
```

The component receives no network, process, or shell capability; tests should not mock one.

- [ ] **Step 2: Run RED**

```sh
cd atlas
node_modules/.bin/vitest run src/app/WorkstreamsDashboard.test.tsx --configLoader runner --no-cache
```

- [ ] **Step 3: Implement chart geometry and accessible fallback**

```ts
export function velocityPoints(
  samples: readonly VelocitySample[],
  width: number,
  height: number,
): string {
  const values = samples.map((sample) => sample.value);
  const minimum = Math.min(...values);
  const maximum = Math.max(...values);
  const span = Math.max(1, maximum - minimum);
  return samples.map((sample, index) => {
    const x = samples.length === 1 ? 0 : (index / (samples.length - 1)) * width;
    const y = height - ((sample.value - minimum) / span) * height;
    return `${x},${y}`;
  }).join(" ");
}
```

Render a slate polyline, cyan current-point circle, visible `runs / hour`,
min/max, `12h`, `+18%`, the stored trend sentence, and a `<details>` table with
twelve rows. Give the SVG `role="img"`, a `<title>Agent velocity over 12
hours</title>`, and `<desc>{velocity.summary}</desc>`. Render one SVG `<circle
tabIndex={0}>` per sample with `aria-label={`${sample.label}: ${sample.value}
${velocity.unit}`}`; the last point alone uses cyan. The sample table owns the
exact values and is the non-graphical equivalent.

Use this component boundary so navigation focus and queue state remain
testable:

```ts
export type AgentVelocityCardProps = {
  agent: AgentSnapshot;
  recentRunsHeadingRef: RefObject<HTMLHeadingElement | null>;
};
```

Task 6 replaces Task 5's empty `#agent-velocity` section with
`AgentVelocityCard`. `WorkstreamsDashboard` owns `recentRunsHeadingRef`; its
Runs control focuses that heading. `AgentVelocityCard` owns only command text,
queued runs, and the live-region message. Its rendered run list is
`[...queuedRuns, ...agent.runs]`, so the bounded local queue appears above the
three repository-reference runs without mutating the snapshot.

- [ ] **Step 4: Implement local command flow**

Initialize queue state lazily from `readLocalQueue(window.sessionStorage)`.
Submission calls `queueLocalCommand`, catches validation/storage errors, updates
one atomic `role="status" aria-live="polite"` message, clears the input only
on success, and returns focus to the input. Use a normal `<form>` for button and
Enter semantics; in the input's `onKeyDown`, call `preventDefault()` and the
same submit function only when `event.key === "Enter" && event.shiftKey`. A
normal unmodified Enter continues through form submission. `View Recent Runs`
focuses a `tabIndex={-1}` list heading and announces the current count through
the same live region. In test `beforeEach`, call `window.sessionStorage.clear()`
so persistence tests cannot contaminate one another.

- [ ] **Step 5: Run GREEN and commit**

```sh
cd atlas
node_modules/.bin/vitest run src/app/WorkstreamsDashboard.test.tsx --configLoader runner --no-cache
node_modules/.bin/tsc -b
cd ..
git add atlas/src/app/AgentVelocityCard.tsx atlas/src/app/WorkstreamsDashboard.tsx atlas/src/app/WorkstreamsDashboard.test.tsx
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "feat(atlas): add agent velocity controls"
```

### Task 7: Add insights, watchlist, search, and navigation behavior

**Files:**
- Create: `atlas/src/app/ResearchInsights.tsx`
- Create: `atlas/src/app/WatchlistTable.tsx`
- Modify: `atlas/src/app/WorkstreamsDashboard.tsx`
- Modify: `atlas/src/app/WorkstreamsDashboard.test.tsx`

- [ ] **Step 1: Write the interaction RED tests**

Implement the listed cases as separate tests. These two establish the keyboard
and filter contracts; use the same explicit Testing Library style for the
remaining cases:

```tsx
it("keeps insight tab selection local to one card", async () => {
  const user = userEvent.setup();
  render(
    <WorkstreamsDashboard snapshot={workstreamsReference} onNavigate={vi.fn()} />,
  );
  const firstCard = screen.getByRole("article", { name: "Crouzeix route closure" });
  const firstResearch = within(firstCard).getByRole("tab", { name: "Research" });
  firstResearch.focus();
  await user.keyboard("{ArrowRight}");
  expect(within(firstCard).getByRole("tab", { name: "Spec" })).toHaveAttribute(
    "aria-selected", "true",
  );
  const secondCard = screen.getByRole("article", { name: "Autodiff curriculum" });
  expect(within(secondCard).getByRole("tab", { name: "Research" })).toHaveAttribute(
    "aria-selected", "true",
  );
});

it("filters cards and watchlist rows with one query", async () => {
  const user = userEvent.setup();
  render(
    <WorkstreamsDashboard snapshot={workstreamsReference} onNavigate={vi.fn()} />,
  );
  await user.type(screen.getByRole("searchbox", { name: "Search workstreams" }), "crouzeix");
  expect(screen.getByRole("article", { name: "Crouzeix Conjecture" })).toBeInTheDocument();
  expect(screen.queryByRole("article", { name: "Autodiff Geometry" })).not.toBeInTheDocument();
  expect(screen.getByRole("row", { name: /Crouzeix Conjecture/ })).toBeInTheDocument();
  expect(screen.queryByRole("row", { name: /Autodiff Geometry/ })).not.toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Clear search" }));
  expect(screen.getAllByRole("article", { name: /Conjecture|Geometry|Research|Foundations/ })).toHaveLength(4);
});
```

Test all of the following:

- three insight cards each expose Research, Spec, and Formalization tabs;
- ArrowLeft/ArrowRight moves focus and selection within one card only;
- Home/End selects the first/last tab;
- Evaluators selects the evaluator-facing artifact;
- `MISSING` remains literal text with its dedicated class;
- search filters both cards and table, clear restores all four, and empty results retain the table header;
- Meta+K and Control+K focus search;
- Knowledge calls `onNavigate("#knowledge")`; Task 8 proves the hash change;
- Runs and Agents focus in-page targets without changing hash.
- Workstreams focuses the page heading, and Settings expands and closes its
  read-only reference-mode panel with focus restored to the trigger.

- [ ] **Step 2: Run RED**

```sh
cd atlas
node_modules/.bin/vitest run src/app/WorkstreamsDashboard.test.tsx --configLoader runner --no-cache
```

- [ ] **Step 3: Implement roving tabs per insight card**

`WorkstreamsDashboard` owns one keyed selection per insight, initialized to
`"research"`, so cards remain independent while the Evaluators sidebar action
can select each card's declared evaluator artifact:

```ts
function selectionsFor(
  insights: readonly InsightSnapshot[],
  choose: (insight: InsightSnapshot) => InsightArtifactKind,
): Record<string, InsightArtifactKind> {
  return insights.reduce<Record<string, InsightArtifactKind>>((result, insight) => {
    result[insight.id] = choose(insight);
    return result;
  }, {});
}

const [selectedArtifacts, setSelectedArtifacts] = useState(() =>
  selectionsFor(snapshot.insights, () => "research"),
);

function selectEvaluators(): void {
  setSelectedArtifacts(selectionsFor(
    snapshot.insights,
    (insight) => insight.evaluatorArtifact,
  ));
  insightsHeadingRef.current?.focus();
}
```

Each `InsightCard` receives its selected kind plus an `onSelect(kind)` callback
and owns a three-element `buttonRefs` array for focus. Use this exact ordering
and behavior:

```tsx
const artifactKinds = ["research", "spec", "formalization"] as const;

function nextTabIndex(
  current: number,
  key: string,
): number | null {
  if (key === "ArrowRight") return (current + 1) % artifactKinds.length;
  if (key === "ArrowLeft") return (current - 1 + artifactKinds.length) % artifactKinds.length;
  if (key === "Home") return 0;
  if (key === "End") return artifactKinds.length - 1;
  return null;
}
```

Wrap each card's three buttons in `role="tablist"` with
`aria-label={`${insight.title} artifacts`}`. For each button, set `role="tab"`, `id={`${insight.id}-${kind}-tab`}`,
`aria-controls={`${insight.id}-${kind}-panel`}`, `aria-selected`, and `tabIndex`
of zero only for the selected tab. On a handled key, prevent default, update the
card-local selection, and focus that card's matching ref. Render exactly one
`role="tabpanel"` with the reverse `aria-labelledby` reference, artifact
summary, date, action button calling `onNavigate(artifact.href)`, and literal
`MISSING` when `artifact.evidence === "missing"`.

- [ ] **Step 4: Implement one normalized search predicate**

```ts
function searchTerms(workstream: WorkstreamSnapshot): string {
  return [
    workstream.title,
    workstream.status,
    workstream.progressLabel,
    workstream.blocker ?? "",
    workstream.nextAction,
    ...workstream.stages.flatMap((stage) => [stage.name, stage.status]),
  ].join(" ").toLocaleLowerCase();
}

const normalizedQuery = query.trim().toLocaleLowerCase();
const visibleWorkstreams = snapshot.workstreams.filter(
  (workstream) => normalizedQuery === "" || searchTerms(workstream).includes(normalizedQuery),
);
const visibleIds = new Set(visibleWorkstreams.map((workstream) => workstream.id));
const visibleWatchlist = snapshot.watchlist.filter(
  (row) => visibleIds.has(row.workstreamId),
);
```

Normalize the query with `trim().toLocaleLowerCase()`.
Derive `visibleWorkstreams` and `visibleWatchlist` with `useMemo`; do not
persist search state. Register one `keydown` listener on mount that focuses the
search input for `(event.metaKey || event.ctrlKey) && event.key.toLowerCase()
=== "k"`; call `preventDefault()` and remove the listener on unmount. The clear
button empties the query and returns focus to the input.

- [ ] **Step 5: Implement `WatchlistTable`**

Render this fixed table structure inside
`<div className="watchlist-scroll" role="region"
aria-label="Workstream watchlist, scroll for more columns" tabIndex={0}>`:

```tsx
<table>
  <thead>
    <tr><th>Workstream</th><th>Status</th><th>Blocker</th><th>Next action</th></tr>
  </thead>
  <tbody>
    {rows.length === 0 ? (
      <tr><td colSpan={4}>No workstreams match this search.</td></tr>
    ) : rows.map((row) => (
      <tr key={row.id}>
        <th scope="row">{row.workstream}</th>
        <td><span className="status-chip" data-state={row.status}>{workstreamPresentation[row.status].label}</span></td>
        <td>{row.blocker ?? "None recorded"}</td>
        <td><button type="button" onClick={() => onNavigate(row.href)}>{row.nextAction}</button></td>
      </tr>
    ))}
  </tbody>
</table>
```

Add `<p className="watchlist-overflow-cue">Scroll for more columns</p>`
immediately before the region; CSS hides it above 899px. Wire Evaluators to
`selectEvaluators()`, which sets each card to its declared
`evaluatorArtifact`, then focuses the Insights heading. Settings toggles a
read-only `<section id="reference-settings" aria-label="Reference mode
settings">` through a button with `aria-expanded` and `aria-controls`. The
panel states that no live data is connected, lists Cmd/Ctrl+K and Shift+Enter,
and has a Close button that hides the panel and returns focus to Settings.

- [ ] **Step 6: Run GREEN and commit**

```sh
cd atlas
node_modules/.bin/vitest run src/app/WorkstreamsDashboard.test.tsx --configLoader runner --no-cache
node_modules/.bin/tsc -b
cd ..
git add atlas/src/app/ResearchInsights.tsx atlas/src/app/WatchlistTable.tsx atlas/src/app/WorkstreamsDashboard.tsx atlas/src/app/WorkstreamsDashboard.test.tsx
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "feat(atlas): add workstream insights and search"
```

### Task 8: Integrate the dedicated console shell into Atlas

**Files:**
- Modify: `atlas/src/app/AtlasApp.tsx`
- Modify: `atlas/src/app/ReaderApp.test.tsx`
- Modify: `atlas/src/app/WorkstreamsDashboard.test.tsx`

- [ ] **Step 1: Write shell-switch RED tests**

```tsx
it("opens the workstreams console without the reader shell", async () => {
  const user = userEvent.setup();
  window.location.hash = "#knowledge";
  render(<AtlasApp />);
  await user.click(screen.getByRole("button", { name: "Open Console / Workstreams" }));
  expect(window.location.hash).toBe("#workstreams");
  expect(screen.getByRole("heading", { name: "Harp Workstreams" })).toBeInTheDocument();
  expect(screen.queryByText("Harp Atlas / source-bound technical reader")).not.toBeInTheDocument();
});

it("returns from Workstreams to the existing knowledge reader", async () => {
  const user = userEvent.setup();
  window.location.hash = "#workstreams";
  render(<AtlasApp />);
  await user.click(screen.getByRole("button", { name: "Open Harp knowledge Atlas" }));
  expect(window.location.hash).toBe("#knowledge");
  expect(screen.getByText("Harp Atlas / source-bound technical reader")).toBeInTheDocument();
});
```

- [ ] **Step 2: Run RED**

```sh
cd atlas
node_modules/.bin/vitest run src/app/ReaderApp.test.tsx src/app/WorkstreamsDashboard.test.tsx --configLoader runner --no-cache
```

- [ ] **Step 3: Split the shell at the top of `AtlasApp`**

```tsx
if (route.kind === "workstreams") {
  return (
    <WorkstreamsDashboard
      snapshot={workstreamsReference}
      onNavigate={(href) => {
        window.location.hash = href;
      }}
    />
  );
}
```

Place this branch after route state/effect setup and before any
reader-only corpus lookups, so the console does not construct the reader shell.
The existing reader shell remains the return value for all other routes. Add
`case "workstreams": return "Workstreams";` to the exhaustive label switch.
Add one reader top-nav button with visible text `Console / Workstreams`,
`aria-label="Open Console / Workstreams"`, and
`onClick={() => navigate({ kind: "workstreams" })}`. Preserve every existing
button and content branch.

- [ ] **Step 4: Run app tests and commit**

```sh
cd atlas
node_modules/.bin/vitest run src/app --configLoader runner --no-cache
node_modules/.bin/tsc -b
cd ..
git add atlas/src/app/AtlasApp.tsx atlas/src/app/ReaderApp.test.tsx atlas/src/app/WorkstreamsDashboard.test.tsx
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "feat(atlas): integrate workstreams console"
```

### Task 9: Implement the reviewed schematic visual system

**Files:**
- Create: `atlas/src/styles/workstreams.css`
- Modify: `atlas/src/app/WorkstreamsDashboard.tsx`
- Modify: `atlas/src/app/WorkstreamsDashboard.test.tsx`

- [ ] **Step 1: Add structural accessibility assertions before CSS**

```tsx
it("exposes the console structure without relying on color", () => {
  const { container } = render(
    <WorkstreamsDashboard snapshot={workstreamsReference} onNavigate={vi.fn()} />,
  );
  expect(container.firstElementChild).toHaveClass("workstreams-shell");
  expect(screen.getAllByRole("heading", { level: 1 })).toHaveLength(1);
  expect(screen.getByRole("searchbox", { name: "Search workstreams" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Clear search" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Queue local command" })).toBeInTheDocument();
  expect(screen.getByRole("region", {
    name: "Workstream watchlist, scroll for more columns",
  })).toHaveAttribute("tabindex", "0");
  expect(screen.getByText("View 12-hour samples")).toBeInTheDocument();
  expect(container.querySelectorAll(".lifecycle-rail [data-state]")).toHaveLength(20);
  expect(screen.getAllByText("Blocked", { exact: true }).length).toBeGreaterThan(0);
});
```

In the same test, inspect `h1, h2, h3` in document order and assert no heading
level jumps by more than one. Add an accessible name to every icon-only button;
Lucide SVGs inside labeled buttons remain `aria-hidden="true"`.

- [ ] **Step 2: Import the stylesheet only from the dashboard**

Add `import "../styles/workstreams.css";` to `WorkstreamsDashboard.tsx`. Do not import it from a reader component.

- [ ] **Step 3: Define isolated console tokens**

```css
.workstreams-shell {
  --ops-canvas: #020617;
  --ops-sidebar: #0f172a;
  --ops-panel: #131c2e;
  --ops-panel-raised: #1e293b;
  --ops-line: #334155;
  --ops-cyan: #38bdf8;
  --ops-green: #3fb950;
  --ops-amber: #d29922;
  --ops-red: #f85149;
  --ops-text: #f8fafc;
  --ops-muted: #94a3b8;
  min-height: 100dvh;
  color: var(--ops-text);
  background: var(--ops-canvas);
  font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
}

.workstreams-shell :focus-visible {
  outline: 2px solid var(--ops-cyan);
  outline-offset: 2px;
  box-shadow: none;
}
```

The full-viewport shell covers the inherited blueprint body grid. Do not modify reader `:root`, `body`, or existing tokens.

- [ ] **Step 4: Build the desktop schematic layout**

Implement a 224px fixed sidebar, sticky 64px header offset by the sidebar, 12–16px panel padding, one-pixel nested borders, no gradients/glow/drop shadows, maximum 6px radius, two-column cards plus a narrow velocity column at 1280px+, one saturated accent per resting card, visible 12px lifecycle labels, and monospace tabular evidence lines.

- [ ] **Step 5: Add responsive and motion rules**

At 1279px, move Agent Velocity below cards and reduce the sidebar to a 72px icon rail while keeping 24px targets and tooltips. At 899px, use a top drawer/header, one-column content, 16px inputs, 44px controls with 8px gaps, and focusable horizontal watchlist overflow. Use only opacity/transform transitions of 120–180ms and disable them under `prefers-reduced-motion: reduce`. Do not animate the SVG line.

- [ ] **Step 6: Add print behavior**

Hide the local command form and fit the watchlist in print while keeping reference-snapshot provenance visible.

- [ ] **Step 7: Run focused checks and commit**

```sh
cd atlas
node_modules/.bin/vitest run src/app/WorkstreamsDashboard.test.tsx --configLoader runner --no-cache
node_modules/.bin/eslint . --ignore-pattern dist --ignore-pattern target
node_modules/.bin/tsc -b
cd ..
git add atlas/src/styles/workstreams.css atlas/src/app/WorkstreamsDashboard.tsx atlas/src/app/WorkstreamsDashboard.test.tsx
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "style(atlas): add schematic workstreams console"
```

### Task 10: Preserve offline export and document the route

**Files:**
- Modify: `atlas/tests/static-export.test.mjs`
- Modify: `atlas/README.md`
- Modify: `README.md`
- Regenerate: `atlas/dist/harp-atlas.html`
- Regenerate: `atlas/dist/harp-atlas.receipt.json`
- Regenerate if changed: `atlas/src/content/generated/corpus.json`

- [ ] **Step 1: Extend the static-export test before regeneration**

Append this exact block to the existing offline-export test while retaining its
current no-fetch, no-external-script, no-external-stylesheet, and font guards:

```js
for (const label of [
  "Console / Workstreams",
  "Harp Workstreams",
  "REFERENCE SNAPSHOT",
  "Crouzeix Conjecture",
  "Autodiff Geometry",
  "Agentic Research",
  "NNG4 / Foundations",
  "Research",
  "Spec",
  "Formalization",
  "Scroll for more columns",
]) {
  assert.match(decodedModule, new RegExp(label.replace("/", "\\/")));
}
assert.doesNotMatch(atlasShell, /<(?:script|link|img)[^>]+https?:\/\//i);
```

Do not reject citation URLs embedded in canonical prose. Do not assert minified
Lucide symbol names; verify visible labels and the absence of executable or
render-blocking external assets.

- [ ] **Step 2: Document the route accurately**

Add this paragraph under `atlas/README.md`'s Reading and diagnosis section:

```md
Open `#workstreams` for the Harp Workstreams console. It renders a validated,
repository-owned reference snapshot and keeps submitted command text in a
bounded tab-local queue. The browser does not execute commands or read live
agent services.
```

Add this bullet under the top-level `README.md` product description and this
route example under its Atlas section:

```md
- **Workstreams console:** an offline operational view of proof, research,
  evaluator, and agentic-engineering work with explicit evidence boundaries.

Open `atlas/dist/harp-atlas.html#workstreams` for the offline Workstreams
reference snapshot.
```

Do not call it live monitoring or production control.

- [ ] **Step 3: Run the complete Atlas gate and regenerate outputs**

```sh
mise run verify-atlas
```

Expected: lint, typecheck, all Vitest suites, production build, HTML export, and decoded export test pass. Inspect `atlas/src/content/generated/corpus.json`, `atlas/dist/harp-atlas.html`, and `atlas/dist/harp-atlas.receipt.json` together.

- [ ] **Step 4: Commit docs and generated artifacts**

```sh
git add README.md atlas/README.md atlas/tests/static-export.test.mjs atlas/dist/harp-atlas.html atlas/dist/harp-atlas.receipt.json
if ! git diff --quiet -- atlas/src/content/generated/corpus.json; then git add atlas/src/content/generated/corpus.json; fi
git diff --cached --check
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "docs(atlas): publish workstreams console"
```

### Task 11: Browser verification at three viewport regimes

**Files:**
- Inspect: `atlas/dist/harp-atlas.html`
- Inspect: `/private/tmp/harp-workstreams-1440.png`
- Inspect: `/private/tmp/harp-workstreams-1024.png`
- Inspect: `/private/tmp/harp-workstreams-390.png`
- Modify on a failed check: the owning dashboard source, test, or style file

- [ ] **Step 1: Start the application**

Run `cd atlas && node_modules/.bin/vite --host 127.0.0.1 --port 4178 --strictPort` in a terminal session. Use the source app for interactions and the checked-in `dist/harp-atlas.html` separately for offline checks.

- [ ] **Step 2: Verify 1440px desktop and accessibility**

```sh
agent-browser skills get core >/dev/null
agent-browser --session harp-workstreams open 'http://127.0.0.1:4178/#workstreams'
agent-browser --session harp-workstreams set viewport 1440 1000
agent-browser --session harp-workstreams snapshot -i
agent-browser --session harp-workstreams screenshot --full /private/tmp/harp-workstreams-1440.png
agent-browser --session harp-workstreams a11y --tags wcag2a,wcag2aa
```

Expected: no Critical accessibility violation; fixed sidebar/header; two-column workstream field; narrow velocity panel; all four cards, three insights, and watchlist in document order.

- [ ] **Step 3: Exercise interactions**

Start with `agent-browser --session harp-workstreams network requests --clear`.
Using fresh snapshot refs: press `Meta+k` to focus search; fill it with
`crouzeix`; activate Clear search; focus the first Research tab and press
ArrowRight; activate `View 12-hour samples`; fill `Local command` with `verify
Jin route` and press Shift+Enter; activate Knowledge, then `Console /
Workstreams`, and verify the queued text persists. Run `agent-browser --session
harp-workstreams network requests --type xhr,fetch` and expect no rows. Inspect
the page for no execution or live-connection affordance.

- [ ] **Step 4: Verify 1024px and 390px layouts**

```sh
agent-browser --session harp-workstreams set viewport 1024 900
agent-browser --session harp-workstreams screenshot --full /private/tmp/harp-workstreams-1024.png
agent-browser --session harp-workstreams set viewport 390 844
agent-browser --session harp-workstreams screenshot --full /private/tmp/harp-workstreams-390.png
agent-browser --session harp-workstreams snapshot -i
```

Expected at 1024px: accessible icon rail and velocity below cards. Expected at 390px: top drawer, one-column sections, 44px controls, 16px input, visible table-overflow cue, and no page-level horizontal overflow.

- [ ] **Step 5: Verify the checked-in offline export**

```sh
offline_url="file://$PWD/atlas/dist/harp-atlas.html#workstreams"
agent-browser --session harp-workstreams-offline --allow-file-access open "$offline_url"
agent-browser --session harp-workstreams-offline set offline on
agent-browser --session harp-workstreams-offline snapshot -i
agent-browser --session harp-workstreams-offline a11y --tags wcag2a,wcag2aa
agent-browser close --session harp-workstreams
agent-browser close --session harp-workstreams-offline
```

Expected: the same dashboard and local interactions without external requests.

- [ ] **Step 6: Repair browser findings RED-first**

For each defect, add the smallest failing regression, repair it, rerun the focused test, then repeat Tasks 10 and 11. Commit repairs by concern.

### Task 12: Final review, receipt, and landing handoff

**Files:**
- Create: `docs/superpowers/reviews/2026-09-03-harp-workstreams-dashboard-spec-review.md`
- Create: `docs/superpowers/reviews/2026-09-03-harp-workstreams-dashboard-quality-review.md`
- Modify last: `docs/import-receipt.md`

- [ ] **Step 1: Run the full non-Lean feature gate**

```sh
mise run verify-atlas
mise run verify-rust
git diff --check
git status --short --branch
```

Expected: both verifier tasks and the whitespace check pass; status contains
only intended review/receipt work, if any. This feature does not modify Lean, so
do not run Lean during iteration. The final landing gate in Step 4 must run the
full repository verifier against the pinned warm cache.

- [ ] **Step 2: Obtain independent reviews of the frozen candidate**

Freeze the source candidate with the Task 10 commit first. Request two
independent read-only reviews of that SHA and save their complete reports at the
two review paths above. The Spec review checks every approved section, interaction,
offline boundary, evidence label, and generated artifact. The quality review
checks React state, session parsing, keyboard behavior, responsive overflow, CSS
scoping, contrast, target sizes, icon semantics, bundle behavior, and
existing-reader regressions. Each report records reviewed commit/tree, reviewer
identity/model, verdict, and Critical/Important/Minor findings. Repair every
Critical or Important finding in a separate commit and repeat both reviews on
the new immutable SHA until both verdicts are PASS with zero Critical and zero
Important findings. Reports may point to the reviewed source SHA even though a
later metadata-only commit records the reports and receipt.

- [ ] **Step 3: Refresh the tracked payload digest last**

Stage the final review reports, then run:

```sh
git add docs/superpowers/reviews/2026-09-03-harp-workstreams-dashboard-spec-review.md \
  docs/superpowers/reviews/2026-09-03-harp-workstreams-dashboard-quality-review.md
cargo run -q -p harp -- repository verify
```

Expected first result: failure `import receipt payload digest is stale;
expected <sha256>`. Because the verifier hashes `git ls-files`, first stage
every intended tracked payload file except `docs/import-receipt.md`. Update
only that digest in `docs/import-receipt.md` with `apply_patch`, stage it, and
rerun `cargo run -q -p harp -- repository verify` until it exits zero.

- [ ] **Step 4: Run the full repository gate through the warm Lean cache**

Before the full gate, prove that the feature worktree reuses the canonical Lean
cache without invoking Lake or changing dependencies:

```sh
primary=/Users/bytedance/workspace/harp
test -L formalization/lean/.lake
test "$(readlink formalization/lean/.lake)" = "$primary/formalization/lean/.lake"
test -s formalization/lean/lean-toolchain
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run verify
```

Expected: the cache symlink and pinned toolchain checks pass, then the complete
repository gate passes without `lake update`, cache hydration, or dependency
mutation. If the cache precondition fails, stop and report it; do not rebuild or
download common Lean dependencies. If the full gate changes tracked generated
bytes, review and stage them, repeat Step 3's digest refresh, then rerun the full
gate.

- [ ] **Step 5: Commit the review metadata and verify clean state**

```sh
git diff --cached --check
git -c user.name=phi9t -c user.email=phissenschaft@gmail.com \
  -c core.hooksPath=/dev/null commit -m "chore: record dashboard review and receipt"
git status --short --branch
```

Expected: clean feature worktree, focused commits, and no ignored node modules, browser profiles, screenshots, or build directories staged.

- [ ] **Step 6: Present the landing choice**

Use `superpowers:finishing-a-development-branch`. Do not merge to `master`, remove worktrees, or push without explicit project-owner instruction.

## Definition of done

- `#workstreams` is a first-class route with a dedicated scoped console shell.
- Four named workstreams, focus strip, velocity panel, insights, and watchlist render from validated reference data.
- Search, shortcuts, tabs, chart disclosure, local queue, route navigation, and session persistence work.
- The UI follows the reviewed near-monochrome schematic language and passes target-size, contrast, focus, status-label, reduced-motion, and responsive checks.
- Existing Atlas reader routes remain unchanged.
- The static single-file export works offline with no runtime fetch or external asset dependency.
- Source, tests, docs, generated Atlas files, and the repository receipt agree.
- Independent Spec and quality reviews pass the exact final candidate.
- No Lean dependency, live agent state, proof claim, GitHub state, or publication authority changes.
