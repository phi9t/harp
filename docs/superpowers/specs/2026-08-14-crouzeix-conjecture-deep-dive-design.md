# Crouzeix conjecture two-proof deep-dive design

**Status:** Adversarial re-review complete, pending user approval
**Date:** 2026-08-14
**Audience:** MTS-level readers comfortable with linear algebra, complex
analysis, operator theory, and proof assistants
**Packet root:** `knowledge/crouzeix_conjecture/`

## 1. Objective

Build a source-backed Harp packet that explains and compares two public proof
artifacts that state proofs of Crouzeix's conjecture:

1. Shanmu Jin's pinned Git manuscript and Lean route through positive-real
   completion and a Herglotz kernel; and
2. Emiel Lorist and Felix Schwenninger's arXiv preprint route through a
   perturbation lemma for \(2\)-dilations.

The packet must teach the shared double-layer background first, derive each
proof independently, audit Jin's Lean formalization, inspect the proof
interfaces of both arguments, and reconstruct AI-assisted discovery only to
the limit allowed by public evidence.

The packet should let a reader:

1. state Crouzeix's conjecture as a sharp numerical-range spectral-set bound;
2. explain why the earlier symmetrized double-layer estimate stops at
   \(1+\sqrt{2}\);
3. explain why both proofs retain the whole power family discarded by
   one-step norm estimates;
4. derive the positive-real completion theorem used in Jin's candidate proof;
5. show how one origin sample cancels the unknown analytic correction exactly;
6. follow the remaining block inequality through weighted Gramians to the norm
   bound \(2\);
7. understand how simple-spectrum and outer-domain limits remove Jin's
   auxiliary assumptions;
8. derive the Lorist-Schwenninger recurrence for uniformly controlled
   \(2\)-dilation perturbations;
9. trace the double-layer realization
   \(E_n=\alpha(f^n)(A)\) and the route from power iterates to \(\|f(A)\|\le2\);
10. supply the one-line product-bound estimate compressed in the
    Lorist-Schwenninger lemma proof;
11. compare the two proof objects, hypotheses, order-sensitive steps, and
    completely bounded limitation;
12. trace Jin's Lean endpoint and its trust boundary;
13. separate Jin's formalized manuscript revision from later manuscript and
    Lean changes;
14. identify which parts of both AI-assisted discovery histories are public
    and which remain unavailable.

This is an explanation and source audit. It is not a mathematical peer-review
certificate, a publication-status claim, or an independent proof of the
conjecture.

## 2. Status language

Harp will call Jin's result a **candidate proof** because the repository says
formal peer review is pending. It will call Lorist-Schwenninger
**arXiv preprint authors who state a proof**. arXiv v1 is not a peer-reviewed
publication record. A first-party Preprints.org record for Jin reports version
1 as submitted on 2026-07-24, posted on 2026-07-27, and not peer reviewed; the
implementation research pass must capture or hash-pin that record before using
those dates as evidence.

The packet may say:

- the manuscript states a proof;
- the Lean library exports a theorem with the Crouzeix inequality as its type;
- the pinned Lean source contains no `sorry`, `admit`, custom `axiom`, unsafe
  proof construction, or native-decision shortcut found by the documented
  scan;
- a successful Lean build checks the encoded theorem chain under Lean and
  Mathlib's trust boundary; and
- the Lorist-Schwenninger preprint states another proof of the same conjecture
  and says Jin's proof appeared independently.

The packet must not say:

- peer review has accepted Jin's proof;
- Lean independently proves that the informal manuscript is correct;
- the formalization proves that every definition matches the intended
  mathematics;
- the Lorist-Schwenninger preprint validates Jin's distinct mechanism;
- Jin's proof validates Lorist-Schwenninger's distinct mechanism;
- public repository history reveals the full agent search process; or
- Harp has independently reproduced the mathematical review performed by the
  repository author.

The index and status chapter will state this boundary before using the word
"proof" without attribution.

## 3. Source identity model

Jin's repository changes the manuscript and formalization on different
schedules. Lorist-Schwenninger publish a separate versioned arXiv artifact.
The packet will preserve these identities instead of calling one mutable file
"the proof."

