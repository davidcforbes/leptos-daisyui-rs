//! Imperative motion primitives that CSS cannot (yet) express.
//!
//! [`crate::tokens::UiAnimationsPreamble`]'s `.ld-eased`/keyframe world is
//! fixed-duration and declarative — great for hover/press/entrance states,
//! but a CSS transition has no velocity, so retargeting it mid-flight
//! restarts the ramp from scratch (a visible "snap"). A damped-harmonic
//! [`Spring`] carries velocity instead: retargeting it produces a
//! continuous, physically-plausible overshoot-and-settle. Ported
//! near-verbatim from `d2d-ui::motion::spring` (Rust-DeskApp's native
//! Direct2D UI toolkit), which uses the identical numeric core for its own
//! press/settle motion.
//!
//! ## Module placement
//! This lives at the crate top level — a peer of [`crate::components`] and
//! [`crate::tokens`] — rather than under [`crate::utils`]. `utils` holds
//! small, single-purpose helpers (`ClassAttributes`, ripple math); motion is
//! a small but self-contained *system* (pure numeric core + a reactive rAF
//! driver +, eventually, more than one primitive sharing that driver), and
//! `d2d-ui` itself groups the equivalent code the same way
//! (`motion::{spring, transition, clock, easing}`). A future duration-based
//! `Transition<T>`/`Lerp` primitive is expected to land alongside [`Spring`]
//! in this module, reusing [`start_raf_loop`].
//!
//! ## What's here
//! - [`Spring`] — the pure numeric core (no `leptos`/`web-sys` dependency).
//!   Semi-implicit Euler, sub-stepped in `<=8ms` slices, velocity-preserving
//!   `set_target`, `is_settled`. Fully unit-tested; drive it yourself for a
//!   custom (non-Leptos) integration.
//! - [`use_spring`] / [`use_spring_with_params`] — the Leptos CSR hook: a
//!   `requestAnimationFrame` loop drives a `ReadSignal<f32>`, running only
//!   while the spring is un-settled, and is torn down on scope disposal.
//! - [`start_raf_loop`] — the shared rAF driver behind `use_spring`, reused
//!   by [`use_animated`] below rather than a second trampoline.
//! - [`Lerp`]/[`Transition`]/[`Keyframe`]/[`Track`] — the fixed-duration
//!   counterpart to [`Spring`]: pure, `leptos`-free interpolation types
//!   ported from `d2d-ui::motion::transition`, evaluated purely from a
//!   timestamp (`Transition::value(now_ms)`), with mid-flight retarget and
//!   multi-stage keyframe choreography.
//! - [`ease`] — the cubic-Bezier solver (ported from `d2d-ui::motion::easing`)
//!   that turns a [`ui_tokens::motion::Easing`]'s control points into an
//!   actual timing function; used internally by [`Transition`]/[`Track`].
//! - [`use_animated`] — the Leptos CSR hook wrapping [`Transition`] in a
//!   `requestAnimationFrame`-driven signal, reusing [`start_raf_loop`] the
//!   same way [`use_spring`] does.

mod easing;
mod hook;
mod raf_loop;
mod spring;
mod transition;

pub use easing::*;
pub use hook::*;
pub use raf_loop::*;
pub use spring::*;
pub use transition::*;

/// Re-exported so callers can build [`Transition`]/[`Keyframe`]/[`Track`]/
/// [`use_animated`] values without taking a direct dependency on `ui-tokens`
/// themselves.
pub use ui_tokens::motion::Easing;
