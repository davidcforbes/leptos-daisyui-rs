//! Timed value transitions and keyframe tracks, ported from
//! `d2d-ui::motion::transition`.
//!
//! A [`Transition<T>`] eases one value from `from` to `to` over a fixed
//! duration; [`Track<T>`] chains several keyframes for multi-stage motion.
//! Both evaluate purely from a timestamp, so a caller stores the transition
//! and reads `value(now_ms)` — no per-frame mutation needed to *read* it
//! (the reactive wrapper, [`super::use_animated`], still needs an
//! `requestAnimationFrame` loop to re-render on each frame).

use super::easing::ease;
use ui_tokens::motion::Easing;

/// Linear interpolation between two values of the same type.
///
/// `t` is the eased fraction in `[0, 1]`: `t = 0` yields `self`, `t = 1`
/// yields `to`. Implemented for `f32`/`f64` and `[f32; 4]` (an RGBA color —
/// this crate has no dedicated color struct, so a plain `[r, g, b, a]` array
/// in `[0.0, 1.0]` per channel is the color representation `Transition`/
/// `Track` animate; see `demo/src/demos/motion.rs`'s color swatch for a
/// worked example converting to/from a CSS `rgba()` string).
pub trait Lerp: Copy {
    /// Interpolate from `self` toward `to` by fraction `t` (clamped by
    /// callers before this is invoked; implementations need not re-clamp).
    fn lerp(self, to: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(self, to: Self, t: f32) -> Self {
        self + (to - self) * t
    }
}

impl Lerp for f64 {
    fn lerp(self, to: Self, t: f32) -> Self {
        self + (to - self) * t as f64
    }
}

impl Lerp for [f32; 4] {
    fn lerp(self, to: Self, t: f32) -> Self {
        [
            self[0].lerp(to[0], t),
            self[1].lerp(to[1], t),
            self[2].lerp(to[2], t),
            self[3].lerp(to[3], t),
        ]
    }
}

/// A value easing from `from` to `to` over `duration_ms`, starting at
/// `start_ms`. Cheap to copy and store.
#[derive(Clone, Copy, Debug)]
pub struct Transition<T> {
    from: T,
    to: T,
    start_ms: f64,
    duration_ms: f32,
    easing: Easing,
}

impl<T: Lerp> Transition<T> {
    /// A transition in flight from `from` to `to`.
    pub fn new(from: T, to: T, start_ms: f64, duration_ms: f32, easing: Easing) -> Self {
        Self {
            from,
            to,
            start_ms,
            duration_ms: duration_ms.max(0.0),
            easing,
        }
    }

    /// A transition already resting at `value` — no motion.
    pub fn settled(value: T) -> Self {
        Self {
            from: value,
            to: value,
            start_ms: 0.0,
            duration_ms: 0.0,
            easing: Easing::Linear,
        }
    }

    /// Raw progress `[0, 1]` — elapsed fraction of the duration.
    pub fn progress(&self, now_ms: f64) -> f32 {
        if self.duration_ms <= 0.0 {
            return 1.0;
        }
        (((now_ms - self.start_ms) as f32) / self.duration_ms).clamp(0.0, 1.0)
    }

    /// The eased, interpolated value at `now_ms`.
    pub fn value(&self, now_ms: f64) -> T {
        self.from
            .lerp(self.to, ease(self.easing, self.progress(now_ms)))
    }

    /// The value this transition is heading toward.
    pub fn target(&self) -> T {
        self.to
    }

    /// Whether the transition is moving at `now_ms` — within its time
    /// window: at or after the start, before the end.
    pub fn is_active(&self, now_ms: f64) -> bool {
        self.duration_ms > 0.0
            && now_ms >= self.start_ms
            && now_ms < self.start_ms + self.duration_ms as f64
    }

    /// Redirect toward a new target, starting from wherever the value is
    /// *now* — so interrupting a transition mid-flight stays smooth.
    pub fn retarget(&mut self, new_to: T, now_ms: f64, duration_ms: f32, easing: Easing) {
        let current = self.value(now_ms);
        self.from = current;
        self.to = new_to;
        self.start_ms = now_ms;
        self.duration_ms = duration_ms.max(0.0);
        self.easing = easing;
    }
}

/// A single stop in a [`Track`]: reach `value` at `time_ms` (relative to the
/// track start), easing in from the previous keyframe with `easing`.
#[derive(Clone, Copy, Debug)]
pub struct Keyframe<T> {
    /// When this keyframe is reached, in milliseconds relative to the
    /// owning [`Track`]'s start.
    pub time_ms: f32,
    /// The value to hold at `time_ms`.
    pub value: T,
    /// The easing curve used for the segment ending at this keyframe (i.e.
    /// how the value ramps in from the previous keyframe).
    pub easing: Easing,
}

/// A multi-stage animation: an ordered list of keyframes evaluated against a
/// track-relative timestamp. Useful for entrance choreography that a single
/// [`Transition`] cannot express (e.g. overshoot-then-settle).
#[derive(Clone, Debug)]
pub struct Track<T> {
    keyframes: Vec<Keyframe<T>>,
    start_ms: f64,
}

impl<T: Lerp> Track<T> {
    /// Build a track from keyframes (sorted by time on construction) that
    /// begins playing at `start_ms`.
    pub fn new(mut keyframes: Vec<Keyframe<T>>, start_ms: f64) -> Self {
        keyframes.sort_by(|a, b| a.time_ms.total_cmp(&b.time_ms));
        Self {
            keyframes,
            start_ms,
        }
    }

