//! Leptos CSR hook wrapping [`Spring`] in a `requestAnimationFrame`-driven
//! signal.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::*;

use super::raf_loop::{start_raf_loop, RafLoopHandle};
use super::spring::Spring;

/// Reactive handle returned by [`use_spring`] / [`use_spring_with_params`].
///
/// Cheap to clone (it is a bundle of `Rc`s and `Copy` signals) — hand it to
/// event handlers and view closures freely.
#[derive(Clone)]
pub struct SpringHandle {
    /// The spring's animated value. Updated once per rendered frame while
    /// the spring is moving toward its target; unchanged while settled.
    /// Read it inside a tracking context (a view closure, `Memo`, `Effect`)
    /// like any other signal.
    pub value: ReadSignal<f32>,
    set_value: WriteSignal<f32>,
    spring: Rc<RefCell<Spring>>,
    loop_handle: Rc<RefCell<Option<RafLoopHandle>>>,
    cancelled: Rc<Cell<bool>>,
}

impl SpringHandle {
    /// Aim the spring at a new `target`. Velocity carries over from
    /// wherever the spring currently is, so a retarget mid-flight stays
    /// visually continuous (see [`Spring::set_target`]). Wakes the
    /// `requestAnimationFrame` loop if it had settled and stopped.
    pub fn set_target(&self, target: f32) {
        self.spring.borrow_mut().set_target(target);
        self.ensure_running();
    }

    /// Whether the spring has reached its target and the animation loop is
    /// idle (see [`Spring::is_settled`]).
    pub fn is_settled(&self) -> bool {
        self.spring.borrow().is_settled()
    }

    /// Starts the rAF loop if it is not already running. Called after every
    /// [`set_target`](Self::set_target) since that is the only way a
    /// settled spring becomes un-settled again.
    fn ensure_running(&self) {
        if self.cancelled.get() {
            return;
        }
        let already_running = self
            .loop_handle
            .borrow()
            .as_ref()
            .map(|h| !h.is_cancelled())
            .unwrap_or(false);
        if already_running {
            return;
        }

        let spring = self.spring.clone();
        let set_value = self.set_value;
        let cancelled = self.cancelled.clone();
        let last_time = Rc::new(Cell::new(js_sys::Date::now()));

        let handle = start_raf_loop(move |now| {
            if cancelled.get() {
                return false;
            }
            let dt_ms = (now - last_time.get()).max(0.0) as f32;
            last_time.set(now);

            let (value, settled) = {
                let mut s = spring.borrow_mut();
                s.step(dt_ms);
                (s.value, s.is_settled())
            };
            // `try_set` is a safe no-op if the owning scope has already
            // been disposed (the signal is gone) — but that alone would
            // not stop this closure from being rescheduled forever, hence
            // the `cancelled` check above on every frame too.
            let _ = set_value.try_set(value);
            !settled
        });
        *self.loop_handle.borrow_mut() = Some(handle);
    }
}

/// A spring-driven `f32` signal with the default snappy tuning — matches
/// [`Spring::new`]: `220.0` stiffness, `26.0` damping (quick, no visible
/// bounce). See [`use_spring_with_params`] for custom tuning.
///
/// [`SpringHandle::value`] only updates while the spring is moving: a
/// `requestAnimationFrame` loop drives it, gated by [`Spring::is_settled`],
/// and stops itself the instant the spring comes to rest — so an idle
/// spring costs nothing per frame. Call [`SpringHandle::set_target`] to
/// send it somewhere new; the loop restarts automatically.
///
/// The rAF loop is torn down when the calling component's reactive scope is
/// disposed (via `on_cleanup`), so a removed component never keeps
/// scheduling frames into a dropped signal.
///
/// CSR-only: internally calls into `web-sys`'s `window()`, so it must run
/// in a browser context (not during SSR).
pub fn use_spring(initial: f32) -> SpringHandle {
    use_spring_with_params(initial, 220.0, 26.0)
}

/// Like [`use_spring`], but with explicit `stiffness` (pull strength) and
/// `damping` (velocity bleed) — see [`Spring::with_params`]. Lower damping
/// relative to stiffness produces a visible overshoot before settling.
pub fn use_spring_with_params(initial: f32, stiffness: f32, damping: f32) -> SpringHandle {
    let spring = Rc::new(RefCell::new(Spring::with_params(
        initial, stiffness, damping,
    )));
    let (value, set_value) = signal(initial);
    let cancelled = Rc::new(Cell::new(false));
    let loop_handle: Rc<RefCell<Option<RafLoopHandle>>> = Rc::new(RefCell::new(None));

    // `Rc<Cell<_>>`/`Rc<RefCell<_>>` are not `Send`/`Sync`, but `on_cleanup`
    // requires both (the reactive graph is generic over native multithreaded
    // use). This hook only ever runs single-threaded (wasm32 in the
    // browser), so `SendWrapper` documents and encodes that assumption
    // rather than working around it silently — same pattern as
    // `components::toolbar`'s `ResizeObserver` cleanup.
    let cleanup_guard = send_wrapper::SendWrapper::new((cancelled.clone(), loop_handle.clone()));
    on_cleanup(move || {
        let (cancelled, loop_handle) = cleanup_guard.take();
        cancelled.set(true);
        if let Some(h) = loop_handle.borrow_mut().take() {
            h.cancel();
        }
    });

    SpringHandle {
        value,
        set_value,
        spring,
        loop_handle,
        cancelled,
    }
}
