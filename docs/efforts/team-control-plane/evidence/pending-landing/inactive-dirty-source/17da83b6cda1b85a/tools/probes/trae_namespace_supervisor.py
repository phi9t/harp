"""Offline stand-in supervision; arbitrary executable/runtime entry is absent."""
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import time


def nspid_values(status, expected_pid=None):
    lines=[line for line in status.splitlines() if line.startswith('NSpid:')]
    if len(lines)!=1:
        raise RuntimeError('missing or ambiguous readable NSpid; inventory uncertain')
    values=lines[0].split()[1:]
    if not values or any(not value.isascii() or not value.isdigit() or int(value)<=0 for value in values):
        raise RuntimeError('invalid readable NSpid; inventory uncertain')
    if expected_pid is not None and int(values[0])!=expected_pid:
        raise RuntimeError('host-proc NSpid frame mismatch')
    return list(map(int,values))


class ProcessFence:
    """Pins recorded namespace identity; pidfd signaling never adopts a PID."""
    def __init__(self, anchor):
        self.anchor=dict(anchor)
        self.namespace_fd=None
        path=Path('/proc')/str(anchor['pid'])/'ns/pid'
        descriptor=os.open(path,os.O_RDONLY|os.O_CLOEXEC)
        try:
            pinned=os.fstat(descriptor)
            current=path.stat()
            if (pinned.st_dev,pinned.st_ino)!=(current.st_dev,current.st_ino) or anchor['namespace']!='pid:['+str(pinned.st_ino)+']' or NamespaceSupervisor.identity(anchor['pid'])!=anchor:
                raise RuntimeError('namespace pin identity mismatch; refused')
            self.namespace_fd=descriptor
            self.device=pinned.st_dev
            self.inode=pinned.st_ino
        except BaseException:
            os.close(descriptor)
            raise

    def validate(self):
        if self.namespace_fd is None:
            raise RuntimeError('namespace pin closed; inventory uncertain')
        pinned=os.fstat(self.namespace_fd)
        if (pinned.st_dev,pinned.st_ino)!=(self.device,self.inode):
            raise RuntimeError('namespace pin changed; inventory uncertain')

    def signal_recorded(self, expected):
        self.validate()
        if expected!=self.anchor:
            raise RuntimeError('recorded signal identity mismatch; refused')
        if not hasattr(os,'pidfd_open') or not hasattr(signal,'pidfd_send_signal'):
            raise RuntimeError('pidfd capability unavailable; quarantine')
        try:
            descriptor=os.pidfd_open(expected['pid'],0)
        except ProcessLookupError:
            return 'recorded_process_disappeared'
        try:
            # Recheck AFTER pidfd_open: a reused numeric PID is never adopted.
            try:
                current=NamespaceSupervisor.identity(expected['pid'])
            except (FileNotFoundError,ProcessLookupError):
                return 'recorded_process_disappeared'
            if current!=expected:
                raise RuntimeError('post-pidfd identity mismatch; refused')
            signal.pidfd_send_signal(descriptor,signal.SIGKILL,None,0)
            return 'pidfd_SIGKILL'
        finally:
            os.close(descriptor)

    def close(self):
        if self.namespace_fd is not None:
            os.close(self.namespace_fd)
            self.namespace_fd=None


class NamespaceSetupError(RuntimeError):
    def __init__(self, proof):
        super().__init__('offline namespace setup failed; '+proof['reason'])
        self.proof=proof


