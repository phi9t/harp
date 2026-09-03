# Harp Workstreams dashboard design

## Summary

Add a first-class `#workstreams` route to Harp Atlas. The route is a dense
desktop operations dashboard for monitoring research workstreams, proof effort,
agent activity, and verification state. Its visual thesis is a working research
schematic: near-monochrome technical surfaces, nested hairline enclosures,
directly labeled lifecycle nodes, and quantitative evidence immediately below
the structure it verifies. It uses its own fixed-sidebar console shell while
preserving every existing Atlas reader route and static-export contract.

The first release is an offline, deterministic interface backed by a typed
repository-owned reference snapshot. It does not start agents, execute shell
commands, read private runtime state, or call a network service. The command
entry updates an in-memory recent-command list so the interaction is real
without widening frontend authority. A later runtime adapter can replace the
reference snapshot only after it has a separate verified data contract.

## Product job and audience

The dashboard has one job: let a research lead answer, within a few seconds,
which workstream needs attention and what the next intervention should be.

The audience is an MTS-level researcher or research engineer overseeing several
formalization, research, evaluation, and publication tracks. They already know
the terminology. The interface should emphasize state, blockers, provenance,
and next action instead of teaching basic concepts.

## Placement in Atlas

`AtlasRoute` gains a `{ kind: "workstreams" }` variant, parsed from and
formatted as `#workstreams`. `AtlasApp` selects between two shells:

- `workstreams` renders a dedicated `WorkstreamsDashboard`;
- all current routes continue through the existing `atlas-shell`, reader
  header, navigation, and content switch.

The existing top navigation gains a Workstreams entry so readers can enter the
console. The dashboard sidebar includes an Atlas link that returns to the
existing knowledge experience. No current route, document contract, or reader
component is renamed or removed.

The visual transition is intentional: Atlas readers remain a light blueprint
workspace, while Workstreams is a dark operations console. The Workstreams
entry is labeled `Console / Workstreams`, uses a terminal-grid icon, and exposes
the destination in its accessible name. `.workstreams-shell` overrides the
global body grid, font family, surface color, and focus ring so the light reader
theme cannot leak into the console. Returning to Atlas restores the existing
reader shell unchanged.

## Information architecture

The desktop view fills the viewport and follows this hierarchy.

```text
+----------------------+-----------------------------------------------+
| HARP / R&D PLATFORM  | top header: search, health, last updated      |
|                      +-----------------------------------------------+
| Workstreams          | good morning + local snapshot chip           |
| Runs                 | focus strip                                  |
| Agents               +-----------------------------------------------+
| Evaluators           | workstream cards       | agent velocity      |
| Knowledge            |                        |                     |
| Settings             +-----------------------------------------------+
|                      | latest research insights, 3 columns           |
| Recently Active      +-----------------------------------------------+
|                      | watchlist table                               |
| online status        |                                               |
+----------------------+-----------------------------------------------+
```

### Sidebar

The fixed left sidebar contains the Harp wordmark, the small label `R&D
PLATFORM`, primary navigation, a three-row Recently Active list, and the active
agent row at the bottom. The active agent row uses a green status dot, avatar,
name, and `Online` status. The labels are operational navigation, not a second
copy of the existing Atlas reader menu.

Behavior in the first release:

- Workstreams selects the dashboard root.
- Runs scrolls to the Recent Runs section inside Agent Velocity.
- Agents focuses the Recently Active block.
- Evaluators opens the evaluator-facing artifact tab in Latest Research
  Insights.
- Knowledge navigates to `#knowledge`.
- Settings opens a small read-only panel describing the local reference-data
  mode and available keyboard shortcuts.

### Top header

The header is short and quiet. A wide search field includes a search icon,
placeholder, and `Cmd K` hint. On the right, a green dot precedes `System
Health` and `Operational`, followed by a monospace last-updated timestamp.

Search filters workstream cards and watchlist rows by title, status, label,
blocker, and next action. `Cmd K` or `Ctrl K` focuses the field. The health
label describes the loaded dashboard model and UI integrity, not an external
production service. A visible `REFERENCE SNAPSHOT` chip near the page greeting
prevents that distinction from being hidden.

### Focus strip

A single wide focus strip appears below the greeting. Its surface stays neutral
and uses one amber left rule plus an amber `FOCUS NOW` label, rather than a
fully filled alert panel. It contains:

