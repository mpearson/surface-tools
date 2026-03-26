use bevy::prelude::*;

mod basic_scene;
mod common;
mod orbit_camera;

use bevy::math::{DQuat, DVec3};
use rand::distr::StandardUniform;
use rand::prelude::*;
use std::f64::consts::{PI, TAU};

const EARTH_RADIUS: f64 = 6378137.0;

/// Compute the rotation that takes `from` to `to`, using unnormalized vectors
/// to preserve precision for near-coincident points at large distances from the origin.
fn great_circle_rotation(from: DVec3, to: DVec3) -> DQuat {
    let cross = from.cross(to);
    let cross_len = cross.length();
    let dot = from.dot(to);

    // Use cross product magnitude for the degenerate-case check instead of
    // dot / len_product, because the cross product retains the small-angle
    // information that the dot product loses at earth scale.
    // cross_len = |from||to|sin(θ), which is well-conditioned even for tiny θ
    // when |from| and |to| are large.
    let len_product = from.length() * to.length();

    if len_product < f64::EPSILON {
        return DQuat::IDENTITY;
    }

    if cross_len < len_product * f64::EPSILON {
        if dot >= 0.0 {
            // from ≈ to
            return DQuat::IDENTITY;
        } else {
            // from ≈ -to
            return DQuat::from_axis_angle(from.normalize().any_orthonormal_vector(), PI);
        }
    }

    let axis = cross / cross_len;
    // atan2 avoids the precision-losing division by len_product that asin would need.
    // atan2(|from||to|sin(θ), |from||to|cos(θ)) = θ, with the scale factors cancelling.
    let angle = f64::atan2(cross_len, dot);
    DQuat::from_axis_angle(axis, angle)
}

fn test_quat_precision() {
    let mut rng = rand::rng();

    let offset = 0.001;
    let mut min_error_bevy: f64 = f64::MAX;
    let mut max_error_bevy: f64 = 0.0;
    let mut min_error_ours: f64 = f64::MAX;
    let mut max_error_ours: f64 = 0.0;

    for _ in 0..5000 {
        let r0 = DQuat::from_euler(
            EulerRot::YXZ,
            rng.sample::<f64, _>(StandardUniform) * TAU,
            rng.sample::<f64, _>(StandardUniform) * TAU - PI,
            0.0,
        );

        // Rotate two points into a random position on the sphere. One point is offset from the
        // other by a known distance.
        let p1 = r0 * DVec3::new(0.0, 0.0, -EARTH_RADIUS);
        let p2 = r0 * DVec3::new(0.0, offset, -EARTH_RADIUS);

        // Bevy implementation
        // Compute the rotation between the two points:
        let q_bevy = DQuat::from_rotation_arc(p1.normalize(), p2.normalize());
        // Apply that rotation to the first point and see how close it is to the second point.
        let error_bevy = ((q_bevy * p1) - p2).length();
        min_error_bevy = min_error_bevy.min(error_bevy);
        max_error_bevy = max_error_bevy.max(error_bevy);

        // Our implementation
        // Compute the rotation between the two points:
        let q_ours = great_circle_rotation(p1, p2);
        // Apply that rotation to the first point and see how close it is to the second point.
        let error_ours = ((q_ours * p1) - p2).length();
        min_error_ours = min_error_ours.min(error_ours);
        max_error_ours = max_error_ours.max(error_ours);
    }

    println!("offset = {offset}m:");
    println!("  from_rotation_arc:      best {min_error_bevy:.6e}  worst {max_error_bevy:.6e}");
    println!("  rotation_between_points: best {min_error_ours:.6e}  worst {max_error_ours:.6e}");
    println!();
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
