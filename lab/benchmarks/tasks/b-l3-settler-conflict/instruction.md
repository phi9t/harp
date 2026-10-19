The workspace is a small Blade project that models an L3 settler problem.

`BUILD.share` applies the package's default library-settler policy. The root
`BUILD` currently adds a conflicting policy for `//lib:chooser`, and
`blade-offline build //:settler_app` fails in a Blade-visible way. Repair the
settler configuration so the app builds and prints:

```text
SETTLER_RESULT value=7
```

Build with:

```sh
blade-offline build //...
```

Do not remove the library implementation or bypass the target graph. The fix
should be in the settler configuration.
