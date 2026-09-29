# ldui-design

The Leptos-free design layer of `leptos-daisyui-rs` (bead `ldui-3u3p`). It holds:

- `tokens`: the `--ld-*` custom-property block (`ui_tokens_css`), the animation primitives
  (`ui_animations_css`) and the dark form-control palette (`dark_form_css_vars`) as CSS strings, plus
  `tokens::generated`, the token values they are built from.
- `components`: the variant enums and class helpers of 71 components, each the component's former
  `src/components/<name>/style.rs`, moved unchanged. They are also flattened into the crate root.
- `contracts`: the page-contract vocabulary (`PageState`, `PageContractV2`, `ClientSnapshotContract`, …).
- `fixtures` (bead `ldui-rz43`): the examples the demo's badge, button and alert variant sections render,
  as plain data (`fixtures::badge::SECTIONS`, …). The demo renders them through its Leptos components and
  4iiz-kit's gallery through its askama ones, so pixelproof-parity compares identical inputs. The root
  crate's `tests/demo_shared_fixtures.rs` fails if a demo section stops reading its fixture.

`leptos-daisyui-rs` depends on this crate and re-exports every moved item at its old path, so no Leptos
consumer changes a line of Rust. The crate has no dependency on Leptos, `web-sys`, `wasm-bindgen` or
`ui-tokens`, and no path dependency outside its own directory, so it can be consumed by git tag.
`tests/leptos_free.rs` enforces that on the manifest; the full check is:

```bash
# must print nothing. `--prefix none` + dropping the root line matters: the
# root line carries this checkout's path, which contains "leptos-daisyui-rs".
cargo tree -p ldui-design -e normal,build --prefix none | rg -v '^ldui-design ' | rg -i 'leptos|ui-tokens'
```

## Where the token values come from (option a)

`ui-tokens` lives in Rust-DeskApp and is reachable only as the sibling path `../Rust-DeskApp`, which does
not exist for a git consumer. Of the three options on the bead, this crate takes **(a)**:

- `cargo xtask gen-tokens` writes two files from the real `ui-tokens` values: `styles/tokens.css` and
  `crates/ldui-design/src/tokens/generated.rs`.
- `cargo xtask gen-tokens --check` (the `tokens-fresh` step of `cargo xtask verify`) fails when either
  committed file drifts from `ui-tokens`.
- `cargo xtask check-sibling-tokens` now parses `xtask/src/design_tokens.rs`, the generator, so every
  `ui_tokens` item the generated file copies must exist on the sibling's default branch.

Options (b), a git-tag dependency on Rust-DeskApp, and (c), moving `ui-tokens` into this workspace, were
not taken: (b) would drag a Windows desktop repository into every web consumer's build, and (c) is a
cross-repository move the desktop face would have to adopt first.

To change a token, edit it in `../Rust-DeskApp/crates/ui-tokens`, run `cargo xtask gen-tokens`, and
commit both regenerated files.

## What stayed in `leptos-daisyui-rs`

- `components/ai_chat/style.rs`: it imports the sibling `texts`/`types` modules and the out-of-repo
  `ai-chat-core` crate.
- The Leptos mount points `UiTokensPreamble` and `UiAnimationsPreamble`.
- The filter-schema projection of `patterns/contracts.rs` (`FilterSchema`, `SnapshotViewDefaults`,
  `LocalFilterDefaults`, `FilterProjectionError`): its default-view payload carries `EntityTablePreferences`,
  a Leptos table type.

## Tailwind

The class literals now live in this crate's sources, so a Tailwind build must scan them as well as
`leptos-daisyui-rs/src`:

```css
@source "../leptos-daisyui-rs/src/**/*.rs";
@source "../leptos-daisyui-rs/crates/ldui-design/src/**/*.rs";
```