| Source ID | Immutable identity | Role | Claim ceiling |
|---|---|---|---|
| `JIN-V4-AUDITED` | Git commit `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`; `preprint/the_numerical_range_is_a_2_spectral_set_v4.tex`; 1,106 lines; SHA-256 `5713de029c4a7486e25e86d16e6413d04929bdf5f92439c3237d4a930b1c9242` | Manuscript revision named by the Lean formalization map and manuscript audit | Informal argument and the repository author's claimed manuscript-to-Lean correspondence at this revision |
| `JIN-REPO-HEAD` | Git commit `9df07838327b988e3924453daa29c8cd726d34b0` | Latest pinned repository inspected for this packet | Repository contents, current theorem declarations, current README status, and later source evolution |
| `JIN-V4-HEAD` | File at `JIN-REPO-HEAD`; 1,979 lines; SHA-256 `27f77d75a02faa39a613e18a6b52f3548195b73ceee4996ced38f54319a00315` | Later v4 manuscript state | Later manuscript claims, not byte-for-byte coverage by the stale 1,106-line formalization map |
| `JIN-ANNMATH` | File at `JIN-REPO-HEAD`; `AnnMath/the_numerical_range_is_a_2_spectral_set.tex`; SHA-256 `abc7944c185fe34004de1367ba9d8d0c8a1912a0ac989230ed810feef8a11695` | July Annals-formatted manuscript described by the repository as a submission manuscript | Its own mechanism; actual journal submission status is `MISSING` |
| `JIN-PREPRINTS-V1` | Preprints.org manuscript `202607.1919`, version 1; immutable page or artifact digest to be recorded during implementation | Public preprint metadata record cited by Lorist-Schwenninger | Posted metadata and non-peer-reviewed status only, unless the actual manuscript bytes are acquired and mapped to a Git artifact |
| `LS-ARXIV-V1` | arXiv:2608.03841v1, published 2026-08-04; source archive SHA-256 `b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9`; TeX SHA-256 `20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a`; PDF SHA-256 `f08667c21f62c170afa4470d7cb0b76dc72341f5feb669ff4fb931dcc989f7cb` | Lorist-Schwenninger primary preprint | Their theorem, proof mechanism, self-reported independence, limitations, and AI disclosure |

The Lean prose files at `JIN-REPO-HEAD` still name the 1,106-line digest while
the same v4 path contains 1,979 lines. This is a source-identity contradiction,
not evidence that a theorem is false. The source registry and claim ledger will
record it explicitly.

The Jin mathematical derivation will use `JIN-V4-AUDITED` because it is the
only manuscript revision with an explicit matching digest in the Lean audit.
The Lean chapter will inspect and build both `JIN-V4-AUDITED` and
`JIN-REPO-HEAD` as separate observations. The Lorist-Schwenninger derivation
will use `LS-ARXIV-V1`. The status chapter will keep all publication,
formalization, and peer-review states separate.

The packet must not imply that `JIN-V4-AUDITED` is the manuscript posted at
Preprints.org. If implementation acquires the posted manuscript bytes, it will
record their digest and either map them byte-for-byte to one Git artifact or
register them as a separate proof revision. Without that evidence,
`JIN-PREPRINTS-V1` remains metadata only.

## 4. Evidence corpus

### 4.1 Required primary sources

The implementation research pass is capped at:

1. `JIN-V4-AUDITED`;
2. `JIN-REPO-HEAD`, including:
   - `README.md`;
   - `crouzeix_conjecture_prompt.txt`;
   - `Lean/README.md`;
   - `Lean/STATUS.md`;
   - `Lean/FORMALIZATION_MAP.md`;
   - `Lean/MANUSCRIPT_AUDIT.md`;
   - `Lean/AXIOM_AUDIT.md`;
   - `Lean/AxiomAudit.lean`;
   - `Lean/verify.sh`;
   - endpoint and dependency modules under `Lean/CrouzeixConjecture/`; and
   - repository commit history;
3. `JIN-PREPRINTS-V1`;
4. `LS-ARXIV-V1`, including its TeX source and versioned arXiv metadata;
5. the Crouzeix-Palencia paper and the minimum primary-source follow-up needed
   to establish the \(1+\sqrt{2}\) mechanism and its stated limitation;
6. the Delyon-Delyon and Crouzeix sources needed to audit the shared
   double-layer prerequisites used by both proofs;
7. first-party GitHub, Preprints.org, and arXiv metadata; and
8. local verification output produced during implementation.

Secondary news coverage may motivate a reader to open the packet, but it cannot
support a mathematical, formalization, attribution, or status claim.

### 4.2 Redistribution boundary

The Jin repository has no license file at `JIN-REPO-HEAD`.
`LS-ARXIV-V1` uses arXiv's perpetual non-exclusive distribution license, which
does not grant Harp a general relicensing right. Harp will not vendor either
proof's manuscripts, PDFs, TeX, Lean files, prompt, or other upstream bytes.
Canonical Harp prose may use short, attributed quotations when a theorem
interface or disclosure cannot be represented accurately by paraphrase. Each
quotation must be under 50 words, linked to an immutable locator, and included
in the packet's quotation audit. The original prompt will be summarized, not
reproduced.

The packet will use remote-only hash-pinned receipts:

```text
evidence/crouzeix_conjecture/
├── PROVENANCE.md
├── acquire.sh
├── source_manifest.tsv
├── verification_manifest.tsv
└── verification/
    ├── jin-565b6a3-build.log
    ├── jin-565b6a3-scan.log
    ├── jin-9df0783-build.log
    └── jin-9df0783-scan.log
```

`source_manifest.tsv` uses this exact header:

```text
schema_version	receipt_id	source_id	source_class	role	immutable_identity	source_url	upstream_path	bytes	sha256	local_path	observed	license_status	redistribution_status
```

Every row has `schema_version=crouzeix-source-receipt/v1`.
`receipt_id` is unique and matches `[A-Z0-9][A-Z0-9-]*`. `source_id` is
repeatable. `source_class` is one of `git-artifact`, `arxiv-artifact`,
`journal-record`, or `dated-page`. `role` is one of `manuscript`, `source`,
`pdf`, `metadata`, `formalization`, `prompt`, `history`, `prerequisite`, or
`license`.