- `Focus now`;
- the highest-priority blocked workstream title;
- one concise sentence explaining the blocker;
- one compact action link.

The panel is computed from the typed snapshot's explicit `priority` and
`status` fields. It is not inferred from color or card order.

### Active workstreams

The main grid is asymmetric: a large workstreams field at left and the smaller
Agent Velocity card at right. The workstreams field uses two columns at wide
desktop widths. It contains the four reference workstreams:

1. Crouzeix Conjecture
2. Autodiff Geometry
3. Agentic Research
4. NNG4 / Foundations

Each card has a title, status chip, last-run metadata, directly labeled
lifecycle schematic, and one status-specific detail:

- theorem work shows `9/14 lemmas`;
- planning work shows `5/7 planned`;
- research work shows `Drafting`;
- complete foundation work shows `All levels proved`.

Lifecycle rails encode named stages such as `Research`, `Spec`, `Formalize`, and
`Verify`. Each rail is a sequence of labeled nodes joined by one-pixel edges,
not an unlabeled segmented progress bar. The node label is always visible below
the rail. Completed nodes include a check glyph, the active node has a cyan fill
and ring, queued nodes use a muted hollow mark, and blocked nodes include a red
stop glyph. Tooltips provide secondary detail on hover and focus, but never own
the stage name or state.

Within every card, the visual order is diagram first and numbers second: title
and status, lifecycle rail, then a compact monospace evidence line such as
`9/14 lemmas · last run 2h ago`. This mirrors the reference diagram's
overview-then-quantification sequence.

### Agent Velocity

The right-hand card is narrower and vertically dense. It shows:

- the active agent identity;
- a restrained slate line chart for the last twelve hours, with cyan used only
  for the current point;
- `12h` and `+18%`;
- a primary `View Recent Runs` button;
- three recent runs with status dot, short SHA, elapsed time, and relative
  timestamp;
- a bottom command input with a terminal icon and send button.

The chart is labeled `runs / hour`, names its minimum and maximum values, and is
followed by a one-sentence trend summary. A `View 12-hour samples` disclosure
opens a keyboard-reachable two-column time/value table. Hover and focus may
highlight one point, but no information exists only in a tooltip.

Submitting a non-empty command appends a local `queued` run to the visible list
and clears the field. It does not execute the command. The bounded queue is
validated and persisted in versioned `sessionStorage`, so navigation to the
reader and back within the same tab does not discard it; closing the tab clears
it. Empty commands are rejected in place. Shift+Enter and the send button share
the same handler. `View Recent Runs` focuses the list and announces its count
for assistive technology.

### Latest Research Insights

Three equal cards form the lower insight grid. Each card has a tab row:

- `Research`;
- `Spec`;
- `Formalization`.

Tabs switch the visible artifact summary, last-updated metadata, and action
link without changing routes. An amber inline `MISSING` badge remains visible
for unresolved evaluation evidence. It is an outlined, hatched token with the
literal word `MISSING`, visually distinct from the focus strip's single amber
rule. The prose follows Harp's evidence
discipline: no card calls a route proved, verified, or production-ready unless
its typed snapshot says so.

### Watchlist

The watchlist is a compact table with columns `Workstream`, `Status`, `Blocker`,
and `Next action`. Four rows use small status chips and action links. Search
applies to the table as well as the card grid. When no row matches, the table
retains its header and shows one explicit empty-state row.

## Data model

The dashboard data lives in a dedicated TypeScript module, separate from JSX.
It defines discriminated unions and validates the reference snapshot at module
load. The primary types are:

```ts
type WorkstreamStatus =
  | "active"
  | "blocked"
  | "planning"
  | "drafting"
  | "complete-local";

type StageStatus = "complete" | "active" | "queued" | "blocked";

type WorkstreamSnapshot = {
  id: string;
  title: string;
  status: WorkstreamStatus;
  priority: number;
  lastRun: string;
  progressLabel: string;
  stages: Array<{ name: string; status: StageStatus }>;
  blocker: string | null;
  nextAction: string;
};
```

Additional types cover recent runs, agent identity, sparkline samples, insight
artifacts, health state, and watchlist rows. The initial snapshot is declared as
`source: "repository-reference"` with a fixed `observedAt` timestamp. UI code
does not hardcode workstream names, counts, statuses, or stage colors.

The dashboard does not import private XDG files or postmortem receipts into the
browser bundle. A future live adapter must project a public-safe schema and is
outside this design.

## Visual system

