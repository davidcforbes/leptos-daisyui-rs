//! Click-ripple primitive used by opt-in components (Button, Card).
//!
//! [`use_ripple`] hands out a [`RippleHandle`] that any component can hook
//! into its own `on:click` handler. The companion [`RippleOverlay`]
//! component renders the visible spans, each of which auto-dismisses on
//! `animationend`. The animation itself lives in
//! [`UiAnimationsPreamble`](crate::tokens::UiAnimationsPreamble) — the
//! Rust side here only manages instance bookkeeping.

use leptos::prelude::*;

/// One active ripple. Coordinates are offsets within the host element so
/// the rendered `<span>` lands directly under the click.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RippleInstance {
    /// Monotonic id used as the `<For>` key and as the dismiss handle.
    pub id: u64,
    /// `MouseEvent::offset_x` in CSS pixels.
    pub x: i32,
    /// `MouseEvent::offset_y` in CSS pixels.
    pub y: i32,
}

/// Reactive handle returned by [`use_ripple`]. Cheap to copy.
#[derive(Clone, Copy)]
pub struct RippleHandle {
    /// Call from `on:click` to spawn a new ripple at the event coords.
    pub trigger: Callback<web_sys::MouseEvent>,
    /// Read by [`RippleOverlay`] to render the live ripple spans.
    pub instances: ReadSignal<Vec<RippleInstance>>,
    /// Removes the ripple with the given id. Called from `on:animationend`.
    pub dismiss: Callback<u64>,
}

/// Create the bookkeeping for click ripples. Returns a handle whose
/// `trigger` should be invoked from `on:click` and whose `instances`
/// should be rendered through [`RippleOverlay`].
pub fn use_ripple() -> RippleHandle {
    let (instances, set_instances) = signal(Vec::<RippleInstance>::new());
    let next_id = RwSignal::new(0_u64);

    let trigger = Callback::new(move |ev: web_sys::MouseEvent| {
        let id = next_id.get_untracked();
        next_id.set(id.wrapping_add(1));
        let inst = RippleInstance {
            id,
            x: ev.offset_x(),
            y: ev.offset_y(),
        };
        set_instances.update(|v| push_instance(v, inst));
    });

    let dismiss = Callback::new(move |id: u64| {
        set_instances.update(|v| dismiss_instance(v, id));
    });

    RippleHandle {
        trigger,
        instances,
        dismiss,
    }
}

/// Renders the live ripple `<span>` elements. Drop this just inside the
/// element that triggered the ripples (must be `position: relative` —
/// the `ld-ripple-host` utility takes care of that).
#[component]
pub fn RippleOverlay(
    /// Handle from [`use_ripple`].
    handle: RippleHandle,
) -> impl IntoView {
    view! {
        <For
            each=move || handle.instances.get()
            key=|r| r.id
            children=move |r| view! {
                <span
                    class="ld-ripple-element"
                    style:left=format!("{}px", r.x)
                    style:top=format!("{}px", r.y)
                    on:animationend=move |_| handle.dismiss.run(r.id)
                />
            }
        />
    }
}

fn push_instance(v: &mut Vec<RippleInstance>, inst: RippleInstance) {
    v.push(inst);
}

fn dismiss_instance(v: &mut Vec<RippleInstance>, id: u64) {
    v.retain(|r| r.id != id);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(id: u64, x: i32, y: i32) -> RippleInstance {
        RippleInstance { id, x, y }
    }

    #[test]
    fn push_instance_appends_in_order() {
        let mut v = Vec::new();
        push_instance(&mut v, make(0, 10, 20));
        push_instance(&mut v, make(1, 30, 40));
        assert_eq!(v, vec![make(0, 10, 20), make(1, 30, 40)]);
    }

    #[test]
    fn dismiss_removes_by_id_only() {
        let mut v = vec![make(0, 0, 0), make(1, 1, 1), make(2, 2, 2)];
        dismiss_instance(&mut v, 1);
        assert_eq!(v, vec![make(0, 0, 0), make(2, 2, 2)]);
    }

    #[test]
    fn dismiss_unknown_id_is_noop() {
        let mut v = vec![make(0, 0, 0), make(1, 1, 1)];
        dismiss_instance(&mut v, 999);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn dismiss_then_repush_works() {
        let mut v = vec![make(0, 0, 0)];
        dismiss_instance(&mut v, 0);
        push_instance(&mut v, make(1, 5, 5));
        assert_eq!(v, vec![make(1, 5, 5)]);
    }

    #[test]
    fn instance_struct_is_copy() {
        let a = make(7, 1, 2);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn instance_equality_is_structural() {
        assert_eq!(make(0, 1, 2), make(0, 1, 2));
        assert_ne!(make(0, 1, 2), make(1, 1, 2));
        assert_ne!(make(0, 1, 2), make(0, 9, 2));
        assert_ne!(make(0, 1, 2), make(0, 1, 9));
    }
}
