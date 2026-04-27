//! # Constraint Solver
//!
//! Sequential impulse constraint solver with φ-governed iteration count,
//! Baumgarte stabilization, and warm-starting.

use crate::math::Vec3;
use crate::narrowphase::Contact;
use crate::body::RigidBody;
use crate::phi;

/// Cached impulse for warm-starting across frames.
#[derive(Debug, Clone, Copy)]
pub struct ContactConstraint {
    pub body_a: u32,
    pub body_b: u32,
    pub contact: Contact,
    pub normal_impulse: f64,
    pub tangent_impulse_1: f64,
    pub tangent_impulse_2: f64,
    pub restitution: f64,
    pub friction: f64,
}

impl ContactConstraint {
    pub fn new(body_a: u32, body_b: u32, contact: Contact, restitution: f64, friction: f64) -> Self {
        Self {
            body_a, body_b, contact,
            normal_impulse: 0.0,
            tangent_impulse_1: 0.0,
            tangent_impulse_2: 0.0,
            restitution,
            friction,
        }
    }
}

/// Solve all contact constraints using sequential impulses.
///
/// Iteration count is φ-governed (SOLVER_ITERATIONS = 8, a Fibonacci number).
/// Baumgarte factor is φ-scaled for optimal stabilization.
pub fn solve_contacts(bodies: &mut [RigidBody], constraints: &mut [ContactConstraint], dt: f64) {
    if constraints.is_empty() || dt < 1e-12 { return; }
    let inv_dt = 1.0 / dt;

    for _iter in 0..phi::SOLVER_ITERATIONS {
        for constraint in constraints.iter_mut() {
            let (a_idx, b_idx) = (constraint.body_a as usize, constraint.body_b as usize);
            let contact = &constraint.contact;
            let n = contact.normal;

            // Get body properties
            let inv_mass_a = bodies[a_idx].mass_props.inv_mass;
            let inv_mass_b = bodies[b_idx].mass_props.inv_mass;
            let inv_i_a = bodies[a_idx].mass_props.inv_inertia_world(&bodies[a_idx].state.orientation);
            let inv_i_b = bodies[b_idx].mass_props.inv_inertia_world(&bodies[b_idx].state.orientation);

            let r_a = contact.point_a - bodies[a_idx].state.position;
            let r_b = contact.point_b - bodies[b_idx].state.position;

            // ─── Normal impulse ───
            let vel_a = bodies[a_idx].state.linear_velocity + bodies[a_idx].state.angular_velocity.cross(r_a);
            let vel_b = bodies[b_idx].state.linear_velocity + bodies[b_idx].state.angular_velocity.cross(r_b);
            let rel_vel = vel_b - vel_a;
            let normal_vel = rel_vel.dot(n);

            // Effective mass along normal
            let rn_a = r_a.cross(n);
            let rn_b = r_b.cross(n);
            let k_normal = inv_mass_a + inv_mass_b
                + rn_a.dot(inv_i_a.mul_vec(rn_a))
                + rn_b.dot(inv_i_b.mul_vec(rn_b));

            if k_normal < 1e-14 { continue; }

            // Baumgarte stabilization — bias velocity to resolve penetration
            let bias = phi::BAUMGARTE_FACTOR * inv_dt
                * (contact.depth - phi::PENETRATION_SLOP).max(0.0);

            // Restitution
            let restitution_bias = if normal_vel < -1.0 {
                -constraint.restitution * normal_vel
            } else { 0.0 };

            let lambda = -(normal_vel + bias + restitution_bias) / k_normal;
            let old_impulse = constraint.normal_impulse;
            constraint.normal_impulse = (old_impulse + lambda).max(0.0);
            let impulse_n = n * (constraint.normal_impulse - old_impulse);

            // Apply normal impulse
            apply_impulse(bodies, a_idx, b_idx, impulse_n, r_a, r_b);

            // ─── Friction impulse ───
            // Recompute relative velocity after normal impulse
            let vel_a2 = bodies[a_idx].state.linear_velocity + bodies[a_idx].state.angular_velocity.cross(r_a);
            let vel_b2 = bodies[b_idx].state.linear_velocity + bodies[b_idx].state.angular_velocity.cross(r_b);
            let rel_vel2 = vel_b2 - vel_a2;
            let tangent_vel = rel_vel2 - n * rel_vel2.dot(n);
            let tangent_speed = tangent_vel.length();
            if tangent_speed > 1e-10 {
                let t = tangent_vel * (1.0 / tangent_speed);
                let rt_a = r_a.cross(t);
                let rt_b = r_b.cross(t);
                let k_tangent = inv_mass_a + inv_mass_b
                    + rt_a.dot(inv_i_a.mul_vec(rt_a))
                    + rt_b.dot(inv_i_b.mul_vec(rt_b));

                if k_tangent > 1e-14 {
                    let lambda_t = -tangent_speed / k_tangent;
                    let max_friction = constraint.friction * constraint.normal_impulse;
                    let clamped = lambda_t.clamp(-max_friction, max_friction);
                    let impulse_t = t * clamped;
                    apply_impulse(bodies, a_idx, b_idx, impulse_t, r_a, r_b);
                }
            }
        }
    }
}

#[inline]
fn apply_impulse(bodies: &mut [RigidBody], a: usize, b: usize, impulse: Vec3, r_a: Vec3, r_b: Vec3) {
    let inv_mass_a = bodies[a].mass_props.inv_mass;
    let inv_mass_b = bodies[b].mass_props.inv_mass;
    let inv_i_a = bodies[a].mass_props.inv_inertia_world(&bodies[a].state.orientation);
    let inv_i_b = bodies[b].mass_props.inv_inertia_world(&bodies[b].state.orientation);

    bodies[a].state.linear_velocity -= impulse * inv_mass_a;
    bodies[a].state.angular_velocity -= inv_i_a.mul_vec(r_a.cross(impulse));
    bodies[b].state.linear_velocity += impulse * inv_mass_b;
    bodies[b].state.angular_velocity += inv_i_b.mul_vec(r_b.cross(impulse));
}
