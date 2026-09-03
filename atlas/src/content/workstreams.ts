const ID_RE = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
const ISO_DATE_RE = /^\d{4}-\d{2}-\d{2}$/;
const DASHBOARD_HREFS = [
  "#crouzeix-conjecture",
  "#mathematical-foundations",
  "#knowledge",
] as const;
const WORKSTREAM_STATUSES = [
  "active",
  "blocked",
  "planning",
  "drafting",
  "complete-local",
] as const;
const STAGE_STATUSES = ["complete", "active", "queued", "blocked"] as const;
const INSIGHT_ARTIFACT_KINDS = [
  "research",
  "spec",
  "formalization",
] as const;
const EVIDENCE_STATUSES = ["supported", "missing"] as const;
const RUN_KINDS = ["completed", "failed", "queued"] as const;
const AGENT_STATUSES = ["online", "offline"] as const;

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
  | {
      kind: "completed";
      id: string;
      label: string;
      sha: string;
      elapsed: string;
      relativeTime: string;
    }
  | {
      kind: "failed";
      id: string;
      label: string;
      sha: string;
      elapsed: string;
      relativeTime: string;
    }
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

function record(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as Record<string, unknown>;
}

function array(value: unknown, label: string): readonly unknown[] {
  if (!Array.isArray(value)) {
    throw new Error(`${label} must be an array`);
  }
  return value;
}

