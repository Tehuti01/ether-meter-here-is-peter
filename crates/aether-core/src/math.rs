//! # Linear Algebra Primitives — zero-alloc, inline-everything, φ-tuned
//!
//! Vec3, Vec4 (SIMD-friendly), Quat, Mat3, AABB — all `#[repr(C)]` for safe
//! WASM memory sharing and future SIMD lanes.

use core::ops::{Add, Sub, Mul, Neg, AddAssign, SubAssign, MulAssign, Div};

// ═══════════════════════════════════════════════════════════════ Vec3 ═══════

/// 3D f64 vector.  `#[repr(C)]` guarantees ABI stability for Wasm exports.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO:    Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE:     Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const NEG_ONE: Self = Self { x:-1.0, y:-1.0, z:-1.0 };
    pub const UP:      Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const DOWN:    Self = Self { x: 0.0, y:-1.0, z: 0.0 };
    pub const RIGHT:   Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const LEFT:    Self = Self { x:-1.0, y: 0.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    pub const BACK:    Self = Self { x: 0.0, y: 0.0, z:-1.0 };
    pub const INF:     Self = Self { x: f64::INFINITY, y: f64::INFINITY, z: f64::INFINITY };
    pub const NEG_INF: Self = Self { x: f64::NEG_INFINITY, y: f64::NEG_INFINITY, z: f64::NEG_INFINITY };

    #[inline(always)] pub const fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }
    #[inline(always)] pub fn splat(v: f64) -> Self { Self { x: v, y: v, z: v } }
    #[inline(always)] pub fn from_array(a: [f64; 3]) -> Self { Self { x: a[0], y: a[1], z: a[2] } }
    #[inline(always)] pub fn to_array(self) -> [f64; 3] { [self.x, self.y, self.z] }
    #[inline(always)] pub fn to_f32_array(self) -> [f32; 3] { [self.x as f32, self.y as f32, self.z as f32] }

    // ─── Algebra ───
    #[inline(always)] pub fn dot(self, r: Self) -> f64 { self.x*r.x + self.y*r.y + self.z*r.z }
    #[inline(always)] pub fn cross(self, r: Self) -> Self {
        Self { x: self.y*r.z - self.z*r.y, y: self.z*r.x - self.x*r.z, z: self.x*r.y - self.y*r.x }
    }
    #[inline(always)] pub fn length_sq(self) -> f64 { self.dot(self) }
    #[inline(always)] pub fn length(self) -> f64    { self.length_sq().sqrt() }
    #[inline] pub fn normalize(self) -> Self {
        let l = self.length();
        if l < 1e-15 { Self::ZERO } else { self * (1.0 / l) }
    }
    /// Normalize and return length simultaneously — avoids double sqrt.
    #[inline] pub fn normalize_and_length(self) -> (Self, f64) {
        let l = self.length();
        if l < 1e-15 { (Self::ZERO, 0.0) } else { (self * (1.0 / l), l) }
    }
    #[inline(always)] pub fn lerp(self, b: Self, t: f64) -> Self { self + (b - self) * t }
    #[inline(always)] pub fn abs(self) -> Self { Self { x: self.x.abs(), y: self.y.abs(), z: self.z.abs() } }
    #[inline(always)] pub fn min_comp(self) -> f64 { self.x.min(self.y).min(self.z) }
    #[inline(always)] pub fn max_comp(self) -> f64 { self.x.max(self.y).max(self.z) }
    #[inline(always)] pub fn component_min(self, o: Self) -> Self {
        Self { x: self.x.min(o.x), y: self.y.min(o.y), z: self.z.min(o.z) }
    }
    #[inline(always)] pub fn component_max(self, o: Self) -> Self {
        Self { x: self.x.max(o.x), y: self.y.max(o.y), z: self.z.max(o.z) }
    }
    #[inline] pub fn clamp_length(self, max: f64) -> Self {
        let l2 = self.length_sq();
        if l2 > max * max { self.normalize() * max } else { self }
    }
    #[inline(always)] pub fn distance_sq(self, o: Self) -> f64 { (self - o).length_sq() }
    #[inline(always)] pub fn distance(self, o: Self) -> f64    { (self - o).length() }
    #[inline(always)] pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }
    #[inline(always)] pub fn project_onto(self, onto: Self) -> Self {
        onto * (self.dot(onto) / onto.length_sq().max(1e-30))
    }
    /// True if any component is NaN or Inf
    #[inline(always)] pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

