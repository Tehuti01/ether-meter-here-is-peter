//! # Narrow-Phase Collision Detection & Contact Generation
//!
//! Sphere-Sphere, Sphere-Cuboid, Cuboid-Cuboid (SAT), and Sphere-Plane
//! collision detection with contact point and normal generation.

use crate::math::{Vec3, Quat};
use crate::collider::Shape;
use crate::phi;

/// A contact point between two bodies.
#[derive(Debug, Clone, Copy)]
pub struct Contact {
    /// World-space contact point on body A
    pub point_a: Vec3,
    /// World-space contact point on body B
    pub point_b: Vec3,
    /// Contact normal (from A toward B)
    pub normal: Vec3,
    /// Penetration depth (positive = overlapping)
    pub depth: f64,
}

/// Result of narrow-phase collision test.
#[derive(Debug, Clone)]
pub struct ContactManifold {
    pub contacts: Vec<Contact>,
}

impl ContactManifold {
    pub fn empty() -> Self { Self { contacts: Vec::new() } }
    pub fn single(c: Contact) -> Self { Self { contacts: vec![c] } }
    pub fn has_contacts(&self) -> bool { !self.contacts.is_empty() }
}

/// Perform narrow-phase collision between two shapes at given transforms.
pub fn collide(
    shape_a: &Shape, pos_a: Vec3, rot_a: Quat,
    shape_b: &Shape, pos_b: Vec3, rot_b: Quat,
) -> ContactManifold {
    match (shape_a, shape_b) {
        (Shape::Sphere { radius: ra }, Shape::Sphere { radius: rb }) => {
            sphere_sphere(pos_a, *ra, pos_b, *rb)
        }
        (Shape::Sphere { radius }, Shape::Plane { normal, offset }) => {
            sphere_plane(pos_a, *radius, *normal, *offset)
        }
        (Shape::Plane { normal, offset }, Shape::Sphere { radius }) => {
            let mut m = sphere_plane(pos_b, *radius, *normal, *offset);
            for c in &mut m.contacts { c.normal = -c.normal; std::mem::swap(&mut c.point_a, &mut c.point_b); }
            m
        }
        (Shape::Sphere { radius }, Shape::Cuboid { half_extents }) => {
            sphere_cuboid(pos_a, *radius, pos_b, rot_b, *half_extents)
        }
        (Shape::Cuboid { half_extents }, Shape::Sphere { radius }) => {
            let mut m = sphere_cuboid(pos_b, *radius, pos_a, rot_a, *half_extents);
            for c in &mut m.contacts { c.normal = -c.normal; std::mem::swap(&mut c.point_a, &mut c.point_b); }
            m
        }
        (Shape::Cuboid { half_extents: ha }, Shape::Cuboid { half_extents: hb }) => {
            cuboid_cuboid(pos_a, rot_a, *ha, pos_b, rot_b, *hb)
        }
        (Shape::Sphere { radius }, Shape::Capsule { half_height, radius: cr }) => {
            sphere_capsule(pos_a, *radius, pos_b, rot_b, *half_height, *cr)
        }
        (Shape::Capsule { half_height, radius: cr }, Shape::Sphere { radius }) => {
            let mut m = sphere_capsule(pos_b, *radius, pos_a, rot_a, *half_height, *cr);
            for c in &mut m.contacts { c.normal = -c.normal; std::mem::swap(&mut c.point_a, &mut c.point_b); }
            m
        }
        _ => ContactManifold::empty(), // TODO: remaining pairs
    }
}

fn sphere_sphere(pos_a: Vec3, ra: f64, pos_b: Vec3, rb: f64) -> ContactManifold {
    let diff = pos_b - pos_a;
    let dist_sq = diff.length_squared();
    let sum_r = ra + rb;
    if dist_sq > sum_r * sum_r || dist_sq < 1e-14 {
        return ContactManifold::empty();
    }
    let dist = dist_sq.sqrt();
    let normal = diff * (1.0 / dist);
    let depth = sum_r - dist;
    ContactManifold::single(Contact {
        point_a: pos_a + normal * ra,
        point_b: pos_b - normal * rb,
        normal,
        depth,
    })
}

fn sphere_plane(sphere_pos: Vec3, radius: f64, plane_normal: Vec3, plane_offset: f64) -> ContactManifold {
    let n = plane_normal.normalize();
    let dist = sphere_pos.dot(n) - plane_offset;
    if dist > radius + phi::PENETRATION_SLOP {
        return ContactManifold::empty();
    }
    let depth = radius - dist;
    ContactManifold::single(Contact {
        point_a: sphere_pos - n * radius,
        point_b: sphere_pos - n * dist,
        normal: n,
        depth,
    })
}