class NamespaceSupervisor:
    def __init__(self, evidence=None):
        self.process=None
        self.namespace=None
        self.anchor=None
        self.fence=None
        self.observations=[]
        self.evidence=evidence
        self.setup_failure=None
        self.shutdown=None

    def retain(self, stage, **values):
        if self.evidence is None:
            return
        record=dict(stage=stage,launcher=self.process.pid if self.process else None,
                    namespace=self.namespace,anchor=self.anchor,**values)
        fd=os.open(self.evidence,os.O_WRONLY|os.O_CREAT|os.O_APPEND|os.O_NOFOLLOW,0o600)
        try:
            data=(json.dumps(record,sort_keys=True)+'\n').encode()
            if os.write(fd,data)!=len(data):
                raise OSError('short public evidence write')
            os.fsync(fd)
        finally:
            os.close(fd)

    @staticmethod
    def identity(pid, proc_root=Path('/proc')):
        directory=proc_root/str(pid)
        namespace=os.readlink(directory/'ns/pid')
        fields=(directory/'stat').read_text().rsplit(')',1)[1].split()
        values=nspid_values((directory/'status').read_text(),expected_pid=pid)
        return {'pid':pid,'start_ticks':int(fields[19]),
                'namespace':namespace,'inner_pid':values[-1]}

    @staticmethod
    def host_frame(proc_root=Path('/proc')):
        values=nspid_values((proc_root/'self/status').read_text(),expected_pid=os.getpid())
        if len(values)!=1:
            raise RuntimeError('observer requires readable host-proc NSpid frame')

    @classmethod
    def members(cls, namespace, proc_root=Path('/proc')):
        cls.host_frame(proc_root)
        found=[]
        for directory in proc_root.iterdir():
            if not directory.name.isdigit():
                continue
            try:
                # Every fixed capsule descendant retains this host UID.
                if directory.stat().st_uid!=os.getuid():
                    continue
                values=nspid_values((directory/'status').read_text(),expected_pid=int(directory.name))
                # A single component in this validated host /proc frame proves
                # this task is outside every child namespace. Read BEFORE ns/pid.
                if len(values)==1:
                    continue
                identity=cls.identity(int(directory.name),proc_root)
                if identity['namespace']==namespace:
                    found.append(identity)
            except (FileNotFoundError,ProcessLookupError):
                # Disappearance is distinct from unavailable namespace identity.
                continue
        return sorted(found,key=lambda member:member['pid'])

    def _establish(self, deadline):
        host_namespace=os.readlink('/proc/self/ns/pid')
        while self.anchor is None and time.monotonic()<deadline:
            pending=[self.process.pid]
            while pending:
                pid=pending.pop()
                try:
                    identity=self.identity(pid)
                    if identity['namespace']!=host_namespace and identity['inner_pid']==1:
                        self.anchor=identity
                        self.namespace=identity['namespace']
                        self.fence=ProcessFence(identity)
                        return
                    children=Path('/proc/'+str(pid)+'/task/'+str(pid)+'/children').read_text().split()
                    pending.extend(map(int,children))
                except FileNotFoundError:
                    continue
            time.sleep(.01)
        raise RuntimeError('owned namespace init identity unavailable')

    def start_fixture(self, owner_ledger, noise=False):
        """Only an offline fixed stand-in; noise exercises setup cleanup later."""
        if self.process is not None:
            raise RuntimeError('supervisor incarnation already spent')
        fixture=Path(__file__).with_name('trae_namespace_fixture.py').resolve()
        command=['/usr/bin/bwrap','--unshare-all','--die-with-parent','--new-session','--as-pid-1',
                 '--ro-bind','/usr','/usr','--ro-bind','/lib','/lib','--ro-bind','/lib64','/lib64',
                 '--proc','/proc','--dev','/dev','--tmpfs','/tmp','--dir','/fixture',
                 '--ro-bind',str(fixture),'/fixture/standin.py','--clearenv',
                 '--setenv','PATH','/usr/bin','--setenv','PYTHONDONTWRITEBYTECODE','1',
                 '/usr/bin/python3','-u','/fixture/standin.py',str(owner_ledger)]
        if noise:
            command.append('--noise')
        self.host_frame()
        self.process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
        # Every operation after Popen, including retention/parsing, is protected.
        try:
            deadline=time.monotonic()+5
            self._establish(deadline)
            self.retain('namespace_owned',inventory=self.inventory())
            buffer=b''
            while len(self.observations)<2 and time.monotonic()<deadline:
                if select.select([self.process.stdout],[],[],.1)[0]:
                    chunk=os.read(self.process.stdout.fileno(),65536)
                    if not chunk:
                        break
                    buffer+=chunk
                    if len(buffer)>65536:
                        raise RuntimeError('bounded setup output exceeded')
                    while b'\n' in buffer:
                        line,buffer=buffer.split(b'\n',1)
                        self.observations.append(json.loads(line))
            if len(self.observations)!=2:
                raise RuntimeError('two stand-in identities not observed')
            if any(record['namespace']!=self.namespace for record in self.observations):
                raise RuntimeError('namespace observation mismatch')
            self.retain('fixture_ready',observations=self.observations)
            return self.observations
        except BaseException as error:
            # Cleanup runs even if the public evidence sink itself has failed.
            proof={'reason':type(error).__name__+': '+str(error),'namespace':self.namespace,
                   'anchor':self.anchor,'launcher':self.process.pid,'observations':self.observations}
            try:
                proof['shutdown']=self.stop()
            except BaseException as cleanup_error:
                proof['cleanup_uncertain']=type(cleanup_error).__name__+': '+str(cleanup_error)
            self.setup_failure=proof
            try:
                self.retain('setup_failed',proof=proof)
            except BaseException as retention_error:
                proof['retention_uncertain']=type(retention_error).__name__+': '+str(retention_error)
            raise NamespaceSetupError(proof) from error

    def inventory(self):
        if not self.namespace:
            raise RuntimeError('no established owned namespace')
        if self.fence is None:
            raise RuntimeError('validated namespace pin unavailable')
        self.fence.validate()
        return self.members(self.namespace)

    def stop(self):
        if self.shutdown is not None:
            return self.shutdown
        before=None
        inventory_error=None
        try:
            before=self.inventory()
        except BaseException as error:
            inventory_error=type(error).__name__+': '+str(error)
        physical_error=None
        signal_outcome=None
        if self.anchor is not None:
            try:
                if self.fence is None:
                    self.fence=ProcessFence(self.anchor)
                signal_outcome=self.fence.signal_recorded(self.anchor)
            except (FileNotFoundError,ProcessLookupError):
                signal_outcome='recorded_process_disappeared'
            except BaseException as error:
                physical_error=type(error).__name__+': '+str(error)
                if self.process.poll() is None:
                    self.process.kill()  # Owned unreaped Popen child only; not init adoption.
        elif self.process is not None and self.process.poll() is None:
            self.process.kill()
        deadline=time.monotonic()+2
        after=None
        while time.monotonic()<deadline and self.namespace:
            try:
                after=self.inventory()
                if not after:
                    break
            except BaseException as error:
                inventory_error=type(error).__name__+': '+str(error)
                break
            time.sleep(.01)
        launcher_terminal=True
        try:
            self.process.wait(timeout=max(.01,deadline-time.monotonic()))
        except subprocess.TimeoutExpired:
            launcher_terminal=False
        finally:
            self.process.stdout.close()
            self.process.stderr.close()
        proof={'exit':self.process.returncode,'namespace':self.namespace,'anchor':self.anchor,
               'inventory_before':before,'inventory_after':after,'launcher_terminal':launcher_terminal,
               'quarantined':not self.namespace or after!=[] or not launcher_terminal or physical_error is not None,
               'physical_error':physical_error,
               'inventory_error':inventory_error,'physical_stop':signal_outcome or 'owned launcher only; namespace unknown',
               'namespace_pin':{'device':self.fence.device,'inode':self.fence.inode} if self.fence else None}
        self.shutdown=proof
        try:
            self.retain('stopped',proof=proof)
        except (OSError,ValueError) as error:
            proof['retention_uncertain']=str(error)
        finally:
            if self.fence is not None:
                self.fence.close()
        return proof

    def wait_terminal_then_stop(self, terminal, timeout):
        deadline=time.monotonic()+timeout
        observed=None
        while time.monotonic()<deadline:
            observed=terminal()
            if observed is not None:
                break
            time.sleep(.01)
        shutdown=self.stop()
        return {'terminal':observed,'terminal_unknown':observed is None,'shutdown':shutdown}
