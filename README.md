<div align="center">

# Aether-Net

### AI-Driven Decentralized Real-Time Physics Engine

[![CI](https://github.com/Tehuti01/ether-meter-here-is-peter/actions/workflows/ci.yml/badge.svg)](https://github.com/Tehuti01/ether-meter-here-is-peter/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-gold.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-teal.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-Ready-purple.svg)](https://webassembly.org)

---

A high-performance rigid-body physics engine built in Rust, compiled to WebAssembly, and wrapped in a TypeScript SDK for seamless browser integration. Every physical constant in the engine is derived from the golden ratio (φ = 1.618...) for mathematically harmonious simulation behaviour.

</div>

---

## Overview

Aether-Net is a modular physics engine designed for real-time spatial computing in the browser. It provides:

- **Zero-allocation Rust core** — no garbage collection pauses, deterministic frame times
- **WebAssembly compilation** — near-native performance in every modern browser
- **Float32 bulk transfer** — GPU-ready data arrays for direct Three.js / Babylon.js instancing
- **Sequential impulse solver** — warm-started constraint resolution with two-tangent Coulomb friction
- **Golden ratio governance** — all damping, convergence, and threshold constants derive from φ

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        TypeScript SDK                               │
│   AetherWorld · ShapeDescriptor · ThreeAdapter · BodyState          │
├─────────────────────────────────────────────────────────────────────┤
│                     wasm-bindgen Bridge                             │
│   AetherWorld (JS) ←→ Float32Array bulk transfer ←→ Rust World     │
├─────────────────────────────────────────────────────────────────────┤
│                        Rust Physics Core                            │
│                                                                     │
│   ┌──────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │
│   │  phi.rs   │  │   math.rs    │  │ collider.rs  │  │ body.rs  │  │
│   │  Golden   │  │  Vec3, Quat  │  │ Sphere, Box  │  │ Dynamics │  │
│   │  Ratio    │  │  Mat3, AABB  │  │ Capsule,Plane│  │ Forces   │  │
│   └──────────┘  └──────────────┘  └──────────────┘  └──────────┘  │
│                                                                     │
│   ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐  │
│   │broadphase.rs │  │narrowphase.rs│  │      solver.rs         │  │
│   │ Spatial Hash  │  │ SAT / GJK   │  │ Sequential Impulse     │  │
│   │ Grid          │  │ Contact Gen │  │ Warm-Start + Friction  │  │
│   └──────────────┘  └──────────────┘  └────────────────────────┘  │
│                                                                     │
│   ┌────────────────────────────────────────────────────────────┐   │
│   │                       world.rs                             │   │
│   │   Sub-stepping · Broadphase · Sleep Detection · Events     │   │
│   └────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Golden Ratio Constants

Every fundamental constant derives from φ = (1 + √5) / 2:

| Constant | Derivation | Value | Purpose |
|---|---|---|---|
| `LINEAR_DAMPING` | 1 − φ⁻⁴ | 0.854 | Velocity decay per second |
| `ANGULAR_DAMPING` | 1 − φ⁻³ | 0.764 | Rotational decay |
| `SOLVER_EPSILON` | φ⁻⁸ | 0.0213 | Convergence threshold |
| `BAUMGARTE_FACTOR` | 0.2 × φ⁻³ | 0.0472 | Penetration correction bias |
| `SLEEP_THRESHOLD` | φ⁻⁵ | 0.0902 | Body sleep velocity gate |
| `RESTITUTION` | φ⁻¹ | 0.618 | Default bounciness |
| `FRICTION` | 1 − φ⁻¹ | 0.382 | Default surface friction |
| `MAX_VELOCITY` | φ⁵ | 11.09 | Tunnelling prevention cap |
| `SUBSTEPS` | Fib(5) | 5 | Simulation sub-divisions |
| `ITERATIONS` | Fib(6) | 8 | Solver iteration count |

## Quick Start

### Rust

```rust
use aether_core::prelude::*;

let mut world = World::new();

// Static ground plane
let _ground = world.create_static(Shape::Plane {
    normal: Vec3::UP,
    offset: 0.0,
});

// Dynamic sphere
let ball = world.create_body(Shape::Sphere { radius: 0.5 }, 1.0);
world.set_position(ball, Vec3::new(0.0, 10.0, 0.0));

// Simulate at 60 fps
for _ in 0..360 {
    world.step(1.0 / 60.0);
    let pos = world.body(ball).unwrap().state.position;
    println!("y = {:.3}", pos.y);
}
```

### TypeScript

```typescript
import { AetherWorld, ThreeAdapter } from '@aether-net/core';

const world = await AetherWorld.create({
  gravity: { x: 0, y: -9.81, z: 0 },
});

const ball = world.createBody(
  { type: 'sphere', radius: 0.5 },
  { mass: 1, position: { x: 0, y: 10, z: 0 } }
);

const adapter = new ThreeAdapter(world);
adapter.bind(ball, ballMesh);

function animate() {
  world.step(1 / 60);
  adapter.sync();  // Bulk Float32Array transfer — zero copy to GPU
  renderer.render(scene, camera);
  requestAnimationFrame(animate);
}
animate();
```

## Build & Test

```bash
# Run the full test suite
cargo test --workspace

# Run benchmarks
cargo bench -p aether-core

# Build WebAssembly module
wasm-pack build crates/aether-wasm --target web --out-dir ../../ts/src/pkg

# Build TypeScript SDK
cd ts && npm install && npm run build
```

## Project Structure

```
aether-net/
├── Cargo.toml                     # Workspace manifest
├── crates/
│   ├── aether-core/               # Pure Rust physics engine
│   │   ├── src/
│   │   │   ├── phi.rs             # Golden ratio constants
│   │   │   ├── math.rs            # Vec3, Quat, Mat3, AABB
│   │   │   ├── collider.rs        # Shape geometry & inertia tensors
│   │   │   ├── body.rs            # Rigid body dynamics
│   │   │   ├── broadphase.rs      # Spatial hash grid
│   │   │   ├── narrowphase.rs     # SAT contact generation
│   │   │   ├── solver.rs          # Sequential impulse solver
│   │   │   └── world.rs           # Simulation orchestrator
│   │   └── benches/
│   │       └── physics_bench.rs   # Criterion benchmarks
│   └── aether-wasm/               # wasm-bindgen bindings
│       └── src/lib.rs
├── ts/                            # TypeScript SDK
│   └── src/
│       ├── types.ts               # Core type definitions
│       ├── world.ts               # AetherWorld class
│       ├── adapters/three.ts      # Three.js adapter
│       └── index.ts               # Public API surface
├── index.html                     # Product landing page
├── styles.css                     # Design system
└── main.js                        # Interactive visualisation
```

## Collision Support Matrix

|            | Sphere | Cuboid | Capsule | Plane |
|------------|--------|--------|---------|-------|
| **Sphere** | ✅      | ✅      | ✅       | ✅     |
| **Cuboid** | ✅      | ✅ SAT  | —       | —     |
| **Capsule**| ✅      | —      | —       | —     |
| **Plane**  | ✅      | —      | —       | —     |

## Test Results

```
running 21 tests

test math::tests::aabb_merge .......................... ok
test math::tests::aabb_surface_area ................... ok
test math::tests::mat3_inverse_identity ............... ok
test math::tests::quat_identity_rotates ............... ok
test math::tests::quat_slerp_endpoints ................ ok
test math::tests::vec3_cross .......................... ok
test math::tests::vec3_normalize_and_length ........... ok
test math::tests::vec3_reflect ........................ ok
test phi::tests::fibonacci_ratio_converges_to_phi ..... ok
test phi::tests::phi_hierarchy_consistent ............. ok
test phi::tests::phi_lerp_at_one_is_b ................. ok
test phi::tests::phi_reciprocal_identity .............. ok
test phi::tests::phi_sigmoid_at_phi_inv_is_half ....... ok
test phi::tests::phi_squared_identity ................. ok
test phi::tests::phi_weighted_avg_golden_split ........ ok
test phi::tests::sleep_threshold_from_phi5 ............ ok
test solver::tests::tangent_basis_diagonal_normal ..... ok
test solver::tests::tangent_basis_orthonormal ......... ok
test world::tests::body_count_tracking ................ ok
test world::tests::gravity_makes_body_fall ............ ok
test world::tests::sphere_rests_on_ground_plane ....... ok

test result: ok. 21 passed; 0 failed; 0 ignored
```

## License

Dual-licensed under MIT and Apache 2.0. See [LICENSE](LICENSE) for details.

---

<div align="center">

*Built with Rust · Compiled to WebAssembly · Governed by φ*

**φ² = φ + 1**

</div>