`immutable_identity` is `git:<40-hex>`, `arxiv:<id>vN`, `doi:<value>`, or
`sha256:<64-lower-hex>`. `source_url` is HTTPS and identity-bound.
`upstream_path` is `-` when the source has no path. `bytes` is a decimal
integer and `sha256` is 64 lowercase hex when measured; both are `-` only for
dated metadata pages whose bytes could not be acquired. `local_path` is always
`-`. `observed` is an ISO date. `license_status` is one of
`not-present-at-revision`, `arxiv-nonexclusive`, `publisher-record`,
`unknown`, or `not-redistributable`. `redistribution_status` is
`remote-only`, `quotation-only`, or `metadata-only`.

Receipt IDs must be unique. A source ID may own many artifact rows. The tuple
`(source_id, immutable_identity, upstream_path)` must also be unique. The
verifier will require GitHub URLs to contain the declared 40-hex commit, arXiv
URLs to contain the exact `vN`, and other source classes to use their declared
immutable identity or a dated observation.

`verification_manifest.tsv` uses this exact header:

```text
schema_version	receipt_id	source_id	source_commit	source_tree	operation	command_sha256	acquisition_script_sha256	normalization_version	toolchain	mathlib_revision	observed_at_utc	exit_code	result	log_path	log_bytes	log_sha256
```

Every row has `schema_version=crouzeix-verification-receipt/v1`.
`operation` is `lean-build` or `source-scan`. `command_sha256` hashes the exact
NUL-delimited argv. `acquisition_script_sha256` binds the row to `acquire.sh`.
`normalization_version` is `crouzeix-log-normalization/v1`. `result` is
`passed`, `failed`, or `blocked`. Exit code is a decimal integer for an
attempted command and `-` only for `blocked`. Build and scan are separate
observations. A build failure remains a failed receipt, not a source-inspection
success.

Both TSV files are UTF-8, LF-terminated, have one row per physical line, forbid
NUL, CR, tab, or newline inside fields, and reject unknown enum values or
columns.

`harp sources verify` will enforce the exact allowed file roster shown above,
reject symlinks and every undeclared regular file, require all upstream
artifact rows to use `local_path=-`, validate both manifests, and verify every
local log byte count and digest. Since verification is offline, it cannot prove
that a remote URL still serves the recorded bytes. The packet will state that
limit.

`acquire.sh` is the explicit networked path. It creates fresh temporary
checkouts, verifies remote and commit identity before running commands,
downloads versioned arXiv and preprint records for hashing without copying
those bytes into Harp, runs both Jin verification and scan jobs, normalizes the
logs, and updates the manifests. It refuses dirty or preexisting source
checkouts, Git alternates, replacement refs, config includes, symlinks, and
identity mismatches.

The acquisition runner will normalize each local log before it enters the
repository:

- replace the ephemeral checkout root with `<source-checkout>`;
- preserve non-source command output and exit status;
- reject NUL bytes and logs over 2 MiB;
- scan for credentials and abort rather than redact a secret-bearing log; and
- record the raw-to-normalized transformation in `PROVENANCE.md`.

The normalized logs are Harp-generated observations. They are not substitutes
for upstream source and must not contain copied manuscript or source-file
bodies. Source scans emit only path, line number, token class, and line digest,
never the matching line. Build-log normalization detects source-line excerpts
from the pinned checkout; if any appear, raw output remains outside Harp and
the tracked log contains only synthesized command metadata, diagnostic class,
and digests.

## 5. Mathematical teaching contract

### 5.1 Shared starting point and prior barrier

The packet will derive the earlier interface

\[
2\Phi(f)=f(B)+g_f(B)^*.
\]

Positivity controls the sum, while the target is \(f(B)\). Treating the
companion as an unrelated operator loses the algebraic relation required for
the constant \(2\). The packet will attribute the \(1+\sqrt{2}\) bound and
every claim about its sharpness to the primary sources that establish them.

Both new routes retain the complete power family rather than applying a
one-step estimate only to \(f\):

- Jin sends the Cayley family of \(f\) through \(\Phi\), producing one
  matrix-valued positive-real function whose kernel couples all sampled
  values; and
- Lorist-Schwenninger apply the double-layer identity to every \(f^n\),
  producing perturbations \(E_n\) of a \(2\)-dilation.

This common reading is a Harp `INFERENCE`, not terminology used by either
source. Its ledger entry must say what would weaken or falsify the comparison.

### 5.2 Jin route: complete positive-real completion contract

For a simple-spectrum auxiliary matrix

\[
B=S\operatorname{diag}(\beta_i)S^{-1},
\]

the target

\[
T=S\operatorname{diag}(\lambda_i)S^{-1}
\]

uses the same basis. The target values may repeat. The completion interface is

\[
|\lambda_i|\le1,\qquad
H\text{ analytic on }\mathbb D,\qquad
H(0)=I,
\]

and, for every \(w\in\mathbb D\),

