# jj executable identity preflight

Corenius base read: `963f58b31e518a8162dfe719e386ef9d9843321f`.
Selected jj release: `0.44.0`, source `af45d57de7163cc5f17f3df178a31577e4951ea3`.
Executable SHA256: `15fe28e44398552177ce01ba69dd17b52151e8e604ccf82fe587acd40ba13be4`.
Observed version: `jj 0.44.0-af45d57de7163cc5f17f3df178a31577e4951ea3`.
Status: bounded model-free preflight, not ticket19 acceptance or backend adoption.

The lead supplied the source-built release binary, reporting `cargo --locked`
release build with Rust1.95; this probe independently checks executable hash and
version, not build provenance. The earlier read-only assessment inspected main
`030ecf22c56b98bf6507067dca0df3dffa6de498`; its reported Rust1.97.1 requirement
was unavailable to this preparation. This executable evidence is specific to
the selected 0.44.0 release and does not supersede the earlier source assessment.
The unrelated `/opt/tiger/ss_bin/jj` SSH helper was never invoked.

## Reproduction and evidence

[Probe](probes/jj_identity_probe.py) accepts an absolute or resolved pinned binary
path and creates only private disposable repositories. `JJ_CONFIG=''` excludes
default jj configuration, as documented in the selected release's
[config documentation](https://github.com/jj-vcs/jj/blob/af45d57de7163cc5f17f3df178a31577e4951ea3/docs/config.md).
The process environment is constructed explicitly; Git global/system config
is `/dev/null`, editor commands refuse interaction, and no credentials, hooks
or remotes are installed. No shared checkout is converted. `HOME` is not reassigned.
Each command has a 20-second deadline and captures its exit and output.

```
python3 docs/efforts/team-control-plane/research/probes/jj_identity_probe.py \
  --binary .scratch/team-control-plane/jj-preflight/jj-pinned \
  --guard change-id --output docs/efforts/team-control-plane/evidence/jj-preflight-red.json
python3 docs/efforts/team-control-plane/research/probes/jj_identity_probe.py \
  --binary .scratch/team-control-plane/jj-preflight/jj-pinned \
  --output docs/efforts/team-control-plane/evidence/jj-preflight-results.json
```

Meaningful RED: actual snapshot rewrite changed commit and tree while change ID
remained equal; a deliberately naive change-ID evidence guard accepted that
changed candidate and the public fixture failed with exit1. All jj commands
preceding that assertion succeeded. This is a failing acceptance guard, not a
claim that jj fails its CLI contract. [Red record](../evidence/jj-preflight-red.json)
retains measured IDs, command output and successful fixture cleanup.

GREEN: exact commit/tree guard rejected the rewritten candidate; ignored-file,
bundle/import and peer-workspace checks also passed, exit0. The initial optional
peer check had a CLI argument error (`describe --revisions`); correcting it to the
documented positional revision produced the successful complete run. That setup
error is not counted as a behavioral red. [Green record](../evidence/jj-preflight-results.json)
retains exact events and artifact hashes. Syntax compilation passed separately.

## Observed boundaries

- Editing `artifact.txt` and invoking a snapshotting command preserved the
  change ID while changing exact Git commit and tree IDs. Both captured
  revisions were conflict-free.
- A never-tracked synthetic `private.env` matched the fixture `.gitignore` and
  appeared in neither captured commit nor imported bundle tree. Only
  `.gitignore` and `artifact.txt` were captured. This does not prove exclusion
  of previously tracked secrets or a complete private-file policy.
- An exact candidate commit was exported into a local Git bundle before source
  mutation. An independent Git repository imported that bundle after mutation,
  recovered the original commit/tree and original bytes, and preserved its
  captured-path list. Bundle/file SHA256 values are retained.
- A second jj workspace sharing the repository successfully changed the first
  workspace's change description: same change ID/tree, different exact commit.
  This was a sequential metadata rewrite, not a concurrent race or a tested
  stale-workspace repair.
- Both red and green temporary repositories were removed after their commands
  ended. No models, network services, remotes, shared installation change,
  worktree commit, push or integration occurred.

The selected release's [working-copy documentation](https://github.com/jj-vcs/jj/blob/af45d57de7163cc5f17f3df178a31577e4951ea3/docs/working-copy.md)
describes snapshots and shared-store workspaces; these observations exercise a
small subset of that contract. Inference: a change ID alone cannot identify
accepted evidence, and a shared jj store must not be treated as a worker
security boundary. Exact immutable candidates plus mediation or separate stores
remain necessary where cross-attempt writes are prohibited.

## Limits and dependent work

Only the bounded identity and retention hazard was exercised. Fixture bundles
and repositories were deleted; retained digests/IDs are historical evidence,
not presently available objects. The probe regenerates equivalent fixtures;
random IDs and commit timestamps need not reproduce the same hashes.

No capsule, crash recovery, concurrent/divergent operation, conflict rejection,
supervisor cancellation, production artifact service, repository gate, GC,
protected landing or workspace-interface integration guarantee follows.
Ticket19 remains blocked by tickets15/16 and needs their accepted workspace
contracts plus independent review. No adopt/defer disposition is made here.
