/**
 * @module Three.js Adapter
 *
 * Synchronizes Aether-Net physics bodies with Three.js Object3D instances.
 * Uses bulk position/orientation transfer for maximum performance.
 */

import type { AetherWorld } from '../world.js';
import { PHI } from '../types.js';

/** Minimal Three.js interfaces to avoid hard dependency */
interface ThreeVector3 {
  set(x: number, y: number, z: number): void;
}

interface ThreeQuaternion {
  set(x: number, y: number, z: number, w: number): void;
}

interface ThreeObject3D {
  position: ThreeVector3;
  quaternion: ThreeQuaternion;
  userData: Record<string, unknown>;
}

/** Mapping between a physics body ID and a Three.js object */
export interface BodyBinding {
  bodyId: number;
  object: ThreeObject3D;
}

/**
 * ThreeAdapter — Bridges Aether-Net physics to Three.js rendering.
 *
 * Usage:
 * ```ts
 * const adapter = new ThreeAdapter(world);
 * adapter.bind(ballId, ballMesh);
 * // In render loop:
 * adapter.sync();
 * ```
 */
export class ThreeAdapter {
  private world: AetherWorld;
  private bindings: BodyBinding[] = [];

  constructor(world: AetherWorld) {
    this.world = world;
  }

  /** Bind a physics body to a Three.js Object3D. */
  bind(bodyId: number, object: ThreeObject3D): void {
    object.userData['aetherBodyId'] = bodyId;
    this.bindings.push({ bodyId, object });
  }

  /** Unbind a body. */
  unbind(bodyId: number): void {
    this.bindings = this.bindings.filter(b => b.bodyId !== bodyId);
  }

  /**
   * Sync all bound objects from physics state.
   * Uses bulk transfer for optimal Wasm↔JS performance.
   */
  sync(): void {
    const positions = this.world.getAllPositionsF32();
    const orientations = this.world.getAllOrientationsF32();

    for (const binding of this.bindings) {
      const pi = binding.bodyId * 3;
      const qi = binding.bodyId * 4;

      if (pi + 2 < positions.length) {
        binding.object.position.set(
          positions[pi], positions[pi + 1], positions[pi + 2],
        );
      }
      if (qi + 3 < orientations.length) {
        binding.object.quaternion.set(
          orientations[qi], orientations[qi + 1],
          orientations[qi + 2], orientations[qi + 3],
        );
      }
    }
  }

  /**
   * Sync a single body (for when bulk isn't needed).
   */
  syncOne(bodyId: number): void {
    const binding = this.bindings.find(b => b.bodyId === bodyId);
    if (!binding) return;
    const pos = this.world.getPosition(bodyId);
    const ori = this.world.getOrientation(bodyId);
    binding.object.position.set(pos.x, pos.y, pos.z);
    binding.object.quaternion.set(ori.x, ori.y, ori.z, ori.w);
  }

  /** Get the φ-interpolation factor for fixed-timestep rendering. */
  static interpolationAlpha(accumulator: number, fixedDt: number): number {
    // φ-biased interpolation for visually smooth motion
    const raw = accumulator / fixedDt;
    return Math.pow(raw, 1.0 / PHI);
  }

  /** Number of active bindings. */
  get bindingCount(): number {
    return this.bindings.length;
  }

  /** Dispose all bindings. */
  dispose(): void {
    this.bindings = [];
  }
}
