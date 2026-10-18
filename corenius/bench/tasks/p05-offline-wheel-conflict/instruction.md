Fix the offline Python project so `uv run --offline --locked python -m unittest discover -s tests` succeeds with only the wheels already checked into `vendor/`.

The project must install `shared-math`, `alpha-calc`, and `beta-calc` from local wheel files. `alpha_score(value)` and `beta_score(value)` must both use the `shared_math.scale(value)` behavior from `shared-math` 1.0.0, so the expected results are `value * 10 + 1` and `value * 10 + 2`.

Do not download packages, do not remove the tests, and keep the lockfile consistent with the local wheel pins.
