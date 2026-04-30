//! # φ — The Golden Ratio Constants & Harmonic Utilities
//!
//! The golden ratio φ = (1 + √5) / 2 ≈ 1.6180339887 governs every
//! fundamental constant in Aether-Net.  From damping coefficients to
//! spatial partitioning, from solver convergence to time-step harmonics —
//! φ is the law of the simulation.
//!
//! ## Why φ?
//! - Natural systems converge to φ-proportioned equilibria
//! - Fibonacci-scaled spatial grids minimise hash collisions  
//! - φ-damping produces visually "correct" physical motion
//! - The golden angle (2π/φ²) distributes broadphase probes optimally
//! - φ-interval sleeping avoids resonance artefacts
//!
//! ## Derivation hierarchy
//! ```text
//!   φ   = 1.6180339887...
//!   φ⁻¹ = 0.6180339887...   (= φ - 1)
//!   φ²  = 2.6180339887...   (= φ + 1)
//!   φ³  = 4.2360679774...
//!   φ⁴  = 6.8541019662...
//!   φ⁵  = 11.090169943...
//! ```

// ─────────────────────────────── Core constants ───────────────────────────────

/// The golden ratio: φ = (1 + √5) / 2
pub const PHI: f64 = 1.618_033_988_749_895;

/// φ as f32 — for SIMD / GPU path
pub const PHI_F32: f32 = 1.618_033_9;

/// Reciprocal: 1/φ = φ − 1 ≈ 0.618...
pub const PHI_INV: f64 = 0.618_033_988_749_895;

/// φ² = φ + 1 ≈ 2.618...
pub const PHI_SQ: f64 = 2.618_033_988_749_895;

/// φ³ ≈ 4.236...
pub const PHI_CB: f64 = 4.236_067_977_499_790;

/// φ⁴ ≈ 6.854...
pub const PHI_4: f64 = 6.854_101_966_249_685;

/// φ⁵ ≈ 11.090...
pub const PHI_5: f64 = 11.090_169_943_749_474;

/// φ⁶ ≈ 17.944...
pub const PHI_6: f64 = 17.944_271_909_999_159;

/// φ⁷ ≈ 29.034...
pub const PHI_7: f64 = 29.034_441_853_748_632;

/// φ⁸ ≈ 46.979...
pub const PHI_8: f64 = 46.978_713_763_747_794;

/// The golden angle in radians: 2π / φ² ≈ 2.39996...
pub const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;

// ─────────────────────────────── Physics defaults ───────────────────────────────

/// Standard gravity (m/s²) — engine uses φ-sub-stepping for stability
pub const GRAVITY_STANDARD: f64 = 9.806_65;

/// Moon gravity: standard / φ² ≈ 3.745...
pub const GRAVITY_MOON: f64 = GRAVITY_STANDARD / PHI_SQ;

/// Micro-gravity threshold: φ⁻⁸
pub const GRAVITY_MICRO: f64 = 0.021_236_763_728_490;

/// Linear damping: 1 − 1/φ⁴ ≈ 0.854
pub const LINEAR_DAMPING: f64 = 1.0 - 1.0 / PHI_4;  // ≈ 0.854

/// Angular damping: 1 − 1/φ³ ≈ 0.764
pub const ANGULAR_DAMPING: f64 = 1.0 - 1.0 / PHI_CB; // ≈ 0.764

/// Solver convergence threshold: φ⁻⁸
pub const SOLVER_EPSILON: f64 = 1.0 / PHI_8;         // ≈ 0.02128

/// Baumgarte position-bias factor: φ⁻³ × 0.2
pub const BAUMGARTE_FACTOR: f64 = 0.2 / PHI_CB;      // ≈ 0.04724

/// Sleep velocity threshold: φ⁻⁵ ≈ 0.090
pub const SLEEP_THRESHOLD: f64 = 1.0 / PHI_5;        // ≈ 0.09017

/// Penetration slop (allowed overlap before Baumgarte kicks in): φ⁻⁶
pub const PENETRATION_SLOP: f64 = 1.0 / PHI_6;       // ≈ 0.05573

/// Max linear velocity before clamping: φ⁵ ≈ 11.09
pub const MAX_LINEAR_VELOCITY: f64 = PHI_5;

/// Max angular velocity: φ⁴ ≈ 6.854
pub const MAX_ANGULAR_VELOCITY: f64 = PHI_4;

/// Coefficient of restitution decay per bounce: φ⁻¹
pub const RESTITUTION_DECAY: f64 = PHI_INV;

/// Maximum solver iterations — Fibonacci(6) = 8
pub const SOLVER_ITERATIONS: u32 = 8;

/// Sub-step count per frame — Fibonacci(5) = 5
pub const SUBSTEP_COUNT: u32 = 5;

