import CrouzeixTextbook.Correspondence
import Mathlib.Lean.CoreM
import Mathlib.Util.AssertNoSorry

/-!
Maintained compiler-environment exporter for the generated textbook correspondence surface.
The declaration roster is discovered from the imported environment; it is not duplicated here.
-/

open Lean Meta

namespace CrouzeixTextbook.ExportReceipt

def maxDeclarations : Nat := 4096
def maxRelations : Nat := 4096
def maxStringBytes : Nat := 64 * 1024
def maxReceiptBytes : Nat := 4 * 1024 * 1024

def isTextbookName (name : Name) : Bool :=
  name.toString.startsWith "CrouzeixTextbook."

def isReceiptMarkerName (name : Name) : Bool :=
  name.toString.startsWith "CrouzeixTextbook.ReceiptMarker."

def isMaintainedName (name : Name) : Bool :=
  ["CrouzeixTextbook.", "CrouzeixConjecture.", "Crouzeix.Jin.", "CrouzeixJin.",
    "Crouzeix.LoristSchwenninger.", "CrouzeixLoristSchwenninger.", "Crouzeix.Harp.",
    "CrouzeixHarp."].any fun namespacePrefix => name.toString.startsWith namespacePrefix

partial def collectConstants (expr : Expr) (found : NameHashSet := {}) : NameHashSet :=
  match expr with
  | .const name _ => found.insert name
  | .app function argument => collectConstants argument (collectConstants function found)
  | .lam _ type body _ => collectConstants body (collectConstants type found)
  | .forallE _ type body _ => collectConstants body (collectConstants type found)
  | .letE _ type value body _ =>
      collectConstants body (collectConstants value (collectConstants type found))
  | .mdata _ body | .proj _ _ body => collectConstants body found
  | _ => found

def directConstants (info : ConstantInfo) : Array Name :=
  let fromType := collectConstants info.type
  let all := info.value? (allowOpaque := true) |>.map (collectConstants · fromType) |>.getD fromType
  all.toArray.filter isMaintainedName |>.qsort (fun left right => left.toString < right.toString)

def bodyConstants (info : ConstantInfo) : Array Name :=
  let found := info.value? (allowOpaque := true) |>.map collectConstants |>.getD {}
  found.toArray.filter isMaintainedName |>.qsort (fun left right => left.toString < right.toString)

def exactEtaTarget? (value : Expr) : Option Name :=
  let (binderCount, body) := peelLambdas value
  let (head, arguments) := peelApplications body
  match head.constName? with
  | none => none
  | some target =>
      if arguments.size != binderCount then none
      else if (Array.range binderCount).all fun index =>
          match arguments[index]! with
          | .bvar binder => binder == binderCount - index - 1
          | _ => false
        then some target
        else none
where
  peelLambdas : Expr → Nat × Expr
  | .lam _ _ body _ =>
      let (count, result) := peelLambdas body
      (count + 1, result)
  | expression => (0, expression)
  peelApplications : Expr → Expr × Array Expr
  | .app function argument =>
      let (head, arguments) := peelApplications function
      (head, arguments.push argument)
  | expression => (expression, #[])

def directAliasTarget? (info : ConstantInfo) : Option Name :=
  info.value? (allowOpaque := true) |>.bind exactEtaTarget?

private def etaX : Expr := .lam `x (.sort .zero) (.app (.const `f []) (.bvar 0)) .default
private def etaXY : Expr := .lam `x (.sort .zero)
  (.lam `y (.sort .zero) (.app (.app (.const `f []) (.bvar 1)) (.bvar 0)) .default) .default
private def etaYX : Expr := .lam `x (.sort .zero)
  (.lam `y (.sort .zero) (.app (.app (.const `f []) (.bvar 0)) (.bvar 1)) .default) .default
