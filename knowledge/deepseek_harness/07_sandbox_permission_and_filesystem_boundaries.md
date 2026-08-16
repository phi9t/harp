---
id: deepseek-harness-sandbox-permission-and-filesystem-boundaries
title: DeepSeek Harness decision 07 - Sandbox, permission, and filesystem boundaries
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, sandbox, permissions, filesystem, shell]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, DSH-SMOKE]
---

# Decision 07: Sandbox, permission, and filesystem boundaries

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C011].** DSH chooses a shared per-call sandbox policy
service for file-effect boundaries, then lets filesystem and shell providers
enforce that policy through their own surfaces. The sandbox vocabulary is not a
general security label; it is a file-effect policy with explicit limits.

## Evidence

**EVIDENCE - mode vocabulary.** The sandbox doc defines `read-only`,
`workspace-write`, and `danger-full-access` as filesystem-effect modes, and
states that network and process visibility are outside the vocabulary
([[evidence/implementations/deepseek_harness/snapshot/docs/subsystems/sandbox.md|sandbox docs]]
([lines 9-30](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/sandbox.md#L9))).

**EVIDENCE - per-call policy.** The same doc says complete execution policy is
resolved per capability call, including mode, workspace root, and optional
session id; `ctx.sandboxPolicy.resolve()` owns precedence and root fallback
([lines 41-80](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/sandbox.md#L41)).

**EVIDENCE - policy implementation.** `SandboxPolicyService` defaults to
`read-only`, resolves request mode from explicit override, session override, or
deployment default, and contributes a runtime context section describing the
current policy
([[evidence/implementations/deepseek_harness/snapshot/packages/sandbox/sandbox-policy/src/index.ts|sandbox policy source]]
([lines 60-75](../../evidence/implementations/deepseek_harness/snapshot/packages/sandbox/sandbox-policy/src/index.ts#L60),
[lines 126-151](../../evidence/implementations/deepseek_harness/snapshot/packages/sandbox/sandbox-policy/src/index.ts#L126))).

**EVIDENCE - filesystem and shell consumers.** `SandboxedFileSystem` denies
mutations in `read-only`, re-canonicalizes targets for `workspace-write`, and
delegates unfenced in `danger-full-access`; `SandboxBashExecutor` wraps bash
argv through `ctx.sandbox`, reports enforcement and denial facts, and treats
runner launch failure as a sandbox infrastructure failure
([[evidence/implementations/deepseek_harness/snapshot/packages/fs/fs-sandbox/src/index.ts|fs sandbox]]
([lines 1-30](../../evidence/implementations/deepseek_harness/snapshot/packages/fs/fs-sandbox/src/index.ts#L1),
[lines 115-148](../../evidence/implementations/deepseek_harness/snapshot/packages/fs/fs-sandbox/src/index.ts#L115));
[[evidence/implementations/deepseek_harness/snapshot/packages/shell/bash-sandbox/src/index.ts|bash sandbox]]
([lines 1-8](../../evidence/implementations/deepseek_harness/snapshot/packages/shell/bash-sandbox/src/index.ts#L1),
[lines 88-114](../../evidence/implementations/deepseek_harness/snapshot/packages/shell/bash-sandbox/src/index.ts#L88))).

## Why this matters

**INFERENCE.** DSH's most important sandbox decision is to keep policy
resolution out of individual tools. The tool layer asks for a capability call;
the shared policy service decides the effective mode and workspace root; the
capability provider enforces through the mechanism it owns. That reduces drift
between in-process file edits and spawned shell commands.

The claim ceiling matters. DSH docs explicitly say the vocabulary governs file
effects and not network or process visibility. Harp should not turn a
file-policy design into a blanket security claim.

## Verification reading

**EVIDENCE - [DSH-SMOKE].** Local smoke supports source-level inspection of
policy resolution and provider code. It does not prove Landlock, Seatbelt,
Windows ACL, or other platform runner behavior on this machine.

## Failure modes

**MISSING.** Future receipts should run read-only denial, workspace-write
success, workspace escape denial, and runner-failure classification on the
target OS. This packet does not claim those empirical checks have been run.
