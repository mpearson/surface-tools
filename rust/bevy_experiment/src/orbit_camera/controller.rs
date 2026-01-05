use bevy::{
    ecs::prelude::*,
    math::{prelude::*, DVec2, DVec3},
    prelude::Camera,
    time::Time,
    transform::components::{GlobalTransform, Transform},
};

use crate::orbit_camera::{
    events::OrbitCameraInputEvent,
    state::{OrbitCameraState, PanState},
};

use super::config::OrbitCameraConfig;

const POSITION_EPSILON: f32 = 1e-4;

fn distance_to_zoom_level(distance: f32) -> f32 {
    -distance.ln()
}

fn zoom_level_to_distance(zoom_level: f32) -> f32 {
    (-zoom_level).exp()
}

fn initialize_zoom_state(state: &mut OrbitCameraState) {
    if state.current_zoom_level == 0.0 && state.zoom_level_target == 0.0 {
        let radius = state.radius.max(f32::EPSILON);
        let zoom_level = distance_to_zoom_level(radius);
        state.current_zoom_level = zoom_level;
        state.zoom_level_target = zoom_level;
    }
}

fn update_zoom(config: &OrbitCameraConfig, state: &mut OrbitCameraState, zoom_delta: f32, dt: f32) {
    initialize_zoom_state(state);

    if zoom_delta != 0.0 {
        state.zoom_level_target -= zoom_delta;
    }

    let min_zoom_level = distance_to_zoom_level(config.max_distance);
    let max_zoom_level = distance_to_zoom_level(config.min_distance);
    state.zoom_level_target = state
        .zoom_level_target
        .clamp(min_zoom_level, max_zoom_level);

    let smoothing = (config.zoom_smoothing * dt).min(1.0);
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

fn cursor_to_world_on_plane(
    cursor: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    plane_height: f32,
) -> Option<DVec3> {
    let viewport_pos = Vec2::new(cursor.x, cursor.y);
    let ray = camera
        .viewport_to_world(camera_transform, viewport_pos)
        .ok()?;
    let plane_origin = Vec3::new(0.0, plane_height, 0.0);
    let plane = InfinitePlane3d::new(Vec3::Y);

    let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
        return None;
    };

    let intersection = ray.get_point(distance);
    Some(DVec3::new(
        intersection.x as f64,
        plane_height as f64,
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
            if let Some(pan_start_world_space) =
                cursor_to_world_on_plane(pan_start_screen_space, camera, camera_transform, 0.0)
            {
                state.pan = Some(PanState {
                    start_screen_space: pan_start_screen_space,
                    offset_screen_space: Vec2::ZERO,
                    start_world_space: DVec2::new(
                        pan_start_world_space.x as f64,
                        pan_start_world_space.z as f64,
                    ),
                });

                // state.is_panning = true;
                // // state.pan_cursor_position = Vec2::new(start.x, start.y);
                // state.pan_start_screen_space = pan_start_screen_space;
                // state.pan_start_world_space.x = pan_start_world_space.x as f64;
                // state.pan_start_world_space.y = pan_start_world_space.y as f64;
                // state.pan_offset_screen_space = Vec2::ZERO;
                // // state.pan_offset_start = state.pan_offset_world_space;
                // // state.pan_offset_target = state.pan_offset_world_space;
            } else {
                state.pan = None;
            }
        } else if state.pan.is_some() {
            // state.pan_cursor_position += Vec2::new(pan_delta.x, pan_delta.y);
            state.pan.as_mut().unwrap().offset_screen_space += Vec2::new(pan_delta.x, pan_delta.y);
        }

        if let Some(pan_state) = state.pan.as_mut() {
            if let Some(mouse_pos_world_space) = cursor_to_world_on_plane(
                pan_state.start_screen_space + pan_state.offset_screen_space,
                camera,
                camera_transform,
                0.0,
            ) {
                let pan_scale = config.pan_sensitivity;
                let desired_offset = (DVec2::new(mouse_pos_world_space.x, mouse_pos_world_space.y)
                    - pan_state.start_world_space)
                    * pan_scale;
                state.position_target += desired_offset;
            }
        }
    } else {
        state.pan = None;
        // state.is_panning = false;
        // state.pan_offset_start = state.pan_offset_world_space;
    }
}

fn smooth_pan(state: &mut OrbitCameraState, config: &OrbitCameraConfig, dt: f32) {
    if let Some(pan_state) = &state.pan {
        let smoothing = (config.pan_smoothing * dt as f64).min(1.0);
        if smoothing > 0.0 {
            pan_state.offset_world_space +=
                (pan_state.offset_target - pan_state.offset_world_space) * smoothing;
        } else {
            pan_state.offset_world_space = pan_state.offset_target;
        }
    }
}

fn update_position(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    camera_transform: &mut Transform,
    dt: f32,
) {
    smooth_pan(state, config, dt);

    let center = state.position_target + state.pan_offset_world_space;
    let radius = state.radius.max(f32::EPSILON);

    let yaw_rad = state.current_euler_angles.y.to_radians();
    let pitch_rad = state.current_euler_angles.x.to_radians();

    let cos_pitch = pitch_rad.cos();
    let sin_pitch = pitch_rad.sin();
    let sin_yaw = yaw_rad.sin();
    let cos_yaw = yaw_rad.cos();

    let offset = Vec3::new(
        radius * cos_pitch * sin_yaw,
        radius * sin_pitch,
        radius * cos_pitch * cos_yaw,
    );

    let new_translation = center + offset;
    let position_changed =
        (new_translation - camera_transform.translation).length_squared() > POSITION_EPSILON;

    camera_transform.translation = new_translation;

    let look_target = center;
    if (new_translation - look_target).length_squared() > f32::EPSILON {
        camera_transform.look_at(look_target, Vec3::Y);
    }
}

pub fn step(
    time: Res<Time>,
    mut input_reader: MessageReader<OrbitCameraInputEvent>,
    mut cameras: Query<(
        &OrbitCameraConfig,
        &mut OrbitCameraState,
        &Camera,
        &GlobalTransform,
        &mut Transform,
    )>,
) {
    let Some(input) = input_reader.read().next() else {
        return;
    };

    let frame_dt = time.delta_secs().min(0.02);

    for (config, mut state, camera, camera_transform, mut transform) in &mut cameras {
        update_pan_targets(config, &mut state, &input, camera, camera_transform);
        update_zoom(config, &mut state, input.zoom_delta, frame_dt);
        update_orbit(config, &mut state, input.orbit_delta, frame_dt);
        update_position(config, &mut state, &mut transform, frame_dt);
    }
}
