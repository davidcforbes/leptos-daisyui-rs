//! Damped-harmonic spring — ported near-verbatim from
//! `d2d-ui::motion::spring` (Rust-DeskApp's native Direct2D UI toolkit),
//! which shares this exact numeric core for its own press/settle motion.
//!
//! Unlike a fixed-duration CSS transition or a duration-based `Transition<T>`
//! primitive, a spring carries velocity: retargeting it mid-flight produces
//! a physically continuous overshoot-and-settle rather than restarting a
//! ramp from wherever it was interrupted.
//!
//! The integrator is semi-implicit Euler, sub-stepped in `<=8ms` slices so a
//! long frame (a GC pause, a backgrounded tab regaining focus) cannot make
//! it explode. This module is pure Rust with no `leptos`/`web-sys`
//! dependency — see [`crate::motion::use_spring`] for the reactive,
//! `requestAnimationFrame`-driven wrapper.

/// A 1-D spring oscillator pulling `value` toward `target`.
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    /// Current position.
    pub value: f32,
    /// Current velocity (units per second).
    pub velocity: f32,
    /// Position the spring is pulled toward.
    pub target: f32,
    stiffness: f32,
    damping: f32,
}

/// Settle thresholds — below both, the spring is treated as at rest.
const REST_VALUE_EPS: f32 = 0.0005;
const REST_VELOCITY_EPS: f32 = 0.0005;

impl Spring {
    /// A spring at rest at `initial`, with the default snappy tuning.
    pub fn new(initial: f32) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target: initial,
            stiffness: 220.0,
            damping: 26.0,
        }
    }

    /// A spring with explicit `stiffness` (pull strength) and `damping`
    /// (velocity bleed). Higher stiffness = faster; higher damping = less
    /// overshoot.
    pub fn with_params(initial: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target: initial,
            stiffness: stiffness.max(0.0),
            damping: damping.max(0.0),
        }
    }

    /// Aim the spring at a new target. Velocity is preserved, so a change
    /// mid-flight stays continuous.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Advance the simulation by `dt_ms` milliseconds.
    ///
    /// Long frames are sub-stepped so a hitch cannot make the integrator
    /// explode; once settled the spring snaps exactly onto the target.
    pub fn step(&mut self, dt_ms: f32) {
        if dt_ms <= 0.0 {
            return;
        }
        // Sub-step in <=8ms slices for integrator stability across hitches.
        let mut remaining = (dt_ms / 1000.0).min(0.25);
        let max_slice = 0.008;
        while remaining > 0.0 {
            let dt = remaining.min(max_slice);
            let force = -self.stiffness * (self.value - self.target) - self.damping * self.velocity;
            self.velocity += force * dt;
            self.value += self.velocity * dt;
            remaining -= dt;
        }
        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
        }
    }

    /// Whether the spring has effectively reached its target and stopped.
    pub fn is_settled(&self) -> bool {
        (self.value - self.target).abs() < REST_VALUE_EPS && self.velocity.abs() < REST_VELOCITY_EPS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_spring_is_at_rest() {
        let s = Spring::new(1.0);
        assert_eq!(s.value, 1.0);
        assert_eq!(s.velocity, 0.0);
        assert!(s.is_settled());
    }

    #[test]
    fn zero_dt_step_is_a_noop() {
        let mut s = Spring::new(0.0);
        s.set_target(1.0);
        s.step(0.0);
        assert_eq!(s.value, 0.0);
    }

    #[test]
    fn spring_converges_to_target() {
        let mut s = Spring::new(0.0);
        s.set_target(1.0);
        // Two seconds of 16ms frames is far longer than the settle time.
        for _ in 0..125 {
            s.step(16.0);
        }
        assert!(s.is_settled(), "value={} vel={}", s.value, s.velocity);
        assert!((s.value - 1.0).abs() < 1e-3);
    }

    #[test]
    fn settled_spring_snaps_exactly_onto_target() {
        let mut s = Spring::new(0.0);
        s.set_target(0.98);
        for _ in 0..200 {
            s.step(16.0);
        }
        assert_eq!(s.value, 0.98);
        assert_eq!(s.velocity, 0.0);
    }

    #[test]
    fn spring_moves_toward_target_after_one_step() {
        let mut s = Spring::new(0.0);
        s.set_target(1.0);
        s.step(16.0);
        assert!(s.value > 0.0 && s.value < 1.0);
        assert!(s.velocity > 0.0);
    }

    #[test]
    fn long_frame_is_substepped_without_exploding() {
        let mut s = Spring::new(0.0);
        s.set_target(1.0);
        // A pathological 800ms hitch must not blow the integrator up.
        s.step(800.0);
        assert!(s.value.is_finite());
        assert!(s.value.abs() < 5.0);
    }

    #[test]
    fn retargeting_preserves_velocity_continuity() {
        let mut s = Spring::new(0.0);
        s.set_target(1.0);
        for _ in 0..3 {
            s.step(16.0);
        }
        let v_before = s.velocity;
        s.set_target(0.5);
        // set_target alone must not zero the velocity.
        assert_eq!(s.velocity, v_before);
    }

    #[test]
    fn low_damping_overshoots_before_settling() {
        // A loose, lightly-damped spring should fly past its target before
        // settling back onto it — the visible "overshoot" the demo page
        // relies on to show off the difference from a duration-based ease.
        let mut s = Spring::with_params(0.0, 90.0, 6.0);
        s.set_target(1.0);
        let mut max_value = f32::MIN;
        for _ in 0..500 {
            s.step(8.0);
            max_value = max_value.max(s.value);
        }
        assert!(
            max_value > 1.05,
            "expected an overshoot past the target, max_value={max_value}"
        );
        assert!(s.is_settled(), "spring should settle by 4s: value={}", s.value);
        assert_eq!(s.value, 1.0);
    }
}
