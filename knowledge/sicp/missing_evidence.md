# SICP missing-evidence ledger

**EVIDENCE — policy.** Every item below is absent from the fetched corpus. It
must remain `MISSING` or `SPECULATION` until a separately captured source or
local experiment supplies an anchor.

| Label | Status | Missing item | Why it matters | Promotion path |
|---|---|---|---|---|
| EVIDENCE | MISSING | Official MIT Press edition identity and errata crosswalk | The seed is an unofficial Texinfo rendering and may contain conversion defects | Capture the official edition metadata and maintained errata, then compare section and exercise identities |
| EVIDENCE | MISSING | Primary lineage sources in the registry | The seed can establish citation topology but cannot prove what cited works claim | Fetch each source under a dated cohort with immutable locator and digest |
| EVIDENCE | MISSING | Independent account of Scheme's historical development | The book's authors are participants in that history | Add primary artifacts and a distinct historical source without merging their claims |
| EVIDENCE | MISSING | Exercise-completion evidence | SICP's learning value depends heavily on constructing and modifying systems | Maintain a separate exercise ledger with runnable receipts rather than marking chapters “read” |
| EVIDENCE | MISSING | Direct-versus-analyzed evaluator measurements | §4.1.7 provides a qualitative model and Exercise 4.24 asks the reader to measure it | Implement an analyzed path, fix workloads, and report parser, runtime, warmup, repetitions, and uncertainty |
| EVIDENCE | MISSING | Direct-versus-compiled evaluator measurements | The source explains lowering but the fetched pass does not establish performance envelopes | Build controlled instruction-count and wall-clock experiments |
| EVIDENCE | MISSING | Tail-call and storage behavior in the Rust companion | Host Rust recursion does not model Scheme's proper tail calls or SICP's explicit-control machine | Add an explicit machine with stack-depth counters |
| EVIDENCE | MISSING | Garbage-collection behavior | The companion delegates allocation and collection to Rust ownership | Implement or instrument a heap model before making storage claims |
| EVIDENCE | MISSING | Full Scheme conformance | The companion intentionally implements only a teaching subset | Define a versioned language surface and conformance suite before expanding the claim boundary |
| SPECULATION | MISSING | A modern course should retain the evaluator/compiler spine while changing surface language | The abstraction sequence may transfer, but this pass contains no learner study | Compare curricula and learning outcomes using separately sourced evidence |
