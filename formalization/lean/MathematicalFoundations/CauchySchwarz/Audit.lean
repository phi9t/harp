import Lean

/-! A dependency-policy check for trusted local teaching fixtures.
This is not a sandbox or an acceptance service for hostile elaborator code.
The caller owns the target, policy, and pinned imported environment.
-/

namespace TextbookBench.Audit

open Lean Meta

structure Policy where
  /-- Explicitly reviewed imported declarations. Unknown names fail closed. -/
  allowed : Array Name
  forbidden : Array Name := #[]
  allowedAxioms : Array Name := #[``propext, ``Classical.choice, ``Quot.sound]
  maxDeclarations : Nat := 50000
  deriving Inhabited

inductive Failure where
  | missingDeclaration (name : Name)
  | notTheorem (name : Name)
  | typeMismatch (name : Name)
  | disallowedAxiom (name : Name)
  | forbiddenDependency (name : Name)
  | unreviewedDependency (name : Name)
  | declarationLimit
  deriving Repr, BEq

structure Report where
  dependencies : Array Name
  axioms : Array Name
  deriving Repr

/-- Traverse bodies as well as types, including renamed helper theorems.
The imported allowlist never short-circuits recursive inspection. -/
private def closure (env : Environment) (roots : Array Name) (policy : Policy) :
    Except Failure (Array Name) := Id.run do
  let allowed := policy.allowed.foldl (fun s n => s.insert n) ({} : NameSet)
  let forbidden := policy.forbidden.foldl (fun s n => s.insert n) ({} : NameSet)
  let mut seen : NameSet := {}
  let mut scheduled : NameSet := roots.foldl (fun s n => s.insert n) {}
  let mut pending := scheduled.toList
  for _ in [:policy.maxDeclarations] do
    match pending with
    | [] => return .ok (seen.toArray.qsort Name.lt)
    | name :: rest =>
      pending := rest
      if seen.contains name then continue
      seen := seen.insert name
      if forbidden.contains name then return .error (.forbiddenDependency name)
      let some info := env.find? name
        | return .error (.missingDeclaration name)
      if info matches .axiomInfo _ then
        if !policy.allowedAxioms.contains name then
          return .error (.disallowedAxiom name)
      if env.isImportedConst name && !allowed.contains name then
        return .error (.unreviewedDependency name)
      for used in info.getUsedConstantsAsSet do
        if !scheduled.contains used then
          scheduled := scheduled.insert used
          pending := used :: pending
  if pending.isEmpty then return .ok (seen.toArray.qsort Name.lt)
  else return .error .declarationLimit

/-- Check the exact trusted proposition, declaration kind, transitive axioms,
and a closed imported dependency policy. This does not certify provenance. -/
def audit (target candidate : Name) (policy : Policy) :
    MetaM (Except Failure Report) := do
  let env ← getEnv
  let some info := env.find? candidate
    | return .error (.missingDeclaration candidate)
  unless info matches .thmInfo _ do
    return .error (.notTheorem candidate)
  if !info.levelParams.isEmpty then return .error (.typeMismatch candidate)
  let some targetInfo := env.find? target
    | return .error (.missingDeclaration target)
  if !targetInfo.levelParams.isEmpty then return .error (.typeMismatch target)
  unless ← isDefEq info.type (mkConst target) do
    return .error (.typeMismatch candidate)
  let axioms ← collectAxioms candidate
  for name in axioms do
    unless policy.allowedAxioms.contains name do
      return .error (.disallowedAxiom name)
  match closure env #[candidate] policy with
  | .error failure => return .error failure
  | .ok dependencies => return .ok { dependencies, axioms }

/-- Authorize only the transitive closure of an explicitly reviewed foundation
boundary in the pinned environment, plus named owned declarations. Never infer
the foundation boundary from a candidate during acceptance. Even foundation
bodies are checked for forbidden dependencies and unexpected axioms. -/
def fromFoundations (foundations owned forbidden : Array Name) :
    MetaM (Except Failure Policy) := do
  let env ← getEnv
  let all := env.constants.fold (init := #[]) (fun acc name _ => acc.push name)
  let base : Policy := { allowed := all, forbidden }
  for root in foundations do
    for name in ← collectAxioms root do
      unless base.allowedAxioms.contains name do
        return .error (.disallowedAxiom name)
  match closure env foundations base with
  | .error failure => return .error failure
  | .ok dependencies =>
    return .ok { base with allowed := dependencies ++ owned }

end TextbookBench.Audit
