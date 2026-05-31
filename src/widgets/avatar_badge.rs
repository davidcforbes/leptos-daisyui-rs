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
/// `bg_class` defaults to `"bg-primary text-primary-content"` — pass any
/// Tailwind colour combo (e.g. `"bg-blue-600 text-white"`) for per-row
/// theming.
#[component]
pub fn AvatarBadge(
    /// Initials to render. Pass `initials_from_name(name)` if not pre-computed.
    #[prop(into)]
    initials: String,
    /// Visual size — Xs/Sm/Md/Lg.
    #[prop(default = AvatarBadgeSize::Md)]
    size: AvatarBadgeSize,
    /// Override background + text colour. Defaults to primary.
    #[prop(into, default = "bg-primary text-primary-content".into())]
    bg_class: String,
) -> impl IntoView {
    let dimensions = size.classes();
    view! {
        <span
            class=move || format!(
                "inline-flex items-center justify-center rounded-full font-semibold {dimensions} {bg_class}"
            )
        >
            {initials}
        </span>
    }
}
