import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('temp_root', HERE / 'temp_root.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class TemporaryRootTests(unittest.TestCase):
    def test_landing_rejects_forbidden_effective_parent_before_git(self):
        import runpy
        from unittest.mock import patch
        namespace = runpy.run_path(str(HERE / 'agents' / 'land.py'))
        with patch.dict(os.environ, TMPDIR=str(Path('/') / 'tmp')):
            with self.assertRaises(ValueError):
                with namespace['temporary_worktree'](Path('/not-a-repository'), 'HEAD', 'land-', True):
                    self.fail('forbidden allocation reached Git')

    def test_direct_harness_initializes_and_rejects_forbidden_root(self):
        code = 'import sys,tempfile; sys.path.insert(0,sys.argv[1]); import tasklib; print(tempfile.mkdtemp())'
        command = [sys.executable, '-c', code, str(module.ROOT / 'bench' / 'harness')]
        env = dict(os.environ)
        env.pop('TMPDIR', None)
        env.pop('TEMP', None)
        env.pop('TMP', None)
        result = subprocess.run(command, env=env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        created = Path(result.stdout.strip())
        self.assertEqual(created.parent, module.ROOT / '.scratch' / 'tmp')
        created.rmdir()
        env['TMPDIR'] = str(Path('/') / 'tmp')
        denied = subprocess.run(command, env=env, capture_output=True, text=True)
        self.assertNotEqual(denied.returncode, 0)
        self.assertIn('outside the system temporary directory', denied.stderr)

    def test_temporal_script_rejects_forbidden_root_before_startup(self):
        command = ['bash', str(module.ROOT / 'crates' / 'team-control-plane' / 'tests' / 'run-temporal.sh')]
        forbidden = str(Path('/') / 'tmp')
        for overrides in ({'TMPDIR': forbidden},
                          {'TEMPORAL_FIXTURE_EVIDENCE': forbidden + '/corenius-forbidden-evidence'}):
            with self.subTest(overrides=overrides):
                environment = dict(os.environ, TMPDIR=str(module.temporary_root()))
                environment.update(overrides)
                result = subprocess.run(command, env=environment,
                                        capture_output=True, text=True, timeout=5)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertIn('outside the system temporary directory', result.stderr)

    def test_ban_scanner_catches_shell_and_quoted_paths(self):
        import runpy
        expression = runpy.run_path(str(HERE / "check_temp_paths.py"))["FORBIDDEN"]
        forbidden = '/' + 'tmp'
        for line in ['tmp_root="${TMPDIR:-' + forbidden + '}"', 'TMPDIR=' + forbidden, "'" + forbidden + "'", '"' + forbidden + '/file"']:
            self.assertIsNotNone(expression.search(line))
        self.assertIsNone(expression.search('/data/devx/tmp/file'))

    def test_rejects_system_root_and_symlink_escape(self):
        with tempfile.TemporaryDirectory(dir=module.ROOT / '.scratch') as directory:
            link = Path(directory) / 'escape'
            link.symlink_to(Path('/') / 'tmp')
            for path in [Path('/') / 'tmp', Path('/') / 'tmp' / 'child', link / 'child']:
                with self.assertRaises(ValueError):
                    module.temporary_root(str(path))

    def test_allocation_and_child_inherit_selected_root(self):
        with tempfile.TemporaryDirectory(dir=module.ROOT / '.scratch') as directory:
            root = Path(directory) / 'owned'
            env = dict(os.environ, TMPDIR=str(root))
            command = [sys.executable, str(HERE / 'temp_root.py'), '--', sys.executable, '-c',
                       'import tempfile,os,json; f=tempfile.NamedTemporaryFile(); '
                       'print(json.dumps([f.name,os.environ["TMPDIR"],os.environ["TEMP"]]))']
            output = subprocess.check_output(command, env=env, text=True)
            name, selected, other = json.loads(output)
            self.assertEqual(Path(name).parent, root)
            self.assertEqual(selected, str(root))
            self.assertEqual(other, str(root))

    def test_rejects_shared_writable_root(self):
        with tempfile.TemporaryDirectory(dir=module.ROOT / '.scratch') as directory:
            Path(directory).chmod(0o777)
            with self.assertRaises(ValueError):
                module.temporary_root(directory)


if __name__ == '__main__':
    (module.ROOT / '.scratch').mkdir(exist_ok=True)
    unittest.main()
