// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::prelude::*,
    prelude::Camera3dBundle,
    prelude::ReflectDefault,
    reflect::Reflect,
    time::Time,
    transform::components::Transform,
};

use crate::orbit_camera::{events::*, state::*};

pub fn control_system(
    time: Res<Time>,
    mut events: EventReader<OrbitCameraInput>,
    mut cameras: Query<(&OrbitCameraController, &mut LookTransform, &Transform)>,
) {
    // Can only control one camera at a time.
    let (mut transform, scene_transform) =
        if let Some((_, transform, scene_transform)) = cameras.iter_mut().find(|c| c.0.enabled) {
            (transform, scene_transform)
        } else {
            return;
        };

    let mut look_angles = LookAngles::from_vector(-transform.look_direction().unwrap());
    let mut radius_scalar = 1.0;
    let radius = transform.radius();

    let dt = time.delta_seconds();
    for event in events.read() {
        match event {
            OrbitCameraInput::Orbit(delta) => {
                look_angles.add_yaw(dt * -delta.x);
                look_angles.add_pitch(dt * delta.y);
            }
            OrbitCameraInput::TranslateTarget(delta) => {
                let right_dir = scene_transform.rotation * -Vec3::X;
                let up_dir = scene_transform.rotation * Vec3::Y;
                transform.target += dt * delta.x * right_dir + dt * delta.y * up_dir;
            }
            OrbitCameraInput::Zoom(scalar) => {
                radius_scalar *= scalar;
            }
        }
    }

    look_angles.assert_not_looking_up();

    let new_radius = (radius_scalar * radius).min(1000000.0).max(0.001);
    transform.eye = transform.target + new_radius * look_angles.unit_vector();
}
