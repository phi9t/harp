import {
  Activity,
  BadgeCheck,
  Check,
  Circle,
  CircleDot,
  OctagonX,
  Pencil,
  ListTodo,
  type LucideIcon,
} from "lucide-react";

import type {
  StageStatus,
  WorkstreamStatus,
} from "../content/workstreams";

export const stageStatusPresentation: Record<
  StageStatus,
  { label: "Complete" | "Active" | "Queued" | "Blocked"; Icon: LucideIcon }
> = {
  complete: { label: "Complete", Icon: Check },
  active: { label: "Active", Icon: CircleDot },
  queued: { label: "Queued", Icon: Circle },
  blocked: { label: "Blocked", Icon: OctagonX },
};

export const workstreamStatusPresentation: Record<
  WorkstreamStatus,
  {
    label:
      | "Active"
      | "Blocked"
      | "Planning"
      | "Drafting"
      | "Complete locally";
    Icon: LucideIcon;
  }
> = {
  active: { label: "Active", Icon: Activity },
  blocked: { label: "Blocked", Icon: OctagonX },
  planning: { label: "Planning", Icon: ListTodo },
  drafting: { label: "Drafting", Icon: Pencil },
  "complete-local": { label: "Complete locally", Icon: BadgeCheck },
};

export function workstreamStageDetailId(
  workstreamId: string,
  stageIndex: number,
): string {
  return `${workstreamId}-stage-${stageIndex}-detail`;
}
