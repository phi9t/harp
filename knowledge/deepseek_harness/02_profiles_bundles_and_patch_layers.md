---
id: deepseek-harness-profiles-bundles-and-patch-layers
title: DeepSeek Harness decision 02 - Profiles, bundles, and patch layers
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, profiles, bundles, config, patching]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, DSH-SMOKE]
---

# Decision 02: Profiles, bundles, and patch layers

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C003].** DSH chooses an ordered profile and bundle stack
as its boot-time product definition. A running harness is not assembled by a
single hard-coded main function; it is a Cordis plugin tree built from profile
templates, bundle rows, profile patches, home patches, and optional overlays.

## Evidence

**EVIDENCE - profile vocabulary.** The architecture guide defines a profile as
a named composition stored in the Harness home and a bundle as a distribution
format for Cordis config rows and mounted code
([[evidence/implementations/deepseek_harness/snapshot/docs/architecture.md|architecture]]
([lines 15-27](../../evidence/implementations/deepseek_harness/snapshot/docs/architecture.md#L15))).

**EVIDENCE - row replacement.** The same guide states that layers apply in
bundle order, then profile patch, home patch, and `--patch` overlay; a patch
targets a row by id and replaces the whole config or inserts new rows
([lines 27-35](../../evidence/implementations/deepseek_harness/snapshot/docs/architecture.md#L27)).

**EVIDENCE - local surface.** The root package scripts expose `dsh`, config
catalog generation, Cordis config verification, and package invariant checks
through package-manager scripts recorded in [[evidence/deepseek_harness_study/README.md|the smoke receipt]].

## Why this matters

**INFERENCE.** Ordered composition gives a harness owner a concrete object to
review: a tree of rows and patches. That is materially different from asking
an evaluator to infer the harness from an installed binary or a scattered set
of imports. It also gives a future candidate-search loop a bounded mutation
surface: propose a row, replace a row, add a bundle, or add an overlay.

The tradeoff is that configuration identity becomes load-bearing. If row IDs
are unstable or patches are broad, an apparent small change can remount more
of the harness than intended. DSH's docs make row identity explicit; Harp
should preserve that as a design lesson rather than treating config as
incidental deployment detail.

## Verification reading

**EVIDENCE - [DSH-SMOKE].** The local receipt is intentionally limited to
source identity, package scripts, static config examples, and keyless docs. It
does not run `dsh --profile web --dump-config` from an installed release or
prove that a user's Harness home patches are well formed.

## Failure modes

**MISSING.** The current Harp evidence does not include a full booted profile
dump, a profile patch mutation test, or an HMR row-diff test. Those would be
the next receipts needed before claiming operational patch safety.
