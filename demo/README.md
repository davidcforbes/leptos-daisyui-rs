# Demo showcase

This Leptos CSR application renders the library catalog and the dedicated
browser-test fixtures. It is a validation host, not a separate component
implementation or a source of CSS for consuming applications.

## Run locally

From the repository root:

```powershell
Set-Location demo
npm install
Set-Location ..
cargo make dev
```

The showcase is served at `http://127.0.0.1:3010`. `npm install` is required
because Trunk's pre-build step runs the local Tailwind CLI.

## Stylesheet contract

[`input.css`](./input.css) imports the repository's generated
`../styles/tokens.css` and scans `../src/**/*.rs`. A sibling consumer must do
the equivalent with paths relative to its own stylesheet; linking the Rust
crate does not copy this demo's CSS. This is especially important for the
opinionated table header, aligned filter band, and cell-grid tokens.

## Verification entry points

- `cargo xtask verify` runs the 14-step native gate and never starts this app.
- `cargo xtask test-reactivity` selectively builds and serves the catalog for
  exactly 32 real-browser DOM/interaction checks.
- `cargo xtask verify-pattern client-snapshot-list --browser` builds the
  smaller client-snapshot fixture rather than the full catalog.
- `cargo xtask verify-full` runs 19 native and browser/Wasm steps.
- `cargo make test-visual` runs the manual PixelProof screenshot workflow.

The current test policy and gate composition live in
[`../doc/ci-cd.md`](../doc/ci-cd.md); historical demo checklists are not the
issue tracker. Use `bd ready --json` from the repository root for planned work.
