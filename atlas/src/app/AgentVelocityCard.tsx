import {
  Activity,
  CheckCircle2,
  Clock3,
  GitCommitHorizontal,
  PlayCircle,
  SendHorizontal,
} from "lucide-react";
import {
  useRef,
  useState,
  type FormEvent,
  type KeyboardEvent,
  type RefObject,
} from "react";

import type {
  AgentSnapshot,
  QueuedRun,
  RecentRun,
  VelocitySample,
} from "../content/workstreams";
import { queueLocalCommand, readLocalQueue } from "./workstreamsSession";

export function velocityPoints(
  samples: readonly VelocitySample[],
  width: number,
  height: number,
): string {
  const values = samples.map((sample) => sample.value);
  const minimum = Math.min(...values);
  const maximum = Math.max(...values);
  const span = Math.max(1, maximum - minimum);

  return samples
    .map((sample, index) => {
      const x =
        samples.length === 1 ? 0 : (index / (samples.length - 1)) * width;
      const y = height - ((sample.value - minimum) / span) * height;
      return `${x},${y}`;
    })
    .join(" ");
}

export type AgentVelocityCardProps = {
  agent: AgentSnapshot;
  recentRunsHeadingRef: RefObject<HTMLHeadingElement | null>;
};

function sessionStorageOrNull(): Storage | null {
  try {
    return window.sessionStorage;
  } catch {
    return null;
  }
}

function formatQueueError(error: unknown): string {
  if (!(error instanceof Error)) {
    return "Unable to queue command";
  }
  if (
    error.message === "command must not be empty" ||
    error.message === "command must be at most 240 characters"
  ) {
    return "Command must contain 1–240 characters";
  }
  return error.message;
}

function runKindLabel(run: RecentRun): string {
  switch (run.kind) {
    case "completed":
      return "Completed";
    case "failed":
      return "Failed";
    case "queued":
      return "Queued locally";
  }
}

function runKindIcon(run: RecentRun) {
  switch (run.kind) {
    case "completed":
      return CheckCircle2;
    case "failed":
      return Activity;
    case "queued":
      return Clock3;
  }
}

