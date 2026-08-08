import { useState } from "react";

type GateId = "Outcome" | "Integrity" | "Authority";

const gateIds: readonly GateId[] = ["Outcome", "Integrity", "Authority"];

export function LineageLoom() {
  const [closedGates, setClosedGates] = useState<ReadonlySet<GateId>>(
    () => new Set(),
  );
  const promotionOpen = closedGates.size === 0;

  const toggleGate = (gateId: GateId) => {
    setClosedGates((current) => {
      const next = new Set(current);
      if (next.has(gateId)) {
        next.delete(gateId);
      } else {
        next.add(gateId);
      }
      return next;
    });
  };

  return (
    <section className="panel loom-panel" aria-label="Lineage promotion gates">
      <div className="loom-stage">
        <article className="loom-node loom-node--parent">
          <span className="node-kicker">accepted parent</span>
          <strong>Cₜ</strong>
        </article>
        <div className="loom-bridge" aria-hidden="true">
          <span />
          <span />
          <span />
        </div>
        <div className="loom-shutters" role="group" aria-label="Promotion gates">
          {gateIds.map((gateId) => {
            const closed = closedGates.has(gateId);
            return (
              <button
                className={closed ? "shutter closed" : "shutter"}
                type="button"
                key={gateId}
                aria-pressed={closed}
                aria-label={`${closed ? "Open" : "Close"} ${gateId} gate`}
                onClick={() => toggleGate(gateId)}
              >
                <span className="shutter-blade" aria-hidden="true" />
                <span className="shutter-label">{gateId}</span>
                <small>{closed ? "closed" : "open"}</small>
              </button>
            );
          })}
        </div>
        <div className="loom-bridge loom-bridge--right" aria-hidden="true">
          <span />
          <span />
          <span />
        </div>
        <article
          className={
            "loom-node loom-node--child "
            + (promotionOpen ? "promotable" : "held")
          }
        >
          <span className="node-kicker">candidate child</span>
          <strong>Cₜ₊₁</strong>
        </article>
      </div>
      <div
        className="loom-status"
        data-promotion={promotionOpen ? "open" : "blocked"}
        role="status"
        aria-label="Promotion status"
      >
        <strong>
          {promotionOpen ? "Promotion path open" : "Promotion blocked"}
        </strong>
        {!promotionOpen ? (
          <span>{Array.from(closedGates).join(", ")}</span>
        ) : null}
      </div>
    </section>
  );
}
