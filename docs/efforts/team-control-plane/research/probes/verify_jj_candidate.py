#!/usr/bin/env python3
"""Independent disposable-candidate acceptance suite; never reads producer state."""
import argparse
import hashlib
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--expected', choices=['original', 'changed'], required=True)
    args = parser.parse_args()
    expected = {'original': b'verified-original\n', 'changed': b'rewritten-source\n'}[args.expected]
    artifact = Path('artifact.txt').read_bytes()
    assert artifact == expected, 'exported candidate contains unexpected artifact bytes'
    assert Path('.gitignore').read_bytes() == b'private.env\n', 'candidate exclusion rule changed'
    assert not Path('private.env').exists(), 'private content appeared in verifier checkout'
    print(json.dumps({'outcome': 'passed', 'artifact_sha256': hashlib.sha256(artifact).hexdigest(),
                      'expected': args.expected}, sort_keys=True))


if __name__ == '__main__':
    main()
