# Crouzeix conjecture evidence provenance

Captured at: `2026-08-14T23:44:51Z`

This bundle keeps upstream proof artifacts remote-only. `source_manifest.tsv`
records immutable identities, byte counts, SHA-256 digests, rights status, and
semantic source paths. It does not vendor manuscript, PDF, TeX, Lean, prompt,
or repository bytes.

As of `2026-08-19`, `formalization/lean/CrouzeixConjecture/` is a tracked
Harp-local Lean 4.32 proof port derived from Jin commit
`565b6a3e0659b6e0785f783b016c3f6d9f171fa5`. That tree is proof implementation,
not raw evidence: the unmodified upstream archive remains represented here only
by remote URL, byte count, and digest, and the upstream revision still had no
license file present. Treat the port as Harp-maintained formalization code with
explicit source attribution, not as a normalized copy of the evidence bundle.

As of `2026-08-26`, Harp also publishes three local route certifications under
`evidence/crouzeix_conjecture/routes/`:

- `jin/receipt.json` with raw publication hash
  `5931d52d80b2c2c6440900cc85d61cb48169c70e9803f0615ed4ade9b58c4331`
  and review hash
  `a1962f75a98a3ebf73ee33a472ace2c4ff91eead538156f777b8dcf21dcddb4e`
- `lorist-schwenninger/receipt.json` with raw publication hash
  `f672bb002c9d7ebc516683d61ba87b2c1ff2bed891daac0e6781aeb50cc7a01b`
  and review hash
  `9f3fa1cffafc84b82e64c406cd843f43bbb4b1fc83197d69ed44c3f36d921d0a`
- `harp/receipt.json` with raw publication hash
  `5bc448e9a27968d6b73c7448fe72e41b52e2c1ec82d7b7ddf898dcf1aaa79883`
  and review hash
  `60e9bbc25979c8b2e4f38e0d03fa3fc68c8ff077949f94e81a024a56dd1957d6`

Each published route is `complete-local` only. These receipts certify named
local Lean declarations under the shared Harp formalization root; they do not
upgrade the evidence ceiling to peer review, author endorsement, clean-room
upstream Jin build success, or complete boundedness.

The corresponding six-row bundle is
`evidence/crouzeix_conjecture/local_formalization/manifest.tsv` with SHA-256
`efc8469255219938b958d687d4a62506df937918bd659df18e47eaeb85a71de7`.
It binds the three terminal declarations
`CrouzeixConjecture.crouzeixConjecture`,
`CrouzeixConjecture.loristSchwenningerMainTheorem`, and
`CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem`, plus the three
closed-range consequence declarations
`CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet`,
`CrouzeixConjecture.loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet`,
and
`CrouzeixConjecture.harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet`.
The Lorist-Schwenninger closed-range row uses `source_node_id` `-`, so the
bundle certifies the consequence declaration without attributing it to a named
route node.

As of `2026-08-20`, the Lorist-Schwenninger scalar endpoint also has a
Harp-local Lean proof-slice receipt at
`labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/proof-slices/ls-scalar-contradiction/attempt-001/receipt.json`
with file SHA-256
`ad13640a3a8676aa6dd7a2bfdb9f5c1975a0fdbee037cf0555767c8e40549781`.
This is a local proof-slice receipt, not upstream source evidence.

`acquire.sh` created fresh detached checkouts for the two Jin revisions,
verified remote and Git identity, ran each revision's `Lean/verify.sh`, and
performed a metadata-only tracked-source scan. Build logs replace the checkout
root with `<source-checkout>`, abort on secret-like text or output above 2 MiB,
and collapse any detected source excerpt to metadata and digests. Scan logs
contain only path, line number, token class, and line SHA-256.

Preprints.org returned status `403` on `2026-08-14`. The
receipt is metadata-only unless status 200 produced measured bytes. No claim
maps the posted manuscript to a Git revision without byte equality.

The five prerequisite DOI records are metadata-only publisher routes. Their
presence does not prove access to full text, reproduce a theorem, or establish
the correctness of either candidate proof.
