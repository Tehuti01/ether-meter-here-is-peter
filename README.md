<div align="center">

# Aether-Net

### **AI-Driven Decentralized Real-Time Physics Engine**

*φ-governed rigid-body dynamics for the browser*

[![CI](https://github.com/Tehuti01/ether-meter-here-is-peter/actions/workflows/ci.yml/badge.svg)](https://github.com/Tehuti01/ether-meter-here-is-peter/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-gold.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-teal.svg)](https://www.rust-lang.org)
[![Wasm](https://img.shields.io/badge/WebAssembly-Ready-purple.svg)](https://webassembly.org)

---

**φ = 1.618033988749895**

*The golden ratio governs every constant in this engine.*
*From damping coefficients to spatial partitioning, from solver convergence to time-step harmonics — φ is the law.*

</div>

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        TypeScript SDK                              │
│   AetherWorld · ShapeDescriptor · ThreeAdapter · BodyState         │
├─────────────────────────────────────────────────────────────────────┤
│                     wasm-bindgen Bridge                            │
│   AetherWorld (JS) ←→ Float64Array bulk transfer ←→ Rust World    │
├─────────────────────────────────────────────────────────────────────┤
│                     Rust Physics Core                              │
│                                                                    │
│   ┌──────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
│   │   phi.rs  │  │   math.rs    │  │  collider.rs  │  │  body.rs │ │
│   │   φ law   │  │  Vec3/Quat   │  │  Sphere/Box   │  │ Dynamics │ │
│   │  1.618... │  │  Mat3/AABB   │  │  Capsule/Plane│  │ Forces   │ │
│   └──────────┘  └──────────────┘  └──────────────┘  └──────────┘ │
│                                                                    │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐│
│   │ broadphase.rs│  │narrowphase.rs│  │       solver.rs          ││
│   │ φ-scaled     │  │ SAT/GJK      │  │  Sequential Impulse      ││
│   │ Spatial Grid │  │ Contact Gen  │  │  Baumgarte Stabilization ││
│   └──────────────┘  └──────────────┘  └──────────────────────────┘│
│                                                                    │
│   ┌────────────────────────────────────────────────────────────┐   │
│   │                      world.rs                              │   │
│   │  φ-sub-stepping · Plane bypass · Sleep detection · Events  │   │
│   └────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## The φ Law

Every fundamental constant derives from the golden ratio:

| Constant | Formula | Value | Purpose |
|---|---|---|---|
| `LINEAR_DAMPING` | `1 - 1/φ⁴` | `0.854` | Velocity decay per second |
| `ANGULAR_DAMPING` | `1 - 1/φ³` | `0.764` | Rotational decay |
| `SOLVER_EPSILON` | `1/φ⁸` | `0.0557` | Convergence threshold |
| `BAUMGARTE_FACTOR` | `0.2 × φ⁻¹` | `0.1236` | Penetration correction bias |
| `SLEEP_THRESHOLD` | `φ⁻⁵` | `0.0902` | Body sleep velocity gate |
| `RESTITUTION` | `φ⁻¹` | `0.618` | Default bounciness |
| `FRICTION` | `1 - φ⁻¹` | `0.382` | Default surface friction |
| `MAX_VELOCITY` | `φ⁵` | `11.09` | Tunneling prevention cap |
| `SUBSTEPS` | `Fib(5)` | `5` | Simulation sub-divisions |
| `ITERATIONS` | `Fib(6)` | `8` | Solver iteration count |

## Quick Start

### Rust

```rust
use aether_core::prelude::*;

let mut world = World::new();

// Ground plane
let ground = world.create_static(Shape::Plane {
    normal: Vec3::UP,
    offset: 0.0,
});

// Falling sphere
let ball = world.create_body(Shape::Sphere { radius: 0.5 }, 1.0);
world.set_position(ball, Vec3::new(0.0, 10.0, 0.0));

// Simulate at 60fps
loop {
    world.step(1.0 / 60.0);
    let pos = world.body(ball).unwrap().state.position;
    println!("Ball: ({:.2}, {:.2}, {:.2})", pos.x, pos.y, pos.z);
}
```

### TypeScript (with Three.js)

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
  adapter.sync(); // Bulk Float64Array transfer
  renderer.render(scene, camera);
  requestAnimationFrame(animate);
}
```

## Building

```bash
# Test
cargo test --workspace

# Benchmark
cargo bench -p aether-core

# Build Wasm
cargo build -p aether-wasm --target wasm32-unknown-unknown --release

# TypeScript SDK
cd ts && npm install && npm run build
```

## Project Structure

```
aether-net/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── aether-core/              # Pure Rust physics engine
│   │   ├── src/
│   │   │   ├── phi.rs            # φ constants & utilities
│   │   │   ├── math.rs           # Vec3, Quat, Mat3, AABB
│   │   │   ├── collider.rs       # Shape definitions & inertia
│   │   │   ├── body.rs           # Rigid body dynamics
│   │   │   ├── broadphase.rs     # Spatial hash grid
│   │   │   ├── narrowphase.rs    # SAT, GJK, contact generation
│   │   │   ├── solver.rs         # Sequential impulse solver
│   │   │   └── world.rs          # Simulation orchestrator
│   │   └── benches/
│   │       └── physics_bench.rs  # Criterion benchmarks
│   └── aether-wasm/              # wasm-bindgen bindings
│       └── src/lib.rs
├── ts/                           # TypeScript SDK
│   └── src/
│       ├── types.ts              # Core type definitions
│       ├── world.ts              # AetherWorld class
│       ├── adapters/three.ts     # Three.js sync adapter
│       └── index.ts              # Public API
├── .github/workflows/ci.yml     # CI pipeline
├── index.html                    # Product landing page
├── styles.css
└── main.js
```

## Collision Matrix

| | Sphere | Cuboid | Capsule | Plane |
|---|---|---|---|---|
| **Sphere** | ✅ | ✅ | ✅ | ✅ |
| **Cuboid** | ✅ | ✅ SAT | — | — |
| **Capsule** | ✅ | — | — | — |
| **Plane** | ✅ | — | — | — |

## Test Results

```
running 11 tests
test math::tests::vec3_cross_product .......... ok
test math::tests::quat_rotate_identity ........ ok
test math::tests::mat3_inverse_identity ....... ok
test math::tests::aabb_intersection ........... ok
test phi::tests::phi_identity ................. ok
test phi::tests::phi_reciprocal ............... ok
test phi::tests::fibonacci_ratio_converges .... ok
test phi::tests::fib_scale_approaches_phi ..... ok
test world::tests::gravity_makes_body_fall .... ok
test world::tests::sphere_rests_on_ground ..... ok
test world::tests::body_count_tracking ........ ok

test result: ok. 11 passed; 0 failed; 0 ignored
```

## License

MIT OR Apache-2.0

---

<div align="center">

*Built with Rust · Compiled to WebAssembly · Governed by φ*

**φ² = φ + 1**

</div>