\[
\operatorname{Re}H(w)\succeq0,\qquad
H(w)-(I-wT)^{-1}\in\operatorname{alg}(B^*).
\]

The mathematical chapter will explain why the auxiliary basis turns the
unknown defect into a diagonal analytic correction. It will not replace this
interface with the older, stronger requirement
\(\operatorname{alg}(T)=\operatorname{alg}(B)\). It will also show where
normalizing \(f\) establishes \(|\lambda_i|\le1\), why this keeps the sampled
denominators nonzero, and where analyticity enters Herglotz-kernel positivity.

### 5.3 Jin route: exact cancellation

Let \(G=S^*S\). Define the weighted matrices

\[
P_{ij}=\frac{G_{ij}}
 {1-\overline{\lambda_i}\lambda_j/4},
\qquad
Q_{ij}=\frac{G_{ij}}
 {1-\overline{\lambda_i}\lambda_j/2},
\qquad
Y=Q-P.
\]

The Herglotz kernel is sampled at

\[
w_i=\frac{\overline{\lambda_i}}{2},
\qquad
\xi_i=u_i e_i.
\]

The additional origin vector is

\[
\xi_0=v=-G^{-1}Pu.
\]

The packet will show the cancellation equation

\[
Gv+Pu=0
\]

before discussing its significance. This is the central mechanism. The
unknown analytic correction disappears by construction, not by a norm bound.

### 5.4 Jin route: ordered Gramian endpoint

After cancellation, kernel positivity gives the ordered inequality

\[
4Y-YG^{-1}P-PG^{-1}Y\succeq0.
\]

Balancing the common eigenbasis produces two convergent Gramians with weights
\(4^{-k}\) and \(2^{-k}\). The packet will not claim that their positive
difference alone bounds the first Gramian. It will derive both:

\[
\widehat Y=\widehat Q-\widehat P\succeq0
\]

and

\[
4\widehat Y-\widehat Y\widehat P-\widehat P\widehat Y\succeq0.
\]

An eigenvector contradiction using both inequalities gives
\(\widehat P\preceq2I\). The first nonconstant Gramian term then yields
\(\widetilde T^*\widetilde T\preceq4I\).

The formalized 1,106-line route and current Lean audit describe the direct
first-term endpoint. The July Annals manuscript uses a Stein identity and an
older algebra-generation perturbation. The packet will not merge these two
routes into one proof.

### 5.5 Jin route: limit order

The formalized route takes two limits in this order:

1. on a fixed admissible outer domain, approximate \(A\) by
   simple-spectrum matrices \(B_k\);
2. shrink canonical convex outer domains to \(W(A)\).

The packet will state whether a third function perturbation occurs for each
manuscript revision. It must not import the Annals manuscript's perturbation
into the formalized v4 route.

### 5.6 Lorist-Schwenninger route: perturbation lemma

The source states the following lemma. For a finite-dimensional Hilbert space
\(H\), an operator \(T\), a contraction \(Q\) on \(K\), and an isometry
\(V:H\to K\), define

\[
E_n=2V^*Q^{*n}V-T^{*n}.
\]

The authors assume that the \(E_n\) are uniformly bounded and commute with
\(T\), and conclude \(\|T\|\le2\).

The packet will derive the proof recurrence, not merely quote the lemma. Let
\(\kappa=\|T\|>1\), choose a unit norm-attaining vector \(x\), and define

\[
m_n=\operatorname{Re}\langle E_nT^nx,x\rangle.
\]

Completing the square gives a lower recurrence

\[
\kappa m_n-m_{n+1}\ge r_n,
\]

which iterates to

\[
m_1\ge \kappa^{-N}m_{N+1}
      +\sum_{n=1}^{N}\kappa^{-n}r_n.
\]

The rest of the proof combines the limiting lower bound for \(m_1\) with the
contraction estimate

\[
\|Q^*VTx-\kappa Vx\|^2
\le 2\kappa^2-\kappa m_1-\kappa^3
\]

to rule out \(\kappa>2\).

### 5.7 Lorist-Schwenninger route: compressed boundedness step

The source says uniform boundedness of \(\{E_n\}\) carries over to
\(\{E_nT^n\}\), then uses this to make
\(\kappa^{-N}m_{N+1}\) vanish. The text compresses the following direct
estimate. If \(M=\sup_n\|E_n\|<\infty\), Equation 1 gives:

\[
\|T^n\|
=\|T^{*n}\|
\le 2\|V^*Q^{*n}V\|+\|E_n\|
\le 2+M.
\]

Therefore:

\[
\|E_nT^n\|\le M(2+M),
\]

and for \(\kappa>1\):

\[
\kappa^{-N}|m_{N+1}|
\le \kappa^{-N}M(2+M)
\longrightarrow0.
\]

The packet will present the source sentence and this derivation as one
`EVIDENCE` block. It will not classify the step as a missing hypothesis or
unresolved gap.

### 5.8 Lorist-Schwenninger route: double-layer realization

For a smooth convex outer domain \(\Omega\) and
\(\|f\|_{\infty,\Omega}=1\), the source constructs:

- a positive unital map \(\Phi\);
- an isometry
  \(Vx=2^{-1/2}P_\Omega(\cdot)^{1/2}x\);