fn sphere_cuboid(
    sphere_pos: Vec3, radius: f64,
    box_pos: Vec3, box_rot: Quat, half: Vec3,
) -> ContactManifold {
    // Transform sphere center to box-local space
    let local_center = box_rot.conjugate().rotate_vec(sphere_pos - box_pos);
    // Clamp to box surface
    let closest = Vec3::new(
        local_center.x.clamp(-half.x, half.x),
        local_center.y.clamp(-half.y, half.y),
        local_center.z.clamp(-half.z, half.z),
    );
    let diff = local_center - closest;
    let dist_sq = diff.length_squared();
    if dist_sq > radius * radius {
        return ContactManifold::empty();
    }
    let dist = dist_sq.sqrt().max(1e-12);
    let local_normal = diff * (1.0 / dist);
    let normal = box_rot.rotate_vec(local_normal);
    let depth = radius - dist;
    let point_b = box_pos + box_rot.rotate_vec(closest);
    let point_a = sphere_pos - normal * radius;
    ContactManifold::single(Contact { point_a, point_b, normal, depth })
}

fn sphere_capsule(
    sphere_pos: Vec3, sr: f64,
    cap_pos: Vec3, cap_rot: Quat, half_h: f64, cr: f64,
) -> ContactManifold {
    let local_sphere = cap_rot.conjugate().rotate_vec(sphere_pos - cap_pos);
    let clamped_y = local_sphere.y.clamp(-half_h, half_h);
    let closest = Vec3::new(0.0, clamped_y, 0.0);
    let diff = local_sphere - closest;
    let dist = diff.length();
    let sum_r = sr + cr;
    if dist > sum_r {
        return ContactManifold::empty();
    }
    let normal_local = if dist > 1e-12 { diff * (1.0 / dist) } else { Vec3::UP };
    let normal = cap_rot.rotate_vec(normal_local);
    let depth = sum_r - dist;
    ContactManifold::single(Contact {
        point_a: sphere_pos - normal * sr,
        point_b: cap_pos + cap_rot.rotate_vec(closest + normal_local * cr),
        normal,
        depth,
    })
}

fn cuboid_cuboid(
    pos_a: Vec3, rot_a: Quat, ha: Vec3,
    pos_b: Vec3, rot_b: Quat, hb: Vec3,
) -> ContactManifold {
    // SAT test with 15 axes (3 face normals each + 9 edge crosses)
    let axes_a = [
        rot_a.rotate_vec(Vec3::RIGHT),
        rot_a.rotate_vec(Vec3::UP),
        rot_a.rotate_vec(Vec3::FORWARD),
    ];
    let axes_b = [
        rot_b.rotate_vec(Vec3::RIGHT),
        rot_b.rotate_vec(Vec3::UP),
        rot_b.rotate_vec(Vec3::FORWARD),
    ];
    let ha_arr = [ha.x, ha.y, ha.z];
    let hb_arr = [hb.x, hb.y, hb.z];
    let d = pos_b - pos_a;

    let mut min_depth = f64::MAX;
    let mut min_axis = Vec3::UP;

    // Test face normals of A
    for i in 0..3 {
        let axis = axes_a[i];
        let (depth, sep) = sat_overlap(&axis, &d, &axes_a, &ha_arr, &axes_b, &hb_arr);
        if depth < 0.0 { return ContactManifold::empty(); }
        if depth < min_depth { min_depth = depth; min_axis = if sep { -axis } else { axis }; }
    }
    // Test face normals of B
    for i in 0..3 {
        let axis = axes_b[i];
        let (depth, sep) = sat_overlap(&axis, &d, &axes_a, &ha_arr, &axes_b, &hb_arr);
        if depth < 0.0 { return ContactManifold::empty(); }
        if depth < min_depth { min_depth = depth; min_axis = if sep { -axis } else { axis }; }
    }
    // Test edge cross products
    for i in 0..3 {
        for j in 0..3 {
            let axis = axes_a[i].cross(axes_b[j]);
            if axis.length_squared() < 1e-10 { continue; }
            let axis = axis.normalize();
            let (depth, sep) = sat_overlap(&axis, &d, &axes_a, &ha_arr, &axes_b, &hb_arr);
            if depth < 0.0 { return ContactManifold::empty(); }
            if depth < min_depth { min_depth = depth; min_axis = if sep { -axis } else { axis }; }
        }
    }

    // Generate contact point at midpoint along minimum axis
    let point = pos_a + d * 0.5;
    ContactManifold::single(Contact {
        point_a: point - min_axis * (min_depth * 0.5),
        point_b: point + min_axis * (min_depth * 0.5),
        normal: min_axis,
        depth: min_depth,
    })
}

fn sat_overlap(
    axis: &Vec3, d: &Vec3,
    axes_a: &[Vec3; 3], ha: &[f64; 3],
    axes_b: &[Vec3; 3], hb: &[f64; 3],
) -> (f64, bool) {
    let proj_d = d.dot(*axis);
    let mut ra = 0.0;
    for i in 0..3 { ra += ha[i] * axes_a[i].dot(*axis).abs(); }
    let mut rb = 0.0;
    for i in 0..3 { rb += hb[i] * axes_b[i].dot(*axis).abs(); }
    let depth = ra + rb - proj_d.abs();
    (depth, proj_d < 0.0)
}
