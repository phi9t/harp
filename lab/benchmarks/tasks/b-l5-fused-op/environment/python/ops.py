import ctypes
from pathlib import Path


_ROOT = Path(__file__).resolve().parents[1]
_PLUGIN = _ROOT / "build64_release" / "csrc" / "libfused_plugin.so"


class Params(ctypes.Structure):
    _fields_ = [("scale", ctypes.c_float)]


def _entry():
    lib = ctypes.CDLL(str(_PLUGIN))
    func = lib.fused_op_apply
    func.argtypes = [
        ctypes.POINTER(Params),
        ctypes.POINTER(ctypes.c_float),
        ctypes.POINTER(ctypes.c_float),
        ctypes.c_int,
    ]
    func.restype = ctypes.c_int
    return func


def fused_scale_add(values, scale):
    n = len(values)
    input_array = (ctypes.c_float * n)(*values)
    output_array = (ctypes.c_float * n)()
    params = Params(float(scale))
    rc = _entry()(ctypes.byref(params), input_array, output_array, n)
    if rc:
        raise RuntimeError(f"fused_op_apply failed: {rc}")
    return [float(output_array[i]) for i in range(n)]
