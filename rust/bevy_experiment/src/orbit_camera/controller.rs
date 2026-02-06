use bevy::{
    color::Color,
    ecs::prelude::*,
    gizmos::prelude::Gizmos,
    math::{
        bounding::{BoundingSphere, RayCast3d},
        prelude::*,
        DQuat, DVec3,
    },
    prelude::Camera,
    time::Time,
    transform::components::{GlobalTransform, Transform},
};

use crate::orbit_camera::{
    events::OrbitCameraInputEvent,
    plugin::CameraPivot,
    state::{OrbitCameraState, PanState},
};

use super::config::OrbitCameraConfig;

fn distance_to_zoom_level(distance: f64) -> f64 {
    -distance.ln()
}

fn zoom_level_to_distance(zoom_level: f64) -> f64 {
    (-zoom_level).exp()
}

fn initialize_zoom_state(state: &mut OrbitCameraState) {
    if state.current_zoom_level == 0.0 && state.zoom_level_target == 0.0 {
        let radius = state.radius.max(f64::EPSILON);
        let zoom_level = distance_to_zoom_level(radius);
        state.current_zoom_level = zoom_level;
        state.zoom_level_target = zoom_level;
    }
}

fn update_zoom(config: &OrbitCameraConfig, state: &mut OrbitCameraState, zoom_delta: f64, dt: f32) {
    initialize_zoom_state(state);

    if zoom_delta != 0.0 {
        state.zoom_level_target -= zoom_delta;
    }

    let min_zoom_level = distance_to_zoom_level(config.max_distance);
    let max_zoom_level = distance_to_zoom_level(config.min_distance);
    state.zoom_level_target = state
        .zoom_level_target
        .clamp(min_zoom_level, max_zoom_level);

    let smoothing = (config.zoom_smoothing * dt as f64).min(1.0);
    if smoothing > 0.0 {
        state.current_zoom_level +=
            (state.zoom_level_target - state.current_zoom_level) * smoothing;
    } else {
        state.current_zoom_level = state.zoom_level_target;
    }

    state.radius = zoom_level_to_distance(state.current_zoom_level);
}

fn update_orbit(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    orbit_delta: Option<Vec2>,
    dt: f32,
) {
    if let Some(delta) = orbit_delta {
        state.euler_angles_target_delta.x -= delta.y;
        state.euler_angles_target_delta.y += delta.x;
    }

    let smoothing = config.orbit_smoothing * dt;
    let euler_step = if smoothing > 0.0 {
        let delta = state.euler_angles_target_delta * smoothing;
        state.euler_angles_target_delta -= delta;
        delta
    } else {
        let delta = state.euler_angles_target_delta;
        state.euler_angles_target_delta = Vec3::ZERO;
        delta
    };

    state.current_euler_angles += euler_step;

    if state.current_euler_angles.x > 180.0 {
        state.current_euler_angles.x -= 360.0;
    } else if state.current_euler_angles.x < -180.0 {
        state.current_euler_angles.x += 360.0;
    } else if state.current_euler_angles.x > 90.0 {
        state.current_euler_angles.x = 0.0;
    }

    state.current_euler_angles.x = state
        .current_euler_angles
        .x
        .clamp(config.min_theta, config.max_theta);

    state.current_euler_angles.y = state.current_euler_angles.y.rem_euclid(360.0);
    state.current_euler_angles.z = 0.0;
}

fn cursor_to_world_on_sphere(
    cursor: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    sphere_radius: f32,
) -> Option<DVec3> {
    let viewport_pos = Vec2::new(cursor.x, cursor.y);
    let ray = camera
        .viewport_to_world(camera_transform, viewport_pos)
        .ok()?;
    let ray_cast = RayCast3d::from_ray(ray, f32::MAX);
    let sphere = BoundingSphere::new(Vec3::ZERO, sphere_radius);

    let distance = ray_cast.sphere_intersection_at(&sphere)?;

    let intersection = ray.get_point(distance);
    Some(DVec3::new(
        intersection.x as f64,
        intersection.y as f64,
        intersection.z as f64,
    ))
}