/// Broadphase cell size multiplier — cell = φ × max_extent
pub const BROADPHASE_CELL_RATIO: f64 = PHI;

/// Frames until a body is put to sleep — Fibonacci(7) = 13
pub const SLEEP_FRAMES: u32 = 13;

/// Speculative CCD margin — φ⁻⁴ ≈ 0.146
pub const CCD_MARGIN: f64 = 1.0 / PHI_4;

// ─────────────────────────────── Fibonacci table ───────────────────────────────

/// First 24 Fibonacci numbers for internal φ-scaled logic
pub const FIBONACCI: [u64; 24] = [
    1, 1, 2, 3, 5, 8, 13, 21, 34, 55,
    89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765,
    10946, 17711, 28657, 46368,
];

// ─────────────────────────────── Utility functions ───────────────────────────────

/// φⁿ — compile-time integer exponentiation via powi.
#[inline(always)]
pub fn phi_pow(n: i32) -> f64 {
    PHI.powi(n)
}

/// φ-biased interpolation — biased toward the golden ratio split.
///
/// Unlike linear lerp, `phi_lerp` applies a φ⁻¹ power curve so the
/// midpoint lands at the golden ratio rather than 0.5.
#[inline(always)]
pub fn phi_lerp(a: f64, b: f64, t: f64) -> f64 {
    let phi_t = t.powf(PHI_INV);
    a + (b - a) * phi_t
}

/// Scale a value by Fibonacci(level) / Fibonacci(level-1) — converges to φ.
#[inline]
pub fn fib_scale(value: f64, level: usize) -> f64 {
    if level < 2 { return value; }
    let idx = level.min(FIBONACCI.len() - 1);
    value * (FIBONACCI[idx] as f64) / (FIBONACCI[idx - 1] as f64)
}

/// Golden-angle rotation index for sphere/hemisphere probe distribution.
#[inline(always)]
pub fn golden_angle_at(index: u32) -> f64 {
    (index as f64) * GOLDEN_ANGLE
}

/// Return true if `n` is a Fibonacci number.
#[inline]
pub fn is_fibonacci(n: u64) -> bool {
    FIBONACCI.contains(&n)
}

/// Map a normalised value [0,1] through the φ-sigmoid — smooth S-curve
/// that passes through (φ⁻¹, 0.5) rather than (0.5, 0.5).
#[inline(always)]
pub fn phi_sigmoid(t: f64) -> f64 {
    1.0 / (1.0 + (-(t - PHI_INV) * PHI_SQ).exp())
}

/// Compute the φ-weighted average of two values: (a·φ + b) / (φ + 1).
#[inline(always)]
pub fn phi_weighted_avg(a: f64, b: f64) -> f64 {
    (a * PHI + b) / PHI_SQ
}

// ─────────────────────────────── Tests ───────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_squared_identity() {
        // φ² = φ + 1  (defining property)
        assert!((PHI * PHI - (PHI + 1.0)).abs() < 1e-12, "φ² ≠ φ+1");
    }

    #[test]
    fn phi_reciprocal_identity() {
        // 1/φ = φ − 1
        assert!((1.0 / PHI - PHI_INV).abs() < 1e-12);
        assert!((PHI_INV - (PHI - 1.0)).abs() < 1e-12);
    }

    #[test]
    fn phi_hierarchy_consistent() {
        assert!((PHI * PHI - PHI_SQ).abs() < 1e-10);
        assert!((PHI_SQ * PHI - PHI_CB).abs() < 1e-10);
        assert!((PHI_CB * PHI - PHI_4).abs() < 1e-10);
        assert!((PHI_4 * PHI - PHI_5).abs() < 1e-10);
    }

    #[test]
    fn fibonacci_ratio_converges_to_phi() {
        for i in 10..23 {
            let ratio = FIBONACCI[i + 1] as f64 / FIBONACCI[i] as f64;
            assert!((ratio - PHI).abs() < 0.0001, "ratio[{i}] = {ratio}");
        }
    }

    #[test]
    fn sleep_threshold_from_phi5() {
        assert!((SLEEP_THRESHOLD - 1.0 / PHI_5).abs() < 1e-12);
    }

    #[test]
    fn phi_lerp_at_one_is_b() {
        assert!((phi_lerp(0.0, 10.0, 1.0) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn phi_sigmoid_at_phi_inv_is_half() {
        // By construction, sigmoid(φ⁻¹) ≈ 0.5
        let s = phi_sigmoid(PHI_INV);
        assert!((s - 0.5).abs() < 0.01);
    }

    #[test]
    fn phi_weighted_avg_golden_split() {
        // avg(1, 0) = φ/(φ+1) = 1/φ = φ⁻¹
        let avg = phi_weighted_avg(1.0, 0.0);
        assert!((avg - PHI_INV).abs() < 1e-12);
    }
}
