# Harp provenance: SICP second edition

## Artifact identity

- Title: *Structure and Interpretation of Computer Programs*, second edition.
- Authors: Harold Abelson and Gerald Jay Sussman, with Julie Sussman.
- Copyright notice: © 1996 by the Massachusetts Institute of Technology.
- Captured edition: Unofficial Texinfo Format `2.andresraba5.6`, dated
  2016-02-02 and based on `2.neilvandyke4` dated 2007-01-10.
- Source locator: `https://web.mit.edu/6.001/6.037/sicp.pdf`.
- Retrieval time: `2026-08-01T05:03:41Z` (`2026-07-31T22:03:41-07:00`).
- HTTP metadata: status `200`, content type `application/pdf`, content length
  `7416886`, ETag `"712c36-580c990f04300"`, last modified
  `2019-01-31T23:41:32Z`.
- Local artifact: `evidence/sicp/sicp.pdf`.
- Size: `7,416,886` bytes.
- SHA-256: `08709a87567d8311d6fd29c4f4a5386801153e71450e628c4a5a5d7e85feda8b`.
- PDF identity: version 1.5, 883 pages, unencrypted, no forms or JavaScript.

## License

PDF page 2 states that the work is licensed under Creative Commons
Attribution-ShareAlike 4.0 International. The complete legal code is vendored
as `LICENSE.txt`, fetched from
`https://creativecommons.org/licenses/by-sa/4.0/legalcode.txt`; it is 20,138
bytes with SHA-256
`28a9529c7d0bb4dc51f4bf5c116a3d16ef247a052f7591466768ddf563fd1cf5`.
It was fetched at `2026-08-01T05:06:29Z`.

Attribution must preserve the title, authors, MIT copyright notice, license,
and the fact that this artifact is the unofficial Texinfo rendering rather
than an official MIT Press typeset edition. Adaptations must be shared under
the same or a compatible license as specified by CC BY-SA 4.0.

## Capture and verification

The PDF was fetched with `curl -L --fail --max-time 120`. Poppler `pdfinfo`
26.07.0 verified the container and page count; Poppler `pdftotext` 26.07.0
produced a non-empty disposable text sidecar under ignored private state for
analysis. Representative full-page renders of the title/license page, the
opening evaluator pages, the environment representation, and the
analysis/execution split were visually inspected with Poppler rendering.

The disposable text and PNG renderings are not canonical artifacts. All
maintained claims cite the vendored PDF by source token, printed page, and PDF
page.