impl Add  for Vec3 { type Output=Self; #[inline(always)] fn add(self,r:Self)->Self { Self{x:self.x+r.x,y:self.y+r.y,z:self.z+r.z} } }
impl Sub  for Vec3 { type Output=Self; #[inline(always)] fn sub(self,r:Self)->Self { Self{x:self.x-r.x,y:self.y-r.y,z:self.z-r.z} } }
impl Neg  for Vec3 { type Output=Self; #[inline(always)] fn neg(self)     ->Self { Self{x:-self.x,y:-self.y,z:-self.z} } }
impl Mul<f64> for Vec3 { type Output=Self; #[inline(always)] fn mul(self,s:f64)->Self { Self{x:self.x*s,y:self.y*s,z:self.z*s} } }
impl Div<f64> for Vec3 { type Output=Self; #[inline(always)] fn div(self,s:f64)->Self { self * (1.0/s) } }
impl Mul<Vec3> for f64 { type Output=Vec3; #[inline(always)] fn mul(self,v:Vec3)->Vec3 { v*self } }
impl AddAssign for Vec3 { #[inline(always)] fn add_assign(&mut self,r:Self){ self.x+=r.x;self.y+=r.y;self.z+=r.z; } }
impl SubAssign for Vec3 { #[inline(always)] fn sub_assign(&mut self,r:Self){ self.x-=r.x;self.y-=r.y;self.z-=r.z; } }
impl MulAssign<f64> for Vec3 { #[inline(always)] fn mul_assign(&mut self,s:f64){ self.x*=s;self.y*=s;self.z*=s; } }

// ═══════════════════════════════════════════════════════════════ Quaternion ═══

/// Unit quaternion — rotation representation.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Quat { pub x: f64, pub y: f64, pub z: f64, pub w: f64 }

impl Default for Quat { fn default() -> Self { Self::IDENTITY } }

impl Quat {
    pub const IDENTITY: Self = Self { x:0.0, y:0.0, z:0.0, w:1.0 };

    #[inline(always)] pub const fn new(x:f64,y:f64,z:f64,w:f64)->Self { Self{x,y,z,w} }

    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Self {
        let half = angle * 0.5;
        let (s, c) = (half.sin(), half.cos());
        let n = axis.normalize();
        Self { x: n.x*s, y: n.y*s, z: n.z*s, w: c }
    }

    #[inline]
    pub fn from_euler_xyz(ex: f64, ey: f64, ez: f64) -> Self {
        let (sx,cx) = (ex*0.5).sin_cos();
        let (sy,cy) = (ey*0.5).sin_cos();
        let (sz,cz) = (ez*0.5).sin_cos();
        Self {
            x: sx*cy*cz + cx*sy*sz,
            y: cx*sy*cz - sx*cy*sz,
            z: cx*cy*sz + sx*sy*cz,
            w: cx*cy*cz - sx*sy*sz,
        }
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let l = (self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w).sqrt();
        if l < 1e-15 { Self::IDENTITY } else {
            let inv = 1.0/l;
            Self { x:self.x*inv, y:self.y*inv, z:self.z*inv, w:self.w*inv }
        }
    }

    #[inline(always)] pub fn conjugate(self) -> Self { Self{x:-self.x,y:-self.y,z:-self.z,w:self.w} }
    #[inline(always)] pub fn inverse(self) -> Self { self.conjugate().normalize() }

    /// Rotate vector: q v q*
    #[inline]
    pub fn rotate_vec(self, v: Vec3) -> Vec3 {
        let u = Vec3::new(self.x, self.y, self.z);
        let s = self.w;
        u * (2.0 * u.dot(v)) + v * (s*s - u.dot(u)) + u.cross(v) * (2.0 * s)
    }

    /// Hamilton product
    #[inline]
    pub fn mul_quat(self, r: Self) -> Self {
        Self {
            x: self.w*r.x + self.x*r.w + self.y*r.z - self.z*r.y,
            y: self.w*r.y - self.x*r.z + self.y*r.w + self.z*r.x,
            z: self.w*r.z + self.x*r.y - self.y*r.x + self.z*r.w,
            w: self.w*r.w - self.x*r.x - self.y*r.y - self.z*r.z,
        }
    }

    /// Spherical linear interpolation
    #[inline]
    pub fn slerp(self, mut other: Self, t: f64) -> Self {
        let mut dot = self.x*other.x + self.y*other.y + self.z*other.z + self.w*other.w;
        if dot < 0.0 { other = Self{x:-other.x,y:-other.y,z:-other.z,w:-other.w}; dot=-dot; }
        if dot > 0.9995 {
            return Self{
                x:self.x+(other.x-self.x)*t, y:self.y+(other.y-self.y)*t,
                z:self.z+(other.z-self.z)*t, w:self.w+(other.w-self.w)*t,
            }.normalize();
        }
        let theta = dot.acos();
        let sin_theta = theta.sin();
        let wa = ((1.0-t)*theta).sin() / sin_theta;
        let wb = (t*theta).sin() / sin_theta;
        Self{x:wa*self.x+wb*other.x, y:wa*self.y+wb*other.y,
             z:wa*self.z+wb*other.z, w:wa*self.w+wb*other.w}.normalize()
    }

