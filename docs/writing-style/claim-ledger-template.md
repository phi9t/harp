# Claim ledger template

Each `##` heading is a stable link target. Keep IDs stable after publication.

## EX-001: Released selector reads the shallow score

- Class: `EVIDENCE`
- Statement: The released selector reads `overall_performance`.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Example implementation registry entry](source-registry-template.md#example-repo-released-dgm-implementation)
- Locator: [`DGM_outer.py:63`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L63)
- Scope: Example pinned implementation scope.
- Reproduction: Source inspected, not executed.
- Confidence: `high`
- Confidence basis: Direct source-code path.
- Caveat: The expanded result may be stored under a different field.

## EX-002: Authors report the benchmark result

- Class: `SOURCE CLAIM`
- Statement: The authors report 50.0% on a 200-task benchmark subset.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Example paper registry entry](source-registry-template.md#example-paper-darwin-godel-machine-paper)
- Locator: [DGM paper 200-task accounting, line 317](../../evidence/weng/text/dgm.txt#L317)
  and [reported 50.0% result, line 346](../../evidence/weng/text/dgm.txt#L346)
- Scope: 200-task subset, named model and metric from the source.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct paper result statement.
- Caveat: Selected-run result; do not generalize to the full benchmark.

## EX-003: The result supports harness improvement

- Class: `INFERENCE`
- Statement: The current corpus supports harness improvement only; it does not
  establish that accepted children become better producers of later
  improvements.
- Source: [EX-001](#ex-001-released-selector-reads-the-shallow-score) and
  [EX-002](#ex-002-authors-report-the-benchmark-result)
- Locator: [`DGM_outer.py:63`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L63)
  and [DGM paper results, line 346](../../evidence/weng/text/dgm.txt#L346)
- Scope: The inspected mechanism and reported experiment only.
- Reproduction: Source results not independently reproduced here.
- Confidence: `medium`
- Confidence basis: Direct mechanism evidence; missing next-cycle experiment.
- Caveat: The external improvement process may carry unmeasured capability.
- Weakens if: The reported benchmark gain does not persist under a matched
  rerun of the evaluated child.
- Falsified by: An auditable matched next-cycle study shows accepted children
  outperform their parents at producing later accepted improvements.

## EX-004: Next-cycle evidence is missing

- Class: `MISSING`
- Statement: The inspected corpus contains no matched parent-versus-child
  next-cycle experiment.
- Source: [Example paper registry entry](source-registry-template.md#example-paper-darwin-godel-machine-paper)
  and [example implementation registry entry](source-registry-template.md#example-repo-released-dgm-implementation)
- Locator: [DGM paper method, lines 120-138](../../evidence/weng/text/dgm.txt#L120)
  and [released implementation inventory](../../evidence/implementations/dgm/snapshot/README.md#L65)
- Scope: Current inspected corpus.
- Reproduction: Not applicable.
- Confidence: `high`
- Confidence basis: Explicit search of the inspected method and evaluation.
- Caveat: A non-public experiment may exist outside the corpus.
- Resolves when: A source reports the matched experiment with auditable
  artifacts.

## EX-005: Repository describes empirical validation

- Class: `EVIDENCE`
- Statement: The repository describes DGM as a system that "empirically
  validates each change using coding benchmarks."
- Mode: `quote`
- Source stability: `pinned`
- Source: [Example implementation registry entry](source-registry-template.md#example-repo-released-dgm-implementation)
- Locator: [`README.md:14`](../../evidence/implementations/dgm/snapshot/README.md#L14)
- Scope: Wording in the pinned repository README.
- Reproduction: Source wording inspected; behavior not independently reproduced.
- Confidence: `high`
- Confidence basis: Direct quotation from the inspected artifact.
- Caveat: The quotation describes intended system behavior, not a reproduced
  execution.

## EX-006: Paper and repository describe the same loop

- Class: `EVIDENCE`
- Statement: Both pinned sources describe DGM as iteratively modifying its own
  code and empirically validating changes on coding benchmarks.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Example paper registry entry](source-registry-template.md#example-paper-darwin-godel-machine-paper) and
  [example implementation registry entry](source-registry-template.md#example-repo-released-dgm-implementation)
- Locator: [DGM paper abstract, lines 29-31](../../evidence/weng/text/dgm.txt#L29) and
  [`README.md:14`](../../evidence/implementations/dgm/snapshot/README.md#L14)
- Scope: The shared bounded mechanism description; either source independently
  supports the statement.
- Reproduction: Source content inspected; system behavior not independently
  reproduced.
- Confidence: `high`
- Confidence basis: Two independently sufficient primary-source descriptions.
- Caveat: Agreement between descriptions does not establish runtime behavior.
