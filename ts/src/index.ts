/**
 * @module @aether-net/core
 *
 * Public API surface for the Aether-Net TypeScript SDK.
 */

// Core types
export type {
  Vec3, Quat, BodyType, ShapeDescriptor, BodyOptions,
  WorldConfig, BodyState, StepMetrics,
} from './types.js';

// Constants
export { PHI, PHI_INV, PHI_SQ } from './types.js';

// World
export { AetherWorld } from './world.js';

// Adapters
export { ThreeAdapter } from './adapters/three.js';
export type { BodyBinding } from './adapters/three.js';
