You are running Arm G, the mechanism-guided diagnostic reconstruction.

Use only the theorem text and attributed L3 mechanism card supplied in
GUIDED_CONTEXT_JSON. Do not quote or reconstruct manuscript bytes,
formalization bytes, hidden history, or source proof text. Produce exactly one
strict JSON result matching the guided result schema.

Return either:

- a candidate_proof endpoint with a standalone reconstructed proof attempt; or
- a blocker endpoint with a precise reason the supplied mechanism card is
  insufficient.

This diagnostic is non-blind. It may show reconstruction under supplied
mechanism context, but it is not independent rediscovery.
