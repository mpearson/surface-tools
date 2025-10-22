use bevy::{
    ecs::prelude::*,
    math::{prelude::*, DVec3},
    time::Time,
    transform::components::Transform,
};

use crate::orbit_camera::{events::OrbitCameraInputEvent, state::OrbitCameraState};

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

fn update_zoom(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    zoom_delta: f32,
    dt: f64,
) {
    initialize_zoom_state(state);

    if zoom_delta != 0.0 {
        state.zoom_level_target += zoom_delta as f64;
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
    orbit_delta: Vec2,
    dt: f64,
) {
    if orbit_delta != Vec2::ZERO {
        state.euler_angles_target_delta.x -= orbit_delta.y as f64;
        state.euler_angles_target_delta.y += orbit_delta.x as f64;
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
    state.current_euler_angles.x =
        state.current_euler_angles.x.clamp(min_theta, max_theta);

    state.current_euler_angles.y = state.current_euler_angles.y.rem_euclid(360.0);
    state.current_euler_angles.z = 0.0;
}

fn update_position(state: &OrbitCameraState, camera_transform: &mut Transform) -> bool {
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
    let previous_translation = camera_transform.translation;

    camera_transform.translation = new_translation;

    let look_target = center.as_vec3();
    if (new_translation - look_target).length_squared() > f32::EPSILON {
        camera_transform.look_at(look_target, Vec3::Y);
    }

    (new_translation - previous_translation).length_squared() > f32::EPSILON
}

pub fn step(
    time: Res<Time>,
    mut input_reader: MessageReader<OrbitCameraInputEvent>,
    mut cameras: Query<(&OrbitCameraConfig, &mut OrbitCameraState, &mut Transform)>,
) {
    let Some(input) = input_reader.read().next() else {
        return;
    };

    let frame_dt = time.delta_secs_f64().min(0.02);

    for (config, mut state, mut transform) in &mut cameras {
        update_zoom(config, &mut state, input.zoom_delta, frame_dt);
        update_orbit(config, &mut state, input.orbit_delta, frame_dt);
        let _ = update_position(&state, &mut transform);
    }
}
