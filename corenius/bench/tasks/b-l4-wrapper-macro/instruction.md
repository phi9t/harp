The workspace is a Blade project with a wrapper macro in `BUILD.rules`.
Packages under `feature/` use that wrapper for six libraries, and the verifier
also has a protected macro consumer under `tests/`.

`blade-offline build //...` currently fails because the wrapper mishandles a
target option. Fix the wrapper once so every dependent target builds and the app
prints:

```text
WRAPPER_RESULT total=75
```

Do not patch only the visible feature packages; protected verifier targets use
the same wrapper.
