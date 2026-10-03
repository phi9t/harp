"""Owned offline stand-in: two processes including a setsid descendant."""
import json
import os
from pathlib import Path
import socket
import signal
import sys
import time


def report(role):
    denied = False
    try:
        connection=socket.create_connection(('192.0.2.1',443),timeout=.15)
        connection.close()
    except OSError:
        denied=True
    os.write(1,(json.dumps({'role':role,'inner_pid':os.getpid(),'namespace':os.readlink('/proc/self/ns/pid'),
                      'network_namespace':os.readlink('/proc/self/ns/net'),'external_connect_denied':denied,
                      'owner_ledger_visible':Path(sys.argv[1]).exists(),
                      'private_roots_visible':any(Path(p).exists() for p in ('/home','/data02','/etc'))})+'\n').encode())

# Independent finite offline fixture lifetime; PID1 explicitly handles expiry.
signal.signal(signal.SIGALRM,lambda *_:os._exit(124))
signal.alarm(10)
read_ready,write_ready=os.pipe()
child=os.fork()
if child==0:
    os.close(read_ready)
    os.setsid();report('setsid-descendant')
    os.write(write_ready,b'1');os.close(write_ready)
else:
    os.close(write_ready)
    if os.read(read_ready,1)!=b'1':
        raise RuntimeError('descendant setup acknowledgement missing')
    os.close(read_ready)
    if '--noise' in sys.argv:
        # Existing setsid descendant has acknowledged readiness before failure.
        os.write(1,b'owned injected non-JSON setup noise\n')
    report('namespace-init')
while True:time.sleep(.1)
