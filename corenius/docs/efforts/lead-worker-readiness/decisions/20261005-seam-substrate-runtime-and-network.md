# Decision: test seam, substrate, worker runtime and network default

Date: 2026-10-05.
Authority: the owner answered the four questions put to him after the spec draft, in the lead session:
"1 yes bench; 2 c; 3 let's actually try to get codex first then traecli (they share pretty much the same
harness design); 4 network almost always (default on)".

## Accepted direction

1. **Test seam.** The benchmark harness is the one seam. Each readiness scenario is a benchmark task or
   condition with a scripted participant and an independent verifier on a disposable repository.
2. **Substrate.** The local controller is the target that must pass the scenarios. The agent toolbox is
   changed only for scenario groups that no other effort covers.
3. **Worker runtime.** Codex workers first, then traecli workers, through the same harness design.
4. **Network.** A worker has network access by default. A launch may turn it off.

## What this does not decide

- Whether a worker may run container builds or write to a shared cache outside its workspace. The owner's
  answer covered network only. Lead's reading, to be confirmed: these stay off by default and are declared
  per launch.
- Whether the network default is enforced or only recorded. The controller spec read at `d7f6748` exposes
  provider connectivity narrowly and nothing else; default-on network differs from that and is raised with
  that effort, not changed here.
- The order in which uncovered scenario groups are built in the toolbox.

## Consequence for existing rules

The toolbox's worker prompt and forbidden-command list today say "No network access" and treat `curl` and
`wget` as violations. Under decision 4 that becomes a per-launch restriction, not the default.

## The questions as put to the owner

Recorded after the independent critique noted that only the answers were on file. The lead's message read:

1. "Test seam. I propose one: the benchmark harness. Each scenario becomes a benchmark task with a scripted
   participant and an independent verifier on a disposable repository."
2. "Substrate. Which component must pass for Corenius to count as ready: (a) the local controller, with this
   effort contributing scenarios and building nothing parallel; (b) the Python toolbox, changed to the rules
   above; or (c) the controller as target, with the toolbox changed only where nothing else covers a
   scenario."
3. "Worker runtime. Whether readiness requires traecli workers, Codex workers, or either."
4. "Access profile. Whether a worker may ever get network, container builds or writes to a shared cache."

Everything in the sections above beyond the owner's quoted words is the lead's elaboration, including the
list of four uncovered groups and the rule that a scenario counts for a runtime only when run with it.
Item 2 is superseded by `20261005-three-instantiations.md`.
