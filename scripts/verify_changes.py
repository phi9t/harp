#!/usr/bin/env python3
"""Run verification for changed surfaces without installing prerequisites."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root).decode().strip()


def changed_paths(root, base=None):
    pending = git(root, "status", "--porcelain", "--untracked-files=all")
    if base is None:
        base = "HEAD" if pending else "HEAD^"
    revision = git(root, "rev-parse", "--verify", "--end-of-options", base + "^{commit}")
    tracked = subprocess.check_output(
        ["git", "diff", "--name-only", "--no-renames", "-z", revision, "--"], cwd=root
    )
    untracked = subprocess.check_output(
        ["git", "ls-files", "--others", "--exclude-standard", "-z"], cwd=root
    )
    return revision, sorted(set(p.decode() for p in (tracked + untracked).split(b"\0") if p))


def check_docs(root, paths):
    for relative in paths:
        path = root / relative
        if not path.exists():  # A deleted document has no remaining structure to validate.
            continue
        if path.is_symlink():
            raise ValueError("refusing symlinked guidance: " + relative)
        text = path.read_text(encoding="utf-8")
        if path.name == "SKILL.md":
            frontmatter = re.match(r"\A---\n(.*?)\n---(?:\n|$)", text, re.S)
            if not frontmatter:
                raise ValueError("missing skill frontmatter: " + relative)
            fields = frontmatter.group(1)
            name = re.search(r"(?m)^name:\s*([a-z0-9-]+)\s*$", fields)
            if not name or name.group(1) != path.parent.name:
                raise ValueError("skill name must match its directory: " + relative)
            if not re.search(r"(?m)^description:\s*\S", fields):
                raise ValueError("missing skill description: " + relative)
        # Fence examples are not navigable links. Anchor semantics remain owned
        # by the relevant prose/compiler checks, rather than a second renderer.
        prose = re.sub(r"(?ms)^[ \t]*```[^\n]*\n.*?^[ \t]*```[^\n]*$", "", text)
        prose = re.sub(r"(`+)(.*?)\1", "", prose, flags=re.S)
        for link in re.findall(r"\]\(([^)\n]+)\)", prose):
            target = link.split("#", 1)[0].strip("<>")
            if not target or re.match(r"[a-zA-Z][a-zA-Z0-9+.-]*:", target):
                continue
            if not (path.parent / target).exists():
                raise ValueError(f"broken local link in {relative}: {link}")


def plan(root, paths):
    jobs = []
    def add(label, *command):
        if command[-2:] == ("--test", "crouzeix_textbook"):
            preflight = {"surface": "lean-preflight", "command": ["mise", "run", "lean-env"]}
            if preflight not in jobs:
                jobs.append(preflight)
        job = {"surface": label, "command": list(command)}
        if job not in jobs:
            jobs.append(job)

    docs = []
    rust_paths = []
    relocation = root / "lab/math/relocation.json"
    unchanged_math = {}
    if relocation.is_file():
        record = json.loads(relocation.read_text())
        unchanged_math = {entry["destination"]: entry["source_sha256"]
                          for entry in record["entries"]
                          if entry["disposition"] == "retained"}
    for path in paths:
        if path == "content/harness_deep_dive_manifest.tsv":
            path = "evidence/implementations/harness_deep_dive_manifest.tsv"
        for old, new in (("crates/harp-session-index/", "sessions/"),
                         ("crates/harp/", "cli/"),
                         ("benchmarks/dynamic-workflow/", "lab/benchmarks/dynamic-workflow/"),
                         ("benchmarks/codex-architecture/schemas/", "lab/benchmarks/legacy_runtime/schemas/"),
                         ("content/crouzeix_textbook/", "lab/math/textbook/"),
                         ("content/sicp/", "lab/sicp/contracts/"),
                         ("content/context_control/", "durable/intent/bundles/"),
                         ("content/generated/", "lab/generated/"),
                         ("content/sources/", "evidence/sources/")):
            if path.startswith(old):
                path = new + path[len(old):]
                break
        legacy = ("harp-artifacts", "harp-cli-process", "harp-cli-test-support",
                  "harp-contracts", "harp-engine", "harp-runtime", "harp-state")
        if any(path.startswith("crates/" + name + "/") for name in legacy):
            path = "lab/benchmarks/legacy_runtime/" + path[len("crates/"):]
        for old, new in (("formalization/", "lab/math/"),
                         ("labs/crouzeix_proof_reproduction/", "lab/math/proof_reproduction/"),
                         ("labs/lean_trae_benchmark/", "lab/math/lean_trae_benchmark/"),
                         ("labs/sicp-evaluator/", "lab/sicp/")):
            if path.startswith(old):
                path = new + path[len(old):]
                break
        parts = Path(path).parts
        if path.startswith("assets/") and not (root / path).exists():
            add("repository", "mise", "run", "verify-repository")
        elif path.startswith(("reference/", "labs/meta_harness_trae/")) or path == "scripts/build_weng_course.py":
            # Article-only projections and reproduction tooling were retired.
            # Repository provenance checks cover their explicit disposition.
            add("repository", "mise", "run", "verify-repository")
        elif path.startswith(("corenius/", "harp_durable/")):
            add("absorption", "python3", "scripts/verify_absorption.py")
            add("architecture", "python3", "scripts/check_architecture.py")
            add("durable-runtime", "env", "PYTHONDONTWRITEBYTECODE=1", "python3", "-m", "unittest", "discover", "-s", "durable", "-t", ".", "-p", "test_*.py")
        elif path.startswith("durable/intent/bundles/"):
            add("rust", "cargo", "test", "--offline", "--locked", "--profile", "test-small", "-p", "harp", "--lib", "context_control::", "--", "--test-threads=1")
            add("rust-contract", "cargo", "test", "--offline", "--locked", "--profile", "test-small", "-p", "harp", "--test", "context_control_cli")
        elif path.startswith(("durable/", "insula/", "runner/")):
            if path.endswith(".md") and "/provenance/" not in path:
                docs.append(path)
            add("absorption", "python3", "scripts/verify_absorption.py")
            add("architecture", "python3", "scripts/check_architecture.py")
            add("durable-runtime", "env", "PYTHONDONTWRITEBYTECODE=1", "python3", "-m", "unittest", "discover", "-s", "durable", "-t", ".", "-p", "test_*.py")
            add("insula", "python3", "-m", "unittest", "discover", "-s", "insula", "-t", ".", "-p", "test_*.py")
            add("runner", "python3", "-m", "unittest", "discover", "-s", "runner", "-t", ".", "-p", "test_*.py")
            add("runner", "python3", "runner/privacy.py")
            add("native-lifecycle", "python3", "scripts/verify_native_receipt.py", "--suite", "integrated")
            if path.startswith("insula/supervisor/") and "/provenance/" not in path:
                rust_paths.append(path)
        elif (path.startswith(("lab/agentic_engineering/", "lab/benchmarks/harness_benchmarks/",
                               "lab/benchmarks/evaluator_integrity/", "lab/generated/"))
              or path in ("lab/article_survey.md", "lab/harp_knowledge_home.md")
              or (path.endswith(".md") and path.startswith(("lab/math/crouzeix_conjecture/",
                  "lab/math/crouzeix_textbook/", "lab/math/mathematical_foundations/",
                  "lab/math/autodiff_geometry/", "lab/sicp/")))):
            for command in ("check", "build"):
                args = ("--check",) if command == "build" else ()
                add("corpus", "cargo", "run", "--offline", "--locked", "--profile", "test-small", "-p", "harp", "--", command, *args)
        elif path.startswith("lab/benchmarks/legacy_runtime/"):
            add("legacy-runtime", "cargo", "test", "--manifest-path", "lab/benchmarks/legacy_runtime/Cargo.toml", "--offline", "--locked", "--workspace", "--all-features", "--profile", "test-small")
        elif path.startswith("evidence/sources/") or path == "evidence/implementations/harness_deep_dive_manifest.tsv":
            for command in ("check", "build"):
                args = ("--check",) if command == "build" else ()
                add("corpus", "cargo", "run", "--offline", "--locked", "--profile", "test-small", "-p", "harp", "--", command, *args)
        elif path.startswith(("evidence/engineering_history/", "evidence/provenance/", "lab/benchmarks/")):
            add("absorption", "python3", "scripts/verify_absorption.py")
            if path.startswith("lab/benchmarks/") and not path.startswith("lab/benchmarks/studies/"):
                add("native-benchmarks", "python3", "scripts/verify_native_receipt.py", "--suite", "benchmarks")
        elif path.startswith("lab/soma/"):
            add("soma", "python3", "lab/soma/verify_preservation.py")
            add("soma", "python3", "-m", "unittest", "discover", "-s", "lab/soma", "-p", "test_verify_preservation.py")
            for test in sorted((root / "lab/soma/scripts/insula").glob("test*.py")):
                add("soma", "env", "PYTHONPATH=lab/soma", "python3", str(test.relative_to(root)))
        elif path.startswith("lab/math/proof_reproduction/") or path in ("lab/math/relocation.py", "lab/math/relocation.json", "lab/math/test_relocation.py"):
            add("math-relocation", "python3", "-m", "unittest", "discover", "-s", "lab/math", "-p", "test_relocation.py")
            add("proof", "python3", "-m", "unittest", "discover", "-s", "lab/math/proof_reproduction")
            add("math-relocation", "python3", "lab/math/relocation.py")
        elif path.startswith("lab/math/lean_trae_benchmark/"):
            add("lean-benchmark", "python3", "-m", "unittest", "discover", "-s", "lab/math/lean_trae_benchmark")
        elif path.startswith("lab/sicp/") and not path.startswith("lab/sicp/contracts/"):
            rust_paths.append(path)
        elif path in ("docs/import-receipt.md", "docs/import-map.tsv") or path.startswith("evidence/") or path == ".gitattributes":
            add("repository", "mise", "run", "verify-repository")
        elif path.startswith("tools/obsidian/") or path in ("scripts/validate_obsidian_assets.py", "scripts/tests/test_obsidian_assets.py") or (path.startswith("lab/obsidian/") and not path.endswith(".md")):
            add("obsidian", "python3", "scripts/validate_obsidian_assets.py")
            add("obsidian", "python3", "-m", "unittest", "discover", "-s", "scripts/tests", "-p", "test_obsidian_assets.py")
        elif path.startswith(("knowledge/", "lab/math/textbook/", "lab/sicp/contracts/", "lessons/")):
            for command in ("check", "build"):
                args = ("--check",) if command == "build" else ()
                add("corpus", "cargo", "run", "--offline", "--locked", "--profile", "test-small", "-p", "harp", "--", command, *args)
        elif path.startswith(("formalization/", "lab/math/")) and not path.endswith(".md"):
            add("math-relocation", "python3", "lab/math/relocation.py")
            add("math-relocation", "python3", "-m", "unittest", "discover", "-s", "lab/math", "-p", "test_relocation.py")
            if not (root / path).exists() or not path.endswith((".lean", "lean-toolchain", "lakefile.toml", "lake-manifest.json")):
                continue
            if path in unchanged_math and hashlib.sha256((root / path).read_bytes()).hexdigest() == unchanged_math[path]:
                # Relocation validates source identities and original bytes. The
                # fresh exporter/publication checks cover its changed path seam.
                continue
            routes = {
                "TrainingDynamics": "lean-training", "training_dynamics": "lean-training",
                "MathematicalFoundations": "lean-foundations", "mathematical_foundations": "lean-foundations",
                "NNG4Intro": "lean-nng4", "nng4_intro": "lean-nng4",
                "AutodiffGeometry": "lean-autodiff", "autodiff_geometry": "lean-autodiff",
                "CrouzeixTextbook": "lean-crouzeix-textbook",
            }
            target = next((task for directory, task in routes.items() if directory in parts or Path(path).stem == directory), "lean-all")
            for route, task in (("Crouzeix/Jin/", "lean-crouzeix-jin"), ("Crouzeix/Harp/", "lean-crouzeix-harp"), ("Crouzeix/LoristSchwenninger/", "lean-crouzeix-ls")):
                if route in path:
                    target = task
            add("lean", "mise", "run", target)
        elif path == ".github/workflows/pages.yml" and not (root / path).exists():
            add("verification", "python3", "-m", "unittest", "discover", "-s", "scripts", "-p", "test_verify_changes.py")
        elif path.endswith(".md") and not path.startswith(("cli/", "sessions/", "crates/", "labs/", "tools/textbook_pdf/")):
            docs.append(path)
        elif path.startswith("atlas/") and not (root / path).exists():
            add("repository", "mise", "run", "verify-repository")
        elif path.startswith(("cli/", "sessions/", "crates/")) or path in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml") or path.startswith(".cargo/"):
            rust_paths.append(path)
        elif path.startswith("labs/") and path.endswith(".rs"):
            rust_paths.append(path)
        elif path.startswith("labs/"):
            lab = parts[1]
            test = path if path.startswith(f"labs/{lab}/tests/test_") and path.endswith(".py") else f"labs/{lab}/tests/test_{Path(path).stem}.py"
            if (root / test).is_file():
                add("lab", "python3", "-m", "unittest", test.replace("/", ".")[:-3])
            elif (root / f"labs/{lab}/tests").is_dir():
                add("lab", "python3", "-m", "unittest", "discover", "-s", f"labs/{lab}/tests")
            else:
                raise ValueError("no focused lab check mapped for " + path)
        elif path.startswith("tools/textbook_pdf/"):
            tests = sorted(str(p.relative_to(root)) for p in (root / "tools/textbook_pdf").glob("*.test.cjs"))
            add("pdf", "node", "--test", *tests)
        elif path == "mise.toml" or path.startswith("scripts/"):
            if not (root / path).exists():
                continue
            if path == "mise.toml" or path in ("scripts/verify_changes.py", "scripts/test_verify_changes.py"):
                add("verification", "python3", "-m", "unittest", "discover", "-s", "scripts", "-p", "test_verify_changes.py")
            elif path in ("scripts/check_architecture.py", "scripts/test_check_architecture.py"):
                add("architecture", "python3", "scripts/check_architecture.py")
                add("architecture", "python3", "-m", "unittest", "discover", "-s", "scripts", "-p", "test_check_architecture.py")
            elif path in ("scripts/verify_native_receipt.py", "scripts/test_verify_native_receipt.py"):
                add("native-receipt", "python3", "-m", "unittest", "discover", "-s", "scripts", "-p", "test_verify_native_receipt.py")
            elif path in ("scripts/verify_absorption.py", "scripts/git_objects.py", "scripts/test_verify_absorption.py"):
                add("absorption", "python3", "scripts/verify_absorption.py")
                add("absorption", "python3", "-m", "unittest", "discover", "-s", "scripts", "-p", "test_verify_absorption.py")
            elif path.endswith(".sh"):
                if (root / path).exists():
                    add("shell", "bash", "-n", path)
                test = "scripts/test_" + Path(path).stem + ".sh"
                if Path(path).name.startswith("test_"):
                    test = path
                if (root / test).exists():
                    add("shell", "bash", test)
                else:
                    add("shell", "mise", "run", "test-shell")
            else:
                test = path if path.startswith("scripts/tests/test_") else "scripts/tests/test_" + Path(path).stem + ".py"
                if (root / test).exists():
                    add("scripts", "python3", "-m", "unittest", "discover", "-s", "scripts/tests", "-p", Path(test).name)
                else:
                    add("scripts", "python3", "-m", "unittest", "discover", "-s", "scripts/tests")
        elif path.endswith(".md") or path.startswith(".agents/") or path in (".gitignore", ".github/CODEOWNERS"):
            if path.endswith(".md"):
                docs.append(path)
        else:
            raise ValueError("no check mapped for " + path + "; add a focused mapping before landing")
    if rust_paths:
        metadata = json.loads(subprocess.check_output([
            "cargo", "metadata", "--offline", "--locked", "--no-deps", "--format-version", "1"
        ], cwd=root))
        packages = {p["name"]: p for p in metadata["packages"] if p["id"] in metadata["workspace_members"]}
        affected = {name for name, package in packages.items() if any(
            path.startswith(str(Path(package["manifest_path"]).parent.relative_to(root)) + "/") for path in rust_paths
        )}
        if any(not p.startswith(("cli/", "sessions/", "crates/", "labs/", "lab/", "insula/supervisor/")) for p in rust_paths):
            affected = set(packages)
        # Changing a test alone does not change a package's public contract.
        runtime = {name for name in affected if any(
            path.startswith(str(Path(packages[name]["manifest_path"]).parent.relative_to(root)) + "/")
            and "/tests/" not in path for path in rust_paths
        )}
        if any(not p.startswith(("cli/", "sessions/", "crates/", "labs/", "lab/", "insula/supervisor/")) for p in rust_paths):
            runtime = set(packages)
        while True:
            consumers = {name for name, package in packages.items() if any(d["name"] in runtime for d in package["dependencies"])}
            if consumers <= runtime:
                break
            runtime |= consumers
        affected |= runtime
        for name in sorted(affected):
            prefix = ("cargo", "test", "--offline", "--locked", "--profile", "test-small", "-p", name)
            test_targets = {str(Path(t["src_path"]).relative_to(root)): t["name"]
                            for t in packages[name].get("targets", []) if "test" in t["kind"]}
            directory = str(Path(packages[name]["manifest_path"]).parent.relative_to(root))
            owned = [p for p in rust_paths if p.startswith(directory + "/")]
            if name not in runtime and owned and all(Path(p).parent == Path(directory) / "tests" and (root / p).exists() for p in owned):
                for p in owned:
                    add("rust-contract", *prefix, "--test", Path(p).stem)
                continue
            if name == "insula-supervisor":
                add("native-supervisor", "python3", "scripts/verify_native_receipt.py", "--suite", "supervisor")
                continue
            if name != "harp":
                add("rust", *prefix)
                continue
            local = [p for p in rust_paths if p.startswith("cli/")]
            modules = {"corpus" if "/src/corpus/" in p else "context_control" if "/src/context_control/" in p else "" for p in local}
            if local and modules <= {"corpus", "context_control"}:
                for module in sorted(modules):
                    add("rust", *prefix, "--lib", module + "::", "--", "--test-threads=1")
            else:
                add("rust", *prefix, "--lib", "--", "--test-threads=1")
            for p in local:
                if not (root / p).exists():
                    continue
                if p.endswith("/src/main.rs"):
                    add("cli", *prefix, "--test", "cli")
                    if (root / "cli/src/durable_cli_test.rs").exists():
                        add("rust-contract", *prefix, "--test", "durable_cli")
                elif p.endswith("/src/durable.rs"):
                    add("rust-contract", *prefix, "--test", "durable_cli")
                elif p in test_targets:
                    add("rust-contract", *prefix, "--test", test_targets[p])
                elif p.startswith("cli/src/test_support/"):
                    add("rust-contract", *prefix, "--test", "crouzeix_textbook")
    if docs:
        add("docs", "python3", "scripts/verify_changes.py", "--check-docs", *docs)
    return jobs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="compare the whole candidate against this Git commit/ref")
    parser.add_argument("--plan", action="store_true", help="print selected checks without executing them")
    parser.add_argument("--check-docs", nargs="+", help=argparse.SUPPRESS)
    args = parser.parse_args()
    root = Path(git(Path.cwd(), "rev-parse", "--show-toplevel")).resolve()
    if args.check_docs:
        check_docs(root, args.check_docs)
        return
    base, paths = changed_paths(root, args.base)
    # Imported Corenius bytes are checked against retained Git objects by their
    # own gate. Existing whitespace in those source captures is not an edit.
    jobs = [{"surface": "diff", "command": ["git", "diff", "--check", base, "--", ".", ":(exclude)corenius", ":(exclude)evidence/engineering_history", ":(exclude)lab/benchmarks/studies", ":(exclude)lab/benchmarks/inventories", ":(exclude)lab/benchmarks/tasks/t05-git-conflict-marker-cleanup/environment/generated.conf", ":(exclude)insula/supervisor/provenance", ":(exclude)insula/supervisor/docs/TEST_OUTPUT.txt"]}] + plan(root, paths)
    if args.plan:
        print(json.dumps({"base": base, "paths": paths, "checks": jobs}, indent=2))
        return
    environment = os.environ.copy()
    environment.setdefault("CARGO_TARGET_DIR", environment.get("HARP_TARGET_DIR", str(root / ".build/harp-target")))
    for job in jobs:
        print("[verify:" + job["surface"] + "] " + " ".join(job["command"]), flush=True)
        subprocess.run(job["command"], cwd=root, env=environment, check=True)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, FileNotFoundError, subprocess.CalledProcessError) as error:
        print("verification failed: " + str(error), file=sys.stderr)
        sys.exit(1)
