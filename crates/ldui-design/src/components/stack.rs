/// # Stack Placement Variants
#[derive(Clone, Debug, Default, PartialEq)]
pub enum StackPlacement {
    /// Top vertical placement
    Top,

    /// Bottom vertical placement
    #[default]
    Bottom,

    /// Start horizontal placement
    Start,

    /// End horizontal placement
    End,
}

impl StackPlacement {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "stack-top",
            Self::Bottom => "stack-bottom",
            Self::Start => "stack-start",
            Self::End => "stack-end",
        }
    }
}
