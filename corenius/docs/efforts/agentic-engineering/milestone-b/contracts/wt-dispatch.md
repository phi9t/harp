# Contract: `wt.py` subcommands and the `land` plug-in (F5, F6)

`tools/agents/wt.py` (ticket 01) owns `new`, `status`, `finish`, `gc`. `tools/agents/land.py` (ticket 02) owns `land`. They are separate files
written by different workers, so the join is fixed here.

**wt.py side.** `main(argv=None) -> int` builds an `argparse` parser with subparsers `new`, `status`, `finish`, `gc`, then calls
`_load_plugins(subparsers)`, which does `importlib.import_module("land")` (the directory `tools/agents` is on `sys.path` because `wt.py` inserts
its own directory), ignores the failure **only** when the missing module is `land` itself (`ModuleNotFoundError` with `exc.name == "land"`), and
otherwise calls `land.add_parser(subparsers)`. Each subparser sets `func=<callable(args) -> int>` and `main` returns `args.func(args)`.
A `land` module that exists but raises any other error must surface that error, not be swallowed. Usage errors exit 64.

**land.py side.** Exposes `add_parser(subparsers)` (registers `land` with `func=run`), `run(args) -> int`, and `main(argv=None) -> int` so
`python3 tools/agents/land.py BRANCH --gate CMD ...` works standalone. Arguments: `BRANCH`, repeatable `--gate CMD` (run with `sh -c`), `--no-push`,
`--repo PATH`, `--remote NAME` (default `origin`), `--mainline NAME` (default `main`), `--attempts N` (default 3). Exit codes: 0 landed (prints the sha),
3 overlap with tracked changes in the main checkout, 4 merge commits found, 5 a gate failed, 6 gave up after lost races, 64 usage error.

**Shared helpers.** Both import `git`, `main_worktree`, `now`, `WorkerError` from `worker.py` in the same directory.

**Integration check (lead).** After both land, `python3 tools/agents/wt.py land BRANCH --gate true --no-push --repo <throwaway>` must behave like `land.py`.