    /// The interpolated value at `now_ms`. Before the first keyframe holds
    /// the first value; after the last holds the last value.
    pub fn value(&self, now_ms: f64) -> Option<T> {
        let first = self.keyframes.first()?;
        let last = self.keyframes.last()?;
        let rel = (now_ms - self.start_ms) as f32;
        if rel <= first.time_ms {
            return Some(first.value);
        }
        if rel >= last.time_ms {
            return Some(last.value);
        }
        // Find the segment [a, b] containing `rel`.
        let seg = self.keyframes.windows(2).find(|w| rel < w[1].time_ms)?;
        let (a, b) = (&seg[0], &seg[1]);
        let span = (b.time_ms - a.time_ms).max(1e-3);
        let local = ((rel - a.time_ms) / span).clamp(0.0, 1.0);
        Some(a.value.lerp(b.value, ease(b.easing, local)))
    }

    /// Whether the track is still playing at `now_ms`.
    pub fn is_active(&self, now_ms: f64) -> bool {
        match self.keyframes.last() {
            Some(last) => ((now_ms - self.start_ms) as f32) < last.time_ms,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_lerp_endpoints_and_midpoint() {
        assert_eq!(2.0_f32.lerp(10.0, 0.0), 2.0);
        assert_eq!(2.0_f32.lerp(10.0, 1.0), 10.0);
        assert_eq!(2.0_f32.lerp(10.0, 0.5), 6.0);
    }

    #[test]
    fn f64_lerp_endpoints_and_midpoint() {
        assert_eq!(2.0_f64.lerp(10.0, 0.0), 2.0);
        assert_eq!(2.0_f64.lerp(10.0, 1.0), 10.0);
        assert_eq!(2.0_f64.lerp(10.0, 0.5), 6.0);
    }

    #[test]
    fn color_lerp_blends_each_channel() {
        let a = [0.0, 0.0, 0.0, 0.0];
        let b = [1.0, 1.0, 1.0, 1.0];
        let mid = a.lerp(b, 0.5);
        assert_eq!(mid, [0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn rect_lerp_blends_each_edge() {
        let a = [0.0, 0.0, 0.0, 0.0];
        let b = [10.0, 20.0, 30.0, 40.0];
        assert_eq!(a.lerp(b, 0.5), [5.0, 10.0, 15.0, 20.0]);
    }

    #[test]
    fn settled_transition_never_moves() {
        let t = Transition::settled(7.0_f32);
        assert_eq!(t.value(0.0), 7.0);
        assert_eq!(t.value(100_000.0), 7.0);
        assert!(!t.is_active(0.0));
    }

    #[test]
    fn transition_eases_from_start_to_target() {
        let t = Transition::new(0.0_f32, 100.0, 1000.0, 200.0, Easing::Linear);
        assert_eq!(t.value(1000.0), 0.0);
        assert_eq!(t.value(1200.0), 100.0);
        assert!((t.value(1100.0) - 50.0).abs() < 1e-3);
        assert_eq!(t.target(), 100.0);
    }

    #[test]
    fn transition_clamps_outside_its_window() {
        let t = Transition::new(0.0_f32, 100.0, 1000.0, 200.0, Easing::Linear);
        assert_eq!(t.value(0.0), 0.0); // before start
        assert_eq!(t.value(99_999.0), 100.0); // long after end
    }

    #[test]
    fn is_active_only_within_window() {
        let t = Transition::new(0.0_f32, 1.0, 100.0, 50.0, Easing::Standard);
        assert!(!t.is_active(99.0));
        assert!(t.is_active(120.0));
        assert!(!t.is_active(150.0));
    }

    #[test]
    fn retarget_starts_from_current_value() {
        let mut t = Transition::new(0.0_f32, 100.0, 0.0, 100.0, Easing::Linear);
        // Halfway: value is 50. Retarget back toward 0.
        t.retarget(0.0, 50.0, 100.0, Easing::Linear);
        assert!((t.value(50.0) - 50.0).abs() < 1e-3);
        assert!((t.value(100.0) - 25.0).abs() < 1e-3);
        assert_eq!(t.value(150.0), 0.0);
    }

    #[test]
    fn track_holds_before_and_after_its_keyframes() {
        let track = Track::new(
            vec![
                Keyframe {
                    time_ms: 0.0,
                    value: 0.0_f32,
                    easing: Easing::Linear,
                },
                Keyframe {
                    time_ms: 100.0,
                    value: 1.0,
                    easing: Easing::Linear,
                },
            ],
            0.0,
        );
        assert_eq!(track.value(-50.0), Some(0.0));
        assert!((track.value(50.0).unwrap() - 0.5).abs() < 1e-3);
        assert_eq!(track.value(999.0), Some(1.0));
        assert!(track.is_active(50.0));
        assert!(!track.is_active(200.0));
    }

    #[test]
    fn track_sorts_unordered_keyframes() {
        let track = Track::new(
            vec![
                Keyframe {
                    time_ms: 100.0,
                    value: 1.0_f32,
                    easing: Easing::Linear,
                },
                Keyframe {
                    time_ms: 0.0,
                    value: 0.0,
                    easing: Easing::Linear,
                },
            ],
            0.0,
        );
        assert_eq!(track.value(0.0), Some(0.0));
        assert_eq!(track.value(100.0), Some(1.0));
    }

    #[test]
    fn track_with_no_keyframes_is_never_active_and_has_no_value() {
        let track: Track<f32> = Track::new(vec![], 0.0);
        assert_eq!(track.value(0.0), None);
        assert!(!track.is_active(0.0));
    }
}