function nonEmptyString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${label} must be a non-empty string`);
  }
  return value;
}

function nullableString(value: unknown, label: string): string | null {
  if (value === null) {
    return null;
  }
  return nonEmptyString(value, label);
}

function integer(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value) || !Number.isInteger(value)) {
    throw new Error(`${label} must be a finite integer`);
  }
  return value;
}

function nonNegativeFiniteNumber(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    throw new Error(`${label} must be a non-negative finite number`);
  }
  return value;
}

function positiveInteger(value: unknown, label: string): number {
  const parsed = integer(value, label);
  if (parsed <= 0) {
    throw new Error(`${label} must be positive`);
  }
  return parsed;
}

function enumValue<T extends readonly string[]>(
  value: unknown,
  label: string,
  allowed: T,
): T[number] {
  if (typeof value !== "string" || !allowed.includes(value)) {
    throw new Error(`${label} must be one of ${allowed.join(", ")}`);
  }
  return value as T[number];
}

function identifier(value: unknown, label: string): string {
  const parsed = nonEmptyString(value, label);
  if (!ID_RE.test(parsed)) {
    throw new Error(`${label} must match ${ID_RE.source}`);
  }
  return parsed;
}

function isoDatetime(value: unknown, label: string): string {
  const parsed = nonEmptyString(value, label);
  const date = new Date(parsed);
  if (!Number.isFinite(date.getTime()) || date.toISOString() !== parsed) {
    throw new Error(`${label} must be an exact ISO datetime`);
  }
  return parsed;
}

function isoDate(value: unknown, label: string): string {
  const parsed = nonEmptyString(value, label);
  if (!ISO_DATE_RE.test(parsed)) {
    throw new Error(`${label} must be a YYYY-MM-DD date`);
  }
  const date = new Date(`${parsed}T00:00:00.000Z`);
  if (!Number.isFinite(date.getTime()) || date.toISOString().slice(0, 10) !== parsed) {
    throw new Error(`${label} must be a YYYY-MM-DD date`);
  }
  return parsed;
}

function dashboardHref(value: unknown, label: string): DashboardHref {
  return enumValue(value, label, DASHBOARD_HREFS);
}

function uniqueValues(values: readonly string[], label: string): void {
  const seen = new Set<string>();
  for (const value of values) {
    if (seen.has(value)) {
      throw new Error(`${label} must be unique`);
    }
    seen.add(value);
  }
}

function parseStageSnapshot(input: unknown, index: number): StageSnapshot {
  const parsed = record(input, `stage[${index}]`);
  return {
    name: nonEmptyString(parsed.name, `stage[${index}].name`),
    status: enumValue(parsed.status, `stage[${index}].status`, STAGE_STATUSES),
    detail: nonEmptyString(parsed.detail, `stage[${index}].detail`),
  };
}

function parseWorkstreamSnapshot(input: unknown, index: number): WorkstreamSnapshot {
  const parsed = record(input, `workstreams[${index}]`);
  const status = enumValue(
    parsed.status,
    `workstreams[${index}].status`,
    WORKSTREAM_STATUSES,
  );
  const blocker = nullableString(parsed.blocker, `workstreams[${index}].blocker`);
  const stages = array(parsed.stages, `workstreams[${index}].stages`).map(
    (stage, stageIndex) => parseStageSnapshot(stage, stageIndex),
  );
  if (stages.length < 4 || stages.length > 6) {
    throw new Error(`workstreams[${index}].stages must contain 4-6 stages`);
  }
  uniqueValues(
    stages.map((stage) => stage.name),
    `workstreams[${index}].stages`,
  );
  if (status === "blocked") {
    if (blocker === null) {
      throw new Error(`workstreams[${index}].blocker is required for blocked workstreams`);
    }
    if (!stages.some((stage) => stage.status === "blocked")) {
      throw new Error(`workstreams[${index}] requires a blocked stage`);
    }
  }
  if (status === "complete-local") {
    if (blocker !== null) {
      throw new Error(`workstreams[${index}] complete-local workstreams cannot have a blocker`);
    }
    if (stages.some((stage) => stage.status !== "complete")) {
      throw new Error(`workstreams[${index}] complete-local workstreams require complete stages`);
    }
  }
  return {
    id: identifier(parsed.id, `workstreams[${index}].id`),
    title: nonEmptyString(parsed.title, `workstreams[${index}].title`),
    status,
    priority: positiveInteger(parsed.priority, `workstreams[${index}].priority`),
    lastRun: nonEmptyString(parsed.lastRun, `workstreams[${index}].lastRun`),
    progressLabel: nonEmptyString(
      parsed.progressLabel,
      `workstreams[${index}].progressLabel`,
    ),
    stages,
    blocker,
    nextAction: nonEmptyString(
      parsed.nextAction,
      `workstreams[${index}].nextAction`,
    ),
    href: dashboardHref(parsed.href, `workstreams[${index}].href`),
  };
}

function parseRecentRun(input: unknown, index: number): RecentRun {
  const parsed = record(input, `agent.runs[${index}]`);
  const kind = enumValue(parsed.kind, `agent.runs[${index}].kind`, RUN_KINDS);
  const id = identifier(parsed.id, `agent.runs[${index}].id`);
  if (kind === "queued") {
    const command = nonEmptyString(parsed.command, `agent.runs[${index}].command`);
    if (command.length > 240) {
      throw new Error(`agent.runs[${index}].command must be 240 characters or fewer`);
    }
    return {
      kind,
      id,
      command,
      queuedAt: isoDatetime(parsed.queuedAt, `agent.runs[${index}].queuedAt`),
    };
  }
  return {
    kind,
    id,
    label: nonEmptyString(parsed.label, `agent.runs[${index}].label`),
    sha: nonEmptyString(parsed.sha, `agent.runs[${index}].sha`),
    elapsed: nonEmptyString(parsed.elapsed, `agent.runs[${index}].elapsed`),
    relativeTime: nonEmptyString(
      parsed.relativeTime,
      `agent.runs[${index}].relativeTime`,
    ),
  };
}

function parseVelocitySample(input: unknown, index: number): VelocitySample {
  const parsed = record(input, `agent.velocity.samples[${index}]`);
  return {
    label: nonEmptyString(parsed.label, `agent.velocity.samples[${index}].label`),
    value: nonNegativeFiniteNumber(
      parsed.value,
      `agent.velocity.samples[${index}].value`,
    ),
  };
}

function parseAgentSnapshot(input: unknown): AgentSnapshot {
  const parsed = record(input, "agent");
  const velocity = record(parsed.velocity, "agent.velocity");
  const samples = array(velocity.samples, "agent.velocity.samples").map(
    (sample, index) => parseVelocitySample(sample, index),
  );
  if (samples.length !== 12) {
    throw new Error("agent.velocity.samples must contain exactly 12 samples");
  }
  const runs = array(parsed.runs, "agent.runs").map((run, index) =>
    parseRecentRun(run, index),
  );
  uniqueValues(
    runs.map((run) => run.id),
    "agent.runs ids",
  );
  const queuedRuns = runs.filter((run): run is QueuedRun => run.kind === "queued");
  if (queuedRuns.length > 8) {
    throw new Error("agent.runs must contain at most 8 queued runs");
  }
  return {
    id: identifier(parsed.id, "agent.id"),
    name: nonEmptyString(parsed.name, "agent.name"),
    status: enumValue(parsed.status, "agent.status", AGENT_STATUSES),
    initials: nonEmptyString(parsed.initials, "agent.initials"),
    velocity: {
      windowLabel: enumValue(
        velocity.windowLabel,
        "agent.velocity.windowLabel",
        ["12h"] as const,
      ),
      unit: enumValue(
        velocity.unit,
        "agent.velocity.unit",
        ["runs / hour"] as const,
      ),
      deltaLabel: nonEmptyString(velocity.deltaLabel, "agent.velocity.deltaLabel"),
      summary: nonEmptyString(velocity.summary, "agent.velocity.summary"),
      samples,
    },
    runs,
  };
}

function parseInsightArtifact(input: unknown, index: number, label: string): InsightArtifact {
  const parsed = record(input, `${label}.artifacts[${index}]`);
  return {
    kind: enumValue(
      parsed.kind,
      `${label}.artifacts[${index}].kind`,
      INSIGHT_ARTIFACT_KINDS,
    ),
    summary: nonEmptyString(
      parsed.summary,
      `${label}.artifacts[${index}].summary`,
    ),
    updatedAt: isoDate(
      parsed.updatedAt,
      `${label}.artifacts[${index}].updatedAt`,
    ),
    href: dashboardHref(parsed.href, `${label}.artifacts[${index}].href`),
    evidence: enumValue(
      parsed.evidence,
      `${label}.artifacts[${index}].evidence`,
      EVIDENCE_STATUSES,
    ),
  };
}

function parseInsightSnapshot(input: unknown, index: number): InsightSnapshot {
  const parsed = record(input, `insights[${index}]`);
  const artifacts = array(parsed.artifacts, `insights[${index}].artifacts`).map(
    (artifact, artifactIndex) =>
      parseInsightArtifact(artifact, artifactIndex, `insights[${index}]`),
  );
  if (artifacts.length !== INSIGHT_ARTIFACT_KINDS.length) {
    throw new Error(`insights[${index}] must include exactly one artifact of each kind`);
  }
  uniqueValues(
    artifacts.map((artifact) => artifact.kind),
    `insights[${index}].artifact kinds`,
  );
  return {
    id: identifier(parsed.id, `insights[${index}].id`),
    title: nonEmptyString(parsed.title, `insights[${index}].title`),
    evaluatorArtifact: enumValue(
      parsed.evaluatorArtifact,
      `insights[${index}].evaluatorArtifact`,
      INSIGHT_ARTIFACT_KINDS,
    ),
    artifacts,
  };
}

function parseWatchlistRow(input: unknown, index: number): WatchlistRow {
  const parsed = record(input, `watchlist[${index}]`);
  return {
    id: identifier(parsed.id, `watchlist[${index}].id`),
    workstreamId: identifier(
      parsed.workstreamId,
      `watchlist[${index}].workstreamId`,
    ),
    workstream: nonEmptyString(parsed.workstream, `watchlist[${index}].workstream`),
    status: enumValue(
      parsed.status,
      `watchlist[${index}].status`,
      WORKSTREAM_STATUSES,
    ),
    blocker: nullableString(parsed.blocker, `watchlist[${index}].blocker`),
    nextAction: nonEmptyString(
      parsed.nextAction,
      `watchlist[${index}].nextAction`,
    ),
    href: dashboardHref(parsed.href, `watchlist[${index}].href`),
  };
}

export function parseWorkstreamsSnapshot(input: unknown): WorkstreamsSnapshot {
  const parsed = record(input, "snapshot");
  if (parsed.schemaVersion !== "harp-workstreams/v1") {
    throw new Error("schemaVersion must be harp-workstreams/v1");
  }
  if (parsed.source !== "repository-reference") {
    throw new Error("source must be repository-reference");
  }
  const observedAt = isoDatetime(parsed.observedAt, "observedAt");
  const health = record(parsed.health, "health");
  if (health.status !== "operational") {
    throw new Error("health.status must be operational");
  }
  const workstreams = array(parsed.workstreams, "workstreams").map((workstream, index) =>
    parseWorkstreamSnapshot(workstream, index),
  );
  uniqueValues(
    workstreams.map((workstream) => workstream.id),
    "workstreams ids",
  );
  uniqueValues(
    workstreams.map((workstream) => String(workstream.priority)),
    "workstreams priorities",
  );
  const insights = array(parsed.insights, "insights").map((insight, index) =>
    parseInsightSnapshot(insight, index),
  );
  uniqueValues(
    insights.map((insight) => insight.id),
    "insights ids",
  );
  const watchlist = array(parsed.watchlist, "watchlist").map((row, index) =>
    parseWatchlistRow(row, index),
  );
  uniqueValues(
    watchlist.map((row) => row.id),
    "watchlist ids",
  );
  const workstreamIds = new Set(workstreams.map((workstream) => workstream.id));
  for (const row of watchlist) {
    if (!workstreamIds.has(row.workstreamId)) {
      throw new Error(`watchlist workstream ${row.workstreamId} must reference a known workstream`);
    }
  }
  return {
    schemaVersion: "harp-workstreams/v1",
    source: "repository-reference",
    observedAt,
    health: {
      status: "operational",
      label: nonEmptyString(health.label, "health.label"),
      detail: nonEmptyString(health.detail, "health.detail"),
    },
    workstreams,
    agent: parseAgentSnapshot(parsed.agent),
    insights,
    watchlist,
  };
}

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
        {
          name: "Research",
          status: "complete",
          detail: "Jin and Lorist–Schwenninger sources mapped",
        },
        {
          name: "Spec",
          status: "complete",
          detail: "Three route contracts frozen",
        },
        {
          name: "Formalize",
          status: "complete",
          detail: "Jin, LS, and Harp routes compiled",
        },
        {
          name: "Verify",
          status: "complete",
          detail: "Program verification recorded",
        },
        {
          name: "Publish",
          status: "active",
          detail: "Prepare public presentation and external review",
        },
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
        {
          name: "Acquire",
          status: "complete",
          detail: "Core source registry captured",
        },
        {
          name: "Parse",
          status: "complete",
          detail: "Normalize source structure and notation",
        },
        {
          name: "Curriculum",
          status: "blocked",
          detail: "Source-to-lesson dependency graph remains incomplete",
        },
        {
          name: "Formalize",
          status: "queued",
          detail: "Extend finite-coordinate foundations",
        },
        {
          name: "Verify",
          status: "queued",
          detail: "Run focused Lean route only after formalization",
        },
      ],
      blocker:
        "The source-to-curriculum map is incomplete beyond finite-coordinate identities.",
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
        {
          name: "Research",
          status: "complete",
          detail: "Public evidence graph captured",
        },
        {
          name: "Synthesize",
          status: "complete",
          detail: "RSI and harness model assembled",
        },
        {
          name: "Design",
          status: "active",
          detail: "Production-control interfaces under review",
        },
        {
          name: "Prototype",
          status: "queued",
          detail: "Bounded implementation slices remain",
        },
        {
          name: "Evaluate",
          status: "queued",
          detail: "Matched-cost evaluation remains",
        },
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
        {
          name: "Natural",
          status: "complete",
          detail: "Natural-number levels complete",
        },
        {
          name: "Addition",
          status: "complete",
          detail: "Addition levels complete",
        },
        {
          name: "Multiplication",
          status: "complete",
          detail: "Multiplication levels complete",
        },
        {
          name: "Power",
          status: "complete",
          detail: "Power levels complete",
        },
        {
          name: "Advanced",
          status: "complete",
          detail: "Introductory game closure complete",
        },
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
        { label: "05:00", value: 2 },
        { label: "06:00", value: 3 },
        { label: "07:00", value: 3 },
        { label: "08:00", value: 5 },
        { label: "09:00", value: 4 },
        { label: "10:00", value: 6 },
        { label: "11:00", value: 7 },
        { label: "12:00", value: 6 },
        { label: "13:00", value: 8 },
        { label: "14:00", value: 8 },
        { label: "15:00", value: 9 },
        { label: "16:00", value: 10 },
      ],
    },
    runs: [
      {
        kind: "completed",
        id: "jin-route",
        label: "Jin route",
        sha: "e08152d",
        elapsed: "25s",
        relativeTime: "program gate 2026-08-27",
      },
      {
        kind: "completed",
        id: "ls-route",
        label: "Lorist–Schwenninger route",
        sha: "e08152d",
        elapsed: "10s",
        relativeTime: "program gate 2026-08-27",
      },
      {
        kind: "completed",
        id: "harp-route",
        label: "Harp route",
        sha: "e08152d",
        elapsed: "16s",
        relativeTime: "program gate 2026-08-27",
      },
    ],
  },
  insights: [
    {
      id: "crouzeix-route-closure",
      title: "Crouzeix route closure",
      evaluatorArtifact: "formalization",
      artifacts: [
        {
          kind: "research",
          summary:
            "Two source-faithful routes and one explicitly derived route are documented.",
          updatedAt: "2026-08-27",
          href: "#crouzeix-conjecture",
          evidence: "supported",
        },
        {
          kind: "spec",
          summary:
            "The three-route completion contract and claim ceiling are frozen.",
          updatedAt: "2026-08-27",
          href: "#crouzeix-conjecture",
          evidence: "supported",
        },
        {
          kind: "formalization",
          summary:
            "All three route validators report complete-local; external mathematical review is not claimed.",
          updatedAt: "2026-08-27",
          href: "#crouzeix-conjecture",
          evidence: "supported",
        },
      ],
    },
    {
      id: "autodiff-curriculum",
      title: "Autodiff curriculum",
      evaluatorArtifact: "spec",
      artifacts: [
        {
          kind: "research",
          summary:
            "JAX, Spivak, SICM, and FDG are registered as the source family.",
          updatedAt: "2026-08-18",
          href: "#mathematical-foundations",
          evidence: "supported",
        },
        {
          kind: "spec",
          summary:
            "A complete source-to-curriculum dependency graph is still required.",
          updatedAt: "2026-08-18",
          href: "#mathematical-foundations",
          evidence: "missing",
        },
        {
          kind: "formalization",
          summary:
            "Finite-coordinate gradient, JVP, VJP, HVP, and duality identities are available.",
          updatedAt: "2026-08-18",
          href: "#mathematical-foundations",
          evidence: "supported",
        },
      ],
    },
    {
      id: "agentic-production",
      title: "Agentic path to production",
      evaluatorArtifact: "research",
      artifacts: [
        {
          kind: "research",
          summary:
            "The RSI, harness, verifier, and agentic-engineering evidence graph is assembled.",
          updatedAt: "2026-09-03",
          href: "#knowledge",
          evidence: "supported",
        },
        {
          kind: "spec",
          summary:
            "Human authority, bounded execution, independent verification, and observability are separated.",
          updatedAt: "2026-09-03",
          href: "#knowledge",
          evidence: "supported",
        },
        {
          kind: "formalization",
          summary:
            "No theorem connects the engineering loop to recursive self-improvement guarantees.",
          updatedAt: "2026-09-03",
          href: "#knowledge",
          evidence: "missing",
        },
      ],
    },
  ],
  watchlist: [
    {
      id: "watch-crouzeix",
      workstreamId: "crouzeix-conjecture",
      workstream: "Crouzeix Conjecture",
      status: "active",
      blocker: null,
      nextAction: "Prepare external mathematical review packet",
      href: "#crouzeix-conjecture",
    },
    {
      id: "watch-autodiff",
      workstreamId: "autodiff-geometry",
      workstream: "Autodiff Geometry",
      status: "blocked",
      blocker: "Curriculum dependency map incomplete",
      nextAction: "Map JAX to geometry sources",
      href: "#mathematical-foundations",
    },
    {
      id: "watch-agentic",
      workstreamId: "agentic-research",
      workstream: "Agentic Research",
      status: "drafting",
      blocker: null,
      nextAction: "Select a bounded prototype",
      href: "#knowledge",
    },
    {
      id: "watch-nng4",
      workstreamId: "nng4-foundations",
      workstream: "NNG4 / Foundations",
      status: "complete-local",
      blocker: null,
      nextAction: "Retain as regression surface",
      href: "#mathematical-foundations",
    },
  ],
} satisfies WorkstreamsSnapshot;

export const workstreamsReference = parseWorkstreamsSnapshot(referenceInput);
