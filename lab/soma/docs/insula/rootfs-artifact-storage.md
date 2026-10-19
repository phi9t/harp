# Rootfs Artifact Storage Handoff

Soma keeps materialized rootfs payloads out of git. Soma owns the selected
rootfs identity, local archive manifest, and restore-validation requirements.
Waystone owns effectful storage traffic. It owns HDFS today; TOS upload/download
belongs there rather than in Soma.

## Roles

- `scripts/insula/rootfs_artifact_storage.py` owns selected-rootfs archive
  naming, artifact manifest writing, and credential-free Waystone storage
  request files.
- `scripts/insula/run-rootfs-artifact-storage.sh` is the thin stable Soma
  entrypoint for planning, packing, and request-file emission.
- `~/workspace/waystone` is the storage traffic owner. It already owns the
  developer-facing HDFS stack; TOS traffic should be added there instead of
  reintroducing TOS execution in Soma.
- `.scratch/emacs-gym/insula/artifacts/` is the local staging area for packed
  rootfs archives, artifact manifests, and Waystone request files. It is
  git-ignored.

Storage is an artifact boundary. It stores or retrieves the selected rootfs
payload; it does not make a benchmark run hermetic by itself. Hermetic evidence
still comes from the selected rootfs contract, ABI manifest, and runner plans.

## Current Target

- Storage owner: Waystone
- Provider: `tos`
- Bucket name: `example-artifacts`
- Site: `example-region`
- Profile hint: `soma-insula-artifacts`
- Object prefix: `soma/insula-rootfs/`
- Current selected rootfs store id:
  `soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`

Soma request files intentionally omit AK, SK, tokens, numeric bucket ids, and
bytedcli credential material. Any local vault or credential profile for TOS
belongs to Waystone.

## Plan

Print a credential-free upload request plan:

```sh
scripts/insula/run-rootfs-artifact-storage.sh plan
```

Print a download request plan:

```sh
scripts/insula/run-rootfs-artifact-storage.sh plan --operation download
```

The plan records the selected rootfs store id, local artifact paths, TOS object
keys, the Waystone ownership boundary, and the required post-restore validation.
It is not an executable transport command.

## HDFS Introspection

Use the repo-local skill for the full round-trip workflow:
`.agents/skills/waystone-hdfs-rootfs-artifacts/SKILL.md`.

Inspect Waystone HDFS readiness and the selected-rootfs remote artifact
directory:

```sh
scripts/insula/run-rootfs-artifact-storage.sh hdfs-introspect --expect-remote
```

The command checks Waystone doctor output, resolves the HDFS storage prefix, and
lists the remote artifact directory. When a Waystone-side check fails, it writes
a ticket draft under:

```text
~/workspace/waystone/.scratch/soma-hdfs-artifact-tickets/
```

## Pack

Create a local `tar.zst` archive, artifact manifest, and Waystone upload
request for the selected rootfs:

```sh
scripts/insula/run-rootfs-artifact-storage.sh pack
```

The default local output is:

```text
.scratch/emacs-gym/insula/artifacts/<store-id>/<store-id>.tar.zst
.scratch/emacs-gym/insula/artifacts/<store-id>/<store-id>.manifest.json
.scratch/emacs-gym/insula/artifacts/<store-id>/<store-id>.waystone-storage-request.json
```

The artifact manifest records the selected rootfs store id, rootfs contract
hash, tree-manifest fields, base image identity, recipe SHA, storage descriptor,
archive size, archive SHA256, and restore-validation command.

## Waystone Handoff

Give the generated `*.waystone-storage-request.json` file to Waystone for the
effectful upload or download. Until Waystone grows a committed TOS interface,
this request file is the stable Soma-to-Waystone contract:

```json
{
  "kind": "soma_insula_waystone_storage_request",
  "operation": "upload",
  "effectful_storage_owner": "waystone",
  "storage": {
    "provider": "tos",
    "owner": "waystone"
  }
}
```

Soma must not run TOS upload or download commands from repo-owned scripts. That
traffic belongs in Waystone alongside its HDFS auth, transfer, verification,
and live-I/O safety surfaces. Until that Waystone transport lands, Soma request
files are handoff evidence, not completed transfer evidence.

## Restore

A downloaded rootfs payload must be unpacked under
`rootfs-store/content/<store-id>` and validated before it is selected:

```sh
scripts/insula/validate-emacs-rootfs-contract.py \
  .scratch/emacs-gym/insula/rootfs-store/content/<store-id> \
  --verify-rootfs-tree
```

Only after that validation should `selected.json` point at the restored store
id.