- a multiplication contraction \(Qg=f\,g\);
- the target \(T=f(A)\); and
- perturbations

\[
E_n
=2V^*Q^{*n}V-T^{*n}
=\alpha(f^n)(A).
\]

The functional calculus makes \(E_n\) commute with \(T\). Boundedness of
\(\alpha\) gives a uniform \(E_n\) bound. The packet will also show the
application-specific product estimate
\[
E_nT^n=(\alpha(f^n)f^n)(A),
\]
and
\[
\|E_nT^n\|
\le \|\theta\|\,\|\alpha\|\,\|f^n\|_\infty^2
\le \|\theta\|\,\|\alpha\|,
\]
subject to the exact function-algebra and functional-calculus hypotheses
established by the primary sources. This is a Harp derivation from the source's
displayed identities, not a sentence stated by the authors. It independently
checks the general product bound inside the Crouzeix application; the lemma
already supplies the needed bound through Equation 1.

The proof then applies the perturbation argument on each smooth convex outer
domain and uses the cited finite-dimensional reduction and outer-domain limit
to obtain the rational \(2\)-spectral-set statement for bounded operators.
Every reduction delegated to earlier work must be cited as such rather than
reproved by assertion.

### 5.9 Proof comparison and common limit

The comparison chapter will use this fixed contract:

| Question | Jin | Lorist-Schwenninger |
|---|---|---|
| Retained family | Cayley parameter \(w\) and positive kernel | Powers \(f^n\) and perturbations \(E_n\) |
| New finite-dimensional lemma | Positive-real completion modulo \(\operatorname{alg}(B^*)\) | Uniform-power \(2\)-dilation perturbation lemma |
| Decisive cancellation or estimate | Origin sample cancels diagonal correction | Recurrence controls the negative first-step defect |
| Order-sensitive input | Anticommutator Gramian inequality | Commutation of \(E_n\) with \(T\) |
| Route from double layer | Cayley completion | Stinespring-style boundary isometry and multiplication contraction |
| Formalization evidence | Lean source and local build receipts | None found in the bounded corpus |
| Known direct limitation | No immediate completely bounded extension | Commutation is unavailable after matrix amplification |

The table is a synthesis. Each row needs separate source-backed premises and an
`INFERENCE` ledger entry for the cross-proof comparison.

## 6. Jin Lean verification contract

The Lean chapter follows declarations, not the stale prose line map.

It will draw two graphs rather than one misleading linear list.

The main-bound dependency graph includes:

1. `FinalTheorems.lean:crouzeixConjecture`;
2. `HolomorphicOuterLimit.lean:holomorphicCrouzeixBound`;
3. fixed-domain holomorphic evaluation and double-layer construction;
4. `CompletionStatement.lean:PositiveRealCompletionStatement`, the proposition
   definition;
5. `CompletionDiagonalization.lean:positiveRealCompletionStatement`, the
   theorem proving that proposition;
6. the completion kernel, cancellation, Gramian, and norm modules; and
7. simple-spectrum approximation and canonical outer-domain modules.

The consequence graph separately includes:

- `FinalTheorems.lean:crouzeixRationalSpectralSetCorollary`;
- `FinalTheorems.lean:crouzeixRationalBound`; and
- `Sharpness.lean:crouzeixConstantTwo_isLeast_finTwo`.

Sharpness is downstream of the main bound, not a dependency needed to prove
the main bound.

The chapter will distinguish:

- a checked Lean theorem;
- a repository-authored formalization map;
- a successful Harp-local rerun of `Lean/verify.sh`;
- correspondence between manuscript and formalization; and
- independent mathematical review.

These are different claims.

Implementation will create detached, clean source checkouts for both Jin
commits and run the revision's own verification command:

```sh
git -C <checkout> rev-parse HEAD
cd <checkout>/Lean
./verify.sh
```

It will also run a separate tracked-source scan over each commit's `.lean`
files for `sorry`, `admit`, custom `axiom`, `unsafe`, `native_decide`,
`implemented_by`, and option overrides. The scan command, pattern, path scope,
and exclusions belong in the receipt. Build success does not imply scan
success, and scan success does not imply endpoint axiom purity.

Each result will be recorded in the corresponding local receipt and bounded
log. If dependency materialization or the build fails, the packet will
preserve the failure and will not downgrade it to a source-inspection success.
The 565b6a3 build supports a claim about the manuscript revision named by that
commit's formalization audit. The 9df0783 build supports only the current
source-level theorem graph. Neither build proves byte correspondence to a
different revision.

The trust boundary includes Lean 4.28.0, Mathlib commit
`8f9d9cff6bd728b17a24e163c9402775d9e6a365`, and the standard axioms printed
by `AxiomAudit.lean`. The chapter will explain that kernel checking does not
prove that an encoded definition is the intended mathematical object.

## 7. AI-assisted discovery contract

The discovery chapter may establish only what public artifacts show.

For Jin, supported facts include:

- the original prompt required a complete standalone proof, diverse agent
  approaches, adversarial review, and continued search after blocked routes;
