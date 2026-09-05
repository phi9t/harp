# Publishing the Atlas

[Documentation](README.md) / Atlas publication

The hosted reader is [phi9t.github.io/harp](https://phi9t.github.io/harp/).
It serves the same single-file export as `atlas/dist/harp-atlas.html`.
The downloaded file remains usable offline.

## What is published

The [Publish Atlas workflow](../.github/workflows/pages.yml) runs on pushes to
`master` and supports manual dispatch on that branch. It publishes only:

- `index.html`, an unchanged copy of the checked-in Atlas export;
- `harp-atlas.receipt.json`, the corresponding export receipt.

The preparation command validates the receipt schema and checks its HTML and
corpus SHA-256 digests before staging those files. GitHub Actions does not
rebuild the Atlas or run the full repository gate. Contributors must regenerate
and verify the canonical inputs and export before landing, as described in
[contributing](contributing.md).

The site does not expose the checkout, local state, credentials, source
snapshots, or captured evidence as separate files. The export includes the
compiled research notes. Repository-relative source locators require the
checkout or the [GitHub repository](https://github.com/phi9t/harp); publishing
the reader does not make every local artifact link available on the site.

## Publish an update

1. Make the canonical content or reader change and regenerate the corpus and
   single-file Atlas together.
2. Follow the [landing checklist](contributing.md#land-a-change), including the
   full `mise run verify` gate and owner-authorized push to `master`.
3. Check the **Publish Atlas** run in the repository's Actions tab.
4. Open the hosted reader and confirm the changed route. The site's
   [export receipt](https://phi9t.github.io/harp/harp-atlas.receipt.json) records
   the deployed HTML and corpus digests.

To redeploy the existing export, dispatch **Publish Atlas** on `master`.
Manual runs on other branches do not deploy. An unsuccessful preparation does
not replace the live site.

For a local packaging check, run from the repository root:

```sh
python3 -m unittest discover -s scripts/tests -p test_prepare_pages.py -v
python3 scripts/prepare_pages.py
```

The output is ignored `.build/pages/`. Preparation refuses an existing output
directory or symlinked inputs/outputs. Move a previous output aside before
running it again.

## Repository settings

GitHub Pages uses **GitHub Actions** as its publishing source. The deployment
job uses the `github-pages` environment with Pages-write and OIDC permissions;
the preparation job has read-only repository access. Action versions are
pinned to commit hashes.

The workflow follows GitHub's
[custom Pages workflow](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
model. It does not configure a custom domain or publish a binary release.