export function AgentVelocityCard({
  agent,
  recentRunsHeadingRef,
}: AgentVelocityCardProps) {
  const [command, setCommand] = useState("");
  const [statusMessage, setStatusMessage] = useState("");
  const [queuedRuns, setQueuedRuns] = useState<readonly QueuedRun[]>(() => {
    const storage = sessionStorageOrNull();
    return storage === null ? [] : readLocalQueue(storage);
  });
  const inputRef = useRef<HTMLInputElement>(null);
  const recentRuns = [...queuedRuns, ...agent.runs];
  const velocity = agent.velocity;
  const values = velocity.samples.map((sample) => sample.value);
  const minimum = Math.min(...values);
  const maximum = Math.max(...values);
  const polylinePoints = velocityPoints(velocity.samples, 264, 72);
  const titleId = `${agent.id}-velocity-title`;
  const descId = `${agent.id}-velocity-desc`;

  const submitCommand = () => {
    const storage = sessionStorageOrNull();
    if (storage === null) {
      setStatusMessage("Local queue unavailable");
      return;
    }

    try {
      const nextRuns = queueLocalCommand({
        storage,
        command,
        now: new Date().toISOString(),
      });
      setQueuedRuns(nextRuns);
      setCommand("");
      setStatusMessage(`Queued locally: ${nextRuns[0]?.command ?? ""}`);
      inputRef.current?.focus();
    } catch (error) {
      setStatusMessage(formatQueueError(error));
    }
  };

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    submitCommand();
  };

  const onCommandKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter" && event.shiftKey) {
      event.preventDefault();
      submitCommand();
    }
  };

  const focusRecentRuns = () => {
    recentRunsHeadingRef.current?.focus();
    setStatusMessage(`${recentRuns.length} recent runs`);
  };

  return (
    <section
      id="agent-velocity"
      aria-label="Agent velocity"
      title="Agent velocity"
      className="agent-velocity"
    >
      <header className="agent-velocity__header">
        <div>
          <p>Online at snapshot</p>
          <h2>{agent.name}</h2>
        </div>
        <button type="button" onClick={focusRecentRuns}>
          <PlayCircle aria-hidden="true" />
          <span>View Recent Runs</span>
        </button>
      </header>

      <p role="status" aria-live="polite">
        {statusMessage}
      </p>

      <div className="agent-velocity__summary">
        <p>{velocity.unit}</p>
        <p>{minimum}</p>
        <p>{maximum}</p>
        <p>{velocity.windowLabel}</p>
        <p>{velocity.deltaLabel}</p>
        <p>{velocity.summary}</p>
      </div>

      <svg
        role="img"
        aria-labelledby={titleId}
        aria-describedby={descId}
        className="agent-velocity__chart"
        viewBox="0 0 288 96"
      >
        <title id={titleId}>Agent velocity over 12 hours</title>
        <desc id={descId}>{velocity.summary}</desc>
        <polyline
          fill="none"
          points={polylinePoints}
          strokeWidth="2"
          transform="translate(12 12)"
        />
        {velocity.samples.map((sample, index) => {
          const x =
            velocity.samples.length === 1
              ? 0
              : (index / (velocity.samples.length - 1)) * 264;
          const span = Math.max(1, maximum - minimum);
          const y = 72 - ((sample.value - minimum) / span) * 72;
          const isCurrent = index === velocity.samples.length - 1;
          return (
            <circle
              key={sample.label}
              aria-label={`${sample.label}: ${sample.value} ${velocity.unit}`}
              className={isCurrent ? "agent-velocity__point--current" : "agent-velocity__point"}
              cx={x + 12}
              cy={y + 12}
              data-current={isCurrent ? "true" : "false"}
              fill={isCurrent ? "#06b6d4" : "#64748b"}
              r={isCurrent ? 5 : 4}
              role="img"
              stroke="#0f172a"
              strokeWidth="1"
              tabIndex={0}
            />
          );
        })}
      </svg>

      <details>
        <summary>View 12-hour samples</summary>
        <table>
          <thead>
            <tr>
              <th scope="col">Time</th>
              <th scope="col">Value</th>
            </tr>
          </thead>
          <tbody>
            {velocity.samples.map((sample) => (
              <tr key={sample.label}>
                <td>{sample.label}</td>
                <td>{sample.value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </details>

      <form onSubmit={onSubmit}>
        <label htmlFor="workstreams-local-command">Local command</label>
        <input
          id="workstreams-local-command"
          maxLength={240}
          onChange={(event) => setCommand(event.target.value)}
          onKeyDown={onCommandKeyDown}
          ref={inputRef}
          type="text"
          value={command}
        />
        <button type="submit">
          <SendHorizontal aria-hidden="true" />
          <span>Queue command</span>
        </button>
      </form>

      <div className="agent-velocity__runs">
        <h3 ref={recentRunsHeadingRef} tabIndex={-1}>
          Recent Runs
        </h3>
        <ol aria-label="Recent runs list">
          {recentRuns.map((run) => {
            const Icon = runKindIcon(run);
            if (run.kind === "queued") {
              return (
                <li key={run.id}>
                  <p>
                    <Icon aria-hidden="true" />
                    <span>Queued locally</span>
                  </p>
                  <p>{run.command}</p>
                  <p>{run.queuedAt}</p>
                </li>
              );
            }

            return (
              <li key={run.id}>
                <p>
                  <Icon aria-hidden="true" />
                  <span>{runKindLabel(run)}</span>
                </p>
                <p>{run.label}</p>
                <p>
                  <GitCommitHorizontal aria-hidden="true" />
                  <span>{run.sha.slice(0, 7)}</span>
                </p>
                <p>{run.elapsed}</p>
                <p>{run.relativeTime}</p>
              </li>
            );
          })}
        </ol>
      </div>
    </section>
  );
}
