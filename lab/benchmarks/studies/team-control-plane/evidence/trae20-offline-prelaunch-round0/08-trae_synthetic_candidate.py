#!/usr/bin/env python3
"""Preparation entry: validates/renders public proposal; runtime always refused."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tomllib


def configuration():
    path=Path(__file__).with_suffix('.toml')
    config=tomllib.loads(path.read_text())
    provider=config['model_providers']['corenius-synthetic']
    if config['model_provider']!='corenius-synthetic' or config['model']!='gpt-5.5':
        raise ValueError('frozen provider/model mismatch')
    if provider['request_max_retries']!=0 or provider['stream_max_retries']!=0 or provider['requires_openai_auth'] or provider['supports_websockets']:
        raise ValueError('unreviewed transport/retry proposal')
    if provider['base_url']!='http://127.0.0.1:1234/v1' or provider['wire_api']!='responses' or provider['request_overrides']['extra_body']['max_output_tokens']!=32:
        raise ValueError('unreviewed route/cap proposal')
    return config


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--render-config',action='store_true')
    parser.add_argument('--offline-check',action='store_true')
    parser.add_argument('--launch-trae',action='store_true')
    args=parser.parse_args()
    if args.launch_trae:
        print('REFUSED: runtime admission withheld; this preparation entry has no Trae launcher.',file=sys.stderr)
        return 78
    config=configuration()
    if args.render_config:
        print(json.dumps({'status':'unverified_pinned_binary_hypothesis','configuration':config},indent=2))
    if args.offline_check:
        result=subprocess.run([sys.executable,'-m','unittest','test_trae_round','test_trae_transport','test_trae_synthetic_guard','test_trae_candidate'],cwd=Path(__file__).parent,timeout=30)
        return result.returncode
    if not args.render_config:
        parser.print_help()
    return 0

if __name__=='__main__':sys.exit(main())
