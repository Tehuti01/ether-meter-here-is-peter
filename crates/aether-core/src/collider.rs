//! # Collider Shapes
//!
//! Geometric primitives used for collision detection.
//! Each shape can compute its own AABB, inertia tensor, and support point.

use crate::math::{Vec3, Mat3, AABB};
use crate::phi;

/// Unique handle for a collider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColliderId(pub u32);

/// Geometric shape variants for collision.
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    /// Sphere with radius
    Sphere { radius: f64 },
    /// Box with half-extents
    Cuboid { half_extents: Vec3 },
    /// Capsule (line segment + radius) along Y axis
    Capsule { half_height: f64, radius: f64 },
    /// Infinite plane defined by normal + offset
    Plane { normal: Vec3, offset: f64 },
}

impl Shape {
    /// Compute AABB in local space
    pub fn local_aabb(&self) -> AABB {
        match self {
            Shape::Sphere { radius } => {
                let r = Vec3::splat(*radius);
                AABB::new(-r, r)
            }
            Shape::Cuboid { half_extents } => {
                AABB::new(-*half_extents, *half_extents)
            }
            Shape::Capsule { half_height, radius } => {
                let he = Vec3::new(*radius, *half_height + *radius, *radius);
                AABB::new(-he, he)
            }
            Shape::Plane { normal, offset } => {
                // Use a large but finite AABB for the plane
                let _ = (normal, offset);
                let big = 50.0;
                AABB::new(
                    Vec3::new(-big, -0.01, -big),
                    Vec3::new(big, 0.01, big),
                )
            }
        }
    }

    /// Compute the inertia tensor for this shape given a mass.
    /// Uses standard physics formulas.
    pub fn inertia_tensor(&self, mass: f64) -> Mat3 {
        match self {
            Shape::Sphere { radius } => {
                let i = 0.4 * mass * radius * radius; // (2/5)mr²
                Mat3::from_diagonal(Vec3::splat(i))
            }
            Shape::Cuboid { half_extents } => {
                let e = *half_extents;
                let sx = e.x * 2.0;
                let sy = e.y * 2.0;
                let sz = e.z * 2.0;
                let factor = mass / 12.0;
                Mat3::from_diagonal(Vec3::new(
                    factor * (sy * sy + sz * sz),
                    factor * (sx * sx + sz * sz),
                    factor * (sx * sx + sy * sy),
                ))
            }
            Shape::Capsule { half_height, radius } => {
                let r = *radius;
                let h = *half_height * 2.0;
                // Approximate as cylinder + 2 hemispheres
                let cyl_mass = mass * h / (h + (4.0 / 3.0) * r);
                let cap_mass = mass - cyl_mass;
                let ix_cyl = cyl_mass * (3.0 * r * r + h * h) / 12.0;
                let iy_cyl = cyl_mass * r * r / 2.0;
                let ix_cap = cap_mass * (2.0 * r * r / 5.0 + h * h / 4.0 + 3.0 * h * r / 8.0);
                let iy_cap = cap_mass * 2.0 * r * r / 5.0;
                Mat3::from_diagonal(Vec3::new(
                    ix_cyl + ix_cap,
                    iy_cyl + iy_cap,
                    ix_cyl + ix_cap,
                ))
            }
            Shape::Plane { .. } => {
                // Infinite mass / static — return zero inverse inertia
                Mat3::ZERO
            }
        }
    }

    /// GJK support function: furthest point in direction `dir` (local space)
    pub fn support(&self, dir: Vec3) -> Vec3 {
        match self {
            Shape::Sphere { radius } => dir.normalize() * *radius,
            Shape::Cuboid { half_extents } => Vec3::new(
                if dir.x >= 0.0 { half_extents.x } else { -half_extents.x },
                if dir.y >= 0.0 { half_extents.y } else { -half_extents.y },
                if dir.z >= 0.0 { half_extents.z } else { -half_extents.z },
            ),
            Shape::Capsule { half_height, radius } => {
                let base = if dir.y >= 0.0 {
                    Vec3::new(0.0, *half_height, 0.0)
                } else {
                    Vec3::new(0.0, -*half_height, 0.0)
                };
                base + dir.normalize() * *radius
            }
            Shape::Plane { normal, offset } => {
                // Not meaningful for GJK, but provide something
                *normal * *offset
            }
        }
    }
}

/// A collider attached to a rigid body.
#[derive(Debug, Clone)]
pub struct Collider {
    pub id: ColliderId,
    pub shape: Shape,
    /// Local-space offset from the body's center of mass
    pub local_position: Vec3,
    /// Coefficient of restitution (bounciness), decayed by φ⁻¹ per bounce
    pub restitution: f64,
    /// Coefficient of friction
    pub friction: f64,
    /// Density for mass computation (if mass not set directly)
    pub density: f64,
}

impl Collider {
    pub fn new(id: ColliderId, shape: Shape) -> Self {
        Self {
            id,
            shape,
            local_position: Vec3::ZERO,
            restitution: phi::PHI_INV, // φ⁻¹ ≈ 0.618 — golden bounciness
            friction: 1.0 - phi::PHI_INV, // 1 - φ⁻¹ ≈ 0.382
            density: phi::PHI, // φ ≈ 1.618 kg/m³ base density
        }
    }

    pub fn with_restitution(mut self, r: f64) -> Self { self.restitution = r; self }
    pub fn with_friction(mut self, f: f64) -> Self { self.friction = f; self }
    pub fn with_density(mut self, d: f64) -> Self { self.density = d; self }
    pub fn with_offset(mut self, pos: Vec3) -> Self { self.local_position = pos; self }
}