- the audited manuscript disclosure credits OpenAI ChatGPT with the idea of
  sampling the matrix Herglotz kernel at half the conjugate eigenvalues and
  adding the origin sample that cancels the correction;
- the disclosure also credits exposition, bibliography, and typesetting help;
- Git history records manuscript and formalization revisions; and
- the repository says ChatGPT contributed to proof development, preparation,
  and adversarial checking.

For Lorist-Schwenninger, arXiv v1 says ChatGPT 5.6 Pro was used to explore
proof strategies and that the authors wrote the note and take responsibility
for it.

The inspected public artifacts do not contain either effort's underlying agent
transcripts, complete approach registry, rejected proof attempts, prompts for
each subtask, token or time accounting, or a complete causal record from model
interaction to theorem. These items will be `MISSING`.

The chapter may use commit history to describe Jin artifact evolution. It must
not turn commit messages into a fabricated reasoning transcript. Its action
matrix will map only documented actions to source, context, tool, and
verification artifact.

## 8. Packet structure

```text
knowledge/crouzeix_conjecture/
├── crouzeix_conjecture_index.md
├── 01_problem_and_prior_barrier.md
├── 02_shared_power_family.md
├── 03_jin_proof_spine.md
├── 04_jin_positive_real_completion.md
├── 05_lorist_schwenninger_proof.md
├── 06_proof_comparison.md
├── 07_jin_lean_verification.md
├── 08_ai_assisted_discovery.md
├── 09_status_and_critical_assessment.md
├── glossary.md
├── source_registry.md
└── claim_evidence_ledger.md
```

The tree contains thirteen files, not twelve: one index, nine numbered
chapters, and three reference documents. No `README.md` will be created. The
named index is the packet entrypoint.

Every file will contain frontmatter with:

- `id`;
- `title`;
- `type`;
- `status`;
- `created`;
- `updated`;
- `tags`;
- `confidence`; and
- `canonical`.

Every non-index file will link back to
`crouzeix_conjecture_index.md`.

## 9. Document contracts

### 9.1 Index

`crouzeix_conjecture_index.md` owns:

- publication and peer-review boundaries for both preprints;
- the Jin revision identities and Lorist-Schwenninger arXiv identity;
- a five-minute two-proof mechanism summary;
- guided and expert reading routes;
- question-driven routes;
- the shared-power-family interpretation;
- formalization and local-verification status; and
- links to every packet document.

### 9.2 Shared background

`01_problem_and_prior_barrier.md` owns the conjecture, function classes,
spectral-set terminology, sharpness, the prior \(1+\sqrt2\) result, and the
symmetrized double-layer identity.

`02_shared_power_family.md` owns the bounded Harp inference that both proofs
recover information by retaining all powers or the equivalent Cayley family.
It states the limits of that comparison before branching into the two routes.

### 9.3 Jin route

`03_jin_proof_spine.md` owns the end-to-end argument from normalized
holomorphic function through fixed-domain and outer-domain limits.

`04_jin_positive_real_completion.md` owns the complete theorem hypotheses,
Herglotz-kernel positivity, common-basis correction, exact cancellation,
ordered Gramian inequalities, eigenvector contradiction, and first-term norm
endpoint. A worked \(2\times2\) symbolic example is allowed only if every
assumption is explicit. It is explanatory, not proof evidence.

`07_jin_lean_verification.md` owns source identities, both revision-bound
build and scan receipts, the dependency and consequence graphs, proposition
definition versus proof theorem, axiom boundary, manuscript mismatch, and the
limits of formal verification.

### 9.4 Lorist-Schwenninger route

`05_lorist_schwenninger_proof.md` owns the stated perturbation lemma, its
recurrence, the limiting step, the double-layer realization, rational
spectral-set conclusion, and the direct product-bound estimate compressed in
the source.

The chapter labels the source's lemma and conclusion as `SOURCE CLAIM`, direct
inspection of the limiting step as `EVIDENCE`, the direct general estimate as
`INFERENCE` from Equation 1, and the application-level functional-calculus
estimate as a separate `INFERENCE`.

### 9.5 Comparison, discovery, and status

`06_proof_comparison.md` owns the fixed comparison contract in Section 5.9 and
the common completely bounded limitation.

`08_ai_assisted_discovery.md` owns both AI disclosures, Jin's original task
contract, the artifact timeline, the action-to-context/tool matrix, and missing
process evidence.

`09_status_and_critical_assessment.md` owns publication status, source
contradictions, proof-interface gaps, formalization correspondence limits,
strongest justified conclusions, and evidence needed to raise confidence.

### 9.6 Reference documents

`glossary.md` defines only terms needed by the packet.

`source_registry.md` fixes source identities, source classes, immutable
locators, quotation audit, and claim ceilings.

`claim_evidence_ledger.md` owns every material claim under the schema below.

## 10. Claim-ledger and locator contract

Claim IDs match `CC-[0-9]{3}[A-Z]?`. Each canonical claim heading is:

```markdown
## CC-001: Short descriptive title
```

Reader-facing material claims use exactly:

```markdown
**[EVIDENCE - CC-001](claim_evidence_ledger.md#cc-001-short-descriptive-title).**
```

