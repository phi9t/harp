# Native release candidates

[Documentation](README.md) / Native release candidates

The first release milestone is a locally installable native Harp executable
whose manifest identifies the verified Git commit. The packaging command does
not create a GitHub repository, push commits, create a tag, or publish a release.

To read the Atlas or build the CLI without packaging, use
[getting started](getting-started.md).

## Prepare the checkout

Follow [CLI setup](getting-started.md#build-the-cli), including reviewing and
trusting Mise configuration before installing tools. Fetch the LFS objects.
Mise pins Rust, Node, and Git LFS; bootstrap installs locked Cargo and Atlas
dependencies.

The full gate also requires the pinned Lean toolchain and an already populated
Lean dependency cache. Follow the Lean cache discipline in `AGENTS.md` and
validate it with `mise run lean-env`. Feature worktrees use the primary
checkout's cache. Dependency-cache hydration is a separate owner-authorized
maintenance operation; release packaging never performs it.

Commit the settled payload and its refreshed import receipt before packaging.
Both tracked modifications and untracked, non-ignored files block a candidate.

```sh
mise run release-candidate
```

This runs the full `mise run verify` gate, builds the Cargo `size` profile with
`--locked` and the host Rust target, creates an archive, and runs the extracted
`harp --version` from a temporary directory. It checks that the checkout and
commit remain unchanged. There is no option to skip verification.

Outputs appear together under ignored
`dist/harp-VERSION-TARGET-COMMIT/` only after those checks pass:

- a `.tar.gz` archive containing `harp` and `RELEASE_NOTES.md`;
- `manifest.json`, recording the full commit, Cargo version, target, compiler,
  build host, verification result, and binary/archive SHA-256 digests;
- `SHA256SUMS`, covering the archive, manifest, and release notes;
- `RELEASE_NOTES.md`, also available outside the archive.

Existing candidate directories are preserved and cause an error. To build
again, move the previous directory aside deliberately. Archives are identified
by their recorded digest; byte-for-byte reproducible archives are not promised.

## Install and check

In the candidate directory, check the supplied files before extraction:

```sh
shasum -a 256 -c SHA256SUMS
tar -xzf harp-VERSION-TARGET-COMMIT.tar.gz
./harp --version
mkdir -p "$HOME/.local/bin"
install -m 755 harp "$HOME/.local/bin/harp"
```

Replace the archive filename with the actual generated name. Ensure
`$HOME/.local/bin` is on PATH. The install command replaces an existing `harp`
in that directory; retain the previous executable if rollback is needed.
On Linux, `sha256sum -c SHA256SUMS` is also suitable.

The executable is built and smoke-tested on the recorded host only. This
workflow supports native Unix builds; it does not establish compatibility with
older operating-system versions or other architectures. macOS signing and
notarization are separate release work.

The binary archive does not include the knowledge corpus, evidence, Atlas,
Lean cache, or provider CLIs. Run repository commands such as `harp check`,
`harp build --check`, and `harp search refresh` from a matching content checkout.
The offline Atlas is `atlas/dist/harp-atlas.html` in that checkout. Live provider
commands require a separately configured provider installation; passing the
offline gate does not verify a live provider session.

## GitHub publication

The source repository is [phi9t/harp](https://github.com/phi9t/harp).
Publishing source commits does not publish a binary release. Select the release
commit and review its contents before uploading artifacts.
Repository publication must account for tracked evidence, source-specific
licenses, private inputs, Git history, and Git LFS objects. The technical gate
does not decide redistribution permission. No repository-wide license is
selected by this workflow.

Publish only artifacts whose manifest commit matches the selected release
commit. Keep the manifest, checksums, archive, and notes together. Separate
source-migration or licensing proposals require their own reviewed integration
decision; they are not part of the packaging command.

Architecture follow-ups and milestone status are recorded in the
[release-readiness planner](superpowers/plans/2026-09-04-github-release-readiness.md).
