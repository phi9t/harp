"""Explicit host selection; arguments are passed to the remote durable CLI."""
import argparse
import json
from pathlib import Path
from .transport import host, request


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--inventory',type=Path,default=Path('runner/hosts.local.toml'))
    p.add_argument('--host',required=True)
    p.add_argument('arguments',nargs=argparse.REMAINDER)
    a=p.parse_args()
    args=a.arguments[1:] if a.arguments[:1]==['--'] else a.arguments
    response,code=request(host(a.inventory,a.host),args)
    print(json.dumps(response))
    return code

if __name__=='__main__':raise SystemExit(main())
