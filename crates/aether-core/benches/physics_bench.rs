use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use aether_core::prelude::*;

fn bench_gravity_freefall(c: &mut Criterion) {
    let mut group = c.benchmark_group("gravity_freefall");

    for body_count in [10, 100, 500, 1000, 5000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(body_count),
            &body_count,
            |b, &n| {
                b.iter(|| {
                    let mut world = World::new();
                    for i in 0..n {
                        let id = world.create_body(
                            Shape::Sphere { radius: 0.5 },
                            1.0,
                        );
                        world.set_position(id, Vec3::new(
                            (i % 50) as f64 * 2.0,
                            10.0 + (i / 50) as f64 * 2.0,
                            0.0,
                        ));
                    }
                    // Simulate 10 frames at 60fps
                    for _ in 0..10 {
                        world.step(black_box(1.0 / 60.0));
                    }
                    world.active_body_count()
                });
            },
        );
    }
    group.finish();
}

fn bench_sphere_collisions(c: &mut Criterion) {
    let mut group = c.benchmark_group("sphere_collisions");

    for body_count in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(body_count),
            &body_count,
            |b, &n| {
                b.iter(|| {
                    let mut world = World::new();
                    let _ground = world.create_static(Shape::Plane {
                        normal: Vec3::UP,
                        offset: 0.0,
                    });
                    for i in 0..n {
                        let id = world.create_body(
                            Shape::Sphere { radius: 0.5 },
                            1.0,
                        );
                        world.set_position(id, Vec3::new(
                            (i % 10) as f64 * 1.5,
                            2.0 + (i / 10) as f64 * 1.5,
                            0.0,
                        ));
                    }
                    for _ in 0..10 {
                        world.step(black_box(1.0 / 60.0));
                    }
                    world.active_body_count()
                });
            },
        );
    }
    group.finish();
}

fn bench_step_single(c: &mut Criterion) {
    // Measure raw step() cost with 100 bodies already in steady state
    let mut world = World::new();
    let _ground = world.create_static(Shape::Plane {
        normal: Vec3::UP,
        offset: 0.0,
    });
    for i in 0..100 {
        let id = world.create_body(Shape::Sphere { radius: 0.5 }, 1.0);
        world.set_position(id, Vec3::new(
            (i % 10) as f64 * 2.0,
            5.0 + (i / 10) as f64 * 2.0,
            0.0,
        ));
    }
    // Warm up
    for _ in 0..60 { world.step(1.0 / 60.0); }

    c.bench_function("step_100_bodies_60fps", |b| {
        b.iter(|| {
            world.step(black_box(1.0 / 60.0));
        });
    });
}

fn bench_phi_math(c: &mut Criterion) {
    c.bench_function("phi_pow_10", |b| {
        b.iter(|| {
            black_box(phi::phi_pow(black_box(10)))
        });
    });

    c.bench_function("vec3_cross_product", |b| {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let bb = Vec3::new(4.0, 5.0, 6.0);
        b.iter(|| {
            black_box(black_box(a).cross(black_box(bb)))
        });
    });

    c.bench_function("quat_rotate_vec", |b| {
        let q = Quat::from_axis_angle(Vec3::UP, 0.5);
        let v = Vec3::new(1.0, 2.0, 3.0);
        b.iter(|| {
            black_box(black_box(q).rotate_vec(black_box(v)))
        });
    });
}

criterion_group!(benches, bench_gravity_freefall, bench_sphere_collisions, bench_step_single, bench_phi_math);
criterion_main!(benches);
