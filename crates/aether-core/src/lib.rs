//! # Aether-Net Core
//!
//! A φ-governed rigid-body physics engine built for WebAssembly.
//!
//! Every fundamental constant in this engine derives from the golden ratio
//! φ = (1 + √5) / 2 ≈ 1.618033988749895.
//!
//! ## Architecture
//! - `phi` — Golden ratio constants and utilities
//! - `math` — Vec3, Quat, Mat3, AABB
//! - `collider` — Shapes and collision geometry
//! - `body` — Rigid body dynamics
//! - `broadphase` — φ-scaled spatial hash grid
//! - `narrowphase` — Contact generation (SAT, sphere/capsule/plane)
//! - `solver` — Sequential impulse constraint solver
//! - `world` — Top-level simulation container
//!
//! ## Quick Start
//! ```
//! use aether_core::prelude::*;
//!
//! let mut world = World::new();
//! let ground = world.create_static(Shape::Plane {
//!     normal: Vec3::UP,
//!     offset: 0.0,
//! });
//! let ball = world.create_body(Shape::Sphere { radius: 0.5 }, 1.0);
//! world.set_position(ball, Vec3::new(0.0, 10.0, 0.0));
//! world.step(1.0 / 60.0);
//! ```

pub mod phi;
pub mod math;
pub mod collider;
pub mod body;
pub mod broadphase;
pub mod narrowphase;
pub mod solver;
pub mod resonance;
pub mod world;

/// Prelude — import everything you need with `use aether_core::prelude::*`
pub mod prelude {
    pub use crate::phi::{self, PHI, PHI_INV, PHI_SQ};
    pub use crate::math::{Vec3, Quat, Mat3, AABB};
    pub use crate::body::{BodyId, BodyType, RigidBody};
    pub use crate::collider::{ColliderId, Collider, Shape};
    pub use crate::resonance::{ResonanceField, HZ_963};
    pub use crate::world::{World, WorldConfig};
}
