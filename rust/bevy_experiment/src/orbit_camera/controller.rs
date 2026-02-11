use bevy::{
    color::Color,
    ecs::prelude::*,
    gizmos::prelude::Gizmos,
    math::{
        bounding::{BoundingSphere, RayCast3d},
        prelude::*,
        DQuat, DVec3,
    },
    prelude::info,
    prelude::Camera,
    time::Time,
    transform::components::{GlobalTransform, Transform},
};

use crate::orbit_camera::{
    events::OrbitCameraInputEvent,
    plugin::{OrbitCameraChildRef, OrbitCameraRig},
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
        state.euler_angles_target_delta.x += delta.y;
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
        .clamp(config.min_pitch, config.max_pitch);

    state.current_euler_angles.y = state.current_euler_angles.y.rem_euclid(360.0);
    state.current_euler_angles.z = 0.0;
}

fn cursor_to_world_on_sphere(
    cursor: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    sphere_radius: f32,
) -> Option<Vec3> {
    let viewport_pos = Vec2::new(cursor.x, cursor.y);
    let ray = camera
        .viewport_to_world(camera_transform, viewport_pos)
        .ok()?;
    let ray_cast = RayCast3d::from_ray(ray, f32::MAX);
    let sphere = BoundingSphere::new(Vec3::ZERO, sphere_radius);

    let distance = ray_cast.sphere_intersection_at(&sphere)?;

    let intersection = ray.get_point(distance);
    Some(intersection)
}

fn update_position_target(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    input: &OrbitCameraInputEvent,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    gizmos: &mut Gizmos,
) {
    if let Some(pan_delta) = input.pan_delta {
        if let Some(pan_start_screen_space) = input.pan_start_screen_space {
            if let Some(cursor_world_space) = cursor_to_world_on_sphere(
                pan_start_screen_space,
                camera,
                camera_transform,
                config.earth_radius,
            ) {
                let start_world_space = DVec3::from(cursor_world_space);
                state.pan = Some(PanState {
                    start_screen_space: pan_start_screen_space,
                    offset_screen_space: Vec2::ZERO,
                    start_world_space,
                    start_radius: start_world_space.length(),
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
                state.pan_rotation_target = DQuat::from_rotation_arc(
                    mouse_pos_world_space.as_dvec3().normalize(),
                    pan_state.start_world_space.normalize(),
                );
                // Debug gizmo for current mouse position on sphere
                gizmos.sphere(
                    Isometry3d::from_translation(mouse_pos_world_space),
                    0.05,
                    Color::srgb(0.0, 1.0, 0.0),
                );
            }

            // Debug gizmos for initial mouse position on sphere
            gizmos.sphere(
                Isometry3d::from_translation(pan_state.start_world_space.as_vec3()),
                0.05,
                Color::srgb(1.0, 0.0, 0.0),
            );
        }
    } else {
        state.pan = None;
    }
}

fn update_camera_rig_rotation(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    camera_rig_transform: &mut Transform,
    dt: f32,
) {
    let smoothing = (config.pan_smoothing * dt as f64).min(1.0);

    if state.pan.is_some() {
        let delta_rotation = DQuat::slerp(DQuat::IDENTITY, state.pan_rotation_target, smoothing);
        state.camera_rig_rotation = delta_rotation * state.camera_rig_rotation;
    }

    // Derive world-space center point from rotation
    state.camera_rig_position_world_space =
        state.camera_rig_rotation * DVec3::new(0.0, 0.0, config.earth_radius as f64);

    // Update camera rig transform from f64 state
    camera_rig_transform.rotation = state.camera_rig_rotation.as_quat();
    camera_rig_transform.translation = state.camera_rig_position_world_space.as_vec3();
}

/// Position the camera in the camera rig's local space using orbit euler angles and radius.
fn update_camera_rotation(state: &OrbitCameraState, camera_transform: &mut Transform) {
    let radius = state.radius.max(f64::EPSILON) as f32;
    let pitch = state.current_euler_angles.x.to_radians();
    let yaw = state.current_euler_angles.y.to_radians();

    let orbit_rotation = Quat::from_euler(EulerRot::ZXY, yaw, pitch, 0.0);
    camera_transform.translation = orbit_rotation * Vec3::new(0.0, 0.0, radius);
    camera_transform.rotation = orbit_rotation;
}

pub fn step(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut input_reader: MessageReader<OrbitCameraInputEvent>,
    mut camera_rigs: Query<
        (
            Entity,
            &OrbitCameraConfig,
            &mut OrbitCameraState,
            &OrbitCameraChildRef,
        ),
        With<OrbitCameraRig>,
    >,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    mut transforms: Query<&mut Transform>,
) {
    let Some(input) = input_reader.read().next() else {
        return;
    };

    let frame_dt = time.delta_secs().min(0.001);

    for (camera_rig, config, mut state, child_ref) in &mut camera_rigs {
        // Draw a wireframe sphere to help visualize camera movements
        gizmos.sphere(Vec3::ZERO, config.earth_radius as f32, Color::WHITE);

        // Get camera directly using the stored reference
        if let Ok((camera_entity, camera, camera_global_transform)) =
            cameras.get(child_ref.camera_entity)
        {
            update_position_target(
                config,
                &mut state,
                &input,
                camera,
                camera_global_transform,
                &mut gizmos,
            );
            update_zoom(config, &mut state, input.zoom_delta, frame_dt);
            update_orbit(config, &mut state, input.orbit_delta, frame_dt);

            {
                let mut camera_rig_transform = transforms.get_mut(camera_rig).unwrap();
                update_camera_rig_rotation(config, &mut state, &mut camera_rig_transform, frame_dt);
            }

            {
                let mut camera_transform = transforms.get_mut(camera_entity).unwrap();
                update_camera_rotation(&state, &mut camera_transform);
            }
        }
    }
}
