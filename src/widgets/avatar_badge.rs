//! Avatar badge — initials in a coloured circle.
//!
//! Shared component used by the topbar avatar, user-list cards (e.g.
//! /enrollment), and chat bubbles. Falls back to a single character when
//! initials are missing.
//!
//! Composes the daisyUI Avatar component so the daisyUI styling stays consistent.

use leptos::prelude::*;

/// Visual size of the avatar circle. The component maps each variant to a
/// fixed Tailwind size class so callers don't repeat magic numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarBadgeSize {
    /// 1.5rem / 24px — for table rows and tight chat bubbles.
    Xs,
    /// 2rem / 32px — for inline chips.
    Sm,
    /// 2.5rem / 40px — for the topbar.
    Md,
    /// 4rem / 64px — for the right-rail user profile.
    Lg,
}

impl AvatarBadgeSize {
    fn classes(self) -> &'static str {
        match self {
            AvatarBadgeSize::Xs => "w-6 h-6 text-[10px]",
            AvatarBadgeSize::Sm => "w-8 h-8 text-xs",
            AvatarBadgeSize::Md => "w-10 h-10 text-sm",
            AvatarBadgeSize::Lg => "w-16 h-16 text-xl",
        }
    }
}

/// The deterministic name→color palette: daisyUI semantic `bg-*` paired with
/// its matching `*-content` text token, so every entry is contrast-safe in
/// every theme by construction (the pairs are the theme's own guarantee).
///
/// Consumers were each re-deriving "hash the name into five colors" — seven
/// copies in one app — and drifting on contrast and class names. The
/// framework owns the palette now; the count and order are part of the
/// visual contract, so append rather than reorder when extending.
pub const NAME_PALETTE: [&str; 8] = [
    "bg-primary text-primary-content",
    "bg-secondary text-secondary-content",
    "bg-accent text-accent-content",
    "bg-info text-info-content",
    "bg-success text-success-content",
    "bg-warning text-warning-content",
    "bg-error text-error-content",
    "bg-neutral text-neutral-content",
];

/// Deterministic, theme-contrast-safe color classes for a display name:
/// the same name always maps to the same [`NAME_PALETTE`] entry, on every
/// screen and in every app. FNV-1a over the name's bytes — stable across
/// runs and platforms (no `DefaultHasher` seed randomness).
///
/// An empty name gets the neutral (last) entry rather than hashing "".
pub fn name_color_class(name: &str) -> &'static str {
    if name.is_empty() {
        return NAME_PALETTE[NAME_PALETTE.len() - 1];
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in name.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    NAME_PALETTE[(hash % NAME_PALETTE.len() as u64) as usize]
}

/// Returns up to two uppercase initials from a full name.
///
/// - "John Smith" → "JS"
/// - "Cher"      → "C"
/// - empty       → "?"
pub fn initials_from_name(full_name: &str) -> String {
    let parts: Vec<&str> = full_name.split_whitespace().collect();
    match parts.as_slice() {
        [] => "?".into(),
        [single] => single
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default(),
        _ => {
            let first = parts.first().and_then(|p| p.chars().next()).unwrap_or('?');
            let last = parts.last().and_then(|p| p.chars().next()).unwrap_or('?');
            format!("{}{}", first.to_uppercase(), last.to_uppercase())
        }
    }
}

/// Avatar badge with initials.
///
/// Colour resolution, most explicit first:
/// 1. `bg_class`, when supplied — any Tailwind combo for bespoke theming;
/// 2. `name`, when supplied — the deterministic [`name_color_class`]
///    palette, so "Maria Gonzalez" is the same colour on every screen of
///    every app with no caller-side hash;
/// 3. neither — the primary default, exactly as before.
#[component]
pub fn AvatarBadge(
    /// Initials to render. Pass `initials_from_name(name)` if not pre-computed.
    #[prop(into)]
    initials: String,
    /// Visual size — Xs/Sm/Md/Lg.
    #[prop(default = AvatarBadgeSize::Md)]
    size: AvatarBadgeSize,
    /// Full display name driving the deterministic colour palette
    /// ([`name_color_class`]). Ignored when `bg_class` is supplied.
    #[prop(into, optional)]
    name: Option<String>,
    /// Override background + text colour, trumping `name`.
    #[prop(into, optional)]
    bg_class: Option<String>,
) -> impl IntoView {
    let dimensions = size.classes();
    let color = bg_class.unwrap_or_else(|| {
        name.as_deref()
            .map(name_color_class)
            .unwrap_or("bg-primary text-primary-content")
            .to_string()
    });
    view! {
        <span
            class=move || format!(
                "inline-flex items-center justify-center rounded-full font-semibold {dimensions} {color}"
            )
        >
            {initials}
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── name_color_class ──

    #[test]
    fn same_name_always_maps_to_the_same_class() {
        let a = name_color_class("Maria Gonzalez");
        for _ in 0..10 {
            assert_eq!(name_color_class("Maria Gonzalez"), a);
        }
    }

    #[test]
    fn the_hash_is_stable_across_releases() {
        // These exact assignments are part of the visual contract: a person
        // keeps their colour across app versions. If this test fails, the
        // hash or palette order changed — that is a breaking visual change,
        // not a refactor.
        assert_eq!(name_color_class("Maria Gonzalez"), NAME_PALETTE[3]);
        assert_eq!(name_color_class("John Smith"), NAME_PALETTE[7]);
    }

    #[test]
    fn different_names_spread_across_the_palette() {
        let names = [
            "Maria Gonzalez",
            "John Smith",
            "Aiko Tanaka",
            "Omar Haddad",
            "Priya Patel",
            "Lars Nielsen",
            "Chen Wei",
            "Fatima Zahra",
            "Diego Rivera",
            "Anna Kowalska",
        ];
        let distinct: std::collections::HashSet<&str> =
            names.iter().map(|n| name_color_class(n)).collect();
        assert!(
            distinct.len() >= 4,
            "10 names should hit at least 4 palette entries, got {}",
            distinct.len()
        );
    }

    #[test]
    fn empty_name_gets_the_neutral_entry() {
        assert_eq!(name_color_class(""), "bg-neutral text-neutral-content");
    }

    #[test]
    fn every_palette_entry_pairs_bg_with_its_content_token() {
        // The bg-X / text-X-content pairing is what makes the palette
        // contrast-safe in every daisyUI theme by construction.
        for entry in NAME_PALETTE {
            let bg = entry
                .split_whitespace()
                .find(|c| c.starts_with("bg-"))
                .expect("entry has a bg class");
            let text = entry
                .split_whitespace()
                .find(|c| c.starts_with("text-"))
                .expect("entry has a text class");
            let color = bg.trim_start_matches("bg-");
            assert_eq!(
                text,
                format!("text-{color}-content"),
                "palette entry {entry:?} must pair bg with its own content token"
            );
        }
    }

    // ── initials_from_name (previously untested here) ──

    #[test]
    fn initials_two_words() {
        assert_eq!(initials_from_name("John Smith"), "JS");
    }

    #[test]
    fn initials_single_word() {
        assert_eq!(initials_from_name("Cher"), "C");
    }

    #[test]
    fn initials_empty() {
        assert_eq!(initials_from_name(""), "?");
    }

    #[test]
    fn initials_three_words_take_first_and_last() {
        assert_eq!(initials_from_name("Maria del Carmen"), "MC");
    }
}
