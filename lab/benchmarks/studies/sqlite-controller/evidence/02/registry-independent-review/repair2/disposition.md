# Registry repair2: tested, review rejected

Source `6f55bc67624b2399c3430fe2c5e003baeb4c22e0`, crate tree `9aead302dc0826d69b91112e8e5dca228d9a90cb`; scoped review base `cbe7854b59898f9750d1478d2277584c7eca7b42`.

Root independently ran all64 tests, formatting, strict Clippy and a normal offline build on unchanged source. Tests passed in761.814 seconds. Read-only observation of476 exactly retained owned identities found0 matching live;171 whole-process disposal receipts were all terminal. The source and root gate are retained here; these are deterministic actual native/kernel fixtures, not live model qualification.

Independent Standards PASS; independent Spec FAIL and branch641 FAIL agree on two new important repair-delta defects. A confirmed thread/start receipt adopted after a binding-write crash is not activated in the receive allowlist before subsequent RPCs, so current-thread notifications can disappear. Subscription cleanup retries can overwrite already-confirmed terminal stop evidence on a later failed history read, then revoke without reconciling pending unsubscribe. Existing GREEN checks omit these sequences.

Author clean evidence-only handoff is `cc774921be6e8e00b8de54942b56fa0fee01d56d`; unchanged source/crate and all57 retained evidence hashes independently match. Author complete64-test gate passes762.44s plus fmt/strictClippy/normalbuild. Root reobserved724 author-retained identities with0matchinglive;260 author whole-process receipts are terminal. Preserve this rejected candidate and evidence before the consolidated final registry repair3/3. Ticket01 and the separately qualified supervised-process core02 remain unchanged; ticket02 remains claimed and unaccepted. User sessions and source mainline remain untouched.
