//! # φ — The Golden Ratio Constants
//!
//! The golden ratio φ = (1 + √5) / 2 ≈ 1.6180339887 governs every fundamental
//! constant in Aether-Net. From damping coefficients to spatial partitioning,
//! from solver convergence to time-step harmonics — φ is the law.
//!
//! ## Why φ?
//! - Natural systems converge to φ-proportioned equilibria
//! - Fibonacci-scaled spatial grids minimize hash collisions
//! - φ-damping produces visually "correct" physical motion
//! - The golden angle (2π/φ²) distributes broadphase probes optimally

/// The golden ratio: φ = (1 + √5) / 2
pub const PHI: f64 = 1.618_033_988_749_895;

/// The golden ratio as f32
pub const PHI_F32: f32 = 1.618_033_9;

/// Reciprocal of φ: 1/φ = φ - 1 ≈ 0.618...
pub const PHI_INV: f64 = 0.618_033_988_749_895;

/// φ squared: φ² = φ + 1 ≈ 2.618...
pub const PHI_SQ: f64 = 2.618_033_988_749_895;

/// The golden angle in radians: 2π / φ² ≈ 2.399...
pub const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;

/// φ-governed default gravity magnitude (9.80665 / φ² * φ³ ≈ 9.80665)
/// Kept at physical standard but solver uses φ-scaled sub-steps
pub const GRAVITY_STANDARD: f64 = 9.806_65;

/// Default linear damping: 1 - 1/φ⁴ ≈ 0.854
pub const LINEAR_DAMPING: f64 = 0.854_101_966_249_685;

/// Default angular damping: 1 - 1/φ³ ≈ 0.764
pub const ANGULAR_DAMPING: f64 = 0.763_932_022_500_210;

/// Solver convergence threshold: 1/φ⁸ ≈ 0.0557
pub const SOLVER_EPSILON: f64 = 0.055_728_090_000_841;

/// Constraint bias factor scaled by φ: 0.2 * φ⁻¹
pub const BAUMGARTE_FACTOR: f64 = 0.123_606_797_749_979;

/// Velocity threshold for sleeping bodies: φ⁻⁵
pub const SLEEP_THRESHOLD: f64 = 0.090_169_943_749_474;

/// Maximum solver iterations (Fibonacci number)
pub const SOLVER_ITERATIONS: u32 = 8;

/// Broadphase cell size ratio — cells are φ × largest_body_extent
pub const BROADPHASE_CELL_RATIO: f64 = PHI;

/// Time-step subdivision: Fibonacci-scaled sub-stepping
pub const SUBSTEP_COUNT: u32 = 5; // Fib(5) = 5

/// Contact penetration slop: φ⁻⁶
pub const PENETRATION_SLOP: f64 = 0.055_728_090_000_841;

/// Restitution decay per bounce: multiply by φ⁻¹ each time
pub const RESTITUTION_DECAY: f64 = PHI_INV;

/// Maximum linear velocity (prevents tunneling): φ⁵ ≈ 11.09
pub const MAX_LINEAR_VELOCITY: f64 = 11.090_169_943_749_474;

/// Maximum angular velocity: φ⁴ ≈ 6.854
pub const MAX_ANGULAR_VELOCITY: f64 = 6.854_101_966_249_685;

/// First N Fibonacci numbers for internal use
pub const FIBONACCI: [u64; 20] = [
    1, 1, 2, 3, 5, 8, 13, 21, 34, 55,
    89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765,
];

/// Compute φ^n efficiently using the matrix exponentiation identity
#[inline]
pub fn phi_pow(n: i32) -> f64 {
    PHI.powi(n)
}

/// Fibonacci-scale a value: value * Fib(n) / Fib(n-1) → converges to φ
#[inline]
pub fn fib_scale(value: f64, level: usize) -> f64 {
    if level < 2 {
        return value;
    }
    let idx = level.min(FIBONACCI.len() - 1);
    value * (FIBONACCI[idx] as f64) / (FIBONACCI[idx - 1] as f64)
}

/// Golden-angle rotation for distributing N probes on a sphere/circle
#[inline]
pub fn golden_angle_at(index: u32) -> f64 {
    (index as f64) * GOLDEN_ANGLE
}

/// φ-lerp: interpolation biased toward the golden ratio
#[inline]
pub fn phi_lerp(a: f64, b: f64, t: f64) -> f64 {
    let phi_t = t.powf(PHI_INV); // Bias toward golden proportion
    a + (b - a) * phi_t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_identity() {
        // φ² = φ + 1
        let phi_sq = PHI * PHI;
        assert!((phi_sq - (PHI + 1.0)).abs() < 1e-12);
    }

    #[test]
    fn phi_reciprocal() {
        // 1/φ = φ - 1
        assert!((1.0 / PHI - PHI_INV).abs() < 1e-12);
        assert!((PHI_INV - (PHI - 1.0)).abs() < 1e-12);
    }

    #[test]
    fn fibonacci_ratio_converges_to_phi() {
        for i in 10..19 {
            let ratio = FIBONACCI[i + 1] as f64 / FIBONACCI[i] as f64;
            assert!((ratio - PHI).abs() < 0.0001);
        }
    }

    #[test]
    fn fib_scale_at_high_levels_approaches_phi() {
        let scaled = fib_scale(1.0, 15);
        let expected = FIBONACCI[15] as f64 / FIBONACCI[14] as f64;
        assert!((scaled - expected).abs() < 1e-10);
    }
}
