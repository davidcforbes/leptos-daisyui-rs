use crate::components::button::ButtonSize;

/// # Toolbar Size Variants
///
/// Controls the size of each toolbar button (mapped to a daisyUI [`ButtonSize`])
/// and the join-row density. daisyUI's `join` layout has no gap between items
/// (borders touch), so this only scales the buttons themselves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarSize {
    /// Extra small buttons — dense command strips (e.g. an editor format bar).
    Xs,
    /// Small buttons.
    Sm,
    /// Medium buttons (default).
    #[default]
    Md,
    /// Large buttons.
    Lg,
}

impl ToolbarSize {
    /// The [`ButtonSize`] used for each toolbar item button.
    pub fn button_size(&self) -> ButtonSize {
        match self {
            ToolbarSize::Xs => ButtonSize::Xs,
            ToolbarSize::Sm => ButtonSize::Sm,
            ToolbarSize::Md => ButtonSize::Md,
            ToolbarSize::Lg => ButtonSize::Lg,
        }
    }
}
