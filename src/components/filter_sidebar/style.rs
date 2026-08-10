/// # Sidebar Side
///
/// Which page edge a [`FilterSidebar`](super::FilterSidebar) is docked
/// against. Filed as `ldui-vh6`: 4iiz-Office's Client Coordinator workspace
/// needs a collapsible *Assistant* panel on the right that mirrors the filter
/// panel on the left, and the panel was left-oriented in four separate places
/// with no way to say otherwise.
///
/// ## What actually flips
///
/// Four things, each a pure function on this enum so the mapping is testable
/// and cannot rot silently:
///
/// - [`as_border_class`](Self::as_border_class) — the hairline goes on the
///   panel's **inner** edge, the one facing the content it sits beside.
/// - [`chevron_name`](Self::chevron_name) — the arrow points the way the
///   panel would *move*, which depends on both the side and the collapsed
///   state, so this one takes `collapsed` rather than being a plain mapping.
/// - [`as_header_class`](Self::as_header_class) — the toggle button belongs at
///   the inner edge too, next to the content it reveals, not glued to the
///   window frame.
/// - [`as_rail_title_class`](Self::as_rail_title_class) — the collapsed rail's
///   vertical title reads bottom-to-top on a left edge and top-to-bottom on a
///   right one, which is the convention every IDE tool-window and spreadsheet
///   side-tab already follows.
///
/// ## What deliberately does NOT flip
///
/// The filter-search box keeps its magnifier on the left and its `pl-[30px]`
/// on the left. That inset is **reading direction**, not panel orientation: a
/// search field looks the same whichever edge of the screen it is near, and
/// mirroring it would put the icon where the caret goes. Likewise the header's
/// `px-3`, the toggle's `rounded-md` and the rail's `items-center` are
/// symmetric already and have nothing to mirror.
///
/// ## `#[non_exhaustive]`, unlike [`ShiftState`](crate::components::ShiftState)
///
/// This is a **knob the consumer picks from**, in the same family as
/// [`RosterDensity`](crate::components::RosterDensity), which carries the
/// attribute for the same reason. Nothing hands a `SidebarSide` back — there
/// is no callback that yields one and no labelling function a consumer has to
/// supply per variant — so no consumer is ever *forced* to match on it, and a
/// `_ =>` arm in one that chooses to is a harmless layout fallback rather than
/// the wrong string read aloud to a screen reader.
///
/// That matters because a third variant is genuinely plausible: logical
/// `Start`/`End` for RTL, where the panel follows the document's writing
/// direction instead of the viewport's geometry. Adding those should be a
/// minor release, and without `#[non_exhaustive]` it could not be.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SidebarSide {
    /// Docked against the left edge — the original and only behaviour before
    /// `ldui-vh6`, and the default, so every existing caller is untouched.
    #[default]
    Left,

    /// Docked against the right edge: the mirror image of [`Left`](Self::Left)
    /// in all four respects listed on the type.
    Right,
}

impl SidebarSide {
    /// Hairline border on the panel's INNER edge — the edge facing the page
    /// content, which is the one that reads as the seam between the two.
    /// Putting it on the outer edge draws a line against the window frame,
    /// where there is nothing to separate.
    pub fn as_border_class(&self) -> &'static str {
        match self {
            SidebarSide::Left => "border-r",
            SidebarSide::Right => "border-l",
        }
    }

    /// Icon name for the toggle, given whether the panel is currently
    /// collapsed.
    ///
    /// The arrow points where the panel would *go*, not where it is: a
    /// collapsed left panel expands rightwards, so it shows `chevron-right`;
    /// expanded, it would collapse leftwards, so `chevron-left`. A right panel
    /// is the exact mirror. This is why the mapping takes `collapsed` instead
    /// of being a bare per-variant lookup — side alone does not determine it.
    pub fn chevron_name(&self, collapsed: bool) -> &'static str {
        match (self, collapsed) {
            (SidebarSide::Left, true) => "chevron-right",
            (SidebarSide::Left, false) => "chevron-left",
            (SidebarSide::Right, true) => "chevron-left",
            (SidebarSide::Right, false) => "chevron-right",
        }
    }

    /// Flex-direction class for the 52px header row, so the toggle button
    /// lands on the panel's inner edge and the title takes the outer one.
    ///
    /// [`Left`](Self::Left) returns `""` rather than the equivalent
    /// `flex-row`, and that is deliberate: this component is consumed as a
    /// path dependency by sibling repos, and the default orientation's emitted
    /// `class` attribute has to stay byte-for-byte what it was. `flex-row` is
    /// a visual no-op and a textual change, which is the worst combination —
    /// nothing to see, and a diff in someone else's DOM snapshot.
    /// [`join_side_class`] does the empty-aware join.
    pub fn as_header_class(&self) -> &'static str {
        match self {
            SidebarSide::Left => "",
            SidebarSide::Right => "flex-row-reverse",
        }
    }

    /// Writing-mode and rotation for the collapsed rail's vertical title.
    ///
    /// `writing-mode: vertical-rl` alone runs the text top-to-bottom with the
    /// glyphs turned 90° clockwise — the natural reading for a label on a
    /// RIGHT edge. Adding `rotate-180` turns it bottom-to-top, which is the
    /// natural reading for a LEFT edge. There is no `origin-*` involved: the
    /// span is rotated about its own centre, so a half turn needs no origin
    /// correction and adding one would shift the label off the rail.
    pub fn as_rail_title_class(&self) -> &'static str {
        match self {
            SidebarSide::Left => "[writing-mode:vertical-rl] [text-orientation:mixed] rotate-180",
            SidebarSide::Right => "[writing-mode:vertical-rl] [text-orientation:mixed]",
        }
    }
}

/// Joins a base class list with an orientation class, skipping the separator
/// when the orientation contributes nothing.
///
/// Exists so [`SidebarSide::as_header_class`] can return `""` for
/// [`SidebarSide::Left`] and still compose — see that method for why the
/// default orientation must emit the unchanged string rather than an
/// equivalent one.
pub fn join_side_class(base: &str, side_class: &str) -> String {
    if side_class.is_empty() {
        base.to_string()
    } else {
        format!("{base} {side_class}")
    }
}
