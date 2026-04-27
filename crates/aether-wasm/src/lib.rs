//! # Aether-Net Wasm Bindings
//!
//! Exposes the Rust physics engine to JavaScript/TypeScript via wasm-bindgen.
//! Every public API returns/accepts plain types or JsValue for seamless interop.

use wasm_bindgen::prelude::*;
use aether_core::prelude::*;

/// Initialize panic hook for better error messages in browser console.
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

    /// Create a dynamic sphere body. Returns body ID.
    #[wasm_bindgen(js_name = createSphere)]
    pub fn create_sphere(&mut self, radius: f64, mass: f64) -> u32 {
        self.world.create_body(Shape::Sphere { radius }, mass).0
    }

    /// Create a dynamic cuboid body. Returns body ID.
    #[wasm_bindgen(js_name = createCuboid)]
    pub fn create_cuboid(&mut self, hx: f64, hy: f64, hz: f64, mass: f64) -> u32 {
        self.world.create_body(
            Shape::Cuboid { half_extents: Vec3::new(hx, hy, hz) },
            mass,
        ).0
    }

    /// Create a dynamic capsule body. Returns body ID.
    #[wasm_bindgen(js_name = createCapsule)]
    pub fn create_capsule(&mut self, half_height: f64, radius: f64, mass: f64) -> u32 {
        self.world.create_body(
            Shape::Capsule { half_height, radius },
            mass,
        ).0
    }

    /// Create a static ground plane.
    #[wasm_bindgen(js_name = createGroundPlane)]
    pub fn create_ground_plane(&mut self, y_offset: f64) -> u32 {
        self.world.create_static(Shape::Plane {
            normal: Vec3::UP,
            offset: y_offset,
        }).0
    }

    /// Create a static cuboid (e.g., walls, platforms).
    #[wasm_bindgen(js_name = createStaticCuboid)]
    pub fn create_static_cuboid(&mut self, hx: f64, hy: f64, hz: f64) -> u32 {
        self.world.create_static(Shape::Cuboid {
            half_extents: Vec3::new(hx, hy, hz),
        }).0
    }

    /// Set a body's position.
    #[wasm_bindgen(js_name = setPosition)]
    pub fn set_position(&mut self, body_id: u32, x: f64, y: f64, z: f64) {
        self.world.set_position(BodyId(body_id), Vec3::new(x, y, z));
    }

    /// Set a body's linear velocity.
    #[wasm_bindgen(js_name = setVelocity)]
    pub fn set_velocity(&mut self, body_id: u32, vx: f64, vy: f64, vz: f64) {
        self.world.set_velocity(BodyId(body_id), Vec3::new(vx, vy, vz));
    }

    /// Apply a force to a body.
    #[wasm_bindgen(js_name = applyForce)]
    pub fn apply_force(&mut self, body_id: u32, fx: f64, fy: f64, fz: f64) {
        self.world.apply_force(BodyId(body_id), Vec3::new(fx, fy, fz));
    }

    /// Apply an impulse to a body.
    #[wasm_bindgen(js_name = applyImpulse)]
    pub fn apply_impulse(&mut self, body_id: u32, ix: f64, iy: f64, iz: f64) {
        self.world.apply_impulse(BodyId(body_id), Vec3::new(ix, iy, iz));
    }

    /// Get a body's position as [x, y, z].
    #[wasm_bindgen(js_name = getPosition)]
    pub fn get_position(&self, body_id: u32) -> Vec<f64> {
        if let Some(b) = self.world.body(BodyId(body_id)) {
            vec![b.state.position.x, b.state.position.y, b.state.position.z]
        } else {
            vec![0.0, 0.0, 0.0]
        }
    }

    /// Get a body's orientation as [x, y, z, w] quaternion.
    #[wasm_bindgen(js_name = getOrientation)]
    pub fn get_orientation(&self, body_id: u32) -> Vec<f64> {
        if let Some(b) = self.world.body(BodyId(body_id)) {
            let q = b.state.orientation;
            vec![q.x, q.y, q.z, q.w]
        } else {
            vec![0.0, 0.0, 0.0, 1.0]
        }
    }

    /// Get a body's linear velocity as [vx, vy, vz].
    #[wasm_bindgen(js_name = getVelocity)]
    pub fn get_velocity(&self, body_id: u32) -> Vec<f64> {
        if let Some(b) = self.world.body(BodyId(body_id)) {
            vec![b.state.linear_velocity.x, b.state.linear_velocity.y, b.state.linear_velocity.z]
        } else {
            vec![0.0, 0.0, 0.0]
        }
    }

    /// Get all body positions as a flat Float64Array [x0,y0,z0, x1,y1,z1, ...].
    /// Optimized for bulk transfer to Three.js.
    #[wasm_bindgen(js_name = getAllPositions)]
    pub fn get_all_positions(&self) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.world.bodies.len() * 3);
        for b in &self.world.bodies {
            out.push(b.state.position.x);
            out.push(b.state.position.y);
            out.push(b.state.position.z);
        }
        out
    }

    /// Get all body orientations as a flat Float64Array [x0,y0,z0,w0, ...].
    #[wasm_bindgen(js_name = getAllOrientations)]
    pub fn get_all_orientations(&self) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.world.bodies.len() * 4);
        for b in &self.world.bodies {
            out.push(b.state.orientation.x);
            out.push(b.state.orientation.y);
            out.push(b.state.orientation.z);
            out.push(b.state.orientation.w);
        }
        out
    }

    /// Total number of bodies.
    #[wasm_bindgen(js_name = bodyCount)]
    pub fn body_count(&self) -> u32 {
        self.world.body_count() as u32
    }

    /// Number of active (non-sleeping) dynamic bodies.
    #[wasm_bindgen(js_name = activeBodyCount)]
    pub fn active_body_count(&self) -> u32 {
        self.world.active_body_count() as u32
    }

    /// Current simulation time.
    #[wasm_bindgen(js_name = getTime)]
    pub fn get_time(&self) -> f64 {
        self.world.time()
    }

    /// The golden ratio — exposed for JS-side math.
    #[wasm_bindgen(getter)]
    pub fn phi() -> f64 {
        PHI
    }

    /// Frame counter.
    #[wasm_bindgen(getter)]
    pub fn frame(&self) -> u64 {
        self.world.frame
    }
}
