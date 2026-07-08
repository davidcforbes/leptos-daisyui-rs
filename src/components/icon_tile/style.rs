/// Semantic color used for IconTile's background tint and icon foreground.
///
/// The same variant set drives both the `bg` and `fg` props, but each prop
/// maps it through a different method ([`as_bg_class`](Self::as_bg_class) vs
/// [`as_fg_class`](Self::as_fg_class)) so a tile can freely mix a subtle
/// tinted background with an independently-chosen solid icon color -- e.g.
/// `bg=IconTileColor::Error fg=IconTileColor::Neutral`. Ported from d2d-ui's
/// `IconTile`, which stored independent `D2D1_COLOR_F` values for `bg`/`fg`
/// rather than deriving one from the other.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum IconTileColor {
    /// Neutral color
    Neutral,

    /// Primary theme color (default -- matches d2d-ui's subtle-accent tile)
    #[default]
    Primary,

    /// Secondary theme color
    Secondary,

    /// Accent theme color
    Accent,

    /// Info color
    Info,

    /// Success color
    Success,

    /// Warning color
    Warning,

    /// Error color
    Error,
}

impl IconTileColor {
    /// CSS class for use as the tile's tinted background fill.
    pub fn as_bg_class(&self) -> &'static str {
        match self {
            IconTileColor::Neutral => "bg-neutral/10",
            IconTileColor::Primary => "bg-primary/10",
            IconTileColor::Secondary => "bg-secondary/10",
            IconTileColor::Accent => "bg-accent/10",
            IconTileColor::Info => "bg-info/10",
            IconTileColor::Success => "bg-success/10",
            IconTileColor::Warning => "bg-warning/10",
            IconTileColor::Error => "bg-error/10",
        }
    }

    /// CSS class for use as the icon glyph's solid foreground color.
    pub fn as_fg_class(&self) -> &'static str {
        match self {
            IconTileColor::Neutral => "text-neutral",
            IconTileColor::Primary => "text-primary",
            IconTileColor::Secondary => "text-secondary",
            IconTileColor::Accent => "text-accent",
            IconTileColor::Info => "text-info",
            IconTileColor::Success => "text-success",
            IconTileColor::Warning => "text-warning",
            IconTileColor::Error => "text-error",
        }
    }
}

/// Size variants for the IconTile component.
///
/// Each variant sets a fixed width/height plus a matching icon text size, so
/// a single class list controls both the tile's footprint and how large its
/// centered glyph renders.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum IconTileSize {
    /// Extra small tile (24px)
    Xs,

    /// Small tile (32px)
    Sm,

    /// Medium tile (40px, default)
    #[default]
    Md,

    /// Large tile (48px)
    Lg,

    /// Extra large tile (64px)
    Xl,
}

impl IconTileSize {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            IconTileSize::Xs => "w-6 h-6 text-xs",
            IconTileSize::Sm => "w-8 h-8 text-sm",
            IconTileSize::Md => "w-10 h-10 text-base",
            IconTileSize::Lg => "w-12 h-12 text-lg",
            IconTileSize::Xl => "w-16 h-16 text-2xl",
        }
    }
}

/// CSS class for the tile's corner radius: `rounded-full` for circles, `rounded-lg` for rounded squares.
///
/// Mirrors d2d-ui's `with_corner_radius(size / 2.0)` circle override.
pub fn radius_class(circle: bool) -> &'static str {
    if circle { "rounded-full" } else { "rounded-lg" }
}
