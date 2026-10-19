import MathematicalFoundations.CauchySchwarz.Controls
import MathematicalFoundations.CauchySchwarz.Audit
import MathematicalFoundations.CauchySchwarz.FoundationPolicy

open Lean Meta

/-! Trusted local acceptance checks for the four educational routes.
Compiled acceptance assumes the pinned, trusted import environment. It is
not a hermetic receipt or an entry point for hostile learner submissions. -/

run_elab do
  let .ok policy ← TextbookBench.Audit.fromFoundations
      TextbookBench.FoundationPolicy.foundations TextbookBench.FoundationPolicy.owned
      TextbookBench.FoundationPolicy.forbidden
    | throwError "Cauchy-Schwarz foundation policy could not be established"
  for candidate in TextbookBench.FoundationPolicy.routes do
    let otherRoutes := TextbookBench.FoundationPolicy.routes.filter (· != candidate)
    let result ← TextbookBench.Audit.audit ``TextbookBench.CauchySchwarzTarget candidate
      { policy with forbidden := policy.forbidden ++ otherRoutes }
    match result with
    | .error failure => throwError m!"{candidate}: {repr failure}"
    | .ok report =>
      logInfo m!"{candidate}: dependency policy passed; {report.dependencies.size} constants; axioms {report.axioms}"
