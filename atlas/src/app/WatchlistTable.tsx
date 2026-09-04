import type {
  DashboardHref,
  WatchlistRow,
  WorkstreamSnapshot,
} from "../content/workstreams";
import { workstreamStatusPresentation } from "./workstreamsPresentation";

export type WatchlistTableProps = {
  rows: readonly WatchlistRow[];
  workstreams: readonly WorkstreamSnapshot[];
  onNavigate: (href: DashboardHref) => void;
  empty: boolean;
};

function emptyRowLabel(rows: readonly WatchlistRow[], empty: boolean): string {
  if (empty) {
    return "No workstreams match this search.";
  }
  if (rows.length === 0) {
    return "No matching workstreams";
  }
  return "";
}

function workstreamTitleById(
  workstreams: readonly WorkstreamSnapshot[],
  workstreamId: string,
): string {
  const workstream = workstreams.find((candidate) => candidate.id === workstreamId);
  return workstream?.title ?? workstreamId;
}

export function WatchlistTable({ rows, workstreams, onNavigate, empty }: WatchlistTableProps) {
  const emptyLabel = emptyRowLabel(rows, empty);
  return (
    <section aria-label="Workstream watchlist">
      <p className="watchlist-overflow-cue">Scroll for more columns</p>
      <div
        className="watchlist-scroll"
        role="region"
        aria-label="Workstream watchlist, scroll for more columns"
        tabIndex={0}
      >
        <table aria-label="Workstream watchlist">
          <thead>
            <tr>
              <th scope="col">Workstream</th>
              <th scope="col">Status</th>
              <th scope="col">Blocker</th>
              <th scope="col">Next action</th>
            </tr>
          </thead>
          <tbody>
            {emptyLabel !== "" ? (
              <tr>
                <td colSpan={4}>{emptyLabel}</td>
              </tr>
            ) : (
              rows.map((row) => (
                <tr key={row.id}>
                  <th scope="row">{workstreamTitleById(workstreams, row.workstreamId)}</th>
                  <td>
                    <span className="status-chip" data-state={row.status}>
                      {workstreamStatusPresentation[row.status].label}
                    </span>
                  </td>
                  <td>{row.blocker ?? "None recorded"}</td>
                  <td>
                    <button type="button" onClick={() => onNavigate(row.href)}>
                      {row.nextAction}
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
