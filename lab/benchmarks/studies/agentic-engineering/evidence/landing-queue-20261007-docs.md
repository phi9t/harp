# Pending documentation publication — 2026-10-07

Source read and publication base: Corenius `9a92c9523102af409e86f9d59a480f32f0a77c2b`.
Recorded at 2026-10-07T08:23:48Z. The owner authorized review and landing of ready pending work.
This record covers documentation publication and the Pi probe snapshot; it does
not authorize new workloads or claim product implementation acceptance.

## Source selection

- Pi: authoritative integrated `corenius-codex-pi-env-deep-dive` snapshot, 14 files; worker-tree duplicates are excluded. Upstream Pi source is pinned at `ae92585d3b3e5f1e4b123d14a34314d826d8d9f5`.
- Qwen3: `corenius-qwen3-training-bringup`, 21 effort and evidence files. Native owning-repository changes are outside this publication.
- Semantic layout: `corenius-codex-semantic-polyglot-layout`, 9 design/specification/ticket files. No migration is implemented.
- Controller portfolio: 36 files from immutable commit `fec99d6cc3527d38ffed148643c58a9d8341faf2`, including CONTEXT.md. Main's existing CONTEXT.md equals the branch's original base version; its 96-line glossary addition can be imported without overwriting competing additions.

Original source trees and refs are preserved. Each source byte hash was verified
again during copying. Inference: these are coherent publication snapshots based
on the retained intent, source identities, reviews and explicit claim limits.

## Exact normalization

Six semantic tickets change their title to `NN slug: title`, add the tracker-ordered
Type/Status/Owner/Blocked by/Claimed headers, and represent unstarted lifecycle as
`Status: open` with `Triage: ready-for-agent`. Existing bold Status/Blocked by
fields are removed; numeric blocker IDs retain the exact dependency graph.
`Acceptance criteria` becomes `Acceptance`; every original checkbox and all
remaining body text are preserved. A timestamped Comments entry records only
publication normalization. No ticket is claimed or resolved.

Portfolio EFFORT status becomes owner-approved design and reviewed
specification/tickets, with no stabilization actions authorized. SPEC status
records completed independent reviews and unstarted implementation. Both follow
the branch's retained STATE; append-only decisions and STATE bytes are unchanged.
The other 72 imported files are byte-identical to their selected sources.

## Review and fresh checks

Prior clean-context review covered Standards and publication Spec. Its only
blocking finding was semantic tracker formatting, repaired above. Portfolio
status inconsistency is reconciled. Product/runtime qualification remains outside
this review. No actual secret was identified in selected evidence; common
private-key and GitHub/OpenAI credential signature checks found none. No evidence
was excluded merely because it discusses credentials or redaction.

- `python3 tools/agents/tracker.py lint pi-env-durable qwen3-training-bringup semantic-polyglot-layout controller-portfolio-stabilization`: exit 0.
- `python3 tools/test_sync_agentic_workflow.py`: exit 0, 24 tests passed.
- Source stability and SHA-256 comparisons: all 80 files verified; exactly 8 intentional normalizations.
- Original semantic acceptance checkbox and numeric blocker graph comparisons: all six tickets preserved.
- Local inline Markdown links: 36 valid targets; JSON parse: 3 valid files.
- Pi retained report and runnable probe hashes: all match recorded evidence.
- Staged `git diff --cached --check`: exit 0 after empty Blocked by headers were stripped of trailing spaces.

No GPU, provider, live model, installation, build or probe execution occurred.
Historical checkpoint deletion and source/test limitations travel with their
original records. Existing native logs/artifacts remain in their owning trees.

## File identities

SHA-256 hashes refer to exact bytes. Source is the selected pre-normalization
snapshot; published is the normalized artifact. This evidence file is additional
to the 80 imported source files and is not self-hashed.

