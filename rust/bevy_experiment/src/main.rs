use bevy::prelude::*;

mod basic_scene;
mod common;
mod orbit_camera;

use bevy::{
    // color::Color,
    // ecs::prelude::*,
    // gizmos::prelude::Gizmos,
    math::{prelude::*, DQuat, DVec3},
    // prelude::info,
    // prelude::Camera,
    // time::Time,
    // transform::components::{GlobalTransform, Transform},
};
use rand::distr::StandardUniform;
use rand::prelude::*;
use std::f64::consts::{PI, TAU};

// Get an RNG:

const EARTH_RADIUS: f64 = 6378137.0;

fn test_quat_precision() {
    let mut rng = rand::rng();

    let mut best_error: f64 = f64::MAX;
    let mut worst_error: f64 = 0.0;

    for _ in 0..5000 {
        let r0 = DQuat::from_euler(
            EulerRot::YXZ,
            rng.sample::<f64, _>(StandardUniform) * TAU,
            rng.sample::<f64, _>(StandardUniform) * TAU - PI,
            0.0,
        );
        let offset = 0.001;

        // Rotate two points into a random position on the sphere. One point is offset from the
        // other by a known distance.
        let p1 = r0 * DVec3::new(0.0, 0.0, -EARTH_RADIUS);
        let p2 = r0 * DVec3::new(0.0, offset, -EARTH_RADIUS);
        // println!("initial delta: {:?}", (p2 - p1).length());

        // Compute the rotation between the two points.
        let q = DQuat::from_rotation_arc(p1.normalize(), p2.normalize());

        // Apply that rotation to the first point and see how close it is to the second point.
        let p1_rotated = q * p1;
        let error = f64::abs((p1_rotated - p2).length());

        if error >= offset {
            println!(
                "error: {:?} for euler angles {:?}",
                error,
                DVec3::from(r0.to_euler(EulerRot::YXZ)) * (180.0 / PI)
            );
            // println!("    p1: {:?}", p1);
            // println!("    p1_rotated: {:?}", p1_rotated);
            // println!("    rotated delta: {:?}", (p1_rotated - p1).length());
        }
        best_error = best_error.min(error);
        worst_error = worst_error.max(error);
    }
    println!("best error: {:?}", best_error);
    println!("worst error: {:?}", worst_error);
}

fn main() {
    println!("wow, such bevy");
    println!("very webGPU");
    println!("much borrow check");
    println!("wow");

    test_quat_precision();
    return;

    App::new()
        .add_plugins((
            DefaultPlugins,
            basic_scene::BasicScenePlugin,
            // common::fps_plugin::FpsCounterPlugin,
            orbit_camera::plugin::OrbitCameraPlugin,
        ))
        .run();
}
