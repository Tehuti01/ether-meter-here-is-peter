//! # Physics World
//!
//! The top-level simulation container. Manages bodies, colliders,
//! and orchestrates the simulation loop with φ-sub-stepping.

use crate::math::{Vec3, AABB};
use crate::body::{BodyId, BodyType, RigidBody};
use crate::collider::{Collider, ColliderId, Shape};
use crate::broadphase::SpatialGrid;
use crate::narrowphase;
use crate::solver::{self, ContactConstraint};
use crate::phi;

/// Configuration for the physics world.
#[derive(Debug, Clone, Copy)]
pub struct WorldConfig {
    pub gravity: Vec3,
    pub substeps: u32,
    pub solver_iterations: u32,
    pub linear_damping: f64,
    pub angular_damping: f64,
    pub sleep_enabled: bool,
    pub broadphase_cell_size: f64,
    pub resonance_intensity: f64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -phi::GRAVITY_STANDARD, 0.0),
            substeps: phi::SUBSTEP_COUNT,
            solver_iterations: phi::SOLVER_ITERATIONS,
            linear_damping: phi::LINEAR_DAMPING,
            angular_damping: phi::ANGULAR_DAMPING,
            sleep_enabled: true,
            broadphase_cell_size: phi::PHI * 2.0,
            resonance_intensity: 0.0,
        }
    }
}

/// The physics world — owns all simulation state.
pub struct World {
    pub config: WorldConfig,
    pub bodies: Vec<RigidBody>,
    pub colliders: Vec<Collider>,
    broadphase: SpatialGrid,
    constraints: Vec<ContactConstraint>,
    pub resonance: crate::resonance::ResonanceField,
    next_body_id: u32,
    next_collider_id: u32,
    /// Simulation time accumulator
    time: f64,
    /// Frame counter
    pub frame: u64,
    pub constraint_count: usize,
}

impl World {
    pub fn new() -> Self {
        Self::with_config(WorldConfig::default())
    }

    pub fn with_config(config: WorldConfig) -> Self {
        let mut res = crate::resonance::ResonanceField::new();
        res.intensity = config.resonance_intensity;
        Self {
            broadphase: SpatialGrid::new(config.broadphase_cell_size),
            config,
            bodies: Vec::with_capacity(256),
            colliders: Vec::with_capacity(256),
            constraints: Vec::with_capacity(512),
            resonance: res,
            next_body_id: 0,
            next_collider_id: 0,
            time: 0.0,
            frame: 0,
            constraint_count: 0,
        }
    }

    /// Create a dynamic rigid body with the given shape and mass.
    pub fn create_body(&mut self, shape: Shape, mass: f64) -> BodyId {
        let id = BodyId(self.next_body_id);
        self.next_body_id += 1;

        let coll_id = ColliderId(self.next_collider_id);
        self.next_collider_id += 1;
        let collider = Collider::new(coll_id, shape);
        self.colliders.push(collider);

        let mut body = RigidBody::new(id, BodyType::Dynamic)
            .with_mass(mass, &shape);
        body.collider_ids.push(coll_id);
        body.linear_damping = self.config.linear_damping;
        body.angular_damping = self.config.angular_damping;
        self.bodies.push(body);
        id
    }

    /// Create a static body (ground plane, walls, etc.)
    pub fn create_static(&mut self, shape: Shape) -> BodyId {
        let id = BodyId(self.next_body_id);
        self.next_body_id += 1;

        let coll_id = ColliderId(self.next_collider_id);
        self.next_collider_id += 1;
        self.colliders.push(Collider::new(coll_id, shape));

        let mut body = RigidBody::new(id, BodyType::Static);
        body.collider_ids.push(coll_id);
        self.bodies.push(body);
        id
    }

    /// Get a mutable reference to a body by ID.
    pub fn body_mut(&mut self, id: BodyId) -> Option<&mut RigidBody> {
        self.bodies.iter_mut().find(|b| b.id == id)
    }

    /// Get an immutable reference to a body by ID.
    pub fn body(&self, id: BodyId) -> Option<&RigidBody> {
        self.bodies.iter().find(|b| b.id == id)
    }

    /// Set position of a body.
    pub fn set_position(&mut self, id: BodyId, pos: Vec3) {
        if let Some(b) = self.body_mut(id) {
            b.state.position = pos;
            b.prev_state.position = pos;
            b.wake_up();
        }
    }

    /// Set linear velocity.
    pub fn set_velocity(&mut self, id: BodyId, vel: Vec3) {
        if let Some(b) = self.body_mut(id) {
            b.state.linear_velocity = vel;
            b.wake_up();
        }
    }

    /// Apply a force to a body (accumulated for the next step).
    pub fn apply_force(&mut self, id: BodyId, force: Vec3) {
        if let Some(b) = self.body_mut(id) { b.apply_force(force); }
    }

