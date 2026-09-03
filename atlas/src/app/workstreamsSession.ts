import type { QueuedRun } from "../content/workstreams";

export const QUEUE_KEY = "harp.workstreams.queue.v1";
export const SCHEMA_VERSION = "harp-workstreams-queue/v1";

const MAX_QUEUE_RUNS = 8;
const MAX_COMMAND_CHARACTERS = 240;
const ID_RE = /^local-[a-z0-9]+(?:-[a-z0-9]+)*$/;

type QueueDocument = {
  schemaVersion: typeof SCHEMA_VERSION;
  runs: readonly QueuedRun[];
};

export function readLocalQueue(storage: Storage): readonly QueuedRun[] {
  let raw: string | null;
  try {
    raw = storage.getItem(QUEUE_KEY);
  } catch {
    return [];
  }
  if (raw === null) {
    return [];
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return [];
  }

  const document = parseQueueDocument(parsed);
  return document === null ? [] : document.runs.slice(0, MAX_QUEUE_RUNS);
}

export function queueLocalCommand({
  storage,
  command,
  now,
  createId = () => crypto.randomUUID(),
}: {
  storage: Storage;
  command: string;
  now: string;
  createId?: () => string;
}): readonly QueuedRun[] {
  const trimmedCommand = validateCommand(command);
  const queuedAt = validateIsoDatetime(now);
  const id = validateId(normalizeGeneratedId(createId()));
  const queuedRun: QueuedRun = {
    kind: "queued",
    id,
    command: trimmedCommand,
    queuedAt,
  };
  const nextRuns: readonly QueuedRun[] = [queuedRun, ...readLocalQueue(storage)].slice(
    0,
    MAX_QUEUE_RUNS,
  );
  const payload: QueueDocument = {
    schemaVersion: SCHEMA_VERSION,
    runs: nextRuns,
  };

  storage.setItem(QUEUE_KEY, JSON.stringify(payload));
  return nextRuns;
}

function parseQueueDocument(value: unknown): QueueDocument | null {
  const record = asRecord(value);
  if (record === null) {
    return null;
  }
  if (!hasOnlyKeys(record, ["schemaVersion", "runs"])) {
    return null;
  }
  if (record.schemaVersion !== SCHEMA_VERSION || !Array.isArray(record.runs)) {
    return null;
  }

  const runs: QueuedRun[] = [];
  for (const entry of record.runs) {
    const run = parseQueuedRun(entry);
    if (run === null) {
      return null;
    }
    runs.push(run);
  }
  return { schemaVersion: SCHEMA_VERSION, runs };
}

function parseQueuedRun(value: unknown): QueuedRun | null {
  const record = asRecord(value);
  if (record === null) {
    return null;
  }
  if (!hasOnlyKeys(record, ["kind", "id", "command", "queuedAt"])) {
    return null;
  }
  if (record.kind !== "queued") {
    return null;
  }
  if (typeof record.id !== "string" || typeof record.command !== "string" || typeof record.queuedAt !== "string") {
    return null;
  }

  try {
    return {
      kind: "queued",
      id: validateId(record.id),
      command: validateCommand(record.command),
      queuedAt: validateIsoDatetime(record.queuedAt),
    };
  } catch {
    return null;
  }
}

function validateCommand(command: string): string {
  if (typeof command !== "string") {
    throw new Error("command must be a string");
  }
  const trimmed = command.trim();
  if (trimmed.length === 0) {
    throw new Error("command must not be empty");
  }
  if (Array.from(trimmed).length > MAX_COMMAND_CHARACTERS) {
    throw new Error(`command must be at most ${MAX_COMMAND_CHARACTERS} characters`);
  }
  return trimmed;
}

function validateIsoDatetime(value: string): string {
  if (typeof value !== "string" || value.trim() !== value) {
    throw new Error("queuedAt must be an exact ISO datetime");
  }
  const date = new Date(value);
  if (!Number.isFinite(date.getTime()) || date.toISOString() !== value) {
    throw new Error("queuedAt must be an exact ISO datetime");
  }
  return value;
}

function normalizeGeneratedId(value: string): string {
  return value.startsWith("local-") ? value : `local-${value}`;
}

function validateId(value: string): string {
  if (typeof value !== "string" || !ID_RE.test(value)) {
    throw new Error("id must be a local slug-safe identifier");
  }
  return value;
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return null;
  }
  return value as Record<string, unknown>;
}

function hasOnlyKeys(
  record: Record<string, unknown>,
  expectedKeys: readonly string[],
): boolean {
  const keys = Object.keys(record);
  return (
    keys.length === expectedKeys.length &&
    expectedKeys.every((key) => Object.prototype.hasOwnProperty.call(record, key))
  );
}