| Path | Source SHA-256 | Published SHA-256 |
| --- | --- | --- |
| `docs/efforts/pi-env-durable/EFFORT.md` | `dbf44b7f96664ec2afd9a731b62d98eb3b26a5455e28243c76cd8da2124fd19b` | `dbf44b7f96664ec2afd9a731b62d98eb3b26a5455e28243c76cd8da2124fd19b` |
| `docs/efforts/pi-env-durable/STATE.md` | `2398053e6c2493ec675f524e70c0b76b17dc044e5cecff325d45d4645d8d5cb9` | `2398053e6c2493ec675f524e70c0b76b17dc044e5cecff325d45d4645d8d5cb9` |
| `docs/efforts/pi-env-durable/evidence/document-checks.json` | `467fd0668c953d46000c8c8e31e623ab46467797e10700658e6933d7ed1b1aae` | `467fd0668c953d46000c8c8e31e623ab46467797e10700658e6933d7ed1b1aae` |
| `docs/efforts/pi-env-durable/evidence/interrupted-effect-probe/result.json` | `e7033886aa0bf92c3b0458be980d947351de3b34fb8ca219eb84479a7344a2fb` | `e7033886aa0bf92c3b0458be980d947351de3b34fb8ca219eb84479a7344a2fb` |
| `docs/efforts/pi-env-durable/evidence/interrupted-effect-probe/review.md` | `a9d31a8f2ba29a2642ec4005faf889a1a3111f33cfdae4aa724bf94158ef41d4` | `a9d31a8f2ba29a2642ec4005faf889a1a3111f33cfdae4aa724bf94158ef41d4` |
| `docs/efforts/pi-env-durable/tickets/01-pi-research.md` | `b2777da5cdbebca00965531137b729d1be849785adeb5d776e1e2968e3c3ac59` | `b2777da5cdbebca00965531137b729d1be849785adeb5d776e1e2968e3c3ac59` |
| `docs/efforts/pi-env-durable/tickets/02-interrupted-effect-probe.md` | `5fed9b53bbb6265b9ba88d3d1e415ef0c494e770c57935fb4733288594405a9c` | `5fed9b53bbb6265b9ba88d3d1e415ef0c494e770c57935fb4733288594405a9c` |
| `docs/spikes/pi-env-durable/README.md` | `7f7eda3c2d48071922d762c3c469ef757181a300dbf839c7b58da3c12bc5a9e9` | `7f7eda3c2d48071922d762c3c469ef757181a300dbf839c7b58da3c12bc5a9e9` |
| `docs/spikes/pi-env-durable/interrupted-effect-probe.md` | `4258f99931a2b81fc95e87753d4151163faa3fd67d9d83058ac3f9f34a700c30` | `4258f99931a2b81fc95e87753d4151163faa3fd67d9d83058ac3f9f34a700c30` |
| `docs/spikes/pi-env-durable/pi-durable-source-review.md` | `c9e270b4d5b3cfaf9838e51289a8703c4af447ad69494beef6f55d52fabe1758` | `c9e270b4d5b3cfaf9838e51289a8703c4af447ad69494beef6f55d52fabe1758` |
| `docs/spikes/pi-env-durable/pi-env-source-review.md` | `58db3119226878ec8f2597494cef5580457119bb41cf3014428e000956a7d799` | `58db3119226878ec8f2597494cef5580457119bb41cf3014428e000956a7d799` |
| `experiments/pi-env-durable/README.md` | `e059bc63dba667d4b7242123661854e7f8200b696510d9f5e9c4c552fabf200f` | `e059bc63dba667d4b7242123661854e7f8200b696510d9f5e9c4c552fabf200f` |
| `experiments/pi-env-durable/effect-worker.ts` | `cc55a69f2fef08c16d9b91474f0b63a2360246e8934f261699273f45bbea22b0` | `cc55a69f2fef08c16d9b91474f0b63a2360246e8934f261699273f45bbea22b0` |
| `experiments/pi-env-durable/interrupted-effect-probe.py` | `d0dcda08998a7fd7d2de473dce4a556e262fa2467292fde497ab3d33efee641d` | `d0dcda08998a7fd7d2de473dce4a556e262fa2467292fde497ab3d33efee641d` |
| `docs/efforts/qwen3-training-bringup/DESIGN.md` | `d24fbed18dcb5109c7aa14413fad40e818b6fe52dc5afca24e1656ce1ff5e5ec` | `d24fbed18dcb5109c7aa14413fad40e818b6fe52dc5afca24e1656ce1ff5e5ec` |
| `docs/efforts/qwen3-training-bringup/EFFORT.md` | `5cf0274d82643b991a05f3234281eb02fdbbb9c80d82b09724e77f221e9dc091` | `5cf0274d82643b991a05f3234281eb02fdbbb9c80d82b09724e77f221e9dc091` |
| `docs/efforts/qwen3-training-bringup/SPEC.md` | `7d47e543f087c7d14e4ed325e7ddc36ee33a33d7283ff72f4cab1d7acd1e7d1b` | `7d47e543f087c7d14e4ed325e7ddc36ee33a33d7283ff72f4cab1d7acd1e7d1b` |
| `docs/efforts/qwen3-training-bringup/STATE.md` | `720b209c06b312f64508887012deba39fc243322fc965da252ac65a604bf8faf` | `720b209c06b312f64508887012deba39fc243322fc965da252ac65a604bf8faf` |
| `docs/efforts/qwen3-training-bringup/evidence/checkpoint-deletion.json` | `89b1c1bf1e0ed351ad65225b5b4ffe72cc351f7e24bf4ce3461bfd9b28408b65` | `89b1c1bf1e0ed351ad65225b5b4ffe72cc351f7e24bf4ce3461bfd9b28408b65` |
| `docs/efforts/qwen3-training-bringup/evidence/cpt-cleanup-rl-prelaunch-gpus.csv` | `7af7b829456c5c3a10f61002ac33c72348f468d0e9a87a8a29930e9120c4692a` | `7af7b829456c5c3a10f61002ac33c72348f468d0e9a87a8a29930e9120c4692a` |
| `docs/efforts/qwen3-training-bringup/evidence/cpt-cleanup-rl-prelaunch-processes.csv` | `e8c6a2a7a21a1425b0534a85e4d92bd0909502f234d65599f6ea94c74d2e82d6` | `e8c6a2a7a21a1425b0534a85e4d92bd0909502f234d65599f6ea94c74d2e82d6` |
| `docs/efforts/qwen3-training-bringup/evidence/cpt-final-review.md` | `ded7ffced890554fb6e76868fa6befb5ec23c971933e0d431d755343c85af78e` | `ded7ffced890554fb6e76868fa6befb5ec23c971933e0d431d755343c85af78e` |
| `docs/efforts/qwen3-training-bringup/evidence/cpt-prelaunch-gpus.csv` | `98767eacc76e3ac1d72ba46e25d4324e698bf7a7f2f4531a5ee664dd924c90f7` | `98767eacc76e3ac1d72ba46e25d4324e698bf7a7f2f4531a5ee664dd924c90f7` |
| `docs/efforts/qwen3-training-bringup/evidence/cpt-prelaunch-processes.csv` | `264b20e4c4204243fca39e2e389d64c93ae9c26d3a24f6a4db3cb0ca6289400f` | `264b20e4c4204243fca39e2e389d64c93ae9c26d3a24f6a4db3cb0ca6289400f` |
| `docs/efforts/qwen3-training-bringup/evidence/cpt-resume-prelaunch-gpus.csv` | `4fe90eec04cdc06d6c03b789dbca21fbccd0f3a63a0ce52883b3f1273a92a693` | `4fe90eec04cdc06d6c03b789dbca21fbccd0f3a63a0ce52883b3f1273a92a693` |
| `docs/efforts/qwen3-training-bringup/evidence/cpt-resume-prelaunch-processes.csv` | `c4164a8dee0548539785338eb268136dd59ed1260f3d819d5171e409efb22bf0` | `c4164a8dee0548539785338eb268136dd59ed1260f3d819d5171e409efb22bf0` |
| `docs/efforts/qwen3-training-bringup/evidence/design-review-2026-10-06.md` | `3da7ea342a081ddfb8e3f0b706ace18d5c2f1ecbedc17a02985736e5fcdd8ec0` | `3da7ea342a081ddfb8e3f0b706ace18d5c2f1ecbedc17a02985736e5fcdd8ec0` |
| `docs/efforts/qwen3-training-bringup/evidence/final-cleanup-gpus.csv` | `7c480f16097f4cb8b090b15811ec6c6a6992a240c13ab15eea1abef5f72e2a7d` | `7c480f16097f4cb8b090b15811ec6c6a6992a240c13ab15eea1abef5f72e2a7d` |
| `docs/efforts/qwen3-training-bringup/evidence/gpu-budget.md` | `1d0d2675ffd7f9b4e3090f2e19e734f3f0c950c544b8d54b55e50334e1019e66` | `1d0d2675ffd7f9b4e3090f2e19e734f3f0c950c544b8d54b55e50334e1019e66` |
| `docs/efforts/qwen3-training-bringup/evidence/rl-final-review.md` | `eb36ca9f98c1e65479798e2a87bd569a1f114403c4ca6d3e7d230fafab7666bd` | `eb36ca9f98c1e65479798e2a87bd569a1f114403c4ca6d3e7d230fafab7666bd` |
| `docs/efforts/qwen3-training-bringup/evidence/torchtitan-readiness-2026-10-06.md` | `925299f23adad23367b0707c030fb8b91bd66f8fee3cf08923b1a584a9cb774a` | `925299f23adad23367b0707c030fb8b91bd66f8fee3cf08923b1a584a9cb774a` |
| `docs/efforts/qwen3-training-bringup/evidence/verl-readiness-2026-10-06.md` | `a29893825efbf730d480b6b406548dd2ac5c089f2610f61eb66f50d2d813cf86` | `a29893825efbf730d480b6b406548dd2ac5c089f2610f61eb66f50d2d813cf86` |
| `docs/efforts/qwen3-training-bringup/tickets/01-workload-contract.md` | `7e73819555ac575865aa7cb5c607f5e2f95b370db7903c0179e4cc2c1fb5927a` | `7e73819555ac575865aa7cb5c607f5e2f95b370db7903c0179e4cc2c1fb5927a` |
| `docs/efforts/qwen3-training-bringup/tickets/02-torchtitan-cpt.md` | `bdacd9ccbc65c3fbda9e9ff0bc41decacc64965795e45b062eadc9b29ebc0632` | `bdacd9ccbc65c3fbda9e9ff0bc41decacc64965795e45b062eadc9b29ebc0632` |
| `docs/efforts/qwen3-training-bringup/tickets/03-verl-grpo.md` | `b31fdade140a834e4c89e45a63bcbc133008adfd7f6bb9f7531f7d2d52a79fb4` | `b31fdade140a834e4c89e45a63bcbc133008adfd7f6bb9f7531f7d2d52a79fb4` |
| `docs/adr/0001-semantic-corenius-component-layout.md` | `31c2cac3659368e3dd2b4348a254c94e6f33b7b8a892d912623d7b3f8386f749` | `31c2cac3659368e3dd2b4348a254c94e6f33b7b8a892d912623d7b3f8386f749` |
| `docs/efforts/semantic-polyglot-layout/EFFORT.md` | `04d42de17436c18a6d1e2232b9b8223f7bb4a9d4c59bb64258dea0a6f74c05e1` | `04d42de17436c18a6d1e2232b9b8223f7bb4a9d4c59bb64258dea0a6f74c05e1` |
| `docs/efforts/semantic-polyglot-layout/SPEC.md` | `cb5885cca2bb138cb581a827c04fe653f273ab03ad215a83250032119bdd1b1a` | `cb5885cca2bb138cb581a827c04fe653f273ab03ad215a83250032119bdd1b1a` |
| `docs/efforts/semantic-polyglot-layout/tickets/01-explicit-controller-launch.md` | `4b63bf101c7c3551b09dd63218c75fb6585e486e85190c0f9f798efbc9e9e08b` | `2f8215b225f21ddf5282c65409a7e81cf1722066a317608f39bd3fce7d3837e6` |
| `docs/efforts/semantic-polyglot-layout/tickets/02-relocate-terminal-agent.md` | `339c1fc63b93072de0ac5c2e55c6b703f4af50476ee2b6534fe7482f6c7c8d08` | `1f42f1f7c3e7385c786a3965a59e20ebac8f6a64649206891dcf03f1264cb3d2` |
| `docs/efforts/semantic-polyglot-layout/tickets/03-relocate-controller.md` | `ea1090435eb6f3a9312342b3192fa697e85f163692d0a2a154800722c88cf0e9` | `a4861f591824ee757ce4ff5f8a50d6a4e6a253d9e33a977b3781ddabb1099e8e` |
| `docs/efforts/semantic-polyglot-layout/tickets/04-relocate-parked-control-plane.md` | `de9255370aeace9de9851600a9a01446c5d2e815660b0b5216ca8291c4b01716` | `03e9124a7d1e5762f76245f34a78add4322dcd86ffd3d65c03b8ef8271f803f8` |
| `docs/efforts/semantic-polyglot-layout/tickets/05-consolidate-cargo.md` | `981ec34b3321ee96ab958deabdbd02ec0f904e59fc809b0398e3b8aa6c5ed013` | `8767523ebbaddabc2fbc5659267b630db15ec2b15d276a675b5e187b62e65a6c` |
| `docs/efforts/semantic-polyglot-layout/tickets/06-verify-migration.md` | `7ad8bf67cf1808dbf60257a5f26e1725a8ab90ece6d93bad9dc2c84629866a39` | `2a5210a282f314a6d10e8d5617a9f4abe94fe5020c3c68366d41274214dd9b8a` |
| `CONTEXT.md` | `d6dd45075dcfe90a72452717091458ee3800bde7c13c6e7e5c0b11c60ea2c5f9` | `d6dd45075dcfe90a72452717091458ee3800bde7c13c6e7e5c0b11c60ea2c5f9` |
| `docs/efforts/controller-portfolio-stabilization/EFFORT.md` | `b199c34ce16a4418fa5099142595a353845f74610e64e2f896ae803735d3ba10` | `24f7962d1e175f1984de76c2df0cfc3fb056f493d995b926ff4f7b356dc30066` |
| `docs/efforts/controller-portfolio-stabilization/SPEC.md` | `252effbbf12a31f3146f8260fd366ef367a7ff638c9f45c493f180a35868f141` | `be9935037308342a71d77e973b3ac5b9adec11f3daeba10a6fa931569dcf7e16` |
| `docs/efforts/controller-portfolio-stabilization/STATE.md` | `c0ae5736348eb1f1560868c61103f00c6fd015bec178c380de1590b576b89789` | `c0ae5736348eb1f1560868c61103f00c6fd015bec178c380de1590b576b89789` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-bootstrap-lead-supersedes-first-scout.md` | `a9fd4088d8ef87a379c9a7c27449c6ff91121c391c233dab854a279c87f785d0` | `a9fd4088d8ef87a379c9a7c27449c6ff91121c391c233dab854a279c87f785d0` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-controller-first-director-interface.md` | `da659260f9c4db819028c47f81b6db95679c05d6d31c3b7c875dfabead6394ee` | `da659260f9c4db819028c47f81b6db95679c05d6d31c3b7c875dfabead6394ee` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-corenius-program-of-record.md` | `84475e850f9dfd726c705eadb34bdca26efba603caffbc610f97a92f778c8208` | `84475e850f9dfd726c705eadb34bdca26efba603caffbc610f97a92f778c8208` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-distinct-corenius-lead.md` | `ef61191b80b53c94d1895b56a42beb84c3aa4b5aa81fcc0650065ad48b42b4e7` | `ef61191b80b53c94d1895b56a42beb84c3aa4b5aa81fcc0650065ad48b42b4e7` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-first-role-epoch-is-scout-only.md` | `9a594065fbcbbe999bd2d70dc08e8bcc2a1ef863e8378a83dcf7c7e7bff59c5a` | `9a594065fbcbbe999bd2d70dc08e8bcc2a1ef863e8378a83dcf7c7e7bff59c5a` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-harvest-i1-before-i2-completion.md` | `7c28bbebe8de821174443ea027dac3506b31c7264943e96dd87914c3d3b5db03` | `7c28bbebe8de821174443ea027dac3506b31c7264943e96dd87914c3d3b5db03` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-independent-five-minute-observer.md` | `fe710c22f9c8fed1820a98d565508b49f73b98d9c86e4b7d91d5d6eb1fbda7d9` | `fe710c22f9c8fed1820a98d565508b49f73b98d9c86e4b7d91d5d6eb1fbda7d9` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-recovery-first-i2-tracer.md` | `1bad5ca39fc9037bb3ef4942a5b5c6417c5b4bb057637f8db428391eb6ed74f4` | `1bad5ca39fc9037bb3ef4942a5b5c6417c5b4bb057637f8db428391eb6ed74f4` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-redact-evidence-before-persistence.md` | `3c15dd2be343786257192830c9b00e99475d161affed5ebfb85c5e4bb70670e5` | `3c15dd2be343786257192830c9b00e99475d161affed5ebfb85c5e4bb70670e5` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-remote-autonomous-lead.md` | `19d523261284b95dc595507654875f7c8ba9ae873d9974d5d3dbc2a3bf255e4c` | `19d523261284b95dc595507654875f7c8ba9ae873d9974d5d3dbc2a3bf255e4c` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-risk-tiered-mechanism-promotion.md` | `1763159129d5fc8cda08822b5424a7e51b86c79eaca153f85b68064365322705` | `1763159129d5fc8cda08822b5424a7e51b86c79eaca153f85b68064365322705` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-rung-zero-uses-trial-repository.md` | `1e985036ef4d2ab1867b077374fecd2383be6073df241c69d9c2f3a55cb38df2` | `1e985036ef4d2ab1867b077374fecd2383be6073df241c69d9c2f3a55cb38df2` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-schema-driven-trial-repository-factory.md` | `ac79507d33d7f0562e285fe57fd8052057fe2d4e848daefbd485435116de5670` | `ac79507d33d7f0562e285fe57fd8052057fe2d4e848daefbd485435116de5670` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-trae-analogue-and-codex-reclassification.md` | `0077b3ac1e7a3a327de53cd7e8eff033a6d9ed026eee8e5f5fa1250243439dc6` | `0077b3ac1e7a3a327de53cd7e8eff033a6d9ed026eee8e5f5fa1250243439dc6` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-trae-first-provider-ladder.md` | `b6827c9086cdbb8a620f65545a6d53b625f0bdd6b015f34a533f59a163f5f879` | `b6827c9086cdbb8a620f65545a6d53b625f0bdd6b015f34a533f59a163f5f879` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-trae-single-worker-is-rung-zero.md` | `26b590e9df5dbaf385071182afc1b36440cfc152bd29c7fedfb2dc79a7c78258` | `26b590e9df5dbaf385071182afc1b36440cfc152bd29c7fedfb2dc79a7c78258` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-two-pass-tiered-stabilization.md` | `cfe568cc87cdc9abe5a60b0e7096b674cdf1edc87af045eb7d8c9994ba9edd82` | `cfe568cc87cdc9abe5a60b0e7096b674cdf1edc87af045eb7d8c9994ba9edd82` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-two-round-live-inventory.md` | `3e84469d1d7308d8ec9fb4566cc262f8ef6c314be92ea66d159dc5cd81071631` | `3e84469d1d7308d8ec9fb4566cc262f8ef6c314be92ea66d159dc5cd81071631` |
| `docs/efforts/controller-portfolio-stabilization/decisions/20261006-typed-evidence-and-reviewed-action-packets.md` | `c2c63ff6711d871f0b69a1d49824a7180f2db6c685e080f72a0df961dc4eba46` | `c2c63ff6711d871f0b69a1d49824a7180f2db6c685e080f72a0df961dc4eba46` |
| `docs/efforts/controller-portfolio-stabilization/tickets/01-admit-developer-insula.md` | `094c91bdefbb00d13e7a9505225f2d2e05139d4aa4c5503969e340f8be9434ab` | `094c91bdefbb00d13e7a9505225f2d2e05139d4aa4c5503969e340f8be9434ab` |
| `docs/efforts/controller-portfolio-stabilization/tickets/02-bootstrap-lead-roundtrip.md` | `d937a932ca3f1629c1788ce39c72abe0639e8cc8c90718304ef4f1553863bd68` | `d937a932ca3f1629c1788ce39c72abe0639e8cc8c90718304ef4f1553863bd68` |
| `docs/efforts/controller-portfolio-stabilization/tickets/03-one-source-sealed-evidence.md` | `198e113047ded384ef8aca1f0c0b0d48862231db2249a77a8097df17474256c3` | `198e113047ded384ef8aca1f0c0b0d48862231db2249a77a8097df17474256c3` |
| `docs/efforts/controller-portfolio-stabilization/tickets/04-repository-stabilization-envelope.md` | `fea90d2f0f1036fcb47b3a1719bf72f4ee6b7b0ef4ceb040bb417e7c03831f45` | `fea90d2f0f1036fcb47b3a1719bf72f4ee6b7b0ef4ceb040bb417e7c03831f45` |
| `docs/efforts/controller-portfolio-stabilization/tickets/05-controller-portfolio-closure.md` | `cdb875a1dfb7aca287f0cec9000bb167a634ac5130612121d2d2e07ae682d2f6` | `cdb875a1dfb7aca287f0cec9000bb167a634ac5130612121d2d2e07ae682d2f6` |
| `docs/efforts/controller-portfolio-stabilization/tickets/06-evidence-bound-action-packet.md` | `e9b03b41c8ec0b99c169a10979a03f5562b9cad6f836f38a0662e12033eabf32` | `e9b03b41c8ec0b99c169a10979a03f5562b9cad6f836f38a0662e12033eabf32` |
| `docs/efforts/controller-portfolio-stabilization/tickets/07-i1-mechanism-harvest.md` | `01c447846024c051ffb2de7e0ff9cfab8f7be6661db42f7dbbdc7a7303e08aa4` | `01c447846024c051ffb2de7e0ff9cfab8f7be6661db42f7dbbdc7a7303e08aa4` |
| `docs/efforts/controller-portfolio-stabilization/tickets/08-trial-repository-factory.md` | `099319bdbaa544ffb3369219121c9f7e9e3a5cf779ccef815016ed2abe0f8fa5` | `099319bdbaa544ffb3369219121c9f7e9e3a5cf779ccef815016ed2abe0f8fa5` |
| `docs/efforts/controller-portfolio-stabilization/tickets/09-trae-rung0-happy-path.md` | `15d48358983d60052e4590831be41a309382ddb427dad1bf1c789770ac61abbd` | `15d48358983d60052e4590831be41a309382ddb427dad1bf1c789770ac61abbd` |
| `docs/efforts/controller-portfolio-stabilization/tickets/10-trae-rung0-recovery.md` | `868fee4c54fc5ef5e0432b2eadc023c45f7b024b21deea1e914a95549fbb0114` | `868fee4c54fc5ef5e0432b2eadc023c45f7b024b21deea1e914a95549fbb0114` |
| `docs/efforts/controller-portfolio-stabilization/tickets/11-trae-rung1-concurrency.md` | `885cd0f82644cba9be317279d50b4ed6f30423bfc98e9d900a645ed7de36852e` | `885cd0f82644cba9be317279d50b4ed6f30423bfc98e9d900a645ed7de36852e` |
| `docs/efforts/controller-portfolio-stabilization/tickets/12-codex-parity.md` | `32eb18f55b23def03d9a32bb372f143f6443f7acf8b1e8040f92fe525a20e9fc` | `32eb18f55b23def03d9a32bb372f143f6443f7acf8b1e8040f92fe525a20e9fc` |
| `docs/efforts/controller-portfolio-stabilization/tickets/13-mixed-provider-faults.md` | `0d2731907a4df88fcab9ef4c036950cda22a81ea4176852d74f6dcb124c74345` | `0d2731907a4df88fcab9ef4c036950cda22a81ea4176852d74f6dcb124c74345` |
