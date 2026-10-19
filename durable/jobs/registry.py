"""Supported production jobs; demonstrations are explicitly separate."""
from durable.jobs.coding.workflow import coding, WORKFLOW_VERSION as CODING_VERSION
from durable.jobs.coding.candidates import verification, VERSION as VERIFICATION_VERSION
from durable.jobs.command.workflow import workload, WORKFLOW_VERSION as COMMAND_VERSION

REGISTRY = {"coding": (CODING_VERSION, coding),
            "verification": (VERIFICATION_VERSION, verification),
            "workload": (COMMAND_VERSION, workload)}
