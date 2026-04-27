//! # Rigid Body
//!
//! The fundamental simulation entity. Each body has position, orientation,
//! velocity, mass properties, and a list of attached colliders.

use crate::math::{Vec3, Quat, Mat3};
use crate::collider::{ColliderId, Shape};
use crate::phi;

/// Unique handle for a rigid body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyId(pub u32);

/// Body type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    /// Fully simulated — affected by forces and collisions
    Dynamic,
    /// Infinite mass — only moved by user, affects dynamics
    Kinematic,
    /// Immovable and infinite mass
    Static,
}

/// Motion state — separates position/velocity for cleaner integration.
#[derive(Debug, Clone, Copy)]
pub struct MotionState {
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
}

impl Default for MotionState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
        }
    }
}

/// Mass properties — precomputed for solver efficiency.
#[derive(Debug, Clone, Copy)]
pub struct MassProperties {
    pub mass: f64,
    pub inv_mass: f64,
    pub inertia_local: Mat3,
    pub inv_inertia_local: Mat3,
}

impl MassProperties {
    pub fn from_shape(shape: &Shape, mass: f64) -> Self {
        if mass <= 0.0 || mass.is_infinite() {
            return Self::infinite();
        }
        let inertia = shape.inertia_tensor(mass);
        Self {
            mass,
            inv_mass: 1.0 / mass,
            inertia_local: inertia,
            inv_inertia_local: inertia.inverse(),
        }
    }

    pub fn infinite() -> Self {
        Self {
            mass: f64::INFINITY,
            inv_mass: 0.0,
            inertia_local: Mat3::ZERO,
            inv_inertia_local: Mat3::ZERO,
        }
    }

    /// Compute world-space inverse inertia tensor
    #[inline]
    pub fn inv_inertia_world(&self, orientation: &Quat) -> Mat3 {
        let rot = orientation.to_mat3();
        rot.mul_mat(self.inv_inertia_local).mul_mat(rot.transpose())
    }
}

/// Flags controlling body behavior.
#[derive(Debug, Clone, Copy)]
pub struct BodyFlags {
    pub gravity_enabled: bool,
    pub continuous_collision: bool,
    pub sleeping: bool,
    pub sleep_counter: u32,
}

impl Default for BodyFlags {
    fn default() -> Self {
        Self {
            gravity_enabled: true,
            continuous_collision: false,
            sleeping: false,
            sleep_counter: 0,
        }
    }
}

/// A rigid body in the physics world.
#[derive(Debug, Clone)]
pub struct RigidBody {
    pub id: BodyId,
    pub body_type: BodyType,
    pub state: MotionState,
    pub prev_state: MotionState,
    pub mass_props: MassProperties,
    pub flags: BodyFlags,
    pub collider_ids: Vec<ColliderId>,
    /// Accumulated force this frame
    pub force_accumulator: Vec3,
    /// Accumulated torque this frame
    pub torque_accumulator: Vec3,
    /// φ-scaled linear damping
    pub linear_damping: f64,
    /// φ-scaled angular damping
    pub angular_damping: f64,
    /// User data slot
    pub user_data: u64,
}

impl RigidBody {
    pub fn new(id: BodyId, body_type: BodyType) -> Self {
        let mass_props = match body_type {
            BodyType::Dynamic => MassProperties::from_shape(&Shape::Sphere { radius: 0.5 }, 1.0),
            _ => MassProperties::infinite(),
        };
        Self {
            id,
            body_type,
            state: MotionState::default(),
            prev_state: MotionState::default(),
            mass_props,
            flags: BodyFlags::default(),
            collider_ids: Vec::new(),
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            linear_damping: phi::LINEAR_DAMPING,
            angular_damping: phi::ANGULAR_DAMPING,
            user_data: 0,
        }
    }

    pub fn with_position(mut self, pos: Vec3) -> Self {
        self.state.position = pos;
        self.prev_state.position = pos;
        self
    }

    pub fn with_orientation(mut self, q: Quat) -> Self {
        self.state.orientation = q;
        self.prev_state.orientation = q;
        self
    }

    pub fn with_mass(mut self, mass: f64, shape: &Shape) -> Self {
        self.mass_props = MassProperties::from_shape(shape, mass);
        self
    }

    /// Apply a force at the center of mass (accumulated over the frame)
    #[inline]
    pub fn apply_force(&mut self, force: Vec3) {
        if self.body_type != BodyType::Dynamic { return; }
        self.force_accumulator += force;
    }

    /// Apply a force at a world-space point (generates torque)
    #[inline]
    pub fn apply_force_at_point(&mut self, force: Vec3, point: Vec3) {
        if self.body_type != BodyType::Dynamic { return; }
        self.force_accumulator += force;
        self.torque_accumulator += (point - self.state.position).cross(force);
    }

    /// Apply an impulse at center of mass
    #[inline]
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type != BodyType::Dynamic { return; }
        self.state.linear_velocity += impulse * self.mass_props.inv_mass;
    }

    /// Apply an impulse at a world-space point
    #[inline]
    pub fn apply_impulse_at_point(&mut self, impulse: Vec3, point: Vec3) {
        if self.body_type != BodyType::Dynamic { return; }
        self.state.linear_velocity += impulse * self.mass_props.inv_mass;
        let r = point - self.state.position;
        let inv_i = self.mass_props.inv_inertia_world(&self.state.orientation);
        self.state.angular_velocity += inv_i.mul_vec(r.cross(impulse));
    }

    /// Clear accumulated forces and torques
    #[inline]
    pub fn clear_forces(&mut self) {
        self.force_accumulator = Vec3::ZERO;
        self.torque_accumulator = Vec3::ZERO;
    }

    /// Check if velocity is below φ-sleep threshold
    pub fn check_sleeping(&mut self) {
        if self.body_type != BodyType::Dynamic { return; }
        let lin = self.state.linear_velocity.length_squared();
        let ang = self.state.angular_velocity.length_squared();
        let threshold_sq = phi::SLEEP_THRESHOLD * phi::SLEEP_THRESHOLD;
        if lin < threshold_sq && ang < threshold_sq {
            self.flags.sleep_counter += 1;
            if self.flags.sleep_counter > phi::FIBONACCI[6] as u32 { // 13 frames
                self.flags.sleeping = true;
                self.state.linear_velocity = Vec3::ZERO;
                self.state.angular_velocity = Vec3::ZERO;
            }
        } else {
            self.flags.sleep_counter = 0;
            self.flags.sleeping = false;
        }
    }

    /// Wake up from sleep
    #[inline]
    pub fn wake_up(&mut self) {
        self.flags.sleeping = false;
        self.flags.sleep_counter = 0;
    }

    #[inline]
    pub fn is_dynamic(&self) -> bool { self.body_type == BodyType::Dynamic }

    #[inline]
    pub fn is_static(&self) -> bool { self.body_type == BodyType::Static }
}
