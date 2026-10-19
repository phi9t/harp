# Runtime ticket evidence

Original test port red: `python3 tools/temp_root.py -- python3 -m unittest controller.durable.runtime_test -v` exited 1 before runtime implementation: ImportError cannot import Activity from controller.durable.

Original test port green: same unittest command, 35 tests passed in 12.380s.

Uncertain effect regressions red: `python3 tools/temp_root.py -- python3 -m unittest controller.durable.runtime_test.RuntimeTests.test_cancelled_started_manual_effect_requires_resolution controller.durable.runtime_test.RuntimeTests.test_failed_sibling_started_manual_effect_requires_resolution -v` ran 2 tests in 0.578s, exited 1: expected BLOCKED, observed CANCELLED and FAILED respectively.

Uncertain effect regressions green: same focused command, 2 tests passed in 0.605s.
Partial resolution red: `python3 tools/temp_root.py -- python3 -m unittest controller.durable.runtime_test.RuntimeTests.test_resolving_one_effect_preserves_other_uncertainty -v`, 1 test in 0.179s, exited 1: observed RUNNING instead of BLOCKED after resolving only one of two uncertain manual effects.

Partial resolution green: focused partial resolution and sibling tests, 2 passed in 0.609s.
Receipt type regression red: `python3 tools/temp_root.py -- python3 -m unittest controller.durable.runtime_test.RuntimeTests.test_invalid_receipt_types_block_recovery -v`, 1 test / 6 independent subcases in 1.449s, exited 1: five uncontrolled AttributeError/TypeError/ProgrammingError/KeyError exceptions; protocol=true incorrectly accepted.

Receipt types green: same focused command, six subcases passed in 1.516s.
Temporary environment red: focused test_activity_temporary_environment_uses_approved_host_root, test_activity_rejects_managed_temporary_overrides, test_activity_spec_does_not_depend_on_host_temporary_root, 3 tests in 0.191s, exited 1: payload lost all five temp variables; all managed overrides accepted. Spec independence already passed.

Temporary environment green: same focused 3 tests passed in 0.201s.
Registry CLI red: `python3 tools/temp_root.py -- python3 -m unittest controller.durable.runtime_test.RuntimeTests.test_cli_loads_registry_and_submits_its_workflow_version -v`, 1 test in 0.071s, exited 1: CLI rejected --registry with status 2.

Registry CLI green: same focused command, 1 test passed in 0.235s.

Final verification:
- `python3 tools/temp_root.py -- python3 -m unittest controller.durable.runtime_test -v`: 43 tests passed in 14.828s, exit 0 (original 35 plus 8 public behavioral regressions).
- `python3 tools/temp_root.py -- python3 -m compileall -q controller/durable`: exit 0.
- `python3 tools/temp_root.py -- rg -n 'sys.path|insula_durable|/tmp|TemporaryDirectory|guard.py|DEMO_VERSION' controller/durable`: only intended demo-version references and explicit approved TemporaryDirectory(dir=temporary_root()); no sys.path hacks, legacy namespace, /tmp allocation or file guardian launch.

Focused temporary regression command: `python3 tools/temp_root.py -- python3 -m unittest controller.durable.runtime_test.RuntimeTests.test_activity_temporary_environment_uses_approved_host_root controller.durable.runtime_test.RuntimeTests.test_activity_rejects_managed_temporary_overrides controller.durable.runtime_test.RuntimeTests.test_activity_spec_does_not_depend_on_host_temporary_root -v`.
Focused partial-resolution green command: `python3 tools/temp_root.py -- python3 -m unittest controller.durable.runtime_test.RuntimeTests.test_resolving_one_effect_preserves_other_uncertainty controller.durable.runtime_test.RuntimeTests.test_failed_sibling_started_manual_effect_requires_resolution -v`.

Compared common, engine, guard, workflows and CLI against supplied reference with difflib; runtime changes are scoped to approved launch/temp handling, registry loading, uncertain effects and receipt validation. No staging, commits, pushes, cgroup edits or parent-worktree edits. Parent owns Insula/cgroup glue and broader repo gates. Capsule mapping must replace host temporaries with /run/corenius-tmp and include tools/temp_root.py in immutable projection.
