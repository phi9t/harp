# Independent final CPT review

Date: 2026-10-07

Decision: approve the bounded CPT milestone with the limitations below. No critical or important issue found in the reviewed implementation and existing execution evidence.

Scope: read-only inspection of launcher.py, preflight.py, evidence.py, test_launcher.py, test_evidence.py, inside.sh, postcheck.py and the corresponding qualification/live receipts under `/data02/home/philip.yang/workspace/torchtitan/.worktrees/qwen3-cpt-insula/experiments/qwen3_cpt_insula`. No GPU execution, installs, builds, or checkpoint payload scans by this reviewer. Only this requested review receipt was written.

Verified evidence:

- Native first attempt explicitly enables HF bootstrap. Both ranks record exact matches to two pinned HF parameter tensors, followed by changed sampled parameter hashes at step 10. This supports real pretrained initialization with sampled tensor confirmation, not independent verification of every tensor.
- The CPU preflight records 2,000 nonempty local FineWeb text rows, tokenizer-derived eligible tokens, actual dataset loading, and exact local content SHA256. The historical upstream revision remains unavailable and is correctly disclosed.
- Native initial attempt runs steps 1–10 and writes full synchronous DCP; a fresh second attempt explicitly disables HF bootstrap, loads step 10, and advances through step 20 with the same 20-step scheduler horizon.
- Both ranks' saved/restored step-10 trainer state, scheduler, learning rate and sampled model hashes match. The completed CPU postcheck compares saved/restored loader state bytes and confirms 10 batches yielded at the boundary on both ranks.
- `results/live/verified-result.json` and successful `postcheck.log` establish 4,127 metadata keys and three hashed files for each step-10/20 checkpoint, including model/optimizer/loader/scheduler/trainer coverage. A native CPU DCP read confirms sampled optimizer steps 10 and 20. Sampled model parameters change during both attempts.
- All four native rank outcomes succeeded. TensorBoard has finite losses for all steps 1–20 and positive gradient norms. Global token progress reaches 20,480. Loss endpoints are 2.9128332138061523 and 2.5557003021240234; these do not prove convergence or quality improvement.
- Both GPU launch plans contain identical hashes of the five declared input trees. The launcher now includes executable __pycache__ contents and recomputes input hashes per launch. Native preflight creation probes returned EROFS for the declared rootfs/source/model/dependency projections, while owned output was writable.
- The two launch receipts total 296.53090452193284 GPU-seconds. Allocation/free-device checks and aggregate cross-workload accounting are coordinator-owned; no additional GPU budget was consumed by this review.

Limits preserved: RNG restoration is not established; no uninterrupted control or numerical continuation-equivalence claim is supported. HF and resumed model equality are sampled. Dataset upstream provenance is incomplete. Auxiliary structured logging encountered a torch2.14 _CUuuid serialization failure; canonical run_evidence, checkpoint receipts and TensorBoard succeeded. The run demonstrates native checkpoint continuation under the declared Insula projections, not hostile-code isolation or a fully independently reconstructed environment.

A minor direct-unittest discovery ordering issue was sent to the author; the already executed discovery mode includes the bytecode identity test, so this does not invalidate the reported run or block acceptance.
