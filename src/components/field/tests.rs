use super::*;
use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use std::sync::{Arc, Mutex};

/// Regression coverage for ldui-1jxa: `Field` calls `provide_context` before
/// invoking its `children`, but a plain component-function call gets no
/// owner of its own -- it runs inside whatever `Owner` is already current
/// for the surrounding view. Without an explicit child owner,
/// `provide_context` lands on that *shared* owner, so anything else built
/// under it afterward (a later sibling `Field`/`Input`, or a standalone
/// `Select`/`DatasetSelector`/`EntityTable` filter control that never wraps
/// in `Field` at all) sees the same `FieldContext` and mints the same id --
/// this crate has no native DOM/SSR renderer to read a rendered `id`
/// attribute back in a unit test, but `Input`/`Select`/`Textarea` all pick up
/// `FieldContext` via the exact same `use_context::<FieldContext>()` call
/// these tests make directly, so proving that call is scoped correctly here
/// proves the fix for all three consumers. A browser-level fixture
/// (`tests/field_context_scoping.rs`) exercises the real rendered ids.
mod context_scope {
    use super::*;

    fn build_field(on_child: impl FnOnce() + Send + 'static) -> impl IntoView {
        Field(FieldProps {
            label: Signal::stored(None),
            help_text: Signal::stored(None),
            error: Signal::stored(None),
            success: Signal::stored(None),
            state: Signal::stored(FieldState::default()),
            required: Signal::stored(false),
            class: "",
            label_class: "",
            node_ref: NodeRef::new(),
            children: ToChildren::to_children(move || {
                on_child();
                view! { <input /> }
            }),
        })
    }

    #[test]
    fn field_context_is_confined_to_its_own_children_not_the_calling_owner() {
        let owner = Owner::new();
        owner.with(|| {
            // No `Field` has run in this owner yet.
            assert!(use_context::<FieldContext>().is_none());

            let seen_inside: Arc<Mutex<Option<FieldContext>>> = Arc::new(Mutex::new(None));
            let seen_inside_child = seen_inside.clone();

            let _ = build_field(move || {
                // The exact call `Input`/`Select`/`Textarea` make internally.
                *seen_inside_child.lock().unwrap() = use_context::<FieldContext>();
            });

            let inside = seen_inside.lock().unwrap().clone();
            let inside = inside.expect("Field's own child must see its FieldContext");
            assert!(inside.input_id.starts_with("ld-field-"));

            // The regression: once `Field` returns, the owner that called it
            // -- exactly where a later sibling `Input`, `Select`,
            // `DatasetSelector`, or `EntityTable` column filter would run its
            // own `use_context::<FieldContext>()` -- must not see it.
            assert!(
                use_context::<FieldContext>().is_none(),
                "FieldContext leaked past Field's own children into the calling owner (ldui-1jxa)"
            );
        });
    }

    #[test]
    fn sibling_fields_do_not_see_each_others_context_and_mint_distinct_ids() {
        let owner = Owner::new();
        owner.with(|| {
            let first_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
            let first_id_child = first_id.clone();
            let _ = build_field(move || {
                *first_id_child.lock().unwrap() = use_context::<FieldContext>().map(|f| f.input_id);
            });

            // A later, unrelated standalone control built in this same owner
            // (nothing wraps it in a `Field`) must not inherit the first
            // `Field`'s context -- this is the DatasetSelector / EntityTable
            // filter shape from the office-perf report.
            assert!(use_context::<FieldContext>().is_none());

            let second_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
            let second_id_child = second_id.clone();
            let _ = build_field(move || {
                *second_id_child.lock().unwrap() =
                    use_context::<FieldContext>().map(|f| f.input_id);
            });

            let first = first_id
                .lock()
                .unwrap()
                .clone()
                .expect("first Field's child ran");
            let second = second_id
                .lock()
                .unwrap()
                .clone()
                .expect("second Field's child ran");
            assert_ne!(first, second, "sibling Fields must mint distinct ids");

            // Neither Field's context is visible from the shared calling
            // owner after both have returned.
            assert!(use_context::<FieldContext>().is_none());
        });
    }
}

// FieldState tests
#[test]
fn test_field_state_default() {
    let val = FieldState::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_field_state_error() {
    let val = FieldState::Error;
    assert_eq!(val.as_str(), "error");
}

#[test]
fn test_field_state_success() {
    let val = FieldState::Success;
    assert_eq!(val.as_str(), "success");
}

#[test]
fn test_field_state_warning() {
    let val = FieldState::Warning;
    assert_eq!(val.as_str(), "warning");
}

#[test]
fn test_field_state_clone() {
    let v1 = FieldState::Error;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_field_state_debug() {
    let val = FieldState::Success;
    assert!(format!("{:?}", val).contains("Success"));
}

// Comprehensive coverage test
#[test]
fn test_all_field_states_return_valid_classes() {
    let variants = vec![
        (FieldState::Default, ""),
        (FieldState::Error, "error"),
        (FieldState::Success, "success"),
        (FieldState::Warning, "warning"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
