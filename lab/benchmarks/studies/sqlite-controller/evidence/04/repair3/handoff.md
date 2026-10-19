# Ticket04 final repair3/3: evidence-helper portability

HELPER commit: `f439e9a1bbab958c0ced1a49026cac4207d7db64`, tree
`183368455b57e2885385e8c205d0805258e44938`, parent clean repair2 EVIDTIP
`0b97408bf8f7cd8663da181425cb35f230af9c01`, branch `sqlite04/git`.
Product SOURCE remains `e31d57bc9ab739d5b4473c387a9320d176adf07b`, with
all 34 source/test/gate files byte-identical. This consumes ticket04's final
repair round **3/3**, in the same phase. Ticket02 remains capped at 3/3;
no ticket/thread/cap reset, source landing or acceptance occurred.

The sole helper-commit change is repair1 `artifact-observations.py`, addressing
branch648's LOW portability finding. Reads now use the current script directory's
`fixtures/<retained filename>`. Archive filenames and provenance keys must be
unique; each local regular archive must match its manifest size and SHA-256.
Final-file symlinks and a symlinked fixtures directory are refused. Original
absolute paths are retained only in `archive_original` and
`archive_retained_original` output metadata. The helper requires explicit
`--output` and creates it exclusively, refusing existing files or symlinks.
The historical repair2 copied helper and every recorded output remain unchanged.

The actual first RED invoked the old helper against a uniquely owned relocated
copy containing all 89 retained fixture archives and original observations.
Original and retained path references were replaced with nonexistent owned
sentinel paths while preserving filenames, archive digests and bytes. The old
helper exited 1 with `FileNotFoundError` for the sentinel's archive. No original
checkout was moved, removed, chmodded or made inaccessible; the sentinel makes
its old absolute-path lookup unavailable without touching the original checkout.

Root source inspection caught the driver's planned overwrite call after deleting
the copied helper. **No GREEN or overwrite invocation had run or been credited.**
The driver was corrected to restore the exact pinned helper for overwrite,
require its actual `FileExistsError`, and confirm unchanged output. The first
genuine RED and intermediate pre-run seals are retained as chronology; those
intermediate seals do not count as additional executed proofs.

The final **identical driver and input manifest** then ran the retained old helper
RED (exit 1, sentinel `FileNotFoundError`) and corrected helper GREEN (exit 0,
main helper invocation 1.68s). GREEN produced all **59** records, matching every
historical content field while adding current archive paths and original-path
provenance. It demonstrated actual exclusive-output refusal with unchanged bytes,
duplicate manifest filename/provenance rejection, and manifest digest mismatch
rejection; invalid controls produced no output. All helper child PID/start
identities and waited exits are retained in the receipts. Copied inputs stayed
byte-identical and the original sentinel directory remained nonexistent.

`historical-before.json` pins all 579 pre-existing retained files. The only
changed historical path is the authorized helper: the other **578** files,
including every fixture, raw log, observation, source archive and bounded process
report, retain their exact hashes. The old helper blob remains reachable in Git
and is copied verbatim into this repair's evidence. `history-check.json` pins
the unchanged product source and helper commit. Paired source archives include
the complete 34 product files plus helper/driver (and old helper for final runs).
`relocated-inputs.tar.gz` and `fixture-pin.json` retain the full actual relocated
inputs; validation-control manifests and execution receipts pin the negatives.

Machine-readable outcomes are in `verification.json`, `retention.json`,
`checkpoint.json`, input/tool pins and paired receipts. Raw logs are retained
losslessly as gzip. The final driver can create a new owned fixture with
`python3 docs/efforts/sqlite-controller/evidence/04/repair3/relocate.py setup`,
then run `green --fixture <printed path> --label <fresh label>`. For old-helper
RED add `--helper docs/efforts/sqlite-controller/evidence/04/repair3/old-artifact-observations.py`.
Only the helper and supplied retained JSON/tar bytes are read; no worker Git,
config, hooks, model inference or fixture programs execute.

No Rust/native/product/gate changes or repeated Rust gates were made in this
evidence-only round. Repair2 author and independent affected49/native4/default34,
fmt, strict Clippy and normal-build qualification on source e31 are inherited.
Root's repair2 retention audit is explicitly inherited: PASS 47 records, 8 logs,
17 archives/578 members, 340 explicit PID/start identities, zero live/unknown.
Root independently reported its own relocated 89-archive old/new helper
RED1/GREEN0, 59 identical-content/provenance records, actual `FileExistsError`
overwrite refusal, unchanged inputs/output and product34, and exact helper
branch649 (`0b97408..f439e9a`) PASS. These are attributed coordinator results,
not author executions. Scoped Standards/Spec and final committed retention
remain pending. Root owns acceptance; those remaining checks are the next
discriminating tests and acceptance stays withheld until they conclude.
Final06's whole-baseline gate remains mandatory in its own phase.

`checkpoint.json` maps public failure, hypothesis, fixture rationale, paired
result, scope, unresolved controls and next test to branch648/F4 provenance.
Root owns the visible-action coverage ledger; private reasoning is excluded.
Any further hard issue must be reported against the exhausted 3/3 cap.

The author audit verified 11 retained records, 3 raw logs, 9 source archives
containing 330 members (including intermediate uncredited seals), all 92 actual
relocated input members and the unchanged 578 historical files. Six explicitly
owned helper PID/start identities were waited; none matched a live process and
no process identity read was unknown. All retained digests matched.