### Reference-derived aesthetic

The visual reference is the infra-oriented DeepSeek-V3 diagram at
`https://deepseek-v3.ezyang.com/studies/01-deepseek-diagram.html`. Its
transferable qualities are structural, not merely dark-mode styling:

- near-monochrome ground with color spent sparingly;
- nested one-pixel schematic enclosures instead of floating card shadows;
- direct labels on visual nodes and edges;
- monospace as the quantitative engineering register; and
- an overview schematic followed by a table that verifies the numbers.

Harp adopts those qualities while rejecting the reference's 11px text, 19px
controls, hover-dependent density, article-column composition, and warm brown
palette. The console uses a cool slate ground, accessible targets, and visible
state labels. At rest, each card may show only one saturated accent; all other
structure is slate, text, or hairline.

### Color tokens

- `--ops-canvas: #020617` for the application background
- `--ops-sidebar: #0f172a` for the fixed navigation plane
- `--ops-panel: #131c2e` for schematic fields
- `--ops-panel-raised: #1e293b` for interactive surfaces
- `--ops-line: #334155` for low-contrast borders
- `--ops-cyan: #38bdf8` for the single active stage and primary action
- `--ops-green: #3fb950` for completed and healthy state
- `--ops-amber: #d29922` for the focus rule and evidence warnings
- `--ops-red: #f85149` for blocked state
- `--ops-text: #f8fafc` and `--ops-muted: #94a3b8` for copy

There are no gradients or ambient glows. Depth comes from adjacent navy values,
one-pixel nested enclosures, and inset lines. Console surfaces do not use soft
drop shadows. Corner radii remain between 4px and 6px. Status colors always
pair with text, icon shape, or both. Every text/background token pair is checked
for at least 4.5:1 contrast at the final implementation gate.

### Typography

- Display and UI: `system-ui`, `-apple-system`, `Segoe UI`, sans-serif
- Dense metadata: `SFMono-Regular`, `Cascadia Code`, `Roboto Mono`, monospace

The static export may not load web fonts or add `@font-face`; this preserves
the existing offline single-file contract. The page title is 30px-34px, card
titles 15px-18px, body copy 14px-15px, and metadata never falls below 12px.
Mobile prose, form controls, and command input text use at least 16px. Monospace
is the explicit quantitative register for SHAs, timestamps, counts, durations,
stage totals, chart labels, and deltas; prose remains sans-serif.

### Signature element

The signature is the labeled research-lifecycle schematic repeated across
workstream cards and echoed by the focus strip. It turns Harp's actual execution
model into the page's main visual grammar. It is not decorative progress
chrome: each node has a visible stage identity, state icon, connector, tooltip,
and accessible label.

### Icons

Add `lucide-react` and use its icons for navigation, search, terminal input,
health, clock, arrow, and action affordances. Icon-only buttons require
`aria-label` and a tooltip. Text glyphs do not stand in for UI icons.

## Component boundaries

The implementation adds these focused modules:

- `WorkstreamsDashboard.tsx`: page composition and local interaction state
- `WorkstreamCard.tsx`: one workstream and lifecycle rail
- `AgentVelocityCard.tsx`: chart, run list, and local command entry
- `ResearchInsights.tsx`: three cards and tab state
- `WatchlistTable.tsx`: filtered operational table
- `workstreams.ts`: types, validation, and reference snapshot
- `workstreams.css`: styles scoped under `.workstreams-shell`

The chart is a small inline SVG generated from numeric samples. It contains no
hardcoded path data and no external charting dependency. A semantic table and
plain-language summary expose the same values. Components share status-label
and icon mappings from one typed utility so styles and semantics cannot drift
independently. `lucide-react` imports are named and tree-shaken; the static
export test continues to reject external fonts, images, stylesheets, and URL
assets.

## Responsive behavior

- At 1280px and above, the dashboard uses the full fixed-sidebar layout and
  two-column workstream grid.
- Between 900px and 1279px, Agent Velocity moves below the workstream cards and
  the sidebar narrows to an icon rail with tooltips and accessible names.
- Below 900px, the sidebar becomes a top drawer, workstream and insight grids
  become one column, the watchlist becomes a horizontally scrollable table, and
  the focus strip retains its priority at the top.

The dense desktop layout is the primary target, but the 12px metadata floor is
never sacrificed to keep more panels above the fold. Keyboard access and
content order remain correct at every width. The scrollable mobile watchlist is
focusable, has a visible `Scroll for more columns` cue, and supports keyboard or
single-pointer scrolling without drag-only interaction. Reduced-motion users
receive no animated chart or card transitions.

