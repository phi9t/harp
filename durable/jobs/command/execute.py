"""Shared namespace effect seam for a declarative workload projection."""
import json
import os
import sys
from insula.capsule import execute_prepared

if __name__ == '__main__':
    execute_prepared(json.load(sys.stdin), env={'PATH':os.defpath})
