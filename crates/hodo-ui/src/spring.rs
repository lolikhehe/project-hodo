//! Analytic spring solver (§K.4). No per-frame integration error, no allocation.
//!
//! m = 1, ω₀ = √k, σ = ζω₀, ω_d = ω₀√(1−ζ²).
//! Underdamped: x(t) = x₁ + e^(−σt)(A·cos(ω_d t) + B·sin(ω_d t)),
//!             A = x₀−x₁, B = (v₀+σA)/ω_d.
//! Critical ζ=1: x(t) = x₁ + (A + C·t)e^(−ω₀t), C = v₀+ω₀A.

/// Frozen spring specification (stiffness k, damping ratio ζ).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringSpec {
    pub k: f32,
    pub zeta: f32,
}

impl SpringSpec {
    pub const fn new(k: f32, zeta: f32) -> Self {
        Self { k, zeta }
    }
}

/// Values verified against the M3E reference ports; default scheme = expressive.
pub const EXPRESSIVE_SPATIAL: [SpringSpec; 3] = [
    SpringSpec::new(800.0, 0.6), // fast
    SpringSpec::new(380.0, 0.8), // default
    SpringSpec::new(200.0, 0.8), // slow
];
pub const STANDARD_SPATIAL: [SpringSpec; 3] = [
    SpringSpec::new(1400.0, 0.9),
    SpringSpec::new(700.0, 0.9),
    SpringSpec::new(300.0, 0.9),
];
pub const EFFECTS: [SpringSpec; 3] = [
    SpringSpec::new(3800.0, 1.0),
    SpringSpec::new(1600.0, 1.0),
    SpringSpec::new(800.0, 1.0),
];

pub const DEFAULT_SPATIAL: SpringSpec = EXPRESSIVE_SPATIAL[1];
pub const DEFAULT_EFFECT: SpringSpec = EFFECTS[1];

/// Settle thresholds. Spatial: within 0.5 dp of target and 1 dp/s. Effects:
/// alpha within a single 8-bit level.
pub const SPATIAL_POSITION_EPS: f32 = 0.5;
pub const SPATIAL_VELOCITY_EPS: f32 = 1.0;
pub const EFFECT_ALPHA_EPS: f32 = 1.0 / 255.0;

/// A running analytic spring between `x0` and `target` with initial velocity `v0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub spec: SpringSpec,
    x0: f32,
    v0: f32,
    target: f32,
    t: f32,
}

impl Spring {
    pub fn new(spec: SpringSpec, x0: f32, target: f32) -> Self {
        Self::with_velocity(spec, x0, 0.0, target)
    }

    pub fn with_velocity(spec: SpringSpec, x0: f32, v0: f32, target: f32) -> Self {
        Self {
            spec,
            x0,
            v0,
            target,
            t: 0.0,
        }
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn elapsed(&self) -> f32 {
        self.t
    }

    pub fn advance(&mut self, dt: f32) {
        self.t += dt;
    }

    pub fn position(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return self.x0;
        }
        // k <= 0 has no restoring force (and sqrt(k) is NaN for k < 0): the
        // spring simply sits at its target with zero velocity.
        let spec = self.spec;
        if spec.k <= 0.0 {
            return self.target;
        }
        let w0 = spec.k.sqrt();
        let sigma = spec.zeta * w0;
        let a = self.x0 - self.target;
        if spec.zeta < 1.0 {
            let wd = w0 * (1.0 - spec.zeta * spec.zeta).sqrt();
            let b = (self.v0 + sigma * a) / wd;
            self.target + (-sigma * t).exp() * (a * (wd * t).cos() + b * (wd * t).sin())
        } else if spec.zeta == 1.0 {
            let c = self.v0 + w0 * a;
            self.target + (a + c * t) * (-w0 * t).exp()
        } else {
            // Overdamped: roots -sigma +/- alpha, alpha = w0*sqrt(zeta^2-1).
            // x = target + e^(-sigma t)(a cosh(alpha t) + b sinh(alpha t)).
            let alpha = w0 * (spec.zeta * spec.zeta - 1.0).sqrt();
            let b = (self.v0 + sigma * a) / alpha;
            let (cosh, sinh) = ((alpha * t).cosh(), (alpha * t).sinh());
            self.target + (-sigma * t).exp() * (a * cosh + b * sinh)
        }
    }

