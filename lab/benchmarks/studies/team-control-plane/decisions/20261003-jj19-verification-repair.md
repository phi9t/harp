# jj19 verification-order repair

Commit read: `d1df44739e56b1cf8ebfa1a349f88e377996cbf5` (reference-design); original evaluation base264756e. Full-goal owner authority permits necessary repairs; no additional architectural authority or runtime budget is created.

Independent Standards review passes. Independent Spec review identifies an Important acceptance gap: the original exported candidate is first independently verified after it was rewritten, and the changed candidate does not receive a fresh proof. The immutable-object preservation evidence remains useful but does not satisfy the literal rewrite-after-verification contract.

Repair1/3 is isolated in `/tmp/corenius-jj19-verification-repair`, branched from264756e. All110 transferred source hashes match the original frozen snapshot. The author must retain a meaningful behavioral red, execute an independent original-candidate verification before rewrite, attempt and refuse proof reuse after change, and freshly verify the changed candidate. Original runs and freezes remain unchanged; new receipts receive distinct names.

This repair does not adopt jj or implement backend25. It does not permit Trae/model/provider execution, shared-store conversion or mainline mutation. Fresh source-bound final outcomes, independent Standards/Spec reconciliation and the root repository/branch gates remain required before acceptance.
