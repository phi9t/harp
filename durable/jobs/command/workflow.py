"""Workload execution uses the existing guardian, journal and recovery policy."""
from durable.runtime.engine import Activity
from durable.jobs.command.admission import WORKFLOW_VERSION


def workload(context, args):
    result = context.activity('workload', Activity(
        argv=('/usr/bin/python3', '-m', 'durable.jobs.command.worker'),
        inputs=args['inputs'], cwd='/workspace', workspace=args['workspace'],
        input={'identity':args['identity']},
        env={'HOME':'/home/bench/run/home'}, timeout=args['timeout'],
        recovery='manual', max_attempts=1, offline=True))
    if result.get('passed') is not True:
        raise ValueError('workload acceptance checks failed')
    return result
