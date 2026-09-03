import { useRef, useState } from "react";

import type { WorkstreamSnapshot } from "../content/workstreams";
import {
  stageStatusPresentation,
  workstreamStageDetailId,
  workstreamStatusPresentation,
} from "./workstreamsPresentation";

export function WorkstreamCard({
  workstream,
}: {
  workstream: WorkstreamSnapshot;
}) {
  const [pinnedStageIndex, setPinnedStageIndex] = useState<number | null>(null);
  const [hoveredStageIndex, setHoveredStageIndex] = useState<number | null>(null);
  const [focusedStageIndex, setFocusedStageIndex] = useState<number | null>(null);
  const stageButtonRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const titleId = `${workstream.id}-title`;
  const status = workstreamStatusPresentation[workstream.status];

  return (
    <article
      className="workstream-card"
      aria-labelledby={titleId}
    >
      <header className="workstream-card__header">
        <h3 id={titleId}>{workstream.title}</h3>
        <p className="workstream-card__status">
          <status.Icon aria-hidden="true" />
          <span>{status.label}</span>
        </p>
        <p className="workstream-card__last-run">{workstream.lastRun}</p>
        <p className="workstream-card__progress">{workstream.progressLabel}</p>
      </header>
      <ol className="workstream-card__lifecycle">
        {workstream.stages.map((stage, index) => {
          const presentation = stageStatusPresentation[stage.status];
          const detailId = workstreamStageDetailId(workstream.id, index);
          const accessibleName = `${stage.name}: ${presentation.label}`;
          const isVisible =
            pinnedStageIndex === index ||
            hoveredStageIndex === index ||
            focusedStageIndex === index;
          return (
            <li key={stage.name} className="workstream-card__stage">
              <button
                type="button"
                className="workstream-card__stage-summary"
                aria-label={accessibleName}
                aria-expanded={isVisible}
                aria-controls={detailId}
                aria-describedby={detailId}
                ref={(element) => {
                  stageButtonRefs.current[index] = element;
                }}
                onClick={() =>
                  setPinnedStageIndex((currentIndex) =>
                    currentIndex === index ? null : index,
                  )
                }
                onMouseEnter={() => setHoveredStageIndex(index)}
                onMouseLeave={() =>
                  setHoveredStageIndex((currentIndex) =>
                    currentIndex === index ? null : currentIndex,
                  )
                }
                onFocus={() => setFocusedStageIndex(index)}
                onBlur={() =>
                  setFocusedStageIndex((currentIndex) =>
                    currentIndex === index ? null : currentIndex,
                  )
                }
                onKeyDown={(event) => {
                  if (event.key !== "Escape") {
                    return;
                  }
                  event.preventDefault();
                  setPinnedStageIndex(null);
                  setHoveredStageIndex(null);
                  setFocusedStageIndex(null);
                  stageButtonRefs.current[index]?.focus();
                }}
              >
                <presentation.Icon aria-hidden="true" />
                <span>{stage.name}</span>
                <span>{presentation.label}</span>
              </button>
              <p
                id={detailId}
                role="tooltip"
                hidden={!isVisible}
              >
                {stage.detail}
              </p>
            </li>
          );
        })}
      </ol>
      <details className="workstream-card__details">
        <summary>{`All lifecycle details for ${workstream.title}`}</summary>
        <div>
          {workstream.stages.map((stage) => {
            const presentation = stageStatusPresentation[stage.status];
            return (
              <p key={stage.name}>
                {`${stage.name}: ${presentation.label}. ${stage.detail}`}
              </p>
            );
          })}
        </div>
      </details>
      <p className="workstream-card__evidence">
        {`Evidence: ${workstream.blocker ?? workstream.nextAction}`}
      </p>
    </article>
  );
}
