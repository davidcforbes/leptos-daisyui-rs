//! Shared demo fixtures (ldui-rz43): the examples the leptos-daisyui-rs demo
//! shows for a component, as plain data.
//!
//! The demo renders these lists through its Leptos components, and
//! 4iiz-kit's gallery renders the same lists through its askama components,
//! so pixelproof-parity compares identical inputs: one list, two renderers.
//! A hand-copied example list on either side drifts silently, and a parity
//! comparison of two different lists passes or fails for the wrong reason.
//!
//! Each component module holds one [`FixtureSection`] per demo section, in
//! page order, and a `SECTIONS` array of them. Every example writes out all
//! of its variant props, component defaults included, so a renderer never
//! has to know ldui's defaults. Labels are the demo's English text; a
//! translating renderer maps them to its own catalogue.
//!
//! Only the variant sections are fixtures. The demos' reactive sections
//! (counters, toggles, form fixtures) are behaviour, not data, and stay in
//! the demo.

pub mod alert;
pub mod badge;
pub mod button;

/// One titled demo section: the heading the demo prints and its examples in
/// render order.
#[derive(Debug, PartialEq)]
pub struct FixtureSection<T: 'static> {
    /// The section heading, and the demo's `data-demo-section` value.
    pub title: &'static str,
    /// The examples, in the order the demo renders them.
    pub examples: &'static [T],
}

#[cfg(test)]
mod tests {
    use super::FixtureSection;
    use std::collections::BTreeSet;

    /// Titles and non-empty examples of one component's `SECTIONS`.
    fn check<T>(component: &str, sections: &[FixtureSection<T>]) {
        let titles: BTreeSet<&str> = sections.iter().map(|s| s.title).collect();
        assert_eq!(
            titles.len(),
            sections.len(),
            "{component}: section titles must be unique, since they name the demo sections"
        );
        for section in sections {
            assert!(
                !section.title.trim().is_empty(),
                "{component}: a section has a blank title"
            );
            assert!(
                !section.examples.is_empty(),
                "{component}: section {:?} has no examples",
                section.title
            );
        }
    }

    #[test]
    fn every_section_is_titled_uniquely_and_non_empty() {
        check("badge", &super::badge::SECTIONS);
        check("button", &super::button::SECTIONS);
        check("alert", &super::alert::SECTIONS);
    }
}
