import math

import ops


def reference(values, scale, bias):
    return [x * scale + bias for x in values]


def assert_close(actual, expected):
    if len(actual) != len(expected):
        raise AssertionError(f"length mismatch: {actual} != {expected}")
    for got, want in zip(actual, expected):
        if not math.isclose(got, want, rel_tol=1e-6, abs_tol=1e-6):
            raise AssertionError(f"value mismatch: {actual} != {expected}")


def main():
    cases = [
        ([1.0, 2.0, -3.5], 2.0, 0.5),
        ([0.25, -4.0, 8.0, 16.0], -1.5, 3.0),
        ([3.0, 4.0], 0.0, -2.25),
        ([], 7.0, 11.0),
    ]
    for values, scale, bias in cases:
        actual = ops.fused_scale_add(values, scale, bias)
        assert_close(actual, reference(values, scale, bias))
    print("FUSED_OP_OK cases=%d" % len(cases))


if __name__ == "__main__":
    main()