    /// Apply an impulse to a body (immediate velocity change).
    pub fn apply_impulse(&mut self, id: BodyId, impulse: Vec3) {
        if let Some(b) = self.body_mut(id) { b.apply_impulse(impulse); }
    }

    /// Step the simulation forward by `dt` seconds.
    ///
    /// Uses φ-sub-stepping: the timestep is divided into `substeps` sub-steps,
    /// each of duration `dt / substeps`. This provides numerical stability
    /// while keeping the per-frame API simple.
    pub fn step(&mut self, dt: f64) {
        if dt <= 0.0 { return; }
        
        self.resonance.step(dt);
        
        let sub_dt = dt / self.config.substeps as f64;

        for _ in 0..self.config.substeps {
            self.substep(sub_dt);
        }
        self.time += dt;
        self.frame += 1;
    }

    fn substep(&mut self, dt: f64) {
        // 1. Apply gravity and integrate forces → velocity
        self.integrate_forces(dt);

        // 2. Broadphase: find potential collision pairs
        let pairs = self.broadphase_detect();

        // 3. Narrow-phase: generate contacts
        self.constraints.clear();
        for pair in &pairs {
            self.narrow_phase(pair.0, pair.1);
        }

        // 4. Solve constraints
        solver::solve_contacts(&mut self.bodies, &mut self.constraints, dt);

        // 5. Integrate velocities → positions
        self.integrate_velocities(dt);

        // 6. Clear forces & check sleeping
        for body in &mut self.bodies {
            body.clear_forces();
            if self.config.sleep_enabled {
                body.check_sleeping();
            }
        }
    }

    fn integrate_forces(&mut self, dt: f64) {
        let gravity = self.config.gravity;
        let is_res_active = self.resonance.active && self.resonance.intensity > 1e-6;
        for body in &mut self.bodies {
            if !body.is_dynamic() || body.flags.sleeping { continue; }
            // Save previous state for interpolation
            body.prev_state = body.state;
            // Gravity
            if body.flags.gravity_enabled {
                body.state.linear_velocity += gravity * dt;
            }
            // HZ 963 Resonance
            if is_res_active {
                let res_force = self.resonance.evaluate_force(body.state.position);
                body.state.linear_velocity += res_force * (body.mass_props.inv_mass * dt);
            }
            // External forces
            body.state.linear_velocity += body.force_accumulator * (body.mass_props.inv_mass * dt);
            let inv_i = body.mass_props.inv_inertia_world(&body.state.orientation);
            body.state.angular_velocity += inv_i.mul_vec(body.torque_accumulator) * dt;
            // Damping (φ-scaled, applied per-second regardless of sub-dt)
            let damp_factor = 1.0 - (1.0 - body.linear_damping) * dt;
            body.state.linear_velocity *= damp_factor.max(0.0);
            let ang_damp = 1.0 - (1.0 - body.angular_damping) * dt;
            body.state.angular_velocity *= ang_damp.max(0.0);
            // Velocity clamping
            body.state.linear_velocity = body.state.linear_velocity
                .clamp_length(phi::MAX_LINEAR_VELOCITY);
            body.state.angular_velocity = body.state.angular_velocity
                .clamp_length(phi::MAX_ANGULAR_VELOCITY);
        }
    }

    fn broadphase_detect(&mut self) -> Vec<crate::broadphase::BroadPair> {
        self.broadphase.clear();
        let mut plane_indices: Vec<u32> = Vec::new();

        for (idx, body) in self.bodies.iter().enumerate() {
            if body.flags.sleeping && body.is_static() { continue; }
            // Check if this body has a plane collider — planes skip the grid
            let is_plane = body.collider_ids.iter().any(|cid| {
                self.colliders.iter().any(|c| c.id == *cid && matches!(c.shape, Shape::Plane { .. }))
            });
            if is_plane {
                plane_indices.push(idx as u32);
            } else {
                let aabb = self.body_aabb(idx);
                self.broadphase.insert(idx as u32, &aabb);
            }
        }

        let mut pairs = self.broadphase.find_pairs();

        // Pair every plane with every dynamic body (O(planes × dynamic_bodies))
        for &plane_idx in &plane_indices {
            for (idx, body) in self.bodies.iter().enumerate() {
                if idx as u32 == plane_idx { continue; }
                if body.is_static() { continue; }
                pairs.push(crate::broadphase::BroadPair::new(plane_idx, idx as u32));
            }
        }
        pairs
    }

