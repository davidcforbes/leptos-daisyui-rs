//! Shared `requestAnimationFrame` driver for "animate only while un-settled"
//! loops.
//!
//! [`start_raf_loop`] is the is-animating gate primitive behind
//! [`use_spring`](super::use_spring). It self-reschedules via
//! `requestAnimationFrame` for as long as the supplied `on_frame` callback
//! returns `true`, and stops — without scheduling another frame — the
//! instant it returns `false`. A future duration-based `Transition<T>`/
//! `Lerp` hook is expected to reuse this same driver rather than hand-roll
//! another rAF trampoline.
//!
//! Deliberately excluded from unit tests (per the crate's testing
//! conventions, rAF plumbing needs a browser and is exercised by the demo
//! app instead) — [`crate::motion::spring`] carries the numeric coverage.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::request_animation_frame;

/// A cancellable handle to a loop started by [`start_raf_loop`].
///
/// Cloning shares the same cancellation flag: cancelling any clone stops the
/// loop for all of them.
#[derive(Clone)]
pub struct RafLoopHandle {
    cancelled: Rc<Cell<bool>>,
}

impl RafLoopHandle {
    /// Stops the loop before its next scheduled frame. Safe to call more
    /// than once, and safe to call after the loop has already stopped on
    /// its own (its `on_frame` callback returned `false`).
    pub fn cancel(&self) {
        self.cancelled.set(true);
    }

    /// Whether [`cancel`](Self::cancel) has been called, or the loop's
    /// `on_frame` callback has already returned `false`.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
}

/// Starts a `requestAnimationFrame`-driven loop that calls `on_frame(now_ms)`
/// every frame, stopping as soon as it returns `false` or the returned
/// [`RafLoopHandle`] is cancelled.
///
/// `now_ms` comes from `js_sys::Date::now()` (wall-clock milliseconds)
/// rather than the browser's high-resolution rAF timestamp, because
/// `leptos`'s [`request_animation_frame`] helper does not forward it to the
/// callback. Callers that need a per-frame `dt` should track the previous
/// timestamp themselves (see `use_spring`'s implementation).
///
/// This function does **not** register an `on_cleanup` hook itself — a loop
/// may legitimately be started and stopped many times within one
/// component's lifetime (e.g. once per `set_target` call on a settled
/// spring), and re-registering `on_cleanup` on every restart would
/// accumulate stale cleanup callbacks for the life of the scope. Callers
/// that own a long-lived loop (like [`use_spring`](super::use_spring))
/// should instead register a single `on_cleanup` at construction time that
/// cancels whichever [`RafLoopHandle`] is current.
pub fn start_raf_loop(on_frame: impl FnMut(f64) -> bool + 'static) -> RafLoopHandle {
    let cancelled = Rc::new(Cell::new(false));
    let on_frame: Rc<RefCell<dyn FnMut(f64) -> bool>> = Rc::new(RefCell::new(on_frame));
    schedule_frame(on_frame, cancelled.clone());
    RafLoopHandle { cancelled }
}

fn schedule_frame(on_frame: Rc<RefCell<dyn FnMut(f64) -> bool>>, cancelled: Rc<Cell<bool>>) {
    if cancelled.get() {
        return;
    }
    request_animation_frame(move || {
        if cancelled.get() {
            return;
        }
        let now = js_sys::Date::now();
        let should_continue = (on_frame.borrow_mut())(now);
        if should_continue && !cancelled.get() {
            schedule_frame(on_frame, cancelled);
        }
    });
}
