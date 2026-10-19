# Soma

Soma is the quest workspace for Emacs-specialist model work: an operator model, a completion model, and the executable Emacs environment used to train and evaluate them.

## Language

**Doom Emacs gym**:
An executable training and evaluation environment that runs controlled Emacs workflows against disposable editor, repository, process, and kernel state.
_Avoid_: Emacs benchmark, toy evaluator

**Insula**:
A selected rootfs entered through `bwrap` whose tool, package, dependency, and
ABI closure is accounted for by repo-owned contracts and evidence. It owns
Emacs, Doom package state, LSP servers, Jupyter kernels, language toolchains,
subprocess environments, cache/runtime paths, and ABI-relevant shared libraries
unless a host projection is explicitly declared.
_Avoid_: container, virtualenv, loose sandbox, namespace-only wrapper

**Rootfs contract**:
The metadata and validators that prove a process is running in the intended rootfs with the intended mount layout, tool paths, and namespace shape.
_Avoid_: environment flag

**Rootfs materialization**:
The build phase that constructs the selected rootfs closure and emits provenance
before any benchmark episode runs. For the production Soma Insula, Spack is the
likely owner of ABI-sensitive compiled closure; `soma.insula` owns runtime
selection and proof.
_Avoid_: ad hoc host install, invisible package bootstrap

**Patched Emacs**:
A source-pinned Emacs build with repo-recorded patches used to expose required
operability or observability evidence that stock Emacs cannot provide faithfully.
It is part of the Insula identity and must be compared against stock Emacs under
the same fixtures.
_Avoid_: local custom binary, invisible instrumentation

**Task fixture**:
A disposable workspace plus initial editor/process state for one gym task, paired with an independent checker for the final state.
_Avoid_: test case, sample repo

**Benchmark**:
A collection of task fixtures, public goals, permitted operations, private graders, budgets, and scoring rules used to evaluate policies against the gym.
_Avoid_: gym, environment

**Episode**:
One resettable run of a policy against a single task fixture inside the gym, from environment construction through final grading.
_Avoid_: session, rollout

**Grader**:
A private checker that evaluates final state and invariants independently of the learner-facing bridge and policy messages.
_Avoid_: success message, verifier

**Action profile**:
A versioned set of policy-visible operations, such as native checked actions or terminal keymap input.
_Avoid_: API mode, tool list

**Internals Atlas**:
A source-grounded map of Emacs, Doom, and package execution semantics, backed by executable probes and derived gym requirements.
_Avoid_: notes, reading list

**Mechanism probe**:
An executable study of one editor mechanism that records a hypothesis, source entry points, observed trace, independent checks, counterexample, and derived gym requirement.
_Avoid_: unit test, demo

**Operational fidelity**:
Evidence that the intended editor or package path ran with the right command context, not merely that the final filesystem or buffer state matches.
_Avoid_: output correctness

**Trace layer**:
The shared recording surface for observations, actions, tool results, revision identities, process events, and verifier outcomes.
_Avoid_: transcript, log dump