private def etaXX : Expr := .lam `x (.sort .zero)
  (.lam `y (.sort .zero) (.app (.app (.const `f []) (.bvar 1)) (.bvar 1)) .default) .default
private def etaOmitted : Expr := .lam `x (.sort .zero)
  (.lam `y (.sort .zero) (.app (.const `f []) (.bvar 1)) .default) .default

example : exactEtaTarget? etaX = some `f := by native_decide
example : exactEtaTarget? etaXY = some `f := by native_decide
example : exactEtaTarget? etaYX = none := by native_decide
example : exactEtaTarget? etaXX = none := by native_decide
example : exactEtaTarget? etaOmitted = none := by native_decide

def isDirectAlias (info : ConstantInfo) : Bool :=
  directAliasTarget? info |>.isSome

def declarationKind (info : ConstantInfo) : String :=
  if isDirectAlias info then "direct-alias"
  else match info with
  | .thmInfo _ => "theorem"
  | .defnInfo _ => "definition"
  | .opaqueInfo _ => "opaque"
  | .axiomInfo _ => "axiom"
  | .quotInfo _ => "quotient"
  | .inductInfo _ => "inductive"
  | .ctorInfo _ => "constructor"
  | .recInfo _ => "recursor"

/-- Exercise solutions published as checked proofs must not elaborate to exact
eta aliases of another declaration. -/
def checkedExerciseTheoremNames : Array Name := #[
  `CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_01_solution,
  `CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_02_solution,
  `CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_03_solution,
  `CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_04_solution,
  `CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_05_solution,
  `CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_06_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_01_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_02_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_03_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_04_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_05_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_06_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_01_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_02_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_03_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_04_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_05_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_06_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_01_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_02_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_03_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_04_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_05_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_06_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_01_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_02_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_03_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_04_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_05_solution,
  `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_06_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_01_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_02_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_03_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_04_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_05_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_06_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_01_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_02_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_03_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_04_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_05_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_06_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_01_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_02_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_03_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_04_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_05_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_06_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_01_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_02_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_03_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_04_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_05_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_06_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_01_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_02_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_03_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_04_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_05_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_06_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_01_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_02_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_03_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_04_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_05_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_06_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_01_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_02_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_03_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_04_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_05_solution,
  `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_06_solution
]

run_cmd do
  let env ← getEnv
  for name in checkedExerciseTheoremNames do
    let some info := env.find? name
      | throwError "checked exercise declaration is missing: {name}"
    unless declarationKind info == "theorem" do
      throwError "checked exercise must compile as theorem, not {declarationKind info}: {name}"

def normalizeWhitespace (value : String) : String :=
  String.intercalate " " <|
    (value.split Char.isWhitespace).filter (fun part => !part.isEmpty) |>.map (·.toString) |>.toList

def normalizedType (info : ConstantInfo) : CoreM String := MetaM.run' do
  let rendered ← withOptions (fun options =>
      options
        |>.setBool `pp.raw false
        |>.setBool `pp.universes true
        |>.setBool `pp.explicit false
        |>.setBool `pp.fullNames false
        |>.setBool `pp.coercions false
        |>.setBool `pp.proofs false) <| ppExpr info.type
  return normalizeWhitespace rendered.pretty

def sha256Constants : Array UInt32 := #[
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
  0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
  0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
  0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
  0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
  0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
  0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
  0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
  0xc67178f2]

def rotateRight (value : UInt32) (amount : Nat) : UInt32 :=
  (value >>> UInt32.ofNat amount) ||| (value <<< UInt32.ofNat (32 - amount))

def uint32Hex (value : UInt32) : String :=
  let digits := Nat.toDigits 16 value.toNat
  String.ofList (List.replicate (8 - digits.length) '0' ++ digits)

def sha256 (value : String) : String := Id.run do
  let source := value.toUTF8.data
  let bitLength := source.size * 8
  let totalLength := ((source.size + 9 + 63) / 64) * 64
  let mut bytes := source.push 0x80
  while bytes.size + 8 < totalLength do
    bytes := bytes.push 0
  for shift in #[56, 48, 40, 32, 24, 16, 8, 0] do
    bytes := bytes.push <| UInt8.ofNat ((bitLength / (2 ^ shift)) % 256)

  let mut h0 : UInt32 := 0x6a09e667
  let mut h1 : UInt32 := 0xbb67ae85
  let mut h2 : UInt32 := 0x3c6ef372
  let mut h3 : UInt32 := 0xa54ff53a
  let mut h4 : UInt32 := 0x510e527f
  let mut h5 : UInt32 := 0x9b05688c
  let mut h6 : UInt32 := 0x1f83d9ab
  let mut h7 : UInt32 := 0x5be0cd19

  for chunk in Array.range (totalLength / 64) do
    let offset := chunk * 64
    let mut words : Array UInt32 := #[]
    for index in Array.range 16 do
      let byte := offset + index * 4
      words := words.push <|
        (UInt32.ofNat bytes[byte]!.toNat <<< 24) |||
        (UInt32.ofNat bytes[byte + 1]!.toNat <<< 16) |||
        (UInt32.ofNat bytes[byte + 2]!.toNat <<< 8) |||
        UInt32.ofNat bytes[byte + 3]!.toNat
    for index in [16:64] do
      let s0 := rotateRight words[index - 15]! 7 ^^^ rotateRight words[index - 15]! 18 ^^^
        (words[index - 15]! >>> 3)
      let s1 := rotateRight words[index - 2]! 17 ^^^ rotateRight words[index - 2]! 19 ^^^
        (words[index - 2]! >>> 10)
      words := words.push <| words[index - 16]! + s0 + words[index - 7]! + s1

    let mut a := h0
    let mut b := h1
    let mut c := h2
    let mut d := h3
    let mut e := h4
    let mut f := h5
    let mut g := h6
    let mut h := h7
    for index in Array.range 64 do
      let sum1 := rotateRight e 6 ^^^ rotateRight e 11 ^^^ rotateRight e 25
      let choose := (e &&& f) ^^^ ((~~~e) &&& g)
      let temporary1 := h + sum1 + choose + sha256Constants[index]! + words[index]!
      let sum0 := rotateRight a 2 ^^^ rotateRight a 13 ^^^ rotateRight a 22
      let majority := (a &&& b) ^^^ (a &&& c) ^^^ (b &&& c)
      let temporary2 := sum0 + majority
      h := g
      g := f
      f := e
      e := d + temporary1
      d := c
      c := b
      b := a
      a := temporary1 + temporary2
    h0 := h0 + a
    h1 := h1 + b
    h2 := h2 + c
    h3 := h3 + d
    h4 := h4 + e
    h5 := h5 + f
    h6 := h6 + g
    h7 := h7 + h
  return String.join <| [h0, h1, h2, h3, h4, h5, h6, h7].map uint32Hex

