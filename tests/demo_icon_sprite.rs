//! The showcase's inline icon sprite covers every symbol `Icon` can ask for
//! (ldui-kcs7).
//!
//! `Icon` renders `<use href="#<symbol>">` against a sprite the host page
//! inlines. The showcase inlined none, so every `Icon` on every demo page
//! drew an empty box, and nothing failed: an unresolved `<use>` is not an
//! error anywhere. `demo/icons.svg` is generated from Lucide by
//! `demo/build-icon-sprite.mjs`; this test fails when the map gains a symbol
//! the sprite lacks, or when the page stops inlining it.

use leptos_daisyui_rs::components::{UNRESOLVED_SYMBOL, lucide_sprite_names, lucide_to_sprite};
use std::collections::BTreeSet;

fn demo_file(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("demo")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn sprite_symbols(sprite: &str) -> BTreeSet<String> {
    sprite
        .split("<symbol id=\"")
        .skip(1)
        .map(|rest| rest.split('"').next().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn every_mapped_symbol_is_in_the_showcase_sprite() {
    let symbols = sprite_symbols(&demo_file("icons.svg"));
    let missing: Vec<String> = lucide_sprite_names()
        .into_iter()
        .map(|name| format!("{name} -> {}", lucide_to_sprite(name)))
        .filter(|pair| !symbols.contains(pair.rsplit(" -> ").next().unwrap()))
        .collect();
    assert!(
        missing.is_empty(),
        "demo/icons.svg lacks symbols the Icon map uses; run \
         `node build-icon-sprite.mjs` in demo/: {missing:?}"
    );
}

#[test]
fn the_blank_fallback_stays_empty() {
    // `circle` maps to the blank glyph on purpose, and an unmapped name falls
    // back to it, so it must draw nothing.
    let sprite = demo_file("icons.svg");
    let open = format!("<symbol id=\"{UNRESOLVED_SYMBOL}\"");
    let start = sprite.find(&open).expect("the blank symbol exists");
    let tag_end = start + sprite[start..].find('>').unwrap() + 1;
    assert!(
        sprite[tag_end..].starts_with("</symbol>"),
        "the `{UNRESOLVED_SYMBOL}` symbol must be empty"
    );
}

#[test]
fn every_symbol_carries_its_own_stroke() {
    // A `<use>` clone inherits from the `<use>`, never from <defs>, so the
    // stroke must sit on each <symbol> or every glyph renders filled black.
    let sprite = demo_file("icons.svg");
    for tag in sprite.split("<symbol ").skip(1) {
        let tag = &tag[..tag.find('>').unwrap()];
        assert!(
            tag.contains("fill=\"none\"") && tag.contains("stroke=\"currentColor\""),
            "a symbol lost its stroke attributes: <symbol {tag}>"
        );
    }
}

#[test]
fn the_showcase_page_inlines_the_sprite() {
    // Inline, not an external `<use href="icons.svg#id">`: Firefox has never
    // resolved an external `<use>`.
    let index = demo_file("index.html");
    assert!(
        index.contains(r#"<link data-trunk rel="inline" href="icons.svg" />"#),
        "demo/index.html must inline icons.svg"
    );
}
