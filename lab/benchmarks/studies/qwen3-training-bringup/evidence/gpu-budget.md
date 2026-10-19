# Approved GPU budget

Total ceiling: 3600 GPU-seconds. Workloads execute serially, at most two GPUs.
Each GPU attempt has a wall-clock timeout no greater than 900 seconds.

Initial allocation: CPT lane 1800 GPU-seconds, RL lane 1800 GPU-seconds.
Parent can transfer unused allocation without exceeding the total approved cap.
Include GPU preflights and failed runs. Reconciled attempts are recorded below.

Agent-owned native receipts supply exact process start/end and selected devices;
append reconciled accounting here before completion. CPU-only preparation is not
GPU consumption. Device occupancy is checked before each launch.

## Execution ownership

- CPT implementer: Kata `mkqk`, source worktree
  `/data02/home/philip.yang/workspace/torchtitan/.worktrees/qwen3-cpt-insula`.
- RL implementer: Kata `eb6b`, source worktree
  `/data02/home/philip.yang/workspace/verl.worktrees/eb6b-qwen3-grpo`.
- Parent owns aggregate accounting, GPU-lane handoff and independent review.
- Selected devices must be checked for competing workloads before every attempt;
  no process is displaced. CPU-only qualification has no device projection.

## CPT reservation

Granted devices 0 and 1 after recorded prelaunch snapshots showed no compute jobs
or allocated memory on either; GPU4 has an unrelated workload and is excluded.
Reservation: up to 1660 GPU-seconds (two 400-second training/resume attempts on
2 GPUs with up to 10-second cleanup each, plus a <=10-second 2-GPU preflight).
This fits CPT's 1800 GPU-second lane. Reconcile actual receipts after each run;
RL remains CPU-only until CPT releases the lane.

## CPT first attempt

Reported native exit0, 80.157413 seconds on 2 GPUs = 160.314826 GPU-seconds.
Step10 full DCP completed; GPU process groups destroyed. Aggregate consumed:
160.314826 GPU-seconds. Approved ceiling remaining: 3439.685174 GPU-seconds.
Resume reservation <=820 GPU-seconds including termination grace, contingent
on the recorded device0/1 free check.

## CPT resume and lane handoff

Native resume exit0:68.10803915 seconds x2 =136.21607830 GPU-seconds.
CPT total:296.53090452 GPU-seconds (0.08237 GPU-hours). Both native attempts
exited successfully and released GPUs0/1, confirmed by the cleanup snapshot.
Aggregate remaining approved ceiling:3303.46909548 GPU-seconds.
RL receives its initial1800 GPU-second allocation; additional remaining budget
can be reassigned by parent if justified, never beyond the aggregate ceiling.

## RL planned reservations

Implementer reserves GPU0 preflight <=400 seconds including20-second cleanup,
then initial and resume attempts <=700 seconds each including cleanup.
Total reservations <=1800 GPU-seconds. Native free-device recheck occurs
immediately before GPU exposure. Count actual elapsed wall time for all failures.

## RL preflight 1 and budget reallocation

Preflight1 failed after56 reported GPU-envelope seconds: native TorchTitan
pretrained forward/backward and parameter-change checks passed; SGLang could not
find a usable nvidia-smi. Narrow driver-interface projection is being repaired.
Global consumed:352.53090452 GPU-seconds. Remaining:3247.46909548 GPU-seconds.
Unused CPT allocation released; RL total ceiling becomes3303.46909548 GPU-seconds,
including its first56 seconds. This is a reallocation within the existing one
GPU-hour approval, not an increase. Per-attempt cap remains900 seconds including
cleanup, with current preflight400/train700/resume700 planned envelopes.

## RL preflight 2

Preflight2 failed after54 GPU-envelope seconds, reconstructed from launch argv
and log timestamps after a SGLang child SIGQUIT terminated the outer shell.
TorchTitan strict pretrained load and parameter-update checks passed again.
SGLang initialization failed in optional DeepGEMM because nvcc was unavailable.
The retry will isolate the child process group for reliable exit accounting and
use the source-supported dense-model configuration disabling JIT DeepGEMM.
RL consumed:110 GPU-seconds. Global consumed:406.53090452 GPU-seconds.
Remaining approved aggregate budget:3193.46909548 GPU-seconds.

## RL preflight 3

Native preflight.gpu-envelope-seconds reports69 seconds; SGLang optional
DeepGEMM was disabled, but a core JIT kernel still required nvcc. No combined
GRPO update has occurred. A read-only, content-hashed CUDA toolkit projection
is the next prerequisite. RL consumed:179 GPU-seconds. Global consumed:
475.53090452 GPU-seconds. Remaining:3124.46909548 GPU-seconds.

## RL preflight 4

Succeeded in109 GPU-envelope seconds with the hashed, read-only CUDA13.2
toolkit projection: native pretrained TorchTitan forward/backward updated a
parameter; SGLang generated text and acknowledged a disk weight reload.
This is isolated qualification, not yet a combined GRPO success.
RL consumed:288 GPU-seconds. Global consumed:584.53090452 GPU-seconds.
Remaining:3015.46909548 GPU-seconds.

## RL combined train 1

Failed in108 GPU-envelope seconds before SGLang server startup: verl used
dataclasses.fields on installed SGLang's migrated ServerArgs schema.
No combined update completed. A CPU schema compatibility test precedes retry.
RL consumed:396 GPU-seconds. Global consumed:692.53090452 GPU-seconds.
Remaining:2907.46909548 GPU-seconds.

## RL combined train 2

Failed in108 GPU-envelope seconds after repaired SGLang launch adapters:
its memory-saver preload subprocess could not resolve libcudart.so.13.
A CPU subprocess reproduced failure and passed with the already sealed toolkit's
lib64 directory on LD_LIBRARY_PATH. No combined update completed.
RL consumed:504 GPU-seconds. Global consumed:800.53090452 GPU-seconds.
Remaining:2799.46909548 GPU-seconds. Train3 reserved <=700 seconds.

## RL combined train 3

Failed in155 GPU-envelope seconds after server initialization, at first weight
transfer: installed SGLang Engine now requires begin_weight_update and
end_weight_update implementations. Full HTTP session protocol and failure
handling are being qualified on CPU before retry. No GRPO update completed.
RL consumed:659 GPU-seconds. Global consumed:955.53090452 GPU-seconds.
Remaining:2644.46909548 GPU-seconds.

## RL combined train 4: successful initial run

Succeeded in312 GPU-envelope seconds. Two genuine GRPO updates completed:
nonzero advantages/gradients, changing exported policy hashes, successful
SGLang begin/bucket/end/flush acknowledgements, then validation generation.
Full checkpoint2 exists; contents and fresh-process resume remain to verify.
RL consumed:971 GPU-seconds. Global consumed:1267.53090452 GPU-seconds.
Remaining:2332.46909548 GPU-seconds. Resume reserved <=700 seconds.

## RL resume: successful final run

Succeeded in259 GPU-envelope seconds. Fresh processes restored checkpoint2;
its exported policy digest matched the saved step2 digest and was acknowledged
by SGLang before resumed rollout. Updates3–4 completed, checkpoint4 was saved,
and GPU0 was released. Independent final evidence review follows.
RL total:1230 GPU-seconds. Aggregate total:1526.53090452 GPU-seconds
(0.42403636 GPU-hours). Unused approved budget:2073.46909548 GPU-seconds.
No further GPU launches planned. All attempts were serial and within the
per-attempt cap; failed GPU preflights and train launches are included.
