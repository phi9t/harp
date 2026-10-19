The workspace is a small Blade project with an L3 environment matrix in
`BUILD.share`. The macro reads `FEATURE_MODE` and chooses dependencies and
preprocessor definitions for the app.

The app must build and run in both modes:

```sh
env FEATURE_MODE=compact blade-offline build //:matrix_app
env FEATURE_MODE=wide blade-offline build //:matrix_app
```

The compact run should print `MATRIX_RESULT mode=compact value=32`; the wide
run should print `MATRIX_RESULT mode=wide value=64`. One branch in the matrix is
currently wrong. Fix the matrix; do not hard-code the verifier output in the
app.
