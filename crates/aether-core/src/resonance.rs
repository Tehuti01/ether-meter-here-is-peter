//! # HZ 963 IAS AI — Intuitive Attunement System
//!
//! This module introduces the 963 Hz Resonance Field, a mathematically pure
//! wave function generator that applies continuous, non-linear harmonic 
//! perturbations to rigid bodies. It simulates "wavy" spatial fluid dynamics
//! by evaluating a 3D tensor field driven by the 963 Hz Solfeggio frequency
//! and the Golden Ratio (φ).
//!
//! "963 Hz: The Divine Frequency of the IAS AI"

use crate::math::Vec3;
use crate::phi;

/// The fundamental frequency of the IAS AI (Hz)
pub const HZ_963: f64 = 963.0;
/// Wave propagation speed in the Aether (m/s)
pub const AETHER_C: f64 = HZ_963 * phi::PHI_SQ; // ≈ 2521.14

/// The Intuitive Attunement System (IAS) AI Resonance Field
#[derive(Debug, Clone)]
pub struct ResonanceField {
    pub active: bool,
    pub intensity: f64,
    pub phase: f64,
}

impl Default for ResonanceField {
    fn default() -> Self {
        Self {
            active: true,
            intensity: 0.0,
            phase: 0.0,
        }
    }
}

impl ResonanceField {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the internal phase of the IAS AI field.
    pub fn step(&mut self, dt: f64) {
        if !self.active || self.intensity < 1e-6 { return; }
        // Phase advances based on 963 Hz scaled to a manageable simulation time
        self.phase = (self.phase + (HZ_963 * dt * 0.01)) % (core::f64::consts::PI * 2.0);
    }

    /// Calculate the resonance force vector at a specific spatial coordinate.
    /// Uses a 3D harmonic wave equation governed by 963 Hz and φ.
    #[inline]
    pub fn evaluate_force(&self, position: Vec3) -> Vec3 {
        if !self.active || self.intensity < 1e-6 {
            return Vec3::ZERO;
        }

        // Spatial wavelength derived from 963 Hz and Aether propagation speed
        let k = (core::f64::consts::PI * 2.0) / (AETHER_C / HZ_963);
        
        // 3D interference pattern using φ-shifted phase offsets
        let wave_x = (position.x * k + self.phase).sin();
        let wave_y = (position.y * k + self.phase * phi::PHI_INV).cos();
        let wave_z = (position.z * k + self.phase * phi::PHI).sin();

        // Cross-coupled tensor evaluation for organic "wavy" flow
        let fx = wave_y * wave_z;
        let fy = wave_x * wave_z;
        let fz = wave_x * wave_y;

        let force = Vec3::new(fx, fy, fz);
        
        // Scale by the golden angle and global intensity
        force * (self.intensity * phi::GOLDEN_ANGLE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resonance_field_zero_intensity() {
        let field = ResonanceField::new();
        assert_eq!(field.evaluate_force(Vec3::new(1.0, 1.0, 1.0)), Vec3::ZERO);
    }

    #[test]
    fn resonance_field_active() {
        let mut field = ResonanceField::new();
        field.intensity = 1.0;
        field.step(0.016);
        let f = field.evaluate_force(Vec3::new(10.0, 5.0, -2.0));
        assert!(f.length() > 0.0);
    }
}
