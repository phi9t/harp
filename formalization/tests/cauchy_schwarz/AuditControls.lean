import MathematicalFoundations.CauchySchwarz.Audit
import AuditFixture

open Lean Meta TextbookBench.Audit

def auditTarget : Prop := True
theorem auditGood : auditTarget := True.intro
theorem auditWrong : False → True := fun _ => True.intro
axiom auditUntrusted : True
theorem auditHidden : auditTarget := auditUntrusted
theorem auditAlias : auditTarget := auditGood
set_option linter.defProp false in
def auditNotTheorem : True := True.intro

run_elab do
  let policy : Policy := { allowed := #[``True, ``True.intro] }
  let good ← audit ``auditTarget ``auditGood policy
  unless good.isOk do throwError m!"valid theorem rejected: {repr good}"
  let wrong ← audit ``auditTarget ``auditWrong policy
  unless wrong matches .error (.typeMismatch _) do
    throwError m!"extra hypothesis accepted: {repr wrong}"
  let hidden ← audit ``auditTarget ``auditHidden policy
  unless hidden matches .error (.disallowedAxiom ``auditUntrusted) do
    throwError m!"hidden axiom accepted: {repr hidden}"
  let alias ← audit ``auditTarget ``auditAlias
    { policy with forbidden := #[``auditGood] }
  unless alias matches .error (.forbiddenDependency ``auditGood) do
    throwError m!"transitive forbidden theorem accepted: {repr alias}"
  let unknown ← audit ``auditTarget ``auditGood { allowed := #[] }
  unless unknown matches .error (.unreviewedDependency _) do
    throwError m!"unreviewed imported declaration accepted: {repr unknown}"
  let definition ← audit ``auditTarget ``auditNotTheorem policy
  unless definition matches .error (.notTheorem _) do
    throwError m!"definition accepted as theorem: {repr definition}"
  let missing ← audit ``auditTarget `missingCandidate policy
  unless missing matches .error (.missingDeclaration _) do
    throwError m!"missing declaration accepted: {repr missing}"
  let exhausted ← audit ``auditTarget ``auditGood { policy with maxDeclarations := 0 }
  unless exhausted matches .error .declarationLimit do
    throwError m!"budget exhaustion accepted: {repr exhausted}"
  let exactBudget ← audit ``auditTarget ``auditGood { policy with maxDeclarations := 4 }
  unless exactBudget.isOk do
    throwError m!"exact declaration budget rejected: {repr exactBudget}"
  let importedAliasResult ← audit ``auditTarget ``importedAlias
    { policy with allowed := policy.allowed.push ``importedAlias
                  forbidden := #[``importedForbidden] }
  unless importedAliasResult matches .error (.forbiddenDependency ``importedForbidden) do
    throwError m!"imported alias bypassed dependency traversal: {repr importedAliasResult}"
  let importedAxiomResult ← audit ``auditTarget ``importedHiddenAxiom policy
  unless importedAxiomResult matches .error (.disallowedAxiom ``importedUntrusted) do
    throwError m!"imported axiom bypassed axiom traversal: {repr importedAxiomResult}"
  let built ← fromFoundations #[``True] #[] #[]
  let .ok foundationPolicy := built | throwError m!"foundation policy construction failed"
  unless (← audit ``auditTarget ``auditGood foundationPolicy).isOk do
    throwError m!"reviewed foundation closure rejected"
  let badBoundary ← fromFoundations #[``importedAlias] #[] #[``importedForbidden]
  unless badBoundary matches .error (.forbiddenDependency ``importedForbidden) do
    throwError m!"forbidden theorem inside a foundation boundary accepted"
  let axiomBoundary ← fromFoundations #[``importedHiddenAxiom] #[] #[]
  unless axiomBoundary matches .error (.disallowedAxiom ``importedUntrusted) do
    throwError "axiom hidden in foundation boundary accepted"
  let admitted ← audit ``auditTarget `importedSorry policy
  unless admitted matches .error (.disallowedAxiom ``sorryAx) do
    throwError m!"admitted proof was not rejected through axiom inspection: {repr admitted}"
  logInfo "audit controls passed: positive, type, axiom, transitive, unknown, kind, budgets, foundation boundaries"
