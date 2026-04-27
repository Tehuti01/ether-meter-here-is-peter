//! # Linear Algebra Primitives
//!
//! Zero-allocation vector, quaternion, and matrix types for the physics engine.
//! All operations are `#[inline]` for maximum codegen optimization.

use core::ops::{Add, Sub, Mul, Neg, AddAssign, SubAssign, MulAssign};

// ─────────────────────────────── Vec3 ───────────────────────────────

/// A 3D vector with f64 precision.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: 1.0 };

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn splat(v: f64) -> Self {
        Self { x: v, y: v, z: v }
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[inline]
    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len < 1e-12 {
            Self::ZERO
        } else {
            self * (1.0 / len)
        }
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        self + (other - self) * t
    }

    #[inline]
    pub fn max_component(self) -> f64 {
        self.x.max(self.y).max(self.z)
    }

    #[inline]
    pub fn min_component(self) -> f64 {
        self.x.min(self.y).min(self.z)
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self { x: self.x.abs(), y: self.y.abs(), z: self.z.abs() }
    }

    #[inline]
    pub fn clamp_length(self, max: f64) -> Self {
        let len_sq = self.length_squared();
        if len_sq > max * max {
            self.normalize() * max
        } else {
            self
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    #[inline] fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline] fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    #[inline] fn mul(self, rhs: f64) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    #[inline] fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z }
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline] fn neg(self) -> Self {
        Self { x: -self.x, y: -self.y, z: -self.z }
    }
}

impl AddAssign for Vec3 {
    #[inline] fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x; self.y += rhs.y; self.z += rhs.z;
    }
}

impl SubAssign for Vec3 {
    #[inline] fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x; self.y -= rhs.y; self.z -= rhs.z;
    }
}

impl MulAssign<f64> for Vec3 {
    #[inline] fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs; self.y *= rhs; self.z *= rhs;
    }
}

impl Default for Vec3 {
    fn default() -> Self { Self::ZERO }
}

// ─────────────────────────────── Quaternion ───────────────────────────────

/// Unit quaternion for rotation representation.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Quat {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Quat {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Self {
        let half = angle * 0.5;
        let s = half.sin();
        let c = half.cos();
        let n = axis.normalize();
        Self { x: n.x * s, y: n.y * s, z: n.z * s, w: c }
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if len < 1e-12 { return Self::IDENTITY; }
        let inv = 1.0 / len;
        Self { x: self.x * inv, y: self.y * inv, z: self.z * inv, w: self.w * inv }
    }

    #[inline]
    pub fn conjugate(self) -> Self {
        Self { x: -self.x, y: -self.y, z: -self.z, w: self.w }
    }

    /// Rotate a vector by this quaternion: q * v * q⁻¹
    #[inline]
    pub fn rotate_vec(self, v: Vec3) -> Vec3 {
        let u = Vec3::new(self.x, self.y, self.z);
        let s = self.w;
        u * (2.0 * u.dot(v)) + v * (s * s - u.dot(u)) + u.cross(v) * (2.0 * s)
    }

    /// Hamilton product: self * rhs
    #[inline]
    pub fn mul_quat(self, rhs: Self) -> Self {
        Self {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }

    /// Integrate angular velocity over dt: q' = q + 0.5 * ω * q * dt
    #[inline]
    pub fn integrate(self, angular_velocity: Vec3, dt: f64) -> Self {
        let omega = Quat::new(
            angular_velocity.x * 0.5 * dt,
            angular_velocity.y * 0.5 * dt,
            angular_velocity.z * 0.5 * dt,
            0.0,
        );
        let dq = omega.mul_quat(self);
        Self {
            x: self.x + dq.x,
            y: self.y + dq.y,
            z: self.z + dq.z,
            w: self.w + dq.w,
        }.normalize()
    }

    /// Convert to a 3x3 rotation matrix
    #[inline]
    pub fn to_mat3(self) -> Mat3 {
        let xx = self.x * self.x;
        let yy = self.y * self.y;
        let zz = self.z * self.z;
        let xy = self.x * self.y;
        let xz = self.x * self.z;
        let yz = self.y * self.z;
        let wx = self.w * self.x;
        let wy = self.w * self.y;
        let wz = self.w * self.z;

        Mat3 {
            cols: [
                Vec3::new(1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy)),
                Vec3::new(2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx)),
                Vec3::new(2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy)),
            ]
        }
    }
}

impl Default for Quat {
    fn default() -> Self { Self::IDENTITY }
}

// ─────────────────────────────── Mat3 ───────────────────────────────

