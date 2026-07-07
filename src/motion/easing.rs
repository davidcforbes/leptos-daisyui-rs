//! Cubic-Bezier easing evaluation, ported from `d2d-ui::motion::easing`.
//!
//! [`ui_tokens::motion::Easing`] only stores the four Bezier control points.
//! This module turns those points into an actual timing function: given a
//! normalized progress `t` (elapsed / duration), [`ease`] returns the eased
//! output a [`super::Transition`] should interpolate by.
//!
//! The curve is the standard CSS `cubic-bezier()` definition — a parametric
//! Bezier through `P0 = (0,0)`, `(x1,y1)`, `(x2,y2)`, `P3 = (1,1)`. Because
//! the parameter `u` is not the same as the x-axis input `t`, we solve
//! `x(u) = t` for `u` (Newton-Raphson, bisection fallback) and then read off
//! `y(u)`. This is the same approach browsers use for CSS animations.

use ui_tokens::motion::Easing;

/// Evaluate easing curve `easing` at normalized progress `t`.
///
/// `t` is clamped to `[0, 1]`; the result is the eased fraction, also in
/// `[0, 1]` for the Fluent 2 ramp (whose curves stay inside the unit square).
/// `t = 0` always returns `0.0` and `t = 1` always returns `1.0`.
pub fn ease(easing: Easing, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let (x1, y1, x2, y2) = easing.bezier();
    // Linear is its own timing function — skip the solve.
    if x1 == 0.0 && y1 == 0.0 && x2 == 1.0 && y2 == 1.0 {
        return t;
    }
    let bez = UnitBezier::new(x1, y1, x2, y2);
    bez.solve(t)
}

/// A cubic Bezier confined to the unit square, in polynomial form.
///
/// For each axis the curve is `c*u + b*u^2 + a*u^3`, derived from the control
/// points (`P0`/`P3` are fixed at the unit-square corners).
struct UnitBezier {
    ax: f32,
    bx: f32,
    cx: f32,
    ay: f32,
    by: f32,
    cy: f32,
}

impl UnitBezier {
    fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let cx = 3.0 * x1;
        let bx = 3.0 * (x2 - x1) - cx;
        let ax = 1.0 - cx - bx;
        let cy = 3.0 * y1;
        let by = 3.0 * (y2 - y1) - cy;
        let ay = 1.0 - cy - by;
        Self {
            ax,
            bx,
            cx,
            ay,
            by,
            cy,
        }
    }

    fn sample_x(&self, u: f32) -> f32 {
        ((self.ax * u + self.bx) * u + self.cx) * u
    }

    fn sample_y(&self, u: f32) -> f32 {
        ((self.ay * u + self.by) * u + self.cy) * u
    }

    fn sample_dx(&self, u: f32) -> f32 {
        (3.0 * self.ax * u + 2.0 * self.bx) * u + self.cx
    }

    /// Solve `x(u) = x` for `u`, then return `y(u)`.
    fn solve(&self, x: f32) -> f32 {
        self.sample_y(self.solve_for_u(x))
    }

    fn solve_for_u(&self, x: f32) -> f32 {
        // Newton-Raphson — converges fast for the gentle Fluent curves.
        let mut u = x;
        for _ in 0..8 {
            let err = self.sample_x(u) - x;
            if err.abs() < 1e-6 {
                return u;
            }
            let dx = self.sample_dx(u);
            if dx.abs() < 1e-6 {
                break;
            }
            u -= err / dx;
        }
        // Bisection fallback if Newton stalled (near-flat derivative).
        let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
        let mut u = x;
        if u < lo {
            return lo;
        }
        if u > hi {
            return hi;
        }
        for _ in 0..24 {
            let xv = self.sample_x(u);
            if (xv - x).abs() < 1e-6 {
                return u;
            }
            if xv < x {
                lo = u;
            } else {
                hi = u;
            }
            u = (lo + hi) * 0.5;
        }
        u
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_are_exact() {
        for e in [
            Easing::Linear,
            Easing::Standard,
            Easing::Decelerate,
            Easing::Accelerate,
        ] {
            assert_eq!(ease(e, 0.0), 0.0, "{e:?} at 0");
            assert_eq!(ease(e, 1.0), 1.0, "{e:?} at 1");
        }
    }

    #[test]
    fn input_is_clamped() {
        assert_eq!(ease(Easing::Standard, -0.5), 0.0);
        assert_eq!(ease(Easing::Standard, 1.5), 1.0);
    }

    #[test]
    fn linear_is_identity() {
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!((ease(Easing::Linear, t) - t).abs() < 1e-5);
        }
    }

    #[test]
    fn output_stays_in_unit_square() {
        for e in [Easing::Standard, Easing::Decelerate, Easing::Accelerate] {
            for i in 0..=20 {
                let v = ease(e, i as f32 / 20.0);
                assert!((-0.001..=1.001).contains(&v), "{e:?} produced {v}");
            }
        }
    }

    #[test]
    fn curves_are_monotonic_nondecreasing() {
        for e in [
            Easing::Linear,
            Easing::Standard,
            Easing::Decelerate,
            Easing::Accelerate,
        ] {
            let mut prev = 0.0;
            for i in 0..=50 {
                let v = ease(e, i as f32 / 50.0);
                assert!(v >= prev - 1e-4, "{e:?} dipped at {i}: {prev} -> {v}");
                prev = v;
            }
        }
    }

    #[test]
    fn standard_curve_solves_x_back_to_t() {
        // For any t, x(solve_for_u(t)) must equal t — the inverse is correct.
        let (x1, y1, x2, y2) = Easing::Standard.bezier();
        let bez = UnitBezier::new(x1, y1, x2, y2);
        for i in 1..20 {
            let t = i as f32 / 20.0;
            let u = bez.solve_for_u(t);
            assert!((bez.sample_x(u) - t).abs() < 1e-4, "t={t}");
        }
    }

    #[test]
    fn decelerate_is_front_loaded() {
        // A decelerating curve covers more than half its distance by the
        // time the input reaches its midpoint.
        assert!(ease(Easing::Decelerate, 0.5) > 0.5);
    }

    #[test]
    fn accelerate_is_back_loaded() {
        assert!(ease(Easing::Accelerate, 0.5) < 0.5);
    }
}
