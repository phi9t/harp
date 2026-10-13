#!/usr/bin/env python3
"""Zero-model external runtime: EVERY mutation invocation is non-idempotent."""
import json
import os
import pathlib
import sys
import uuid

mode, directory, operation, kind, thread = sys.argv[1:]
root = pathlib.Path(directory)


def durable_record(path, value):
    with path.open('x') as record:
        json.dump(value, record)
        record.flush()
        os.fsync(record.fileno())
    directory_fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory_fd)
    finally:
        os.close(directory_fd)


if mode in ('execute', 'execute_fail_known'):
    invocation = str(uuid.uuid4())
    durable_record(root / (invocation + '.invocation'),
                   {'invocation_id': invocation, 'operation_id': operation, 'kind': kind})
    if mode == 'execute_fail_known':
        print(json.dumps({'known_failure': 'fixture rejected before effect'}))
    else:
        effect = {'effect_id': invocation, 'operation_id': operation,
                  'thread_id': thread or ('thread-' + invocation)}
        durable_record(root / (invocation + '.json'), effect)
        print(json.dumps(effect))
else:
    effects = [json.loads(path.read_text()) for path in root.glob('*.json')]
    matches = [effect for effect in effects if effect['operation_id'] == operation]
    if len(matches) == 1:
        print(json.dumps(matches[0]))
    elif matches:
        print(json.dumps({'multiple_effect_ids': [effect['effect_id'] for effect in matches]}))
    else:
        print(json.dumps({'inventory_id': 'fixture-complete-' + operation}))