/// 3x3 column-major matrix — used for inertia tensors.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat3 {
    pub cols: [Vec3; 3],
}

impl Mat3 {
    pub const ZERO: Self = Self { cols: [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO] };
    pub const IDENTITY: Self = Self {
        cols: [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ]
    };

    #[inline]
    pub fn from_diagonal(d: Vec3) -> Self {
        Self {
            cols: [
                Vec3::new(d.x, 0.0, 0.0),
                Vec3::new(0.0, d.y, 0.0),
                Vec3::new(0.0, 0.0, d.z),
            ]
        }
    }

    #[inline]
    pub fn mul_vec(self, v: Vec3) -> Vec3 {
        self.cols[0] * v.x + self.cols[1] * v.y + self.cols[2] * v.z
    }

    #[inline]
    pub fn transpose(self) -> Self {
        Self {
            cols: [
                Vec3::new(self.cols[0].x, self.cols[1].x, self.cols[2].x),
                Vec3::new(self.cols[0].y, self.cols[1].y, self.cols[2].y),
                Vec3::new(self.cols[0].z, self.cols[1].z, self.cols[2].z),
            ]
        }
    }

    #[inline]
    pub fn mul_mat(self, rhs: Self) -> Self {
        Self {
            cols: [
                self.mul_vec(rhs.cols[0]),
                self.mul_vec(rhs.cols[1]),
                self.mul_vec(rhs.cols[2]),
            ]
        }
    }

    /// Scale all elements
    #[inline]
    pub fn scale(self, s: f64) -> Self {
        Self {
            cols: [self.cols[0] * s, self.cols[1] * s, self.cols[2] * s]
        }
    }

    /// Compute inverse for symmetric positive-definite matrices (inertia tensors)
    pub fn inverse(self) -> Self {
        let a = self.cols[0];
        let b = self.cols[1];
        let c = self.cols[2];
        let det = a.x * (b.y * c.z - b.z * c.y)
                - b.x * (a.y * c.z - a.z * c.y)
                + c.x * (a.y * b.z - a.z * b.y);
        if det.abs() < 1e-14 {
            return Self::IDENTITY;
        }
        let inv_det = 1.0 / det;
        Self {
            cols: [
                Vec3::new(
                    (b.y * c.z - b.z * c.y) * inv_det,
                    (a.z * c.y - a.y * c.z) * inv_det,
                    (a.y * b.z - a.z * b.y) * inv_det,
                ),
                Vec3::new(
                    (b.z * c.x - b.x * c.z) * inv_det,
                    (a.x * c.z - a.z * c.x) * inv_det,
                    (a.z * b.x - a.x * b.z) * inv_det,
                ),
                Vec3::new(
                    (b.x * c.y - b.y * c.x) * inv_det,
                    (a.y * c.x - a.x * c.y) * inv_det,
                    (a.x * b.y - a.y * b.x) * inv_det,
                ),
            ]
        }
    }
}

impl Default for Mat3 {
    fn default() -> Self { Self::IDENTITY }
}

// ─────────────────────────────── AABB ───────────────────────────────

/// Axis-Aligned Bounding Box for broadphase collision detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_center_half(center: Vec3, half_extents: Vec3) -> Self {
        Self { min: center - half_extents, max: center + half_extents }
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Expand AABB by φ-scaled margin for broadphase tolerance
    #[inline]
    pub fn expanded(&self, margin: f64) -> Self {
        let m = Vec3::splat(margin);
        Self { min: self.min - m, max: self.max + m }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec3_cross_product() {
        let a = Vec3::RIGHT;
        let b = Vec3::UP;
        let c = a.cross(b);
        assert!((c.x - 0.0).abs() < 1e-12);
        assert!((c.y - 0.0).abs() < 1e-12);
        assert!((c.z - 1.0).abs() < 1e-12);
    }

    #[test]
    fn quat_rotate_identity() {
        let q = Quat::IDENTITY;
        let v = Vec3::new(1.0, 2.0, 3.0);
        let r = q.rotate_vec(v);
        assert!((r.x - v.x).abs() < 1e-10);
        assert!((r.y - v.y).abs() < 1e-10);
        assert!((r.z - v.z).abs() < 1e-10);
    }

    #[test]
    fn mat3_inverse_identity() {
        let m = Mat3::IDENTITY;
        let inv = m.inverse();
        assert!((inv.cols[0].x - 1.0).abs() < 1e-10);
        assert!((inv.cols[1].y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn aabb_intersection() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
        assert!(a.intersects(&b));
    }
}
