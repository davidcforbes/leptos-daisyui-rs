//! Declarative helpers for defining typed page contracts.

/// Defines a typed page-contract constant without generating behavior or I/O.
///
/// The explicit `: v2` form expands to
/// [`PageContractV2`](crate::patterns::PageContractV2). The unversioned form
/// remains a legacy [`PageContract`](crate::patterns::PageContract) declaration
/// and does not claim v2 validation or export compatibility.
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
        $visibility:vis $name:ident: v2 {
            contract_version: $contract_version:literal,
            id: $id:literal,
            title: $title:literal,
            owner: $owner:literal,
            delivery: $delivery:ident,
            archetype: $archetype:ident,
            route: $route:literal,
            source: $source:expr,
            dataset: $dataset:expr,
            data: $data:expr,
            state: $state:expr,
            mutations: $mutations:expr,
            realtime: $realtime:expr,
            capabilities: $capabilities:expr,
            responsive: $responsive:expr,
            accessibility: $accessibility:expr,
            presentation_states: $presentation_states:expr,
            test_lanes: $test_lanes:expr,
            budgets: $budgets:expr,
            compatibility: $compatibility:expr,
            baselines: $baselines:expr,
            $(,)?
        }
    ) => {
        $visibility const $name: $crate::patterns::PageContractV2 =
            $crate::patterns::PageContractV2 {
                contract_version: $contract_version,
                id: $id,
                title: $title,
                owner: $owner,
                delivery: $crate::patterns::PageDelivery::$delivery,
                archetype: $crate::patterns::PageArchetype::$archetype,
                route: $route,
                source: $source,
                dataset: $dataset,
                data: $data,
                state: $state,
                mutations: $mutations,
                realtime: $realtime,
                capabilities: $capabilities,
                responsive: $responsive,
                accessibility: $accessibility,
                presentation_states: $presentation_states,
                test_lanes: $test_lanes,
                budgets: $budgets,
                compatibility: $compatibility,
                baselines: $baselines,
            };
    };
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

/// Defines a function that returns typed [`EntityColumn`](crate::components::EntityColumn)
/// declarations for one row type.
///
/// The macro deliberately generates only an ordinary Rust function. It keeps
/// the row type adjacent to the column expressions and lets the compiler reject
/// an accessor or renderer for a different entity without generating views or
/// business logic.
///
/// ```compile_fail
/// use leptos_daisyui_rs::{components::EntityColumn, entity_columns};
///
/// struct Client { name: String }
/// struct Matter { title: String }
///
/// entity_columns! {
///     fn client_columns() -> Client => [
///         EntityColumn::text("name", "Name", |matter: &Matter| matter.title.clone()),
///     ]
/// }
/// ```
#[macro_export]
macro_rules! entity_columns {
    (
        $visibility:vis fn $name:ident(
            $($argument:ident : $argument_type:ty),* $(,)?
        ) -> $row:ty => [
            $($column:expr),* $(,)?
        ]
    ) => {
        $visibility fn $name(
            $($argument: $argument_type),*
        ) -> ::std::vec::Vec<$crate::components::EntityColumn<$row>> {
            ::std::vec![$($column),*]
        }
    };
}
