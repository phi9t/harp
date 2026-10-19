Fix the CMake project so the `report_test` executable builds and the report behavior is correct.

`render_report(values)` must return `items=<count> total=<sum> average=<mean>` where the mean has exactly two decimal places. Empty input must report `items=0 total=0 average=0.00`. Negative values count toward the total normally.

The project uses CMake and Ninja. Do not remove the tests.