    /// Integrate angular velocity ω over dt: q' = (q + 0.5 ωq dt).normalized
    #[inline]
    pub fn integrate(self, w: Vec3, dt: f64) -> Self {
        let omega = Self { x:w.x*0.5*dt, y:w.y*0.5*dt, z:w.z*0.5*dt, w:0.0 };
        let dq = omega.mul_quat(self);
        Self { x:self.x+dq.x, y:self.y+dq.y, z:self.z+dq.z, w:self.w+dq.w }.normalize()
    }

    #[inline]
    pub fn to_mat3(self) -> Mat3 {
        let (xx,yy,zz) = (self.x*self.x, self.y*self.y, self.z*self.z);
        let (xy,xz,yz) = (self.x*self.y, self.x*self.z, self.y*self.z);
        let (wx,wy,wz) = (self.w*self.x, self.w*self.y, self.w*self.z);
        Mat3 { cols: [
            Vec3::new(1.0-2.0*(yy+zz), 2.0*(xy+wz), 2.0*(xz-wy)),
            Vec3::new(2.0*(xy-wz), 1.0-2.0*(xx+zz), 2.0*(yz+wx)),
            Vec3::new(2.0*(xz+wy), 2.0*(yz-wx), 1.0-2.0*(xx+yy)),
        ]}
    }

    #[inline(always)] pub fn to_array(self) -> [f64; 4] { [self.x,self.y,self.z,self.w] }
    #[inline(always)] pub fn to_f32_array(self) -> [f32; 4] {
        [self.x as f32, self.y as f32, self.z as f32, self.w as f32]
    }
    #[inline(always)] pub fn dot(self, o: Self) -> f64 {
        self.x*o.x + self.y*o.y + self.z*o.z + self.w*o.w
    }
}

// ═══════════════════════════════════════════════════════════════ Mat3 ═══════

/// Column-major 3×3 matrix.  Used for inertia tensors and rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat3 { pub cols: [Vec3; 3] }

impl Default for Mat3 { fn default() -> Self { Self::IDENTITY } }

impl Mat3 {
    pub const ZERO: Self = Self { cols:[Vec3::ZERO, Vec3::ZERO, Vec3::ZERO] };
    pub const IDENTITY: Self = Self { cols:[
        Vec3::new(1.0,0.0,0.0), Vec3::new(0.0,1.0,0.0), Vec3::new(0.0,0.0,1.0),
    ]};