    pub fn velocity(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return self.v0;
        }
        let spec = self.spec;
        if spec.k <= 0.0 {
            return 0.0;
        }
        let w0 = spec.k.sqrt();
        let sigma = spec.zeta * w0;
        let a = self.x0 - self.target;
        if spec.zeta < 1.0 {
            let wd = w0 * (1.0 - spec.zeta * spec.zeta).sqrt();
            let b = (self.v0 + sigma * a) / wd;
            let e = (-sigma * t).exp();
            let (sin, cos) = (wd * t).sin_cos();
            e * ((b * wd - sigma * a) * cos + (-a * wd - sigma * b) * sin)
        } else if spec.zeta == 1.0 {
            let c = self.v0 + w0 * a;
            (c - w0 * (a + c * t)) * (-w0 * t).exp()
        } else {
            let alpha = w0 * (spec.zeta * spec.zeta - 1.0).sqrt();
            let b = (self.v0 + sigma * a) / alpha;
            let e = (-sigma * t).exp();
            let (cosh, sinh) = ((alpha * t).cosh(), (alpha * t).sinh());
            // b*alpha - sigma*a == v0 by construction of b.
            e * (self.v0 * cosh + (a * alpha - sigma * b) * sinh)
        }
    }

    pub fn sample(&self) -> (f32, f32) {
        (self.position(self.t), self.velocity(self.t))
    }

    /// Interrupt the spring, keeping its current position and velocity as the
    /// new initial conditions. Velocity is exactly continuous at the seam.
    pub fn retarget(&mut self, new_target: f32) {
        let (x, v) = self.sample();
        self.x0 = x;
        self.v0 = v;
        self.target = new_target;
        self.t = 0.0;
    }

    /// Fraction of the step overshoot for an underdamped step from rest; zero
    /// for critically damped (effects must never overshoot).
    pub fn overshoot_ratio(&self) -> f32 {
        let z = self.spec.zeta;
        if z >= 1.0 {
            0.0
        } else {
            (-std::f32::consts::PI * z / (1.0 - z * z).sqrt()).exp()
        }
    }

    pub fn is_settled(&self, position_eps: f32, velocity_eps: f32) -> bool {
        let (x, v) = self.sample();
        (x - self.target).abs() <= position_eps && v.abs() <= velocity_eps
    }

    /// First time (seconds) at which both settle thresholds hold. Used by the
    /// numeric budget tests; scans at 0.1 ms resolution.
    pub fn settle_time(&self, position_eps: f32, velocity_eps: f32, max_t: f32) -> Option<f32> {
        const DT: f64 = 1e-4;
        let mut t = 0.0_f64;
        while t <= max_t as f64 {
            let tf = t as f32;
            let x = self.position(tf);
            let v = self.velocity(tf);
            if (x - self.target).abs() <= position_eps && v.abs() <= velocity_eps {
                return Some(tf);
            }
            t += DT;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step_spring(spec: SpringSpec) -> Spring {
        Spring::new(spec, 0.0, 1.0)
    }

    #[test]
    fn effects_never_overshoot() {
        for spec in EFFECTS {
            let s = step_spring(spec);
            assert_eq!(s.overshoot_ratio(), 0.0);
            let mut peak = 0.0_f32;
            let mut t = 0.0;
            while t < 0.6 {
                peak = peak.max(s.position(t));
                t += 0.001;
            }
            assert!(peak <= 1.0 + 1e-4, "effects overshot: {peak}");
        }
    }

    #[test]
    fn expressive_spatial_overshoot_matches_formula() {
        let (fast, default, slow) = (
            EXPRESSIVE_SPATIAL[0],
            EXPRESSIVE_SPATIAL[1],
            EXPRESSIVE_SPATIAL[2],
        );
        let ov = |s: SpringSpec| step_spring(s).overshoot_ratio() * 100.0;
        assert!((ov(fast) - 9.5).abs() < 0.2, "fast {}", ov(fast));
        assert!((ov(default) - 1.5).abs() < 0.2, "default {}", ov(default));
        assert!((ov(slow) - 1.5).abs() < 0.2, "slow {}", ov(slow));
        assert!(ov(fast) < 10.0);
        assert!(ov(default) < 10.0);
        assert!(ov(slow) < 10.0);
    }

    #[test]
    fn sampled_peak_matches_analytic_overshoot() {
        for spec in EXPRESSIVE_SPATIAL {
            let s = step_spring(spec);
            let mut peak = 0.0_f32;
            let mut t = 0.0;
            while t < 1.0 {
                peak = peak.max(s.position(t));
                t += 0.0002;
            }
            let expected = 1.0 + s.overshoot_ratio();
            assert!((peak - expected).abs() < 2e-3, "peak {peak} vs {expected}");
        }
    }

    #[test]
    fn spatial_settle_times_within_budget() {
        let budgets_ms = [260.0, 300.0, 400.0];
        for (spec, budget) in EXPRESSIVE_SPATIAL.iter().zip(budgets_ms) {
            let t = step_spring(*spec)
                .settle_time(SPATIAL_POSITION_EPS, SPATIAL_VELOCITY_EPS, 1.0)
                .expect("spatial spring settles");
            assert!(
                t * 1000.0 <= budget,
                "settle {}ms > {}ms",
                t * 1000.0,
                budget
            );
        }
    }

    #[test]
    fn effects_settle_times_within_budget() {
        // Effects animate state-layer alpha (a representative 0 -> 0.08 step).
        let budgets_ms = [100.0, 160.0, 240.0];
        for (spec, budget) in EFFECTS.iter().zip(budgets_ms) {
            let s = Spring::new(*spec, 0.0, 0.08);
            let t = s
                .settle_time(EFFECT_ALPHA_EPS, SPATIAL_VELOCITY_EPS, 1.0)
                .expect("effect settles");
            assert!(
                t * 1000.0 <= budget,
                "settle {}ms > {}ms",
                t * 1000.0,
                budget
            );
        }
    }

    #[test]
    fn retarget_preserves_velocity() {
        for spec in [EXPRESSIVE_SPATIAL[0], EXPRESSIVE_SPATIAL[1], EFFECTS[1]] {
            let mut s = Spring::new(spec, 0.0, 1.0);
            s.advance(0.05);
            let (_, v_before) = s.sample();
            s.retarget(-1.0);
            let (x_after, v_after) = s.sample();
            assert!(
                (v_after - v_before).abs() < 1e-6,
                "Δv = {}",
                v_after - v_before
            );
            assert!((x_after - s.position(0.0)).abs() < f32::EPSILON);
            assert!((s.target() + 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn spring_reaches_target() {
        for spec in EXPRESSIVE_SPATIAL {
            let s = step_spring(spec);
            assert!((s.position(2.0) - 1.0).abs() < 1e-3);
            assert!(s.velocity(2.0).abs() < 1e-2);
        }
    }

    #[test]
    fn nonpositive_stiffness_has_no_motion() {
        for k in [0.0, -1.0, -100.0] {
            let mut s = Spring::new(SpringSpec::new(k, 0.6), 0.0, 1.0);
            assert_eq!(s.position(1.0), 1.0, "k={k}");
            assert_eq!(s.velocity(1.0), 0.0, "k={k}");
            s.advance(1.0);
            assert_eq!(s.sample(), (1.0, 0.0), "k={k}");

            let with_v = Spring::with_velocity(SpringSpec::new(k, 0.8), 5.0, 3.0, 1.0);
            assert_eq!(with_v.position(0.5), 1.0, "k={k}");
            assert_eq!(with_v.velocity(0.5), 0.0, "k={k}");
        }
    }

    #[test]
    fn overdamped_spring_approaches_without_overshoot() {
        let s = Spring::new(SpringSpec::new(400.0, 2.0), 0.0, 1.0);
        let mut previous = 0.0;
        let mut t = 0.0;
        while t < 2.0 {
            let (x, v) = (s.position(t), s.velocity(t));
            assert!(x.is_finite() && v.is_finite(), "non-finite sample at t={t}");
            assert!(x >= previous - 1e-6, "overdamped step overshot at t={t}");
            previous = x;
            t += 0.005;
        }
        assert!((s.position(2.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn critical_spring_matches_analytic_solution() {
        let s = Spring::new(SpringSpec::new(400.0, 1.0), 0.0, 1.0);
        let t: f32 = 0.1;
        let expected = 1.0 - (1.0 + 20.0 * t) * (-20.0 * t).exp();
        assert!((s.position(t) - expected).abs() < 1e-5);
    }
}
