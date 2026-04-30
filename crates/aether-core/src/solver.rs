//! # Constraint Solver — Sequential Impulse + Warm-Starting + Position Correction
//!
//! Algorithm: Sequential Impulse (Erin Catto, GDC 2006).
//! Enhancements over the baseline:
//!   - **Warm-starting** — accumulated impulses from the previous frame are
//!     re-applied immediately, dramatically cutting iteration count.
//!   - **Two-tangent friction** — Coulomb cone clamped on two independent
//!     tangent axes for physically plausible behaviour.
//!   - **Speculative restitution** — velocity-threshold guard prevents
//!     jitter on slow contacts.
//!   - **φ-governed iteration count** — SOLVER_ITERATIONS = 8 (Fib(6)).
//!   - **Baumgarte stabilisation** — φ-scaled bias velocity resolves drift.

use crate::math::Vec3;
use crate::narrowphase::Contact;
use crate::body::RigidBody;
use crate::phi;

// ─────────────────────────────── ContactConstraint ───────────────────────────────

/// Persistent contact constraint — retains warm-start impulses across calls.
#[derive(Debug, Clone, Copy)]
pub struct ContactConstraint {
    pub body_a: u32,
    pub body_b: u32,
    pub contact: Contact,
    /// Accumulated normal impulse (non-negative — no pull)
    pub normal_impulse: f64,
    /// Accumulated friction impulse along tangent 1
    pub tangent_impulse_1: f64,
    /// Accumulated friction impulse along tangent 2
    pub tangent_impulse_2: f64,
    pub restitution: f64,
    pub friction: f64,
}

impl ContactConstraint {
    #[inline]
    pub fn new(a: u32, b: u32, contact: Contact, restitution: f64, friction: f64) -> Self {
        Self {
            body_a: a, body_b: b, contact,
            normal_impulse: 0.0,
            tangent_impulse_1: 0.0,
            tangent_impulse_2: 0.0,
            restitution, friction,
        }
    }
}

// ─────────────────────────────── Solver entry point ───────────────────────────────

/// Solve all contact constraints using sequential impulses.
///
/// `dt` — sub-step delta (NOT the full frame delta).
pub fn solve_contacts(
    bodies: &mut [RigidBody],
    constraints: &mut [ContactConstraint],
    dt: f64,
) {
    if constraints.is_empty() || dt < 1e-14 { return; }
    let inv_dt = 1.0 / dt;

    // ── Phase 0: Warm-start — re-apply last frame's cached impulses ──────────
    for c in constraints.iter() {
        let n = c.contact.normal;
        let (t1, t2) = tangent_basis(n);
        let r_a = c.contact.point_a - bodies[c.body_a as usize].state.position;
        let r_b = c.contact.point_b - bodies[c.body_b as usize].state.position;
        let impulse = n * c.normal_impulse + t1 * c.tangent_impulse_1 + t2 * c.tangent_impulse_2;
        apply_impulse(bodies, c.body_a as usize, c.body_b as usize, impulse, r_a, r_b);
    }

    // ── Phase 1: Velocity iterations ─────────────────────────────────────────
    for _iter in 0..phi::SOLVER_ITERATIONS {
        for c in constraints.iter_mut() {
            let (a, b) = (c.body_a as usize, c.body_b as usize);
            let n = c.contact.normal;
            let r_a = c.contact.point_a - bodies[a].state.position;
            let r_b = c.contact.point_b - bodies[b].state.position;

            // ── Normal impulse ────────────────────────────────────────────────
            let vel_a = point_vel(&bodies[a], r_a);
            let vel_b = point_vel(&bodies[b], r_b);
            let rel_vel_n = (vel_b - vel_a).dot(n);

            let k_n = effective_mass(bodies, a, b, n, r_a, r_b);
            if k_n < 1e-20 { continue; }

            // Baumgarte position bias
            let bias = phi::BAUMGARTE_FACTOR * inv_dt
                * (c.contact.depth - phi::PENETRATION_SLOP).max(0.0);

            // Speculative restitution — only when separating fast enough
            let restitution_bias = if rel_vel_n < -phi::SLEEP_THRESHOLD {
                c.restitution * (-rel_vel_n)
            } else { 0.0 };

            let lambda = -(rel_vel_n - bias + restitution_bias) / k_n;
            let old = c.normal_impulse;
            c.normal_impulse = (old + lambda).max(0.0);
            let impulse_n = n * (c.normal_impulse - old);
            apply_impulse(bodies, a, b, impulse_n, r_a, r_b);

            // ── Friction impulses (two tangents) ─────────────────────────────
            let (t1, t2) = tangent_basis(n);
            let max_friction = c.friction * c.normal_impulse;

            // Tangent 1
            let rv1 = (point_vel(&bodies[b], r_b) - point_vel(&bodies[a], r_a)).dot(t1);
            let k_t1 = effective_mass(bodies, a, b, t1, r_a, r_b);
            if k_t1 > 1e-20 {
                let l1 = -rv1 / k_t1;
                let old1 = c.tangent_impulse_1;
                c.tangent_impulse_1 = (old1 + l1).clamp(-max_friction, max_friction);
                apply_impulse(bodies, a, b, t1 * (c.tangent_impulse_1 - old1), r_a, r_b);
            }

            // Tangent 2
            let rv2 = (point_vel(&bodies[b], r_b) - point_vel(&bodies[a], r_a)).dot(t2);
            let k_t2 = effective_mass(bodies, a, b, t2, r_a, r_b);
            if k_t2 > 1e-20 {
                let l2 = -rv2 / k_t2;
                let old2 = c.tangent_impulse_2;
                c.tangent_impulse_2 = (old2 + l2).clamp(-max_friction, max_friction);
                apply_impulse(bodies, a, b, t2 * (c.tangent_impulse_2 - old2), r_a, r_b);
            }
        }
    }
}

