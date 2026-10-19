"""Adapt Soma's profile to its already admitted rootfs and guardian ownership.

Nested namespaces project the existing admitted /, never another host rootfs.
All subprocesses remain members of the outer activity's guardian-owned cgroup.
"""
from __future__ import annotations
import importlib.util
import json
import gzip
import os
from pathlib import Path
import shutil
import subprocess
import sys

SUITES = {'magit':('magit-selective-staging',7), 'comint':('comint-process',11),
          'jupyter':('jupyter-existing-kernel',8), 'lsp':('lsp-source',6),
          'org':('org-structure',6), 'dired':('dired-filesystem',8),
          'bridge':('bridge-transport',30), 'bridge-daemon':('bridge-daemon-transport',63)}


def main():
    if os.environ.get('HARP_WORKLOAD_PROFILE') != 'native' or not os.environ.get('INSULA_DURABLE_CONTRACT'):
        raise ValueError('Soma requires an admitted durable selected-rootfs activity')
    source = Path(os.environ['HARP_WORKLOAD_SOURCE']) / 'lab/soma'
    output = Path(os.environ['HARP_WORKLOAD_OUTPUT'])
    contract_path = Path('/etc/soma-insula-rootfs-contract.json')
    contract = json.loads(contract_path.read_text())
    if contract.get('dependency_closure') != 'rootfs_owned':
        raise ValueError('Soma dependencies must belong to the admitted rootfs')
    if not Path('/usr/bin/bwrap').is_file():
        raise ValueError('nested workload projection requires admitted /usr/bin/bwrap')
    spec = importlib.util.spec_from_file_location('soma_workload_profile', source/'scripts/insula/soma_insula.py')
    adapter = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = adapter
    spec.loader.exec_module(adapter)
    root = source / '.qualification' / 'durable'
    root.mkdir(parents=True)
    selected = adapter.SelectedRootfs('admitted-rootfs',Path('/'),Path('/'),contract_path,contract)
    network = adapter.authorize_network('disabled','durable-soma',None)
    for name, (benchmark, expected) in SUITES.items():
        out = root / name
        out.mkdir()
        runtime = root / ('runtime-'+name)
        runtime.mkdir()
        (runtime/'eln-cache').mkdir()
        sandbox_out = '/workspace/soma/.qualification/durable/'+name
        paths = adapter.RunPaths(source,out,sandbox_out,out/'insula-plan.json',sandbox_out+'/insula-plan.json',runtime)
        daemon = name == 'bridge-daemon'
        jupyter = name == 'jupyter' or daemon
        plan = adapter.common_plan(selected,paths,network,benchmark,
                                   private_jupyter_runtime=jupyter,daemon_bridge_runtime=daemon)
        plan['durable_execution_contract'] = os.environ['INSULA_DURABLE_CONTRACT']
        adapter.write_json(paths.plan_host,plan)
        environment = adapter.environment(paths,private_jupyter_runtime=jupyter,daemon_bridge_runtime=daemon)
        mounts = adapter.mount_args(selected,paths,network,private_jupyter_runtime=jupyter)
        # The admitted code is mounted at /workspace. Replace that mount only in
        # this nested namespace; its source remains available to the outer worker.
        bind_index = len(mounts)-3
        mounts[bind_index:bind_index] = ['--tmpfs','/workspace']
        if daemon:
            command = ['/bin/bash','/workspace/soma/benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark-in-rootfs.sh',sandbox_out]
        else:
            command = ['/opt/soma/emacs/bin/emacs','--quick','--batch','-l',
                       '/workspace/soma/benchmarks/emacs-gym/'+name+'/elisp/run-benchmark.el','--',sandbox_out]
        invocation = adapter.PreparedInvocation(selected,network,paths,plan,mounts,environment,command)
        result = subprocess.run([sys.executable,'/workspace/durable/workloads/execute.py'],
                                input=json.dumps(invocation.argv('/usr/bin/bwrap')),text=True)
        if result.returncode:
            raise ValueError('Soma suite failed: '+name)
        (output/(name+'.json.gz')).write_bytes(gzip.compress((out/'summary.json').read_bytes(),mtime=0))
    (output/'summary.json').write_text(json.dumps({'suite_names':list(SUITES),'source_profile':'soma',
        'execution_contract':os.environ['INSULA_DURABLE_CONTRACT']},sort_keys=True)+'\n')


if __name__ == '__main__':
    main()
