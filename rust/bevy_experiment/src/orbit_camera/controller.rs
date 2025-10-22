use bevy::{
    ecs::prelude::*,
    math::{prelude::*, DVec2, DVec3},
    prelude::Camera,
    time::Time,
    transform::components::{GlobalTransform, Transform},
};

use crate::orbit_camera::{events::OrbitCameraInputEvent, state::OrbitCameraState};

use super::config::OrbitCameraConfig;

const POSITION_EPSILON: f32 = 1e-4;

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

fn update_zoom(config: &OrbitCameraConfig, state: &mut OrbitCameraState, zoom_delta: f32, dt: f64) {
    initialize_zoom_state(state);

    if zoom_delta != 0.0 {
        state.zoom_level_target -= zoom_delta as f64;
    }

    let min_zoom_level = distance_to_zoom_level(config.max_distance as f64);
    let max_zoom_level = distance_to_zoom_level(config.min_distance as f64);
    state.zoom_level_target = state
        .zoom_level_target
        .clamp(min_zoom_level, max_zoom_level);

    let smoothing = (config.zoom_smoothing as f64 * dt).min(1.0);
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
    dt: f64,
) {
    if let Some(delta) = orbit_delta {
        state.euler_angles_target_delta.x -= delta.y as f64;
        state.euler_angles_target_delta.y += delta.x as f64;
    }

    let smoothing = config.orbit_smoothing as f64 * dt;
    let euler_step = if smoothing > 0.0 {
        let delta = state.euler_angles_target_delta * smoothing;
        state.euler_angles_target_delta -= delta;
        delta
    } else {
        let delta = state.euler_angles_target_delta;
        state.euler_angles_target_delta = DVec3::ZERO;
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

    let min_theta = config.min_theta as f64;
    let max_theta = config.max_theta as f64;
    state.current_euler_angles.x = state.current_euler_angles.x.clamp(min_theta, max_theta);

    state.current_euler_angles.y = state.current_euler_angles.y.rem_euclid(360.0);
    state.current_euler_angles.z = 0.0;
}

fn cursor_to_world_on_plane(
    cursor: DVec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    plane_height: f64,
) -> Option<DVec3> {
    let viewport_pos = Vec2::new(cursor.x as f32, cursor.y as f32);
    let ray = camera
        .viewport_to_world(camera_transform, viewport_pos)
        .ok()?;
    let plane_origin = Vec3::new(0.0, plane_height as f32, 0.0);
    let plane = InfinitePlane3d::new(Vec3::Y);

    let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
        return None;
    };

    let intersection = ray.get_point(distance);
    Some(DVec3::new(
        intersection.x as f64,
        plane_height,
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
        if let Some(start) = input.pan_start {
            if let Some(world_point) = cursor_to_world_on_plane(
                DVec2::new(start.x as f64, start.y as f64),
                camera,
                camera_transform,
                0.0,
            ) {
                state.is_panning = true;
                state.pan_cursor_position = DVec2::new(start.x as f64, start.y as f64);
                state.drag_start_point = world_point;
                state.pan_offset_start = state.pan_offset_world_space;
                state.pan_offset_target = state.pan_offset_world_space;
            } else {
                state.is_panning = false;
            }
        } else if state.is_panning {
            state.pan_cursor_position += DVec2::new(pan_delta.x as f64, pan_delta.y as f64);
        }

        if state.is_panning {
            if let Some(world_point) =
                cursor_to_world_on_plane(state.pan_cursor_position, camera, camera_transform, 0.0)
            {
                let pan_scale = config.pan_sensitivity as f64;
                let desired_offset =
                    state.pan_offset_start + (state.drag_start_point - world_point) * pan_scale;
                state.pan_offset_target = desired_offset;
            }
        }
    } else {
        state.is_panning = false;
        state.pan_offset_start = state.pan_offset_world_space;
    }
}

fn smooth_pan(state: &mut OrbitCameraState, config: &OrbitCameraConfig, dt: f64) {
    let smoothing = (config.pan_smoothing as f64 * dt).min(1.0);
    if smoothing > 0.0 {
        state.pan_offset_world_space +=
            (state.pan_offset_target - state.pan_offset_world_space) * smoothing;
    } else {
        state.pan_offset_world_space = state.pan_offset_target;
    }
}

fn update_position(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    camera_transform: &mut Transform,
    dt: f64,
) -> bool {
    smooth_pan(state, config, dt);

    let center = state.center_target + state.pan_offset_world_space;
    let radius = state.radius.max(f64::EPSILON);

    let yaw_rad = state.current_euler_angles.y.to_radians();
    let pitch_rad = state.current_euler_angles.x.to_radians();

    let cos_pitch = pitch_rad.cos();
    let sin_pitch = pitch_rad.sin();
    let sin_yaw = yaw_rad.sin();
    let cos_yaw = yaw_rad.cos();

    let offset = DVec3::new(
        radius * cos_pitch * sin_yaw,
        radius * sin_pitch,
        radius * cos_pitch * cos_yaw,
    );

    let new_translation = (center + offset).as_vec3();
    let position_changed =
        (new_translation - camera_transform.translation).length_squared() > POSITION_EPSILON;

    camera_transform.translation = new_translation;

    let look_target = center.as_vec3();
    if (new_translation - look_target).length_squared() > f32::EPSILON {
        camera_transform.look_at(look_target, Vec3::Y);
    }
    position_changed
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

    let frame_dt = time.delta_secs_f64().min(0.02);

    for (config, mut state, camera, camera_transform, mut transform) in &mut cameras {
        update_pan_targets(config, &mut state, &input, camera, camera_transform);
        update_zoom(config, &mut state, input.zoom_delta, frame_dt);
        update_orbit(config, &mut state, input.orbit_delta, frame_dt);
        let _ = update_position(config, &mut state, &mut transform, frame_dt);
    }
}
