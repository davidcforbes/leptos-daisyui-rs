//! Declarative helpers for defining typed page contracts.

/// Defines a [`PageContract`](crate::patterns::PageContract) constant.
///
/// Typed table columns reject a callback for a different row type, so a page
/// contract cannot drift from the entity model it renders.
///
/// ```compile_fail
/// use leptos_daisyui_rs::components::EntityColumn;
///
/// struct Client {
///     name: String,
/// }
///
/// let _: EntityColumn<Client> =
///     EntityColumn::text("client", "Client", |wrong: &String| wrong.clone());
/// ```
#[macro_export]
macro_rules! page_contract {
    (
        $visibility:vis $name:ident {
            id: $id:literal,
            route: $route:literal,
            pattern: $pattern:expr,
            dataset: $dataset:expr,
            local_state: [$($local_state:literal),* $(,)?],
            required_states: [$($state:ident),* $(,)?],
            breakpoints: [$($breakpoint:ident),* $(,)?],
            $(,)?
        }
    ) => {
        $visibility const $name: $crate::patterns::PageContract =
            $crate::patterns::PageContract {
                id: $id,
                route: $route,
                pattern: $pattern,
                dataset: $dataset,
                local_state: &[$($local_state),*],
                required_states: &[$($crate::patterns::PageState::$state),*],
                breakpoints: &[$($crate::patterns::PageBreakpoint::$breakpoint),*],
            };
    };
}

/// Defines a typed local-filter schema and verifies its named fields at compile time.
///
/// A misspelled or undeclared filter field is a compile error, so a page
/// contract cannot silently promise a control its filter model does not own.
///
/// ```compile_fail
/// use leptos_daisyui_rs::filter_schema;
///
/// pub struct Filters {
///     pub search: String,
/// }
///
/// filter_schema! {
///     pub FILTERS: Filters {
///         dataset_selector: "office",
///         filters: [search, status],
///     }
/// }
/// ```
#[macro_export]
macro_rules! filter_schema {
    (
        $visibility:vis $name:ident: $filter_state:ty {
            dataset_selector: $dataset_selector:literal,
            filters: [$($filter:ident),* $(,)?],
            $(,)?
        }
    ) => {
        $visibility const $name: $crate::patterns::FilterSchema<$filter_state> =
            $crate::patterns::FilterSchema::new(
                $dataset_selector,
                &[$(stringify!($filter)),*],
            );

        const _: () = {
            #[allow(dead_code)]
            fn assert_filter_fields(value: &$filter_state) {
                $(let _ = &value.$filter;)*
            }
        };
    };
}
