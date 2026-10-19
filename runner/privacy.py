"""Reject private inventory and obvious endpoint/credential bytes in tracked runner files."""
from __future__ import annotations
import ipaddress
from pathlib import Path, PurePosixPath
import re
import subprocess


def check(root='.'):
    paths=subprocess.check_output(['git','ls-files','-z','--','runner'],cwd=root).split(b'\0')
    for raw in paths:
        if not raw:
            continue
        relative=raw.decode()
        if PurePosixPath(relative).name.startswith('hosts.local.'):
            raise ValueError('private runner inventory is tracked; remove it from the index')
        text=(Path(root)/relative).read_text()
        if re.search(r'(?:gh[pousr]_[A-Za-z0-9_]{20,}|github_pat_[A-Za-z0-9_]{20,})',text):
            raise ValueError('credential-shaped bytes in public runner artifact')
        for address in re.findall(r'\b(?:\d{1,3}\.){3}\d{1,3}\b',text):
            parsed=ipaddress.ip_address(address)
            examples=[ipaddress.ip_network(n) for n in ('192.0.2.0/24','198.51.100.0/24','203.0.113.0/24')]
            if not any(parsed in network for network in examples):
                raise ValueError('non-documentation network address in public runner artifact')


if __name__=='__main__':
    check()
