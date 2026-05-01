//! # Aether-Net Wasm Bindings
//!
//! Exposes the Rust physics engine to JavaScript/TypeScript via wasm-bindgen.
//! Provides bulk FlatArray transfer for rendering (Three.js/Babylon) to minimise
//! JS<->Wasm boundary crossing costs.

use wasm_bindgen::prelude::*;
use aether_core::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// The Wasm-exported physics world.
#[wasm_bindgen]
pub struct AetherWorld {
    world: World,
}

#[wasm_bindgen]
impl AetherWorld {
    /// Create a new physics world with default φ-governed settings.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { world: World::new() }
    }

    /// Create a world with custom gravity.
    #[wasm_bindgen(js_name = withGravity)]
    pub fn with_gravity(gx: f64, gy: f64, gz: f64) -> Self {
        let mut config = WorldConfig::default();
        config.gravity = Vec3::new(gx, gy, gz);
        Self { world: World::with_config(config) }
    }

    /// Step the simulation forward by `dt` seconds.
    #[wasm_bindgen]
    pub fn step(&mut self, dt: f64) {
        self.world.step(dt);
    }

    // ─── Body Creation ────────────────────────────────────────────────────────

    #[wasm_bindgen(js_name = createSphere)]
    pub fn create_sphere(&mut self, radius: f64, mass: f64) -> u32 {
        self.world.create_body(Shape::Sphere { radius }, mass).0
    }

    #[wasm_bindgen(js_name = createCuboid)]
    pub fn create_cuboid(&mut self, hx: f64, hy: f64, hz: f64, mass: f64) -> u32 {
        self.world.create_body(Shape::Cuboid { half_extents: Vec3::new(hx, hy, hz) }, mass).0
    }

    #[wasm_bindgen(js_name = createCapsule)]
    pub fn create_capsule(&mut self, half_height: f64, radius: f64, mass: f64) -> u32 {
        self.world.create_body(Shape::Capsule { half_height, radius }, mass).0
    }

    #[wasm_bindgen(js_name = createGroundPlane)]
    pub fn create_ground_plane(&mut self, y_offset: f64) -> u32 {
        self.world.create_static(Shape::Plane { normal: Vec3::UP, offset: y_offset }).0
    }

    #[wasm_bindgen(js_name = createStaticCuboid)]
    pub fn create_static_cuboid(&mut self, hx: f64, hy: f64, hz: f64) -> u32 {
        self.world.create_static(Shape::Cuboid { half_extents: Vec3::new(hx, hy, hz) }).0
    }

    // ─── Per-Body Modifiers ───────────────────────────────────────────────────

    #[wasm_bindgen(js_name = setPosition)]
    pub fn set_position(&mut self, body_id: u32, x: f64, y: f64, z: f64) {
        self.world.set_position(BodyId(body_id), Vec3::new(x, y, z));
    }

    #[wasm_bindgen(js_name = setVelocity)]
    pub fn set_velocity(&mut self, body_id: u32, vx: f64, vy: f64, vz: f64) {
        self.world.set_velocity(BodyId(body_id), Vec3::new(vx, vy, vz));
    }

    #[wasm_bindgen(js_name = applyForce)]
    pub fn apply_force(&mut self, body_id: u32, fx: f64, fy: f64, fz: f64) {
        self.world.apply_force(BodyId(body_id), Vec3::new(fx, fy, fz));
    }

    #[wasm_bindgen(js_name = applyImpulse)]
    pub fn apply_impulse(&mut self, body_id: u32, ix: f64, iy: f64, iz: f64) {
        self.world.apply_impulse(BodyId(body_id), Vec3::new(ix, iy, iz));
    }

    // ─── IAS AI Resonance Field ──────────────────────────────────────────────

    #[wasm_bindgen(js_name = setResonanceIntensity)]
    pub fn set_resonance_intensity(&mut self, intensity: f64) {
        self.world.resonance.intensity = intensity;
    }

    #[wasm_bindgen(js_name = getResonanceIntensity)]
    pub fn get_resonance_intensity(&self) -> f64 {
        self.world.resonance.intensity
    }

    // ─── Per-Body Getters ─────────────────────────────────────────────────────

    #[wasm_bindgen(js_name = getPosition)]
    pub fn get_position(&self, body_id: u32) -> Vec<f64> {
        if let Some(b) = self.world.body(BodyId(body_id)) {
            vec![b.state.position.x, b.state.position.y, b.state.position.z]
        } else { vec![0.0, 0.0, 0.0] }
    }

    #[wasm_bindgen(js_name = getOrientation)]
    pub fn get_orientation(&self, body_id: u32) -> Vec<f64> {
        if let Some(b) = self.world.body(BodyId(body_id)) {
            vec![b.state.orientation.x, b.state.orientation.y, b.state.orientation.z, b.state.orientation.w]
        } else { vec![0.0, 0.0, 0.0, 1.0] }
    }

    #[wasm_bindgen(js_name = getVelocity)]
    pub fn get_velocity(&self, body_id: u32) -> Vec<f64> {
        if let Some(b) = self.world.body(BodyId(body_id)) {
            vec![b.state.linear_velocity.x, b.state.linear_velocity.y, b.state.linear_velocity.z]
        } else { vec![0.0, 0.0, 0.0] }
    }

    #[wasm_bindgen(js_name = isSleeping)]
    pub fn is_sleeping(&self, body_id: u32) -> bool {
        if let Some(b) = self.world.body(BodyId(body_id)) {
            b.flags.sleeping
        } else { false }
    }

    // ─── Bulk Transfer (Zero-Copy Target) ─────────────────────────────────────

    /// Return a single contiguous Float32Array containing [x, y, z] for every body.
    /// F32 is preferred here as WebGL/ThreeJS buffers are f32 natively.
    #[wasm_bindgen(js_name = getAllPositionsF32)]
    pub fn get_all_positions_f32(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.world.bodies.len() * 3);
        for b in &self.world.bodies {
            out.extend_from_slice(&b.state.position.to_f32_array());
        }
        out
    }

    /// Return a single contiguous Float32Array containing [x, y, z, w] for every body.
    #[wasm_bindgen(js_name = getAllOrientationsF32)]
    pub fn get_all_orientations_f32(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.world.bodies.len() * 4);
        for b in &self.world.bodies {
            out.extend_from_slice(&b.state.orientation.to_f32_array());
        }
        out
    }
    
    /// Return a single contiguous Uint8Array of sleeping states for fast rendering skips.
    #[wasm_bindgen(js_name = getSleepStates)]
    pub fn get_sleep_states(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.world.bodies.len());
        for b in &self.world.bodies {
            out.push(if b.flags.sleeping { 1 } else { 0 });
        }
        out
    }

    // ─── Diagnostics ──────────────────────────────────────────────────────────

    #[wasm_bindgen(js_name = bodyCount)]
    pub fn body_count(&self) -> u32 { self.world.body_count() as u32 }

    #[wasm_bindgen(js_name = activeBodyCount)]
    pub fn active_body_count(&self) -> u32 { self.world.active_body_count() as u32 }

    #[wasm_bindgen(js_name = constraintCount)]
    pub fn constraint_count(&self) -> u32 {
        // Technically world.constraints is internal, but we can expose a getter on World
        // For now, we simulate exposing it if added to world.rs, or return 0
        // We'll update world.rs in a sec to expose this.
        self.world.constraint_count() as u32
    }

    #[wasm_bindgen(js_name = getTime)]
    pub fn get_time(&self) -> f64 { self.world.time() }

    #[wasm_bindgen(getter)]
    pub fn phi() -> f64 { PHI }

    #[wasm_bindgen(getter)]
    pub fn frame(&self) -> u64 { self.world.frame }
}