The allowed classes are `EVIDENCE`, `SOURCE CLAIM`, `INFERENCE`, and
`MISSING`.

Every entry contains exactly one value for:

- `Class`;
- `Statement`;
- `Source`;
- `Locator`;
- `Scope`;
- `Reproduction`;
- `Confidence`;
- `Confidence basis`; and
- `Caveat`.

`EVIDENCE` and `SOURCE CLAIM` also require:

- `Mode`, exactly `quote` or `paraphrase`; and
- `Source stability`, exactly `pinned` or `dated observation`.

Dated observations include `Observed` with the repository's existing ISO-date
validation. Every `INFERENCE` also has
`Weakens if` and `Falsified by`. Every `MISSING` has `Resolves when`.
`Source` may list several ledger or registry links. `Locator` must still route
to the premises or inspected corpus.

Allowed relationship values are `contradicts`, `unresolved-with`,
`supersedes`, and `narrows`. `contradicts` and `unresolved-with` must be
reciprocal. Every target must exist and no claim may target itself.

`Relationship` is an optional repeatable field excluded from the exactly-once
field set. Its exact syntax is:

```markdown
- Relationship: `unresolved-with CC-002`
```

Each value is `<kind> <CC-ID>`. Duplicate edges are rejected.

`Locator` always points to an exact local record under
`evidence/crouzeix_conjecture/`, usually a source-manifest row or verification
log line. The optional repeatable `Upstream locator` field provides the
source-class-specific semantic route:

- Git sources use a commit-pinned HTTPS URL and path plus line or declaration;
- arXiv sources use exact `vN` HTML, PDF, or source identity plus section,
  theorem, equation, or source line;
- journal sources use DOI or stable publisher/author artifact plus section,
  theorem, or page; and
- absent evidence uses a local manifest or provenance record describing the
  inspected corpus, not a mutable remote URL.

The source verifier binds every upstream URL to its declared immutable
identity. Reader prose links to the exact ledger heading, and the ledger links
to the verified local record. Direct upstream links supplement that local
route but never replace it.

Short quotations are listed in `source_registry.md` with claim ID, source,
locator, exact word count, and purpose. No quotation exceeds 50 words.

## 11. Atlas reader contract

### 11.1 First-class route

Rust adds reader route ID `crouzeix-conjecture` with label `Crouzeix` and the
packet index path. Atlas updates:

- `ReaderRouteId` and `readerRouteIds`;
- parser fixtures and route tests;
- the header navigation with a visible `Crouzeix` button; and
- `currentLabel` to use the route's configured label rather than the raw ID.

The route hash is `#crouzeix-conjecture`.

### 11.2 Stable heading fragments

Registered-document links preserve sections using:

```text
#documents/<document-id>?section=<heading-id>
```

Markdown headings that serve as link targets use explicit attributes:

```markdown
## CC-001: Short descriptive title {#cc-001-short-descriptive-title}
```

Before rendering any document, the corpus compiler builds one path registry
covering `READER_ROUTES`, `AUXILIARY_DOCUMENTS`, coverage-owned chapters, and
published system documents. Reader-route ownership has precedence. Link
rewriting uses that registry:

- a link to `crouzeix_conjecture_index.md` becomes
  `#crouzeix-conjecture`;
- a link to an auxiliary packet file becomes
  `#documents/<document-id>`; and
- a validated auxiliary fragment adds `?section=<heading-id>`.

The Rust renderer enables heading attributes and validates unique canonical
IDs. `heading_ids` reads parser-provided IDs when present and keeps the
existing slug fallback for older documents. Link rewriting retains a validated
fragment as the `section` query. Tests require every non-index backlink to
resolve exactly to `#crouzeix-conjecture`.

`AtlasRoute.document` carries `sectionId: string | null`. After document
render, `CanonicalDocumentView` finds the target inside its own article,
assigns `tabIndex=-1`, focuses it without initial scroll, then scrolls it into
view. Invalid or absent sections fail closed to the document root. Route,
focus, fragment-preservation, generated-corpus, and static-export tests cover
this behavior.

### 11.3 Deterministic math

For sources beneath `knowledge/crouzeix_conjecture/` only, the Rust renderer
enables `Options::ENABLE_MATH`, which emits escaped `math-inline` and
`math-display` spans containing TeX. Other corpus roots keep their current
parser options, so existing currency text is not reinterpreted as math.

Atlas pins `katex` 0.18.4 and `@types/katex` 0.16.8.
`CanonicalDocumentView` renders each span with `katex.render` using:

```text
output = "mathml"
throwOnError = true
trust = false
strict = "error"
```

MathML output avoids external font assets and remains inside the single-file
offline export. The original escaped TeX stays in `data-tex` and remains
visible with a `math-error` marker if rendering fails. Packet tests enumerate
all packet math events and require KaTeX success, so a committed packet cannot
rely on the runtime fallback.

Tests cover representative inline and display formulas, escaping, invalid
commands, accessible MathML, no network or external font reference, and
component-level rendering after route navigation. Static-export tests decode
the bundled module and assert that the KaTeX renderer, packet formulas, and
route support are present while no external script, stylesheet, font, or
network fetch remains.