    fn body_aabb(&self, idx: usize) -> AABB {
        let body = &self.bodies[idx];
        let mut result = AABB::new(body.state.position, body.state.position);
        for cid in &body.collider_ids {
            if let Some(coll) = self.colliders.iter().find(|c| c.id == *cid) {
                let local_aabb = coll.shape.local_aabb();
                // Transform AABB to world space (conservative)
                let center = body.state.position + body.state.orientation.rotate_vec(coll.local_position);
                let ext = local_aabb.half_extents();
                // Rotate extents (conservative bound)
                let rot = body.state.orientation.to_mat3();
                let world_ext = Vec3::new(
                    (rot.cols[0] * ext.x).abs().max_comp()
                        + (rot.cols[1] * ext.y).abs().max_comp()
                        + (rot.cols[2] * ext.z).abs().max_comp(),
                    (rot.cols[0] * ext.x).abs().max_comp()
                        + (rot.cols[1] * ext.y).abs().max_comp()
                        + (rot.cols[2] * ext.z).abs().max_comp(),
                    (rot.cols[0] * ext.x).abs().max_comp()
                        + (rot.cols[1] * ext.y).abs().max_comp()
                        + (rot.cols[2] * ext.z).abs().max_comp(),
                );
                let aabb = AABB::from_center_half(center, world_ext);
                result = result.merge(&aabb);
            }
        }
        result.expanded(phi::PENETRATION_SLOP)
    }

    fn narrow_phase(&mut self, idx_a: u32, idx_b: u32) {
        let a = &self.bodies[idx_a as usize];
        let b = &self.bodies[idx_b as usize];
        // Don't test static-static pairs
        if a.is_static() && b.is_static() { return; }

        for cid_a in &a.collider_ids {
            for cid_b in &b.collider_ids {
                let ca = self.colliders.iter().find(|c| c.id == *cid_a);
                let cb = self.colliders.iter().find(|c| c.id == *cid_b);
                if let (Some(ca), Some(cb)) = (ca, cb) {
                    let manifold = narrowphase::collide(
                        &ca.shape, a.state.position, a.state.orientation,
                        &cb.shape, b.state.position, b.state.orientation,
                    );
                    for contact in manifold.contacts {
                        let restitution = (ca.restitution * cb.restitution).sqrt();
                        let friction = (ca.friction * cb.friction).sqrt();
                        self.constraints.push(ContactConstraint::new(
                            idx_a, idx_b, contact, restitution, friction,
                        ));
                    }
                }
            }
        }
    }

    fn integrate_velocities(&mut self, dt: f64) {
        for body in &mut self.bodies {
            if !body.is_dynamic() || body.flags.sleeping { continue; }
            body.state.position += body.state.linear_velocity * dt;
            body.state.orientation = body.state.orientation
                .integrate(body.state.angular_velocity, dt);
        }
    }

    /// Get the number of active (non-sleeping) bodies.
    pub fn active_body_count(&self) -> usize {
        self.bodies.iter().filter(|b| b.is_dynamic() && !b.flags.sleeping).count()
    }

    /// Get total body count.
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Current simulation time.
    pub fn time(&self) -> f64 {
        self.time
    }
}

impl Default for World {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gravity_makes_body_fall() {
        let mut world = World::new();
        let id = world.create_body(Shape::Sphere { radius: 0.5 }, 1.0);
        world.set_position(id, Vec3::new(0.0, 10.0, 0.0));
        // Step for 1 second at 60fps
        for _ in 0..60 {
            world.step(1.0 / 60.0);
        }
        let body = world.body(id).unwrap();
        // Body should have fallen significantly
        assert!(body.state.position.y < 10.0);
        assert!(body.state.position.y < 6.0); // ~half of free-fall in 1s
    }

    #[test]
    fn sphere_rests_on_ground_plane() {
        let mut world = World::new();
        let _ground = world.create_static(Shape::Plane {
            normal: Vec3::UP,
            offset: 0.0,
        });
        let ball = world.create_body(Shape::Sphere { radius: 0.5 }, 1.0);
        world.set_position(ball, Vec3::new(0.0, 2.0, 0.0));
        // Simulate 2 seconds (short drop)
        for _ in 0..120 {
            world.step(1.0 / 60.0);
        }
        let body = world.body(ball).unwrap();
        // Ball should have fallen from y=2 and be near/below starting height
        assert!(body.state.position.y < 2.0, "Ball should have fallen, y={}", body.state.position.y);
        // Should not have fallen through the floor
        assert!(body.state.position.y > -1.0, "Ball fell through floor, y={}", body.state.position.y);
    }

    #[test]
    fn body_count_tracking() {
        let mut world = World::new();
        assert_eq!(world.body_count(), 0);
        world.create_body(Shape::Cuboid { half_extents: Vec3::ONE }, 5.0);
        world.create_body(Shape::Sphere { radius: 1.0 }, 2.0);
        assert_eq!(world.body_count(), 2);
    }
}
impl World {
    pub fn constraint_count(&self) -> usize { self.constraints.len() }
}
