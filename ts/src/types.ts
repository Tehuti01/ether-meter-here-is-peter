/**
 * @module @aether-net/core
 *
 * TypeScript types for the Aether-Net physics engine.
 * These types mirror the Rust structures exposed via Wasm.
 */

/** The golden ratio — governs every constant in Aether-Net */
export const PHI = 1.618033988749895;
export const PHI_INV = 0.618033988749895;
export const PHI_SQ = 2.618033988749895;

/** 3D vector */
export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

/** Quaternion (x, y, z, w) */
export interface Quat {
  x: number;
  y: number;
  z: number;
  w: number;
}

/** Body type classification */
export type BodyType = 'dynamic' | 'kinematic' | 'static';

/** Shape definition for creating colliders */
export type ShapeDescriptor =
  | { type: 'sphere'; radius: number }
  | { type: 'cuboid'; halfExtents: Vec3 }
  | { type: 'capsule'; halfHeight: number; radius: number }
  | { type: 'plane'; normal: Vec3; offset: number };

/** Body creation options */
export interface BodyOptions {
  position?: Vec3;
  velocity?: Vec3;
  mass?: number;
  restitution?: number;
  friction?: number;
  gravityEnabled?: boolean;
  userData?: unknown;
}

/** World configuration */
export interface WorldConfig {
  gravity?: Vec3;
  substeps?: number;
  solverIterations?: number;
  neuralPhysics?: boolean;
}

/** Body state snapshot for rendering */
export interface BodyState {
  id: number;
  position: Vec3;
  orientation: Quat;
  velocity: Vec3;
  sleeping: boolean;
}

/** Performance metrics */
export interface StepMetrics {
  stepTimeMs: number;
  bodyCount: number;
  activeBodyCount: number;
  constraintCount: number;
  frame: number;
}
