---
name: waystone-hdfs-rootfs-artifacts
description: Use when persisting, retrieving, validating, or debugging Soma selected-rootfs artifacts through Waystone HDFS, especially for Insula rootfs archive round trips, HDFS storage-prefix checks, restore validation, or Waystone ticket creation after storage failures.
---

# Waystone HDFS Rootfs Artifacts

Use Waystone as the storage client. Soma owns selected-rootfs identity,
archive/manifest generation, and restore validation; Waystone owns HDFS traffic.

## Commands

From the Soma repo:

```sh
scripts/insula/run-rootfs-artifact-storage.sh hdfs-introspect
scripts/insula/run-rootfs-artifact-storage.sh hdfs-introspect --expect-remote
scripts/insula/run-rootfs-artifact-storage.sh pack --provider hdfs
```

From `~/workspace/waystone`:

```sh
./scripts/waystone doctor --json
./scripts/waystone storage-prefix --child "soma/insula-rootfs/<store-id>" --json
./scripts/waystone --mkdir-parents --overwrite put <local-file> <remote-file>
./scripts/waystone --overwrite get <remote-file> <local-file>
./scripts/waystone ls <remote-dir>
```

## Workflow

1. Run `hdfs-introspect` first. It checks Waystone doctor output, resolves the
   HDFS storage prefix, and can list the remote artifact directory.
2. If local archive files are missing, run `pack --provider hdfs`. A tar warning
   about unavailable POSIX ACL support is acceptable only when the command exits
   zero.
3. Resolve the remote directory with Waystone:
   `./scripts/waystone storage-prefix --child "soma/insula-rootfs/<store-id>" --json`.
4. Upload through Waystone only: archive, manifest, and
   `*.waystone-storage-request.json`.
5. Retrieve into a separate local directory, then compare every retrieved file
   with the original using `sha256sum` and `cmp -s`.
6. Report the remote HDFS directory, file sizes, and SHA-256s. Do not claim a
   restored rootfs is valid until the archive has been unpacked and
   `scripts/insula/validate-emacs-rootfs-contract.py <rootfs> --verify-rootfs-tree`
   passes.

## Tickets

If a Waystone HDFS check fails, run:

```sh
scripts/insula/run-rootfs-artifact-storage.sh hdfs-introspect --expect-remote
```

By default, failed introspection writes a ticket draft under:

```text
~/workspace/waystone/.scratch/soma-hdfs-artifact-tickets/
```

Use that draft as the Waystone issue/ticket body. If the prompt says
“waypoint,” use the Waystone repo unless a real `~/workspace/waypoint` checkout
exists and the user explicitly wants that repository.

## Guardrails

- Do not commit archives, retrieved payloads, local vaults, tokens, AK/SK
  material, or generated transfer scratch files.
- Do not use TOS or `bytedcli` from Soma for this workflow.
- Do not delete remote artifact directories unless the user explicitly asks.
- Do not treat a matching downloaded archive hash as rootfs runtime validation;
  it only proves storage integrity.