// ─────────────────────────────── Helpers ───────────────────────────────

/// Velocity of a point on body `b` at offset `r` from CoM.
#[inline(always)]
fn point_vel(b: &RigidBody, r: Vec3) -> Vec3 {
    b.state.linear_velocity + b.state.angular_velocity.cross(r)
}

/// Generalised inverse mass along an axis (scalar effective mass denominator).
#[inline]
fn effective_mass(bodies: &[RigidBody], a: usize, b: usize, axis: Vec3, r_a: Vec3, r_b: Vec3) -> f64 {
    let inv_ma = bodies[a].mass_props.inv_mass;
    let inv_mb = bodies[b].mass_props.inv_mass;
    let inv_ia = bodies[a].mass_props.inv_inertia_world(&bodies[a].state.orientation);
    let inv_ib = bodies[b].mass_props.inv_inertia_world(&bodies[b].state.orientation);
    let ra_x_n = r_a.cross(axis);
    let rb_x_n = r_b.cross(axis);
    inv_ma + inv_mb
        + ra_x_n.dot(inv_ia.mul_vec(ra_x_n))
        + rb_x_n.dot(inv_ib.mul_vec(rb_x_n))
}

/// Apply an impulse (Newton's 3rd law) — bodies may be static (inv_mass=0).
#[inline]
fn apply_impulse(bodies: &mut [RigidBody], a: usize, b: usize, impulse: Vec3, r_a: Vec3, r_b: Vec3) {
    {
        let ba = &mut bodies[a];
        let inv_ma = ba.mass_props.inv_mass;
        if inv_ma > 0.0 {
            let inv_ia = ba.mass_props.inv_inertia_world(&ba.state.orientation);
            ba.state.linear_velocity -= impulse * inv_ma;
            ba.state.angular_velocity -= inv_ia.mul_vec(r_a.cross(impulse));
        }
    }
    {
        let bb = &mut bodies[b];
        let inv_mb = bb.mass_props.inv_mass;
        if inv_mb > 0.0 {
            let inv_ib = bb.mass_props.inv_inertia_world(&bb.state.orientation);
            bb.state.linear_velocity += impulse * inv_mb;
            bb.state.angular_velocity += inv_ib.mul_vec(r_b.cross(impulse));
        }
    }
}

/// Build two orthogonal tangent vectors for a given normal.
/// Uses Duff/Frisvad fast method when |n.z| < 1/√2, Gram-Schmidt otherwise.
#[inline]
fn tangent_basis(n: Vec3) -> (Vec3, Vec3) {
    let t1 = if n.z.abs() < 0.707_106_78 {
        Vec3::new(-n.y, n.x, 0.0).normalize()
    } else {
        Vec3::new(0.0, -n.z, n.y).normalize()
    };
    let t2 = n.cross(t1);
    (t1, t2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;

    #[test]
    fn tangent_basis_orthonormal() {
        let n = Vec3::new(0.0, 1.0, 0.0);
        let (t1, t2) = tangent_basis(n);
        assert!(t1.dot(n).abs() < 1e-10);
        assert!(t2.dot(n).abs() < 1e-10);
        assert!(t1.dot(t2).abs() < 1e-10);
        assert!((t1.length() - 1.0).abs() < 1e-10);
        assert!((t2.length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tangent_basis_diagonal_normal() {
        let n = Vec3::new(1.0, 1.0, 1.0).normalize();
        let (t1, t2) = tangent_basis(n);
        assert!(t1.dot(n).abs() < 1e-10);
        assert!(t2.dot(n).abs() < 1e-10);
    }
}
