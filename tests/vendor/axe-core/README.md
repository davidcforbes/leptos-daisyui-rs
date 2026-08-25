# Vendored axe-core (test-only)

- **Package**: `axe-core`
- **Version**: 4.13.0
- **License**: MPL-2.0 (see `LICENSE` beside this file)
- **Source**: https://registry.npmjs.org/axe-core/-/axe-core-4.13.0.tgz
  (the official npm package; `package/axe.min.js` copied verbatim)
- **SHA-256 of `axe.min.js`**:
  `c24f097bd2f451d4f933e8bc7d8d539f8672a2ebcb5cc9f9f3eec8ca9470a0c1`

Vendored for the reactivity suite's accessibility gate (`ldui-9tr.6`):
`pixelproof_web::a11y::Axe::from_path` injects this file into the page under
test, so the axe run needs no network at test time. This asset is test-only —
it must never be referenced from `demo/`, Trunk, or any non-dev dependency.

## Upgrading

```powershell
npm pack axe-core@<version> --pack-destination .review\axe-core
tar -xf .review\axe-core\axe-core-<version>.tgz -C .review\axe-core
Get-Content .review\axe-core\package\package.json | ConvertFrom-Json | Select-Object name,version,license
Copy-Item .review\axe-core\package\axe.min.js tests\vendor\axe-core\axe.min.js
Copy-Item .review\axe-core\package\LICENSE tests\vendor\axe-core\LICENSE
Get-FileHash tests\vendor\axe-core\axe.min.js -Algorithm SHA256   # update this README
```
