import { describe, expect, it } from "vitest";

import type { QueuedRun } from "../content/workstreams";
import {
  QUEUE_KEY,
  queueLocalCommand,
  readLocalQueue,
  SCHEMA_VERSION,
} from "./workstreamsSession";

class MemoryStorage implements Storage {
  private readonly entries = new Map<string, string>();

  get length(): number {
    return this.entries.size;
  }

  clear(): void {
    this.entries.clear();
  }

  getItem(key: string): string | null {
    return this.entries.get(key) ?? null;
  }

  key(index: number): string | null {
    const keys = [...this.entries.keys()];
    return keys[index] ?? null;
  }

  removeItem(key: string): void {
    this.entries.delete(key);
  }

  setItem(key: string, value: string): void {
    this.entries.set(key, value);
  }
}

class ThrowingGetStorage extends MemoryStorage {
  override getItem(_key: string): string | null {
    throw new Error("getItem failed");
  }
}

class ThrowingSetStorage extends MemoryStorage {
  override setItem(_key: string, _value: string): void {
    throw new Error("setItem failed");
  }
}

function queueEntry(
  overrides: Partial<QueuedRun> = {},
): QueuedRun {
  return {
    kind: "queued",
    id: "local-alpha-1",
    command: "vitest run src/app/workstreamsSession.test.ts",
    queuedAt: "2026-09-03T16:00:00.000Z",
    ...overrides,
  };
}

function writeQueue(storage: Storage, runs: readonly QueuedRun[], schemaVersion = SCHEMA_VERSION): void {
  storage.setItem(
    QUEUE_KEY,
    JSON.stringify({
      schemaVersion,
      runs,
    }),
  );
}

describe("workstreamsSession", () => {
  it("returns an empty queue for missing storage state", () => {
    const storage = new MemoryStorage();

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("rejects whitespace-only queued commands", () => {
    const storage = new MemoryStorage();

    expect(() =>
      queueLocalCommand({
        storage,
        command: "   \n\t  ",
        now: "2026-09-03T16:00:00.000Z",
      }),
    ).toThrow(/command/i);
  });

  it("rejects queued commands longer than 240 unicode characters after trim", () => {
    const storage = new MemoryStorage();
    const command = `${"a".repeat(239)}é`;

    expect(command.length).toBe(240);
    expect(`${command}é`.length).toBe(241);
    expect(() =>
      queueLocalCommand({
        storage,
        command: `${command}é`,
        now: "2026-09-03T16:00:00.000Z",
      }),
    ).toThrow(/240/i);
  });

  it("returns an empty queue for malformed JSON", () => {
    const storage = new MemoryStorage();
    storage.setItem(QUEUE_KEY, "{");

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("returns an empty queue for a non-object root", () => {
    const storage = new MemoryStorage();
    storage.setItem(QUEUE_KEY, JSON.stringify([]));

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("returns an empty queue when runs is missing", () => {
    const storage = new MemoryStorage();
    storage.setItem(
      QUEUE_KEY,
      JSON.stringify({
        schemaVersion: SCHEMA_VERSION,
      }),
    );

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("returns an empty queue when runs is not an array", () => {
    const storage = new MemoryStorage();
    storage.setItem(
      QUEUE_KEY,
      JSON.stringify({
        schemaVersion: SCHEMA_VERSION,
        runs: { id: "local-alpha-1" },
      }),
    );

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("returns an empty queue for an unknown schema", () => {
    const storage = new MemoryStorage();
    writeQueue(storage, [queueEntry()], "harp-workstreams-queue/v2");

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("returns an empty queue for invalid or extra entry fields", () => {
    const storage = new MemoryStorage();
    storage.setItem(
      QUEUE_KEY,
      JSON.stringify({
        schemaVersion: SCHEMA_VERSION,
        runs: [
          {
            kind: "queued",
            id: "local-alpha-1",
            command: "vitest run src/app/workstreamsSession.test.ts",
            queuedAt: "2026-09-03T16:00:00.000Z",
            extra: true,
          },
        ],
      }),
    );

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("returns an empty queue for a bad id", () => {
    const storage = new MemoryStorage();
    writeQueue(storage, [queueEntry({ id: "Local Bad" })]);

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("returns an empty queue for a non-canonical timestamp", () => {
    const storage = new MemoryStorage();
    writeQueue(storage, [queueEntry({ queuedAt: "2026-09-03T16:00:00Z" })]);

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("truncates reads to the newest eight entries", () => {
    const storage = new MemoryStorage();
    const runs = Array.from({ length: 10 }, (_, index) =>
      queueEntry({
        id: `local-run-${index}`,
        command: `echo ${index}`,
        queuedAt: `2026-09-03T16:00:0${index}.000Z`,
      }),
    );
    writeQueue(storage, runs);

    expect(readLocalQueue(storage)).toEqual(runs.slice(0, 8));
  });

  it("roundtrips a queued command through storage", () => {
    const storage = new MemoryStorage();

    const queued = queueLocalCommand({
      storage,
      command: "  vitest run src/app/workstreamsSession.test.ts  ",
      now: "2026-09-03T16:00:00.000Z",
      createId: () => "local-session-red",
    });

    expect(queued).toEqual([
      {
        kind: "queued",
        id: "local-session-red",
        command: "vitest run src/app/workstreamsSession.test.ts",
        queuedAt: "2026-09-03T16:00:00.000Z",
      },
    ]);
    expect(readLocalQueue(storage)).toEqual(queued);
  });

  it("returns an empty queue when getItem throws", () => {
    const storage = new ThrowingGetStorage();

    expect(readLocalQueue(storage)).toEqual([]);
  });

  it("propagates setItem failures", () => {
    const storage = new ThrowingSetStorage();

    expect(() =>
      queueLocalCommand({
        storage,
        command: "vitest run src/app/workstreamsSession.test.ts",
        now: "2026-09-03T16:00:00.000Z",
      }),
    ).toThrow(/setItem failed/);
  });

  it("uses the provided createId and prepends new entries while capping at eight", () => {
    const storage = new MemoryStorage();
    const existing = Array.from({ length: 8 }, (_, index) =>
      queueEntry({
        id: `local-existing-${index}`,
        command: `echo existing-${index}`,
        queuedAt: `2026-09-03T15:00:0${index}.000Z`,
      }),
    );
    writeQueue(storage, existing);

    const queued = queueLocalCommand({
      storage,
      command: "echo newest",
      now: "2026-09-03T16:00:00.000Z",
      createId: () => "local-deterministic-id",
    });

    expect(queued).toHaveLength(8);
    expect(queued[0]).toEqual({
      kind: "queued",
      id: "local-deterministic-id",
      command: "echo newest",
      queuedAt: "2026-09-03T16:00:00.000Z",
    });
    expect(queued[7]).toEqual(existing[6]);
    expect(queued.map((run) => run.id)).not.toContain("local-existing-7");
  });
});
