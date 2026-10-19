"""Typed runtime failures shared by the journal and replay engine."""
class Park(BaseException):
    """Internal cooperative suspension; workflow code must not catch BaseException."""


class ReplayMismatch(RuntimeError):
    pass


class ActivityFailed(RuntimeError):
    pass


class ControllerBusy(RuntimeError):
    pass


class CleanupUnproven(RuntimeError):
    pass
