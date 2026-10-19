import MathematicalFoundations.CauchySchwarz.Accepted
import MathematicalFoundations.TextbookCauchySchwarz

open Lean Meta TextbookBench

theorem referenceAlias : CauchySchwarzTarget := cauchySchwarzReference
theorem referenceAliasTwice : CauchySchwarzTarget := referenceAlias

run_elab do
  let .ok policy ← Audit.fromFoundations FoundationPolicy.foundations
      FoundationPolicy.owned FoundationPolicy.forbidden
    | throwError "could not establish reviewed policy"
  for candidate in #[``cauchySchwarzReference, ``referenceAlias, ``referenceAliasTwice] do
    let result ← Audit.audit ``CauchySchwarzTarget candidate policy
    unless result matches .error (.forbiddenDependency ``cauchySchwarzReference) do
      throwError m!"reference shortcut was not rejected: {candidate}: {repr result}"
  let crossRoute ← Audit.audit ``CauchySchwarzTarget ``cauchySchwarzLagrange
    { policy with forbidden := policy.forbidden.push ``cauchySchwarzLagrange }
  unless crossRoute matches .error (.forbiddenDependency _) do
    throwError "cross-route reuse was not rejected"
  logInfo "policy controls passed: direct reference, alias, transitive alias, cross-route"