## Accessibility and interaction rules

- Navigation uses real buttons or links with current-route state.
- Search has a visible label for assistive technology and a clear control when
  non-empty.
- Tabs implement `tablist`, `tab`, and `tabpanel` semantics plus arrow-key
  navigation.
- Status dots and progress colors always have text equivalents.
- Tooltips appear on hover and keyboard focus.
- Every web control has at least a 24px by 24px target. At touch breakpoints,
  navigation, tabs, send, clear-search, disclosure, and icon buttons use at
  least 44px by 44px targets with 8px separation.
- `.workstreams-shell :focus-visible` overrides the light Atlas focus style
  with a minimum 2px cyan ring, 2px offset, and at least 3:1 state contrast on
  every console surface. Sticky header and sidebar offsets keep focus visible.
- The command form announces local queue success and validation errors through
  an `aria-live` region.

## Tests and verification

Implementation is test-driven. Required tests are:

1. Route parsing and formatting for `#workstreams`, including unknown-route
   fallback preservation.
2. Reference snapshot validation, including duplicate IDs, empty stages, invalid
   stage transitions, and missing blocker/next-action fields.
3. Dashboard rendering of the four required workstreams, focus strip, health
   state, three insight cards, and four watchlist rows.
4. Search filtering and empty state.
5. `Cmd K` / `Ctrl K` focus behavior.
6. Insight tab mouse and keyboard behavior.
7. Local command submission, empty rejection, Shift+Enter, and recent-run update.
8. Session queue persistence across Workstreams-to-Atlas route changes, bounded
   validation, and tab-scoped clearing.
9. Chart summary, point focus, and sample-table equivalence.
10. Pointer-target, visible focus, status-label, and mobile table-overflow
    accessibility assertions.
11. Existing Atlas route tests and reader component tests.
12. Static single-file export checks, including required dashboard labels,
   named icon tree-shaking, no network fetch, and no external stylesheet or
   web-font dependency.

The implementation gate is:

```sh
cd atlas
corepack pnpm run typecheck
corepack pnpm run test
corepack pnpm run lint
corepack pnpm run build
corepack pnpm run test:export
```

The controller also opens the built page in a browser at desktop, narrow
desktop, and mobile widths. It verifies route entry, filtering, tabs, command
submission, focus order, overflow, contrast, and static-export behavior.

## Files and derived artifacts

Expected source changes:

- `atlas/package.json` and `atlas/pnpm-lock.yaml`
- `atlas/src/app/AtlasApp.tsx`
- `atlas/src/app/routes.ts` and `atlas/src/app/routes.test.ts`
- new dashboard component and test files under `atlas/src/app/`
- new typed data module under `atlas/src/content/`
- new scoped stylesheet under `atlas/src/styles/`
- `atlas/tests/static-export.test.mjs`

After source verification, regenerate both required derived artifacts together:

- `atlas/src/content/generated/corpus.json` when the canonical content build
  changes it
- `atlas/dist/harp-atlas.html` and `atlas/dist/harp-atlas.receipt.json` through
  the static export command

Any final landing also refreshes public-release bookkeeping and
`docs/import-receipt.md` through the repository verifier.

## Non-goals

- No live agent daemon, websocket, polling API, database, or authentication.
- No shell execution from the browser.
- No new authority over Kata, roborev, AgentsView, GitHub, or provider sessions.
- No mutation of proof, evaluator, or publication state.
- No claim that reference snapshot timestamps or statuses are current runtime
  measurements.
- No replacement of existing Atlas research routes.

## Acceptance criteria

- `#workstreams` renders the approved fixed-sidebar operations dashboard.
- All requested sections, workstream names, metrics, statuses, tabs, table
  columns, and command controls are present.
- Search, tabs, keyboard shortcut, navigation, command queue, and recent-runs
  interactions work.
- The page clearly identifies its data as a repository reference snapshot.
- Existing Atlas routes and offline export continue to work.
- The visual system is deep navy, cyan/green/amber/red, compact, border-led,
  low-radius, schematic, near-monochrome at rest, and gradient-free.
- The dashboard uses the reference's nested hairline, direct-label, and
  overview-then-quantification grammar without copying its undersized controls.
- The full Atlas verification gate and browser checks pass.
