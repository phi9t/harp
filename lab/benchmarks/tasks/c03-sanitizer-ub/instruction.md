Fix `count_above_limit(values, limit)` so it counts exactly the elements in `values` that are strictly greater than `limit`.

The function must return zero for an empty vector, must not inspect memory outside the vector, and must preserve duplicate values in the count. The verifier builds with AddressSanitizer and UndefinedBehaviorSanitizer enabled.

The project uses CMake and Ninja. Do not remove the tests.