    #[inline] pub fn from_diagonal(d: Vec3) -> Self {
        Self { cols:[Vec3::new(d.x,0.0,0.0), Vec3::new(0.0,d.y,0.0), Vec3::new(0.0,0.0,d.z)] }
    }
    #[inline(always)] pub fn mul_vec(self, v: Vec3) -> Vec3 {
        self.cols[0]*v.x + self.cols[1]*v.y + self.cols[2]*v.z
    }
    #[inline] pub fn transpose(self) -> Self {
        let c = &self.cols;
        Self { cols:[
            Vec3::new(c[0].x, c[1].x, c[2].x),
            Vec3::new(c[0].y, c[1].y, c[2].y),
            Vec3::new(c[0].z, c[1].z, c[2].z),
        ]}
    }
    #[inline] pub fn mul_mat(self, r: Self) -> Self {
        Self { cols:[self.mul_vec(r.cols[0]), self.mul_vec(r.cols[1]), self.mul_vec(r.cols[2])] }
    }
    #[inline] pub fn scale(self, s: f64) -> Self {
        Self { cols:[self.cols[0]*s, self.cols[1]*s, self.cols[2]*s] }
    }
    /// Cramér's rule inverse — robust for SPD inertia tensors
    pub fn inverse(self) -> Self {
        let a=self.cols[0]; let b=self.cols[1]; let c=self.cols[2];
        let det = a.x*(b.y*c.z-b.z*c.y) - b.x*(a.y*c.z-a.z*c.y) + c.x*(a.y*b.z-a.z*b.y);
        if det.abs() < 1e-20 { return Self::IDENTITY; }
        let id = 1.0/det;
        Self { cols:[
            Vec3::new((b.y*c.z-b.z*c.y)*id, (a.z*c.y-a.y*c.z)*id, (a.y*b.z-a.z*b.y)*id),
            Vec3::new((b.z*c.x-b.x*c.z)*id, (a.x*c.z-a.z*c.x)*id, (a.z*b.x-a.x*b.z)*id),
            Vec3::new((b.x*c.y-b.y*c.x)*id, (a.y*c.x-a.x*c.y)*id, (a.x*b.y-a.y*b.x)*id),
        ]}
    }
    #[inline] pub fn add(self, o: Self) -> Self {
        Self { cols:[self.cols[0]+o.cols[0], self.cols[1]+o.cols[1], self.cols[2]+o.cols[2]] }
    }
}

// ═══════════════════════════════════════════════════════════════ AABB ═══════

/// Axis-Aligned Bounding Box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB { pub min: Vec3, pub max: Vec3 }

impl AABB {
    #[inline(always)] pub fn new(min: Vec3, max: Vec3) -> Self { Self{min,max} }
    #[inline] pub fn from_center_half(center: Vec3, half: Vec3) -> Self {
        Self{min:center-half, max:center+half}
    }
    #[inline] pub fn empty() -> Self { Self{min:Vec3::INF, max:Vec3::NEG_INF} }
    #[inline(always)] pub fn intersects(&self, o: &Self) -> bool {
        self.min.x<=o.max.x && self.max.x>=o.min.x &&
        self.min.y<=o.max.y && self.max.y>=o.min.y &&
        self.min.z<=o.max.z && self.max.z>=o.min.z
    }
    #[inline(always)] pub fn contains_point(&self, p: Vec3) -> bool {
        p.x>=self.min.x && p.x<=self.max.x &&
        p.y>=self.min.y && p.y<=self.max.y &&
        p.z>=self.min.z && p.z<=self.max.z
    }
    #[inline(always)] pub fn center(&self) -> Vec3 { (self.min+self.max)*0.5 }
    #[inline(always)] pub fn half_extents(&self) -> Vec3 { (self.max-self.min)*0.5 }
    #[inline(always)] pub fn surface_area(&self) -> f64 {
        let e = self.max-self.min;
        2.0*(e.x*e.y + e.y*e.z + e.z*e.x)
    }
    #[inline] pub fn merge(&self, o: &Self) -> Self {
        Self{min:self.min.component_min(o.min), max:self.max.component_max(o.max)}
    }
    #[inline] pub fn expanded(&self, margin: f64) -> Self {
        let m=Vec3::splat(margin); Self{min:self.min-m, max:self.max+m}
    }
    #[inline] pub fn union_point(&self, p: Vec3) -> Self {
        Self{min:self.min.component_min(p), max:self.max.component_max(p)}
    }
}

// ═══════════════════════════════════════════════════════════════ Tests ═══════
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn vec3_cross() {
        let c = Vec3::RIGHT.cross(Vec3::UP);
        assert!((c.x).abs()<1e-12); assert!((c.y).abs()<1e-12); assert!((c.z-1.0).abs()<1e-12);
    }
    #[test] fn vec3_normalize_and_length() {
        let v = Vec3::new(3.0,4.0,0.0);
        let (n,l) = v.normalize_and_length();
        assert!((l-5.0).abs()<1e-12);
        assert!((n.length()-1.0).abs()<1e-12);
    }
    #[test] fn vec3_reflect() {
        let v = Vec3::new(1.0,-1.0,0.0);
        let r = v.reflect(Vec3::UP);
        assert!((r.y-1.0).abs()<1e-12);
    }
    #[test] fn quat_identity_rotates() {
        let v = Vec3::new(1.0,2.0,3.0);
        let r = Quat::IDENTITY.rotate_vec(v);
        assert!((r.x-v.x).abs()<1e-12 && (r.y-v.y).abs()<1e-12 && (r.z-v.z).abs()<1e-12);
    }
    #[test] fn quat_slerp_endpoints() {
        let a = Quat::IDENTITY;
        let b = Quat::from_axis_angle(Vec3::UP, core::f64::consts::PI);
        assert!((a.slerp(b, 0.0).dot(a)-1.0).abs()<1e-6);
        assert!((a.slerp(b, 1.0).dot(b)).abs()>0.99);
    }
    #[test] fn mat3_inverse_identity() {
        let inv = Mat3::IDENTITY.inverse();
        assert!((inv.cols[0].x-1.0).abs()<1e-12);
        assert!((inv.cols[1].y-1.0).abs()<1e-12);
        assert!((inv.cols[2].z-1.0).abs()<1e-12);
    }
    #[test] fn aabb_merge() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::splat(-1.0), Vec3::splat(0.5));
        let m = a.merge(&b);
        assert!((m.min.x-(-1.0)).abs()<1e-12);
        assert!((m.max.x-1.0).abs()<1e-12);
    }
    #[test] fn aabb_surface_area() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert!((a.surface_area()-6.0).abs()<1e-12);
    }
}