## 12. Harp integration

The packet is a registered research packet on AI-assisted theorem discovery,
formal verification, and evidence discipline. It does not enter the RSI
retained-concept taxonomy, system registry, diagnostic cases, or evidence
graph. `docs/product-contract.md` will state this narrow product rationale.

Implementation will:

1. add one `READER_ROUTES` entry for the packet index;
2. add the other twelve files to `AUXILIARY_DOCUMENTS`;
3. add `knowledge/crouzeix_conjecture` to search roots;
4. update Rust and TypeScript route contracts;
5. implement fragment-safe registered-document links;
6. implement deterministic MathML rendering;
7. update exact corpus counts and `docs/product-contract.md`;
8. add packet, source-receipt, route, math, and export tests;
9. regenerate `atlas/src/content/generated/corpus.json`;
10. run `corepack pnpm run export:html` to regenerate
    `atlas/dist/harp-atlas.html` and
    `atlas/dist/harp-atlas.receipt.json`; and
11. refresh `docs/import-receipt.md` after each coherent commit's other tracked
    files settle.

The expected corpus change is:

- reader routes: 11 to 12;
- auxiliary documents: 16 to 28; and
- compiled documents: 69 to 82.

No retained concept, coverage entry, RSI system, Weng section, diagnostic
case, lesson, source-registry row, or evidence-edge count changes.

## 13. Test contract

`crates/harp/tests/crouzeix_conjecture_knowledge_packet.rs` verifies:

- the exact thirteen-file packet roster;
- required frontmatter and maintained authority notice;
- index backlinks, no absolute local paths, and no placeholders;
- every local path and explicit heading anchor;
- the complete claim-ledger grammar and routed claim markers;
- source-class-specific immutable locators;
- all Jin and Lorist-Schwenninger source identities and digests;
- both publication-status boundaries;
- Jin's formalization-map mismatch;
- the full positive-real completion hypotheses;
- Lorist-Schwenninger's direct general product bound and independent
  application-level product bound;
- separate definition, proof, consequence, and sharpness Lean declarations;
- both AI disclosures and missing transcript evidence;
- no mutual-validation claim;
- the exact evidence-root file roster and no symlinks or upstream local paths;
- verification receipts and log hashes; and
- packet registration in corpus and search.

Rust corpus tests verify explicit heading IDs, fragment-preserving rewrites,
math events, and count changes. Atlas tests verify the route union and parser,
visible navigation, human-readable label, document section focus, KaTeX
MathML, invalid-math behavior, and static export. CLI tests update exact
canonical-document and source-report accounting.

## 14. Verification

Focused iteration:

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test cli
cd atlas && corepack pnpm run test
cd atlas && corepack pnpm run test:export
```

Generated output checks:

```sh
cargo run -p harp -- build
cargo run -p harp -- build --check
cd atlas && corepack pnpm run export:html
```

For each commit, after all other staged paths settle:

```sh
cargo run -p harp -- repository verify
```

Copy the reported payload digest into `docs/import-receipt.md`, restage it, run
the focused gate against staged content, then run:

```sh
mise run verify
```

## 15. Commit and landing plan

The current design commit will be amended with this revision and its refreshed
import receipt.

Implementation uses three focused, independently verifiable commits:

1. **Evidence boundary.** Source and verification manifests, normalized build
   and scan receipts, exact evidence-root verifier, tests, product-contract
   evidence wording, and refreshed import receipt.
2. **Reader substrate.** Fragment-safe document routes, heading IDs, MathML
   rendering, generic document-route parsing, reader tests, regenerated Atlas
   HTML and export receipt, and refreshed import receipt. This commit does not
   add `crouzeix-conjecture` to the closed route roster.
3. **Two-proof packet.** All thirteen canonical documents, the
   `crouzeix-conjecture` Rust and TypeScript route entries, visible navigation,
   packet and integration tests, corpus/search registration, count changes,
   regenerated corpus/HTML/export receipt, product-contract packet wording,
   and refreshed import receipt.

Each commit stages explicit paths, passes its focused tests and
`mise run verify`, and contains the trailer:

```text
Co-authored-by: TRAE CLI <noreply@bytedance.com>
```

The branch will be landed locally only after the release gate passes. It will
not be pushed unless the project owner asks.

## 16. Completion criteria

The packet is complete when:

1. a reader can derive Jin's cancellation and ordered Gramian endpoint;
2. a reader can derive the Lorist-Schwenninger recurrence and double-layer
   realization;
3. the Lorist-Schwenninger compressed product-bound step is derived explicitly
   and evidence-routed;
4. every material claim reaches an exact ledger entry and immutable source;
5. source identities, publication status, build status, and peer-review status
   remain separate;
6. both Jin revisions have dated build and scan receipts or explicit failures;
7. both AI disclosures are sourced and missing process evidence is explicit;
8. no unlicensed upstream artifact is vendored;
9. Atlas exposes all thirteen documents with working section links and MathML;
10. search indexes all thirteen documents;
11. packet-specific and repository-wide tests pass; and
12. the import and Atlas receipts match the final tracked payload.