fn update_pan_targets(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    input: &OrbitCameraInputEvent,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    if let Some(pan_delta) = input.pan_delta {
        if let Some(pan_start_screen_space) = input.pan_start_screen_space {
            if let Some(pan_start_world_space) = cursor_to_world_on_sphere(
                pan_start_screen_space,
                camera,
                camera_transform,
                config.earth_radius,
            ) {
                let start_world_space = DVec3::from(pan_start_world_space);
                state.pan = Some(PanState {
                    start_screen_space: pan_start_screen_space,
                    offset_screen_space: Vec2::ZERO,
                    start_world_space,
                    start_radius: start_world_space.length(),
                    current_world_space: start_world_space,
                });
            } else {
                state.pan = None;
            }
        } else if let Some(pan_state) = state.pan.as_mut() {
            // Already panning, so just update the screen-space offset with the latest delta.
            pan_state.offset_screen_space += Vec2::new(pan_delta.x, pan_delta.y);
        }

        if let Some(pan_state) = state.pan.as_mut() {
            if let Some(mouse_pos_world_space) = cursor_to_world_on_sphere(
                pan_state.start_screen_space + pan_state.offset_screen_space,
                camera,
                camera_transform,
                pan_state.start_radius as f32,
            ) {
                pan_state.current_world_space = mouse_pos_world_space;
            }

            state.pan_rotation_target = DQuat::from_rotation_arc(
                pan_state.current_world_space.normalize(),
                pan_state.start_world_space.normalize(),
            );
        }
    } else {
        state.pan = None;
    }
}

/// Apply pan rotation to the pivot's f64 state and sync to the pivot's Transform.
fn update_pivot(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    pivot_transform: &mut Transform,
    gizmos: &mut Gizmos,
    dt: f32,
) {
    gizmos.sphere(Vec3::ZERO, config.earth_radius as f32, Color::WHITE);

    // Debug gizmos for pan positions
    if let Some(pan_state) = &state.pan {
        gizmos.sphere(
            Isometry3d::from_translation(pan_state.start_world_space.as_vec3()),
            0.05,
            Color::srgb(1.0, 0.0, 0.0),
        );
        gizmos.sphere(
            Isometry3d::from_translation(pan_state.current_world_space.as_vec3()),
            0.05,
            Color::srgb(0.0, 1.0, 0.0),
        );
    }

    let smoothing = (config.pan_smoothing * dt as f64).min(1.0);

    if state.pan.is_some() {
        let delta_rotation =
            DQuat::slerp(DQuat::IDENTITY, state.pan_rotation_target, smoothing);
        state.camera_center_rotation = delta_rotation * state.camera_center_rotation;
    }

    // Derive world-space center point from rotation
    state.camera_center_world_space =
        state.camera_center_rotation * DVec3::new(0.0, 0.0, config.earth_radius as f64);

    // Update pivot transform from f64 state
    pivot_transform.rotation = state.camera_center_rotation.as_quat();
}

/// Position the camera in the pivot's local space using orbit euler angles and radius.
fn update_camera_local(state: &OrbitCameraState, camera_transform: &mut Transform) {
    let radius = state.radius.max(f64::EPSILON) as f32;
    let pitch_rad = state.current_euler_angles.x.to_radians();
    let yaw_rad = state.current_euler_angles.y.to_radians();

    let orbit_rotation = Quat::from_euler(EulerRot::YXZ, yaw_rad, pitch_rad, 0.0);
    camera_transform.translation = orbit_rotation * Vec3::new(0.0, 0.0, radius);
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);
}

pub fn step(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut input_reader: MessageReader<OrbitCameraInputEvent>,
    mut pivots: Query<
        (Entity, &OrbitCameraConfig, &mut OrbitCameraState, &Children),
        With<CameraPivot>,
    >,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    mut transforms: Query<&mut Transform>,
) {
    let Some(input) = input_reader.read().next() else {
        return;
    };

    let frame_dt = time.delta_secs().min(0.02);

    for (pivot_entity, config, mut state, children) in &mut pivots {
        // Find the camera child entity
        let Some(camera_entity) = children.iter().find(|e| cameras.contains(*e)) else {
            continue;
        };
        let (_, camera, camera_global_transform) = cameras.get(camera_entity).unwrap();

        update_pan_targets(config, &mut state, &input, camera, camera_global_transform);
        update_zoom(config, &mut state, input.zoom_delta, frame_dt);
        update_orbit(config, &mut state, input.orbit_delta, frame_dt);

        // Update pivot transform (pan applies here)
        {
            let mut pivot_transform = transforms.get_mut(pivot_entity).unwrap();
            update_pivot(config, &mut state, &mut pivot_transform, &mut gizmos, frame_dt);
        }

        // Update camera local transform (orbit + zoom apply here)
        {
            let mut camera_transform = transforms.get_mut(camera_entity).unwrap();
            update_camera_local(&state, &mut camera_transform);
        }
    }
}
