# Claude Design sync: repo notes

- **Not a React DS.** Components are Rust/Leptos (wasm). Claude Design renders
  React only, so this syncs as a **tokens-only** design system: the converter
  runs over a synthetic package (`.design-sync/.cache/pkg/`, generated) whose
  `index.js` exports nothing and whose `styles.css` is the showcase's REAL
  compiled CSS. Components reach the design agent as reference pages in
  `guidelines/guides/<group>/<slug>.md`: each showcase example's real rendered
  markup and a screenshot, never a reimplementation.
- **Pipeline** (re-sync runs all of it):
  1. `cargo xtask capture-design` builds and serves the showcase, walks every
     sidebar route with PixelProof's harness (`tests/design_capture.rs`), and
     writes `target/design-capture/` (manifest, per-example html + png, every
     loaded stylesheet, every runtime `<style>` block).
  2. `node .design-sync/build-from-capture.mjs pkg` generates the synthetic
     package (CSS + guides).
  3. `node .ds-sync/resync.mjs --config .design-sync/config.json --node-modules
     .ds-sync/node_modules --entry .design-sync/.cache/pkg/index.js --out
     ./ds-bundle` (add `--remote <saved _ds_sync.json>` on a re-sync).
  4. `node .design-sync/build-from-capture.mjs images` copies the screenshots
     into `ds-bundle/guidelines/`. It must run AFTER step 3, because the build
     recreates `ds-bundle/`.
- **Converter deps in `.ds-sync/`**: esbuild, ts-morph, @types/react,
  react@18 + react-dom@18 (the converter vendors React even for tokens-only),
  and playwright installed with `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`. The
  render check uses the installed Chrome via
  `DS_CHROMIUM_PATH="C:\Program Files\Google\Chrome\Application\chrome.exe"`.
- **Examples are found by `data-demo-section`** (demo/src/core/section.rs).
  About 17 pages build raw markup with no `Section` and are captured whole.
  Six pages have no `<h1>` (swap, theme_controller, config-provider, kanban,
  auto-complete, color-picker), so the driver waits for `main > *`.
- **The freeze sheet is excluded by attribute**: PixelProof's
  `<style data-pixelproof="freeze">` (animations off) is capture scaffolding.
  The generator drops the inline style whose manifest attrs carry
  `data-pixelproof=freeze`, never by index.
- **Config keys are strict**: the converter rejects unknown `config.json` keys,
  so capture-only settings must not go there.
- Node one-liners with regex backslashes break in Git Bash; the check scripts
  live in files.

## Re-sync risks

- The CSS is **purged** to classes the crate and showcase use. A design that
  reaches for another Tailwind utility silently gets nothing; the conventions
  guide says so.
- Reference markup reflects the showcase, so a component the showcase does not
  demo has no page. At first sync these modules had no showcase page of their
  own (by module-name match only; some may still render inside another page):
  `ai_assistant_workspace`,
  `combobox`, `date_picker`, `image_attachment_field`, `loading_bar`,
  `message_bar`, `segmented_bar`, `slider`, `spin_button`, `tag_picker`,
  `time_picker`, `upload_file`, and five theming components with no route.
- Four routes are absent from the sidebar and so are never captured:
  `base_theme_selector`, `component_customizer`, `field-context-scope`,
  `typography_customizer`.
- Examples taller than 3200px have clipped screenshots (4 at first sync);
  their markup is complete up to the 12,000-character cap per example.