def sourceLocation (env : Environment) (name : Name) : Option (String × Nat × Nat) := do
  let ranges ← declRangeExt.find? (level := .exported) env name <|>
    declRangeExt.find? (level := .server) env name
  let moduleName ← env.getModuleFor? name
  let position := ranges.selectionRange.pos
  return (s!"formalization/lean/{moduleName.toString.replace "." "/"}.lean",
    position.line, position.column + 1)

structure AxiomCache where
  seen : NameMap (Array Name) := {}

abbrev AxiomM := ReaderT Environment (StateM AxiomCache)

def insertNames (set : NameSet) (names : Array Name) : NameSet :=
  names.foldl (init := set) fun result name => result.insert name

partial def cachedAxioms (name : Name) : AxiomM (Array Name) := do
  if let some axioms := (← get).seen.find? name then
    return axioms
  -- The empty sentinel gives mutually recursive kernel declarations the same cycle behavior as
  -- `Lean.collectAxioms`.
  modify fun state => { state with seen := state.seen.insert name #[] }
  let env ← read
  let mut axioms : NameSet := {}
  let collectExpr (axioms : NameSet) (expr : Expr) : AxiomM NameSet := do
    let mut result := axioms
    for dependency in expr.getUsedConstants do
      result := insertNames result (← cachedAxioms dependency)
    return result
  match env.checked.get.find? name with
  | some (.axiomInfo value) =>
      axioms := axioms.insert name
      axioms ← collectExpr axioms value.type
  | some (.defnInfo value) =>
      axioms ← collectExpr axioms value.type
      axioms ← collectExpr axioms value.value
  | some (.thmInfo value) =>
      axioms ← collectExpr axioms value.type
      axioms ← collectExpr axioms value.value
  | some (.opaqueInfo value) =>
      axioms ← collectExpr axioms value.type
      axioms ← collectExpr axioms value.value
  | some (.quotInfo _) => pure ()
  | some (.ctorInfo value) => axioms ← collectExpr axioms value.type
  | some (.recInfo value) => axioms ← collectExpr axioms value.type
  | some (.inductInfo value) =>
      axioms ← collectExpr axioms value.type
      for constructor in value.ctors do
        axioms := insertNames axioms (← cachedAxioms constructor)
  | none => pure ()
  let result := axioms.toArray.qsort Name.lt
  modify fun state => { state with seen := state.seen.insert name result }
  return result

def allAxioms (env : Environment) (names : Array Name) : NameMap (Array Name) :=
  let computation : AxiomM Unit := names.forM fun name => discard <| cachedAxioms name
  (computation.run env |>.run {}).2.seen

def receiptRow (env : Environment) (axiomMap : NameMap (Array Name))
    (name : Name) (info : ConstantInfo) : CoreM Json := do
  let type ← normalizedType info
  if type.toUTF8.size > maxStringBytes then
    throwError "normalized type for {name} exceeds the receipt bound"
  let dependencies ← match directAliasTarget? info with
    | some target => pure <| if isMaintainedName target then #[target] else #[]
    | none => pure <| directConstants info |>.filter (· != name)
  if dependencies.size > maxRelations then
    throwError "dependency list for {name} exceeds the receipt bound"
  let some axioms := axiomMap.find? name
    | throwError "axiom closure is unavailable for {name}"
  let axioms := axioms.qsort (fun left right => left.toString < right.toString)
  if axioms.size > maxRelations then
    throwError "axiom list for {name} exceeds the receipt bound"
  let some (sourcePath, line, column) := sourceLocation env name
    | throwError "compiler source range is unavailable for {name}"
  let digest := sha256 type
  let dependencyJson := dependencies.map (Json.str ·.toString)
  let axiomJson := axioms.map (Json.str ·.toString)
  return json% {
    name: $(name.toString),
    kind: $(declarationKind info),
    source_path: $(sourcePath),
    line: $(line),
    column: $(column),
    normalized_type: $(type),
    type_sha256: $(digest),
    direct_dependencies: $(dependencyJson),
    axioms: $(axiomJson)
  }

def receiptRows (env : Environment) (names : Array Name) : CoreM (Array Json) := do
  let axiomMap := allAxioms env names
  names.mapM fun name => do
    let some info := env.find? name | throwError "declaration disappeared: {name}"
    receiptRow env axiomMap name info

def declarationNames (env : Environment) : CoreM (Array Name) := do
  let markers := env.constants.toList.filterMap fun (name, info) =>
    if isReceiptMarkerName name then some (name, info) else none
  if markers.length > maxDeclarations then
    throwError "receipt marker count exceeds {maxDeclarations}"
  let mut selected : NameHashSet := {}
  for (marker, info) in markers do
    let targets := bodyConstants info |>.filter fun name => !isReceiptMarkerName name
    let [name] := targets.toList
      | throwError "receipt marker {marker} must directly reference exactly one maintained declaration"
    selected := selected.insert name
  return selected.toArray.qsort (fun left right => left.toString < right.toString)

private def declarationTokenAt (sourcePath : String) (line column : Nat) : IO String := do
  let rootPrefix := "formalization/lean/"
  unless sourcePath.startsWith rootPrefix do
    throw <| IO.userError s!"receipt source path is outside the Lean root: {sourcePath}"
  if line == 0 || column == 0 then
    throw <| IO.userError s!"receipt source coordinate is not one-based: {sourcePath}:{line}:{column}"
  let localPath := (sourcePath.drop rootPrefix.length).toString
  let lines ← IO.FS.lines localPath
  let some sourceLine := lines[line - 1]?
    | throw <| IO.userError s!"receipt line is outside its source: {sourcePath}:{line}:{column}"
  let suffix := sourceLine.drop (column - 1)
  return (suffix.takeWhile fun character =>
    !character.isWhitespace && character != ':' && character != '(' &&
      character != '{' && character != '[').toString

run_cmd do
  let env ← getEnv
  let names ← Lean.Elab.Command.liftCoreM <| declarationNames env
  for name in names do
    let some (sourcePath, line, column) := sourceLocation env name
      | throwError "compiler source range is unavailable for {name}"
    let token ← declarationTokenAt sourcePath line column
    unless token != "" && name.toString.endsWith token do
      throwError s!"receipt coordinate does not resolve to declaration {name}: \
        {sourcePath}:{line}:{column} starts with {token}"

def buildReceipt : CoreM String := do
  unless sha256 "abc" == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" do
    throwError "internal SHA-256 self-test failed"
  let env ← getEnv
  let names ← declarationNames env
  if names.size > maxDeclarations then
    throwError "receipt declaration count exceeds {maxDeclarations}"
  let rows ← receiptRows env names
  let receipt := json% {
    schema_version: "crouzeix-textbook-lean-receipt/v1",
    toolchain: "leanprover/lean4:v4.32.1",
    target: "CrouzeixTextbook",
    declarations: $(rows)
  }
  let bytes := receipt.pretty ++ "\n"
  if bytes.toUTF8.size > maxReceiptBytes then
    throwError "serialized receipt exceeds {maxReceiptBytes} bytes"
  return bytes

end CrouzeixTextbook.ExportReceipt

unsafe def CrouzeixTextbook.ExportReceipt.writeReceipt
    (output : System.FilePath) (receipt : String) : IO Unit := do
  let handle ← IO.FS.Handle.mk output .writeNew
  try
    handle.putStr receipt
    handle.flush
  catch error =>
    try IO.FS.removeFile output catch _ => pure ()
    throw error

unsafe def CrouzeixTextbook.ExportReceipt.run (args : List String) : IO UInt32 := do
  match args with
  | [output] =>
      let outputPath := System.FilePath.mk output
      unless outputPath.isAbsolute do
        IO.eprintln "receipt output must be absolute"
        return 2
      Lean.initSearchPath (← Lean.findSysroot)
      let receipt ← CoreM.withImportModules #[`CrouzeixTextbook.Correspondence]
        CrouzeixTextbook.ExportReceipt.buildReceipt
      CrouzeixTextbook.ExportReceipt.writeReceipt outputPath receipt
      return 0
  | _ =>
      IO.eprintln "usage: CrouzeixTextbook --receipt-output <absolute-output-file>"
      return 2
