import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import socket
from unittest.mock import patch
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('temp_root', HERE / 'temp_root.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class TemporaryRootTests(unittest.TestCase):
    def test_long_default_supports_nested_unix_socket(self):
        with patch.dict(os.environ, {}, clear=True), patch.object(module, 'ROOT', Path('/' + 'long' * 30)):
            root = module.temporary_root()
        self.assertLessEqual(len(os.fsencode(root)), 70)
        with tempfile.TemporaryDirectory(dir=root, prefix='pymp-') as directory:
            address = str(Path(directory) / ('listener-' + 'x' * 20))
            with socket.socket(socket.AF_UNIX) as stream:
                stream.bind(address)

    def test_inherited_overlong_root_is_rejected_before_allocation(self):
        with tempfile.TemporaryDirectory(dir=module.temporary_root()) as directory:
            root = Path(directory) / ('long-' * 20)
            with patch.dict(os.environ, TMPDIR=str(root)):
                with self.assertRaisesRegex(ValueError, 'Unix socket budget'):
                    module.temporary_root(socket_suffix_bytes=48)
            self.assertFalse(root.exists())

    def test_socket_budget_counts_encoded_bytes(self):
        with tempfile.TemporaryDirectory(dir=module.temporary_root()) as directory:
            root = Path(directory) / ('é' * 30)
            with self.assertRaisesRegex(ValueError, 'Unix socket budget'):
                module.temporary_root(str(root), socket_suffix_bytes=48)

    def test_fallback_refuses_shared_writable_directory(self):
        with tempfile.TemporaryDirectory(dir=Path.home(), prefix='t') as directory:
            fallback = Path(directory) / '.ctmp'
            fallback.mkdir(mode=0o777)
            fallback.chmod(0o777)
            with patch.dict(os.environ, {}, clear=True), patch.object(Path, 'home', return_value=Path(directory)), patch.object(module, 'ROOT', Path('/' + 'long' * 30)):
                with self.assertRaisesRegex(ValueError, 'writable by other users'):
                    module.temporary_root()
            self.assertEqual(fallback.stat().st_mode & 0o777, 0o777)

    def test_complete_socket_budget_binds_at_linux_path_limit(self):
        with tempfile.TemporaryDirectory(dir=module.temporary_root()) as directory:
            length = 59 - len(os.fsencode(directory)) - 1
            root = module.temporary_root(str(Path(directory) / ('r' * length)), socket_suffix_bytes=48)
            address = str(root / ('s' * 47))
            self.assertEqual(len(os.fsencode(address)), 107)
            with socket.socket(socket.AF_UNIX) as stream:
                stream.bind(address)

    def test_safe_long_data_root_is_allowed_but_wrapper_rejects_socket_budget(self):
        with tempfile.TemporaryDirectory(dir=module.temporary_root()) as directory:
            root = Path(directory) / ('data' * 20)
            self.assertEqual(module.temporary_root(str(root)), root)
            result = subprocess.run([sys.executable, str(HERE / 'temp_root.py')],
                                    env=dict(os.environ, TMPDIR=str(root)), capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn('Unix socket budget', result.stderr)

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
        with patch.dict(os.environ, {}, clear=True):
            self.assertEqual(created.parent, module.temporary_root())
        created.rmdir()
        env['TMPDIR'] = str(Path('/') / 'tmp')
        denied = subprocess.run(command, env=env, capture_output=True, text=True)
        self.assertNotEqual(denied.returncode, 0)
        self.assertIn('outside the system temporary directory', denied.stderr)

    def test_ban_scanner_catches_shell_and_quoted_paths(self):
        import runpy
        expression = runpy.run_path(str(HERE / "check_temp_paths.py"))["FORBIDDEN"]
        forbidden = '/' + 'tmp'
        for line in ['tmp_root="${TMPDIR:-' + forbidden + '}"', 'TMPDIR=' + forbidden, "'" + forbidden + "'", '"' + forbidden + '/file"']:
            self.assertIsNotNone(expression.search(line))
        self.assertIsNone(expression.search('/data/devx/tmp/file'))

    def test_rejects_system_root_and_symlink_escape(self):
        with tempfile.TemporaryDirectory(dir=module.temporary_root()) as directory:
            link = Path(directory) / 'escape'
            link.symlink_to(Path('/') / 'tmp')
            for path in [Path('/') / 'tmp', Path('/') / 'tmp' / 'child', link / 'child']:
                with self.assertRaises(ValueError):
                    module.temporary_root(str(path))

    def test_allocation_and_child_inherit_selected_root(self):
        with tempfile.TemporaryDirectory(dir=module.temporary_root()) as directory:
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
        with tempfile.TemporaryDirectory(dir=module.temporary_root()) as directory:
            Path(directory).chmod(0o777)
            with self.assertRaises(ValueError):
                module.temporary_root(directory)


if __name__ == '__main__':
    (module.ROOT / '.scratch').mkdir(exist_ok=True)
    unittest.main()
